# Changelog

All notable changes are documented here. The wire protocol is unstable until v1.0.

## v0.6.0 — foundations *(in progress)*

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
