//! The innermost service: it actually does the work.
//!
//! Accepts a `SkillCommand`, returns a `CommandResult`. Errors that occur inside
//! handlers are converted into `CommandResult { status: Error, error: Some(_), .. }`
//! so the outer layers can keep `Error = Infallible` — every wire message is a
//! response, even on failure.

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use tower::Service;

use crate::App;
use crate::Result;
use crate::protocol::{BatchArgs, CommandResult, SkillCommand, Status};
use crate::session::ConnectionId;

#[derive(Clone)]
pub struct Dispatcher {
    app: Arc<App>,
    owner: ConnectionId,
}

impl Dispatcher {
    pub fn new(app: Arc<App>, owner: ConnectionId) -> Self {
        Self { app, owner }
    }
}

impl Service<SkillCommand> for Dispatcher {
    type Response = CommandResult;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = std::result::Result<CommandResult, Infallible>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<std::result::Result<(), Infallible>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, cmd: SkillCommand) -> Self::Future {
        let app = Arc::clone(&self.app);
        let owner = self.owner;
        Box::pin(async move {
            Ok(execute(app, owner, cmd)
                .await
                .unwrap_or_else(|e| CommandResult::err(e.to_string())))
        })
    }
}

/// Internal recursion target — used by `Batch` so sub-commands re-enter the same
/// machinery without re-cloning the whole Tower stack.
/// 
/// rationale: Central dispatcher that routes SkillCommands to their implementation handlers.
/// links: crate::App::spawn, crate::App::terminal, crate::terminal::Terminal::write, crate::terminal::Terminal::read_screen
async fn execute(app: Arc<App>, owner: ConnectionId, cmd: SkillCommand) -> Result<CommandResult> {
    use SkillCommand::*;
    let mut r = CommandResult::ok();

    match cmd {
        Hello { client_version } => {
            r.version = Some(env!("CARGO_PKG_VERSION").into());
            tracing::info!(client_version, "handshake successful");
        }

        Spawn(args) => {
            let id = app.spawn(owner, args).await?;
            r.id = Some(id);
        }

        ScreenWrite { id, text } => {
            let term = app.terminal(owner, id)?;
            term.write(&crate::shortcuts::parse(&text))?;
            r.id = Some(id);
        }

        ScreenRead { id } => {
            let term = app.terminal(owner, id)?;
            r.id = Some(id);
            r.content = Some(term.read_screen());
        }

        ScreenControl { id, action } => {
            let term = app.terminal(owner, id)?;
            crate::window::control(term.child_pid(), &action)?;
            r.id = Some(id);
        }

        ScreenClose { id, target } => {
            if target == "all" {
                let n = app.close_all(owner);
                r.content = Some(format!("closed {n}"));
            } else if let Some(id) = id {
                app.close(owner, id)?;
                r.id = Some(id);
            }
        }

        List {} => {
            let ids = app.list(owner);
            r.content = Some(serde_json::to_string(&ids).expect("Vec<u32> serialises"));
        }

        Wait { ms } => tokio::time::sleep(Duration::from_millis(ms)).await,

        WaitUntil { id, pattern, timeout_ms } => {
            let term = app.terminal(owner, id)?;
            r.id = Some(id);
            let started = Instant::now();
            let deadline = Duration::from_millis(timeout_ms);
            while started.elapsed() < deadline {
                if term.matches(&pattern) { return Ok(r); }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            return Err(crate::Error::Timeout { what: "wait_until", ms: timeout_ms });
        }

        Batch(BatchArgs { commands, stop_on_error, .. }) => {
            let mut subs = Vec::with_capacity(commands.len());
            let stop = stop_on_error.unwrap_or(false);

            for sub_cmd in commands {
                let started = Instant::now();
                // Box::pin because of the recursive async call.
                let mut sub_r = match Box::pin(execute(Arc::clone(&app), owner, sub_cmd)).await {
                    Ok(r) => r,
                    Err(e) => CommandResult::err(e.to_string()),
                };
                sub_r.duration_ms = started.elapsed().as_millis() as u64;
                let failed = sub_r.status == Status::Error;
                subs.push(sub_r);
                if failed && stop { break; }
            }

            let failures = subs.iter().filter(|s| s.status == Status::Error).count();
            if failures > 0 {
                r.status = Status::Error;
                r.error = Some(format!("{failures} sub-command(s) failed"));
            }
            r.sub_results = Some(subs);
        }
    }
    Ok(r)
}
