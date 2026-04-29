//! Prompt-aware initialisation.
//!
//! The previous `sleep(2000)` was a footgun: too short on slow boxes, wasteful on fast
//! ones. We now poll the screen for a prompt regex at 50ms granularity until the
//! pattern matches — typically <300ms — and bail with a `Timeout` error if the prompt
//! never appears.

use std::time::{Duration, Instant};

use regex::Regex;

use super::{state, Terminal};
use crate::{Error, Result};

pub async fn wait(
    spawning: Terminal<state::Spawning>,
    re: &Regex,
    deadline: Duration,
) -> Result<Terminal<state::Ready>> {
    let started = Instant::now();
    let mut rx = spawning.subscribe();
    while started.elapsed() < deadline {
        let screen = spawning.raw_screen();
        if re.is_match(screen.trim_end()) {
            tracing::info!(id = spawning.id(), "prompt found");
            return Ok(spawning.into_ready());
        }

        // Wait for next screen update or a short timeout as fallback
        let _ = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await;
    }
    Err(Error::Timeout {
        what: "prompt",
        ms: deadline.as_millis() as u64,
    })
}
