import json
import subprocess
import time
import os
import sys
import re

# Paths
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
# Try release first, fallback to debug
RUST_MCP = os.path.join(ROOT, "target", "release", "vterm-mcp.exe")
if not os.path.exists(RUST_MCP):
    RUST_MCP = os.path.join(ROOT, "target", "debug", "vterm-mcp.exe")

PYTHON_MCP = ["uv", "run", "python", "-m", "vterm_python.server"]

VTERM_EXE = os.path.join(ROOT, "target", "release", "vterm.exe")
if not os.path.exists(VTERM_EXE):
    VTERM_EXE = os.path.join(ROOT, "target", "debug", "vterm.exe")

DOC_DIR = os.path.join(ROOT, "docs", "mcp")

REASONING = {
    "spawn": [
        "Task Isolation: Spawn a dedicated terminal for every major task (e.g., one for 'Server Logs', one for 'Build Commands', one for 'Git Operations'). This prevents output interleaving and makes state verification easier.",
        "Resource Guardrails: Use max_lines and timeout_ms to prevent an agent from accidentally triggering an infinite loop of output that would exhaust tokens or crash the host.",
        "Environment Setup: Use the env argument to set up specific project contexts (e.g., DATABASE_URL, NODE_ENV) without affecting the global machine state."
    ],
    "close": [
        "Resource Management: Always close terminals when a task is finished (e.g., after a build or a long-running log tail) to free up system memory and PTY slots.",
        "Session Isolation: Close terminals between unrelated tasks to ensure no environmental leakage (e.g., dirty env vars or background processes) persists.",
        "Admission Control: If the orchestrator reaches max_terminals, the agent should identify and close idle sessions."
    ],
    "list_terminals": [
        "Self-Discovery: If an agent is restarted or loses track of its state, it can use list_terminals to recover IDs of active sessions it previously spawned.",
        "Health Monitoring: Periodically checking the list ensures that spawned processes haven't crashed unexpectedly.",
        "Cleanup Playbooks: An agent can use this to identify 'zombie' terminals that should be closed."
    ],
    "read": [
        "Visual Verification: The primary way to 'see' what the terminal is doing. Agents should read after a write to confirm the command was typed correctly and to see the initial response.",
        "TUI Interaction: Essential for interacting with Text User Interfaces (like htop, vim, or custom CLI menus) where the current state is not just a sequence of lines but a 2D grid.",
        "Debugging: If a wait_until timeout occurs, the agent should read the screen to understand what prevented the pattern from appearing (e.g., an unexpected password prompt or an error message)."
    ],
    "write": [
        "Command Execution: The fundamental way to run commands. Always include <Enter> if you want the shell to execute the line.",
        "TUI Navigation: Use arrow keys and <Tab> to navigate menus in tools like htop, vim, or git log.",
        "Interrupting Processes: Use <C-c> to stop long-running commands (like ping or tail) before starting a new task.",
        "Interactive Prompts: Respond to y/n prompts or enter passwords (though use with caution as output might be echoed)."
    ],
    "wait_until": [
        "Synchronization: The most reliable way to know when a command has finished and it's safe to send the next one (e.g., waiting for the shell prompt PS C:\\>).",
        "Error Detection: Use wait_until to catch specific error strings (e.g., Error: Build failed) as soon as they appear, rather than waiting for a full timeout.",
        "Boot Verification: When starting a service (like a web server), use wait_until to watch for the 'Listening on port XXXX' message before proceeding with tests."
    ],
    "wait_until_stable": [
        "Slow Renders: Perfect for waiting for tools that don't produce a clear 'Done' signal but redraw the screen frequently (e.g., npm install progress bars or top updates).",
        "Prompt Detection Fallback: If the prompt regex is unreliable, waiting for the screen to stabilize is a solid fallback to ensure the command has stopped spitting out text.",
        "Visual Stability: Ensures that a read or screen_diff captured immediately after will represent the final state of the operation."
    ],
    "extract": [
        "Post-Command Verification: After running a command (like git status), use extract to pull specific information (e.g., branch name, modified files) into the agent's memory as structured JSON.",
        "Log Scraping: Search the scrollback (history: true) for specific error codes or trace IDs without re-reading the entire terminal buffer.",
        "Dynamic Variable Injection: Extract a value (like a PID or a generated API key) and use it in a subsequent command within the same batch."
    ],
    "screen_diff": [
        "Live Log Monitoring: Use screen_diff in a loop to tail logs without re-sending the entire scrollback every time.",
        "Progress Tracking: Efficiently check if a long-running process (like npm install or cargo build) is still making progress.",
        "Token Optimization: Minimizes context window usage by only feeding 'what changed' to the LLM."
    ],
    "batch": [
        "Latency Minimization: For workflows requiring multiple round-trips (spawn -> write -> wait -> extract), batch eliminates IPC overhead, reducing total execution time.",
        "Atomic Rollback: If stop_on_error is set, the orchestrator stops at the first failure, preventing operations on an invalid state.",
        "Workflow Snapshots: Perfect for 'Playbooks' — predefined sequences of commands that the agent can trigger with a single tool call."
    ],
    "rust_eval": [
        "Complex Logic: When a task requires math or data manipulation that is awkward in shell, use rust_eval.",
        "System Probing: Use Rust's standard library to check file permissions, network availability, or system entropy in a structured way.",
        "Performance: For CPU-intensive tasks, compiled Rust in the REPL will be significantly faster than interpreted Python or shell scripts."
    ],
    "get_process_state": [
        "Termination Detection: Instead of relying on visual cues, get_process_state provides the definitive OS-level truth of whether a process has exited.",
        "Error Handling: Checking the exit_code allows the agent to distinguish between a successful run (0) and a failure (non-zero).",
        "Hanging Diagnostics: If a terminal is unresponsive, the agent can check if the process is still running or if it has crashed silently."
    ],
    "get_architecture": [
        "Self-Onboarding: An agent entering a new repo can use this to understand how the PTY orchestrator works without manually reading every file.",
        "Contextual Debugging: When an error occurs in a specific module, the agent can look up the architecture to understand the surrounding dependencies.",
        "Protocol Verification: Ensures the agent is interacting with the correct version and layer of the system."
    ]
}

