//! Tests for message types (Issue #43)
//!
//! This test suite validates JSON message formats for Claude Code hook integration
//! and session state persistence, ensuring compatibility with both legacy (Python)
//! and documented (Claude Code) formats.

use maos_core::SessionId;
use maos_core::messages::*;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::str::FromStr;

// =============================================================================
// HookInput Tests - Testing Claude Code format
// =============================================================================

#[test]
fn test_hook_input_pre_tool_use() {
    // Test PreToolUse event format from Claude Code
    let pre_tool_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.txt",
        "cwd": "/home/user/project",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {
            "command": "cargo test"
        }
    });

    let input: HookInput = serde_json::from_value(pre_tool_json.clone()).unwrap();

    // Should extract correct fields
    assert_eq!(
        input.session_id,
        "sess_12345678-1234-1234-1234-123456789012"
    );
    assert_eq!(input.hook_event_name, HookEventName::PreToolUse);
    assert_eq!(input.tool_name(), "Bash");
    assert!(input.is_tool_event());

    // Should extract tool input
    let tool_input = input.tool_input();
    assert_eq!(tool_input.get("command").unwrap(), "cargo test");

    // Should round-trip serialize correctly
    let serialized = serde_json::to_value(&input).unwrap();
    assert_eq!(serialized, pre_tool_json);
}

#[test]
fn test_hook_input_post_tool_use() {
    // Test PostToolUse event with response
    let post_tool_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.txt",
        "cwd": "/home/user/project",
        "hook_event_name": "PostToolUse",
        "tool_name": "Write",
        "tool_input": {
            "file_path": "/test/new_file.txt",
            "content": "Hello, world!"
        },
        "tool_response": {
            "success": true,
            "filePath": "/test/new_file.txt"
        }
    });

    let input: HookInput = serde_json::from_value(post_tool_json).unwrap();

    // Should extract tool response
    let response = input.tool_response().unwrap();
    assert_eq!(response.get("success").unwrap(), true);
    assert_eq!(response.get("filePath").unwrap(), "/test/new_file.txt");
}

#[test]
fn test_hook_input_user_prompt_submit() {
    // Test UserPromptSubmit event
    let prompt_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.txt",
        "cwd": "/home/user/project",
        "hook_event_name": "UserPromptSubmit",
        "prompt": "Help me refactor this code"
    });

    let input: HookInput = serde_json::from_value(prompt_json).unwrap();

    assert_eq!(input.hook_event_name, HookEventName::UserPromptSubmit);
    assert!(!input.is_tool_event());
    assert_eq!(input.user_prompt(), Some("Help me refactor this code"));
    assert!(input.tool_name.is_none());
}

#[test]
fn test_hook_input_subagent_events() {
    // Test SubagentStart event
    let start_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.txt",
        "cwd": "/home/user/project",
        "hook_event_name": "SubagentStart"
    });

    let input: HookInput = serde_json::from_value(start_json).unwrap();
    assert_eq!(input.hook_event_name, HookEventName::SubagentStart);
    assert!(!input.is_tool_event());

    // Test SubagentStop event
    let stop_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.txt",
        "cwd": "/home/user/project",
        "hook_event_name": "SubagentStop"
    });

    let input: HookInput = serde_json::from_value(stop_json).unwrap();
    assert_eq!(input.hook_event_name, HookEventName::SubagentStop);
    assert!(!input.is_tool_event());
}

#[test]
fn test_hook_input_new_event_types() {
    // Test PreCompact event
    let precompact_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.txt",
        "cwd": "/home/user/project",
        "hook_event_name": "PreCompact"
    });

    let input: HookInput = serde_json::from_value(precompact_json).unwrap();
    assert_eq!(input.hook_event_name, HookEventName::PreCompact);
    assert!(!input.is_tool_event());

    // Test SessionStart event
    let session_start_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.txt",
        "cwd": "/home/user/project",
        "hook_event_name": "SessionStart"
    });

    let input: HookInput = serde_json::from_value(session_start_json).unwrap();
    assert_eq!(input.hook_event_name, HookEventName::SessionStart);
    assert!(!input.is_tool_event());

    // Test Stop event
    let stop_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.txt",
        "cwd": "/home/user/project",
        "hook_event_name": "Stop"
    });

    let input: HookInput = serde_json::from_value(stop_json).unwrap();
    assert_eq!(input.hook_event_name, HookEventName::Stop);
    assert!(!input.is_tool_event());

    // Test Notification event
    let notification_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.txt",
        "cwd": "/home/user/project",
        "hook_event_name": "Notification"
    });

    let input: HookInput = serde_json::from_value(notification_json).unwrap();
    assert_eq!(input.hook_event_name, HookEventName::Notification);
    assert!(!input.is_tool_event());
}

