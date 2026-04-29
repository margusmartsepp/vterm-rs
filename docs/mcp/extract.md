# Tool: `extract`

Extracts structured data from the terminal screen or history using a regular expression with named capture groups.

## Metadata
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Arguments

| Argument | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `id` | `int` | Terminal ID. | **Required** |
| `pattern` | `string` | Regex with named groups (e.g., `(?P<name>.*)`). | **Required** |
| `history` | `bool` | Search the entire scrollback buffer. | `false` |

## Example Tool Call

```json
{
  "name": "extract",
  "arguments": {
    "id": 1,
    "pattern": "IP Address\\. . . . . . . . . . . : (?P<ip>[\\d\\.]+)",
    "history": true
  }
}
```

## Verified Output

```json
{
  "status": "success",
  "extracted": {
    "ip": "192.168.1.10"
  }
}
```
