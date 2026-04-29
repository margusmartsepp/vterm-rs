# Tool: `get_process_state`

Returns the current process state (running: bool, exit_code: int?).

## Metadata
- **Status**: Stable
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Example Tool Call

```json
{
  "name": "get_process_state",
  "arguments": {
    "id": 6
  }
}
```

## Verified Output

```text
Error calling tool 'get_process_state': 'builtins.VTermClient' object has no attribute 'get_process_state'
```

## Agent Reasoning & Use Cases

- **Termination Detection**: Instead of relying on visual cues, get_process_state provides the definitive OS-level truth of whether a process has exited.
- **Error Handling**: Checking the exit_code allows the agent to distinguish between a successful run (0) and a failure (non-zero).
- **Hanging Diagnostics**: If a terminal is unresponsive, the agent can check if the process is still running or if it has crashed silently.

