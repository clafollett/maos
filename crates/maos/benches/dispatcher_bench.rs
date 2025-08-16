//! Performance benchmarks for command dispatcher

use async_trait::async_trait;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use maos::cli::{
    Commands,
    handler::{CommandHandler, CommandResult, ExecutionMetrics},
    registry::HandlerRegistry,
};
use maos::io::HookInput;
use maos_core::{ExitCode, Result, config::MaosConfig};
use std::hint::black_box;
use tokio::runtime::Runtime;

/// Mock handler for benchmarking
struct MockHandler {
    name: &'static str,
}

#[async_trait]
impl CommandHandler for MockHandler {
    async fn execute(&self, _input: HookInput) -> Result<CommandResult> {
        Ok(CommandResult {
            exit_code: ExitCode::Success,
            output: Some("Mock output".to_string()),
            metrics: ExecutionMetrics::default(),
        })
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

fn benchmark_handler_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatcher");
    let rt = Runtime::new().unwrap();

    // Build registry with all handlers
    let config = MaosConfig::default();
    let registry = rt.block_on(HandlerRegistry::build(&config)).unwrap();

    // Benchmark O(1) handler lookup
    group.bench_function("handler_lookup", |b| {
        b.iter(|| {
            let command = Commands::PreToolUse;
            registry.get_handler(black_box(&command))
        });
    });

    // Benchmark different command lookups
    for (id, command) in [
        ("pre_tool", Commands::PreToolUse),
        ("post_tool", Commands::PostToolUse),
        ("notify", Commands::Notify),
        ("session", Commands::SessionStart),
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("lookup_by_command", id),
            command,
            |b, cmd| {
                b.iter(|| registry.get_handler(black_box(cmd)));
            },
        );
    }

    group.finish();
}

fn benchmark_handler_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("handler_execution");
    let rt = Runtime::new().unwrap();

    let handler = MockHandler {
        name: "test_handler",
    };
    let input = HookInput {
        session_id: "bench_session".to_string(),
        transcript_path: "/tmp/bench.jsonl".into(),
        cwd: "/tmp".into(),
        hook_event_name: "pre_tool_use".to_string(),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({"command": "ls"})),
        ..Default::default()
    };

    // Benchmark async handler execution
    group.bench_function("async_execute", |b| {
        b.iter(|| rt.block_on(handler.execute(black_box(input.clone()))));
    });

    // Benchmark validation
    group.bench_function("validate_input", |b| {
        b.iter(|| handler.validate_input(black_box(&input)));
    });

    group.finish();
}

fn benchmark_registry_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry");
    let rt = Runtime::new().unwrap();

    let config = MaosConfig::default();

    // Benchmark registry build time
    group.bench_function("build_registry", |b| {
        b.iter(|| rt.block_on(HandlerRegistry::build(black_box(&config))));
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_handler_lookup,
    benchmark_handler_execution,
    benchmark_registry_build
);
criterion_main!(benches);
