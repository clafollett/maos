//! End-to-end integration tests for MAOS CLI
//!
//! These tests verify the complete CLI execution path including:
//! - Command parsing
//! - JSON input processing
//! - Handler execution
//! - Exit code handling

mod common;

use assert_cmd::Command;
use common::{TestInputBuilder, exit_codes, generators};
use serde_json::json;

/// Test successful pre-tool-use command execution
#[test]
fn test_pre_tool_use_success() {
    let input = generators::pre_tool_use("Read");

    Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(exit_codes::SUCCESS);
}

/// Test pre-tool-use with potentially dangerous command
/// NOTE: Blocking is not yet implemented (PRD-06), so this currently succeeds
#[test]
fn test_pre_tool_use_dangerous_command() {
    let input = TestInputBuilder::new("pre_tool_use")
        .with_tool(
            "Bash",
            json!({
                "command": "rm -rf /"
            }),
        )
        .build();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use")
        .write_stdin(input.as_bytes())
        .assert()
        .success() // Currently succeeds - blocking will be added in PRD-06
        .code(exit_codes::SUCCESS);
}

/// Test post-tool-use command execution
#[test]
fn test_post_tool_use_success() {
    let input = generators::post_tool_use("Read", "File contents here");

    Command::cargo_bin("maos")
        .unwrap()
        .arg("post-tool-use")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(exit_codes::SUCCESS);
}

/// Test notification command
#[test]
fn test_notify_command() {
    let input = TestInputBuilder::new("notification")
        .with_message("Test notification")
        .with("urgency", json!("medium"))
        .build();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("notify")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(exit_codes::SUCCESS);
}

/// Test session-start command
#[test]
fn test_session_start_command() {
    let input = generators::session_start();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("session-start")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(exit_codes::SUCCESS);
}

/// Test stop command with --chat flag
#[test]
fn test_stop_command_with_chat() {
    let input = TestInputBuilder::new("stop")
        .with("stop_hook_active", json!(false))
        .build();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("stop")
        .arg("--chat")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(exit_codes::SUCCESS);
}

/// Test user-prompt-submit with --validate flag
#[test]
fn test_user_prompt_submit_with_validate() {
    let input = TestInputBuilder::new("user_prompt_submit")
        .with_message("Test user input")
        .build();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("user-prompt-submit")
        .arg("--validate")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(exit_codes::SUCCESS);
}

/// Test malformed JSON input handling
#[test]
fn test_malformed_json_input() {
    let input = "{ this is not valid json }";

    Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use")
        .write_stdin(input.as_bytes())
        .assert()
        .failure()
        .code(exit_codes::GENERAL_ERROR);
}

/// Test oversized input handling
#[test]
fn test_oversized_input() {
    // Create a 15MB input (exceeds 10MB limit)
    let huge_data = "x".repeat(15 * 1024 * 1024);
    let input = json!({
        "session_id": "test",
        "transcript_path": "/tmp/t.jsonl",
        "cwd": "/tmp",
        "hook_event_name": "pre_tool_use",
        "tool_name": "Read",
        "tool_input": {
            "data": huge_data
        }
    })
    .to_string();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use")
        .write_stdin(input.as_bytes())
        .assert()
        .failure()
        .code(exit_codes::GENERAL_ERROR);
}

/// Test empty stdin handling
#[test]
fn test_empty_stdin() {
    Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use")
        .write_stdin(b"")
        .assert()
        .failure()
        .code(exit_codes::GENERAL_ERROR);
}

/// Test wrong hook event for command
#[test]
fn test_wrong_hook_event() {
    let input = TestInputBuilder::new("post_tool_use")
        .with("tool_name", json!("Read"))
        .build();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-tool-use") // Wrong command for post_tool_use event
        .write_stdin(input.as_bytes())
        .assert()
        .failure()
        .code(exit_codes::GENERAL_ERROR);
}

/// Test concurrent command execution
#[test]
fn test_concurrent_execution() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let success_count = Arc::clone(&success_count);
        let handle = thread::spawn(move || {
            let input = generators::notification("Concurrent test");

            let output = Command::cargo_bin("maos")
                .unwrap()
                .arg("notify")
                .write_stdin(input.as_bytes())
                .output()
                .expect("Failed to execute command");

            if output.status.success() {
                success_count.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(success_count.load(Ordering::SeqCst), 10);
}

/// Test performance - startup time should be < 20ms
#[test]
fn test_startup_performance() {
    use std::time::Instant;

    let input = generators::notification("Performance test");

    let start = Instant::now();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("notify")
        .write_stdin(input.as_bytes())
        .assert()
        .success();

    let duration = start.elapsed();

    // Performance expectations differ between debug and release builds
    #[cfg(debug_assertions)]
    let max_duration_ms = 300; // Debug builds are much slower
    #[cfg(not(debug_assertions))]
    let max_duration_ms = 20; // Release builds should be fast

    assert!(
        duration.as_millis() < max_duration_ms,
        "Command took {}ms, expected < {}ms",
        duration.as_millis(),
        max_duration_ms
    );
}

/// Test subagent-stop command
#[test]
fn test_subagent_stop_command() {
    let input = TestInputBuilder::new("subagent_stop")
        .with("agent_id", json!("test-agent-123"))
        .build();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("subagent-stop")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(exit_codes::SUCCESS);
}

/// Test pre-compact command
#[test]
fn test_pre_compact_command() {
    let input = TestInputBuilder::new("pre_compact").build();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("pre-compact")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(exit_codes::SUCCESS);
}

/// Test command timeout handling
#[test]
#[ignore] // This test might be slow
fn test_command_timeout() {
    use std::thread;
    use std::time::Duration;

    // Create input that would cause a long operation
    let input = TestInputBuilder::new("pre_tool_use")
        .with_tool("LongRunning", json!({}))
        .build();

    let handle = thread::spawn(move || {
        Command::cargo_bin("maos")
            .unwrap()
            .arg("pre-tool-use")
            .timeout(Duration::from_secs(2))
            .write_stdin(input.as_bytes())
            .assert()
            .failure();
    });

    // Should timeout and not hang indefinitely
    handle.join().unwrap();
}
