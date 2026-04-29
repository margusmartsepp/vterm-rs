# Tool: `inspect`

Returns architectural and resource metadata for the entire session.

## Metadata
- **Status**: Stable
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Example Tool Call

```json
{
  "name": "inspect",
  "arguments": {}
}
```

## Verified Output

```json
{
  "status": "success",
  "duration_ms": 46,
  "version": "0.7.20",
  "mem_usage_mb": 25,
  "active_terminals": 0,
  "pool_size": 5,
  "max_terminals": 100
}
```

