//! The crate's single error type.
//!
//! `anyhow` is permitted only at the binary edge (`src/main.rs`). Library code returns
//! `term::Result<T>` so that callers can match on a closed taxonomy of failure modes
//! and so that the wire format ([`crate::protocol::CommandResult::error`]) carries a
//! stable, prefixed message.
//!
//! Variants are `#[non_exhaustive]` to leave room for future categories without it
//! being a SemVer break.

use std::io;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] io::Error),

    #[error("pty: {0}")]
    Pty(String),

    #[error("terminal: id {0} not found")]
    UnknownTerminal(u32),

    #[error("saturation: {0}")]
    SystemSaturation(String),

    #[error("timeout: {what} after {ms}ms")]
    Timeout { what: &'static str, ms: u64 },

    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("window: {0}")]
    Window(String),

    #[error("regex: {0}")]
    Regex(#[from] regex::Error),

    #[error("protocol: {0}")]
    Protocol(String),
}

/// Crate-wide convenience alias.
pub type Result<T, E = Error> = std::result::Result<T, E>;
