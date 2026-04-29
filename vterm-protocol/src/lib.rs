//! Single source of truth for the vterm protocol.
//!
//! This crate contains all wire types used by the orchestrator, supervisor, and SDKs.
//! It also provides JSON Schema generation to ensure all tiers stay in sync.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

// ─── Constants ───────────────────────────────────────────────────────────────

pub const DEFAULT_PIPE_NAME: &str = r"\\.\pipe\vterm-rs-skill";

// ─── Envelopes ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Request {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub req_id: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_token: Option<String>,

    #[serde(flatten)]
    pub command: SkillCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub req_id: Option<u64>,

    #[serde(flatten)]
    pub result: CommandResult,
}

// ─── Events (OOB) ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "payload")]
pub enum Event {
    Progress { 
        #[serde(skip_serializing_if = "Option::is_none")]
        req_id: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        token: Option<String>,
        percentage: f32,
        msg: String 
    },
    TerminalOutput { id: u32, content: String },
}

impl Response {
    pub fn error(req_id: Option<u64>, msg: impl Into<String>) -> Self {
        Self {
            req_id,
            result: CommandResult {
                status: Status::Error,
                error: Some(msg.into()),
                ..CommandResult::default()
            },
        }
    }
}

// ─── Handshake ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HandshakeRequest {
    pub client_version: String,
    pub workspace_id: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HandshakeResponse {
    pub server_version: String,
    pub session_id: String,
    pub status: String, // "ok", "conflict", "takeover"
}

// ─── Commands ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type", content = "payload")]
#[non_exhaustive]
pub enum SkillCommand {
    Hello { client_version: String },
    Spawn(SpawnArgs),
    ScreenWrite { id: u32, text: String },
    ScreenRead { 
        id: u32,
        #[serde(default)]
        history: bool,
    },
    ScreenControl { id: u32, action: String },
    ScreenClose {
        #[serde(default)]
        id: Option<u32>,
        #[serde(default = "default_close_target")]
        target: String,
    },
    List { #[serde(default)] all: bool },
    Wait { timeout_ms: u64 },
    WaitUntil { id: u32, pattern: String, timeout_ms: u64 },
    /// Blocks until the screen buffer remains unchanged for `stable_ms`.
    WaitUntilStable { id: u32, stable_ms: u64, timeout_ms: u64 },
    /// Returns only the visual delta since the last observation for this terminal.
    ScreenDiff { id: u32 },
    Batch(BatchArgs),
    GetProcessState { id: u32 },
    MatchAll { pattern: String },
    /// Returns topological and resource metadata for the entire session.
    Inspect { 
        #[serde(default)]
        assurance: bool,
    },
    /// Requests the current instance to release the pipe for a new version.
    Takeover {
        /// Version string of the incoming instance.
        version: String,
    },
    /// Extracts structured data from terminal history using a regex with named groups.
    Extract {
        id: u32,
        pattern: String,
        #[serde(default)]
        history: bool,
    },
}

impl SkillCommand {
    pub const fn variant_name(&self) -> &'static str {
        match self {
            Self::Hello { .. } => "hello",
            Self::Spawn(_) => "spawn",
            Self::ScreenWrite { .. } => "screen_write",
            Self::ScreenRead { .. } => "screen_read",
            Self::ScreenControl { .. } => "screen_control",
            Self::ScreenClose { .. } => "screen_close",
            Self::List { .. } => "list",
            Self::Wait { .. } => "wait",
            Self::WaitUntil { .. } => "wait_until",
            Self::WaitUntilStable { .. } => "wait_until_stable",
            Self::ScreenDiff { .. } => "screen_diff",
            Self::Batch(_) => "batch",
            Self::GetProcessState { .. } => "get_process_state",
            Self::MatchAll { .. } => "match_all",
            Self::Inspect { .. } => "inspect",
            Self::Takeover { .. } => "takeover",
            Self::Extract { .. } => "extract",
        }
    }
}

fn default_close_target() -> String { "single".into() }

#[derive(Debug, Clone, Deserialize, Serialize, Default, JsonSchema)]
pub struct SpawnArgs {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub max_lines: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub visible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub cols: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub rows: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub env: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub wait: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub semantic: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub extract_pattern: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, JsonSchema)]
pub struct BatchArgs {
    pub commands: Vec<SkillCommand>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub stop_on_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub visible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub parallel: Option<bool>,
}

// ─── Results ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Success,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MatchEntry {
    pub id: u32,
    pub matched: bool,
    pub certain: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct CommandResult {
    pub status: Status,
    #[serde(default)] pub duration_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub spawn_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub ready_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub sub_results: Option<Vec<CommandResult>>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub running: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub matches: Option<Vec<MatchEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub extracted: Option<Vec<HashMap<String, String>>>,
    
    // Resource Assurance Fields
    #[serde(default, skip_serializing_if = "Option::is_none")] pub mem_usage_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub handle_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub stall_index: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub active_terminals: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub pool_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub max_terminals: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub max_mem_mb: Option<u64>,
}

impl CommandResult {
    pub fn ok() -> Self { Self::default() }
    pub fn err(msg: impl Into<String>) -> Self {
        Self { status: Status::Error, error: Some(msg.into()), ..Self::default() }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TerminalInfo {
    pub id: u32,
    pub title: String,
    pub pid: u32,
    pub owner: String,
}
