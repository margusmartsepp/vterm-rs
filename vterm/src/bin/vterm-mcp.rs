use anyhow::Result;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::serve_server;
use rmcp::transport::io::stdio;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use vterm_rs::{OrchestratorClient, SkillCommand, SpawnArgs};

// Using default pipe location

// ─── MCP Tool Structs ────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct McpSpawnArgs {
    title: String,
    command: Option<String>,
    timeout_ms: Option<u64>,
    max_lines: Option<u32>,
    cols: Option<u16>,
    rows: Option<u16>,
    env: Option<std::collections::HashMap<String, String>>,
    /// If true, wait for the command to finish before returning.
    wait: Option<bool>,
    /// If true, include a semantic summary of the terminal output.
    semantic: Option<bool>,
    /// Optional regex with named capture groups to extract structured data from history.
    extract_pattern: Option<String>,
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
struct McpReadArgs {
    id: u32,
    #[serde(default)]
    history: bool,
}

#[derive(Deserialize, JsonSchema)]
struct McpWaitArgs {
    id: u32,
    pattern: String,
    timeout_ms: u64,
}

#[derive(Deserialize, JsonSchema)]
struct McpWaitStableArgs {
    id: u32,
    stable_ms: u64,
    timeout_ms: u64,
}

#[derive(Deserialize, JsonSchema)]
struct McpExtractArgs {
    id: u32,
    pattern: String,
    #[serde(default)]
    history: bool,
}

#[derive(Deserialize, JsonSchema)]
struct McpEvalArgs {
    code: String,
}

// ─── Server ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TerminalServer {
    client: OrchestratorClient,
}

