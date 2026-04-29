import time
import sys
import os

# Add local vterm_python to path
sys.path.append(os.path.abspath("vterm-python/target/debug"))

import vterm_python

def main():
    client = vterm_python.VTermClient()
    
    print("--- VTerm North Star: Agentic Efficiency ---")
    print("Scaling to 4 parallel terminals and scheduling batched inputs...")
    
    # 1. Spawn 4 terminals
    ids = []
    for i in range(4):
        tid = client.spawn(f"Worker {i}", visible=False)
        ids.append(tid)
        print(f"  [+] Spawned Worker {i} (TID {tid})")
        
    # 2. Schedule Batched Inputs
    # We send commands to all terminals in a single batch to reduce tool calls
    batch = []
    tasks = [
        "dir c:\\",
        "systeminfo",
        "whoami",
        "echo 'Hello from VTerm v1.0'"
    ]
    
    for i, tid in enumerate(ids):
        # We use the low-level execute_op style if we had it, 
        # but for now we just use the write method in a loop for the demo
        # or better, use the batch() method if we can construct the dicts.
        cmd = {"type": "ScreenWrite", "payload": {"id": tid, "text": f"{tasks[i]}\r"}}
        batch.append(cmd)
        
    print(f"Executing batch of {len(batch)} commands...")
    client.batch(batch, parallel=True)
    
    # 3. Monitor with ScreenDiff (Ultra-low latency)
    print("\nMonitoring results with ScreenDiff:")
    for _ in range(5):
        time.sleep(1)
        for i, tid in enumerate(ids):
            diff = client.screen_diff(tid)
            if diff:
                print(f"--- [TID {tid}] Delta ---")
                # Print only first 3 lines of delta for brevity
                lines = diff.splitlines()[:3]
                for l in lines:
                    print(f"  {l}")
                if len(lines) > 3:
                    print("  ...")

    # Cleanup
    for tid in ids:
        client.close(tid)
    print("\nNorth Star Demo Complete.")

if __name__ == "__main__":
    main()
