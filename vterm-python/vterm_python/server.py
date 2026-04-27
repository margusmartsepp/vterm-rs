from fastmcp import FastMCP
import vterm_python
import os

# Create the MCP server
mcp = FastMCP("vterm")
client = vterm_python.VTermClient()

@mcp.tool()
def spawn(title: str, visible: bool = False, max_lines: int = 1000) -> int:
    """Spawns a new terminal session. Returns the terminal ID."""
    return client.spawn(title, visible=visible, max_lines=max_lines)

@mcp.tool()
def write(id: int, text: str) -> str:
    """Writes text to a terminal. Supports <Enter>, <C-c>, etc."""
    client.write(id, text)
    return "OK"

@mcp.tool()
def read(id: int) -> str:
    """Reads the current screen state of a terminal."""
    return client.read(id)

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
