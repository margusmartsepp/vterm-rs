# Tool: `list_terminals`

Returns a list of all active terminal IDs and their titles.

## Metadata
- **Status**: Stable
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Example Tool Call

```json
{
  "name": "list_terminals",
  "arguments": {}
}
```

## Verified Output

```json
[]
```

## Agent Reasoning & Use Cases

- **Self-Discovery**: If an agent is restarted or loses track of its state, it can use list_terminals to recover IDs of active sessions it previously spawned.
- **Health Monitoring**: Periodically checking the list ensures that spawned processes haven't crashed unexpectedly.
- **Cleanup Playbooks**: An agent can use this to identify 'zombie' terminals that should be closed.

