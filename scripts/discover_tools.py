import json
import subprocess
import time
import os
import sys

# Paths
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
RUST_MCP = os.path.join(ROOT, "target", "release", "vterm-mcp.exe")
PYTHON_MCP = ["uv", "run", "python", "-m", "vterm_python.server"]
VTERM_EXE = os.path.join(ROOT, "target", "release", "vterm.exe")

def call_mcp(proc, method, params, req_id):
    req = {"jsonrpc": "2.0", "id": req_id, "method": method, "params": params}
    proc.stdin.write(json.dumps(req) + "\n")
    proc.stdin.flush()
    
    # Drain until we find a JSON line
    while True:
        line = proc.stdout.readline()
        if not line: return None
        stripped = line.strip()
        if stripped.startswith("{"):
            try: return json.loads(stripped)
            except: pass

def test_mcp(name, args):
    print(f"--- Testing {name} ---")
    proc = subprocess.Popen(
        args, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, bufsize=1
    )
    
    try:
        # 1. Handshake
        init = call_mcp(proc, "initialize", {
            "protocolVersion": "2024-11-05", "capabilities": {},
            "clientInfo": {"name": "tester", "version": "1.0"}
        }, 1)
        if not init: return f"FAILED: No init response for {name}"
        
        proc.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}) + "\n")
        proc.stdin.flush()
        
        # 2. List
        list_res = call_mcp(proc, "tools/list", {}, 2)
        if not list_res: return f"FAILED: No tool list for {name}"
        
        tools = list_res.get("result", {}).get("tools", [])
        return [t["name"] for t in tools]
    finally:
        proc.terminate()

if __name__ == "__main__":
    # Start orch
    orch = subprocess.Popen([VTERM_EXE, "--headless"])
    time.sleep(2)
    try:
        rust_tools = test_mcp("Rust", [RUST_MCP])
        py_tools = test_mcp("Python", PYTHON_MCP)
        print(f"Rust Tools: {rust_tools}")
        print(f"Python Tools: {py_tools}")
    finally:
        orch.terminate()
