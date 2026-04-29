# Tool: `wait_until_stable`

Blocks until the terminal screen stops changing for a specified duration. Useful for waiting for slow-painting UI elements (like progress bars or TUI menus) to finish rendering.

## Metadata
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Arguments

| Argument | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `id` | `int` | Terminal ID. | **Required** |
| `stable_ms` | `int` | How long the screen must remain identical (in ms). | 500 |
| `timeout_ms` | `int` | Maximum time to wait. | 10000 |

## Example Tool Call

```json
{
  "name": "wait_until_stable",
  "arguments": {
    "id": 1,
    "stable_ms": 1000,
    "timeout_ms": 5000
  }
}
```

## Verified Output

```json
{
  "status": "success",
  "content": "... [Current Screen Buffer] ..."
}
```
