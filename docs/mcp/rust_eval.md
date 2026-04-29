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

}
```

## Agent Reasoning & Use Cases

- **Complex Logic**: When a task requires math or data manipulation that is awkward in shell (e.g., parsing a complex JSON file and calculating a checksum), use `rust_eval`.
- **System Probing**: Use Rust's standard library to check file permissions, network availability, or system entropy in a more structured way than `ls` or `netstat`.
- **Performance**: For CPU-intensive tasks (e.g., processing a large log file), compiled Rust in the REPL will be significantly faster than interpreted Python or shell scripts.

## Prerequisites
Requires `evcxr` to be installed on the host system:
```bash
cargo install evcxr_repl
```