#[tool_router(server_handler)]
impl TerminalServer {
    #[tool(
        name = "get_info",
        description = "Returns version and resource telemetry for the orchestrator."
    )]
    async fn get_info(&self) -> String {
        match self
            .client
            .execute(SkillCommand::Inspect { assurance: true })
            .await
        {
            Ok(res) => serde_json::to_string_pretty(&res).unwrap_or_else(|_| "{}".into()),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "list_terminals",
        description = "Returns a list of all active terminal IDs and their titles."
    )]
    async fn list_terminals(&self) -> String {
        match self.client.execute(SkillCommand::List { all: false }).await {
            Ok(res) => res.summary.unwrap_or_else(|| "[]".into()),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "spawn",
        description = "Spawns a new terminal session. Returns the terminal ID and any extracted data."
    )]
    async fn spawn(&self, args: Parameters<McpSpawnArgs>) -> String {
        let req = SpawnArgs {
            title: args.0.title,
            command: args.0.command,
            timeout_ms: args.0.timeout_ms,
            max_lines: args.0.max_lines,
            visible: Some(false),
            cols: args.0.cols,
            rows: args.0.rows,
            env: args.0.env,
            wait: args.0.wait,
            semantic: args.0.semantic,
            extract_pattern: args.0.extract_pattern,
        };
        match self.client.execute(SkillCommand::Spawn(req)).await {
            Ok(res) => serde_json::to_string_pretty(&res)
                .unwrap_or_else(|_| "Spawned successfully.".into()),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "write",
        description = "Writes keystrokes (like <Enter> or <Up>) or text to a terminal."
    )]
    async fn write(&self, args: Parameters<McpWriteArgs>) -> String {
        match self
            .client
            .execute(SkillCommand::ScreenWrite {
                id: args.0.id,
                text: args.0.text,
            })
            .await
        {
            Ok(_) => "Written successfully.".into(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "read",
        description = "Reads the current contents of the terminal screen. Set history=true for full scrollback."
    )]
    async fn read(&self, args: Parameters<McpReadArgs>) -> String {
        match self
            .client
            .execute(SkillCommand::ScreenRead {
                id: args.0.id,
                history: args.0.history,
            })
            .await
        {
            Ok(res) => res.content.unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "wait_until_stable",
        description = "Blocks until the screen buffer remains unchanged for a specified duration."
    )]
    async fn wait_until_stable(&self, args: Parameters<McpWaitStableArgs>) -> String {
        match self
            .client
            .execute(SkillCommand::WaitUntilStable {
                id: args.0.id,
                stable_ms: args.0.stable_ms,
                timeout_ms: args.0.timeout_ms,
            })
            .await
        {
            Ok(res) => res.content.unwrap_or_else(|| "Screen stabilized.".into()),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "screen_diff",
        description = "Returns only the visual delta since the last observation for this terminal."
    )]
    async fn screen_diff(&self, args: Parameters<McpIdArgs>) -> String {
        match self
            .client
            .execute(SkillCommand::ScreenDiff { id: args.0.id })
            .await
        {
            Ok(res) => res.content.unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "extract",
        description = "Extracts structured data from terminal history using a regex with named groups."
    )]
    async fn extract(&self, args: Parameters<McpExtractArgs>) -> String {
        match self
            .client
            .execute(SkillCommand::Extract {
                id: args.0.id,
                pattern: args.0.pattern,
                history: args.0.history,
            })
            .await
        {
            Ok(res) => {
                if let Some(extracted) = res.extracted {
                    serde_json::to_string_pretty(&extracted).unwrap_or_default()
                } else {
                    "[]".into()
                }
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "wait_until",
        description = "Waits until a regex pattern appears on the screen, streaming live output via progress notifications."
    )]
    async fn wait_until(
        &self,
        args: Parameters<McpWaitArgs>,
        meta: rmcp::model::Meta,
        peer: rmcp::Peer<rmcp::RoleServer>,
    ) -> String {
        let progress_token = meta.get_progress_token();
        let progress_token_str = progress_token.as_ref().map(|t| {
            serde_json::to_string(t)
                .unwrap_or_default()
                .replace("\"", "")
        });

        let mut events = self.client.events();
        let peer_clone = peer.clone();
        let token_clone = progress_token.clone();
        let event_handle = tokio::spawn(async move {
            if let Some(token) = token_clone {
                while let Ok(event) = events.recv().await {
                    if let vterm_rs::protocol::Event::Progress {
                            token: ev_token,
                            percentage,
                            msg,
                            ..
                        } = event {
                        let token_str = serde_json::to_string(&token)
                            .unwrap_or_default()
                            .replace("\"", "");
                        if ev_token == Some(token_str) {
                            let _ = peer_clone
                                .notify_progress(rmcp::model::ProgressNotificationParam {
                                    progress_token: token.clone(),
                                    progress: percentage as f64,
                                    total: Some(100.0),
                                    message: Some(msg),
                                })
                                .await;
                        }
                    }
                }
            }
        });

        let res = self
            .client
            .execute_full(
                SkillCommand::WaitUntil {
                    id: args.0.id,
                    pattern: args.0.pattern,
                    timeout_ms: args.0.timeout_ms,
                },
                progress_token_str,
            )
            .await;

        event_handle.abort();

        match res {
            Ok(res) => res.content.unwrap_or_else(|| "Pattern matched.".into()),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(name = "close", description = "Closes a terminal session.")]
    async fn close(&self, args: Parameters<McpIdArgs>) -> String {
        match self
            .client
            .execute(SkillCommand::ScreenClose {
                id: Some(args.0.id),
                target: "single".into(),
            })
            .await
        {
            Ok(_) => "Terminal closed.".into(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "batch",
        description = "Executes a batch of commands. Supports progress notifications."
    )]
    async fn batch(
        &self,
        args: Parameters<serde_json::Value>,
        meta: rmcp::model::Meta,
        peer: rmcp::Peer<rmcp::RoleServer>,
    ) -> String {
        let batch_args: vterm_rs::BatchArgs = match serde_json::from_value(args.0) {
            Ok(a) => a,
            Err(e) => return format!("Error parsing batch args: {e}"),
        };

        let progress_token = meta.get_progress_token();
        let progress_token_str = progress_token.as_ref().map(|t| {
            serde_json::to_string(t)
                .unwrap_or_default()
                .replace("\"", "")
        });

        let mut events = self.client.events();

        // Spawn a task to listen for events while the command is running
        let peer_clone = peer.clone();
        let token_clone = progress_token.clone();
        let event_handle = tokio::spawn(async move {
            if let Some(token) = token_clone {
                while let Ok(event) = events.recv().await {
                    if let vterm_rs::protocol::Event::Progress {
                            token: ev_token,
                            percentage,
                            msg,
                            ..
                        } = event {
                        let token_str = serde_json::to_string(&token)
                            .unwrap_or_default()
                            .replace("\"", "");
                        if ev_token == Some(token_str) {
                            let _ = peer_clone
                                .notify_progress(rmcp::model::ProgressNotificationParam {
                                    progress_token: token.clone(),
                                    progress: percentage as f64,
                                    total: Some(100.0),
                                    message: Some(msg),
                                })
                                .await;
                        }
                    }
                }
            }
        });

        let res = self
            .client
            .execute_full(SkillCommand::Batch(batch_args), progress_token_str)
            .await;
        event_handle.abort();

        match res {
            Ok(res) => {
                serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Batch completed.".into())
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "get_process_state",
        description = "Returns the current process state (running: bool, exit_code: int?)."
    )]
    async fn get_process_state(&self, args: Parameters<McpIdArgs>) -> String {
        match self
            .client
            .execute(SkillCommand::GetProcessState { id: args.0.id })
            .await
        {
            Ok(res) => serde_json::to_string_pretty(&res).unwrap_or_else(|_| "{}".into()),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "get_architecture",
        description = "Returns the architectural knowledge graph report for this codebase."
    )]
    async fn get_architecture(&self) -> String {
        match tokio::fs::read_to_string("graphify-out/GRAPH_REPORT.md").await {
            Ok(content) => content,
            Err(_) => match tokio::fs::read_to_string("GRAPH_REPORT.md").await {
                Ok(content) => content,
                Err(e) => {
                    format!("Error reading architecture report: {e}. Ensure graphify has been run.")
                }
            },
        }
    }

    #[tool(
        name = "rust_eval",
        description = "Evaluates a Rust code snippet using evcxr. Returns the output or error."
    )]
    async fn rust_eval(&self, args: Parameters<McpEvalArgs>) -> String {
        use std::process::Stdio;
        use tokio::io::AsyncWriteExt;
        use tokio::process::Command;

        let mut child = match Command::new("evcxr")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                return format!(
                    "Error: Failed to spawn evcxr. Ensure it is installed and in your PATH. ({e})"
                )
            }
        };

        let mut stdin = child.stdin.take().unwrap();
        let code = format!("{}\n:quit\n", args.0.code);
        if let Err(e) = stdin.write_all(code.as_bytes()).await {
            return format!("Error writing to evcxr: {e}");
        }
        drop(stdin);

        match child.wait_with_output().await {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // evcxr often outputs a lot of junk/prompts to stderr.
                // We'll return stdout if not empty, otherwise stderr.
                if !stdout.is_empty() {
                    stdout.to_string()
                } else {
                    stderr.to_string()
                }
            }
            Err(e) => format!("Error waiting for evcxr: {e}"),
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
    server
        .client
        .execute(SkillCommand::Hello {
            client_version: "vterm-mcp".into(),
        })
        .await?;

    // tracing::info!("vterm-mcp started, serving over stdio...");
    serve_server(server, stdio())
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}