// =============================================================================
// PreToolMessage and PostToolMessage Tests
// =============================================================================

#[test]
fn test_pre_tool_message_creation() {
    let hook_input = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript",
        "cwd": "/workspace",
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "tool_input": {
            "file_path": "/test.rs"
        }
    });

    let input: HookInput = serde_json::from_value(hook_input).unwrap();
    let message = PreToolMessage::from_hook_input(input).unwrap();

    // Should extract session context
    assert!(message.session_context.session_id.as_str().contains("sess"));

    // Should have tool call information
    assert_eq!(message.tool_call.tool_name, "Edit");
}

#[test]
fn test_pre_tool_message_error_cases() {
    // Test with non-tool event (should fail)
    let hook_input = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript",
        "cwd": "/workspace",
        "hook_event_name": "UserPromptSubmit",
        "prompt": "Hello"
    });

    let input: HookInput = serde_json::from_value(hook_input).unwrap();
    let result = PreToolMessage::from_hook_input(input);
    assert!(result.is_err());

    // Test with SessionStart (non-tool event)
    let hook_input = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript",
        "cwd": "/workspace",
        "hook_event_name": "SessionStart"
    });

    let input: HookInput = serde_json::from_value(hook_input).unwrap();
    let result = PreToolMessage::from_hook_input(input);
    assert!(result.is_err());
}

#[test]
fn test_post_tool_message_creation() {
    let hook_input = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript",
        "cwd": "/workspace",
        "hook_event_name": "PostToolUse",
        "tool_name": "Bash",
        "tool_input": {
            "command": "ls -la"
        },
        "tool_response": {
            "success": true,
            "output": "file1.txt\nfile2.txt"
        }
    });

    let input: HookInput = serde_json::from_value(hook_input).unwrap();
    let message = PostToolMessage::from_hook_input(input).unwrap();

    // Should have tool result
    assert!(message.tool_result.success);

    // Should have original tool call
    assert_eq!(message.tool_call.tool_name, "Bash");
}

// =============================================================================
// HookResponse Tests
// =============================================================================

#[test]
fn test_hook_response_allow() {
    let response = HookResponse::Allow;

    // Should serialize correctly
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json, json!({ "action": "Allow" }));

    // Should have correct exit code
    assert_eq!(response.to_exit_code(), 0);
}

#[test]
fn test_hook_response_block() {
    let response = HookResponse::Block {
        reason: "Security violation: rm -rf detected".to_string(),
    };

    // Should serialize correctly
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(
        json,
        json!({
            "action": "Block",
            "data": {
                "reason": "Security violation: rm -rf detected"
            }
        })
    );

    // Should have correct exit code
    assert_eq!(response.to_exit_code(), 2);
}

#[test]
fn test_hook_response_modify() {
    let response = HookResponse::Modify {
        parameters: json!({
            "file_path": "/safe/path/file.txt"
        }),
    };

    // Should serialize correctly
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["action"], "Modify");
    assert_eq!(
        json["data"]["parameters"]["file_path"],
        "/safe/path/file.txt"
    );

    // Modify not yet supported, should default to allow
    assert_eq!(response.to_exit_code(), 0);
}

#[test]
fn test_hook_response_redirect() {
    let response = HookResponse::Redirect {
        tool_name: "SafeBash".to_string(),
        parameters: json!({
            "command": "ls",
            "safe_mode": true
        }),
    };

    // Should serialize correctly
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["action"], "Redirect");
    assert_eq!(json["data"]["tool_name"], "SafeBash");

    // Redirect not yet supported, should default to allow
    assert_eq!(response.to_exit_code(), 0);
}

// =============================================================================
// SessionContext Tests
// =============================================================================

