# .ai/instructions.md

> Compact agent brief. The full guide is [`AGENTS.md`](../AGENTS.md). When this file
> and `AGENTS.md` disagree, `AGENTS.md` wins.

You are editing **term**, a Rust PTY orchestrator that lets AI drive real terminals on
Windows. The product owner is a principal engineer; ship code that's worth reading.

## Before you write a single line

1. Read [`AGENTS.md`](../AGENTS.md) end to end.
2. Skim [`docs/architecture.md`](../docs/architecture.md) and
   [`docs/protocol.md`](../docs/protocol.md).
3. Check [`ROADMAP.md`](../ROADMAP.md) — is this work in scope for the current milestone?

## Hard rules

- Every command produces exactly one response. Silent drops are bugs.
- `Batch` returns one aggregate response, never N+1 lines.
- `req_id` is echoed verbatim, never invented.
- `parking_lot::Mutex` for short critical sections; `tokio::sync::Mutex` if held across
  `.await`. Never `std::sync::Mutex` on async paths.
- `unwrap()` allowed only in `main`, `OnceLock` initialisers, and tests.
- Add `#[tracing::instrument]` to every public async fn that handles a command. Span
  fields must include `req_id` and (where applicable) terminal `id`.
- New crate dependencies require a justification in the commit message.

## Style cheatsheet

```rust
// good — internally tagged, non-exhaustive, type-state
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
#[non_exhaustive]
pub enum SkillCommand { /* ... */ }

// good — bon builder for many-field structs
#[bon::builder]
pub fn spawn(title: impl Into<String>, #[builder(default)] visible: bool) -> Result<Id> { /* ... */ }

// good — OnceLock for compile-once regex
fn shortcut_re() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"<([A-Za-z0-9-]+)>").unwrap())
}

// bad — std Mutex across await
let guard = std_mutex.lock().unwrap();
some_future.await; // <-- holds the lock the whole time
```

## When you change the wire protocol

1. Update [`docs/protocol.md`](../docs/protocol.md) first.
2. Bump `[package].version` in `Cargo.toml`.
3. Add a `## vX.Y` section to [`CHANGELOG.md`](../CHANGELOG.md).
4. Add a `docs/features/vX.Y.md` if the change is large enough to warrant prose.
5. Update the PowerShell smoke harness and Rust integration tests.

## When you fail

If you can't finish the task — file blocked, missing dependency, ambiguous requirement —
do not partial-implement and call it done. Stop, write a `// TODO(agent): ...` with
specifics, and tell the user what blocked you. Honesty over completion.
