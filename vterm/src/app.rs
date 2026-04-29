//! Process-wide state: terminal pool, connection ownership, tunable defaults.
//!
//! Backed by a `parking_lot::Mutex<HashMap>` rather than `dashmap`, because the most
//! frequent multi-key operation (reaping every terminal owned by a dropped connection)
//! is naturally one-lock-one-pass with `HashMap::retain`. Lock contention is a non-issue:
//! every operation here is microseconds-bounded.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use regex::Regex;
use tokio::sync::mpsc;

use crate::protocol::SpawnArgs;
use crate::session::ConnectionId;
use crate::terminal::{state::Ready, Terminal};
use crate::{Error, Result};

pub struct App {
    terminals: Mutex<HashMap<u32, Owned>>,
    pool: Mutex<VecDeque<Terminal<Ready>>>,
    refill_tx: mpsc::Sender<()>,
    next_id: AtomicU32,
    pub(crate) default_visible: bool,
    pub(crate) prompt_regex: Regex,
    // Admission Control
    pub(crate) max_terminals: u32,
    pub(crate) max_mem_mb: Option<u64>,
}

struct Owned {
    owner: ConnectionId,
    terminal: Terminal<Ready>,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    /// Spawn a fresh terminal under `owner`. Returns the (terminal_id, spawn_ms, ready_ms).
    pub async fn spawn(&self, owner: ConnectionId, args: SpawnArgs) -> Result<(u32, u64, u64)> {
        // 0. Admission Control
        let current_count = self.terminals.lock().len() as u32;
        if current_count >= self.max_terminals {
            return Err(Error::SystemSaturation(format!(
                "max terminals reached: {}/{}",
                current_count, self.max_terminals
            )));
        }

        if let Some(max_mb) = self.max_mem_mb {
            use sysinfo::System;
            let mut sys = System::new_all();
            sys.refresh_all();
            let pid = sysinfo::Pid::from_u32(std::process::id());
            if let Some(proc) = sys.process(pid) {
                let current_mb = proc.memory() / 1024 / 1024;
                if current_mb >= max_mb {
                    return Err(Error::SystemSaturation(format!(
                        "memory limit reached: {}MB/{}MB",
                        current_mb, max_mb
                    )));
                }
            }
        }

        // 1. Best-effort: Try to take from pool if args are mostly standard
        let can_pool = args.env.is_none() && args.cols.is_none() && args.rows.is_none();
        let pool_size = self.pool.lock().len();
        tracing::info!(can_pool, pool_size, ?args.env, ?args.cols, ?args.rows, "pooling check");

        if can_pool {
            let pooled = self.pool.lock().pop_front();
            if let Some(term) = pooled {
                let id = term.id();
                // Re-brand and re-configure
                term.set_metadata(
                    format!("Terminal {id}: {}", args.title),
                    args.max_lines,
                    args.timeout_ms.map(std::time::Duration::from_millis),
                );

                // Speculative Injection: If a command is provided, write it immediately
                if let Some(cmd) = &args.command {
                    let mut payload = cmd.clone();
                    if args.wait.unwrap_or(false) {
                        payload.push_str(" ; exit\r\n");
                    } else {
                        payload.push_str("\r\n");
                    }
                    let _ = term.write(payload.as_bytes());
                }

                self.terminals.lock().insert(
                    id,
                    Owned {
                        owner,
                        terminal: term.clone(),
                    },
                );
                if args.visible.unwrap_or(self.default_visible) {
                    let _ = term.promote_to_visible();
                }
                let _ = self.refill_tx.try_send(()); // Signal refill
                return Ok((id, 0, 0)); // Instant!
            }
        }

        // 2. Cold start (pool empty or custom args)
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let res =
            crate::terminal::spawn(id, args, self.default_visible, &self.prompt_regex).await?;
        self.terminals.lock().insert(
            id,
            Owned {
                owner,
                terminal: res.terminal,
            },
        );
        Ok((id, res.spawn_ms, res.ready_ms))
    }