#[test]
fn test_session_context_from_hook_input() {
    let hook_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript",
        "cwd": "/project",
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "tool_input": {
            "file_path": "/test.rs"
        }
    });

    let input: HookInput = serde_json::from_value(hook_json).unwrap();
    let context = SessionContext::from_hook_input(&input).unwrap();

    assert_eq!(
        context.session_id.as_str(),
        "sess_12345678-1234-1234-1234-123456789012"
    );
    assert_eq!(context.cwd, PathBuf::from("/project"));
    assert_eq!(
        context.transcript_path,
        Some(PathBuf::from("/tmp/transcript"))
    );
}

#[test]
fn test_post_tool_message_error_cases() {
    // Test with PreToolUse event (should fail - needs PostToolUse)
    let hook_input = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript",
        "cwd": "/workspace",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {
            "command": "ls"
        }
    });

    let input: HookInput = serde_json::from_value(hook_input).unwrap();
    let result = PostToolMessage::from_hook_input(input);
    assert!(result.is_err());

    // Test with UserPromptSubmit (non-tool event)
    let hook_input = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript",
        "cwd": "/workspace",
        "hook_event_name": "UserPromptSubmit",
        "prompt": "test"
    });

    let input: HookInput = serde_json::from_value(hook_input).unwrap();
    let result = PostToolMessage::from_hook_input(input);
    assert!(result.is_err());
}

#[test]
fn test_session_context_error_handling() {
    // Test with invalid session ID format
    let hook_input = HookInput {
        session_id: "invalid-session-id".to_string(),
        transcript_path: PathBuf::from("/tmp/transcript"),
        cwd: PathBuf::from("/workspace"),
        hook_event_name: HookEventName::PreToolUse,
        tool_name: Some("Test".to_string()),
        tool_input: None,
        tool_response: None,
        prompt: None,
    };

    let result = SessionContext::from_hook_input(&hook_input);
    assert!(result.is_err());
}

#[test]
fn test_session_context_with_agent_id() {
    let hook_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript",
        "cwd": "/workspace",
        "hook_event_name": "PreToolUse",
        "tool_name": "Task",
        "tool_input": {
            "agent_id": "agent_12345678-1234-1234-1234-123456789012",
            "subagent_type": "backend-engineer"
        }
    });

    let input: HookInput = serde_json::from_value(hook_json).unwrap();
    let context = SessionContext::from_hook_input(&input).unwrap();

    assert_eq!(
        context.agent_id.as_ref().unwrap().as_str(),
        "agent_12345678-1234-1234-1234-123456789012"
    );
}

// =============================================================================
// NotificationMessage Tests
// =============================================================================

#[test]
fn test_notification_message_serialization() {
    let notification = NotificationMessage {
        message: "Build completed successfully".to_string(),
        notification_type: NotificationType::TaskCompletion,
        engineer_name: Some("Marvin".to_string()),
        session_id: Some(SessionId::from_str("sess_12345678-1234-1234-1234-123456789012").unwrap()),
        urgency: NotificationUrgency::Normal,
        timestamp: chrono::Utc::now(),
    };

    // Should serialize to JSON
    let json = serde_json::to_value(&notification).unwrap();
    assert_eq!(json["message"], "Build completed successfully");
    assert_eq!(json["notification_type"], "task_completion");
    assert_eq!(json["urgency"], "normal");
    assert_eq!(json["engineer_name"], "Marvin");

    // Should deserialize from JSON
    let deserialized: NotificationMessage = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.message, notification.message);
}

#[test]
fn test_notification_types() {
    assert_eq!(
        serde_json::to_value(NotificationType::UserInputRequest).unwrap(),
        json!("user_input_request")
    );
    assert_eq!(
        serde_json::to_value(NotificationType::TaskCompletion).unwrap(),
        json!("task_completion")
    );
    assert_eq!(
        serde_json::to_value(NotificationType::AgentSpawned).unwrap(),
        json!("agent_spawned")
    );
    assert_eq!(
        serde_json::to_value(NotificationType::AgentCompleted).unwrap(),
        json!("agent_completed")
    );
    assert_eq!(
        serde_json::to_value(NotificationType::SecurityAlert).unwrap(),
        json!("security_alert")
    );
    assert_eq!(
        serde_json::to_value(NotificationType::SystemError).unwrap(),
        json!("system_error")
    );
}

