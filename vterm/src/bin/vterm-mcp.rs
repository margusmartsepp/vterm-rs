use anyhow::Result;
use rmcp::{tool, tool_router};
use rmcp::serve_server;
use rmcp::transport::io::stdio;
use rmcp::handler::server::wrapper::Parameters;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};
use vterm_rs::{Request, Response, SkillCommand, CommandResult, Status, SpawnArgs, OrchestratorClient};

const PIPE_NAME: &str = r"\\.\pipe\vterm-rs-skill";

// ─── MCP Tool Structs ────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct McpSpawnArgs {
    title: String,
    command: Option<String>,
    timeout_ms: Option<u64>,
    max_lines: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
struct McpWriteArgs {
    id: u32,
    text: String,
}

#[derive(Deserialize, JsonSchema)]
struct McpIdArgs {
    id: u32,
}

#[derive(Deserialize, JsonSchema)]
struct McpWaitArgs {
    id: u32,
    pattern: String,
    timeout_ms: u64,
}

// ─── Server ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TerminalServer {
    client: OrchestratorClient,
}

#[tool_router(server_handler)]
impl TerminalServer {
    #[tool(description = "Spawns a new terminal session. Returns the terminal ID.")]
    async fn terminal_spawn(&self, args: Parameters<McpSpawnArgs>) -> String {
        let req = SpawnArgs {
            title: args.0.title,
            command: args.0.command,
            timeout_ms: args.0.timeout_ms,
            max_lines: args.0.max_lines,
            visible: Some(false),
        };
        match self.client.execute(SkillCommand::Spawn(req)).await {
            Ok(res) => format!("Terminal {} spawned successfully.", res.id.unwrap()),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Writes keystrokes (like <Enter> or <Up>) or text to a terminal.")]
    async fn terminal_write(&self, args: Parameters<McpWriteArgs>) -> String {
        match self.client.execute(SkillCommand::ScreenWrite { id: args.0.id, text: args.0.text }).await {
            Ok(_) => "Written successfully.".into(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Reads the current contents of the terminal screen.")]
    async fn terminal_read(&self, args: Parameters<McpIdArgs>) -> String {
        match self.client.execute(SkillCommand::ScreenRead { id: args.0.id }).await {
            Ok(res) => res.content.unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Waits until a regex pattern appears on the screen, streaming live output via progress notifications.")]
    async fn terminal_wait_until(
        &self, 
        args: Parameters<McpWaitArgs>,
        meta: rmcp::model::Meta,
        peer: rmcp::Peer<rmcp::RoleServer>,
    ) -> String {
        let pattern = match regex::Regex::new(&args.0.pattern) {
            Ok(p) => p,
            Err(e) => return format!("Error: invalid regex: {e}"),
        };

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(args.0.timeout_ms);
        let progress_token = meta.get_progress_token();

        let mut last_content = String::new();

        while start.elapsed() < timeout {
            if let Ok(res) = self.client.execute(SkillCommand::ScreenRead { id: args.0.id }).await {
                if let Some(content) = res.content {
                    if content != last_content {
                        last_content = content.clone();
                        if let Some(token) = &progress_token {
                            let _ = peer.notify_progress(rmcp::model::ProgressNotificationParam {
                                progress_token: token.clone(),
                                progress: 0.0,
                                total: None,
                                // We send the screen content as the progress message.
                                message: Some(content.clone()),
                            }).await;
                        }
                    }
                    if pattern.is_match(&content) {
                        return content;
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        format!("Error: timeout of {}ms exceeded. Last screen:\n{}", args.0.timeout_ms, last_content)
    }

    #[tool(description = "Closes a terminal session.")]
    async fn terminal_close(&self, args: Parameters<McpIdArgs>) -> String {
        match self.client.execute(SkillCommand::ScreenClose {
            id: Some(args.0.id),
            target: "single".into(),
        }).await {
            Ok(_) => "Terminal closed.".into(),
            Err(e) => format!("Error: {e}"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let client = OrchestratorClient::connect().await?;
    let server = TerminalServer { client };
    
    // Auto-handshake with orchestrator
    server.client.execute(SkillCommand::Hello { client_version: "vterm-mcp".into() }).await?;

    tracing::info!("vterm-mcp started, serving over stdio...");
    serve_server(server, stdio()).await.map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}
