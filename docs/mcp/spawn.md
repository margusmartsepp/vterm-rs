# Tool: `spawn`

Spawns a new terminal session. This is the entry point for most terminal automation workflows.

## Metadata
- **Status**: Stable
- **Orchestrator Version**: 0.7.20+
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Arguments

| Argument | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `title` | `string` | A human-readable label for the terminal. | **Required** |
| `command` | `string` | Initial command to execute (e.g., `powershell.exe`). | System Default |
| `cols` | `int` | Terminal width in columns. | 80 |
| `rows` | `int` | Terminal height in rows. | 24 |
| `env` | `dict` | Environment variables for the session. | `{}` |
| `wait` | `bool` | If true, blocks until the command completes or a prompt is found. | `false` |
| `extract_pattern` | `string` | Regex with named groups to extract data from the initial output. | `null` |

## Example Tool Call

```json
{
  "name": "spawn",
  "arguments": {
    "title": "Build Process",
    "command": "cargo build --release",
    "wait": true
  }
}
```

## Verified Output (Success)

```json
{
  "status": "success",
  "id": 1,
  "pid": 1234,
  "title": "Build Process"
}
```

## Verified Output (Error)

```json
{
  "status": "error",
  "error": "Failed to spawn process: The system cannot find the file specified. (os error 2)"
}
```