    async fn fill_pool(&self) {
        let pool_target = 5; // Increased for fleet-wide readiness
        while self.pool.lock().len() < pool_target {
            // Admission Control: Don't over-fill pool if it risks saturation
            let current_total = (self.terminals.lock().len() + self.pool.lock().len()) as u32;
            if current_total >= self.max_terminals {
                tracing::debug!(
                    "pool fill paused: total terminals at limit ({})",
                    self.max_terminals
                );
                break;
            }

            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            let args = SpawnArgs {
                title: "Pre-warmed".into(),
                ..Default::default()
            };
            // Pre-warmed terminals are ALWAYS headless until promoted (BP-009)
            match crate::terminal::spawn(id, args, false, &self.prompt_regex).await {
                Ok(res) => {
                    self.pool.lock().push_back(res.terminal);
                    tracing::info!(id, "terminal pre-warmed (headless) and added to pool");

                    // Signal global readiness event (Zero-latency signaling for tests/clients)
                    #[cfg(windows)]
                    {
                        use std::os::windows::ffi::OsStrExt;
                        use windows_sys::Win32::System::Threading::{CreateEventW, SetEvent};

                        let name_str = "Local\\vterm-rs-ready-event";
                        let name: Vec<u16> = std::ffi::OsStr::new(name_str)
                            .encode_wide()
                            .chain(std::iter::once(0))
                            .collect();
                        unsafe {
                            let handle = CreateEventW(std::ptr::null(), 1, 0, name.as_ptr());
                            if handle != 0 {
                                if SetEvent(handle) != 0 {
                                    tracing::info!("local readiness event signaled: {}", name_str);
                                } else {
                                    tracing::error!("failed to signal local readiness event");
                                }
                                // We don't close the handle here to keep the event alive
                            } else {
                                tracing::error!("failed to create local readiness event");
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "failed to pre-warm terminal");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
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
        self.terminals
            .lock()
            .iter()
            .filter_map(|(id, o)| (o.owner == owner).then_some(*id))
            .collect()
    }

    /// Metadata for all matching terminals.
    pub fn list_metadata(&self, owner: Option<ConnectionId>) -> Vec<crate::protocol::TerminalInfo> {
        let terminals = self.terminals.lock();
        tracing::trace!(count = terminals.len(), ?owner, "listing terminals");
        terminals
            .iter()
            .filter(|(_, o)| owner.is_none_or(|id| o.owner == id))
            .map(|(id, o)| crate::protocol::TerminalInfo {
                id: *id,
                title: o.terminal.title(),
                pid: o.terminal.child_pid(),
                owner: o.owner.to_string(),
            })
            .collect()
    }

    /// High-speed batch match across all terminals owned by `owner`.
    pub fn match_all(
        &self,
        owner: ConnectionId,
        pattern: &str,
    ) -> Vec<crate::protocol::MatchEntry> {
        let mut ids: Vec<u32> = self
            .terminals
            .lock()
            .iter()
            .filter_map(|(id, o)| (o.owner == owner).then_some(*id))
            .collect();
        ids.sort_unstable(); // Strict acquisition order to prevent deadlocks (BP-011)

        let map = self.terminals.lock();
        ids.into_iter()
            .filter_map(|id| {
                map.get(&id).map(|o| {
                    let (matched, certain) = o.terminal.match_probabilistic(pattern);
                    crate::protocol::MatchEntry {
                        id,
                        matched,
                        certain,
                    }
                })
            })
            .collect()
    }

    /// Close one terminal. Returns `Error::UnknownTerminal` if `owner` doesn't own it.
    pub fn close(&self, owner: ConnectionId, id: u32) -> Result<()> {
        let mut map = self.terminals.lock();
        match map.get(&id) {
            Some(o) if o.owner == owner => {
                map.remove(&id);
                Ok(())
            }
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

    pub fn active_count(&self) -> u32 {
        self.terminals.lock().len() as u32
    }

    pub fn pool_size(&self) -> u32 {
        self.pool.lock().len() as u32
    }

    pub fn max_terminals(&self) -> u32 {
        self.max_terminals
    }

    pub fn max_mem_mb(&self) -> Option<u64> {
        self.max_mem_mb
    }

    /// Snapshot every terminal — used by the watchdog. Returns `(id, line_count,
    /// max_lines, spawn_time, max_duration, owner)` tuples.
    pub(crate) fn snapshot_for_watchdog(&self) -> Vec<WatchdogEntry> {
        self.terminals
            .lock()
            .iter()
            .map(|(id, o)| WatchdogEntry {
                id: *id,
                owner: o.owner,
                line_count: o.terminal.line_count(),
                max_lines: o.terminal.max_lines(),
                spawn_time: o.terminal.spawn_time(),
                max_duration: o.terminal.max_duration(),
            })
            .collect()
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
    max_terminals: Option<u32>,
    max_mem_mb: Option<u64>,
}

impl AppBuilder {
    pub fn default_visible(mut self, v: bool) -> Self {
        self.default_visible = Some(v);
        self
    }
    pub fn prompt_regex(mut self, r: impl Into<String>) -> Self {
        self.prompt_regex = Some(r.into());
        self
    }
    pub fn max_terminals(mut self, n: u32) -> Self {
        self.max_terminals = Some(n);
        self
    }
    pub fn max_mem_mb(mut self, m: u64) -> Self {
        self.max_mem_mb = Some(m);
        self
    }

    pub fn build(self) -> Result<Arc<App>> {
        let (refill_tx, mut refill_rx) = mpsc::channel(1);
        let app = Arc::new(App {
            terminals: Mutex::new(HashMap::new()),
            pool: Mutex::new(VecDeque::new()),
            refill_tx,
            next_id: AtomicU32::new(1),
            default_visible: self.default_visible.unwrap_or(true),
            prompt_regex: Regex::new(
                &self
                    .prompt_regex
                    .unwrap_or_else(|| r"(?:PS )?[A-Z]:\\.*> ?$".to_string()),
            )?,
            max_terminals: self.max_terminals.unwrap_or(100),
            max_mem_mb: self.max_mem_mb,
        });

        // Background pool warming task
        let app_clone = Arc::clone(&app);
        tokio::spawn(async move {
            let _ = app_clone.refill_tx.send(()).await; // Initial fill
            while let Some(()) = refill_rx.recv().await {
                app_clone.fill_pool().await;
            }
        });

        Ok(app)
    }
}
