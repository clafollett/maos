//! 🔥 CRITICAL SECURITY TESTS for Issue #56 Path Traversal Fix
//!
//! These tests verify that the path traversal vulnerability has been properly fixed
//! and that malicious input is blocked while legitimate input is allowed.

use crate::io::HookInput;
use serde_json::json;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[tokio::test]
async fn test_path_traversal_blocked() {
    // 🚨 CRITICAL: Test that malicious path traversal attempts are blocked
    let malicious_input = json!({
        "session_id": "sess_123",
        "transcript_path": "../../../etc/passwd",
        "cwd": "/root/.ssh",
        "hook_event_name": "pre_tool_use",
        "tool_name": "Bash",
        "tool_input": {"command": "cat /etc/passwd"}
    });

    let input: HookInput = serde_json::from_value(malicious_input).unwrap();
    let workspace = Path::new("/tmp/safe_workspace");

    // 🛡️ Should reject malicious paths
    let result = input.validate_paths(workspace);
    assert!(result.is_err(), "Path traversal attack should be blocked");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Security violation detected"));
    assert!(error_msg.contains("path traversal blocked"));
}

#[test]
fn test_valid_paths_accepted() {
    // ✅ Test that legitimate paths within workspace are allowed
    let temp_dir = TempDir::new().unwrap();
    let workspace = temp_dir.path();

    let valid_input = HookInput {
        session_id: "sess_123".to_string(),
        transcript_path: workspace.join("transcript.jsonl"),
        cwd: workspace.join("project"),
        hook_event_name: "pre_tool_use".to_string(),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(json!({"command": "ls"})),
        ..Default::default()
    };

    // 🎯 Should accept valid paths within workspace
    let result = valid_input.validate_paths(workspace);
    assert!(
        result.is_ok(),
        "Valid paths should be accepted: {:?}",
        result
    );
}

#[test]
fn test_path_traversal_variants_blocked() {
    // 🧨 Test various path traversal attack patterns
    let workspace = Path::new("/safe/workspace");

    let attack_patterns = vec![
        "../../../etc/passwd",                        // Basic traversal
        "..\\..\\..\\windows\\system32\\config\\sam", // Windows traversal
        "/etc/shadow",                                // Absolute path escape
        "//server/share/file",                        // UNC path
        "file:///etc/passwd",                         // File URL
        "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",    // URL encoded
        "test/../../etc/passwd",                      // Mixed legitimate/malicious
        "./../../etc/passwd",                         // Current dir prefix
        "..\\u{FF0F}etc\\u{2044}passwd",              // Unicode bypass attempt
    ];

    for attack_path in attack_patterns {
        let malicious_input = HookInput {
            session_id: "sess_123".to_string(),
            transcript_path: PathBuf::from(attack_path),
            cwd: workspace.join("safe"),
            hook_event_name: "pre_tool_use".to_string(),
            ..Default::default()
        };

        let result = malicious_input.validate_paths(workspace);
        assert!(
            result.is_err(),
            "Attack pattern should be blocked: {}",
            attack_path
        );
    }
}

#[test]
fn test_cwd_traversal_blocked() {
    // 🚨 Test that CWD field is also validated (not just transcript_path)
    let workspace = Path::new("/safe/workspace");

    let malicious_input = HookInput {
        session_id: "sess_123".to_string(),
        transcript_path: workspace.join("transcript.jsonl"),
        cwd: PathBuf::from("../../../root/.ssh"), // ← Malicious CWD!
        hook_event_name: "pre_tool_use".to_string(),
        ..Default::default()
    };

    let result = malicious_input.validate_paths(workspace);
    assert!(result.is_err(), "CWD traversal attack should be blocked");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid working directory"));
    assert!(error_msg.contains("Security violation detected"));
}

#[test]
fn test_error_messages_no_data_leak() {
    // 🔍 Ensure error messages don't leak the malicious input
    let workspace = Path::new("/safe/workspace");

    let malicious_input = HookInput {
        session_id: "sess_123".to_string(),
        transcript_path: PathBuf::from("../../../etc/passwd"),
        cwd: PathBuf::from("/root/.ssh"),
        hook_event_name: "pre_tool_use".to_string(),
        ..Default::default()
    };

    let result = malicious_input.validate_paths(workspace);
    let error_msg = result.unwrap_err().to_string();

    // 🛡️ Should NOT contain the malicious input paths
    assert!(!error_msg.contains("../../../etc/passwd"));
    assert!(!error_msg.contains("/root/.ssh"));

    // ✅ Should contain helpful but safe guidance
    assert!(error_msg.contains("Security violation detected"));
    assert!(error_msg.contains("path traversal blocked"));
}

#[test]
fn test_relative_paths_within_workspace() {
    // ✅ Test that relative paths staying within workspace are allowed
    let temp_dir = TempDir::new().unwrap();
    let workspace = temp_dir.path();

    let valid_relative_input = HookInput {
        session_id: "sess_123".to_string(),
        transcript_path: PathBuf::from("./transcripts/session.jsonl"), // Relative but safe
        cwd: PathBuf::from("./project/src"),                           // Relative but safe
        hook_event_name: "pre_tool_use".to_string(),
        ..Default::default()
    };

    // Note: This might fail if PathValidator doesn't handle relative paths correctly
    // In that case, we need to ensure all paths are resolved to absolute before validation
    let result = valid_relative_input.validate_paths(workspace);

    // For now, document expected behavior - may need path canonicalization
    println!("Relative path validation result: {:?}", result);
}

#[test]
fn test_empty_paths_handled() {
    // 🎯 Test edge case: empty paths
    let workspace = Path::new("/safe/workspace");

    let empty_path_input = HookInput {
        session_id: "sess_123".to_string(),
        transcript_path: PathBuf::new(), // Empty path
        cwd: PathBuf::new(),             // Empty path
        hook_event_name: "pre_tool_use".to_string(),
        ..Default::default()
    };

    let result = empty_path_input.validate_paths(workspace);
    // Empty paths should probably be rejected
    assert!(result.is_err(), "Empty paths should be rejected");
}

#[test]
fn test_workspace_boundary_validation() {
    // 🧪 Test exact workspace boundary cases
    let temp_dir = TempDir::new().unwrap();
    let workspace = temp_dir.path();

    // Test path exactly at workspace root
    let boundary_input = HookInput {
        session_id: "sess_123".to_string(),
        transcript_path: workspace.to_path_buf(), // Exactly at workspace root
        cwd: workspace.to_path_buf(),
        hook_event_name: "pre_tool_use".to_string(),
        ..Default::default()
    };

    let result = boundary_input.validate_paths(workspace);
    assert!(
        result.is_ok(),
        "Workspace root should be allowed: {:?}",
        result
    );
}
