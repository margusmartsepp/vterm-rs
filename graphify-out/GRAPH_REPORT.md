# Graph Report - term-rs  (2026-04-27)

## Corpus Check
- 43 files · ~18,035 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 248 nodes · 450 edges · 13 communities detected
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
- [[_COMMUNITY_Community 14|Community 14]]

## God Nodes (most connected - your core abstractions)
1. `VTermClient` - 21 edges
2. `spawn()` - 20 edges
3. `execute()` - 17 edges
4. `main()` - 13 edges
5. `batch()` - 12 edges
6. `close()` - 12 edges
7. `handle_connection()` - 11 edges
8. `orchestrate_claude_tui()` - 10 edges
9. `App` - 9 edges
10. `Terminal<S>` - 9 edges

## Surprising Connections (you probably didn't know these)
- `drive_claude_tui()` --calls--> `VTermClient`  [INFERRED]
  scripts\claude_orchestrator.py → vterm-python\src\lib.rs
- `reality_check_claude()` --calls--> `VTermClient`  [INFERRED]
  scripts\reality_check_claude.py → vterm-python\src\lib.rs
- `reality_check_claude()` --calls--> `close()`  [INFERRED]
  scripts\reality_check_claude.py → vterm-python\vterm_python\server.py
- `terminal_bridge()` --calls--> `VTermClient`  [INFERRED]
  scripts\terminal_bridge.py → vterm-python\src\lib.rs
- `terminal_close()` --calls--> `close()`  [INFERRED]
  tests\python_sdk\test_fastmcp.py → vterm-python\vterm_python\server.py

## Communities

### Community 0 - "Community 0"
Cohesion: 0.09
Nodes (22): Simulates cloning a repo and running a CI build, monitoring the terminal until c, trigger_and_monitor_build(), check_disk_space(), ping_internal_services(), Pings an internal service to verify network connectivity., Spawns a terminal, checks system disk space, and returns the result., inspect_docker_container(), Spawns a terminal, execs into a running Docker container, and fetches the top pr (+14 more)

### Community 1 - "Community 1"
Cohesion: 0.08
Nodes (15): App, Owned, WatchdogEntry, CommandResult, Dispatcher, execute(), Ready, Sealed (+7 more)

### Community 2 - "Community 2"
Cohesion: 0.15
Nodes (17): AppBuilder, OrchestratorClient, connect_with_retry(), read_response(), send_request(), test_supreme_orchestration(), spawn(), spawn_viewer_window() (+9 more)

### Community 3 - "Community 3"
Cohesion: 0.1
Nodes (25): drive_claude_tui(), memchr_subseq(), pump_loop(), start(), viewer_loop(), reality_check_claude(), Spawns a new terminal session. Returns the terminal ID., Writes text to a terminal. Supports <Enter>, <C-c>, etc. (+17 more)

### Community 4 - "Community 4"
Cohesion: 0.07
Nodes (12): batch_sub_results_serialize(), BatchArgs, Request, Response, response_omits_none(), SkillCommand, SpawnArgs, Status (+4 more)

### Community 5 - "Community 5"
Cohesion: 0.12
Nodes (5): Inner, Terminal<S>, Terminal<state::Spawning>, wait(), spawn()

### Community 6 - "Community 6"
Cohesion: 0.2
Nodes (3): expand(), parse(), token_re()

### Community 7 - "Community 7"
Cohesion: 0.28
Nodes (3): Correlation<S>, Timing<S>, Tracing<S>

### Community 8 - "Community 8"
Cohesion: 0.33
Nodes (3): pipeline(), Tracing, TracingLayer

### Community 9 - "Community 9"
Cohesion: 0.4
Nodes (4): McpIdArgs, McpSpawnArgs, McpWaitArgs, McpWriteArgs

### Community 10 - "Community 10"
Cohesion: 0.5
Nodes (2): Correlation, CorrelationLayer

### Community 11 - "Community 11"
Cohesion: 0.5
Nodes (2): Timing, TimingLayer

### Community 14 - "Community 14"
Cohesion: 1.0
Nodes (1): Error

## Knowledge Gaps
- **35 isolated node(s):** `Simulates cloning a repo and running a CI build, monitoring the terminal until c`, `Spawns a terminal, checks system disk space, and returns the result.`, `Pings an internal service to verify network connectivity.`, `Spawns a terminal, execs into a running Docker container, and fetches the top pr`, `Demonstrates the 'Brick-Proof' guardrails of vterm-rs.     Uses the 'Batch' API` (+30 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 10`** (4 nodes): `Correlation`, `CorrelationLayer`, `.layer()`, `correlation.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 11`** (4 nodes): `Timing`, `TimingLayer`, `.layer()`, `timing.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 14`** (2 nodes): `Error`, `error.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `execute()` connect `Community 1` to `Community 0`, `Community 2`, `Community 3`, `Community 5`, `Community 6`?**
  _High betweenness centrality (0.162) - this node is a cross-community bridge._
- **Why does `VTermClient` connect `Community 0` to `Community 2`, `Community 3`?**
  _High betweenness centrality (0.099) - this node is a cross-community bridge._
- **Why does `spawn()` connect `Community 3` to `Community 0`, `Community 1`, `Community 2`, `Community 5`?**
  _High betweenness centrality (0.084) - this node is a cross-community bridge._
- **Are the 7 inferred relationships involving `VTermClient` (e.g. with `run_risky_build()` and `manage_terminal_fleet()`) actually correct?**
  _`VTermClient` has 7 INFERRED edges - model-reasoned connections that need verification._
- **Are the 18 inferred relationships involving `spawn()` (e.g. with `drive_claude_tui()` and `reality_check_claude()`) actually correct?**
  _`spawn()` has 18 INFERRED edges - model-reasoned connections that need verification._
- **Are the 15 inferred relationships involving `execute()` (e.g. with `.ok()` and `spawn()`) actually correct?**
  _`execute()` has 15 INFERRED edges - model-reasoned connections that need verification._
- **Are the 8 inferred relationships involving `main()` (e.g. with `parse()` and `.ok()`) actually correct?**
  _`main()` has 8 INFERRED edges - model-reasoned connections that need verification._