#[test]
fn test_notification_urgency_levels() {
    assert_eq!(
        serde_json::to_value(NotificationUrgency::Low).unwrap(),
        json!("low")
    );
    assert_eq!(
        serde_json::to_value(NotificationUrgency::Normal).unwrap(),
        json!("normal")
    );
    assert_eq!(
        serde_json::to_value(NotificationUrgency::High).unwrap(),
        json!("high")
    );
    assert_eq!(
        serde_json::to_value(NotificationUrgency::Critical).unwrap(),
        json!("critical")
    );
}

#[test]
fn test_tts_formatting_with_engineer() {
    let notification = NotificationMessage {
        message: "tests passed".to_string(),
        notification_type: NotificationType::TaskCompletion,
        engineer_name: Some("Marvin".to_string()),
        session_id: None,
        urgency: NotificationUrgency::Normal,
        timestamp: chrono::Utc::now(),
    };

    let tts = notification.to_tts_string();
    assert_eq!(tts, "Marvin, task completed: tests passed");
}

#[test]
fn test_tts_formatting_without_engineer() {
    let notification = NotificationMessage {
        message: "need your input on PR review".to_string(),
        notification_type: NotificationType::UserInputRequest,
        engineer_name: None,
        session_id: None,
        urgency: NotificationUrgency::High,
        timestamp: chrono::Utc::now(),
    };

    let tts = notification.to_tts_string();
    assert_eq!(
        tts,
        "Engineer, I need your input: need your input on PR review"
    );
}

#[test]
fn test_tts_formatting_all_types() {
    // Test each notification type
    let types_and_expected = vec![
        (
            NotificationType::UserInputRequest,
            "test",
            "Marv, I need your input: test",
        ),
        (
            NotificationType::TaskCompletion,
            "done",
            "Marv, task completed: done",
        ),
        (
            NotificationType::AgentSpawned,
            "backend",
            "New agent spawned: backend",
        ),
        (
            NotificationType::AgentCompleted,
            "frontend",
            "Agent finished: frontend",
        ),
        (
            NotificationType::SecurityAlert,
            "danger",
            "Security alert! danger",
        ),
        (
            NotificationType::SystemError,
            "crash",
            "System error: crash",
        ),
    ];

    for (notification_type, message, expected) in types_and_expected {
        let notification = NotificationMessage {
            message: message.to_string(),
            notification_type,
            engineer_name: Some("Marv".to_string()),
            session_id: None,
            urgency: NotificationUrgency::Normal,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(notification.to_tts_string(), expected);
    }
}

// =============================================================================
// Session State File Tests - Proper session management schemas
// =============================================================================

#[test]
fn test_session_file_schema() {
    // Test the main session state file format
    let session_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:01:00Z",
        "status": "active",
        "workspace_root": "/workspace",
        "transcript_path": "/tmp/transcript.txt",
        "metadata": {
            "user": "test_user",
            "project": "maos",
            "environment": "development"
        }
    });

    let session: SessionFile = serde_json::from_value(session_json.clone()).unwrap();
    assert_eq!(
        session.session_id.as_str(),
        "sess_12345678-1234-1234-1234-123456789012"
    );
    assert_eq!(session.status, SessionStatus::Active);

    // Should round-trip serialize
    let serialized = serde_json::to_value(&session).unwrap();
    assert_eq!(serialized["status"], "active");
}

#[test]
fn test_agents_file_schema() {
    // Test agent coordination file format
    let agents_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "agents": [
            {
                "agent_id": "agent_12345678-1234-1234-1234-123456789012",
                "agent_type": "backend-engineer",
                "status": "active",
                "workspace": "/worktrees/backend-sess_123",
                "started_at": "2024-01-01T00:00:00Z",
                "parent_agent": null,
                "current_task": "Implementing API endpoint"
            },
            {
                "agent_id": "agent_87654321-4321-4321-4321-210987654321",
                "agent_type": "qa-engineer",
                "status": "pending",
                "workspace": null,
                "started_at": "2024-01-01T00:00:30Z",
                "parent_agent": "agent_12345678-1234-1234-1234-123456789012",
                "current_task": null
            }
        ]
    });

    let agents_file: AgentsFile = serde_json::from_value(agents_json).unwrap();
    assert_eq!(agents_file.agents.len(), 2);
    assert_eq!(agents_file.agents[0].agent_type, "backend-engineer");
    assert_eq!(agents_file.agents[0].status, AgentStatus::Active);
    assert_eq!(agents_file.agents[1].status, AgentStatus::Pending);

    // Verify parent agent relationship
    assert!(agents_file.agents[0].parent_agent.is_none());
    assert_eq!(
        agents_file.agents[1]
            .parent_agent
            .as_ref()
            .unwrap()
            .as_str(),
        "agent_12345678-1234-1234-1234-123456789012"
    );
}

