# Tool: `batch`

Executes a sequence of operations atomically. This is the most token-efficient way to drive complex workflows.

## Metadata
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Arguments

| Argument | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `commands` | `list` | A list of command objects to execute in order. | **Required** |

## Example Tool Call

```json
{
  "name": "batch",
  "arguments": {
    "commands": [
      { "type": "spawn", "title": "Check Version", "command": "rustc --version" },
      { "type": "wait_until", "id": "$last", "pattern": "rustc" },
      { "type": "extract", "id": "$last", "pattern": "rustc (?P<version>[\\d\\.]+)" }
    ]
  }
}
```

## Verified Output

```json
{
  "status": "success",
  "sub_results": [
    { "status": "success", "id": 12 },
    { "status": "success", "content": "rustc 1.78.0 ..." },
    { "status": "success", "extracted": { "version": "1.78.0" } }
  ]
}
```

## Batch Directives
- `$last`: Refers to the ID of the terminal created or used in the immediately preceding command.
- Parallel execution: Commands without dependencies can be executed concurrently by the orchestrator.
