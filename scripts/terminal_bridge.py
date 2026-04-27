import vterm_python
import sys
import json

def terminal_bridge():
    client = vterm_python.VTermClient()
    tid = None
    
    print("BRIDGE_READY")
    sys.stdout.flush()
    
    for line in sys.stdin:
        try:
            cmd = json.loads(line)
            action = cmd.get("action")
            
            if action == "spawn":
                tid = client.spawn(cmd["title"], visible=True)
                print(json.dumps({"status": "ok", "tid": tid}))
            elif action == "write":
                client.write(tid, cmd["text"])
                print(json.dumps({"status": "ok"}))
            elif action == "read":
                screen = client.read(tid)
                print(json.dumps({"status": "ok", "screen": screen}))
            elif action == "wait":
                client.wait_until(tid, cmd["pattern"], cmd.get("timeout", 10000))
                print(json.dumps({"status": "ok"}))
            elif action == "exit":
                break
            
            sys.stdout.flush()
        except Exception as e:
            print(json.dumps({"status": "error", "message": str(e)}))
            sys.stdout.flush()

if __name__ == "__main__":
    terminal_bridge()
