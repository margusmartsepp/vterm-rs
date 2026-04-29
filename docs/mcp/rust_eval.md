# Tool: `rust_eval`

Evaluates a snippet of Rust code in a persistent REPL environment using `evcxr`. This allows agents to perform complex calculations or logic that would be hard to do via shell scripts alone.

## Metadata
- **Rust Endpoint Only**: `vterm-mcp`

## Arguments

| Argument | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `code` | `string` | Rust code to evaluate. | **Required** |

## Example Tool Call

```json
{
  "name": "rust_eval",
  "arguments": {
    "code": "let x = 10; let y = 20; println!(\"{}\", x + y);"
  }
}
```

## Verified Output

```text
30
```

## Prerequisites
Requires `evcxr` to be installed on the host system:
```bash
cargo install evcxr_repl
```
