//! Cross-platform compatibility tests for MAOS CLI
//!
//! These tests verify correct behavior across different operating systems

mod common;

use assert_cmd::Command;
use common::platform;
use serde_json::json;

// Use platform helpers from common module instead of duplicating

/// Test path handling on different platforms
#[test]
fn test_cross_platform_paths() {
    let transcript_path = platform::transcript_path();
    let cwd = platform::test_path();

    let input = json!({
        "session_id": "test-session",
        "transcript_path": transcript_path,
        "cwd": cwd,
        "hook_event_name": "session_start",
    })
    .to_string();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("session-start")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .code(0);
}

/// Test Windows-style paths on Unix (should be handled gracefully)
#[test]
#[cfg(unix)]
fn test_windows_paths_on_unix() {
    let input = json!({
        "session_id": "test-session",
        "transcript_path": "C:\\Windows\\Temp\\transcript.jsonl",
        "cwd": "C:\\Temp\\test",
        "hook_event_name": "session_start",
    })
    .to_string();

    // Should handle Windows paths gracefully on Unix
    Command::cargo_bin("maos")
        .unwrap()
        .arg("session-start")
        .write_stdin(input.as_bytes())
        .assert()
        .success() // Should not crash
        .code(0);
}

/// Test Unix-style paths on Windows (should be handled gracefully)
#[test]
#[cfg(windows)]
fn test_unix_paths_on_windows() {
    let input = json!({
        "session_id": "test-session",
        "transcript_path": "/tmp/transcript.jsonl",
        "cwd": "/tmp/test",
        "hook_event_name": "session_start",
    })
    .to_string();

    // Should handle Unix paths gracefully on Windows
    Command::cargo_bin("maos")
        .unwrap()
        .arg("session-start")
        .write_stdin(input.as_bytes())
        .assert()
        .success() // Should not crash
        .code(0);
}

/// Test path normalization
#[test]
fn test_path_normalization() {
    // Mixed separators should be normalized
    let paths = vec![
        "/tmp//test///file.txt",
        "/tmp/./test/../test/file.txt",
        "//tmp/test/file.txt",
    ];

    for path in paths {
        let input = json!({
            "session_id": "test-session",
            "transcript_path": path,
            "cwd": "/tmp",
            "hook_event_name": "session_start",
        })
        .to_string();

        Command::cargo_bin("maos")
            .unwrap()
            .arg("session-start")
            .write_stdin(input.as_bytes())
            .assert()
            .success();
    }
}

/// Test Unicode paths
#[test]
fn test_unicode_paths() {
    let unicode_paths = vec![
        "/tmp/测试/文件.txt",       // Chinese
        "/tmp/тест/файл.txt",       // Russian
        "/tmp/テスト/ファイル.txt", // Japanese
        "/tmp/🚀/📁.txt",           // Emoji
    ];

    for path in unicode_paths {
        let input = json!({
            "session_id": "test-unicode",
            "transcript_path": path,
            "cwd": "/tmp",
            "hook_event_name": "session_start",
        })
        .to_string();

        Command::cargo_bin("maos")
            .unwrap()
            .arg("session-start")
            .write_stdin(input.as_bytes())
            .assert()
            .success()
            .code(0);
    }
}

/// Test environment variable expansion differences
#[test]
fn test_environment_variables() {
    let home_var = if cfg!(windows) {
        "$USERPROFILE"
    } else {
        "$HOME"
    };

    let input = json!({
        "session_id": "test-env",
        "transcript_path": format!("{}/transcript.jsonl", home_var),
        "cwd": home_var,
        "hook_event_name": "session_start",
    })
    .to_string();

    // Should handle environment variables (or at least not crash)
    Command::cargo_bin("maos")
        .unwrap()
        .arg("session-start")
        .write_stdin(input.as_bytes())
        .assert()
        .success();
}

