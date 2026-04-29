pub mod app;
pub mod client;
pub mod error;
pub mod protocol;
pub mod service;
pub mod session;
pub mod shortcuts;
pub mod terminal;
pub mod watchdog;
pub mod window;

pub use app::App;
pub use client::OrchestratorClient;
pub use error::{Error, Result};
pub use protocol::{BatchArgs, CommandResult, Request, Response, SkillCommand, SpawnArgs, Status};
pub use session::ConnectionGuard;

/// Alias for [`Request`] to keep compatibility with older test harnesses.
pub type SkillRequest = Request;
