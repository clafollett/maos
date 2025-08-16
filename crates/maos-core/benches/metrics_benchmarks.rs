use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use maos_core::metrics::PerformanceMetrics;
use maos_core::timed_operation;
use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn benchmark_record_execution_time(c: &mut Criterion) {
    let metrics = PerformanceMetrics::new();

    c.bench_function("record_execution_time", |b| {
        b.iter(|| {
            metrics
                .record_execution_time(black_box("test_op"), black_box(Duration::from_millis(10)));
        });
    });
}

fn benchmark_concurrent_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_metrics");

    for thread_count in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            thread_count,
            |b, &thread_count| {
                let metrics = Arc::new(PerformanceMetrics::new());

                b.iter(|| {
                    let handles: Vec<_> = (0..thread_count)
                        .map(|i| {
                            let metrics = Arc::clone(&metrics);
                            thread::spawn(move || {
                                metrics.record_execution_time(
                                    &format!("op_{}", i),
                                    Duration::from_micros(100),
                                );
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }
    group.finish();
}

fn benchmark_timed_operation_macro(c: &mut Criterion) {
    let metrics = PerformanceMetrics::new();

    c.bench_function("timed_operation_macro_overhead", |b| {
        b.iter(|| {
            timed_operation!(metrics, "bench_op", {
                // Minimal operation to measure macro overhead
                black_box(42)
            })
        });
    });
}

fn benchmark_export_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("export_metrics");

    for sample_count in [10, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(sample_count),
            sample_count,
            |b, &sample_count| {
                let metrics = PerformanceMetrics::new();

                // Pre-populate with samples
                for i in 0..sample_count {
                    metrics.record_execution_time("test_op", Duration::from_millis(i as u64));
                }

                b.iter(|| {
                    black_box(metrics.export_metrics());
                });
            },
        );
    }
    group.finish();
}

fn benchmark_percentile_calculation(c: &mut Criterion) {
    let metrics = PerformanceMetrics::new();

    // Populate with 1000 samples (max)
    for i in 0..1000 {
        metrics.record_execution_time("percentile_op", Duration::from_micros(i));
    }

    c.bench_function("percentile_calculation_1000_samples", |b| {
        b.iter(|| {
            let report = metrics.export_metrics();
            black_box(report.execution_stats.get("percentile_op"));
        });
    });
}

criterion_group!(
    benches,
    benchmark_record_execution_time,
    benchmark_concurrent_metrics,
    benchmark_timed_operation_macro,
    benchmark_export_metrics,
    benchmark_percentile_calculation
);
criterion_main!(benches);
