# Tool: `write`

Writes text or keystrokes to a terminal session.

## Metadata
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Arguments

| Argument | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `id` | `int` | The terminal ID to write to. | **Required** |
| `text` | `string` | The text to write. Supports symbolic keys like `<Enter>`, `<Up>`, `<C-c>`. | **Required** |

## Example Tool Call

```json
{
  "name": "write",
  "arguments": {
    "id": 1,
    "text": "ls -la<Enter>"
  }
}
```

## Verified Output

```json
{
  "status": "success"
}
```

## Special Keys
- `<Enter>`: Carriage return
- `<Esc>`: Escape key
- `<C-c>`: Ctrl+C (SIGINT)
- `<Up>`, `<Down>`, `<Left>`, `<Right>`: Arrow keys
- `<Tab>`: Tab key
