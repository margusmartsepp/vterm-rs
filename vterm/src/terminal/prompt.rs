//! Prompt-aware initialisation.
//!
//! The previous `sleep(2000)` was a footgun: too short on slow boxes, wasteful on fast
//! ones. We now poll the screen for a prompt regex at 50ms granularity until the
//! pattern matches — typically <300ms — and bail with a `Timeout` error if the prompt
//! never appears.

use std::time::{Duration, Instant};

use regex::Regex;

use super::{Terminal, state};
use crate::{Error, Result};

pub async fn wait(
    spawning: Terminal<state::Spawning>,
    re: &Regex,
    deadline: Duration,
) -> Result<Terminal<state::Ready>> {
    let started = Instant::now();
    while started.elapsed() < deadline {
        if re.is_match(spawning.raw_screen().trim_end()) {
            return Ok(spawning.into_ready());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err(Error::Timeout { what: "prompt", ms: deadline.as_millis() as u64 })
}
