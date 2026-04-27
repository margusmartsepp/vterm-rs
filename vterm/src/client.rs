use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};
use crate::{Request, Response, SkillCommand, CommandResult, Status};

#[cfg(windows)]
const PIPE_NAME: &str = r"\\.\pipe\vterm-rs-skill";
#[cfg(not(windows))]
const PIPE_NAME: &str = "/tmp/vterm-rs-skill";

/// A multiplexed client for the vterm-rs orchestrator.
#[derive(Clone)]
pub struct OrchestratorClient {
    req_tx: tokio::sync::mpsc::Sender<(Request, oneshot::Sender<Response>)>,
    next_id: Arc<AtomicU64>,
}

impl OrchestratorClient {
    pub async fn connect() -> Result<Self> {
        let client = Self::try_connect().await?;
        let (req_tx, mut req_rx): (tokio::sync::mpsc::Sender<(Request, oneshot::Sender<Response>)>, tokio::sync::mpsc::Receiver<(Request, oneshot::Sender<Response>)>) = tokio::sync::mpsc::channel(32);
        
        let (reader, writer) = tokio::io::split(client);
        let mut buf_reader = BufReader::new(reader);
        let mut writer = writer;
        
        // Map from req_id -> oneshot channel
        let pending = Arc::new(dashmap::DashMap::<u64, oneshot::Sender<Response>>::new());
        let pending_clone = pending.clone();
        
        // Write loop
        tokio::spawn(async move {
            while let Some((req, tx)) = req_rx.recv().await {
                if let Some(id) = req.req_id {
                    pending.insert(id, tx);
                }
                if let Ok(json) = serde_json::to_string(&req) {
                    let _ = writer.write_all(json.as_bytes()).await;
                    let _ = writer.write_all(b"\n").await;
                    let _ = writer.flush().await;
                }
            }
        });
        
        // Read loop
        tokio::spawn(async move {
            let mut line = String::new();
            loop {
                line.clear();
                if buf_reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                    break;
                }
                if let Ok(res) = serde_json::from_str::<Response>(&line) {
                    if let Some(id) = res.req_id {
                        if let Some((_, tx)) = pending_clone.remove(&id) {
                            let _ = tx.send(res);
                        }
                    }
                }
            }
        });
        
        let client = Self { req_tx, next_id: Arc::new(AtomicU64::new(1)) };
        
        // Auto-handshake with orchestrator
        client.execute(SkillCommand::Hello { client_version: "vterm-client".into() }).await?;
        
        Ok(client)
    }

    #[cfg(windows)]
    pub async fn try_connect() -> Result<NamedPipeClient> {
        for i in 0..5 {
            if let Ok(client) = ClientOptions::new().open(PIPE_NAME) {
                return Ok(client);
            }
            if i == 0 {
                // Auto-spawn headless orchestrator if not found
                std::process::Command::new("cargo")
                    .args(["run", "-p", "vterm-rs", "--bin", "vterm", "--", "--headless"])
                    .spawn()
                    .ok();
            }
            sleep(Duration::from_millis(1000)).await;
        }
        anyhow::bail!("Failed to connect to vterm.exe orchestrator")
    }

    #[cfg(not(windows))]
    pub async fn try_connect() -> Result<tokio::net::TcpStream> {
        anyhow::bail!("Orchestrator connection (Named Pipes) is only supported on Windows in v0.7.1. Linux/macOS support is planned for v0.8.0.")
    }

    pub async fn execute(&self, command: SkillCommand) -> Result<CommandResult> {
        let req_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let req = Request { req_id: Some(req_id), command };
        let (tx, rx) = oneshot::channel();
        self.req_tx.send((req, tx)).await?;
        let res = rx.await?;
        if res.result.status == Status::Error {
            anyhow::bail!(res.result.error.unwrap_or_default())
        }
        Ok(res.result)
    }
}
