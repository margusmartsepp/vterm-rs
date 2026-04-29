# Tool: `spawn`

Spawns a new terminal session. Returns the terminal ID and any extracted data.

## Metadata
- **Status**: Stable
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Example Tool Call

```json
{
  "name": "spawn",
  "arguments": {
    "title": "doc-gen-Python MCP",
    "command": "echo 'verified'"
  }
}
```

## Verified Output

```json
{
  "status": "success",
  "duration_ms": 176,
  "spawn_ms": 6,
  "ready_ms": 169,
  "id": 6
}
```

## Agent Reasoning & Use Cases

- **Task Isolation**: Spawn a dedicated terminal for every major task (e.g., one for 'Server Logs', one for 'Build Commands', one for 'Git Operations'). This prevents output interleaving and makes state verification easier.
- **Resource Guardrails**: Use max_lines and timeout_ms to prevent an agent from accidentally triggering an infinite loop of output that would exhaust tokens or crash the host.
- **Environment Setup**: Use the env argument to set up specific project contexts (e.g., DATABASE_URL, NODE_ENV) without affecting the global machine state.

