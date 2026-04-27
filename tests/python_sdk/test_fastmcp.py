from fastmcp import FastMCP
import vterm_python

# Initialize the FastMCP server
mcp = FastMCP("vterm")

# Initialize the Rust-powered orchestrator client
client = vterm_python.VTermClient()

@mcp.tool()
def terminal_spawn(title: str, command: str | None = None, timeout_ms: int | None = None, max_lines: int | None = None) -> int:
    """Spawns a new terminal session. Returns the terminal ID."""
    return client.spawn(title, command, timeout_ms, max_lines)

@mcp.tool()
def terminal_write(id: int, text: str) -> str:
    """Writes keystrokes (like <Enter> or <Up>) or text to a terminal."""
    client.write(id, text)
    return "Written successfully."

@mcp.tool()
def terminal_read(id: int) -> str:
    """Reads the current contents of the terminal screen."""
    return client.read(id)

@mcp.tool()
def terminal_wait_until(id: int, pattern: str, timeout_ms: int) -> str:
    """Waits until a regex pattern appears on the screen."""
    return client.wait_until(id, pattern, timeout_ms)

@mcp.tool()
def terminal_close(id: int) -> str:
    """Closes a terminal session."""
    client.close(id)
    return "Terminal closed."

if __name__ == "__main__":
    mcp.run()
