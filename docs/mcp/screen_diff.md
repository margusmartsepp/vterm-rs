# Tool: `screen_diff`

Returns only the visual changes (deltas) since the last time the screen was read or diffed. This is significantly more token-efficient than full screen reads for monitoring live output.

## Metadata
- **Status**: Stable
- **Orchestrator Version**: 0.7.20+
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Arguments

| Argument | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `id` | `int` | Terminal ID. | **Required** |

## Example Tool Call

```json
{
  "name": "screen_diff",
  "arguments": {
    "id": 1
  }
}
```

## Verified Output

```json
{
  "status": "success",
  "content": "... [Only the new lines or modified characters] ..."
}
```

## Agent Reasoning & Use Cases

- **Live Log Monitoring**: Use `screen_diff` in a loop to tail logs without re-sending the entire scrollback every time.
- **Progress Tracking**: Efficiently check if a long-running process (like `npm install` or `cargo build`) is still making progress.
- **Token Optimization**: Minimizes context window usage by only feeding "what changed" to the LLM.
