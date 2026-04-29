//! Per-connection identity and RAII reaping.
//!
//! Each accepted client connection gets a fresh [`ConnectionId`] and is held alive by
//! a [`ConnectionGuard`]. When the guard drops — for any reason: clean disconnect,
//! pipe error, panic — it tells the [`crate::App`] to reap every terminal owned by
//! that connection. No zombie `powershell.exe` processes.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// A monotonically-increasing identifier for one client connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(u64);

impl ConnectionId {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "conn-{}", self.0)
    }
}

/// RAII handle. Hold one per client connection; drop it to reap.
pub struct ConnectionGuard {
    id: ConnectionId,
    app: Arc<crate::App>,
}

impl ConnectionGuard {
    pub fn new(app: Arc<crate::App>) -> Self {
        let id = ConnectionId::new();
        tracing::info!(connection = %id, "session opened");
        Self { id, app }
    }

    pub fn id(&self) -> ConnectionId {
        self.id
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let reaped = self.app.reap_owner(self.id);
        tracing::info!(connection = %self.id, reaped, "session closed");
    }
}
