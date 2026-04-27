# Graph Report - term-rs  (2026-04-28)

## Corpus Check
- 43 files · ~20,943 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 267 nodes · 469 edges · 20 communities detected
- Extraction: 52% EXTRACTED · 48% INFERRED · 0% AMBIGUOUS · INFERRED: 223 edges (avg confidence: 0.8)
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
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]

## God Nodes (most connected - your core abstractions)
1. `VTermClient` - 24 edges
2. `spawn()` - 20 edges
3. `execute()` - 18 edges
4. `main()` - 13 edges
5. `batch()` - 12 edges
6. `close()` - 12 edges
7. `handle_connection()` - 11 edges
8. `orchestrate_claude_tui()` - 10 edges
9. `read()` - 10 edges
10. `App` - 9 edges

## Surprising Connections (you probably didn't know these)
- `drive_claude_tui()` --calls--> `VTermClient`  [INFERRED]
  scripts\claude_orchestrator.py → vterm-python\src\lib.rs
- `drive_claude_tui()` --calls--> `spawn()`  [INFERRED]
  scripts\claude_orchestrator.py → vterm-python\vterm_python\server.py
- `reality_check_claude()` --calls--> `VTermClient`  [INFERRED]
  scripts\reality_check_claude.py → vterm-python\src\lib.rs
- `reality_check_claude()` --calls--> `spawn()`  [INFERRED]
  scripts\reality_check_claude.py → vterm-python\vterm_python\server.py
- `reality_check_claude()` --calls--> `close()`  [INFERRED]
  scripts\reality_check_claude.py → vterm-python\vterm_python\server.py

## Communities

### Community 0 - "Community 0"
Cohesion: 0.09
Nodes (21): Simulates cloning a repo and running a CI build, monitoring the terminal until c, trigger_and_monitor_build(), check_disk_space(), ping_internal_services(), Pings an internal service to verify network connectivity., Spawns a terminal, checks system disk space, and returns the result., inspect_docker_container(), Spawns a terminal, execs into a running Docker container, and fetches the top pr (+13 more)

### Community 1 - "Community 1"
Cohesion: 0.07
Nodes (32): drive_claude_tui(), memchr_subseq(), pump_loop(), start(), viewer_loop(), reality_check_claude(), architecture_resource(), get_architecture() (+24 more)

### Community 2 - "Community 2"
Cohesion: 0.13
Nodes (19): AppBuilder, OrchestratorClient, connect_with_retry(), read_response(), send_request(), test_supreme_orchestration(), spawn(), spawn_viewer_window() (+11 more)

### Community 3 - "Community 3"
Cohesion: 0.08
Nodes (15): CommandResult, Dispatcher, execute(), Inner, Ready, Sealed, Spawning, State (+7 more)

### Community 4 - "Community 4"
Cohesion: 0.07
Nodes (12): batch_sub_results_serialize(), BatchArgs, Request, Response, response_omits_none(), SkillCommand, SpawnArgs, Status (+4 more)

### Community 5 - "Community 5"
Cohesion: 0.2
Nodes (3): expand(), parse(), token_re()

### Community 6 - "Community 6"
Cohesion: 0.24
Nodes (2): Terminal<S>, spawn()

### Community 7 - "Community 7"
Cohesion: 0.2
Nodes (5): McpIdArgs, McpSpawnArgs, McpWaitArgs, McpWriteArgs, TerminalServer

### Community 8 - "Community 8"
Cohesion: 0.22
Nodes (3): App, Owned, WatchdogEntry

### Community 9 - "Community 9"
Cohesion: 0.28
Nodes (3): Correlation<S>, Timing<S>, Tracing<S>

### Community 10 - "Community 10"
Cohesion: 0.33
Nodes (3): pipeline(), Tracing, TracingLayer

### Community 11 - "Community 11"
Cohesion: 0.5
Nodes (2): Correlation, CorrelationLayer

### Community 12 - "Community 12"
Cohesion: 0.5
Nodes (2): Timing, TimingLayer

### Community 15 - "Community 15"
Cohesion: 1.0
Nodes (1): Error

### Community 21 - "Community 21"
Cohesion: 1.0
Nodes (1): Reads the current screen state of a terminal.

### Community 22 - "Community 22"
Cohesion: 1.0
Nodes (1): Waits for a pattern to appear on the screen.

### Community 23 - "Community 23"
Cohesion: 1.0
Nodes (1): Executes a batch of commands atomically for high performance.

### Community 24 - "Community 24"
Cohesion: 1.0
Nodes (1): Closes a terminal session.

### Community 25 - "Community 25"
Cohesion: 1.0
Nodes (1): Returns the architectural knowledge graph report for this codebase.

### Community 26 - "Community 26"
Cohesion: 1.0
Nodes (1): The definitive architectural map of vterm-rs.

## Knowledge Gaps
- **46 isolated node(s):** `Simulates cloning a repo and running a CI build, monitoring the terminal until c`, `Spawns a terminal, checks system disk space, and returns the result.`, `Pings an internal service to verify network connectivity.`, `Spawns a terminal, execs into a running Docker container, and fetches the top pr`, `Demonstrates the 'Brick-Proof' guardrails of vterm-rs.     Uses the 'Batch' API` (+41 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 6`** (11 nodes): `.snapshot_for_watchdog()`, `Terminal<S>`, `.child_pid()`, `.id()`, `.line_count()`, `.max_duration()`, `.max_lines()`, `.spawn_time()`, `.title()`, `watchdog.rs`, `spawn()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 11`** (4 nodes): `Correlation`, `CorrelationLayer`, `.layer()`, `correlation.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 12`** (4 nodes): `Timing`, `TimingLayer`, `.layer()`, `timing.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 15`** (2 nodes): `Error`, `error.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 21`** (1 nodes): `Reads the current screen state of a terminal.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 22`** (1 nodes): `Waits for a pattern to appear on the screen.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 23`** (1 nodes): `Executes a batch of commands atomically for high performance.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 24`** (1 nodes): `Closes a terminal session.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 25`** (1 nodes): `Returns the architectural knowledge graph report for this codebase.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 26`** (1 nodes): `The definitive architectural map of vterm-rs.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `execute()` connect `Community 3` to `Community 0`, `Community 1`, `Community 2`, `Community 5`, `Community 6`, `Community 8`?**
  _High betweenness centrality (0.166) - this node is a cross-community bridge._
- **Why does `VTermClient` connect `Community 0` to `Community 1`, `Community 2`?**
  _High betweenness centrality (0.102) - this node is a cross-community bridge._
- **Why does `spawn()` connect `Community 2` to `Community 0`, `Community 1`, `Community 3`, `Community 6`, `Community 7`, `Community 8`?**
  _High betweenness centrality (0.097) - this node is a cross-community bridge._
- **Are the 7 inferred relationships involving `VTermClient` (e.g. with `run_risky_build()` and `manage_terminal_fleet()`) actually correct?**
  _`VTermClient` has 7 INFERRED edges - model-reasoned connections that need verification._
- **Are the 18 inferred relationships involving `spawn()` (e.g. with `drive_claude_tui()` and `reality_check_claude()`) actually correct?**
  _`spawn()` has 18 INFERRED edges - model-reasoned connections that need verification._
- **Are the 16 inferred relationships involving `execute()` (e.g. with `.ok()` and `spawn()`) actually correct?**
  _`execute()` has 16 INFERRED edges - model-reasoned connections that need verification._
- **Are the 8 inferred relationships involving `main()` (e.g. with `parse()` and `.ok()`) actually correct?**
  _`main()` has 8 INFERRED edges - model-reasoned connections that need verification._