//! Benchmarks for CliContext initialization with race condition fixes
//!
//! Ensures our synchronization improvements don't impact performance

use criterion::{Criterion, criterion_group, criterion_main};
use maos::cli::context::CliContext;
use std::hint::black_box;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn bench_context_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("context");
    let rt = Runtime::new().unwrap();

    // Benchmark context build time (should be instant due to lazy init)
    group.bench_function("build", |b| {
        b.iter(|| {
            rt.block_on(async {
                let context = CliContext::build().await.unwrap();
                black_box(context)
            })
        });
    });

    // Benchmark config initialization (first access)
    group.bench_function("config_first_access", |b| {
        b.iter(|| {
            rt.block_on(async {
                let context = CliContext::build().await.unwrap();
                let config = context.config().unwrap();
                black_box(config)
            })
        });
    });

    // Benchmark concurrent config access with our race condition fix
    group.bench_function("config_concurrent_access", |b| {
        b.iter(|| {
            rt.block_on(async {
                let context = Arc::new(CliContext::build().await.unwrap());
                let mut handles = vec![];

                for _ in 0..10 {
                    let ctx = context.clone();
                    let handle = tokio::spawn(async move { ctx.config().unwrap() });
                    handles.push(handle);
                }

                for handle in handles {
                    let config = handle.await.unwrap();
                    black_box(config);
                }
            })
        });
    });

    // Benchmark metrics access (should be near-instant)
    group.bench_function("metrics_access", |b| {
        b.iter(|| {
            rt.block_on(async {
                let context = CliContext::build().await.unwrap();
                let metrics = context.metrics();
                black_box(metrics)
            })
        });
    });

    group.finish();
}

criterion_group!(benches, bench_context_initialization);
criterion_main!(benches);
