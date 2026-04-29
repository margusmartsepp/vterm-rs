# Graph Report - term-rs  (2026-04-29)

## Corpus Check
- 60 files · ~35,723 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 394 nodes · 738 edges · 43 communities detected
- Extraction: 53% EXTRACTED · 47% INFERRED · 0% AMBIGUOUS · INFERRED: 347 edges (avg confidence: 0.8)
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
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 28|Community 28]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 32|Community 32]]
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]
- [[_COMMUNITY_Community 35|Community 35]]
- [[_COMMUNITY_Community 36|Community 36]]
- [[_COMMUNITY_Community 37|Community 37]]
- [[_COMMUNITY_Community 38|Community 38]]
- [[_COMMUNITY_Community 39|Community 39]]
- [[_COMMUNITY_Community 40|Community 40]]
- [[_COMMUNITY_Community 41|Community 41]]
- [[_COMMUNITY_Community 42|Community 42]]
- [[_COMMUNITY_Community 43|Community 43]]
- [[_COMMUNITY_Community 44|Community 44]]
- [[_COMMUNITY_Community 45|Community 45]]
- [[_COMMUNITY_Community 46|Community 46]]
- [[_COMMUNITY_Community 47|Community 47]]
- [[_COMMUNITY_Community 48|Community 48]]
- [[_COMMUNITY_Community 49|Community 49]]
- [[_COMMUNITY_Community 50|Community 50]]
- [[_COMMUNITY_Community 51|Community 51]]

## God Nodes (most connected - your core abstractions)
1. `execute()` - 33 edges
2. `VTermClient` - 32 edges
3. `spawn()` - 32 edges
4. `close()` - 22 edges
5. `write()` - 21 edges
6. `batch()` - 19 edges
7. `App` - 16 edges
8. `TerminalServer` - 15 edges
9. `main()` - 15 edges
10. `read()` - 15 edges

## Surprising Connections (you probably didn't know these)
- `main()` --calls--> `spawn()`  [INFERRED]
  examples\batch_efficiency.py → vterm-python\vterm_python\server.py
- `run_audit_dance()` --calls--> `VTermClient`  [INFERRED]
  gemini\fleet_audit_dance.py → vterm-python\src\lib.rs
- `run_audit_dance()` --calls--> `batch()`  [INFERRED]
  gemini\fleet_audit_dance.py → vterm-python\vterm_python\server.py
- `run_audit_dance()` --calls--> `write()`  [INFERRED]
  gemini\fleet_audit_dance.py → vterm-python\vterm_python\server.py
- `run_research()` --calls--> `VTermClient`  [INFERRED]
  gemini\research_inconsistencies.py → vterm-python\src\lib.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.05
Nodes (31): main(), Simulates cloning a repo and running a CI build, monitoring the terminal until c, trigger_and_monitor_build(), check_disk_space(), ping_internal_services(), Pings an internal service to verify network connectivity., Spawns a terminal, checks system disk space, and returns the result., inspect_docker_container() (+23 more)

### Community 1 - "Community 1"
Cohesion: 0.09
Nodes (25): AppBuilder, connect_with_retry(), read_response(), send_request(), test_supreme_orchestration(), test_batch_extraction_parity(), test_inspect_parity(), test_stability_parity() (+17 more)

### Community 2 - "Community 2"
Cohesion: 0.08
Nodes (31): drive_claude_tui(), architecture_resource(), get_architecture(), get_client(), get_info(), get_process_state(), inspect(), list_terminals() (+23 more)

### Community 3 - "Community 3"
Cohesion: 0.11
Nodes (13): Dispatcher, distill_semantic_summary(), execute(), perform_extraction(), Inner, Ready, Sealed, Spawning (+5 more)

### Community 4 - "Community 4"
Cohesion: 0.12
Nodes (9): Terminal<S>, spawn(), spawn_viewer_window(), SpawnResult, wait(), control(), find_hwnd_for_pid(), set_title() (+1 more)

### Community 5 - "Community 5"
Cohesion: 0.13
Nodes (12): measure_gemini_cli_success(), measure_recovery_time(), Self-healing orchestrator check., Expose 'it's not us' metrics., Deterministic resource tracking., Supervisor, kill_vterm(), main() (+4 more)

### Community 6 - "Community 6"
Cohesion: 0.1
Nodes (7): App, Owned, WatchdogEntry, run_audit_dance(), control(), set_title(), show()

### Community 7 - "Community 7"
Cohesion: 0.15
Nodes (9): main(), memchr_subseq(), pump_loop(), start(), viewer_loop(), run_research(), Writes text to a terminal. Supports <Enter>, <C-c>, etc., write() (+1 more)

