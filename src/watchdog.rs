//! Background reaper: enforces `timeout_ms` and `max_lines` per terminal.
//!
//! Polls every 500ms. When a terminal exceeds either limit, it is removed from the
//! map (which drops the `Arc<Inner>` and triggers the `Drop` impl that kills the
//! child). Cheap, simple, correct.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::App;

pub fn spawn(app: Arc<App>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_millis(500));
        loop {
            tick.tick().await;

            let now = Instant::now();
            let mut to_close: Vec<(crate::session::ConnectionId, u32)> = Vec::new();

            for entry in app.snapshot_for_watchdog() {
                if let Some(max_d) = entry.max_duration
                    && now.duration_since(entry.spawn_time) > max_d
                {
                    to_close.push((entry.owner, entry.id));
                    continue;
                }
                if let Some(max_l) = entry.max_lines
                    && entry.line_count > max_l
                {
                    to_close.push((entry.owner, entry.id));
                }
            }

            for (owner, id) in to_close {
                if app.close(owner, id).is_ok() {
                    tracing::info!(id, "watchdog reaped terminal");
                }
            }
        }
    })
}
