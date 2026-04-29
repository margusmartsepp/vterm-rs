use std::time::Duration;
use tokio::time::sleep;
use vterm_rs::protocol::{SkillCommand, SpawnArgs};
use vterm_rs::OrchestratorClient;

#[tokio::test]
async fn test_memory_leak_candidate() -> anyhow::Result<()> {
    println!("\n--- VTerm Memory Leak Stress Test ---");

    // Connect to the orchestrator (must be running)
    let client = OrchestratorClient::connect().await?;

    // 1. Get baseline memory
    let base_res = client
        .execute(SkillCommand::Inspect { assurance: true })
        .await?;
    let base_mem = base_res.mem_usage_mb.unwrap_or(0);
    println!("Baseline Memory: {}MB", base_mem);

    // 2. Stress: Spawn and Close 20 terminals in a loop
    println!("Spawning and closing 20 terminals...");
    for i in 0..20 {
        let spawn_res = client
            .execute(SkillCommand::Spawn(SpawnArgs {
                title: format!("leak-test-{}", i),
                command: Some("echo hello".into()),
                wait: Some(true),
                ..Default::default()
            }))
            .await?;

        let id = spawn_res.id.expect("ID expected");
        client
            .execute(SkillCommand::ScreenClose {
                id: Some(id),
                target: "single".into(),
            })
            .await?;

        if i % 5 == 0 {
            print!(".");
        }
    }
    println!(" Done.");

    // 3. Wait for cleanup (watchdog and OS reaping)
    println!("Waiting for cleanup (5s)...");
    sleep(Duration::from_secs(5)).await;

    // 4. Measure peak memory (during another cycle)
    println!("Spawning 10 terminals simultaneously...");
    let mut ids = Vec::new();
    for i in 0..10 {
        let spawn_res = client
            .execute(SkillCommand::Spawn(SpawnArgs {
                title: format!("leak-test-batch-{}", i),
                ..Default::default()
            }))
            .await?;
        ids.push(spawn_res.id.expect("ID expected"));
    }

    let mid_res = client
        .execute(SkillCommand::Inspect { assurance: true })
        .await?;
    let mid_mem = mid_res.mem_usage_mb.unwrap_or(0);
    println!("Mid-point Memory (10 active): {}MB", mid_mem);

    // Close batch
    for id in ids {
        client
            .execute(SkillCommand::ScreenClose {
                id: Some(id),
                target: "single".into(),
            })
            .await?;
    }

    // 5. Final wait and check
    println!("Waiting for final cleanup (10s)...");
    sleep(Duration::from_secs(10)).await;

    let final_res = client
        .execute(SkillCommand::Inspect { assurance: true })
        .await?;
    let final_mem = final_res.mem_usage_mb.unwrap_or(0);
    println!("Final Memory: {}MB", final_mem);

    // Heuristic: Memory shouldn't be significantly higher than baseline + reasonable overhead
    // We allow some growth for the pool and fragmentation, but not 2x.
    let diff = if final_mem > base_mem {
        final_mem - base_mem
    } else {
        0
    };
    println!("Memory Delta: {}MB", diff);

    if diff > 50 {
        return Err(anyhow::anyhow!(
            "Potential memory leak detected! Delta: {}MB",
            diff
        ));
    }

    println!("ASSERT: SUCCESS (Memory stabilized)");
    Ok(())
}
