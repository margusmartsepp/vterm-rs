//! `vterm.exe` — orchestrator binary. A thin shell over the `vterm-rs` library.
//!
//! Two modes:
//!
//! * **Server (default):** bind a singleton named pipe, accept one client at a time,
//!   pump every line through the Tower pipeline.
//! * **Viewer (`--client <id>`):** connect to a per-terminal viewer pipe and act as a
//!   transparent stdin/stdout bridge — this is what the spawned `cmd /c start
//!   powershell` window runs.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::terminal as ct_term;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeServer, ServerOptions};

use tower::{Service, ServiceExt};
use tracing_subscriber::EnvFilter;

use vterm_rs::{App, ConnectionGuard, Request, Response};

#[cfg(windows)]
const PIPE_NAME: &str = r"\\.\pipe\vterm-rs-skill";
#[cfg(not(windows))]
const PIPE_NAME: &str = "/tmp/vterm-rs-skill";

#[derive(Parser, Debug)]
#[command(name = "vterm", version, about = "PTY orchestrator for AI agents")]
struct Args {
    /// Default: spawn visible PowerShell windows for every Spawn (overridden per-command).
    #[arg(long, conflicts_with = "headless")]
    visible: bool,

    /// Default: spawn no visible windows. Recommended for CI / AI use.
    #[arg(long)]
    headless: bool,

    /// Regex used to detect "shell ready". Must match the rendered screen.
    #[arg(long, default_value = r"PS [A-Z]:\\.*> ?$")]
    prompt_regex: String,

    /// Internal: act as a viewer for terminal `<id>` instead of running the server.
    #[arg(long)]
    client: Option<u32>,

    /// Run a single command against a running orchestrator and exit.
    #[arg(long, value_name = "VARIANT")]
    skill: Option<String>,

    /// JSON payload for the skill (optional).
    #[arg(long)]
    payload: Option<String>,

    /// Print the path to the Graphify architectural map and exit.
    #[arg(long)]
    graph: bool,

    /// Admission Control: Maximum concurrent terminals allowed (default: 100).
    #[arg(long, default_value = "100")]
    max_terminals: u32,

    /// Admission Control: Maximum memory usage in MB for the orchestrator process.
    #[arg(long)]
    max_mem_mb: Option<u64>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();

    if args.graph {
        let path = std::env::current_dir()?.join("graphify-out").join("graph.json");
        if path.exists() {
            println!("{}", path.display());
            return Ok(());
        } else {
            anyhow::bail!("Graph not found. Run scripts/generate_graph.ps1 first.");
        }
    }

    if let Some(id) = args.client {
        return run_client(id).await;
    }

    if let Some(variant) = args.skill {
        return run_skill(variant, args.payload).await;
    }

    let mut builder = App::builder()
        .default_visible(!args.headless)
        .prompt_regex(&args.prompt_regex)
        .max_terminals(args.max_terminals);
    
    if let Some(m) = args.max_mem_mb {
        builder = builder.max_mem_mb(m);
    }

    let app = builder.build().context("App::build")?;

    let _watchdog = vterm_rs::watchdog::spawn(Arc::clone(&app));

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        default_visible = app_default_visible(args.headless, args.visible),
        "vterm orchestrator starting",
    );

    accept_loop(app).await
}

fn app_default_visible(headless: bool, visible: bool) -> &'static str {
    match (headless, visible) {
        (true, _)  => "false (--headless)",
        (_, true)  => "true (--visible)",
        _          => "true (default)",
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_env("VTERM_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info,vterm_rs=debug"));
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_names(true)
        .compact()
        .init();
}

// ── server ────────────────────────────────────────────────────────────────────

#[cfg(windows)]
async fn accept_loop(app: Arc<App>) -> Result<()> {
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

    let mut first = true;
    loop {
        let mut opts = ServerOptions::new();
        opts.first_pipe_instance(first);
        
        let server = match opts.create(PIPE_NAME) {
            Ok(s) => {
                first = false;
                s
            }
            Err(e) if (e.kind() == std::io::ErrorKind::AlreadyExists || e.raw_os_error() == Some(5)) && first => {
                tracing::info!("pipe already bound (os error 5), attempting takeover...");
                if let Err(te) = attempt_takeover().await {
                    anyhow::bail!("takeover failed: {te}");
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }
            Err(e) => anyhow::bail!("pipe create failed: {e}"),
        };

        tracing::info!(pipe = PIPE_NAME, "waiting for connection");

        tokio::select! {
            res = server.connect() => {
                if let Err(e) = res {
                    tracing::warn!(error = %e, "accept failed");
                    continue;
                }
                
                let app = Arc::clone(&app);
                let shutdown_tx = shutdown_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(app, server, shutdown_tx).await {
                        tracing::warn!(error = %e, "connection ended with error");
                    }
                });
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    tracing::info!("shutdown signal received, closing orchestrator");
                    break Ok(());
                }
            }
        }
    }
}

#[cfg(windows)]
async fn attempt_takeover() -> Result<()> {
    use vterm_rs::protocol::SkillCommand;
    let client = ClientOptions::new()
        .open(PIPE_NAME)
        .context("failed to connect for takeover")?;
    
    let (reader, mut writer) = tokio::io::split(client);
    let mut br = BufReader::new(reader);

    let req = Request {
        req_id: Some(999),
        progress_token: None,
        command: SkillCommand::Takeover { version: env!("CARGO_PKG_VERSION").into() },
    };
    let json = serde_json::to_string(&req)?;
    
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    let mut response = String::new();
    if br.read_line(&mut response).await? > 0 {
        let res = serde_json::from_str::<Response>(&response)?;
        if res.result.content == Some("takeover_accepted".into()) {
            tracing::info!("takeover accepted by remote instance");
            return Ok(());
        }
    }
    anyhow::bail!("takeover rejected or timed out")
}

