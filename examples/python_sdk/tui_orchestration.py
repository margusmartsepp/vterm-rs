import vterm_python
import time

# rationale: Demonstrates the 'Reason & Continue' loop for complex TUI orchestration.
# links: vterm_python.VTermClient.screen_read, vterm_python.VTermClient.wait_until

def orchestrate_claude_tui():
    """
    Demonstrates 'State Machine Inspection' using Atomic Batches.
    We bundle the setup and inspection into high-performance units.
    """
    client = vterm_python.VTermClient()
    
    print("Spawning and initializing TUI session via Batch...")
    # Atomic setup: Spawn, Write simulated TUI, and Wait for the visual state
    setup_ops = [
        client.spawn_op("claude_tui_session"),
        client.write_op(1, "echo 'CLAUDE TUI v1.0'; echo '[Press Enter to Start]'; echo '[q] to Quit'<Enter>"),
        client.wait_until_op(1, "[Press Enter to Start]", 5000)
    ]
    
    client.batch(setup_ops)
    
    # 1. REASON: Read the screen to see what's actually there
    res = client.execute(client.read_op(1))
    screen = res.get("content", "")
    
    if "[Press Enter to Start]" in screen:
        print("Detected Start button visually. Sending Enter...")
        # 2. INTERACT: Transition to next state atomically
        transition_ops = [
            client.write_op(1, "<Enter>"),
            client.write_op(1, "echo 'Main Menu:'; echo '1. Generate Code'; echo '2. Run Tests'; echo '> '<Enter>"),
            client.wait_until_op(1, "Main Menu:", 5000)
        ]
        client.batch(transition_ops)
    
    print("TUI is in Main Menu state. Agent can now reason and select options.")
    
    final_res = client.execute(client.read_op(1))
    print(f"\nFinal TUI View:\n{final_res.get('content')}")
    
    client.close(1)

if __name__ == "__main__":
    orchestrate_claude_tui()
