# Tool: `read`

Reads the current contents of the terminal screen buffer.

## Metadata
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Arguments

| Argument | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `id` | `int` | The terminal ID to read from. | **Required** |
| `history` | `bool` | If true, returns the entire scrollback buffer instead of just the current screen. | `false` |

## Example Tool Call

```json
{
  "name": "read",
  "arguments": {
    "id": 1,
    "history": false
  }
}
```

## Verified Output

```json
{
  "status": "success",
  "content": "Microsoft Windows [Version 10.0.22631.3447]\r\n(c) Microsoft Corporation. All rights reserved.\r\n\r\nC:\\Users\\margu\\term-rs> dir\r\n Volume in drive C is Windows\r\n..."
}
```