#[cfg(not(windows))]
async fn accept_loop(_app: Arc<App>) -> Result<()> {
    anyhow::bail!("Orchestrator server is only supported on Windows in v0.7.1")
}

#[cfg(windows)]
async fn handle_connection(
    app: Arc<App>, 
    conn: NamedPipeServer, 
    shutdown_tx: tokio::sync::watch::Sender<bool>
) -> Result<()> {
    let guard = ConnectionGuard::new(Arc::clone(&app));
    let owner = guard.id();
    tracing::info!(owner = %owner, "handling new connection");

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut svc = vterm_rs::service::pipeline(app, owner, event_tx);
    let (reader, writer) = tokio::io::split(conn);
    let writer = Arc::new(tokio::sync::Mutex::new(writer));
    let mut br = BufReader::new(reader);

    // Event handling task
    let writer_clone = Arc::clone(&writer);
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&event) {
                let mut w = writer_clone.lock().await;
                let _ = w.write_all(json.as_bytes()).await;
                let _ = w.write_all(b"\n").await;
                let _ = w.flush().await;
            }
        }
    });

    let mut line = String::new();
    while br.read_line(&mut line).await? > 0 {
        let raw = line.trim();
        tracing::trace!(raw = %raw, "incoming request");
        if raw.is_empty() {
            line.clear();
            continue;
        }

        let response = match serde_json::from_str::<Request>(raw) {
            Ok(req) => {
                let variant = req.command.variant_name();
                let started = std::time::Instant::now();
                
                let res = match svc.ready().await {
                    Ok(ready) => match ready.call(req).await {
                        Ok(mut r) => {
                            r.result.duration_ms = started.elapsed().as_millis() as u64;
                            r
                        }
                        Err(e) => Response::error(None, format!("execute: {e}")),
                    },
                    Err(e) => Response::error(None, format!("service: {e}")),
                };

                // Persist metrics
                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("vterm_metrics.jsonl") {
                    use std::io::Write as _;
                    let entry = serde_json::json!({
                        "ts": chrono::Utc::now().to_rfc3339(),
                        "type": variant,
                        "duration_ms": res.result.duration_ms,
                        "status": format!("{:?}", res.result.status).to_lowercase(),
                        "id": res.result.id,
                    });
                    let _ = writeln!(f, "{}", entry.to_string());
                }
                res
            }
            Err(e) => {
                tracing::warn!(error = %e, raw = %raw, "parse failure");
                Response::error(None, format!("parse: {e}"))
            }
        };

        let is_takeover_accepted = if let Ok(req) = serde_json::from_str::<Request>(raw) {
            req.command.variant_name() == "takeover" && response.result.content == Some("takeover_accepted".into())
        } else {
            false
        };

        let json = serde_json::to_string(&response)?;
        let mut w = writer.lock().await;
        w.write_all(json.as_bytes()).await?;
        w.write_all(b"\n").await?;
        w.flush().await?;
        drop(w);

        if is_takeover_accepted {
            let _ = shutdown_tx.send(true);
            break; // Exit the loop
        }

        line.clear();
    }
    drop(guard); // explicit — reaping happens here
    Ok(())
}

// ── viewer client ─────────────────────────────────────────────────────────────

#[cfg(windows)]
async fn run_client(id: u32) -> Result<()> {
    let pipe_name = format!(r"\\.\pipe\vterm-rs-client-{id}");
    let client = loop {
        match ClientOptions::new().open(&pipe_name) {
            Ok(c) => break c,
            Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    };

    ct_term::enable_raw_mode().ok();
    let (mut pipe_reader, mut pipe_writer) = tokio::io::split(client);

    let read_task = tokio::spawn(async move {
        use std::io::Write as _;
        let mut stdout = std::io::stdout();
        let mut buf = [0u8; 8192];
        while let Ok(n) = pipe_reader.read(&mut buf).await {
            if n == 0 { break; }
            let _ = stdout.write_all(&buf[..n]);
            let _ = stdout.flush();
        }
    });

    let mut buf = [0u8; 1024];
    let mut stdin = tokio::io::stdin();
    while let Ok(n) = stdin.read(&mut buf).await {
        if n == 0 { break; }
        if pipe_writer.write_all(&buf[..n]).await.is_err() { break; }
        let _ = pipe_writer.flush().await;
    }

    read_task.abort();
    let _ = ct_term::disable_raw_mode();
    Ok(())
}

#[cfg(not(windows))]
async fn run_client(_id: u32) -> Result<()> {
    anyhow::bail!("Viewer mode is only supported on Windows in v0.7.1")
}

#[cfg(windows)]
async fn run_skill(variant: String, payload: Option<String>) -> Result<()> {
    let client = ClientOptions::new()
        .open(PIPE_NAME)
        .context("failed to connect to orchestrator")?;
    
    let (reader, mut writer) = tokio::io::split(client);
    let mut br = BufReader::new(reader);

    let raw_payload = payload.unwrap_or_else(|| "{}".to_string());
    let json = format!(r#"{{"type":"{}","payload":{}}}"#, variant, raw_payload);
    
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    let mut response = String::new();
    if br.read_line(&mut response).await? > 0 {
        println!("{}", response.trim());
    }
    Ok(())
}

#[cfg(not(windows))]
async fn run_skill(_variant: String, _payload: Option<String>) -> Result<()> {
    anyhow::bail!("Skill CLI mode is only supported on Windows in v0.7.1")
}
