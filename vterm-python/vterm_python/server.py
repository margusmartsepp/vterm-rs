from fastmcp import FastMCP
import vterm_python
import os

# Create the MCP server
mcp = FastMCP("vterm")
client = vterm_python.VTermClient()

@mcp.tool()
def get_info() -> dict:
    """Returns version and build metadata for this vterm-rs instance."""
    return client.get_info()

@mcp.tool()
def spawn(title: str, max_lines: int = 1000, visible: bool = False, cols: int = 80, rows: int = 24, env: dict = None) -> int:
    """
    Spawns a new terminal session.
    - cols/rows: Set terminal dimensions (e.g., 120, 40)
    - env: Dictionary of environment variables to inject
    """
    return client.spawn(title, max_lines=max_lines, visible=visible, cols=cols, rows=rows, env=env)

@mcp.tool()
def wait_until(id: int, pattern: str, timeout_ms: int = 10000) -> str:
    """Blocks until a regex pattern appears on the terminal screen."""
    return client.wait_until(id, pattern, timeout_ms=timeout_ms)

@mcp.tool()
def get_process_state(id: int) -> dict:
    """Returns the current process state (running: bool, exit_code: int?)."""
    return client.get_process_state(id)

@mcp.tool()
def write(id: int, text: str) -> str:
    """Writes text to a terminal. Supports <Enter>, <C-c>, etc."""
    client.write(id, text)
    return "OK"

@mcp.tool()
def read(id: int, history: bool = False) -> str:
    """Reads the current contents of the terminal screen. Set history=True for full scrollback."""
    return client.read(id, history=history)

@mcp.tool()
def wait_until(id: int, pattern: str, timeout_ms: int = 10000) -> str:
    """Waits for a pattern to appear on the screen."""
    client.wait_until(id, pattern, timeout_ms)
    return "Pattern found"

@mcp.tool()
def batch(commands: list) -> dict:
    """Executes a batch of commands atomically for high performance."""
    return client.batch(commands)

@mcp.tool()
def close(id: int) -> str:
    """Closes a terminal session."""
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
