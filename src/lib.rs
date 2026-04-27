pub mod app;
pub mod error;
pub mod protocol;
pub mod service;
pub mod session;
pub mod shortcuts;
pub mod terminal;
pub mod watchdog;
pub mod window;

pub use app::App;
pub use session::ConnectionGuard;
pub use protocol::{Request, Response, SkillCommand, CommandResult, SpawnArgs, BatchArgs, Status};
pub use error::{Error, Result};

/// Alias for [`Request`] to keep compatibility with older test harnesses.
pub type SkillRequest = Request;
