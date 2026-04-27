# Changelog

All notable changes are documented here. The wire protocol is unstable until v1.0.

## v0.7.8 — MCP Bridge
- Fixed PyPI workflow (corrected artifact upload paths).

## v0.7.7 — MCP Bridge
- Fixed watchdog compilation on Rust 2021 (removed let-chains).

## v0.7.6 — MCP Bridge
- Fixed abi3 auto-detection (removed redundant CLI flag).
- Downgraded to Rust edition 2021 for broader tooling compatibility.
- Added explicit Python signatures to `VTermClient`.

## v0.7.5 — MCP Bridge
- Fixed PyPI workflow (moved abi3 compatibility flag to command line).

## v0.7.4 — MCP Bridge
- Switched to CPython Stable ABI (abi3) for universal Python 3.10+ compatibility.
- Dropped PyPy support to simplify cross-platform wheel builds.

## v0.7.3 — MCP Bridge
- Fixed Python 3.14 compatibility in CI (ABI3 forward compatibility).
- Modularized Windows networking for better platform isolation.

## v0.7.2 — MCP Bridge
- Crates.io and PyPI coordinated release.
- Added GitHub Trusted Publishing for the Python SDK.

## v0.7.1 — MCP Bridge (Internal)
- Reorganized workspace structure (`vterm/` and `vterm-python/`).
- Auto-spawning orchestration process if unavailable.
- Examples demonstrating use cases for DevOps, GitHub CI, and Docker debugging.

## v0.6.0 — foundations

See [`docs/features/v0.6.md`](docs/features/v0.6.md) for the full changeset.

### Added

- `req_id` correlation throughout the protocol; servers echo it on every response.
- Aggregate `Batch` responses with `sub_results: Vec<CommandResult>`.
- Per-connection ownership and automatic reaping of spawned PTYs on disconnect.
- PID→HWND deterministic window control (survives title changes).
- `--headless` and `--prompt-regex` orchestrator flags.
- `tower::Service` command pipeline with `TimingLayer`, `CorrelationLayer`,
  `TracingLayer`.
- `tracing` instrumentation across the public command surface.
- Rust integration tests under `tests/protocol.rs`.
- Documentation suite: `AGENTS.md`, `ROADMAP.md`, `docs/architecture.md`,
  `docs/protocol.md`, `docs/features/v0.2..v0.6.md`.

### Changed

- Crate split: business logic now lives in `lib.rs` + modules; `src/bin/term.rs` is a
  thin shell.
- Pipe server enforces singleton (`first_pipe_instance(true)`).
- Initial command waits for prompt regex instead of `sleep(2000)`.
- PowerShell smoke harness rewritten around real use cases (Ctrl-C interruption,
  vim exit, multi-service spawn).

### Fixed

- `Batch` no longer reports `success` when sub-commands failed under
  `stop_on_error = false`.
- Malformed JSON now returns a structured error instead of silently dropping the line.
- `Spawn` errors surface as `CommandResult` instead of hanging the client.

## v0.5.1 — supreme *(spec only; mostly delivered in v0.6)*

See [`docs/features/v0.5.md`](docs/features/v0.5.md).

## v0.4 — performance and synchronisation

See [`docs/features/v0.4.md`](docs/features/v0.4.md).

## v0.3 — playbooks and batching

See [`docs/features/v0.3.md`](docs/features/v0.3.md).

## v0.2 — eyes, hands, lifecycle

See [`docs/features/v0.2.md`](docs/features/v0.2.md).
