//! Startup performance benchmarks for MAOS CLI
//!
//! Measures cold and warm startup times to ensure we meet the <5ms target.

use assert_cmd::Command;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;

/// Benchmark cold start performance (first execution)
fn bench_cold_start(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");
    group.measurement_time(Duration::from_secs(10));

    // Test different commands to see if there's variance
    let test_input = r#"{"session_id":"test","transcript_path":"/tmp/t.jsonl","cwd":"/tmp","hook_event_name":"pre_tool_use","tool_name":"Read","tool_input":{}}"#;

    let commands = vec![
        ("pre-tool-use", vec!["pre-tool-use"], true),
        ("notify", vec!["notify"], true),
        ("help", vec!["--help"], false),
    ];

    for (name, args, needs_stdin) in commands {
        group.bench_function(BenchmarkId::new("cold", name), |b| {
            b.iter(|| {
                let mut cmd = Command::cargo_bin("maos").unwrap();
                cmd.args(&args);

                if needs_stdin {
                    cmd.write_stdin(test_input);
                }

                let output = cmd.output().expect("Failed to execute command");
                black_box(output);
            });
        });
    }

    group.finish();
}

/// Benchmark warm start performance (subsequent executions)
/// This simulates the OS having the binary in cache
fn bench_warm_start(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");
    group.measurement_time(Duration::from_secs(10));

    // Warm up the OS cache first
    for _ in 0..5 {
        Command::cargo_bin("maos")
            .unwrap()
            .arg("--help")
            .output()
            .expect("Failed to execute warmup");
    }

    group.bench_function("warm/help", |b| {
        b.iter(|| {
            let output = Command::cargo_bin("maos")
                .unwrap()
                .arg("--help")
                .output()
                .expect("Failed to execute command");

            black_box(output);
        });
    });

    group.finish();
}

/// Benchmark CLI argument parsing overhead
fn bench_arg_parsing(c: &mut Criterion) {
    use clap::Parser;
    use maos::cli::Cli;

    let mut group = c.benchmark_group("startup");

    group.bench_function("arg_parsing/simple", |b| {
        b.iter(|| {
            let cli = Cli::try_parse_from(black_box(&["maos", "notify"]));
            let _ = black_box(cli);
        });
    });

    group.bench_function("arg_parsing/with_flags", |b| {
        b.iter(|| {
            let cli = Cli::try_parse_from(black_box(&["maos", "stop", "--chat"]));
            let _ = black_box(cli);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cold_start,
    bench_warm_start,
    bench_arg_parsing
);
criterion_main!(benches);
