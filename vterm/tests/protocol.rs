//! Protocol-level integration tests. These run without spawning a single PTY — they
//! exercise the wire types, the Tower pipeline, and the dispatcher against a stubbed
//! `App`-equivalent (well, against the real wire types).
//!
//! Run with: `cargo test`.

use vterm_rs::protocol::{
    BatchArgs, CommandResult, Request, Response, SkillCommand, SpawnArgs, Status,
};

#[test]
fn request_envelope_round_trip() {
    let json = r#"{"req_id":42,"type":"Wait","payload":{"timeout_ms":250}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    assert_eq!(req.req_id, Some(42));
    assert!(matches!(
        req.command,
        SkillCommand::Wait { timeout_ms: 250 }
    ));
}

#[test]
fn request_without_req_id() {
    let json = r#"{"type":"List", "payload": {}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    assert_eq!(req.req_id, None);
    assert!(matches!(req.command, SkillCommand::List { .. }));
}

#[test]
fn deeply_nested_batch_round_trip() {
    // The exact shape that PowerShell's default ConvertTo-Json depth used to truncate.
    let json = r#"{
        "req_id": 1,
        "type": "Batch",
        "payload": {
            "visible": false,
            "stop_on_error": true,
            "commands": [
                { "type": "Spawn", "payload": { "title": "build" } },
                { "type": "ScreenWrite", "payload": { "id": 1, "text": "cargo build<Enter>" } },
                { "type": "WaitUntil", "payload": { "id": 1, "pattern": "Compiling", "timeout_ms": 30000 } }
            ]
        }
    }"#;
    let req: Request = serde_json::from_str(json).unwrap();
    let SkillCommand::Batch(b) = req.command else {
        panic!("expected Batch")
    };
    assert_eq!(b.commands.len(), 3);
    assert_eq!(b.stop_on_error, Some(true));
    assert_eq!(b.visible, Some(false));
}

#[test]
fn response_carries_req_id_back() {
    let r = Response {
        req_id: Some(99),
        result: CommandResult {
            id: Some(1),
            ..CommandResult::default()
        },
    };
    let s = serde_json::to_string(&r).unwrap();
    assert!(s.contains(r#""req_id":99"#));
    assert!(s.contains(r#""status":"success""#));
    assert!(s.contains(r#""id":1"#));
}

#[test]
fn batch_aggregate_status_serialises() {
    let r = Response {
        req_id: Some(2),
        result: CommandResult {
            status: Status::Error,
            error: Some("1 sub-command(s) failed".into()),
            sub_results: Some(vec![
                CommandResult::ok(),
                CommandResult::err("WaitUntil timeout"),
            ]),
            ..CommandResult::default()
        },
    };
    let s = serde_json::to_string(&r).unwrap();
    assert!(s.contains(r#""status":"error""#));
    assert!(s.contains(r#""sub_results":[{"status":"success""#));
    assert!(s.contains(r#""error":"WaitUntil timeout""#));
}

#[test]
fn shortcut_parser_handles_vim_quit() {
    use vterm_rs::shortcuts;
    assert_eq!(shortcuts::parse("<Esc>:q!<Enter>"), b"\x1b:q!\r\n");
}

#[test]
fn shortcut_parser_handles_ctrl_c() {
    use vterm_rs::shortcuts;
    assert_eq!(shortcuts::parse("<C-c>"), &[0x03]);
}

#[test]
fn variant_names_are_stable() {
    assert_eq!(SkillCommand::List {}.variant_name(), "list");
    assert_eq!(
        SkillCommand::Spawn(SpawnArgs {
            title: "x".into(),
            ..SpawnArgs::default()
        })
        .variant_name(),
        "spawn",
    );
    assert_eq!(
        SkillCommand::Batch(BatchArgs::default()).variant_name(),
        "batch",
    );
    assert_eq!(
        SkillCommand::Takeover {
            version: "1.0.0".into()
        }
        .variant_name(),
        "takeover",
    );
}
