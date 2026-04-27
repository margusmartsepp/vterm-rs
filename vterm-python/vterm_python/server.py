from fastmcp import FastMCP
import vterm_python
import os
import sys

# Create the MCP server
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
def spawn(title: str, max_lines: int = 1000, visible: bool = False, cols: int = 80, rows: int = 24, env: dict = None) -> int:
    """
    Spawns a new terminal session.
    - cols/rows: Set terminal dimensions (e.g., 120, 40)
    - env: Dictionary of environment variables to inject
    """
    client = get_client()
    if not client: return tool_error()
    return client.spawn(title, max_lines=max_lines, visible=visible, cols=cols, rows=rows, env=env)

@mcp.tool()
def wait_until(id: int, pattern: str, timeout_ms: int = 10000) -> str:
    """Blocks until a regex pattern appears on the terminal screen."""
    client = get_client()
    if not client: return tool_error()
    return client.wait_until(id, pattern, timeout_ms=timeout_ms)

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
    client.write(id, text)
    return "OK"

@mcp.tool()
def read(id: int, history: bool = False) -> str:
    """Reads the current contents of the terminal screen. Set history=True for full scrollback."""
    client = get_client()
    if not client: return tool_error()
    return client.read(id, history=history)

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
    client.close(id)
    return "Closed"

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
