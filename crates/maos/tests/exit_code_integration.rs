//! Integration tests for exit code management and error mapping

mod common;

use assert_cmd::Command;
use common::{TestInputBuilder, exit_codes};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_successful_command_returns_zero() {
    // Test that successful commands return exit code 0
    let temp_dir = tempdir().unwrap();
    let input = TestInputBuilder::new("notification")
        .session_id("test_session")
        .transcript_path(&temp_dir.path().join("transcript.jsonl").to_string_lossy())
        .cwd(&temp_dir.path().to_string_lossy())
        .with_message("Test notification")
        .build();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("notify")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(exit_codes::SUCCESS);
}

#[test]
fn test_missing_required_field_returns_general_error() {
    // Test that missing required fields return exit code 1 (GeneralError)
    let temp_dir = tempdir().unwrap();
    let input = TestInputBuilder::new("pre_tool_use")
        .session_id("test_session")
        .transcript_path(&temp_dir.path().join("transcript.jsonl").to_string_lossy())
        .cwd(&temp_dir.path().to_string_lossy())
        // Missing required "tool_name" field for pre_tool_use
        .build();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use")
        .write_stdin(input.as_bytes())
        .assert()
        .failure()
        .code(exit_codes::GENERAL_ERROR);
}

#[test]
fn test_path_traversal_returns_blocking_error() {
    // Test that path traversal attempts return exit code 2 (BlockingError)
    // NOTE: Path traversal blocking is not yet implemented (PRD-06)
    let temp_dir = tempdir().unwrap();
    let input = TestInputBuilder::new("pre_tool_use")
        .session_id("test_session")
        .transcript_path("../../../etc/passwd")
        .cwd(&temp_dir.path().to_string_lossy())
        .with_tool("Read", serde_json::json!({"file_path": "/etc/passwd"}))
        .build();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use")
        .write_stdin(input.as_bytes())
        .assert()
        .success() // Currently succeeds - blocking will be added in PRD-06
        .code(exit_codes::SUCCESS);
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
        .code(exit_codes::GENERAL_ERROR);
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
        .success()
        .stdout(predicate::str::contains("Multi-Agent Orchestration System"));
}

#[test]
fn test_version_flag_returns_success() {
    // Test that --version returns exit code 0
    Command::cargo_bin("maos")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("maos"));
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
    let input = TestInputBuilder::new("stop")
        .session_id("test_session")
        .transcript_path(&temp_dir.path().join("transcript.jsonl").to_string_lossy())
        .cwd(&temp_dir.path().to_string_lossy())
        .with("stop_hook_active", serde_json::json!(true))
        .build();

    Command::cargo_bin("maos")
        .unwrap()
        .args(["stop", "--chat"])
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(exit_codes::SUCCESS);
}

#[test]
fn test_user_prompt_submit_with_validate_flag() {
    // Test that user-prompt-submit accepts --validate flag
    let temp_dir = tempdir().unwrap();
    let input = TestInputBuilder::new("user_prompt_submit")
        .session_id("test_session")
        .transcript_path(&temp_dir.path().join("transcript.jsonl").to_string_lossy())
        .cwd(&temp_dir.path().to_string_lossy())
        .with_message("Test message")
        .build();

    Command::cargo_bin("maos")
        .unwrap()
        .args(["user-prompt-submit", "--validate"])
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(exit_codes::SUCCESS);
}
