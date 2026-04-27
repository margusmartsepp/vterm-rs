use anyhow::Result;
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use tokio::time::{sleep, Duration};
use crate::client::PIPE_NAME;

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
