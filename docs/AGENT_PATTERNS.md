# Agentic Interaction Patterns

This document defines the high-level patterns for AI agents interacting with `vterm-rs`.

## 1. Risky Command Execution (Guardrails)

**rationale**: Agents often need to run commands with unknown output volumes or execution times. `vterm-rs` provides deterministic safety limits to prevent resource exhaustion.
**links**: crate::App::spawn, examples::python_sdk::guardrailed_build

### Pattern: The "Brick-Proof" Build
When running `cargo build` or complex scripts, always set `max_lines` to prevent log floods.
```python
# links: vterm_python.VTermClient.spawn
client.spawn(name="risky_task", max_lines=500, max_duration=300)
```

## 2. TUI Orchestration (State Awareness)

**rationale**: Navigating interactive TUIs requires "seeing" the screen state rather than blind polling.
**links**: crate::terminal::Terminal::read_screen, examples::python_sdk::tui_orchestration

### Pattern: Reason & Continue
1. **Wait**: Use `wait_until` to reach a known UI state.
2. **Inspect**: Use `screen_read` to verify the grid contents.
3. **Act**: Send targeted keystrokes.

## 3. Fleet Management (Multi-Terminal)

**rationale**: Complex agentic workflows require separating concerns into distinct terminal sessions.
**links**: crate::App::list, examples::python_sdk::terminal_fleet

### Pattern: Concurrent Contexts
Maintain one terminal for long-running builds and another for real-time log monitoring.
