import vterm_python
import time

def drive_claude_tui():
    client = vterm_python.VTermClient()
    
    print("Spawning persistent Claude session...")
    tid = client.spawn("claude_orchestration", visible=True)
    
    # 1. Start Claude
    client.write(tid, "claude<Enter>")
    
    # 2. Wait for the Trust Folder prompt
    print("Waiting for Security Handshake...")
    client.wait_until(tid, "trust this folder", 20000)
    
    # 3. Accept trust (Press 1 and Enter)
    print("Trusting folder...")
    client.write(tid, "1<Enter>")
    
    # 4. Wait for the main Claude prompt (usually looks like 'Claude >')
    print("Waiting for main prompt...")
    # Adjusting wait for common Claude Code prompt markers
    time.sleep(10) 
    
    # 5. Change the model
    # Note: In Claude Code, /model is the standard way to change it
    print("Changing model to 'opus'...")
    client.write(tid, "/model claude-3-opus-20240229<Enter>")
    
    # 6. Capture final state
    time.sleep(5)
    screen = client.read(tid)
    print("\n--- FINAL CLAUDE STATE ---")
    print(screen.encode('ascii', errors='replace').decode('ascii'))
    print("---------------------------")
    
    print("\nClaude model changed. You can now see the TUI in the visible window.")
    print("This script will keep the session alive for 2 minutes for you to use it.")
    time.sleep(120)

if __name__ == "__main__":
    drive_claude_tui()
