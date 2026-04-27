# Claude Desktop Integration Guide

This directory contains real-world, **validated** examples of conversations between AI agents (like Claude 3.5 Sonnet) and the `vterm-rs` orchestrator.

## Quick Start: One-Click Installation

If you use [Cursor](https://cursor.com), you can install the MCP server directly using the button on our [main README](../../README.md).

## Manual Integration: Claude Desktop

To add `vterm-rs` to your Claude Desktop client, edit your `%APPDATA%\Claude\claude_desktop_config.json` file:

### Option 1: Via PyPI (Recommended)
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

### Option 2: Local Development
Use this if you are developing `vterm-rs` and want to test local changes.

```json
{
  "mcpServers": {
    "vterm-local": {
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
