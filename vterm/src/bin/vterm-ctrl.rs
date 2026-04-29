use anyhow::Result;
use clap::{Parser, Subcommand};
use comfy_table::{Attribute, Cell, Color, Table};
use vterm_protocol::{SkillCommand, SpawnArgs};
use vterm_rs::OrchestratorClient;

#[derive(Parser)]
#[command(name = "vterm-ctrl")]
#[command(about = "VTerm Orchestrator Control — Fleet Management CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a command safely (wait for completion, auto-kill on limit)
    Run {
        /// Command to run
        cmd: String,
        /// Optional title for the terminal
        #[arg(short, long)]
        title: Option<String>,
        /// Max lines to allow before auto-killing
        #[arg(short, long, default_value = "1000")]
        max_lines: u32,
        /// Max duration in milliseconds before auto-killing
        #[arg(short = 'T', long, default_value = "60000")]
        timeout: u64,
        /// Keep the terminal alive after the command finishes
        #[arg(short, long)]
        keep: bool,
        /// Distill output into a semantic summary (optimized for AI)
        #[arg(short, long)]
        semantic: bool,
        /// Output result as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// List all active terminals in the fleet
    List,
    /// Spawn a new terminal
    Spawn {
        /// Optional title for the terminal
        #[arg(short, long)]
        title: Option<String>,
        /// Optional command to run
        #[arg(short, long)]
        cmd: Option<String>,
    },
    /// Send input to a terminal
    Write {
        /// Terminal ID
        id: u32,
        /// Text to send (supports <Enter>, <C-c>, etc.)
        text: String,
    },
    /// Wait for a pattern to appear in a terminal
    Wait {
        /// Terminal ID
        id: u32,
        /// Regex pattern to wait for
        pattern: String,
        /// Timeout in milliseconds
        #[arg(short, long, default_value = "30000")]
        timeout: u64,
    },
    /// Read the screen of a terminal (uses high-speed SHM)
    Read {
        /// Terminal ID
        id: u32,
    },
    /// Trigger an orchestrator hotswap (takeover)
    Takeover {
        /// Version string of the incoming orchestrator
        version: String,
    },
    /// Inspect orchestrator resource health
    Top,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = OrchestratorClient::connect().await?;

    match cli.command {
        Commands::Run {
            cmd,
            title,
            max_lines,
            timeout,
            keep,
            semantic,
            json,
        } => {
            let res = client
                .execute(SkillCommand::Spawn(SpawnArgs {
                    title: title
                        .unwrap_or_else(|| cmd.split_whitespace().next().unwrap_or("run").into()),
                    command: Some(cmd.clone()),
                    max_lines: Some(max_lines),
                    timeout_ms: Some(timeout),
                    visible: Some(false),
                    wait: Some(true), // We always wait for 'Run' now at the orchestrator level
                    semantic: Some(semantic),
                    ..Default::default()
                }))
                .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&res)?);
                return Ok(());
            }

            let id = res
                .id
                .ok_or_else(|| anyhow::anyhow!("Failed to get terminal ID"))?;

            if semantic {
                if let Some(summary) = res.summary {
                    println!("\n--- SEMANTIC SUMMARY ---");
                    println!("{}", summary);
                    println!("------------------------");
                }
            } else {
                // Legacy streaming loop for human users
                println!("Streaming output for terminal {}...", id);
                let shm_name = format!("vterm-rs-shm-{}", id);
                let mut last_content = String::new();

                let mut shm = None;
                for _ in 0..10 {
                    if let Ok(s) =
                        vterm_rs::terminal::shm::ShmBuffer::open_existing(&shm_name, 4096)
                    {
                        shm = Some(s);
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                let shm = shm.ok_or_else(|| anyhow::anyhow!("Failed to open SHM"))?;

                // We already waited at the orchestrator level, so the process is likely done
                // but we might need to display the buffer.
                println!("{}", shm.read_screen());
            }

            println!(
                "\nProcess exited with code: {:?}",
                res.exit_code.unwrap_or(-1)
            );
            if !keep {
                client
                    .execute(SkillCommand::ScreenClose {
                        id: Some(id),
                        target: "terminal".into(),
                    })
                    .await?;
            }
        }

        Commands::List => {
            let res = client.execute(SkillCommand::List { all: true }).await?;
            if let Some(content) = res.content {
                let list: Vec<vterm_protocol::TerminalInfo> = serde_json::from_str(&content)?;

                let mut table = Table::new();
                table.set_header(vec![
                    Cell::new("ID")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Cyan),
                    Cell::new("Title").add_attribute(Attribute::Bold),
                    Cell::new("PID")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Yellow),
                    Cell::new("Owner")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::DarkGrey),
                ]);

                for entry in list {
                    table.add_row(vec![
                        entry.id.to_string(),
                        entry.title,
                        entry.pid.to_string(),
                        entry.owner,
                    ]);
                }
                println!("{}", table);
            }
        }

        Commands::Spawn { title, cmd } => {
            let res = client
                .execute(SkillCommand::Spawn(SpawnArgs {
                    title: title.unwrap_or_else(|| "cli-spawned".into()),
                    command: cmd,
                    ..Default::default()
                }))
                .await?;
            println!("Terminal spawned successfully. ID: {}", res.id.unwrap_or(0));
            println!(
                "Timings: spawn={}ms, ready={}ms",
                res.spawn_ms.unwrap_or(0),
                res.ready_ms.unwrap_or(0)
            );
        }

        Commands::Write { id, text } => {
            client
                .execute(SkillCommand::ScreenWrite { id, text })
                .await?;
            println!("Input sent to terminal {}", id);
        }

        Commands::Wait {
            id,
            pattern,
            timeout,
        } => {
            println!("Waiting for pattern '{}' in terminal {}...", pattern, id);
            let res = client
                .execute(SkillCommand::WaitUntil {
                    id,
                    pattern,
                    timeout_ms: timeout,
                })
                .await?;
            println!("Pattern matched in {}ms.", res.duration_ms);
        }

        Commands::Read { id } => {
            let shm_name = format!("vterm-rs-shm-{}", id);
            match vterm_rs::terminal::shm::ShmBuffer::open_existing(&shm_name, 4096) {
                Ok(shm) => {
                    println!("{}", shm.read_screen());
                }
                Err(e) => {
                    anyhow::bail!(
                        "Failed to open SHM for terminal {}: {}. Is it running?",
                        id,
                        e
                    );
                }
            }
        }

        Commands::Takeover { version } => {
            println!("Triggering takeover to version {}...", version);
            let res = client.execute(SkillCommand::Takeover { version }).await?;
            println!("Takeover status: {:?}", res.status);
            if let Some(content) = res.content {
                println!("Message: {}", content);
            }
        }

        Commands::Top => {
            let res = client
                .execute(SkillCommand::Inspect { assurance: true })
                .await?;
            println!("Orchestrator Resource Health:");
            if let Some(mem) = res.mem_usage_mb {
                println!("  Memory Usage: {} MB", mem);
            }
            if let Some(handles) = res.handle_count {
                println!("  Handle Count: {}", handles);
            }
            if let Some(stall) = res.stall_index {
                println!("  Stall Index:  {:.2}", stall);
            }
        }
    }

    Ok(())
}
