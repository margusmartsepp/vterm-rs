use crate::client::PIPE_NAME;
use anyhow::Result;
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use tokio::time::{sleep, Duration};

pub async fn try_connect() -> Result<NamedPipeClient> {
    for i in 0..10 {
        if let Ok(client) = ClientOptions::new().open(PIPE_NAME) {
            return Ok(client);
        }
        if i == 0 {
            // Auto-spawn headless orchestrator if not found in PATH or local dir.
            let _ = std::process::Command::new("vterm.exe")
                .arg("--headless")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
        sleep(Duration::from_millis(1000)).await;
    }
    anyhow::bail!("Failed to connect to vterm.exe orchestrator")
}
