# Tool: `close`

Closes a terminal session.

## Metadata
- **Status**: Stable
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Example Tool Call

```json
{
  "name": "close",
  "arguments": {
    "id": 6
  }
}
```

## Verified Output

```text
Closed
```

## Agent Reasoning & Use Cases

- **Resource Management**: Always close terminals when a task is finished (e.g., after a build or a long-running log tail) to free up system memory and PTY slots.
- **Session Isolation**: Close terminals between unrelated tasks to ensure no environmental leakage (e.g., dirty env vars or background processes) persists.
- **Admission Control**: If the orchestrator reaches max_terminals, the agent should identify and close idle sessions.

