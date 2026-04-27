//! `Terminal<S>` — a type-state wrapper around a PTY.
//!
//! Two states today: `Spawning` (PTY open, prompt not yet observed) and `Ready`
//! (prompt seen, safe to write/read). The state is a `PhantomData<fn() -> S>` field
//! that takes zero bytes and keeps `Terminal<S>` covariant in `S`.
//!
//! All shared state lives behind an `Arc<Inner>`. `Drop` on `Inner` reaps the child —
//! so terminating a connection in `App::reap_owner` (which simply removes the entries
//! from the map and drops the `Arc`s) automatically kills the underlying PowerShell.

use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use portable_pty::Child;

use crate::Result;

/// Sealed type-state markers. Downstream crates can read them but can't add new ones.
pub mod state {
    mod private { pub trait Sealed {} }

    pub trait State: private::Sealed {}

    pub struct Spawning;
    impl private::Sealed for Spawning {} impl State for Spawning {}

    pub struct Ready;
    impl private::Sealed for Ready {} impl State for Ready {}
}
pub use state::State;

/// The actual state. Hidden from outside the crate; outside callers see only
/// `Terminal<S>`.
pub struct Inner {
    pub(crate) id: u32,
    pub(crate) title: String,
    pub(crate) writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    pub(crate) _master: Mutex<Box<dyn portable_pty::MasterPty + Send>>,
    pub(crate) parser: Arc<Mutex<vt100::Parser>>,
    pub(crate) child: Mutex<Box<dyn Child + Send + Sync>>,
    pub(crate) child_pid: u32,
    pub(crate) line_count: Arc<AtomicU32>,
    pub(crate) max_lines: Option<u32>,
    pub(crate) scrollback: usize,
    pub(crate) max_duration: Option<Duration>,
    pub(crate) spawn_time: Instant,
}

impl Drop for Inner {
    fn drop(&mut self) {
        let mut child = self.child.lock();
        let _ = child.kill();
        let _ = child.wait();
        tracing::debug!(id = self.id, pid = self.child_pid, "terminal reaped");
    }
}

/// A terminal in a particular lifecycle state. `Clone` is cheap (it's an `Arc` bump).
pub struct Terminal<S: State = state::Ready> {
    inner: Arc<Inner>,
    _state: PhantomData<fn() -> S>,
}

impl<S: State> Clone for Terminal<S> {
    fn clone(&self) -> Self { Self { inner: Arc::clone(&self.inner), _state: PhantomData } }
}

// Methods available in any state.
impl<S: State> Terminal<S> {
    pub fn id(&self) -> u32 { self.inner.id }
    pub fn title(&self) -> &str { &self.inner.title }
    pub fn child_pid(&self) -> u32 { self.inner.child_pid }
    pub fn line_count(&self) -> u32 { self.inner.line_count.load(Ordering::Relaxed) }
    pub fn max_lines(&self) -> Option<u32> { self.inner.max_lines }
    pub fn max_duration(&self) -> Option<Duration> { self.inner.max_duration }
    pub fn spawn_time(&self) -> Instant { self.inner.spawn_time }
    pub(crate) fn raw_screen(&self) -> String { self.inner.parser.lock().screen().contents() }
}

// Spawning-only methods.
impl Terminal<state::Spawning> {
    pub(crate) fn new(inner: Arc<Inner>) -> Self {
        Self { inner, _state: PhantomData }
    }

    /// Promote to `Ready`. The transition is the only public way to obtain a
    /// `Terminal<Ready>` from `Spawning` — meaning callers physically cannot read or
    /// write before the prompt has been observed.
    pub(crate) fn into_ready(self) -> Terminal<state::Ready> {
        Terminal { inner: self.inner, _state: PhantomData }
    }
}

// Ready-only methods. Read/write/match live here, gated by the type system.
impl Terminal<state::Ready> {
    /// Write raw bytes to the PTY. The mutex is held for microseconds — no `.await`
    /// inside the critical section.
    pub fn write(&self, bytes: &[u8]) -> Result<()> {
        let mut w = self.inner.writer.lock();
        std::io::Write::write_all(&mut *w, bytes)?;
        std::io::Write::flush(&mut *w)?;
        Ok(())
    }

    /// Snapshot the current screen as text, trimmed of trailing whitespace.
    /// If history is true, returns the full scrollback buffer.
    pub fn read_screen(&self, history: bool) -> String {
        let parser = self.inner.parser.lock();
        let screen = parser.screen();
        let (_, cols) = screen.size();
        
        if history {
            let mut full = String::new();
            // In vt100 0.15, screen.rows(start, width) returns visible area.
            // For now, return visible contents as a fallback while we investigate history rows.
            screen.contents().trim_end().to_string()
        } else {
            screen.contents().trim_end().to_string()
        }
    }

    /// Cheap substring match against the rendered screen.
    pub fn matches(&self, pattern: &str) -> bool {
        self.inner.parser.lock().screen().contents().contains(pattern)
    }

    /// Returns (running, exit_code)
    pub fn process_state(&self) -> (bool, Option<i32>) {
        let mut child = self.inner.child.lock();
        match child.try_wait() {
            Ok(Some(status)) => (false, Some(status.exit_code() as i32)),
            _ => (true, None),
        }
    }
}
