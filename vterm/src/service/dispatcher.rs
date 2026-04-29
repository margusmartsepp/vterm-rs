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

use crate::protocol::{BatchArgs, CommandResult, Event, Request, SkillCommand, Status};
use crate::session::ConnectionId;
use crate::App;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct Dispatcher {
    app: Arc<App>,
    owner: ConnectionId,
    event_tx: mpsc::UnboundedSender<Event>,
}

impl Dispatcher {
    pub fn new(app: Arc<App>, owner: ConnectionId, event_tx: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            app,
            owner,
            event_tx,
        }
    }
}

impl Service<Request> for Dispatcher {
    type Response = CommandResult;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<CommandResult, Infallible>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<std::result::Result<(), Infallible>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let app = Arc::clone(&self.app);
        let owner = self.owner;
        let event_tx = self.event_tx.clone();
        Box::pin(async move {
            let res: crate::Result<CommandResult> = execute(
                app,
                owner,
                req.command,
                req.req_id,
                req.progress_token,
                event_tx,
            )
            .await;
            Ok(res.unwrap_or_else(|e| CommandResult::err(e.to_string())))
        })
    }
}

async fn execute(
    app: Arc<App>,
    owner: ConnectionId,
    cmd: SkillCommand,
    req_id: Option<u64>,
    progress_token: Option<String>,
    event_tx: mpsc::UnboundedSender<Event>,
) -> crate::Result<CommandResult> {
    use SkillCommand::*;
    let mut r = CommandResult::ok();

    match cmd {
        Hello { client_version } => {
            r.version = Some(env!("CARGO_PKG_VERSION").into());
            tracing::info!(client_version, "handshake successful");
        }

        Spawn(args) => {
            let wait = args.wait.unwrap_or(false);
            let semantic = args.semantic.unwrap_or(false);
            let extract_pattern = args.extract_pattern.clone();
            let timeout_ms = args.timeout_ms.unwrap_or(60000);

            let (id, spawn_ms, ready_ms) = app.spawn(owner, args).await?;
            r.id = Some(id);
            r.spawn_ms = Some(spawn_ms);
            r.ready_ms = Some(ready_ms);

            if wait {
                let term = app.terminal(owner, id)?;
                let started = Instant::now();
                let deadline = Duration::from_millis(timeout_ms);
                let mut rx = term.subscribe();

                loop {
                    let (running, exit_code) = term.process_state();
                    if !running {
                        // Short drain period to ensure the pump catches the last bytes
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        r.running = Some(false);
                        r.exit_code = exit_code;
                        break;
                    }
                    if started.elapsed() > deadline {
                        r.running = Some(true);
                        r.error = Some("timeout waiting for completion".into());
                        break;
                    }

                    // Wait for next terminal event or timeout
                    let _ = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
                }

                if semantic {
                    r.summary = Some(distill_semantic_summary(&term));
                }

                // ── Post-process Spawn: bundle extraction if requested ──────────────────────
                if let Some(pattern) = extract_pattern {
                    let history = term.read_screen(true);
                    tracing::info!(
                        id,
                        history_len = history.len(),
                        pattern,
                        "performing bundled extraction"
                    );
                    r.extracted = Some(perform_extraction(&history, &pattern)?);
                }
            }
        }

        ScreenWrite { id, text } => {
            let term = app.terminal(owner, id)?;
            term.write(&crate::shortcuts::parse(&text))?;
            r.id = Some(id);
        }

        ScreenRead { id, history } => {
            let term = app.terminal(owner, id)?;
            r.id = Some(id);
            r.content = Some(term.read_screen(history));
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

        List { all } => {
            let info = app.list_metadata(if all { None } else { Some(owner) });
            r.content = Some(serde_json::to_string(&info).expect("Vec<TerminalInfo> serialises"));
        }

        Wait { timeout_ms } => tokio::time::sleep(Duration::from_millis(timeout_ms)).await,

        WaitUntil {
            id,
            pattern,
            timeout_ms,
        } => {
            let term = app.terminal(owner, id)?;
            let original_title = term.title();
            let _ = term.set_title(&format!("[AI WAITING: {}] {}", pattern, original_title));

            r.id = Some(id);
            let started = Instant::now();
            let deadline = Duration::from_millis(timeout_ms);
            let mut rx = term.subscribe();

            let res = loop {
                if started.elapsed() >= deadline {
                    break Err(crate::Error::Timeout {
                        what: "wait_until",
                        ms: timeout_ms,
                    });
                }
                if term.matches(&pattern) {
                    break Ok(r);
                }

                if let Some(token) = &progress_token {
                    let elapsed = started.elapsed().as_millis() as f64;
                    let _ = event_tx.send(Event::Progress {
                        req_id,
                        token: Some(token.clone()),
                        percentage: (elapsed / timeout_ms as f64 * 100.0).min(99.0) as f32,
                        msg: format!("waiting for pattern... ({}ms)", elapsed as u64),
                    });
                }

                let _ = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await;
            };

            let _ = term.set_title(&original_title);
            return res;
        }

        WaitUntilStable {
            id,
            stable_ms,
            timeout_ms,
        } => {
            let term = app.terminal(owner, id)?;
            r.id = Some(id);
            let started = Instant::now();
            let deadline = Duration::from_millis(timeout_ms);
            let mut rx = term.subscribe();
            let mut last_change = Instant::now();

            loop {
                if started.elapsed() >= deadline {
                    return Err(crate::Error::Timeout {
                        what: "wait_until_stable",
                        ms: timeout_ms,
                    });
                }

                if let Some(token) = &progress_token {
                    let elapsed = started.elapsed().as_millis() as f64;
                    let _ = event_tx.send(Event::Progress {
                        req_id,
                        token: Some(token.clone()),
                        percentage: (elapsed / timeout_ms as f64 * 100.0).min(99.0) as f32,
                        msg: format!(
                            "waiting for stability ({}ms stable)...",
                            last_change.elapsed().as_millis()
                        ),
                    });
                }

                // If no changes for stable_ms, we are stable.
                if last_change.elapsed() >= Duration::from_millis(stable_ms) {
                    r.content = Some(term.read_screen(false));
                    break;
                }

                // Wait for a change or the next stability check
                if let Ok(Ok(_)) = tokio::time::timeout(Duration::from_millis(200), rx.recv()).await
                {
                    last_change = Instant::now();
                }
            }
        }

        ScreenDiff { id } => {
            let term = app.terminal(owner, id)?;
            r.id = Some(id);
            r.content = Some(term.read_diff());
        }

        GetProcessState { id } => {
            let term = app.terminal(owner, id)?;
            let (running, exit_code) = term.process_state();
            r.id = Some(id);
            r.running = Some(running);
            r.exit_code = exit_code;
        }

        Inspect { assurance } => {
            r.version = Some(env!("CARGO_PKG_VERSION").into());
            r.active_terminals = Some(app.active_count());
            r.pool_size = Some(app.pool_size());
            r.max_terminals = Some(app.max_terminals());
            r.max_mem_mb = app.max_mem_mb();

            if assurance {
                use sysinfo::System;
                let mut sys = System::new_all();
                sys.refresh_all();

                let pid = sysinfo::Pid::from_u32(std::process::id());
                if let Some(proc) = sys.process(pid) {
                    r.mem_usage_mb = Some(proc.memory() / 1024 / 1024);
                }
            }
        }

        Takeover { version } => {
            tracing::info!(incoming_version = %version, "takeover requested");
            r.content = Some("takeover_accepted".into());
            // The bin layer will see this and exit.
        }

        MatchAll { pattern } => {
            let ids = app.list(owner);
            let mut titles: Vec<(u32, String)> = Vec::new();
            for id in ids {
                if let Ok(term) = app.terminal(owner, id) {
                    let original = term.title();
                    let _ = term.set_title(&format!("[AI SENSING...] {original}"));
                    titles.push((id, original));
                }
            }

            let matches = app.match_all(owner, &pattern);
            r.matches = Some(matches);

            for (id, original) in titles {
                if let Ok(term) = app.terminal(owner, id) {
                    let _ = term.set_title(&original);
                }
            }
        }

        Batch(BatchArgs {
            commands,
            stop_on_error,
            parallel,
            ..
        }) => {
            let total = commands.len();
            let mut subs = Vec::with_capacity(total);
            let stop = stop_on_error.unwrap_or(false);
            let is_parallel = parallel.unwrap_or(false);

            if is_parallel {
                let futures: Vec<_> = commands
                    .into_iter()
                    .enumerate()
                    .map(|(idx, sub_cmd)| {
                        let app = Arc::clone(&app);
                        let event_tx = event_tx.clone();
                        let progress_token = progress_token.clone();
                        Box::pin(async move {
                            let started = Instant::now();
                            let mut sub_r =
                                match execute(app, owner, sub_cmd, req_id, None, event_tx.clone())
                                    .await
                                {
                                    Ok(r) => r,
                                    Err(e) => CommandResult::err(e.to_string()),
                                };
                            sub_r.duration_ms = started.elapsed().as_millis() as u64;

                            let pct = (idx + 1) as f32 / total as f32 * 100.0;
                            let _ = event_tx.send(Event::Progress {
                                req_id,
                                token: progress_token,
                                percentage: pct,
                                msg: format!("Completed {}/{}", idx + 1, total),
                            });

                            sub_r
                        })
                    })
                    .collect();

                subs = futures_util::future::join_all(futures).await;
            } else {
                for (idx, sub_cmd) in commands.into_iter().enumerate() {
                    let started = Instant::now();
                    let mut sub_r = match Box::pin(execute(
                        Arc::clone(&app),
                        owner,
                        sub_cmd,
                        req_id,
                        None,
                        event_tx.clone(),
                    ))
                    .await
                    {
                        Ok(r) => r,
                        Err(e) => CommandResult::err(e.to_string()),
                    };
                    sub_r.duration_ms = started.elapsed().as_millis() as u64;

                    let pct = (idx + 1) as f32 / total as f32 * 100.0;
                    let _ = event_tx.send(Event::Progress {
                        req_id,
                        token: progress_token.clone(),
                        percentage: pct,
                        msg: format!("Completed {}/{}", idx + 1, total),
                    });

                    let failed = sub_r.status == Status::Error;
                    subs.push(sub_r);
                    if failed && stop {
                        break;
                    }
                }
            }

            let failures = subs.iter().filter(|s| s.status == Status::Error).count();
            if failures > 0 {
                r.status = Status::Error;
                r.error = Some(format!("{failures} sub-command(s) failed"));
            }
            r.sub_results = Some(subs);
        }

        Extract {
            id,
            pattern,
            history,
        } => {
            let term = app.terminal(owner, id)?;
            let content = term.read_screen(history);
            r.id = Some(id);
            r.extracted = Some(perform_extraction(&content, &pattern)?);
        }

        _ => return Err(crate::Error::Protocol("unsupported command variant".into())),
    }

    Ok(r)
}

fn perform_extraction(
    content: &str,
    pattern: &str,
) -> crate::Result<Vec<std::collections::HashMap<String, String>>> {
    let re = regex::Regex::new(pattern)
        .map_err(|e| crate::Error::Protocol(format!("invalid regex: {e}")))?;

    let mut results = Vec::new();
    for caps in re.captures_iter(content) {
        let mut map = std::collections::HashMap::new();
        for name in re.capture_names().flatten() {
            if let Some(m) = caps.name(name) {
                map.insert(name.to_string(), m.as_str().to_string());
            }
        }
        if !map.is_empty() {
            results.push(map);
        }
    }
    Ok(results)
}

fn distill_semantic_summary(
    term: &crate::terminal::Terminal<crate::terminal::state::Ready>,
) -> String {
    let content = term.read_screen(true);
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return "empty output".into();
    }

    // Patterns for interesting things
    let error_patterns = ["error:", "failed:", "panic!", "exception", "err:"];
    let success_patterns = [
        "finished",
        "success",
        "completed",
        "done",
        "ping statistics",
    ];

    let mut highlights = Vec::new();

    // 1. Look for errors with context
    for (i, line) in lines.iter().enumerate() {
        let low = line.to_lowercase();
        if error_patterns.iter().any(|p| low.contains(p)) {
            // Add a snippet (line before, matching line, line after)
            let start = i.saturating_sub(1);
            let end = (i + 2).min(lines.len());
            highlights.push(format!("--- ERROR SNIPPET (line {}) ---", i + 1));
            for l in &lines[start..end] {
                highlights.push(l.to_string());
            }
            highlights.push(String::new());
        }
    }

    // 2. If no errors, look for success/summary lines
    if highlights.is_empty() {
        for (i, line) in lines.iter().enumerate() {
            let low = line.to_lowercase();
            if success_patterns.iter().any(|p| low.contains(p)) {
                highlights.push(format!("--- SUMMARY (line {}) ---", i + 1));
                highlights.push(line.to_string());
            }
        }
    }

    // 3. Fallback: Last 15 lines
    if highlights.is_empty() {
        highlights.push("--- LAST 15 LINES ---".into());
        let start = lines.len().saturating_sub(15);
        for l in &lines[start..] {
            highlights.push(l.to_string());
        }
    }

    highlights.join("\n")
}