### Community 8 - "Community 8"
Cohesion: 0.12
Nodes (12): BatchArgs, CommandResult, Event, HandshakeRequest, HandshakeResponse, MatchEntry, Request, Response (+4 more)

### Community 9 - "Community 9"
Cohesion: 0.2
Nodes (3): expand(), parse(), token_re()

### Community 10 - "Community 10"
Cohesion: 0.27
Nodes (1): OrchestratorClient

### Community 11 - "Community 11"
Cohesion: 0.22
Nodes (8): McpEvalArgs, McpExtractArgs, McpIdArgs, McpReadArgs, McpSpawnArgs, McpWaitArgs, McpWaitStableArgs, McpWriteArgs

### Community 12 - "Community 12"
Cohesion: 0.28
Nodes (3): Correlation<S>, Timing<S>, Tracing<S>

### Community 13 - "Community 13"
Cohesion: 0.22
Nodes (2): batch_aggregate_status_serialises(), response_carries_req_id_back()

### Community 14 - "Community 14"
Cohesion: 0.5
Nodes (2): Correlation, CorrelationLayer

### Community 15 - "Community 15"
Cohesion: 0.5
Nodes (2): Timing, TimingLayer

### Community 16 - "Community 16"
Cohesion: 0.5
Nodes (2): Tracing, TracingLayer

### Community 17 - "Community 17"
Cohesion: 0.67
Nodes (2): Cli, Commands

### Community 21 - "Community 21"
Cohesion: 1.0
Nodes (1): Error

### Community 28 - "Community 28"
Cohesion: 1.0
Nodes (1): Blocks until a regex pattern appears on the terminal screen.

### Community 29 - "Community 29"
Cohesion: 1.0
Nodes (1): Returns the current process state (running: bool, exit_code: int?).

### Community 30 - "Community 30"
Cohesion: 1.0
Nodes (1): Writes text to a terminal. Supports <Enter>, <C-c>, etc.

### Community 31 - "Community 31"
Cohesion: 1.0
Nodes (1): Reads the current contents of the terminal screen. Set history=True for full scr

### Community 32 - "Community 32"
Cohesion: 1.0
Nodes (1): Executes a batch of commands atomically for high performance.

### Community 33 - "Community 33"
Cohesion: 1.0
Nodes (1): Closes a terminal session.

### Community 34 - "Community 34"
Cohesion: 1.0
Nodes (1): Returns the architectural knowledge graph report for this codebase.

### Community 35 - "Community 35"
Cohesion: 1.0
Nodes (1): The definitive architectural map of vterm-rs.

### Community 36 - "Community 36"
Cohesion: 1.0
Nodes (1): Spawns a new terminal session. Returns the terminal ID.

### Community 37 - "Community 37"
Cohesion: 1.0
Nodes (1): Writes text to a terminal. Supports <Enter>, <C-c>, etc.

### Community 38 - "Community 38"
Cohesion: 1.0
Nodes (1): Blocks until a regex pattern appears on the terminal screen.

### Community 39 - "Community 39"
Cohesion: 1.0
Nodes (1): Writes text to a terminal. Supports <Enter>, <C-c>, etc.

### Community 40 - "Community 40"
Cohesion: 1.0
Nodes (1): Reads the current contents of the terminal screen. Set history=True for full scr

### Community 41 - "Community 41"
Cohesion: 1.0
Nodes (1): Waits for a pattern to appear on the screen.

### Community 42 - "Community 42"
Cohesion: 1.0
Nodes (1): Executes a batch of commands atomically for high performance.

### Community 43 - "Community 43"
Cohesion: 1.0
Nodes (1): Closes a terminal session.

### Community 44 - "Community 44"
Cohesion: 1.0
Nodes (1): Returns the architectural knowledge graph report for this codebase.

### Community 45 - "Community 45"
Cohesion: 1.0
Nodes (1): The definitive architectural map of vterm-rs.

### Community 46 - "Community 46"
Cohesion: 1.0
Nodes (1): Reads the current screen state of a terminal.

### Community 47 - "Community 47"
Cohesion: 1.0
Nodes (1): Waits for a pattern to appear on the screen.

### Community 48 - "Community 48"
Cohesion: 1.0
Nodes (1): Executes a batch of commands atomically for high performance.

### Community 49 - "Community 49"
Cohesion: 1.0
Nodes (1): Closes a terminal session.

### Community 50 - "Community 50"
Cohesion: 1.0
Nodes (1): Returns the architectural knowledge graph report for this codebase.

### Community 51 - "Community 51"
Cohesion: 1.0
Nodes (1): The definitive architectural map of vterm-rs.

