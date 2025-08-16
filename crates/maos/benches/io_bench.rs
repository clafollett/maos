//! Performance benchmarks for IO module (Issue #55)
//!
//! Run with: cargo bench -p maos

use criterion::{Criterion, criterion_group, criterion_main};
use maos::io::HookInput;
use serde_json::json;
use std::hint::black_box;

fn bench_hook_input_parsing(c: &mut Criterion) {
    // Small message (1KB)
    let small_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.jsonl",
        "cwd": "/workspace",
        "hook_event_name": "notification",
        "message": "Task completed"
    });

    // Medium message (10KB)
    let medium_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.jsonl",
        "cwd": "/workspace",
        "hook_event_name": "pre_tool_use",
        "tool_name": "Write",
        "tool_input": {
            "content": "x".repeat(10000)
        }
    });

    // Large message (100KB)
    let large_json = json!({
        "session_id": "sess_12345678-1234-1234-1234-123456789012",
        "transcript_path": "/tmp/transcript.jsonl",
        "cwd": "/workspace",
        "hook_event_name": "post_tool_use",
        "tool_name": "Read",
        "tool_input": {"file": "large.txt"},
        "tool_response": {
            "content": "x".repeat(100000)
        }
    });

    let small_str = small_json.to_string();
    let medium_str = medium_json.to_string();
    let large_str = large_json.to_string();

    c.bench_function("parse_hook_input_small_1kb", |b| {
        b.iter(|| {
            let _input: HookInput = serde_json::from_str(black_box(&small_str)).unwrap();
        })
    });

    c.bench_function("parse_hook_input_medium_10kb", |b| {
        b.iter(|| {
            let _input: HookInput = serde_json::from_str(black_box(&medium_str)).unwrap();
        })
    });

    c.bench_function("parse_hook_input_large_100kb", |b| {
        b.iter(|| {
            let _input: HookInput = serde_json::from_str(black_box(&large_str)).unwrap();
        })
    });
}

fn bench_hook_validation(c: &mut Criterion) {
    let valid_pre_tool = HookInput {
        session_id: "sess_123".to_string(),
        transcript_path: std::path::PathBuf::from("/tmp/transcript.jsonl"),
        cwd: std::path::PathBuf::from("/workspace"),
        hook_event_name: "pre_tool_use".to_string(),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(json!({"command": "ls"})),
        tool_response: None,
        message: None,
        prompt: None,
        stop_hook_active: None,
        trigger: None,
        custom_instructions: None,
        source: None,
    };

    let valid_session = HookInput {
        session_id: "sess_123".to_string(),
        transcript_path: std::path::PathBuf::from("/tmp/transcript.jsonl"),
        cwd: std::path::PathBuf::from("/workspace"),
        hook_event_name: "session_start".to_string(),
        tool_name: None,
        tool_input: None,
        tool_response: None,
        message: None,
        prompt: None,
        stop_hook_active: None,
        trigger: None,
        custom_instructions: None,
        source: Some("startup".to_string()),
    };

    c.bench_function("validate_pre_tool_use", |b| {
        b.iter(|| {
            black_box(&valid_pre_tool).validate().unwrap();
        })
    });

    c.bench_function("validate_session_start", |b| {
        b.iter(|| {
            black_box(&valid_session).validate().unwrap();
        })
    });
}

fn bench_serialization(c: &mut Criterion) {
    let input = HookInput {
        session_id: "sess_12345678-1234-1234-1234-123456789012".to_string(),
        transcript_path: std::path::PathBuf::from("/tmp/transcript.jsonl"),
        cwd: std::path::PathBuf::from("/workspace"),
        hook_event_name: "pre_tool_use".to_string(),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(json!({"command": "cargo test"})),
        tool_response: None,
        message: None,
        prompt: None,
        stop_hook_active: None,
        trigger: None,
        custom_instructions: None,
        source: None,
    };

    c.bench_function("serialize_hook_input", |b| {
        b.iter(|| {
            let _ = serde_json::to_string(black_box(&input)).unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_hook_input_parsing,
    bench_hook_validation,
    bench_serialization
);
criterion_main!(benches);
