//! Performance benchmarks for individual handler execution
//!
//! These benchmarks measure the performance characteristics of each handler implementation
//! to establish baselines for future optimization and identify performance bottlenecks.

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use maos::cli::handler::CommandHandler;
use maos::cli::handlers::*;
use maos::io::HookInput;
use maos_core::hook_constants::*;
use serde_json::json;
use std::hint::black_box;
use std::path::PathBuf;
use tokio::runtime::Runtime;

/// Type alias for handler collections in benchmarks
type HandlerVec = Vec<Box<dyn CommandHandler>>;

/// Create a realistic hook input for benchmarking
fn create_hook_input(hook_event_name: &str) -> HookInput {
    HookInput {
        session_id: "bench_session_12345".to_string(),
        transcript_path: PathBuf::from("/tmp/bench_transcript.jsonl"),
        cwd: PathBuf::from("/tmp/bench_workspace"),
        hook_event_name: hook_event_name.to_string(),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(json!({"command": "ls -la"})),
        tool_response: Some(json!({"stdout": "file1.txt\nfile2.txt", "exit_code": 0})),
        message: Some("Test benchmark message with realistic content".to_string()),
        prompt: Some("Analyze this code for potential security vulnerabilities".to_string()),
        stop_hook_active: Some(true),
        trigger: Some("auto".to_string()),
        custom_instructions: Some("Preserve important context during compaction".to_string()),
        source: Some("startup".to_string()),
    }
}

/// Benchmark individual handler execution performance
fn benchmark_handler_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("handler_execution");

    // Set baseline measurement time to get stable results
    group.measurement_time(std::time::Duration::from_secs(10));

    // Benchmark each handler with appropriate input
    let handlers: HandlerVec = vec![
        Box::new(PreToolUseHandler),
        Box::new(PostToolUseHandler),
        Box::new(NotificationHandler),
        Box::new(StopHandler),
        Box::new(SubagentStopHandler),
        Box::new(UserPromptSubmitHandler),
        Box::new(PreCompactHandler),
        Box::new(SessionStartHandler),
    ];

    for handler in &handlers {
        let event_name = handler.name();
        let input = create_hook_input(event_name);

        group.bench_with_input(
            BenchmarkId::new("execute", event_name),
            &input,
            |b, input| {
                b.to_async(&rt).iter(|| async {
                    let input = input.clone();
                    black_box(handler.execute(input).await.unwrap())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark handler validation performance
fn benchmark_handler_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("handler_validation");

    let handlers: HandlerVec = vec![
        Box::new(PreToolUseHandler),
        Box::new(PostToolUseHandler),
        Box::new(NotificationHandler),
        Box::new(StopHandler),
        Box::new(SubagentStopHandler),
        Box::new(UserPromptSubmitHandler),
        Box::new(PreCompactHandler),
        Box::new(SessionStartHandler),
    ];

    for handler in &handlers {
        let event_name = handler.name();
        let input = create_hook_input(event_name);

        group.bench_with_input(
            BenchmarkId::new("validate", event_name),
            &input,
            |b, input| {
                b.iter_batched(
                    || input.clone(),
                    |input| {
                        let _: () = handler.validate_input(&input).unwrap();
                        black_box(())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark handler name retrieval performance
fn benchmark_handler_name_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("handler_name_access");

    let handlers: HandlerVec = vec![
        Box::new(PreToolUseHandler),
        Box::new(PostToolUseHandler),
        Box::new(NotificationHandler),
        Box::new(StopHandler),
        Box::new(SubagentStopHandler),
        Box::new(UserPromptSubmitHandler),
        Box::new(PreCompactHandler),
        Box::new(SessionStartHandler),
    ];

    for handler in &handlers {
        let event_name = handler.name();
        group.bench_function(event_name, |b| b.iter(|| black_box(handler.name())));
    }

    group.finish();
}

/// Benchmark handler error handling performance
fn benchmark_handler_error_cases(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("handler_error_cases");

    // Test validation errors with wrong event names
    let handlers: HandlerVec = vec![
        Box::new(PreToolUseHandler),
        Box::new(PostToolUseHandler),
        Box::new(NotificationHandler),
        Box::new(StopHandler),
    ];

    for handler in &handlers {
        let input = create_hook_input("wrong_event_name");

        group.bench_with_input(
            BenchmarkId::new("validation_error", handler.name()),
            &input,
            |b, input| {
                b.iter(|| {
                    let input = input.clone();
                    let result = handler.validate_input(&input);
                    black_box(result.is_err())
                });
            },
        );
    }

    // Test execution errors for handlers that require specific fields
    let pre_tool_handler = PreToolUseHandler;
    let mut input_no_tool = create_hook_input(PRE_TOOL_USE);
    input_no_tool.tool_name = None;

    group.bench_function("execution_error_missing_tool", |b| {
        b.to_async(&rt).iter(|| async {
            let input = input_no_tool.clone();
            let result = pre_tool_handler.execute(input).await;
            black_box(result.is_err())
        });
    });

    group.finish();
}

/// Benchmark memory allocation patterns during handler execution
fn benchmark_handler_memory_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("handler_memory_patterns");

    // Focus on handlers that do more string processing
    let handlers: HandlerVec = vec![
        Box::new(NotificationHandler),
        Box::new(UserPromptSubmitHandler),
        Box::new(SessionStartHandler),
    ];

    for handler in &handlers {
        let event_name = handler.name();
        // Create larger input to stress test memory allocation
        let mut input = create_hook_input(event_name);
        input.message = Some("A".repeat(1000)); // 1KB message
        input.prompt = Some("B".repeat(2000)); // 2KB prompt

        group.bench_with_input(
            BenchmarkId::new("large_input", event_name),
            &input,
            |b, input| {
                b.to_async(&rt).iter(|| async {
                    let input = input.clone();
                    black_box(handler.execute(input).await.unwrap())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent handler execution
fn benchmark_concurrent_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_execution");

    let handler = NotificationHandler;
    let inputs: Vec<HookInput> = (0..10)
        .map(|i| {
            let mut input = create_hook_input(NOTIFICATION);
            input.message = Some(format!("Concurrent message {i}"));
            input
        })
        .collect();

    group.bench_function("10_concurrent_notifications", |b| {
        b.to_async(&rt).iter(|| async {
            let inputs = inputs.clone();
            let futures: Vec<_> = inputs
                .into_iter()
                .map(|input| handler.execute(input))
                .collect();

            let results = futures::future::join_all(futures).await;
            black_box(results.into_iter().all(|r| r.is_ok()))
        });
    });

    group.finish();
}

criterion_group!(
    handler_benches,
    benchmark_handler_execution,
    benchmark_handler_validation,
    benchmark_handler_name_access,
    benchmark_handler_error_cases,
    benchmark_handler_memory_patterns,
    benchmark_concurrent_execution
);

criterion_main!(handler_benches);
