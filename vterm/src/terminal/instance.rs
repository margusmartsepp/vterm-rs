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
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use portable_pty::Child;

use crate::Result;

/// Sealed type-state markers. Downstream crates can read them but can't add new ones.
pub mod state {
    mod private {
        pub trait Sealed {}
    }

    pub trait State: private::Sealed {}

    pub struct Spawning;
    impl private::Sealed for Spawning {}
    impl State for Spawning {}

    pub struct Ready;
    impl private::Sealed for Ready {}
    impl State for Ready {}
}
pub use state::State;

/// The actual state. Hidden from outside the crate; outside callers see only
/// `Terminal<S>`.
pub struct Inner {
    pub(crate) id: u32,
    pub(crate) title: Mutex<String>,
    pub(crate) writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    pub(crate) _master: Mutex<Box<dyn portable_pty::MasterPty + Send>>,
    pub(crate) parser: Arc<Mutex<vt100::Parser>>,
    pub(crate) child: Mutex<Box<dyn Child + Send + Sync>>,
    pub(crate) child_pid: u32,
    pub(crate) line_count: Arc<AtomicU32>,
    pub(crate) max_lines: Mutex<Option<u32>>,
    pub(crate) _scrollback: usize,
    pub(crate) max_duration: Mutex<Option<Duration>>,
    pub(crate) spawn_time: Instant,
    pub(crate) notifier: tokio::sync::broadcast::Sender<()>,
    pub(crate) shm: Option<super::shm::ShmBuffer>,
    pub(crate) last_content: Mutex<String>,
    pub(crate) full_history: Mutex<String>,
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
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            _state: PhantomData,
        }
    }
}

// Methods available in any state.
impl<S: State> Terminal<S> {
    pub fn id(&self) -> u32 {
        self.inner.id
    }
    pub fn title(&self) -> String {
        self.inner.title.lock().clone()
    }
    pub fn child_pid(&self) -> u32 {
        self.inner.child_pid
    }
    pub fn line_count(&self) -> u32 {
        self.inner.line_count.load(Ordering::Relaxed)
    }
    pub fn max_lines(&self) -> Option<u32> {
        *self.inner.max_lines.lock()
    }
    pub fn max_duration(&self) -> Option<Duration> {
        *self.inner.max_duration.lock()
    }
    pub fn spawn_time(&self) -> Instant {
        self.inner.spawn_time
    }

    /// Re-brand a pooled terminal.
    pub(crate) fn set_metadata(
        &self,
        title: String,
        max_lines: Option<u32>,
        timeout: Option<Duration>,
    ) {
        *self.inner.title.lock() = title;
        *self.inner.max_lines.lock() = max_lines;
        *self.inner.max_duration.lock() = timeout;
    }

    /// Update the window title dynamically.
    pub fn set_title(&self, title: &str) -> Result<()> {
        *self.inner.title.lock() = title.to_string();
        crate::window::set_title(self.inner.child_pid, title)
    }

    /// Promote a headless terminal to a visible window.
    pub fn promote_to_visible(&self) -> Result<()> {
        crate::window::show(self.inner.child_pid)?;
        crate::terminal::spawn_viewer_window(self.inner.id, &self.title());
        Ok(())
    }
    pub(crate) fn raw_screen(&self) -> String {
        self.inner.parser.lock().screen().contents()
    }

    /// Subscribe to screen update notifications.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.inner.notifier.subscribe()
    }

    /// High-speed probabilistic match using SHM Bloom filter.
    /// Returns (matched, certain).
    pub fn match_probabilistic(&self, pattern: &str) -> (bool, bool) {
        if let Some(shm) = &self.inner.shm {
            // If it's a simple token, we can check the Bloom filter
            let is_simple = pattern.chars().all(|c| c.is_alphanumeric());
            if is_simple && pattern.len() > 1 {
                if !shm.check_bloom(pattern) {
                    return (false, true); // Definitive NO
                }
                // Bloom hit - could be false positive.
                // Let's do a fast substring check on the SHM screen buffer if possible.
                let screen = self.raw_screen();
                if screen.contains(pattern) {
                    return (true, true); // Definitive YES
                }
                return (true, false); // Probabilistic YES (false positive in Bloom or stale)
            }

            // For complex patterns, fallback to full screen search (but still fast since it's SHM)
            let screen = self.raw_screen();
            if screen.contains(pattern) {
                return (true, true);
            }
        }
        (false, true)
    }
}

