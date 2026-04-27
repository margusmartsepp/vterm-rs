# Architecture

## Components at a glance

`vterm-rs` is a tiered system. The core Rust orchestrator manages the PTYs and parses the screen grid, while various bridge layers expose this power to AI agents.

```mermaid
graph TD
    Client["AI Client<br/>(Claude / Cursor / Script)"]
    
    subgraph Consumption Layers
        PySDK["Python SDK / PyO3<br/>(vterm-rs-python-mcp)"]
        PyBridge["FastMCP Server<br/>(vterm-mcp-py)"]
        RustBridge["Native MCP Server<br/>(vterm-mcp)"]
    end
    
    Orchestrator["vterm.exe<br/>(The PTY Host)"]
    
    Client -- "Native PyO3" --> PySDK
    Client -- "MCP (stdio)" --> PyBridge
    Client -- "MCP (stdio)" --> RustBridge
    
    PySDK -- "Named Pipe" --> Orchestrator
    PyBridge -- "Named Pipe" --> Orchestrator
    RustBridge -- "Named Pipe" --> Orchestrator
```

## Internal Rust Architecture

```mermaid
graph TD
    App["App<br/>• DashMap&lt;Id, Terminal&gt;<br/>• next_id (AtomicU32)<br/>• default_visible (bool)<br/>• prompt_regex (Regex)"]
    Watchdog["Watchdog task<br/>500 ms tick<br/>enforces timeout/max_lines"]
    PipeServer["Pipe server task<br/>one accept loop<br/>per-conn Reaper"]
    Pipeline["Tower pipeline<br/>Correlation<br/>Timing<br/>Tracing"]
    Dispatcher["Dispatcher"]
    
    App --> Watchdog
    App --> PipeServer
    PipeServer -- "NDJSON" --> Pipeline
    Pipeline --> Dispatcher
    Dispatcher --> TerminalWrite["Terminal::write"]
    Dispatcher --> TerminalScreen["Terminal::screen"]
    Dispatcher --> AppSpawn["App::spawn"]
    Dispatcher --> AppReap["App::reap"]
```

## Lifecycles

### Terminal (type-state)



```mermaid
stateDiagram-v2
    direction LR
    [*] --> Spawning
    Spawning --> Ready
    Ready --> Running
    Running --> Closing
    Closing --> Reaped

    note right of Ready: PTY open, prompt seen
    note right of Running: initial cmd sent
    note right of Closing: any active command in flight
    note right of Reaped: child.kill, child.wait, pipe shut
```

Each transition is encoded in the type system inside `terminal::instance` so that, for
example, a `Terminal<Spawning>` cannot be passed to `ScreenRead` — the program won't
compile.

### Connection



```mermaid
stateDiagram-v2
    direction LR
    [*] --> Connect
    Connect --> Handshake: req_id 0 Hello
    Handshake --> Active
    Active --> Disconnect
    Disconnect --> Reaper: Reaper drains all owned PTYs
```

The Reaper is a `Drop` impl on the per-connection guard, so reaping happens whether the
client disconnected cleanly or panicked.

## Concurrency model

- One tokio multi-threaded runtime (`flavor = "multi_thread"`).
- One pipe-server task accepts connections sequentially (single-instance pipe).
- Per connection: one read loop + one write Mutex (`tokio::sync::Mutex<WriteHalf>`).
- Per terminal: one PTY-pump thread (`spawn_blocking`) → mpsc → optional client viewer.
- `vt100::Parser` and the PTY writer are guarded by `parking_lot::Mutex` —
  always-short critical sections, never held across `.await`.

## Why these choices

| Choice                         | Why                                                                 |
| ------------------------------ | ------------------------------------------------------------------- |
| `tower::Service` for dispatch  | Aspects (timing, tracing, correlation) compose without forking      |
| `parking_lot::Mutex`           | No poisoning, faster than `std`, fine for short critical sections   |
| `tokio::sync::Mutex<WriteHalf>`| The only Mutex held across `.await` (inside the writer)             |
| `OnceLock<Regex>`              | One-time compilation, no `lazy_static` macro                        |
| `bon::Builder` on `Spawn`      | Long argument lists become fluent and self-documenting              |
| Type-state on `Terminal<S>`    | Compile-time impossibility of "read before ready"                   |
| Single-instance pipe           | Eliminates the multi-orchestrator confusion that bit us in v0.5     |
| Per-connection ownership       | Closes the zombie-PTY class of bugs                                 |

## Module dependency graph



```mermaid
graph BT
    error
    protocol --> error
    window --> error
    session --> error
    service --> error
    
    terminal --> protocol
    terminal --> window
    terminal --> session
    terminal --> service
    
    app --> terminal
    bin_term["bin/vterm"] --> app
```
