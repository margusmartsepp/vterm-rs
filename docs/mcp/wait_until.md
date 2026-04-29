# Tool: `wait_until`

Blocks until a specific regex pattern appears on the terminal screen.

## Metadata
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Arguments

| Argument | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `id` | `int` | The terminal ID to watch. | **Required** |
| `pattern` | `string` | Regex pattern to wait for. | **Required** |
| `timeout_ms` | `int` | Maximum time to wait in milliseconds. | 10000 |

## Example Tool Call

```json
{
  "name": "wait_until",
  "arguments": {
    "id": 1,
    "pattern": "Successfully built",
    "timeout_ms": 30000
  }
}
```

## Verified Output

```json
{
  "status": "success",
  "content": "Successfully built target(s) in 5.2s"
}
```

## Implementation Note
This tool provides real-time progress notifications to the AI client while waiting, allowing the agent to know it's still alive.