// Spawning-only methods.
impl Terminal<state::Spawning> {
    pub(crate) fn new(inner: Arc<Inner>) -> Self {
        Self {
            inner,
            _state: PhantomData,
        }
    }

    /// Promote to `Ready`. The transition is the only public way to obtain a
    /// `Terminal<Ready>` from `Spawning` — meaning callers physically cannot read or
    /// write before the prompt has been observed.
    pub(crate) fn into_ready(self) -> Terminal<state::Ready> {
        Terminal {
            inner: self.inner,
            _state: PhantomData,
        }
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
    /// If history is true, returns the full transcript (scrollback + screen) trimmed of ANSI codes.
    pub fn read_screen(&self, history: bool) -> String {
        let content = if history {
            let raw = self.inner.full_history.lock().clone();
            crate::terminal::instance::strip_ansi_and_normalize(&raw)
        } else {
            let parser = self.inner.parser.lock();
            parser.screen().contents()
        };

        let trimmed = content.trim_end().to_string();

        if !history {
            *self.inner.last_content.lock() = trimmed.clone();
        }

        trimmed
    }

    /// Returns only the lines that have changed since the last `read_screen` or `read_diff`.
    pub fn read_diff(&self) -> String {
        let current = self.read_screen(false);
        let last = self.inner.last_content.lock().clone();

        if current == last {
            return String::new();
        }

        // Simple line-based diff
        let current_lines: Vec<&str> = current.lines().collect();
        let last_lines: Vec<&str> = last.lines().collect();

        let mut diff = String::new();
        for (i, line) in current_lines.iter().enumerate() {
            if i >= last_lines.len() || *line != last_lines[i] {
                diff.push_str(line);
                diff.push('\n');
            }
        }

        *self.inner.last_content.lock() = current;
        diff.trim_end().to_string()
    }

    /// Cheap substring match against the rendered screen.
    pub fn matches(&self, pattern: &str) -> bool {
        self.inner
            .parser
            .lock()
            .screen()
            .contents()
            .contains(pattern)
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

pub(crate) fn strip_ansi_and_normalize(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut state = 0; // 0: normal, 1: esc, 2: csi, 3: osc
    let mut last_was_cr = false;

    for c in raw.chars() {
        match state {
            0 => {
                if c == '\x1b' {
                    state = 1;
                } else if c == '\r' {
                    out.push('\n');
                    last_was_cr = true;
                } else if c == '\n' {
                    if !last_was_cr {
                        out.push('\n');
                    }
                    last_was_cr = false;
                } else {
                    out.push(c);
                    last_was_cr = false;
                }
            }
            1 => match c {
                '[' => state = 2,
                ']' => state = 3,
                _ => state = 0,
            },
            2 => {
                // CSI: Ends with a letter [a-zA-Z] or '@' (64)
                if c.is_ascii_alphabetic() || c == '@' {
                    // HEURISTIC: If it was a cursor move or absolute position, inject a newline
                    // H: Home/Position, f: Position, B: Down, E: Next Line
                    if c == 'H' || c == 'f' || c == 'B' || c == 'E' {
                        out.push('\n');
                    }
                    state = 0;
                }
            }
            3 => {
                if c == '\x07' || c == '\x1b' {
                    // BEL or next ESC ends OSC.
                    state = 0;
                }
            }
            _ => state = 0,
        }
    }
    out
}
