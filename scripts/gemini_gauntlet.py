import time
import sys
import os

# Add local vterm_python to path
sys.path.append(os.path.abspath("vterm-python/target/debug"))

import vterm_python

def run_gemini_gauntlet():
    print("--- Gemini-CLI Gauntlet v1.0 ---")
    try:
        client = vterm_python.VTermClient()
    except Exception as e:
        print(f"FAILED: Could not connect to vterm-rs: {e}")
        return

    # 1. Provision Instance
    tid = client.spawn("Gemini Gauntlet", visible=True)
    print(f"Spawned Terminal {tid}")
    
    # 2. Start Gemini CLI
    client.write(tid, "gemini chat\r")
    time.sleep(2)
    
    # 3. Multi-turn Interaction
    prompts = [
        "What is the capital of Estonia?",
        "Write a simple rust function to add two numbers.",
        "How do I use this in a cargo project?"
    ]
    
    for prompt in prompts:
        print(f"Sending: {prompt}")
        client.write(tid, f"{prompt}\r")
        
        # Use v1.0 WaitUntilStable
        print("Waiting for stability...")
        try:
            # We wait for the screen to be stable for 500ms
            res = client.wait_until_stable(tid, stable_ms=500, timeout_ms=15000)
            print("Response stable. Reading output...")
            # print(res)
        except Exception as e:
            print(f"Stall detected or timeout: {e}")
            
    print("Gauntlet complete.")
    client.close(tid)

if __name__ == "__main__":
    run_gemini_gauntlet()
