use maos_core::metrics::{MetricsReport, PerformanceMetrics};
use maos_core::timed_operation;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_record_execution_time() {
    let metrics = PerformanceMetrics::new();

    // Record some execution times
    metrics.record_execution_time("test_op", Duration::from_millis(10));
    metrics.record_execution_time("test_op", Duration::from_millis(20));
    metrics.record_execution_time("test_op", Duration::from_millis(15));

    // Export and verify
    let report = metrics.export_metrics();
    let stats = report
        .execution_stats
        .get("test_op")
        .expect("Missing stats");

    assert_eq!(stats.count, 3);
    assert_eq!(stats.avg_ms, 15.0); // (10 + 20 + 15) / 3
    assert_eq!(stats.max_ms, 20.0);
    assert_eq!(stats.min_ms, 10.0);
}

#[test]
fn test_record_memory_usage() {
    let metrics = PerformanceMetrics::new();

    // Record memory usage
    metrics.record_memory_usage("allocation", 1024);
    metrics.record_memory_usage("allocation", 2048);
    metrics.record_memory_usage("allocation", 1536);

    // Export and verify
    let report = metrics.export_metrics();
    let stats = report
        .memory_stats
        .get("allocation")
        .expect("Missing stats");

    assert_eq!(stats.count, 3);
    assert_eq!(stats.avg_bytes, 1536); // (1024 + 2048 + 1536) / 3
    assert_eq!(stats.max_bytes, 2048);
    assert_eq!(stats.min_bytes, 1024);
}

#[test]
fn test_error_counting() {
    let metrics = PerformanceMetrics::new();

    // Record errors
    metrics.record_error("parse_error");
    metrics.record_error("parse_error");
    metrics.record_error("io_error");

    // Export and verify
    let report = metrics.export_metrics();

    assert_eq!(*report.error_counts.get("parse_error").unwrap(), 2);
    assert_eq!(*report.error_counts.get("io_error").unwrap(), 1);
}

#[test]
fn test_metrics_export_format() {
    let metrics = PerformanceMetrics::new();

    // Add various metrics
    metrics.record_execution_time("op1", Duration::from_millis(5));
    metrics.record_memory_usage("alloc1", 512);
    metrics.record_error("error1");

    // Export
    let report = metrics.export_metrics();

    // Verify structure
    assert!(report.execution_stats.contains_key("op1"));
    assert!(report.memory_stats.contains_key("alloc1"));
    assert!(report.error_counts.contains_key("error1"));
    assert!(report.generated_at.timestamp() > 0);
}

#[test]
fn test_concurrent_metrics_updates() {
    let metrics = Arc::new(PerformanceMetrics::new());
    let mut handles = vec![];

    // Spawn threads that update metrics concurrently
    for i in 0..10 {
        let metrics = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for j in 0..100 {
                metrics.record_execution_time(
                    &format!("op_{}", i % 3),
                    Duration::from_millis(j as u64),
                );
                if j % 10 == 0 {
                    metrics.record_error(&format!("error_{}", i % 2));
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify metrics were collected
    let report = metrics.export_metrics();

    // Should have metrics for op_0, op_1, op_2
    assert_eq!(report.execution_stats.len(), 3);

    // Should have errors for error_0, error_1
    assert_eq!(report.error_counts.len(), 2);

    // Each op should have multiple samples
    for stats in report.execution_stats.values() {
        assert!(stats.count > 0);
    }
}

#[test]
fn test_sample_size_limiting() {
    let metrics = PerformanceMetrics::new();

    // Record more than 1000 samples
    for i in 0..1500 {
        metrics.record_execution_time("test_op", Duration::from_millis(i));
    }

    // Export and verify only last 1000 are kept
    let report = metrics.export_metrics();
    let stats = report
        .execution_stats
        .get("test_op")
        .expect("Missing stats");

    assert_eq!(stats.count, 1000, "Should keep only last 1000 samples");

    // The min should be 500 (samples 500-1499 were kept)
    assert_eq!(stats.min_ms, 500.0);
    assert_eq!(stats.max_ms, 1499.0);
}

#[test]
fn test_timed_operation_macro() {
    let metrics = PerformanceMetrics::new();

    // ✅ PROPER TEST: Tests our macro logic, not OS timing precision
    let result = timed_operation!(metrics, "test_macro_op", { 42 });

    assert_eq!(result, 42, "Macro should return operation result");

    // Verify timing was recorded (test that metric collection works, not timing accuracy)
    let report = metrics.export_metrics();
    let stats = report
        .execution_stats
        .get("test_macro_op")
        .expect("Missing stats");

    assert_eq!(stats.count, 1);
    assert!(stats.avg_ms > 0.0, "Should record some duration");
}

#[test]
fn test_percentile_calculations() {
    let metrics = PerformanceMetrics::new();

    // Record predictable samples
    for i in 1..=100 {
        metrics.record_execution_time("percentile_test", Duration::from_millis(i));
    }

    let report = metrics.export_metrics();
    let stats = report
        .execution_stats
        .get("percentile_test")
        .expect("Missing stats");

    // For 1-100ms samples:
    assert_eq!(stats.p50_ms, 50.0, "50th percentile should be 50ms");
    assert_eq!(stats.p95_ms, 95.0, "95th percentile should be 95ms");
    assert_eq!(stats.p99_ms, 99.0, "99th percentile should be 99ms");
}

#[test]
fn test_empty_metrics() {
    let metrics = PerformanceMetrics::new();

    // Export empty metrics
    let report = metrics.export_metrics();

    assert!(report.execution_stats.is_empty());
    assert!(report.memory_stats.is_empty());
    assert!(report.error_counts.is_empty());
}

#[test]
fn test_metrics_serialization() {
    let metrics = PerformanceMetrics::new();

    // Add some data
    metrics.record_execution_time("op", Duration::from_millis(10));
    metrics.record_memory_usage("mem", 1024);
    metrics.record_error("err");

    // Export and serialize
    let report = metrics.export_metrics();
    let json = serde_json::to_string(&report).expect("Failed to serialize");

    // Verify JSON contains expected fields
    assert!(json.contains("\"execution_stats\""));
    assert!(json.contains("\"memory_stats\""));
    assert!(json.contains("\"error_counts\""));
    assert!(json.contains("\"generated_at\""));

    // Deserialize back
    let deserialized: MetricsReport = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.execution_stats.len(), 1);
    assert_eq!(deserialized.memory_stats.len(), 1);
    assert_eq!(deserialized.error_counts.len(), 1);
}
