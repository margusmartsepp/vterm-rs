import json
import subprocess
import time
import os
import sys

# Paths
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
RUST_MCP = os.path.join(ROOT, "target", "release", "vterm-mcp.exe")
VTERM_EXE = os.path.join(ROOT, "target", "release", "vterm.exe")

def call_tool(tool_name, args):
    # Ensure orchestrator is running
    subprocess.run(["powershell", "-Command", os.path.join(ROOT, "scripts", "kill_vterm.bat")], capture_output=True)
    orch = subprocess.Popen([VTERM_EXE, "--headless"])
    time.sleep(2)
    
    proc = subprocess.Popen(
        [RUST_MCP], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, bufsize=1
    )
    
    def send(msg):
        proc.stdin.write(json.dumps(msg) + "\n")
        proc.stdin.flush()
        
    def recv():
        while True:
            line = proc.stdout.readline()
            if not line: 
                print("  [DEBUG] EOF reached", file=sys.stderr)
                return None
            print(f"  [DEBUG] RECV: {line.strip()}", file=sys.stderr)
            if line.strip().startswith("{"):
                try: return json.loads(line)
                except: pass

    try:
        # Handshake
        send({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "tester", "version": "1.0"}}
        })
        recv()
        send({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
        
        # Call
        send({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": tool_name, "arguments": args}})
        res = recv()
        return res
    finally:
        proc.terminate()
        orch.terminate()

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python mcp_doc_tool.py <name> <json_args>")
        sys.exit(1)
        
    name = sys.argv[1]
    args = json.loads(sys.argv[2])
    print(json.dumps(call_tool(name, args), indent=2))
