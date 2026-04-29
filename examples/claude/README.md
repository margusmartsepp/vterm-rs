# Claude Desktop Integration Guide

This directory contains real-world, **validated** examples of conversations between AI agents (like Claude 3.5 Sonnet) and the `vterm-rs` orchestrator.

## Quick Start: One-Click Installation

If you use [Cursor](https://cursor.com), you can install the MCP server directly using the button on our [main README](../../README.md).

## Manual Integration: Claude Desktop

To add `vterm-rs` to your Claude Desktop client, edit your `%APPDATA%\Claude\claude_desktop_config.json` file:

### Option 0: Native Rust Binary (Highest Performance)
This is the highest performance option and uses the pure-Rust bridge directly. It has zero overhead and minimal dependencies.

#### A. Pre-built / Installed
```json
{
  "mcpServers": {
    "vterm": {
      "command": "vterm-mcp"
    }
  }
}
```
> [!TIP]
> To use this, run `cargo install vterm-rs`. Ensure the cargo bin directory is in your `PATH`.

#### B. Local Development
Use this if you are developing `vterm-rs` and want to test changes to the Rust bridge.
```json
{
  "mcpServers": {
    "vterm-local": {
      "command": "cargo",
      "args": [
        "run",
        "--release",
        "--bin",
        "vterm-mcp",
        "--manifest-path",
        "C:\\Users\\YOUR_USER\\path\\to\\term-rs\\vterm\\Cargo.toml"
      ]
    }
  }
}
```

### Option 1: Python SDK (Recommended for Python Developers)
Ideal if you want to extend the MCP server using Python or leverage existing Python libraries.

#### A. Via PyPI
This uses `uvx` to automatically handle dependencies and updates.

```json
{
  "mcpServers": {
    "vterm": {
      "command": "uvx",
      "args": ["vterm-rs-python-mcp"]
    }
  }
}
```

#### B. Local Development
Use this if you are developing the Python SDK or want to run from source.

```json
{
  "mcpServers": {
    "vterm-python-local": {
      "command": "uv",
      "args": [
        "--directory",
        "C:\\Users\\YOUR_USER\\path\\to\\term-rs\\vterm-python",
        "run",
        "python",
        "-m",
        "vterm_python.server"
      ]
    }
  }
}
```


## Validated Examples

These examples are real logs of AI behavior. Unlike "AI Slop" placeholders, these have been verified to work with the current orchestrator.

1. [**Network Diagnostics (Ping)**](ping_google.md): Claude reasons about platform-specific command flags (Windows vs. Linux) and provides structured reports.
2. *More coming soon...*
