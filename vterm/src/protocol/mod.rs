//! Wire format. The single source of truth for what crosses the named pipe.
//!
//! See [`docs/protocol.md`](../../docs/protocol.md) for the prose spec; this module is
//! the executable companion. Every variant is `#[non_exhaustive]` and every field is
//! `#[serde(default)]` where optional, so older clients keep working as the protocol
//! grows.

mod command;

pub use command::*;
