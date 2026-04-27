//! Process-wide state: terminal pool, connection ownership, tunable defaults.
//!
//! Backed by a `parking_lot::Mutex<HashMap>` rather than `dashmap`, because the most
//! frequent multi-key operation (reaping every terminal owned by a dropped connection)
//! is naturally one-lock-one-pass with `HashMap::retain`. Lock contention is a non-issue:
//! every operation here is microseconds-bounded.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use parking_lot::Mutex;
use regex::Regex;

use crate::{Error, Result};
use crate::protocol::SpawnArgs;
use crate::session::ConnectionId;
use crate::terminal::{Terminal, state::Ready};

pub struct App {
    terminals: Mutex<HashMap<u32, Owned>>,
    next_id: AtomicU32,
    pub(crate) default_visible: bool,
    pub(crate) prompt_regex: Regex,
}

struct Owned {
    owner: ConnectionId,
    terminal: Terminal<Ready>,
}

impl App {
    pub fn builder() -> AppBuilder { AppBuilder::default() }

    /// Spawn a fresh terminal under `owner`. Returns the terminal `id`.
    pub async fn spawn(&self, owner: ConnectionId, args: SpawnArgs) -> Result<u32> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let term = crate::terminal::spawn(id, args, self.default_visible, &self.prompt_regex).await?;
        self.terminals.lock().insert(id, Owned { owner, terminal: term });
        Ok(id)
    }

    /// Look up a terminal that `owner` is allowed to see. Returns
    /// `Error::UnknownTerminal` if either the id is missing *or* it belongs to
    /// another connection — we don't leak ownership across sessions.
    pub fn terminal(&self, owner: ConnectionId, id: u32) -> Result<Terminal<Ready>> {
        let map = self.terminals.lock();
        match map.get(&id) {
            Some(o) if o.owner == owner => Ok(o.terminal.clone()),
            _ => Err(Error::UnknownTerminal(id)),
        }
    }

    /// IDs of terminals owned by `owner`, in arbitrary order.
    pub fn list(&self, owner: ConnectionId) -> Vec<u32> {
        self.terminals.lock().iter()
            .filter_map(|(id, o)| (o.owner == owner).then_some(*id))
            .collect()
    }

    /// Close one terminal. Returns `Error::UnknownTerminal` if `owner` doesn't own it.
    pub fn close(&self, owner: ConnectionId, id: u32) -> Result<()> {
        let mut map = self.terminals.lock();
        match map.get(&id) {
            Some(o) if o.owner == owner => { map.remove(&id); Ok(()) }
            _ => Err(Error::UnknownTerminal(id)),
        }
    }

    /// Close every terminal owned by `owner`. Returns the number reaped.
    pub fn close_all(&self, owner: ConnectionId) -> usize {
        let mut map = self.terminals.lock();
        let before = map.len();
        map.retain(|_, o| o.owner != owner);
        before - map.len()
    }

    /// Used by [`crate::session::ConnectionGuard`] on drop. Equivalent to
    /// `close_all` but separated so the intent at the call site reads clearly.
    pub fn reap_owner(&self, owner: ConnectionId) -> usize {
        self.close_all(owner)
    }

    /// Snapshot every terminal — used by the watchdog. Returns `(id, line_count,
    /// max_lines, spawn_time, max_duration, owner)` tuples.
    pub(crate) fn snapshot_for_watchdog(&self) -> Vec<WatchdogEntry> {
        self.terminals.lock().iter().map(|(id, o)| WatchdogEntry {
            id: *id,
            owner: o.owner,
            line_count: o.terminal.line_count(),
            max_lines: o.terminal.max_lines(),
            spawn_time: o.terminal.spawn_time(),
            max_duration: o.terminal.max_duration(),
        }).collect()
    }
}

pub(crate) struct WatchdogEntry {
    pub id: u32,
    pub owner: ConnectionId,
    pub line_count: u32,
    pub max_lines: Option<u32>,
    pub spawn_time: std::time::Instant,
    pub max_duration: Option<std::time::Duration>,
}

#[derive(Default)]
pub struct AppBuilder {
    default_visible: Option<bool>,
    prompt_regex: Option<String>,
}

impl AppBuilder {
    pub fn default_visible(mut self, v: bool) -> Self { self.default_visible = Some(v); self }
    pub fn prompt_regex(mut self, r: impl Into<String>) -> Self { self.prompt_regex = Some(r.into()); self }

    pub fn build(self) -> Result<Arc<App>> {
        let app = App {
            terminals: Mutex::new(HashMap::new()),
            next_id: AtomicU32::new(1),
            default_visible: self.default_visible.unwrap_or(true),
            prompt_regex: Regex::new(
                &self.prompt_regex.unwrap_or_else(|| r"(?:PS )?[A-Z]:\\.*> ?$".to_string()),
            )?,
        };
        Ok(Arc::new(app))
    }
}
