//! Performance benchmarks for message types (Issue #43)
//!
//! Run with: cargo bench -p maos-core

use criterion::{Criterion, criterion_group, criterion_main};
use maos_core::messages::{HookInput, HookResponse, NotificationMessage, SchemaValidator};
use serde_json::json;
use std::hint::black_box;

fn bench_hook_input_parsing(c: &mut Criterion) {
    // Test PreToolUse event
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

    // Test UserPromptSubmit event
    let user_prompt_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.txt",
        "cwd": "/home/user/project",
        "hook_event_name": "UserPromptSubmit",
        "prompt": "Help me refactor this code"
    });

    c.bench_function("parse_hook_input_pre_tool", |b| {
        b.iter(|| {
            let _input: HookInput =
                serde_json::from_value(black_box(pre_tool_json.clone())).unwrap();
        })
    });

    c.bench_function("parse_hook_input_post_tool", |b| {
        b.iter(|| {
            let _input: HookInput =
                serde_json::from_value(black_box(post_tool_json.clone())).unwrap();
        })
    });

    c.bench_function("parse_hook_input_user_prompt", |b| {
        b.iter(|| {
            let _input: HookInput =
                serde_json::from_value(black_box(user_prompt_json.clone())).unwrap();
        })
    });
}

fn bench_hook_response_parsing(c: &mut Criterion) {
    let allow_json = json!({ "action": "Allow" });
    let block_json = json!({
        "action": "Block",
        "data": {
            "reason": "Security violation detected"
        }
    });

    c.bench_function("parse_hook_response_allow", |b| {
        b.iter(|| {
            let _response: HookResponse =
                serde_json::from_value(black_box(allow_json.clone())).unwrap();
        })
    });

    c.bench_function("parse_hook_response_block", |b| {
        b.iter(|| {
            let _response: HookResponse =
                serde_json::from_value(black_box(block_json.clone())).unwrap();
        })
    });
}

fn bench_schema_validation(c: &mut Criterion) {
    let validator = SchemaValidator::new();

    let hook_input = json!({
        "tool_name": "Edit",
        "tool_input": {
            "file_path": "/test.rs",
            "old_string": "foo",
            "new_string": "bar"
        }
    });

    let session_file = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:01:00Z",
        "status": "active",
        "workspace_root": "/workspace"
    });

    c.bench_function("validate_hook_input", |b| {
        b.iter(|| {
            validator
                .validate_hook_input(black_box(&hook_input))
                .unwrap();
        })
    });

    c.bench_function("validate_session_file", |b| {
        b.iter(|| {
            validator
                .validate_session_file(black_box(&session_file))
                .unwrap();
        })
    });
}

fn bench_notification_tts(c: &mut Criterion) {
    use chrono::Utc;
    use maos_core::messages::{NotificationType, NotificationUrgency};

    let notification = NotificationMessage {
        message: "Build completed successfully with 42 tests passing".to_string(),
        notification_type: NotificationType::TaskCompletion,
        engineer_name: Some("Marvin".to_string()),
        session_id: None,
        urgency: NotificationUrgency::Normal,
        timestamp: Utc::now(),
    };

    c.bench_function("notification_to_tts", |b| {
        b.iter(|| {
            let _tts = black_box(&notification).to_tts_string();
        })
    });
}

fn bench_hook_output(c: &mut Criterion) {
    use maos_core::messages::HookOutput;

    c.bench_function("hook_output_from_execution", |b| {
        b.iter(|| {
            let _output = HookOutput::from_execution(
                Some("Tool executed successfully".to_string()),
                Some("Warning: deprecated API".to_string()),
                black_box(0),
            );
        })
    });

    let output = HookOutput::from_execution(
        Some("Processing...".to_string()),
        Some("Warning: check path".to_string()),
        0,
    );

    c.bench_function("hook_output_display", |b| {
        b.iter(|| {
            let _display = black_box(&output).display_output();
        })
    });
}

fn bench_path_constraint(c: &mut Criterion) {
    use maos_core::messages::PathConstraint;
    use std::path::{Path, PathBuf};

    let constraint = PathConstraint::new(
        vec![PathBuf::from("/workspace"), PathBuf::from("/tmp")],
        vec![
            ".env".to_string(),
            "*.secret".to_string(),
            "test_*_backup".to_string(),
        ],
        Some(5),
    );

    let test_paths = vec![
        Path::new("/workspace/src/main.rs"),
        Path::new("/tmp/test.txt"),
        Path::new("/etc/passwd"),
        Path::new("/workspace/.env"),
        Path::new("/workspace/test_file_backup"),
    ];

    c.bench_function("path_constraint_validation", |b| {
        b.iter(|| {
            for path in &test_paths {
                let _ = black_box(&constraint).is_allowed(black_box(path));
            }
        })
    });
}

fn bench_session_context(c: &mut Criterion) {
    use maos_core::messages::SessionContext;

    let hook_input = HookInput {
        session_id: "sess_12345678-1234-1234-1234-123456789012".to_string(),
        transcript_path: "/tmp/transcript".into(),
        cwd: "/workspace".into(),
        hook_event_name: maos_core::messages::HookEventName::PreToolUse,
        tool_name: Some("Edit".to_string()),
        tool_input: Some(json!({
            "file_path": "/test.rs",
            "agent_id": "agent_12345678-1234-1234-1234-123456789012"
        })),
        tool_response: None,
        prompt: None,
    };

    c.bench_function("session_context_from_hook_input", |b| {
        b.iter(|| {
            let _context = SessionContext::from_hook_input(black_box(&hook_input)).unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_hook_input_parsing,
    bench_hook_response_parsing,
    bench_schema_validation,
    bench_notification_tts,
    bench_hook_output,
    bench_path_constraint,
    bench_session_context
);
criterion_main!(benches);
