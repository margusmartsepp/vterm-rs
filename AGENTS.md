# AGENTS.md

> Authoritative guide for AI agents (and humans) editing this repository.
> If `AGENTS.md`, `README.md`, and inline doc-comments disagree, **this file wins**.

This is the [agents.md](https://agents.md) standard guide for the **vterm-rs Orchestrator** —
a Rust PTY host that lets AI agents drive real terminals (PowerShell now; bash, zsh, fish later).

---

## 1. Mental model in one paragraph

`vterm.exe` is a long-lived **orchestrator** process. It owns a pool of pseudo-terminals
(PTYs), parses each into an in-memory `vt100` screen grid, and exposes them over a single
**named pipe** (`\\.\pipe\litellm-term-skill`) using newline-delimited JSON commands. A client — the AI, an MCP bridge, or a PowerShell test harness — connects, sends commands,
and reads back results that include `req_id` correlation, structured durations, and an
aggregate status. Each connection owns the terminals it spawns and the orchestrator reaps
them when the connection drops. Visible windows are **optional** — headless is first-class.

---

## 1.5. Agentic philosophy: Safety & Truth

`vterm-rs` is built for **State Machine Inspection**, not blind string manipulation.

1.  **Guardrails are Mandatory**: When running potentially high-volume commands (e.g., `cargo build`), agents MUST use `max_lines` and `max_duration`. This prevents the "Infinite Log Flood" that can brick a host system. (See `examples/python_sdk/guardrailed_build.py`).
2.  **Visual Truth**: For complex TUIs (e.g., `htop`, `claude`), agents should use `screen_read` to inspect the grid and `wait_until` to assert on visual states. Avoid "Are we there yet?" polling. (See `examples/python_sdk/tui_orchestration.py`).
3.  **Fleet Control**: One connection can manage a fleet of terminals. Use this to separate "Build," "Logs," and "Interactive" sessions. (See `examples/python_sdk/terminal_fleet.py`).

---

## 2. Project layout

```
vterm-rs/
├── AGENTS.md                  # this file
├── README.md                  # product front door
├── ROADMAP.md                 # vision + milestones
├── CHANGELOG.md               # release notes
├── Cargo.toml                 # workspace manifest
├── skill.toml                 # AI skill manifest
├── .github/workflows/         # CI/CD (crates.io & PyPI publish)
├── vterm/                     # core Rust orchestrator (vterm-rs crate)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs             # business logic
│   │   └── bin/               # vterm and vterm-mcp binaries
│   └── tests/                 # integration tests
├── vterm-python/              # Python SDK (vterm-rs-python-mcp)
│   ├── Cargo.toml
│   ├── pyproject.toml         # maturin/PyPI config
│   └── src/                   # PyO3 bindings
├── examples/                  # usage examples (Python SDK)
└── tests/                     # cross-workspace test suites
```

---

## 3. Hard invariants

These are non-negotiable. Break them and you ship bugs.

1. **Every command produces exactly one response.** No exceptions. A malformed JSON line,
   an internal panic, a missing terminal ID — all surface as a `CommandResult` with
   `status = "error"` and the original `req_id` echoed back. Silent drops are a defect.
2. **`Batch` is atomic at the wire level.** It produces one response containing
   `sub_results`, never N+1 lines.
3. **`req_id` is opaque to the server.** If the client sends one, the server echoes it
   verbatim on the response. The server never invents `req_id`s.
4. **`first_pipe_instance(true)`.** Only one orchestrator may bind the pipe name.
   Multi-instance routing was the cause of week-one debugging pain — keep it singleton.
5. **Connection ownership reaps terminals.** Drop a connection, lose its PTYs. No
   global terminal soup that survives across clients.
6. **Headless is the production default for CI.** `vterm.exe --headless` exists. Tests
   must never hardcode `visible = true`.
7. **No `unwrap()` on user-reachable code paths.** `unwrap` is permitted only in
   bootstrapping (`main`, `OnceLock` initializers) and tests. Everything else is
   `Result<_, vterm_rs::Error>`.
8. **All shared mutable state goes through `parking_lot::Mutex` or `tokio::sync::*`.**
   Never `std::sync::Mutex` on a hot path inside an async handler.

---

## 4. Code style

We aim for elegant, fluent, type-state-aware Rust. New code should:

- Prefer **internally tagged serde enums** (`#[serde(tag = "type", content = "payload")]`)
  for protocol unions; never untagged.
- Use **`OnceLock<Regex>`** instead of `lazy_static!`; we do not depend on `lazy_static`.
- Use **`thiserror::Error`** at module boundaries; `anyhow` is allowed only in `bin/`.
- Use **`#[tracing::instrument]`** on every public async fn that handles a command. Spans
  must include the `req_id` and terminal `id` as fields.
- Use **`bon::Builder`** for any struct with three or more constructor fields.
- Use **type-state** to encode lifecycle: a `Terminal<Spawning>` cannot be read; a
  `Terminal<Ready>` can. Errors that "can't happen" should be unrepresentable.
- Use **`#[non_exhaustive]`** on every public enum and result struct.

Do not:

- Add untracked dependencies. Update `Cargo.toml` and explain in the commit body.
- Introduce `async-trait`. Use Rust ≥ 1.75 native async-fn-in-trait or `BoxFuture` if
  needed for object safety.
- Reach for `Box<dyn Future>` when `impl Future` works.
- Add a `lazy_static!` macro, an `unsafe` block (outside `window/windows.rs`), or a
  blocking call inside an async context (use `spawn_blocking`).

---

## 5. Build, run, test

```powershell
# build
cargo build --release

# run (visible windows)
cargo run -- --visible

# run (headless, recommended for AI / CI)
cargo run -- --headless

# Rust tests (no PTY needed — protocol-level)
cargo test

# end-to-end smoke (requires the orchestrator already running, see ROADMAP §Bench)
.\tests\playbook_tests.ps1 -Headless
```

---

## 6. When in doubt

- For a wire-format change: update `docs/protocol.md` *first*, then code, then the
  PowerShell smoke harness.
- For a new command type: add it to `protocol::SkillCommand`, then dispatch in
  `service::Dispatcher`, then write a `tests/protocol.rs` round-trip.
- For a new platform target (Linux, macOS): gate via `cfg(target_os = ...)` in the
  `window/` module — that is the only OS-coupled module today.

The codebase is small enough to fit in your context window. Read it before editing it.
