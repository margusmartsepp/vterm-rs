//! Unified protocol re-exports from `vterm-protocol`.

pub use vterm_protocol::*;

/// Map the variant name for tracing.
pub fn variant_name(cmd: &SkillCommand) -> &'static str {
    match cmd {
        SkillCommand::Hello { .. } => "hello",
        SkillCommand::Spawn(_) => "spawn",
        SkillCommand::ScreenWrite { .. } => "screen_write",
        SkillCommand::ScreenRead { .. } => "screen_read",
        SkillCommand::ScreenControl { .. } => "screen_control",
        SkillCommand::ScreenClose { .. } => "screen_close",
        SkillCommand::List { .. } => "list",
        SkillCommand::Wait { .. } => "wait",
        SkillCommand::WaitUntil { .. } => "wait_until",
        SkillCommand::WaitUntilStable { .. } => "wait_until_stable",
        SkillCommand::ScreenDiff { .. } => "screen_diff",
        SkillCommand::Batch(_) => "batch",
        SkillCommand::GetProcessState { .. } => "get_process_state",
        SkillCommand::MatchAll { .. } => "match_all",
        SkillCommand::Inspect { .. } => "inspect",
        SkillCommand::Takeover { .. } => "takeover",
        SkillCommand::Extract { .. } => "extract",
        _ => "unknown",
    }
}
