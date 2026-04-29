from fastmcp import FastMCP
import vterm_python
import os
import sys

# Create the MCP server
# Create the MCP server
os.environ["FASTMCP_LOG_LEVEL"] = "WARN"
mcp = FastMCP("vterm")

_client = None

def get_client():
    """Lazy-initializes the vterm client. Returns None if orchestrator is missing."""
    global _client
    if _client is None:
        try:
            _client = vterm_python.VTermClient()
        except Exception as e:
            # Return None instead of crashing the whole server
            print(f"Warning: Failed to connect to vterm orchestrator: {e}", file=sys.stderr)
            return None
    return _client

def tool_error():
    return "ERROR: Orchestrator not running. Please start vterm.exe manually (e.g., run 'vterm --headless' in a terminal)."

@mcp.tool()
def get_info() -> dict:
    """Returns version and build metadata for this vterm-rs instance."""
    client = get_client()
    if not client: return {"error": tool_error()}
    return client.get_info()

@mcp.tool()
def list_terminals() -> list:
    """Returns a list of all active terminal IDs and their titles."""
    client = get_client()
    if not client: return []
    res = client.execute({"type": "List", "payload": {}})
    import json
    try:
        return json.loads(res.get("content", "[]"))
    except:
        return []

@mcp.tool()
def spawn(title: str, command: str = None, wait: bool = False, semantic: bool = False, extract_pattern: str = None, max_lines: int = 1000, visible: bool = False, cols: int = 80, rows: int = 24, env: dict = None) -> dict:
    """Spawns a new terminal session. Returns the terminal ID and any extracted data."""
    client = get_client()
    if not client: return {"error": tool_error()}
    
    # Use execute() for robust argument passing
    res = client.execute({
        "type": "Spawn",
        "payload": {
            "title": title,
            "command": command,
            "wait": wait,
            "semantic": semantic,
            "extract_pattern": extract_pattern,
            "max_lines": max_lines,
            "visible": visible,
            "cols": cols,
            "rows": rows,
            "env": env
        }
    })
    
    if res.get("status") == "error":
        return {"error": f"Spawn failed: {res.get('error')}"}
    return res

@mcp.tool()
def wait_until(id: int, pattern: str, timeout_ms: int = 10000) -> str:
    """Blocks until a regex pattern appears on the terminal screen."""
    client = get_client()
    if not client: return tool_error()
    res = client.execute({
        "type": "WaitUntil",
        "payload": {
            "id": id,
            "pattern": pattern,
            "timeout_ms": timeout_ms
        }
    })
    if res.get("status") == "error":
        return f"ERROR: WaitUntil failed: {res.get('error')}"
    return res.get("content", "Pattern found")

@mcp.tool()
def get_process_state(id: int) -> dict:
    """Returns the current process state (running: bool, exit_code: int?)."""
    client = get_client()
    if not client: return {"error": tool_error()}
    return client.get_process_state(id)

@mcp.tool()
def write(id: int, text: str) -> str:
    """Writes text to a terminal. Supports <Enter>, <C-c>, etc."""
    client = get_client()
    if not client: return tool_error()
    res = client.execute({
        "type": "ScreenWrite",
        "payload": {
            "id": id,
            "text": text
        }
    })
    if res.get("status") == "error":
        return f"ERROR: Write failed: {res.get('error')}"
    return "OK"

@mcp.tool()
def read(id: int, history: bool = False) -> str:
    """Reads the current contents of the terminal screen. Set history=True for full scrollback."""
    client = get_client()
    if not client: return tool_error()
    res = client.execute({
        "type": "ScreenRead",
        "payload": {
            "id": id,
            "history": history
        }
    })
    if res.get("status") == "error":
        return f"ERROR: Read failed: {res.get('error')}"
    return res.get("content", "")

@mcp.tool()
def batch(commands: list) -> dict:
    """Executes a batch of commands atomically for high performance."""
    client = get_client()
    if not client: return {"error": tool_error()}
    return client.batch(commands)

@mcp.tool()
def close(id: int) -> str:
    """Closes a terminal session."""
    client = get_client()
    if not client: return tool_error()
    res = client.execute({
        "type": "ScreenClose",
        "payload": {
            "id": id,
            "target": "single"
        }
    })
    if res.get("status") == "error":
        return f"ERROR: Close failed: {res.get('error')}"
    return "Closed"

@mcp.tool()
def inspect(assurance: bool = True) -> dict:
    """Returns architectural and resource metadata for the entire session."""
    client = get_client()
    if not client: return {"error": tool_error()}
    return client.execute({"type": "Inspect", "payload": {"assurance": assurance}})

@mcp.tool()
def get_architecture() -> str:
    """Returns the architectural knowledge graph report for this codebase."""
    # First, look for the bundled report in the package
    base_dir = os.path.dirname(__file__)
    bundled_path = os.path.join(base_dir, "GRAPH_REPORT.md")
    
    # Second, look for the local dev path
    dev_path = os.path.join(base_dir, "../../graphify-out/GRAPH_REPORT.md")
    
    for path in [bundled_path, dev_path]:
        if os.path.exists(path):
            with open(path, "r", encoding="utf-8") as f:
                return f.read()
                
    return "Architecture report not found. The package might be missing its bundled GRAPH_REPORT.md."

@mcp.resource("vterm://architecture")
def architecture_resource() -> str:
    """The definitive architectural map of vterm-rs."""
    return get_architecture()

def main():
    mcp.run()

if __name__ == "__main__":
    main()
