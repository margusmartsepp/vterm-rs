# Tool: `wait_until`

Blocks until a regex pattern appears on the terminal screen.

## Metadata
- **Status**: Stable
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Example Tool Call

```json
{
  "name": "wait_until",
  "arguments": {
    "id": 6,
    "pattern": "echo",
    "timeout_ms": 5000
  }
}
```

## Verified Output

```text
Pattern found
```

## Agent Reasoning & Use Cases

- **Synchronization**: The most reliable way to know when a command has finished and it's safe to send the next one (e.g., waiting for the shell prompt PS C:\>).
- **Error Detection**: Use wait_until to catch specific error strings (e.g., Error: Build failed) as soon as they appear, rather than waiting for a full timeout.
- **Boot Verification**: When starting a service (like a web server), use wait_until to watch for the 'Listening on port XXXX' message before proceeding with tests.

