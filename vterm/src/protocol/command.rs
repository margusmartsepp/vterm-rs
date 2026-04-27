//! Wire types: `Request`, `Response`, `SkillCommand`, `CommandResult`.
//!
//! A few patterns worth noting if you're reading this for the first time:
//!
//! * The request envelope uses `#[serde(flatten)]` so that `req_id` lives at the JSON
//!   top level alongside the internally-tagged `SkillCommand` (`type`, `payload`). This
//!   keeps the wire shape flat without sacrificing the elegance of internally-tagged
//!   enum dispatch in Rust.
//! * `Status` is a `Copy` enum with `#[serde(rename_all = "lowercase")]` rather than a
//!   bare `String` — typos become compile errors, not runtime mysteries.
//! * `CommandResult` defaults to `success` because that's the dominant case.
//! * `sub_results` is `Option<Vec<_>>` rather than `Vec<_>`: missing means
//!   "this wasn't a Batch" and serialises away cleanly.

use serde::{Deserialize, Serialize};

// ─── envelope ─────────────────────────────────────────────────────────────────

/// One inbound request. The `req_id` is opaque to the server: if the client supplies
/// it, the server echoes it on the response. The `command` field is flattened so the
/// wire JSON looks like `{"req_id": 7, "type": "Spawn", "payload": {...}}`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Request {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub req_id: Option<u64>,

    #[serde(flatten)]
    pub command: SkillCommand,
}

/// One outbound response. Carries the original `req_id` and the [`CommandResult`]
/// produced by the dispatcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub req_id: Option<u64>,

    #[serde(flatten)]
    pub result: CommandResult,
}

impl Response {
    /// Construct a synthetic error response — used for bottom-of-the-stack failures
    /// (parse errors, service unavailable) where we never reached the dispatcher.
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

// ─── commands ─────────────────────────────────────────────────────────────────

/// The closed set of operations the orchestrator understands.
///
/// Internally tagged: `{"type": "Spawn", "payload": {...}}`. New variants are added
/// as the protocol evolves; clients are expected to tolerate unknown variants in
/// responses (they do, because `Response` doesn't carry an enum here).
#[derive(Debug, Clone, Deserialize, Serialize)]
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
    List {},
    Wait { ms: u64 },
    WaitUntil { id: u32, pattern: String, timeout_ms: u64 },
    Batch(BatchArgs),
    GetProcessState { id: u32 },
}

fn default_close_target() -> String { "single".into() }

impl SkillCommand {
    /// Stable string used as a tracing span field. Keep this in lockstep with the
    /// variant set.
    pub const fn variant_name(&self) -> &'static str {
        match self {
            Self::Hello { .. }        => "hello",
            Self::Spawn(_)            => "spawn",
            Self::ScreenWrite { .. }  => "screen_write",
            Self::ScreenRead { .. }   => "screen_read",
            Self::ScreenControl { .. }=> "screen_control",
            Self::ScreenClose { .. }  => "screen_close",
            Self::List { .. }         => "list",
            Self::Wait { .. }         => "wait",
            Self::WaitUntil { .. }    => "wait_until",
            Self::Batch(_)            => "batch",
            Self::GetProcessState { .. } => "get_process_state",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SpawnArgs {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub max_lines: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub visible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub cols: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub rows: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub env: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BatchArgs {
    pub commands: Vec<SkillCommand>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub stop_on_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub visible: Option<bool>,
}

// ─── results ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Success,
    Error,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandResult {
    pub status: Status,
    #[serde(default)] pub duration_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub sub_results: Option<Vec<CommandResult>>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub running: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub exit_code: Option<i32>,
}

impl CommandResult {
    pub fn ok() -> Self { Self::default() }
    pub fn err(msg: impl Into<String>) -> Self {
        Self { status: Status::Error, error: Some(msg.into()), ..Self::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_round_trip() {
        let json = r#"{"req_id":7,"type":"ScreenWrite","payload":{"id":1,"text":"x"}}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        assert_eq!(req.req_id, Some(7));
        match req.command {
            SkillCommand::ScreenWrite { id, text } => {
                assert_eq!(id, 1);
                assert_eq!(text, "x");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn response_omits_none() {
        let r = Response { req_id: None, result: CommandResult::ok() };
        let s = serde_json::to_string(&r).unwrap();
        assert_eq!(s, r#"{"status":"success","duration_ms":0}"#);
    }

    #[test]
    fn batch_sub_results_serialize() {
        let r = CommandResult {
            status: Status::Error,
            sub_results: Some(vec![CommandResult::ok(), CommandResult::err("x")]),
            error: Some("1 sub-command(s) failed".into()),
            ..CommandResult::default()
        };
        let s = serde_json::to_string(&r).unwrap();
        assert!(s.contains(r#""sub_results":[{"status":"success""#));
        assert!(s.contains(r#""error":"1 sub-command(s) failed""#));
    }
}
