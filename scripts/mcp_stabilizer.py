import os
import time
import subprocess
import json
import sys

PIPE_NAME = r"\\.\pipe\vterm-rs-skill"

def kill_vterm():
    print("Stopping existing vterm processes...")
    subprocess.run(["taskkill", "/F", "/IM", "vterm.exe", "/T"], capture_output=True)
    time.sleep(1)

def start_vterm():
    vterm_path = os.path.join(os.getcwd(), "target", "release", "vterm.exe")
    if not os.path.exists(vterm_path):
        print(f"Error: {vterm_path} not found.")
        sys.exit(1)
    
    print(f"Starting {vterm_path} --headless...")
    # Use CREATE_NEW_CONSOLE to ensure it has its own life
    subprocess.Popen([vterm_path, "--headless"], 
                     creationflags=subprocess.CREATE_NEW_CONSOLE)

def wait_for_connection(timeout=10):
    print(f"Waiting for connection to orchestrator...")
    start_time = time.time()
    while time.time() - start_time < timeout:
        try:
            import vterm_python
            client = vterm_python.VTermClient()
            print("Connected successfully!")
            return True
        except:
            time.sleep(0.5)
    print("Timeout waiting for connection.")
    return False

def verify_health():
    print("Verifying orchestrator health...")
    try:
        import vterm_python
        client = vterm_python.VTermClient()
        info = client.get_info()
        print(f"Orchestrator health check OK: {info}")
        return True
    except Exception as e:
        print(f"Health check failed: {e}")
        return False

def main():
    kill_vterm()
    start_vterm()
    if wait_for_connection():
        # Give it a tiny bit more time to initialize the internal service
        time.sleep(1)
        if verify_health():
            print("STABILIZATION_SUCCESSFUL")
        else:
            print("STABILIZATION_FAILED_HEALTH_CHECK")
    else:
        print("STABILIZATION_FAILED_CONNECTION_TIMEOUT")

if __name__ == "__main__":
    main()
