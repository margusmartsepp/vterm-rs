# Graph Report - term-rs  (2026-04-27)

## Corpus Check
- 43 files · ~20,386 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 252 nodes · 456 edges · 15 communities detected
- Extraction: 51% EXTRACTED · 49% INFERRED · 0% AMBIGUOUS · INFERRED: 222 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 16|Community 16]]

## God Nodes (most connected - your core abstractions)
1. `VTermClient` - 21 edges
2. `spawn()` - 20 edges
3. `execute()` - 17 edges
4. `main()` - 13 edges
5. `batch()` - 12 edges
6. `close()` - 12 edges
7. `handle_connection()` - 11 edges
8. `orchestrate_claude_tui()` - 10 edges
9. `read()` - 10 edges
10. `App` - 9 edges

## Surprising Connections (you probably didn't know these)
- `execute()` --calls--> `Terminal`  [INFERRED]
  vterm\src\service\dispatcher.rs → vterm\src\terminal\instance.rs
- `drive_claude_tui()` --calls--> `VTermClient`  [INFERRED]
  scripts\claude_orchestrator.py → vterm-python\src\lib.rs
- `reality_check_claude()` --calls--> `VTermClient`  [INFERRED]
  scripts\reality_check_claude.py → vterm-python\src\lib.rs
- `reality_check_claude()` --calls--> `close()`  [INFERRED]
  scripts\reality_check_claude.py → vterm-python\vterm_python\server.py
- `terminal_bridge()` --calls--> `VTermClient`  [INFERRED]
  scripts\terminal_bridge.py → vterm-python\src\lib.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.09
Nodes (27): AppBuilder, drive_claude_tui(), OrchestratorClient, spawn(), spawn_viewer_window(), memchr_subseq(), pump_loop(), start() (+19 more)

### Community 1 - "Community 1"
Cohesion: 0.1
Nodes (21): Simulates cloning a repo and running a CI build, monitoring the terminal until c, trigger_and_monitor_build(), check_disk_space(), ping_internal_services(), Pings an internal service to verify network connectivity., Spawns a terminal, checks system disk space, and returns the result., inspect_docker_container(), Spawns a terminal, execs into a running Docker container, and fetches the top pr (+13 more)

### Community 2 - "Community 2"
Cohesion: 0.07
Nodes (12): batch_sub_results_serialize(), BatchArgs, Request, Response, response_omits_none(), SkillCommand, SpawnArgs, Status (+4 more)

### Community 3 - "Community 3"
Cohesion: 0.12
Nodes (13): CommandResult, Dispatcher, execute(), Ready, Terminal<state::Ready>, connect_with_retry(), read_response(), send_request() (+5 more)

### Community 4 - "Community 4"
Cohesion: 0.15
Nodes (4): App, Owned, WatchdogEntry, Terminal<S>

### Community 5 - "Community 5"
Cohesion: 0.15
Nodes (7): Inner, Sealed, Spawning, State, Terminal, Terminal<state::Spawning>, wait()

### Community 6 - "Community 6"
Cohesion: 0.2
Nodes (3): expand(), parse(), token_re()

### Community 7 - "Community 7"
Cohesion: 0.18
Nodes (10): Spawns a new terminal session. Returns the terminal ID., Writes keystrokes (like <Enter> or <Up>) or text to a terminal., Reads the current contents of the terminal screen., Waits until a regex pattern appears on the screen., Closes a terminal session., terminal_close(), terminal_read(), terminal_spawn() (+2 more)

### Community 8 - "Community 8"
Cohesion: 0.28
Nodes (3): Correlation<S>, Timing<S>, Tracing<S>

### Community 9 - "Community 9"
Cohesion: 0.25
Nodes (6): init_tracing(), main(), McpIdArgs, McpSpawnArgs, McpWaitArgs, McpWriteArgs

### Community 10 - "Community 10"
Cohesion: 0.33
Nodes (3): pipeline(), Tracing, TracingLayer

### Community 11 - "Community 11"
Cohesion: 0.4
Nodes (4): architecture_resource(), get_architecture(), Returns the architectural knowledge graph report for this codebase., The definitive architectural map of vterm-rs.

### Community 12 - "Community 12"
Cohesion: 0.5
Nodes (2): Correlation, CorrelationLayer

### Community 13 - "Community 13"
Cohesion: 0.5
Nodes (2): Timing, TimingLayer

### Community 16 - "Community 16"
Cohesion: 1.0
Nodes (1): Error

## Knowledge Gaps
- **37 isolated node(s):** `Simulates cloning a repo and running a CI build, monitoring the terminal until c`, `Spawns a terminal, checks system disk space, and returns the result.`, `Pings an internal service to verify network connectivity.`, `Spawns a terminal, execs into a running Docker container, and fetches the top pr`, `Demonstrates the 'Brick-Proof' guardrails of vterm-rs.     Uses the 'Batch' API` (+32 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 12`** (4 nodes): `Correlation`, `CorrelationLayer`, `.layer()`, `correlation.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 13`** (4 nodes): `Timing`, `TimingLayer`, `.layer()`, `timing.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 16`** (2 nodes): `Error`, `error.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `execute()` connect `Community 3` to `Community 0`, `Community 1`, `Community 4`, `Community 5`, `Community 6`?**
  _High betweenness centrality (0.161) - this node is a cross-community bridge._
- **Why does `VTermClient` connect `Community 1` to `Community 0`, `Community 3`?**
  _High betweenness centrality (0.096) - this node is a cross-community bridge._
- **Why does `spawn()` connect `Community 0` to `Community 1`, `Community 3`, `Community 4`, `Community 7`, `Community 11`?**
  _High betweenness centrality (0.091) - this node is a cross-community bridge._
- **Are the 7 inferred relationships involving `VTermClient` (e.g. with `run_risky_build()` and `manage_terminal_fleet()`) actually correct?**
  _`VTermClient` has 7 INFERRED edges - model-reasoned connections that need verification._
- **Are the 18 inferred relationships involving `spawn()` (e.g. with `drive_claude_tui()` and `reality_check_claude()`) actually correct?**
  _`spawn()` has 18 INFERRED edges - model-reasoned connections that need verification._
- **Are the 15 inferred relationships involving `execute()` (e.g. with `.ok()` and `spawn()`) actually correct?**
  _`execute()` has 15 INFERRED edges - model-reasoned connections that need verification._
- **Are the 8 inferred relationships involving `main()` (e.g. with `parse()` and `.ok()`) actually correct?**
  _`main()` has 8 INFERRED edges - model-reasoned connections that need verification._