#[test]
fn test_locks_file_schema() {
    // Test file locking coordination
    let locks_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "locks": [
            {
                "file_path": "/workspace/src/main.rs",
                "agent_id": "agent_12345678-1234-1234-1234-123456789012",
                "lock_type": "exclusive",
                "operation": "Edit",
                "acquired_at": "2024-01-01T00:00:00Z"
            },
            {
                "file_path": "/workspace/Cargo.toml",
                "agent_id": "agent_87654321-4321-4321-4321-210987654321",
                "lock_type": "shared",
                "operation": "Read",
                "acquired_at": "2024-01-01T00:00:30Z"
            }
        ]
    });

    let locks_file: LocksFile = serde_json::from_value(locks_json).unwrap();
    assert_eq!(locks_file.locks.len(), 2);
    assert_eq!(locks_file.locks[0].lock_type, LockType::Exclusive);
    assert_eq!(locks_file.locks[1].lock_type, LockType::Shared);
}

#[test]
fn test_session_directory_structure() {
    use std::str::FromStr;

    let session_id = SessionId::from_str("sess_12345678-1234-1234-1234-123456789012").unwrap();
    let session_dir = SessionDirectory::new(&session_id).unwrap();

    // Should create proper directory structure
    assert!(session_dir.session_file_path().ends_with("session.json"));
    assert!(session_dir.agents_file_path().ends_with("agents.json"));
    assert!(session_dir.locks_file_path().ends_with("locks.json"));
    assert!(session_dir.progress_file_path().ends_with("progress.json"));
}

// =============================================================================
// Schema Validation Tests
// =============================================================================

#[test]
fn test_schema_validator_hook_input() {
    let validator = SchemaValidator::new();

    // Valid PreToolUse event
    let valid_pre_tool = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript",
        "cwd": "/workspace",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {
            "command": "cargo test"
        }
    });
    assert!(validator.validate_hook_input(&valid_pre_tool).is_ok());

    // Valid UserPromptSubmit event
    let valid_prompt = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript",
        "cwd": "/workspace",
        "hook_event_name": "UserPromptSubmit",
        "prompt": "Help me with this code"
    });
    assert!(validator.validate_hook_input(&valid_prompt).is_ok());

    // Invalid - missing required fields
    let invalid = json!({
        "some_field": "value"
    });
    assert!(validator.validate_hook_input(&invalid).is_err());
}

#[test]
fn test_schema_validator_hook_response() {
    let validator = SchemaValidator::new();

    // Valid Allow response
    let valid_allow = json!({
        "action": "Allow"
    });
    assert!(validator.validate_hook_response(&valid_allow).is_ok());

    // Valid Block response
    let valid_block = json!({
        "action": "Block",
        "data": {
            "reason": "Security violation"
        }
    });
    assert!(validator.validate_hook_response(&valid_block).is_ok());

    // Invalid - missing action
    let invalid = json!({
        "data": {
            "reason": "test"
        }
    });
    assert!(validator.validate_hook_response(&invalid).is_err());
}

#[test]
fn test_schema_validator_session_file() {
    let validator = SchemaValidator::new();

    // Valid session file
    let valid = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:01:00Z",
        "status": "active",
        "workspace_root": "/workspace"
    });
    assert!(validator.validate_session_file(&valid).is_ok());

    // Invalid - bad status value
    let invalid_status = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:01:00Z",
        "status": "invalid_status",
        "workspace_root": "/workspace"
    });
    assert!(validator.validate_session_file(&invalid_status).is_err());
}

#[test]
fn test_schema_error_display() {
    let error = SchemaError::ValidationFailed {
        schema: "HookInput".to_string(),
        errors: vec!["Missing field: tool_name".to_string()],
    };

    let display = format!("{}", error);
    assert!(display.contains("HookInput"));
    assert!(display.contains("Missing field: tool_name"));
}

// =============================================================================
// HookOutput Tests
// =============================================================================