## Knowledge Gaps
- **82 isolated node(s):** `Simulates cloning a repo and running a CI build, monitoring the terminal until c`, `Spawns a terminal, checks system disk space, and returns the result.`, `Pings an internal service to verify network connectivity.`, `Spawns a terminal, execs into a running Docker container, and fetches the top pr`, `Demonstrates the 'Brick-Proof' guardrails of vterm-rs.     Uses the 'Batch' API` (+77 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 10`** (11 nodes): `OrchestratorClient`, `.events()`, `.execute()`, `.execute_full()`, `.clone()`, `.drop()`, `.batch()`, `.rust_eval()`, `.spawn()`, `.wait_until()`, `client.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 13`** (9 nodes): `batch_aggregate_status_serialises()`, `deeply_nested_batch_round_trip()`, `request_envelope_round_trip()`, `request_without_req_id()`, `response_carries_req_id_back()`, `shortcut_parser_handles_ctrl_c()`, `shortcut_parser_handles_vim_quit()`, `variant_names_are_stable()`, `protocol.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 14`** (4 nodes): `Correlation`, `CorrelationLayer`, `.layer()`, `correlation.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 15`** (4 nodes): `Timing`, `TimingLayer`, `.layer()`, `timing.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 16`** (4 nodes): `Tracing`, `TracingLayer`, `.layer()`, `tracing_layer.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 17`** (3 nodes): `Cli`, `Commands`, `vterm-ctrl.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 21`** (2 nodes): `Error`, `error.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 28`** (1 nodes): `Blocks until a regex pattern appears on the terminal screen.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 29`** (1 nodes): `Returns the current process state (running: bool, exit_code: int?).`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 30`** (1 nodes): `Writes text to a terminal. Supports <Enter>, <C-c>, etc.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 31`** (1 nodes): `Reads the current contents of the terminal screen. Set history=True for full scr`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 32`** (1 nodes): `Executes a batch of commands atomically for high performance.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 33`** (1 nodes): `Closes a terminal session.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 34`** (1 nodes): `Returns the architectural knowledge graph report for this codebase.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 35`** (1 nodes): `The definitive architectural map of vterm-rs.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 36`** (1 nodes): `Spawns a new terminal session. Returns the terminal ID.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 37`** (1 nodes): `Writes text to a terminal. Supports <Enter>, <C-c>, etc.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 38`** (1 nodes): `Blocks until a regex pattern appears on the terminal screen.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 39`** (1 nodes): `Writes text to a terminal. Supports <Enter>, <C-c>, etc.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 40`** (1 nodes): `Reads the current contents of the terminal screen. Set history=True for full scr`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 41`** (1 nodes): `Waits for a pattern to appear on the screen.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 42`** (1 nodes): `Executes a batch of commands atomically for high performance.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 43`** (1 nodes): `Closes a terminal session.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 44`** (1 nodes): `Returns the architectural knowledge graph report for this codebase.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 45`** (1 nodes): `The definitive architectural map of vterm-rs.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 46`** (1 nodes): `Reads the current screen state of a terminal.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (1 nodes): `Waits for a pattern to appear on the screen.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 48`** (1 nodes): `Executes a batch of commands atomically for high performance.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 49`** (1 nodes): `Closes a terminal session.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (1 nodes): `Returns the architectural knowledge graph report for this codebase.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 51`** (1 nodes): `The definitive architectural map of vterm-rs.`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `execute()` connect `Community 3` to `Community 0`, `Community 1`, `Community 4`, `Community 5`, `Community 6`, `Community 7`, `Community 9`, `Community 10`?**
  _High betweenness centrality (0.153) - this node is a cross-community bridge._
- **Why does `spawn()` connect `Community 1` to `Community 0`, `Community 2`, `Community 3`, `Community 5`, `Community 7`, `Community 10`?**
  _High betweenness centrality (0.104) - this node is a cross-community bridge._
- **Why does `VTermClient` connect `Community 0` to `Community 1`, `Community 2`, `Community 5`, `Community 6`, `Community 7`?**
  _High betweenness centrality (0.076) - this node is a cross-community bridge._
- **Are the 28 inferred relationships involving `execute()` (e.g. with `.ok()` and `.clone()`) actually correct?**
  _`execute()` has 28 INFERRED edges - model-reasoned connections that need verification._
- **Are the 19 inferred relationships involving `VTermClient` (e.g. with `main()` and `run_risky_build()`) actually correct?**
  _`VTermClient` has 19 INFERRED edges - model-reasoned connections that need verification._
- **Are the 28 inferred relationships involving `spawn()` (e.g. with `main()` and `run_research()`) actually correct?**
  _`spawn()` has 28 INFERRED edges - model-reasoned connections that need verification._
- **Are the 18 inferred relationships involving `close()` (e.g. with `main()` and `trigger_and_monitor_build()`) actually correct?**
  _`close()` has 18 INFERRED edges - model-reasoned connections that need verification._