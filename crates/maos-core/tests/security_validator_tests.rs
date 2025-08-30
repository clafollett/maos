//! Integration tests for unified SecurityValidator
//!
//! Tests the complete security validation system to ensure it matches
//! the Python hook behavior from .claude/hooks/maos/hooks/pre_tool_use.py

use maos_core::security::SecurityValidator;
use maos_core::test_utils::*;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

#[test]
fn test_validator_with_real_tool_calls() {
    let validator = SecurityValidator::new();

    // Simulate real tool calls from Claude Code

    // 1. Bash command that tries to delete system files
    let bash_call = create_tool_call(
        "Bash",
        json!({
            "command": "rm -rf /tmp/test && echo 'done'",
            "description": "Clean up test files"
        }),
    );
    assert!(
        validator.validate(&bash_call).is_ok(),
        "Safe tmp cleanup should be allowed"
    );

    // 2. Read operation on a normal file
    let read_call = create_tool_call(
        "Read",
        json!({
            "file_path": "/home/user/project/README.md",
            "offset": 0,
            "limit": 100
        }),
    );
    assert!(
        validator.validate(&read_call).is_ok(),
        "Normal file read should be allowed"
    );

    // 3. Write operation to create a new file
    let write_call = create_tool_call(
        "Write",
        json!({
            "file_path": "src/new_module.rs",
            "content": "pub fn hello() { println!(\"Hello!\"); }"
        }),
    );
    assert!(
        validator.validate(&write_call).is_ok(),
        "Creating source files should be allowed"
    );

    // 4. Edit operation on existing file
    let edit_call = create_tool_call(
        "Edit",
        json!({
            "file_path": "Cargo.toml",
            "old_string": "version = \"0.1.0\"",
            "new_string": "version = \"0.1.1\""
        }),
    );
    assert!(
        validator.validate(&edit_call).is_ok(),
        "Editing Cargo.toml should be allowed"
    );

    // 5. MultiEdit for refactoring
    let multi_edit_call = create_tool_call(
        "MultiEdit",
        json!({
            "file_path": "src/lib.rs",
            "edits": [
                {"old_string": "foo", "new_string": "bar"},
                {"old_string": "baz", "new_string": "qux"}
            ]
        }),
    );
    assert!(
        validator.validate(&multi_edit_call).is_ok(),
        "MultiEdit should be allowed"
    );
}

#[test]
fn test_validator_matches_python_hook_behavior() {
    let validator = SecurityValidator::new();

    // These test cases are directly from Python pre_tool_use.py

    // Test 1: Block .env file access (line 189-192 in Python)
    let env_access = create_tool_call("Read", json!({"file_path": ".env"}));
    let result = validator.validate(&env_access);
    assert!(result.is_err(), "Should block .env file access");

    // Test 2: Allow .env.sample (exception in Python)
    let sample_access = create_tool_call("Read", json!({"file_path": ".env.sample"}));
    assert!(
        validator.validate(&sample_access).is_ok(),
        "Should allow .env.sample"
    );

    // Test 3: Allow stack.env (MAOS-specific exception)
    let stack_access = create_tool_call("Read", json!({"file_path": "stack.env"}));
    assert!(
        validator.validate(&stack_access).is_ok(),
        "Should allow stack.env"
    );

    // Test 4: Block dangerous rm commands (line 199-201 in Python)
    let dangerous_rm = create_tool_call("Bash", json!({"command": "rm -rf /"}));
    let result = validator.validate(&dangerous_rm);
    assert!(result.is_err(), "Should block dangerous rm command");

    // Test 5: Block home directory removal
    let rm_home = create_tool_call("Bash", json!({"command": "rm -rf ~"}));
    assert!(
        validator.validate(&rm_home).is_err(),
        "Should block home directory removal"
    );

    // Test 6: Block $HOME removal
    let rm_home_var = create_tool_call("Bash", json!({"command": "rm -rf $HOME"}));
    assert!(
        validator.validate(&rm_home_var).is_err(),
        "Should block $HOME removal"
    );

    // Test 7: Allow safe rm commands
    let safe_rm = create_tool_call("Bash", json!({"command": "rm test.txt"}));
    assert!(
        validator.validate(&safe_rm).is_ok(),
        "Should allow safe rm commands"
    );

    // Test 8: Allow rm with -r but not -f
    let rm_recursive_only = create_tool_call("Bash", json!({"command": "rm -r old_dir"}));
    assert!(
        validator.validate(&rm_recursive_only).is_ok(),
        "Should allow rm -r without -f"
    );
}

