import vterm_python
import time

def reality_check_claude():
    print("Connecting to vterm orchestrator...")
    client = vterm_python.VTermClient()
    
    print("Spawning visible Claude terminal...")
    # Now we can explicitly request a visible window!
    tid = client.spawn("claude_reality_check", visible=True)
    
    print(f"Terminal {tid} spawned. Launching 'claude'...")
    client.write(tid, "claude<Enter>")
    
    print("Waiting for Claude TUI to appear (check your screen!)...")
    try:
        # Wait for "Claude" which appears in the TUI
        client.wait_until(tid, "Claude", 20000)
        print("Claude detected!")
    except Exception as e:
        print(f"Wait failed (maybe slow start?): {e}")
    
    print("Reading screen state...")
    screen = client.read(tid)
    
    print("\n--- CLAUDE SCREEN VIEW (UTF-8) ---")
    print(screen.encode('ascii', errors='replace').decode('ascii'))
    print("--------------------------")
    
    # We keep it open so the user can see it
    print("\nClaude is running in the visible window. GO LOOK AT IT!")
    print("Waiting 30 seconds for user observation...")
    time.sleep(30)
    
    client.close(tid)

if __name__ == "__main__":
    reality_check_claude()
