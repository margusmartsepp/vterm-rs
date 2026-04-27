import vterm_python
import asyncio

# rationale: Demonstrates multi-terminal coordination and fleet management.
# links: vterm_python.VTermClient.spawn, vterm_python.VTermClient.list

async def manage_terminal_fleet():
    """
    Demonstrates managing multiple terminal sessions simultaneously.
    Uses the 'Batch' API for high-performance atomic orchestration.
    """
    client = vterm_python.VTermClient()
    
    print("Launching Terminal Fleet via Batch...")
    
    # 1. Spawn the entire fleet in one atomic operation
    spawn_ops = [
        {"type": "Spawn", "payload": {"title": "fleet_builder", "visible": False}},
        {"type": "Spawn", "payload": {"title": "fleet_monitor", "visible": True}},
        {"type": "Spawn", "payload": {"title": "fleet_logs", "visible": False}},
    ]
    
    spawn_result = client.batch(spawn_ops)
    ids = [r["id"] for r in spawn_result["sub_results"]]
    tid_build, tid_monitor, tid_logs = ids
    
    print(f"Fleet active: Build({tid_build}), Monitor({tid_monitor}), Logs({tid_logs})")
    
    # 2. Parallelize interaction: Write and Wait in one go
    fleet_actions = [
        {"type": "ScreenWrite", "payload": {"id": tid_build, "text": "echo 'Building...'; Start-Sleep -s 5; echo 'DONE'<Enter>"}},
        {"type": "ScreenWrite", "payload": {"id": tid_monitor, "text": "echo 'I am the visible dashboard'<Enter>"}},
        {"type": "WaitUntil", "payload": {"id": tid_build, "pattern": "DONE", "timeout_ms": 10000}},
    ]
    
    print("Coordinating fleet states...")
    client.batch(fleet_actions)
    print(f"Build session {tid_build} reported completion.")
    
    # List all active sessions using the generic execute()
    active_sessions = client.execute({"type": "List", "payload": {}})
    print(f"\nActive Sessions: {active_sessions['content']}")
    
    # Cleanup fleet
    client.batch([
        {"type": "ScreenClose", "payload": {"id": tid_build}},
        {"type": "ScreenClose", "payload": {"id": tid_monitor}},
        {"type": "ScreenClose", "payload": {"id": tid_logs}},
    ])
    print("Fleet decommissioned.")

if __name__ == "__main__":
    # Note: VTermClient methods are currently synchronous in the Python bridge
    # but the orchestrator handles the terminals in parallel.
    asyncio.run(manage_terminal_fleet())
