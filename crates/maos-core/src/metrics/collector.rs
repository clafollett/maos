//! Performance metrics collector with thread-safe operations
//!
//! This module implements the core PerformanceMetrics collector that provides
//! thread-safe recording of execution times, memory usage, and error counts.

use crate::metrics::report::{ExecutionStats, MemoryStats, MetricsReport};
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

/// Type alias for execution time samples storage
type ExecutionTimesMap = HashMap<String, VecDeque<Duration>>;

/// Type alias for memory usage samples storage
type MemoryUsageMap = HashMap<String, VecDeque<usize>>;

/// Type alias for error counts storage
type ErrorCountsMap = HashMap<String, usize>;

/// Maximum number of samples to keep per operation to prevent unbounded memory growth
const MAX_SAMPLES_PER_OPERATION: usize = 1000;

/// Thread-safe performance metrics collector
///
/// This collector maintains execution times, memory usage samples, and error counts
/// for various operations. It uses RwLock for thread safety and limits sample sizes
/// to prevent memory bloat in long-running applications.
///
/// # Thread Safety
///
/// All operations are thread-safe using parking_lot::RwLock, allowing concurrent
/// reads and exclusive writes.
///
/// # Sample Limiting
///
/// To prevent unbounded memory growth, only the most recent 1000 samples are kept
/// for each operation. Older samples are automatically discarded.
#[derive(Debug)]
pub struct PerformanceMetrics {
    /// Execution time samples by operation name
    execution_times: RwLock<ExecutionTimesMap>,
    /// Memory usage samples by operation name
    memory_usage: RwLock<MemoryUsageMap>,
    /// Error counts by error type/operation name
    error_counts: RwLock<ErrorCountsMap>,
}

impl PerformanceMetrics {
    /// Create a new performance metrics collector
    ///
    /// # Returns
    ///
    /// A new PerformanceMetrics instance ready for collecting metrics
    ///
    /// # Example
    ///
    /// ```
    /// use maos_core::metrics::PerformanceMetrics;
    ///
    /// let metrics = PerformanceMetrics::new();
    /// ```
    pub fn new() -> Self {
        Self {
            execution_times: RwLock::new(HashMap::new()),
            memory_usage: RwLock::new(HashMap::new()),
            error_counts: RwLock::new(HashMap::new()),
        }
    }

    /// Record execution time for an operation
    ///
    /// This method is thread-safe and will automatically limit the number of
    /// samples kept per operation to prevent memory bloat.
    ///
    /// # Arguments
    ///
    /// * `operation` - The name of the operation being timed
    /// * `duration` - The execution time to record
    ///
    /// # Example
    ///
    /// ```
    /// use maos_core::metrics::PerformanceMetrics;
    /// use std::time::Duration;
    ///
    /// let metrics = PerformanceMetrics::new();
    /// metrics.record_execution_time("database_query", Duration::from_millis(42));
    /// ```
    pub fn record_execution_time(&self, operation: &str, duration: Duration) {
        let mut times = self.execution_times.write();
        let samples = times.entry(operation.to_string()).or_default();

        samples.push_back(duration);

        // Limit sample size to prevent unbounded memory growth
        if samples.len() > MAX_SAMPLES_PER_OPERATION {
            samples.pop_front();
        }
    }

    /// Record memory usage for an operation
    ///
    /// This method is thread-safe and will automatically limit the number of
    /// samples kept per operation to prevent memory bloat.
    ///
    /// # Arguments
    ///
    /// * `operation` - The name of the operation
    /// * `bytes` - The memory usage in bytes
    ///
    /// # Example
    ///
    /// ```
    /// use maos_core::metrics::PerformanceMetrics;
    ///
    /// let metrics = PerformanceMetrics::new();
    /// metrics.record_memory_usage("buffer_allocation", 1024);
    /// ```
    pub fn record_memory_usage(&self, operation: &str, bytes: usize) {
        let mut usage = self.memory_usage.write();
        let samples = usage.entry(operation.to_string()).or_default();

        samples.push_back(bytes);

        // Limit sample size to prevent unbounded memory growth
        if samples.len() > MAX_SAMPLES_PER_OPERATION {
            samples.pop_front();
        }
    }

