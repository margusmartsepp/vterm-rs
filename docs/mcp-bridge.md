# MCP Bridge (`vterm-mcp`)

The Model Context Protocol (MCP) Bridge allows any MCP-compliant AI client (such as Claude Desktop, Cursor, or Cowork) to interact natively with the `vterm-rs` orchestrator. It acts as an adapter, translating standardized MCP tool calls into JSON-RPC messages sent over the orchestrator's named pipe.

## Architecture

The bridge is shipped as a separate binary (`vterm-mcp`) in the workspace. It communicates with the orchestrator (`vterm.exe`) over the `\\.\pipe\vterm-rs-skill` named pipe.

```mermaid
graph LR
    Client["AI Client (Claude/Cursor)"]
    Bridge["vterm-mcp"]
    Orchestrator["vterm.exe"]
    
    Client -- "stdio (MCP protocol)" --> Bridge
    Bridge -- "named pipe (JSON-RPC)" --> Orchestrator
```

### Auto-Spawning

The bridge provides a zero-configuration experience. When launched by an AI client, it automatically attempts to connect to the orchestrator pipe. If the orchestrator is not running, `vterm-mcp` will spawn it invisibly in the background (`cargo run -- --headless`) and retry the connection.

### Connection Reaping

The bridge holds a long-lived multiplexed connection to the orchestrator. Because `vterm-rs` binds terminal lifecycles to the pipe connection, if the AI client drops the stdio stream (e.g., when the Claude Desktop session is closed), the bridge process exits, and the orchestrator instantly reaps all child processes (like `powershell.exe`) spawned during that session.

## Exposed Tools

The `vterm-mcp` router currently exposes the following tools to AI clients:

1. `terminal_spawn`: Spawns a new PTY shell. Returns a numeric Terminal ID.
2. `terminal_write`: Writes string sequences to the shell (handles keystrokes).
3. `terminal_read`: Reads the current text contents of the screen buffer.
4. `terminal_wait_until`: Polls the screen buffer until a provided regular expression matches or a timeout is reached. **Streams live progress updates back to the AI client during the wait.**
5. `terminal_close`: Safely terminates the specified PTY session.

## Configuration

To configure an AI client to use the bridge, provide the execution path in the client's MCP configuration manifest. 

### Claude Desktop / Cowork Configuration (`cowork.json`)

```json
{
  "mcpServers": {
    "vterm-rs": {
      "command": "cargo",
      "args": [
        "run",
        "--release",
        "--bin",
        "vterm-mcp"
      ],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

> **Note**: For production deployments outside the development workspace, replace the `cargo run` commands with the direct path to the compiled `vterm-mcp.exe` binary.