/// Test line ending differences (CRLF vs LF)
#[test]
fn test_line_endings() {
    // Test with Windows-style CRLF
    let input_crlf = "{\r\n  \"session_id\": \"test\",\r\n  \"transcript_path\": \"/tmp/t.jsonl\",\r\n  \"cwd\": \"/tmp\",\r\n  \"hook_event_name\": \"session_start\"\r\n}";

    Command::cargo_bin("maos")
        .unwrap()
        .arg("session-start")
        .write_stdin(input_crlf.as_bytes())
        .assert()
        .success();

    // Test with Unix-style LF
    let input_lf = "{\n  \"session_id\": \"test\",\n  \"transcript_path\": \"/tmp/t.jsonl\",\n  \"cwd\": \"/tmp\",\n  \"hook_event_name\": \"session_start\"\n}";

    Command::cargo_bin("maos")
        .unwrap()
        .arg("session-start")
        .write_stdin(input_lf.as_bytes())
        .assert()
        .success();
}

/// Test case sensitivity differences
#[test]
fn test_case_sensitivity() {
    // On case-insensitive filesystems (Windows, macOS by default),
    // these paths might be equivalent
    let paths = vec![
        "/tmp/Test/File.txt",
        "/TMP/test/file.txt",
        "/Tmp/TEST/FILE.TXT",
    ];

    for path in paths {
        let input = json!({
            "session_id": "test-case",
            "transcript_path": path,
            "cwd": "/tmp",
            "hook_event_name": "session_start",
        })
        .to_string();

        Command::cargo_bin("maos")
            .unwrap()
            .arg("session-start")
            .write_stdin(input.as_bytes())
            .assert()
            .success();
    }
}

/// Test platform-specific special characters in paths
#[test]
fn test_special_characters() {
    // Characters that might be problematic on different platforms
    let test_cases = vec![
        ("spaces in path", "/tmp/my folder/file.txt"),
        ("parentheses", "/tmp/folder(1)/file.txt"),
        ("brackets", "/tmp/folder[test]/file.txt"),
        ("ampersand", "/tmp/folder&files/file.txt"),
        ("at sign", "/tmp/@folder/file.txt"),
        ("hash", "/tmp/#folder/file.txt"),
    ];

    for (description, path) in test_cases {
        let input = json!({
            "session_id": format!("test-{}", description.replace(' ', "-")),
            "transcript_path": path,
            "cwd": "/tmp",
            "hook_event_name": "session_start",
        })
        .to_string();

        Command::cargo_bin("maos")
            .unwrap()
            .arg("session-start")
            .write_stdin(input.as_bytes())
            .assert()
            .success()
            .code(0);
    }
}

/// Test binary execution across platforms
#[test]
fn test_binary_portability() {
    // Test that the binary can be invoked without extension
    Command::cargo_bin("maos")
        .unwrap()
        .arg("--version")
        .assert()
        .success();

    // Test help works the same way
    Command::cargo_bin("maos")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Multi-Agent Orchestration System",
        ));
}

/// Test stdin handling across platforms
#[test]
fn test_stdin_handling() {
    // Test piping stdin using assert_cmd
    let input = json!({
        "session_id": "test-stdin",
        "transcript_path": "/tmp/t.jsonl",
        "cwd": "/tmp",
        "hook_event_name": "notification",
        "message": "Test stdin handling"
    })
    .to_string();

    Command::cargo_bin("maos")
        .unwrap()
        .arg("notify")
        .write_stdin(input.as_bytes())
        .assert()
        .success();
}

/// Test concurrent execution across platforms
#[test]
fn test_platform_concurrency() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Spawn threads on different cores if available
    for i in 0..4 {
        let success_count = Arc::clone(&success_count);
        let handle = thread::spawn(move || {
            let input = json!({
                "session_id": format!("concurrent-{}", i),
                "transcript_path": "/tmp/t.jsonl",
                "cwd": "/tmp",
                "hook_event_name": "notification",
                "message": format!("Thread {}", i)
            })
            .to_string();

            let output = Command::cargo_bin("maos")
                .unwrap()
                .arg("notify")
                .write_stdin(input.as_bytes())
                .output()
                .expect("Failed to execute");

            if output.status.success() {
                success_count.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(success_count.load(Ordering::SeqCst), 4);
}