def call_mcp(proc, method, params, req_id):
    req = {"jsonrpc": "2.0", "id": req_id, "method": method, "params": params}
    try:
        proc.stdin.write(json.dumps(req) + "\n")
        proc.stdin.flush()
    except Exception as e:
        print(f"  [ERROR] Writing to stdin: {e}")
        # Try to read stderr
        stderr = proc.stderr.read() if proc.stderr else "N/A"
        print(f"  [STDERR] {stderr}")
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
    if not os.path.exists(cmd_args[0]) and "python" not in cmd_args[0] and "uv" not in cmd_args[0]:
        print(f"  [SKIP] {cmd_args[0]} not found")
        return {}

    proc = subprocess.Popen(
        cmd_args, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, bufsize=1
    )

    # 1. Handshake
    init_res = call_mcp(proc, "initialize", {
        "protocolVersion": "2024-11-05", "capabilities": {},
        "clientInfo": {"name": "doc-gen", "version": "1.0"}
    }, 1)
    if not init_res:
        print(f"  [ERROR] Initialize failed for {name}")
        proc.terminate()
        return {}
        
    proc.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}) + "\n")
    proc.stdin.flush()
    time.sleep(0.5)

    # 2. List Tools
    tools_res = call_mcp(proc, "tools/list", {}, 2)
    if not tools_res: 
        print(f"  [ERROR] tools/list failed for {name}")
        proc.terminate()
        return {}
        
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
            args = {"commands": [{"type": "Inspect", "payload": {"assurance": True}}]}
        elif tname == "get_info" or tname == "list_terminals" or tname == "get_architecture":
            args = {}
        elif tname == "rust_eval":
            args = {"code": "println!(\"Hello from Rust REPL\");"}
        elif term_id is not None:
            if tname in ["read", "write", "close", "get_process_state", "screen_diff"]:
                args = {"id": term_id}
                if tname == "write": args["text"] = "echo 123<Enter>"
            elif tname == "wait_until":
                args = {"id": term_id, "pattern": "echo", "timeout_ms": 5000}
            elif tname == "wait_until_stable":
                args = {"id": term_id, "stable_ms": 500, "timeout_ms": 5000}
            elif tname == "extract":
                args = {"id": term_id, "pattern": "(?P<val>\\d+)", "history": True}
        else:
            # Cannot test stateful tools without a terminal ID
            continue
        
        res = call_mcp(proc, "tools/call", {"name": tname, "arguments": args}, 100)
        
        if tname == "spawn" and res and "result" in res:
             try:
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
    
    # Check if we should preserve manually added Reasoning
    existing_reasoning = ""
    if os.path.exists(filename):
        with open(filename, "r", encoding="utf-8") as f:
            content = f.read()
            match = re.search(r"## Agent Reasoning & Use Cases.*", content, re.DOTALL)
            if match:
                existing_reasoning = match.group(0)

    content = f"# Tool: `{tool_name}`\n\n"
    content += f"{data['description']}\n\n"
    
    content += "## Metadata\n"
    content += "- **Status**: Stable\n"
    content += "- **Rust Endpoint**: `vterm-mcp`\n"
    content += "- **Python Endpoint**: `vterm_python.server`\n\n"

    content += "## Example Tool Call\n\n"
    content += "```json\n"
    call_json = {
        "name": tool_name,
        "arguments": data['input']
    }
    content += json.dumps(call_json, indent=2) + "\n```\n\n"
    
    content += "## Verified Output\n\n"
    output = data['output']
    if isinstance(output, list) and len(output) > 0 and 'text' in output[0]:
        text = output[0]['text']
        if text.strip().startswith("{"):
            try:
                formatted = json.dumps(json.loads(text), indent=2)
                content += "```json\n" + formatted + "\n```\n\n"
            except:
                content += "```text\n" + text + "\n```\n\n"
        else:
            content += "```text\n" + text + "\n```\n\n"
    elif tool_name == "get_architecture":
        # Special handling for get_architecture: it's a markdown report
        # We need to fix encoding issues like â‰¤ (<=)
        text = ""
        if isinstance(output, list) and len(output) > 0 and 'text' in output[0]:
            text = output[0]['text']
        else:
            text = str(output)
        
        text = text.replace("â‰¤", "≤")
        content += text + "\n"
    else:
        content += "```json\n" + (json.dumps(output, indent=2) if isinstance(output, (dict, list)) else str(output)) + "\n```\n\n"
    
    if tool_name in REASONING:
        content += "## Agent Reasoning & Use Cases\n\n"
        for r in REASONING[tool_name]:
            content += f"- **{r.split(':')[0]}**: {r.split(':', 1)[1].strip()}\n"
        content += "\n"
    elif existing_reasoning:
        content += existing_reasoning
    
    with open(filename, "w", encoding="utf-8") as f:
        f.write(content)

def main():
    os.makedirs(DOC_DIR, exist_ok=True)
    subprocess.run(["powershell", "-Command", os.path.join(ROOT, "scripts", "kill_vterm.bat")], capture_output=True)
    
    print(f"Using orchestrator: {VTERM_EXE}")
    orch = subprocess.Popen([VTERM_EXE, "--headless"])
    time.sleep(3)

    try:
        # print(f"Using Rust MCP: {RUST_MCP}")
        # rust_results = test_server("Rust MCP", [RUST_MCP])
        
        print(f"Using Python MCP: {PYTHON_MCP}")
        py_results = test_server("Python MCP", PYTHON_MCP)
        
        all_results = py_results # prioritising Python for now
        
        for name, data in all_results.items():
            write_doc_file(name, data)
            
        print(f"\nSUCCESS: {len(all_results)} tool docs generated in docs/mcp/")
    finally:
        orch.terminate()

if __name__ == "__main__":
    main()
