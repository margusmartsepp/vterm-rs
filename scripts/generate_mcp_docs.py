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
DOC_DIR = os.path.join(ROOT, "docs", "mcp")

def call_mcp(proc, method, params, req_id):
    req = {"jsonrpc": "2.0", "id": req_id, "method": method, "params": params}
    try:
        proc.stdin.write(json.dumps(req) + "\n")
        proc.stdin.flush()
    except Exception as e:
        print(f"  [ERROR] Writing to stdin: {e}")
        return None

    while True:
        line = proc.stdout.readline()
        if not line: return None
        stripped = line.strip()
        if stripped.startswith("{"):
            try: return json.loads(stripped)
            except: pass

def test_server(name, cmd_args):
    print(f"\n--- Verifying {name} ---")
    proc = subprocess.Popen(
        cmd_args, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, bufsize=1
    )

    # 1. Handshake
    call_mcp(proc, "initialize", {
        "protocolVersion": "2024-11-05", "capabilities": {},
        "clientInfo": {"name": "doc-gen", "version": "1.0"}
    }, 1)
    proc.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}) + "\n")
    proc.stdin.flush()
    time.sleep(1)

    # 2. List Tools
    tools_res = call_mcp(proc, "tools/list", {}, 2)
    tools = tools_res.get("result", {}).get("tools", [])
    
    results = {}
    term_id = None
    
    for tool in tools:
        tname = tool["name"]
        print(f"[{name}]   -> {tname}")
        
        args = {}
        if tname == "spawn":
            args = {"title": f"doc-gen-{name}", "command": "echo 'verified'"}
        elif tname == "batch":
            args = {"commands": [{"type": "Inspect"}]}
        elif term_id is not None:
            if tname in ["read", "write", "close", "get_process_state"]:
                args = {"id": term_id}
                if tname == "write": args["text"] = "whoami<Enter>"
        else:
            if tname in ["read", "write", "close", "get_process_state", "wait_until", "wait_until_stable", "screen_diff", "extract"]:
                continue
        
        res = call_mcp(proc, "tools/call", {"name": tname, "arguments": args}, 100)
        
        if tname == "spawn" and res and "result" in res:
             try:
                # Handle both Rust and Python output formats
                content = res['result']['content']
                text = content[0]['text'] if isinstance(content, list) else content
                if text.strip().startswith("{"):
                    data = json.loads(text)
                    term_id = data.get("id")
             except: pass

        results[tname] = {
            "description": tool["description"],
            "input": args,
            "output": res.get("result", {}).get("content", "N/A") if res else "ERROR"
        }

    proc.terminate()
    return results

def write_doc_file(tool_name, data):
    filename = os.path.join(DOC_DIR, f"{tool_name}.md")
    content = f"# Tool: `{tool_name}`\n\n"
    content += f"{data['description']}\n\n"
    content += "## Example Input\n```json\n" + json.dumps(data['input'], indent=2) + "\n```\n\n"
    content += "## Verified Output\n```json\n" + (json.dumps(data['output'], indent=2) if isinstance(data['output'], (dict, list)) else str(data['output'])) + "\n```\n"
    
    with open(filename, "w", encoding="utf-8") as f:
        f.write(content)

def main():
    os.makedirs(DOC_DIR, exist_ok=True)
    subprocess.run(["powershell", "-Command", os.path.join(ROOT, "scripts", "kill_vterm.bat")], capture_output=True)
    orch = subprocess.Popen([VTERM_EXE, "--headless"])
    time.sleep(3)

    try:
        rust_results = test_server("Rust MCP", [RUST_MCP])
        py_results = test_server("Python MCP", PYTHON_MCP)
        
        # Merge results, prioritize Rust for the doc content if overlapping
        all_results = {**py_results, **rust_results}
        
        for name, data in all_results.items():
            write_doc_file(name, data)
            
        print(f"SUCCESS: {len(all_results)} tool docs generated in docs/mcp/")
    finally:
        orch.terminate()

if __name__ == "__main__":
    main()