    /// Record an error occurrence
    ///
    /// This method is thread-safe and increments the error count for the
    /// specified error type or operation.
    ///
    /// # Arguments
    ///
    /// * `error_type` - The type or category of error that occurred
    ///
    /// # Example
    ///
    /// ```
    /// use maos_core::metrics::PerformanceMetrics;
    ///
    /// let metrics = PerformanceMetrics::new();
    /// metrics.record_error("network_timeout");
    /// metrics.record_error("network_timeout"); // Count is now 2
    /// ```
    pub fn record_error(&self, error_type: &str) {
        let mut counts = self.error_counts.write();
        *counts.entry(error_type.to_string()).or_insert(0) += 1;
    }

    /// Export all collected metrics as a comprehensive report
    ///
    /// This method creates a snapshot of all collected metrics, calculating
    /// statistics and percentiles for execution times and memory usage.
    ///
    /// # Returns
    ///
    /// A MetricsReport containing all current metrics data with calculated statistics
    ///
    /// # Example
    ///
    /// ```
    /// use maos_core::metrics::PerformanceMetrics;
    /// use std::time::Duration;
    ///
    /// let metrics = PerformanceMetrics::new();
    /// metrics.record_execution_time("test_op", Duration::from_millis(10));
    ///
    /// let report = metrics.export_metrics();
    /// assert!(report.execution_stats.contains_key("test_op"));
    /// ```
    pub fn export_metrics(&self) -> MetricsReport {
        // Take snapshots of all data
        let execution_times = self.execution_times.read();
        let memory_usage = self.memory_usage.read();
        let error_counts = self.error_counts.read();

        let mut report = MetricsReport::new();

        // Calculate execution statistics
        for (operation, samples) in execution_times.iter() {
            let samples_vec: Vec<Duration> = samples.iter().copied().collect();
            let stats = ExecutionStats::from_durations(&samples_vec);
            report.execution_stats.insert(operation.clone(), stats);
        }

        // Calculate memory statistics
        for (operation, samples) in memory_usage.iter() {
            let samples_vec: Vec<usize> = samples.iter().copied().collect();
            let stats = MemoryStats::from_samples(&samples_vec);
            report.memory_stats.insert(operation.clone(), stats);
        }

        // Copy error counts
        report.error_counts = error_counts.clone();

        // Set generation timestamp
        report.generated_at = Utc::now();

        report
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_functionality() {
        let metrics = PerformanceMetrics::new();

        // Record some data
        metrics.record_execution_time("test_op", Duration::from_millis(10));
        metrics.record_memory_usage("test_alloc", 1024);
        metrics.record_error("test_error");

        // Export and verify
        let report = metrics.export_metrics();

        assert!(report.execution_stats.contains_key("test_op"));
        assert!(report.memory_stats.contains_key("test_alloc"));
        assert_eq!(*report.error_counts.get("test_error").unwrap(), 1);
    }

    #[test]
    fn test_sample_limiting() {
        let metrics = PerformanceMetrics::new();

        // Add more than MAX_SAMPLES_PER_OPERATION
        for i in 0..1500 {
            metrics.record_execution_time("test_op", Duration::from_millis(i));
        }

        let report = metrics.export_metrics();
        let stats = report.execution_stats.get("test_op").unwrap();

        // Should only keep the last 1000 samples
        assert_eq!(stats.count, MAX_SAMPLES_PER_OPERATION);

        // Min should be 500 (samples 500-1499 were kept, since we start from 0)
        assert_eq!(stats.min_ms, 500.0);
        assert_eq!(stats.max_ms, 1499.0);
    }

    #[test]
    fn test_thread_safety() {
        let metrics = Arc::new(PerformanceMetrics::new());
        let mut handles = vec![];

        // Spawn multiple threads
        for i in 0..10 {
            let metrics = Arc::clone(&metrics);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    metrics
                        .record_execution_time(&format!("op_{}", i % 3), Duration::from_millis(j));
                    if j % 10 == 0 {
                        metrics.record_error(&format!("error_{}", i % 2));
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify data was collected correctly
        let report = metrics.export_metrics();
        assert_eq!(report.execution_stats.len(), 3); // op_0, op_1, op_2
        assert_eq!(report.error_counts.len(), 2); // error_0, error_1

        // Each operation should have samples
        for stats in report.execution_stats.values() {
            assert!(stats.count > 0);
        }
    }
}
