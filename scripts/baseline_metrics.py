import time
import subprocess
import os
import sys
import psutil

def measure_recovery_time():
    print("--- Measuring Recovery Time ---")
    vterm_path = os.path.join(os.getcwd(), "target", "debug", "vterm.exe")
    
    # 1. Ensure clean start
    subprocess.run(["taskkill", "/F", "/IM", "vterm.exe", "/T"], capture_output=True)
    time.sleep(2) # Wait for pipe release
    
    # 2. Start vterm
    print("Starting vterm...")
    proc = subprocess.Popen([vterm_path, "--headless"], creationflags=subprocess.CREATE_NEW_CONSOLE)
    
    # 3. Wait for it to be ready
    import vterm_python
    start = time.time()
    connected = False
    while time.time() - start < 10:
        try:
            client = vterm_python.VTermClient()
            connected = True
            break
        except:
            time.sleep(0.1)
    
    if not connected:
        print("Initial connection failed.")
        return None
    
    initial_ready_time = time.time() - start
    print(f"Initial ready time: {initial_ready_time:.2f}s")
    
    # 4. Kill it
    print("Killing vterm...")
    subprocess.run(["taskkill", "/F", "/PID", str(proc.pid), "/T"], capture_output=True)
    
    # 5. Measure recovery (time to new connection after restart)
    print("Restarting and measuring recovery...")
    start_restart = time.time()
    proc2 = subprocess.Popen([vterm_path, "--headless"], creationflags=subprocess.CREATE_NEW_CONSOLE)
    
    connected = False
    while time.time() - start_restart < 10:
        try:
            client = vterm_python.VTermClient()
            connected = True
            break
        except:
            time.sleep(0.1)
            
    recovery_time = time.time() - start_restart
    print(f"Recovery time: {recovery_time:.2f}s")
    
    # Cleanup
    subprocess.run(["taskkill", "/F", "/IM", "vterm.exe", "/T"], capture_output=True)
    return recovery_time

def measure_gemini_cli_success():
    print("\n--- Measuring Gemini-CLI Success Rate ---")
    successes = 0
    total = 5
    
    # Ensure vterm is running for the tests
    vterm_path = os.path.join(os.getcwd(), "target", "debug", "vterm.exe")
    subprocess.Popen([vterm_path, "--headless"], creationflags=subprocess.CREATE_NEW_CONSOLE)
    time.sleep(2)
    
    import vterm_python
    client = vterm_python.VTermClient()
    
    for i in range(total):
        print(f"Attempt {i+1}/{total}...")
        try:
            tid = client.spawn(f"Baseline Test {i}", visible=False)
            time.sleep(1)
            # Simple prompt to Gemini CLI
            client.write(tid, 'gemini chat "respond with exactly OK"\r')
            
            # Wait for response (current state: polling)
            start_wait = time.time()
            found = False
            while time.time() - start_wait < 15:
                content = client.read(tid)
                if "OK" in content:
                    found = True
                    break
                time.sleep(1)
            
            if found:
                print("  Success!")
                successes += 1
            else:
                print("  Failed (Timeout/Incomplete output)")
            
            client.close(tid)
        except Exception as e:
            print(f"  Error: {e}")
            
    rate = (successes / total) * 100
    print(f"Gemini-CLI Success Rate: {rate}% ({successes}/{total})")
    
    # Cleanup
    subprocess.run(["taskkill", "/F", "/IM", "vterm.exe", "/T"], capture_output=True)
    return rate

if __name__ == "__main__":
    recovery = measure_recovery_time()
    success_rate = measure_gemini_cli_success()
    
    print("\n--- Baseline Results ---")
    print(f"Recovery Time: {recovery:.2f}s" if recovery else "Recovery: FAILED")
    print(f"Gemini-CLI Success Rate: {success_rate}%")