#[test]
fn test_hook_output_from_execution() {
    // Test successful execution with stdout
    let output = HookOutput::from_execution(Some("✅ Tool validated".to_string()), None, 0);

    assert_eq!(output.exit_code, 0);
    assert!(matches!(output.response, HookResponse::Allow));
    assert_eq!(output.stdout, Some("✅ Tool validated".to_string()));
    assert!(output.has_output());

    // Test blocked execution with stderr
    let output =
        HookOutput::from_execution(None, Some("❌ Dangerous command detected".to_string()), 2);

    assert_eq!(output.exit_code, 2);
    assert!(matches!(output.response, HookResponse::Block { .. }));
    assert_eq!(
        output.stderr,
        Some("❌ Dangerous command detected".to_string())
    );
    assert!(output.has_output());
}

#[test]
fn test_hook_output_display() {
    // Test with both stdout and stderr
    let output = HookOutput::from_execution(
        Some("Processing...".to_string()),
        Some("Warning: check path".to_string()),
        0,
    );

    let display = output.display_output().unwrap();
    assert!(display.contains("Processing..."));
    assert!(display.contains("Warning: check path"));

    // Test with no output
    let output = HookOutput::from_execution(None, None, 0);
    assert!(!output.has_output());
    assert!(output.display_output().is_none());
}

#[test]
fn test_hook_output_serialization() {
    let output = HookOutput {
        stdout: Some("Tool executed successfully".to_string()),
        stderr: None,
        exit_code: 0,
        response: HookResponse::Allow,
    };

    let json = serde_json::to_value(&output).unwrap();
    assert_eq!(json["stdout"], "Tool executed successfully");
    assert_eq!(json["exit_code"], 0);
    assert!(json["stderr"].is_null());

    // Should round-trip deserialize
    let deserialized: HookOutput = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.stdout, output.stdout);
    assert_eq!(deserialized.exit_code, output.exit_code);
}

// =============================================================================
// PathConstraint Tests
// =============================================================================

#[test]
fn test_path_constraint_validation() {
    let constraint = PathConstraint {
        allowed_paths: vec![PathBuf::from("/workspace"), PathBuf::from("/tmp")],
        blocked_patterns: vec![".env".to_string(), "*.secret".to_string()],
        max_depth: Some(5),
    };

    // Should allow paths within workspace
    assert!(constraint.is_allowed(Path::new("/workspace/src/main.rs")));
    assert!(constraint.is_allowed(Path::new("/tmp/test.txt")));

    // Should block paths outside allowed
    assert!(!constraint.is_allowed(Path::new("/etc/passwd")));

    // Should block patterns
    assert!(!constraint.is_allowed(Path::new("/workspace/.env")));
    assert!(!constraint.is_allowed(Path::new("/tmp/key.secret")));

    // Should respect max depth
    assert!(!constraint.is_allowed(Path::new("/workspace/a/b/c/d/e/f/too/deep.txt")));
}

#[test]
fn test_path_constraint_complex_patterns() {
    let constraint = PathConstraint {
        allowed_paths: vec![PathBuf::from("/workspace")],
        blocked_patterns: vec![
            "*.log".to_string(),
            "**/node_modules/**".to_string(),
            "test_*_backup".to_string(),
        ],
        max_depth: None,
    };

    // Test complex glob patterns with multiple asterisks
    assert!(!constraint.is_allowed(Path::new("/workspace/debug.log")));
    assert!(!constraint.is_allowed(Path::new("/workspace/test_file_backup")));

    // Test patterns that don't match
    assert!(constraint.is_allowed(Path::new("/workspace/test_file")));
    assert!(constraint.is_allowed(Path::new("/workspace/backup_test")));

    // Test no max_depth restriction
    assert!(constraint.is_allowed(Path::new("/workspace/a/b/c/d/e/f/g/h/i/j/deep.txt")));
}

#[test]
fn test_path_constraint_edge_cases() {
    // Empty allowed paths
    let constraint = PathConstraint {
        allowed_paths: vec![],
        blocked_patterns: vec![],
        max_depth: None,
    };
    assert!(!constraint.is_allowed(Path::new("/any/path")));

    // Path exactly matching allowed path
    let constraint = PathConstraint {
        allowed_paths: vec![PathBuf::from("/workspace")],
        blocked_patterns: vec![],
        max_depth: Some(0),
    };
    assert!(constraint.is_allowed(Path::new("/workspace")));
    assert!(!constraint.is_allowed(Path::new("/workspace/file.txt")));
}
