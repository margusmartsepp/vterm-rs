# Tool: `batch`

Executes a batch of commands atomically for high performance.

## Metadata
- **Status**: Stable
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Example Tool Call

```json
{
  "name": "batch",
  "arguments": {
    "commands": [
      {
        "type": "Inspect",
        "payload": {
          "assurance": true
        }
      }
    ]
  }
}
```

## Verified Output

```json
{
  "status": "success",
  "duration_ms": 234,
  "sub_results": [
    {
      "status": "success",
      "duration_ms": 234,
      "version": "0.7.20",
      "mem_usage_mb": 24,
      "active_terminals": 1,
      "pool_size": 5,
      "max_terminals": 100
    }
  ]
}
```

## Agent Reasoning & Use Cases

- **Latency Minimization**: For workflows requiring multiple round-trips (spawn -> write -> wait -> extract), batch eliminates IPC overhead, reducing total execution time.
- **Atomic Rollback**: If stop_on_error is set, the orchestrator stops at the first failure, preventing operations on an invalid state.
- **Workflow Snapshots**: Perfect for 'Playbooks' — predefined sequences of commands that the agent can trigger with a single tool call.

