//! Integration tests for JSON I/O processing

use maos::io::HookInput;
use serde_json::json;

#[test]
fn test_hook_input_compatibility_with_claude_code() {
    // Test each hook type with exact Claude Code JSON format

    // PreToolUse
    let pre_tool_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/Users/alice/.claude/transcripts/sess_12345678.jsonl",
        "cwd": "/Users/alice/projects/myapp",
        "hook_event_name": "pre_tool_use",
        "tool_name": "Bash",
        "tool_input": {
            "command": "cargo test"
        }
    });

    let pre_tool: HookInput = serde_json::from_value(pre_tool_json).unwrap();
    assert_eq!(pre_tool.hook_event_name, "pre_tool_use");
    assert!(pre_tool.validate().is_ok());

    // PostToolUse
    let post_tool_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/Users/alice/.claude/transcripts/sess_12345678.jsonl",
        "cwd": "/Users/alice/projects/myapp",
        "hook_event_name": "post_tool_use",
        "tool_name": "Bash",
        "tool_input": {
            "command": "cargo test"
        },
        "tool_response": {
            "output": "test result: ok. 42 passed",
            "exit_code": 0
        }
    });

    let post_tool: HookInput = serde_json::from_value(post_tool_json).unwrap();
    assert_eq!(post_tool.hook_event_name, "post_tool_use");
    assert!(post_tool.validate().is_ok());

    // Notification
    let notification_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/Users/alice/.claude/transcripts/sess_12345678.jsonl",
        "cwd": "/Users/alice/projects/myapp",
        "hook_event_name": "notification",
        "message": "Task completed successfully"
    });

    let notification: HookInput = serde_json::from_value(notification_json).unwrap();
    assert_eq!(notification.hook_event_name, "notification");
    assert!(notification.validate().is_ok());

    // UserPromptSubmit
    let prompt_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/Users/alice/.claude/transcripts/sess_12345678.jsonl",
        "cwd": "/Users/alice/projects/myapp",
        "hook_event_name": "user_prompt_submit",
        "prompt": "Please help me fix this bug"
    });

    let prompt: HookInput = serde_json::from_value(prompt_json).unwrap();
    assert_eq!(prompt.hook_event_name, "user_prompt_submit");
    assert!(prompt.validate().is_ok());

    // Stop
    let stop_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/Users/alice/.claude/transcripts/sess_12345678.jsonl",
        "cwd": "/Users/alice/projects/myapp",
        "hook_event_name": "stop",
        "stop_hook_active": true
    });

    let stop: HookInput = serde_json::from_value(stop_json).unwrap();
    assert_eq!(stop.hook_event_name, "stop");
    assert!(stop.validate().is_ok());

    // SubagentStop
    let subagent_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/Users/alice/.claude/transcripts/sess_12345678.jsonl",
        "cwd": "/Users/alice/projects/myapp",
        "hook_event_name": "subagent_stop",
        "stop_hook_active": false
    });

    let subagent: HookInput = serde_json::from_value(subagent_json).unwrap();
    assert_eq!(subagent.hook_event_name, "subagent_stop");
    assert!(subagent.validate().is_ok());

    // PreCompact
    let compact_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/Users/alice/.claude/transcripts/sess_12345678.jsonl",
        "cwd": "/Users/alice/projects/myapp",
        "hook_event_name": "pre_compact",
        "trigger": "auto",
        "custom_instructions": "Keep recent context about the bug fix"
    });

    let compact: HookInput = serde_json::from_value(compact_json).unwrap();
    assert_eq!(compact.hook_event_name, "pre_compact");
    assert!(compact.validate().is_ok());

    // SessionStart
    let session_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/Users/alice/.claude/transcripts/sess_12345678.jsonl",
        "cwd": "/Users/alice/projects/myapp",
        "hook_event_name": "session_start",
        "source": "startup"
    });

    let session: HookInput = serde_json::from_value(session_json).unwrap();
    assert_eq!(session.hook_event_name, "session_start");
    assert!(session.validate().is_ok());
}

#[test]
fn test_hook_input_handles_missing_optional_fields() {
    // Stop without stop_hook_active (optional field)
    let minimal_stop = json!({
        "session_id": "sess_123",
        "transcript_path": "/tmp/transcript.jsonl",
        "cwd": "/workspace",
        "hook_event_name": "stop"
    });

    let stop: HookInput = serde_json::from_value(minimal_stop).unwrap();
    assert!(stop.stop_hook_active.is_none());
    assert!(stop.validate().is_ok());
}

#[test]
fn test_hook_input_rejects_invalid_values() {
    // Invalid trigger value for pre_compact
    let invalid_compact = json!({
        "session_id": "sess_123",
        "transcript_path": "/tmp/transcript.jsonl",
        "cwd": "/workspace",
        "hook_event_name": "pre_compact",
        "trigger": "invalid_trigger",
        "custom_instructions": "Keep context"
    });

    let compact: HookInput = serde_json::from_value(invalid_compact).unwrap();
    assert!(compact.validate().is_err());

    // Invalid source value for session_start
    let invalid_session = json!({
        "session_id": "sess_123",
        "transcript_path": "/tmp/transcript.jsonl",
        "cwd": "/workspace",
        "hook_event_name": "session_start",
        "source": "invalid_source"
    });

    let session: HookInput = serde_json::from_value(invalid_session).unwrap();
    assert!(session.validate().is_err());
}

#[test]
fn test_serialization_omits_null_fields() {
    let input = HookInput {
        session_id: "sess_123".to_string(),
        transcript_path: std::path::PathBuf::from("/tmp/transcript.jsonl"),
        cwd: std::path::PathBuf::from("/workspace"),
        hook_event_name: "notification".to_string(),
        tool_name: None,
        tool_input: None,
        tool_response: None,
        message: Some("Test message".to_string()),
        prompt: None,
        stop_hook_active: None,
        trigger: None,
        custom_instructions: None,
        source: None,
    };

    let json_str = serde_json::to_string(&input).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify only non-None fields are present
    assert!(json.get("session_id").is_some());
    assert!(json.get("transcript_path").is_some());
    assert!(json.get("cwd").is_some());
    assert!(json.get("hook_event_name").is_some());
    assert!(json.get("message").is_some());

    // Verify None fields are omitted
    assert!(json.get("tool_name").is_none());
    assert!(json.get("tool_input").is_none());
    assert!(json.get("prompt").is_none());
}
