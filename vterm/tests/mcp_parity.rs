use std::time::Instant;
use vterm_rs::protocol::{BatchArgs, SkillCommand, SpawnArgs};
use vterm_rs::OrchestratorClient;

#[tokio::test]
async fn test_batch_extraction_parity() -> anyhow::Result<()> {
    println!("\n--- VTerm Rust Parallel Batch Extraction Parity ---");

    // Connect to the orchestrator (must be running)
    let client = OrchestratorClient::connect().await?;

    // Define a common pattern for pings (Windows style)
    let pattern =
        r"Reply from (?P<ip>[\d\.]+): bytes=(?P<bytes>\d+) time=(?P<time>\d+)ms TTL=(?P<ttl>\d+)";

    let commands = vec![
        SkillCommand::Spawn(SpawnArgs {
            title: "ping-local-1".into(),
            command: Some("ping 127.0.0.1 -n 1".into()),
            wait: Some(true),
            extract_pattern: Some(pattern.into()),
            ..Default::default()
        }),
        SkillCommand::Spawn(SpawnArgs {
            title: "ping-local-2".into(),
            command: Some("ping 127.0.0.1 -n 1".into()),
            wait: Some(true),
            extract_pattern: Some(pattern.into()),
            ..Default::default()
        }),
    ];

    let start = Instant::now();

    // Execute batch in parallel
    let res = client
        .execute(SkillCommand::Batch(BatchArgs {
            commands,
            parallel: Some(true),
            ..Default::default()
        }))
        .await?;

    let duration = start.elapsed();
    println!("Total Batch Time: {:.2?}ms", duration.as_millis());

    // Each sub-result contains its own extracted data
    if let Some(sub_results) = res.sub_results {
        for (i, sub) in sub_results.iter().enumerate() {
            let extracted = sub.extracted.as_ref().map(|e| e.len()).unwrap_or(0);
            println!("\n[Term {}] - Extracted {} matches", i, extracted);
            if let Some(matches) = &sub.extracted {
                if !matches.is_empty() {
                    println!("Sample: {:?}", matches[0]);
                }
            }
        }
        assert_eq!(sub_results.len(), 2);
    } else {
        panic!("Expected sub_results");
    }

    Ok(())
}

#[tokio::test]
async fn test_inspect_parity() -> anyhow::Result<()> {
    let client = OrchestratorClient::connect().await?;
    let res = client
        .execute(SkillCommand::Inspect { assurance: true })
        .await?;

    assert!(res.mem_usage_mb.is_some());
    assert!(res.active_terminals.is_some());
    println!(
        "Inspect: Mem Usage: {}MB, Active: {}",
        res.mem_usage_mb.unwrap(),
        res.active_terminals.unwrap()
    );

    Ok(())
}

#[tokio::test]
async fn test_stability_parity() -> anyhow::Result<()> {
    let client = OrchestratorClient::connect().await?;

    // Spawn a terminal that does something slow
    let spawn_res = client
        .execute(SkillCommand::Spawn(SpawnArgs {
            title: "stability-test".into(),
            command: Some("cmd /c \"echo Starting... && timeout /t 2 > nul && echo Done!\"".into()),
            ..Default::default()
        }))
        .await?;

    let id = spawn_res.id.expect("ID expected");

    // Wait until stable (should take at least 2 seconds)
    let start = Instant::now();
    client
        .execute(SkillCommand::WaitUntilStable {
            id,
            stable_ms: 500,
            timeout_ms: 30000,
        })
        .await?;

    let elapsed = start.elapsed();
    println!("Stabilized in: {:.2?}ms", elapsed.as_millis());
    assert!(elapsed.as_millis() >= 500);

    Ok(())
}
