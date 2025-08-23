//! Memory profiling benchmarks for MAOS
//!
//! These benchmarks measure memory usage patterns under various workloads

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use serde_json::json;
use std::hint::black_box;

/// Generate a hook message of specified size
fn generate_hook_message(size_bytes: usize) -> String {
    let padding_size = size_bytes.saturating_sub(200); // Account for JSON structure
    let padding = "x".repeat(padding_size);

    json!({
        "session_id": "bench-session",
        "transcript_path": "/tmp/bench.jsonl",
        "cwd": "/tmp",
        "hook_event_name": "pre_tool_use",
        "tool_name": "Read",
        "tool_input": {
            "file_path": "/tmp/test.txt",
            "data": padding
        }
    })
    .to_string()
}

/// Benchmark memory allocation patterns
fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");

    // Test rapid small allocations
    group.bench_function("rapid_small_allocs", |b| {
        b.iter(|| {
            let mut allocations = Vec::new();
            for _ in 0..1000 {
                let msg = generate_hook_message(100);
                allocations.push(msg);
            }
            black_box(allocations);
        });
    });

    // Test few large allocations
    group.bench_function("few_large_allocs", |b| {
        b.iter(|| {
            let mut allocations = Vec::new();
            for _ in 0..10 {
                let msg = generate_hook_message(100_000);
                allocations.push(msg);
            }
            black_box(allocations);
        });
    });

    // Test mixed allocation patterns
    group.bench_function("mixed_allocs", |b| {
        b.iter(|| {
            let mut allocations = Vec::new();
            for i in 0..100 {
                let size = if i % 10 == 0 { 10_000 } else { 100 };
                let msg = generate_hook_message(size);
                allocations.push(msg);
            }
            black_box(allocations);
        });
    });

    group.finish();
}

/// Benchmark JSON parsing at different sizes
fn bench_json_parsing_memory(c: &mut Criterion) {
    use maos::io::messages::HookInput;

    let mut group = c.benchmark_group("json_memory");

    let sizes = vec![("1KB", 1024), ("10KB", 10 * 1024), ("100KB", 100 * 1024)];

    for (name, size) in sizes {
        let message = generate_hook_message(size);

        group.bench_with_input(BenchmarkId::from_parameter(name), &message, |b, msg| {
            b.iter(|| {
                let parsed: Result<HookInput, _> = serde_json::from_str(msg);
                let _ = black_box(parsed);
            });
        });
    }

    group.finish();
}

/// Benchmark memory cleanup and deallocation
fn bench_memory_cleanup(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_cleanup");

    // Test cleanup of temporary allocations
    group.bench_function("temp_cleanup", |b| {
        b.iter(|| {
            {
                // Scope to ensure cleanup
                let _temp_data: Vec<String> =
                    (0..100).map(|_| generate_hook_message(1024)).collect();
                // Data goes out of scope here
            }
            // Force some work after cleanup
            black_box(42);
        });
    });

    // Test cleanup with drops
    group.bench_function("explicit_drops", |b| {
        b.iter(|| {
            let mut data: Vec<String> = (0..100).map(|_| generate_hook_message(1024)).collect();

            // Explicitly drop half the data
            for _ in 0..50 {
                data.pop();
            }

            black_box(data);
        });
    });

    group.finish();
}

/// Benchmark string operations with various sizes
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_ops");

    // Benchmark string concatenation
    group.bench_function("concatenation", |b| {
        b.iter(|| {
            let mut result = String::new();
            for i in 0..100 {
                result.push_str(&format!("iteration_{i}_"));
            }
            black_box(result);
        });
    });

    // Benchmark string cloning
    group.bench_function("cloning", |b| {
        let source = generate_hook_message(10_000);
        b.iter(|| {
            let cloned = source.clone();
            black_box(cloned);
        });
    });

    group.finish();
}

criterion_group!(
    memory_benches,
    bench_allocation_patterns,
    bench_json_parsing_memory,
    bench_memory_cleanup,
    bench_string_operations
);

criterion_main!(memory_benches);
