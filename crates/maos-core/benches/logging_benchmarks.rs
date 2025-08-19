use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use maos_core::SessionId;
use maos_core::logging::{RollingLogConfig, SessionLogger};
use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

fn benchmark_session_logger_write(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let session_id = SessionId::generate();
    let config = RollingLogConfig::default();
    let mut logger = SessionLogger::new(session_id, temp_dir.path().to_path_buf(), config).unwrap();

    let log_entry = r#"{"level":"info","timestamp":"2024-01-01T00:00:00Z","msg":"Test message"}"#;

    c.bench_function("session_logger_write", |b| {
        b.iter(|| {
            logger.write(black_box(log_entry)).unwrap();
        });
    });
}

fn benchmark_concurrent_logging(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_logging");

    for thread_count in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            thread_count,
            |b, &thread_count| {
                let temp_dir = TempDir::new().unwrap();
                let session_id = SessionId::generate();
                let config = RollingLogConfig::default();
                let logger = Arc::new(
                    SessionLogger::new(session_id, temp_dir.path().to_path_buf(), config)
                        .unwrap()
                        .into_thread_safe(),
                );

                b.iter(|| {
                    let handles: Vec<_> = (0..thread_count)
                        .map(|i| {
                            let logger = Arc::clone(&logger);
                            thread::spawn(move || {
                                logger.write(&format!("Thread {i} entry")).unwrap();
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

fn benchmark_log_rotation(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let session_id = SessionId::generate();

    // Small file size to trigger rotation
    let config = RollingLogConfig {
        max_file_size_bytes: 1024, // 1KB
        max_files_per_session: 5,
        compress_on_roll: false,
        file_pattern: "session-{session_id}.log".to_string(),
    };

    let mut logger = SessionLogger::new(session_id, temp_dir.path().to_path_buf(), config).unwrap();

    let large_entry = "x".repeat(512); // Half the max size

    c.bench_function("log_rotation_trigger", |b| {
        b.iter(|| {
            // This will trigger rotation every 2-3 writes
            logger.write(black_box(&large_entry)).unwrap();
        });
    });
}

fn benchmark_log_compression(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let session_id = SessionId::generate();

    let config = RollingLogConfig {
        max_file_size_bytes: 1024,
        max_files_per_session: 5,
        compress_on_roll: true, // Enable compression
        file_pattern: "session-{session_id}.log".to_string(),
    };

    let mut logger = SessionLogger::new(session_id, temp_dir.path().to_path_buf(), config).unwrap();

    let large_entry = "x".repeat(512);

    c.bench_function("log_rotation_with_compression", |b| {
        b.iter(|| {
            logger.write(black_box(&large_entry)).unwrap();
        });
    });
}

criterion_group!(
    benches,
    benchmark_session_logger_write,
    benchmark_concurrent_logging,
    benchmark_log_rotation,
    benchmark_log_compression
);
criterion_main!(benches);
