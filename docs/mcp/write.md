# Tool: `write`

Writes text to a terminal. Supports <Enter>, <C-c>, etc.

## Metadata
- **Status**: Stable
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Example Tool Call

```json
{
  "name": "write",
  "arguments": {
    "id": 6,
    "text": "echo 123<Enter>"
  }
}
```

## Verified Output

```text
OK
```

## Agent Reasoning & Use Cases

- **Command Execution**: The fundamental way to run commands. Always include <Enter> if you want the shell to execute the line.
- **TUI Navigation**: Use arrow keys and <Tab> to navigate menus in tools like htop, vim, or git log.
- **Interrupting Processes**: Use <C-c> to stop long-running commands (like ping or tail) before starting a new task.
- **Interactive Prompts**: Respond to y/n prompts or enter passwords (though use with caution as output might be echoed).

