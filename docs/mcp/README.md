# VTerm MCP Documentation

This directory contains the definitive, verified documentation for all MCP endpoints provided by the `vterm-rs` orchestrator.

## Terminal Lifecycle
- [**spawn**](spawn.md): Create a new terminal session.
- [**close**](close.md): Terminate a session.
- [**list_terminals**](list_terminals.md): List all active sessions.

## Screen & Interaction
- [**read**](read.md): Read the current screen or history.
- [**write**](write.md): Send text or special keys.
- [**screen_diff**](screen_diff.md): Get high-performance visual deltas.

## Synchronization & Analysis
- [**wait_until**](wait_until.md): Wait for a regex pattern.
- [**wait_until_stable**](wait_until_stable.md): Wait for output to settle.
- [**extract**](extract.md): Pull structured data via regex groups.
- [**get_process_state**](get_process_state.md): Check process liveness.

## Advanced Primitives
- [**batch**](batch.md): Execute atomic command playbooks.
- [**get_architecture**](get_architecture.md): Architecture self-discovery.
- [**rust_eval**](rust_eval.md): Dynamic Rust evaluation.

---
*Last verified against v0.7.20 (Rust) and v0.7.20 (Python SDK).*
