# vterm-rs Agentic Backlog

This document tracks pain points, frustrations, and feature requests from AI agents using `vterm-rs`.

## Pain Points

### [BP-001] Client/Server Signature Mismatch (FIXED)
- **Problem**: The MCP server framework passed all default arguments (cols, rows, etc.) but the `VTermClient.spawn()` binding didn't accept them, causing a crash.
- **Impact**: Blocked all terminal usage immediately.
- **Resolution**: Updated `server.py` with a robust `spawn()` signature and fallback logic.

### [BP-002] Hardcoded Wait Timers (FIXED)
- **Problem**: The orchestrator often used fixed `sleep(100ms)` blocks.
- **Resolution**: Implemented a `tokio::sync::broadcast` notification system and the **Bloom Filter Search Pipeline**. Latency for sensing events is now sub-millisecond.

### [BP-003] Lack of "List" Tool (FIXED)
- **Problem**: No way to see what terminals are currently alive across different tools.
- **Resolution**: Added `list_terminals()` to `server.py` and the `List` command to the orchestrator.

### [BP-004] Terminal Reaping on Standalone Connection Drop
- **Problem**: Spawning a terminal from a script reaps it when the script ends.
- **Goal**: Implement a "detached" mode or use the `mcp_stabilizer.py` pattern for long-running orchestration.

### [BP-005] Bloom Filter False Positives (NEW)
- **Problem**: As terminals run for hours, the additive Bloom filter might saturate.
- **Goal**: Tune the "Snapshot Reset" logic to ensure high accuracy for 100+ terminals.

### [BP-006] MCP Bridge EOF
- **Problem**: The MCP bridge server frequently fails with EOF when the orchestrator is restarted or when multi-instances of `vterm.exe` are running.
- **Impact**: Interrupts AI tool usage and requires manual process cleanup.

### [BP-007] Multi-Instance Collision
- **Problem**: `vterm.exe` does not aggressively kill stale instances holding the named pipe, leading to "Access Denied" or duplicate pipe errors.
- **Goal**: Implement a "forceful bind" or "stale instance cleanup" on startup.
