# Tool: `get_info`

Returns version and build metadata for this vterm-rs instance.

## Metadata
- **Status**: Stable
- **Rust Endpoint**: `vterm-mcp`
- **Python Endpoint**: `vterm_python.server`

## Example Tool Call

```json
{
  "name": "get_info",
  "arguments": {}
}
```

## Verified Output

```json
{
  "status": "success",
  "duration_ms": 0,
  "version": "0.7.20",
  "active_terminals": 0,
  "pool_size": 5,
  "max_terminals": 100
}
```

