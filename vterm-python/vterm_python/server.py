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

def main():
    mcp.run()

if __name__ == "__main__":
    main()
