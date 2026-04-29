# Tool: `get_process_state`

Returns the current execution state of the process running in the terminal.

## Metadata
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Arguments

| Argument | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `id` | `int` | Terminal ID. | **Required** |

## Example Tool Call

```json
{
  "name": "get_process_state",
  "arguments": {
    "id": 1
  }
}
```

## Verified Output (Running)

```json
{
  "status": "success",
  "running": true,
  "pid": 1234
}
```

## Verified Output (Exited)

```json
{
  "status": "success",
  "running": false,
  "exit_code": 0
}
```
