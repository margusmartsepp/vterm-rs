#![cfg(windows)]
use anyhow::Result;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe;
use vterm_rs::{SkillCommand, SkillRequest, CommandResult, SpawnArgs, BatchArgs, Status, Response};
use std::time::Duration;
use tokio::time::sleep;

const PIPE_NAME: &str = r"\\.\pipe\vterm-rs-skill";

async fn connect_with_retry(retries: u32) -> Result<named_pipe::NamedPipeClient> {
    for i in 0..retries {
        match named_pipe::ClientOptions::new().open(PIPE_NAME) {
            Ok(client) => return Ok(client),
            Err(_) => {
                if i == 0 {
                    let _ = std::process::Command::new("cargo")
                        .args(["run", "--bin", "vterm", "--", "--headless"])
                        .spawn();
                }
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
    Err(anyhow::anyhow!("Failed to connect after {} retries", retries))
}

async fn send_request(writer: &mut (impl AsyncWriteExt + Unpin), req_id: u64, cmd: SkillCommand) -> Result<()> {
    let req = SkillRequest { req_id: Some(req_id), command: cmd, progress_token: None };
    let json = serde_json::to_string(&req)?;
    println!("  -> SEND: {}", json);
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

async fn read_response<R: AsyncBufRead + Unpin>(reader: &mut R, expected_id: u64) -> Result<CommandResult> {
    let mut line = String::new();
    loop {
        line.clear();
        reader.read_line(&mut line).await?;
        if line.is_empty() { return Err(anyhow::anyhow!("Connection closed")); }
        if let Ok(res) = serde_json::from_str::<Response>(&line) {
            if res.req_id == Some(expected_id) {
                return Ok(res.result);
            }
        }
    }
}

#[tokio::test]
async fn test_supreme_orchestration() -> Result<()> {
    let client = connect_with_retry(5).await?;
    let (reader, mut writer) = tokio::io::split(client);
    let mut buf_reader = BufReader::new(reader);

    // 0. Handshake
    println!("[HANDSHAKE]");
    send_request(&mut writer, 1, SkillCommand::Hello { client_version: "0.5.1".into() }).await?;
    let welcome = read_response(&mut buf_reader, 1).await?;
    assert_eq!(welcome.status, Status::Success);
    println!("  Server Version: {:?}", welcome.version);

    // 1. TEST 1: Ping Interruption
    println!("\n[TEST 1: Ping Interruption]");
    send_request(&mut writer, 2, SkillCommand::Spawn(SpawnArgs { 
        title: "Test 1: Ping".into(), 
        visible: Some(false),
        ..Default::default() 
    })).await?;
    let r_spawn = read_response(&mut buf_reader, 2).await?;
    let term_id = r_spawn.id.expect("Spawn should return ID");

    let playbook1 = SkillCommand::Batch(BatchArgs {
        commands: vec![
            SkillCommand::ScreenWrite { id: term_id, text: "ping google.com -n 2<Enter>".into() },
            SkillCommand::WaitUntil { id: term_id, pattern: "Ping statistics".into(), timeout_ms: 15000 },
            SkillCommand::ScreenRead { id: term_id, history: false },
        ],
        stop_on_error: Some(true),
        ..Default::default()
    });
    send_request(&mut writer, 3, playbook1).await?;
    let r_batch = read_response(&mut buf_reader, 3).await?;
    
    if r_batch.status != Status::Success {
        println!("  FAILED: {:?}", r_batch.error);
    }
    assert_eq!(r_batch.status, Status::Success);
    println!("  ASSERT: SUCCESS (Duration: {}ms)", r_batch.duration_ms);

    // 2. TEST 2: Arrow Key Recall
    println!("\n[TEST 2: Arrow Key Recall]");
    let playbook2 = SkillCommand::Batch(BatchArgs {
        commands: vec![
            SkillCommand::ScreenWrite { id: term_id, text: "whoami<Enter>".into() },
            SkillCommand::Wait { timeout_ms: 500 },
            SkillCommand::ScreenWrite { id: term_id, text: "<Up>".into() },
            SkillCommand::Wait { timeout_ms: 500 },
            SkillCommand::ScreenRead { id: term_id, history: false },
        ],
        ..Default::default()
    });
    send_request(&mut writer, 4, playbook2).await?;
    let r_batch2 = read_response(&mut buf_reader, 4).await?;
    assert_eq!(r_batch2.status, Status::Success);
    
    let sub_results = r_batch2.sub_results.expect("Batch should have sub_results");
    let screen_read = &sub_results[4];
    let content = screen_read.content.as_ref().expect("ScreenRead should have content");
    assert!(content.contains("whoami"));
    assert!(!content.contains("^[")); 
    println!("  ASSERT: SUCCESS");

    // 3. TEST 3: Destructor (Session Reaping)
    println!("\n[TEST 3: Destructor]");
    send_request(&mut writer, 5, SkillCommand::Spawn(SpawnArgs { 
        title: "Test: Destructor".into(), 
        timeout_ms: Some(2000), 
        visible: Some(false),
        ..Default::default() 
    })).await?;
    let r_spawn3 = read_response(&mut buf_reader, 5).await?;
    let new_id = r_spawn3.id.expect("Spawn should return ID");

    sleep(Duration::from_secs(4)).await;

    send_request(&mut writer, 6, SkillCommand::List {}).await?;
    let r_list = read_response(&mut buf_reader, 6).await?;
    let list_content = r_list.content.expect("List should have content");
    assert!(!list_content.contains(&new_id.to_string()));
    println!("  ASSERT: SUCCESS");

    Ok(())
}
