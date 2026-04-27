import vterm_python
import time

# rationale: Demonstrates the 'Brick-Proof' guardrails for safe agentic execution.
# links: vterm_python.VTermClient.spawn, vterm_python.VTermClient.wait_until

def run_risky_build():
    """
    Demonstrates the 'Brick-Proof' guardrails of vterm-rs.
    Uses the 'Batch' API to atomically set up and monitor the build.
    """
    client = vterm_python.VTermClient()
    
    print("Executing Guardrailed Build via Atomic Batch...")
    
    # 1. Prepare the 'Rogue' command
    risky_command = "Write-Host 'Starting...'; for ($i=1; $i -le 1000; $i++) { Write-Host \"Log $i\" }; echo 'BUILD SUCCESSFUL'<Enter>"
    
    # 2. Bundle Spawn, Write, and Wait into one request
    ops = [
        {
            "type": "Spawn", 
            "payload": {
                "title": "risky_build", 
                "max_lines": 200,   # GUARDRAIL: Anti-Log Flood
                "timeout_ms": 60000 # GUARDRAIL: Anti-Hang
            }
        },
        {"type": "ScreenWrite", "payload": {"id": 1, "text": risky_command}},
        {"type": "WaitUntil", "payload": {"id": 1, "pattern": "BUILD SUCCESSFUL", "timeout_ms": 10000}},
    ]
    
    try:
        result = client.batch(ops)
        print("Batch execution finished.")
        
        # 3. Inspect results
        # The 3rd operation was WaitUntil, let's see if it succeeded
        wait_res = result["sub_results"][2]
        if wait_res["status"] == "success":
            print("Build Success!")
    except Exception as e:
        print(f"Guardrail or Timeout triggered: {e}")
        
    # Inspect final screen using the low-level execute()
    read_res = client.execute({"type": "ScreenRead", "payload": {"id": 1}})
    screen = read_res.get("content", "")
    print(f"\nFinal Screen State (truncated by guardrail):\n{screen[:500]}...")
    
    client.close(1)

if __name__ == "__main__":
    run_risky_build()
