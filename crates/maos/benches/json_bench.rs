//! JSON parsing performance benchmarks
//!
//! Measures parsing performance for typical hook message sizes.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use maos::io::HookInput;
use std::hint::black_box;
use std::time::Duration;

/// Generate a realistic hook input message of specified size
fn generate_hook_message(size_bytes: usize) -> String {
    let padding_size = size_bytes.saturating_sub(200); // Base message is ~200 bytes
    let padding = "x".repeat(padding_size);

    serde_json::json!({
        "session_id": "test-session-12345",
        "transcript_path": "/tmp/transcript.jsonl",
        "cwd": "/home/user/project",
        "hook_event_name": "pre_tool_use",
        "tool_name": "Read",
        "tool_input": {
            "file_path": "/tmp/test.txt",
            "padding": padding
        }
    })
    .to_string()
}

/// Benchmark JSON parsing at different message sizes
fn bench_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");
    group.measurement_time(Duration::from_secs(10));

    // Test typical message sizes
    let sizes = vec![
        (1024, "1KB"),     // Small typical message
        (5120, "5KB"),     // Medium message
        (10240, "10KB"),   // Large message
        (102400, "100KB"), // Very large (edge case)
    ];

    for (size, label) in sizes {
        let input = generate_hook_message(size);
        group.throughput(Throughput::Bytes(input.len() as u64));

        group.bench_function(BenchmarkId::new("parse", label), |b| {
            b.iter(|| {
                let parsed: HookInput =
                    serde_json::from_str(black_box(&input)).expect("Failed to parse JSON");
                black_box(parsed);
            });
        });
    }

    group.finish();
}

/// Benchmark JSON deserialization with pre-allocated buffers
fn bench_json_with_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    let input_1k = generate_hook_message(1024);
    let input_10k = generate_hook_message(10240);

    // Simulate buffer reuse
    let mut buffer = Vec::with_capacity(102400); // Pre-allocate 100KB

    group.bench_function("buffered/1KB", |b| {
        b.iter(|| {
            buffer.clear();
            buffer.extend_from_slice(input_1k.as_bytes());

            let parsed: HookInput =
                serde_json::from_slice(black_box(&buffer)).expect("Failed to parse JSON");
            black_box(parsed);
        });
    });

    group.bench_function("buffered/10KB", |b| {
        b.iter(|| {
            buffer.clear();
            buffer.extend_from_slice(input_10k.as_bytes());

            let parsed: HookInput =
                serde_json::from_slice(black_box(&buffer)).expect("Failed to parse JSON");
            black_box(parsed);
        });
    });

    group.finish();
}

/// Benchmark specific field access patterns
fn bench_field_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    let input = generate_hook_message(1024);
    let parsed: HookInput = serde_json::from_str(&input).unwrap();

    group.bench_function("field_access/session_id", |b| {
        b.iter(|| {
            black_box(&parsed.session_id);
        });
    });

    group.bench_function("field_access/tool_input", |b| {
        b.iter(|| {
            black_box(&parsed.tool_input);
        });
    });

    group.finish();
}

/// Benchmark zero-copy potential with serde_json::Value
fn bench_json_value(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    let input_1k = generate_hook_message(1024);

    group.bench_function("value/1KB", |b| {
        b.iter(|| {
            let parsed: serde_json::Value =
                serde_json::from_str(black_box(&input_1k)).expect("Failed to parse JSON");
            black_box(parsed);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_json_parsing,
    bench_json_with_buffer,
    bench_field_access,
    bench_json_value
);
criterion_main!(benches);
