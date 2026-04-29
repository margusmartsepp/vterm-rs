# Tool: `read`

Reads the current contents of the terminal screen. Set history=True for full scrollback.

## Metadata
- **Status**: Stable
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Example Tool Call

```json
{
  "name": "read",
  "arguments": {
    "id": 6
  }
}
```

## Verified Output

```text
PS C:\Users\margu> echo 'verified'
verified
PS C:\Users\margu>
```

## Agent Reasoning & Use Cases

- **Visual Verification**: The primary way to 'see' what the terminal is doing. Agents should read after a write to confirm the command was typed correctly and to see the initial response.
- **TUI Interaction**: Essential for interacting with Text User Interfaces (like htop, vim, or custom CLI menus) where the current state is not just a sequence of lines but a 2D grid.
- **Debugging**: If a wait_until timeout occurs, the agent should read the screen to understand what prevented the pattern from appearing (e.g., an unexpected password prompt or an error message).