#[test]
fn test_validator_thread_safety() {
    // SecurityValidator should be thread-safe (Send + Sync)
    let validator = Arc::new(SecurityValidator::new());
    let mut handles = vec![];

    for i in 0..10 {
        let validator_clone = Arc::clone(&validator);
        let handle = thread::spawn(move || {
            let tool_call = if i % 2 == 0 {
                // Even threads test dangerous commands
                create_tool_call("Bash", json!({"command": "rm -rf /"}))
            } else {
                // Odd threads test safe commands
                create_tool_call("Bash", json!({"command": "ls -la"}))
            };

            let result = validator_clone.validate(&tool_call);

            if i % 2 == 0 {
                assert!(result.is_err(), "Thread {i} should block dangerous command");
            } else {
                assert!(result.is_ok(), "Thread {i} should allow safe command");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn test_validator_workspace_boundary_enforcement() {
    let workspace_root = PathBuf::from("/home/user/project");
    let validator = SecurityValidator::new().with_workspace_root(workspace_root.clone());

    // Test access within workspace - should be allowed
    let within_workspace = create_tool_call(
        "Read",
        json!({
            "file_path": "/home/user/project/src/main.rs"
        }),
    );
    assert!(
        validator.validate(&within_workspace).is_ok(),
        "Access within workspace should be allowed"
    );

    // Test access outside workspace - should be blocked
    let outside_workspace = create_tool_call(
        "Read",
        json!({
            "file_path": "/etc/passwd"
        }),
    );
    assert!(
        validator.validate(&outside_workspace).is_err(),
        "Access outside workspace should be blocked"
    );

    // Test relative path that stays within workspace
    let relative_safe = create_tool_call(
        "Read",
        json!({
            "file_path": "./src/lib.rs"
        }),
    );
    assert!(
        validator.validate(&relative_safe).is_ok(),
        "Relative paths within workspace should be allowed"
    );

    // Test path traversal attempt
    let traversal = create_tool_call(
        "Read",
        json!({
            "file_path": "../../../etc/passwd"
        }),
    );
    assert!(
        validator.validate(&traversal).is_err(),
        "Path traversal should be blocked"
    );
}

#[test]
fn test_validator_performance_with_many_validations() {
    use std::time::Instant;

    let validator = SecurityValidator::new();
    let mut tool_calls = Vec::new();

    // Create a mix of tool calls
    for i in 0..100 {
        let tool_call = match i % 5 {
            0 => create_tool_call("Bash", json!({"command": "rm -rf /"})),
            1 => create_tool_call("Read", json!({"file_path": ".env"})),
            2 => create_tool_call("Write", json!({"file_path": "test.txt", "content": "data"})),
            3 => create_tool_call(
                "Edit",
                json!({"file_path": "src/main.rs", "old_string": "a", "new_string": "b"}),
            ),
            _ => create_tool_call("CustomTool", json!({"data": "test"})),
        };
        tool_calls.push(tool_call);
    }

    let start = Instant::now();
    for tool_call in &tool_calls {
        let _ = validator.validate(tool_call);
    }
    let duration = start.elapsed();

    let avg_ms = duration.as_millis() as f64 / tool_calls.len() as f64;
    println!("Average validation time: {avg_ms:.3}ms");

    assert!(
        avg_ms < 5.0,
        "Average validation time {avg_ms:.3}ms exceeds 5ms limit"
    );
}

#[test]
fn test_validator_error_messages_are_helpful() {
    let validator = SecurityValidator::new();

    // Test that error messages provide useful information
    let dangerous_cmd = create_tool_call("Bash", json!({"command": "rm -rf /"}));
    let result = validator.validate(&dangerous_cmd);

    if let Err(err) = result {
        let error_msg = err.to_string();
        // Error message should explain why it was blocked
        assert!(
            error_msg.contains("rm")
                || error_msg.contains("dangerous")
                || error_msg.contains("blocked")
                || error_msg.contains("recursive"),
            "Error message should be descriptive: {error_msg}"
        );
    } else {
        panic!("Expected error for dangerous command");
    }

    // Test env file error message
    let env_access = create_tool_call("Read", json!({"file_path": ".env"}));
    let result = validator.validate(&env_access);

    if let Err(err) = result {
        let error_msg = err.to_string();
        assert!(
            error_msg.contains("env")
                || error_msg.contains("sensitive")
                || error_msg.contains("protected")
                || error_msg.contains("secret"),
            "Error message should mention environment files: {error_msg}"
        );
    } else {
        panic!("Expected error for .env access");
    }
}
