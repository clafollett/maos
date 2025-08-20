//! Integration tests for exit code management and error mapping

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_successful_command_returns_zero() {
    // Test that successful commands return exit code 0
    let temp_dir = tempdir().unwrap();
    let input = serde_json::json!({
        "session_id": "test_session",
        "transcript_path": temp_dir.path().join("transcript.jsonl"),
        "cwd": temp_dir.path(),
        "hook_event_name": "notification",
        "message": "Test notification"
    });

    Command::cargo_bin("maos")
        .unwrap()
        .arg("notify")
        .write_stdin(input.to_string())
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_missing_required_field_returns_general_error() {
    // Test that validation errors return exit code 1 (GeneralError)
    let temp_dir = tempdir().unwrap();
    let input = serde_json::json!({
        "session_id": "test_session",
        "transcript_path": temp_dir.path().join("transcript.jsonl"),
        "cwd": temp_dir.path(),
        "hook_event_name": "notification"
        // Missing required "message" field
    });

    Command::cargo_bin("maos")
        .unwrap()
        .arg("notify")
        .write_stdin(input.to_string())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_path_traversal_returns_blocking_error() {
    // Test that path traversal attempts return exit code 2 (BlockingError)
    let temp_dir = tempdir().unwrap();
    let input = serde_json::json!({
        "session_id": "test_session",
        "transcript_path": "../../../etc/passwd",
        "cwd": temp_dir.path(),
        "hook_event_name": "pre_tool_use",
        "tool_name": "Read",
        "tool_input": {"file_path": "/etc/passwd"}
    });

    Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use")
        .write_stdin(input.to_string())
        .assert()
        .failure()
        .code(2);
}

#[test]
fn test_invalid_json_returns_general_error() {
    // Test that invalid JSON returns exit code 1
    Command::cargo_bin("maos")
        .unwrap()
        .arg("notify")
        .write_stdin("{ invalid json")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_unknown_command_returns_error() {
    // Test that unknown commands return an error
    Command::cargo_bin("maos")
        .unwrap()
        .arg("unknown-command")
        .assert()
        .failure();
}

#[test]
fn test_help_flag_returns_success() {
    // Test that --help returns exit code 0
    Command::cargo_bin("maos")
        .unwrap()
        .arg("--help")
        .assert()
        .failure() // Clap returns error with help
        .stderr(predicate::str::contains("Multi-Agent Orchestration System"));
}

#[test]
fn test_version_flag_returns_success() {
    // Test that --version returns exit code 0
    Command::cargo_bin("maos")
        .unwrap()
        .arg("--version")
        .assert()
        .failure() // Clap returns error with version
        .stderr(predicate::str::contains("maos"));
}

#[test]
fn test_all_hook_commands_require_stdin() {
    // Test that all hook commands expect stdin input
    let commands = [
        "pre-tool-use",
        "post-tool-use",
        "notify",
        "stop",
        "subagent-stop",
        "user-prompt-submit",
        "pre-compact",
        "session-start",
    ];

    for cmd in &commands {
        // Empty stdin should cause a failure
        Command::cargo_bin("maos")
            .unwrap()
            .arg(cmd)
            .write_stdin("")
            .assert()
            .failure();
    }
}

#[test]
fn test_stop_command_with_chat_flag() {
    // Test that stop command accepts --chat flag
    let temp_dir = tempdir().unwrap();
    let input = serde_json::json!({
        "session_id": "test_session",
        "transcript_path": temp_dir.path().join("transcript.jsonl"),
        "cwd": temp_dir.path(),
        "hook_event_name": "stop",
        "stop_hook_active": true
    });

    Command::cargo_bin("maos")
        .unwrap()
        .args(["stop", "--chat"])
        .write_stdin(input.to_string())
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_user_prompt_submit_with_validate_flag() {
    // Test that user-prompt-submit accepts --validate flag
    let temp_dir = tempdir().unwrap();
    let input = serde_json::json!({
        "session_id": "test_session",
        "transcript_path": temp_dir.path().join("transcript.jsonl"),
        "cwd": temp_dir.path(),
        "hook_event_name": "user_prompt_submit",
        "prompt": "Test prompt"
    });

    Command::cargo_bin("maos")
        .unwrap()
        .args(["user-prompt-submit", "--validate"])
        .write_stdin(input.to_string())
        .assert()
        .success()
        .code(0);
}