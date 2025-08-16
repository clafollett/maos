//! Metrics reporting structures and statistics calculations
//!
//! This module defines the data structures used to represent collected metrics
//! and provides efficient percentile calculations for performance analysis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Statistics for execution times of operations
///
/// Provides comprehensive performance metrics for analyzing operation timing patterns.
/// Includes percentile calculations to identify performance outliers and bottlenecks.
///
/// # Usage
///
/// Use this struct to monitor Claude Code hook execution performance and identify
/// operations that exceed performance targets.
///
/// # Example
///
/// ```rust
/// use maos_core::metrics::report::ExecutionStats;
/// use std::time::Duration;
///
/// let durations = vec![
///     Duration::from_millis(5),   // Fast operation
///     Duration::from_millis(15),  // Typical operation
///     Duration::from_millis(50),  // Slow operation
/// ];
///
/// let stats = ExecutionStats::from_durations(&durations);
/// println!("Average: {:.1}ms, P95: {:.1}ms", stats.avg_ms, stats.p95_ms);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// Total number of execution time samples collected
    ///
    /// Indicates the statistical significance of the measurements.
    /// Larger sample sizes provide more reliable statistics.
    pub count: usize,

    /// Arithmetic mean execution time in milliseconds
    ///
    /// Useful for understanding typical performance but can be skewed
    /// by outliers. Compare with median (p50_ms) to detect skewness.
    pub avg_ms: f64,

    /// Worst-case execution time in milliseconds
    ///
    /// Critical for identifying performance spikes that could impact
    /// user experience or violate SLA requirements.
    pub max_ms: f64,

    /// Best-case execution time in milliseconds
    ///
    /// Represents optimal performance under ideal conditions.
    /// Large min/max gaps indicate inconsistent performance.
    pub min_ms: f64,

    /// Median (50th percentile) execution time in milliseconds
    ///
    /// The middle value when all execution times are sorted.
    /// More robust than average for skewed distributions.
    pub p50_ms: f64,

    /// 95th percentile execution time in milliseconds
    ///
    /// Only 5% of operations take longer than this value.
    /// Key metric for performance SLAs and user experience monitoring.
    pub p95_ms: f64,

    /// 99th percentile execution time in milliseconds
    ///
    /// Only 1% of operations take longer than this value.
    /// Used to identify rare but severe performance problems.
    pub p99_ms: f64,
}

impl ExecutionStats {
    /// Calculate execution statistics from a collection of durations
    ///
    /// # Arguments
    ///
    /// * `durations` - Vector of Duration samples
    ///
    /// # Returns
    ///
    /// ExecutionStats with calculated percentiles and basic stats
    pub fn from_durations(durations: &[Duration]) -> Self {
        if durations.is_empty() {
            return Self {
                count: 0,
                avg_ms: 0.0,
                max_ms: 0.0,
                min_ms: 0.0,
                p50_ms: 0.0,
                p95_ms: 0.0,
                p99_ms: 0.0,
            };
        }

        // Convert to milliseconds and sort for percentile calculations
        let mut ms_values: Vec<f64> = durations.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
        ms_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = ms_values.len();
        let sum: f64 = ms_values.iter().sum();
        let avg_ms = sum / count as f64;
        let max_ms = ms_values[count - 1];
        let min_ms = ms_values[0];

        // Calculate percentiles using linear interpolation
        let p50_ms = calculate_percentile(&ms_values, 50.0);
        let p95_ms = calculate_percentile(&ms_values, 95.0);
        let p99_ms = calculate_percentile(&ms_values, 99.0);

        Self {
            count,
            avg_ms,
            max_ms,
            min_ms,
            p50_ms,
            p95_ms,
            p99_ms,
        }
    }
}

/// Statistics for memory usage of operations
///
/// Tracks memory consumption patterns to identify memory leaks, inefficient
/// allocations, and operations that may cause out-of-memory conditions.
///
/// # Usage
///
/// Monitor these metrics to ensure MAOS stays within memory budgets and
/// identify operations that consume excessive memory resources.
///
/// # Example
///
/// ```rust
/// use maos_core::metrics::report::MemoryStats;
///
/// let memory_samples = vec![
///     1024 * 1024,      // 1MB baseline
///     2 * 1024 * 1024,  // 2MB during operation
///     1024 * 1024,      // 1MB after cleanup
/// ];
///
/// let stats = MemoryStats::from_samples(&memory_samples);
/// println!("Peak memory: {}MB", stats.max_bytes / (1024 * 1024));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total number of memory usage samples collected
    ///
    /// More samples provide better insight into memory usage patterns
    /// and help identify memory leaks or gradual increases.
    pub count: usize,

    /// Average memory usage across all samples in bytes
    ///
    /// Indicates typical memory consumption during operation.
    /// Combine with max_bytes to understand memory efficiency.
    pub avg_bytes: usize,

    /// Peak memory usage observed in bytes
    ///
    /// Critical for capacity planning and preventing out-of-memory errors.
    /// High values may indicate memory leaks or inefficient algorithms.
    pub max_bytes: usize,

    /// Minimum memory usage observed in bytes
    ///
    /// Represents baseline memory usage when the operation is idle.
    /// Large min/max differences indicate significant memory fluctuation.
    pub min_bytes: usize,
}

impl MemoryStats {
    /// Calculate memory statistics from a collection of byte counts
    ///
    /// # Arguments
    ///
    /// * `memory_samples` - Vector of memory usage samples in bytes
    ///
    /// # Returns
    ///
    /// MemoryStats with calculated averages and extremes
    pub fn from_samples(memory_samples: &[usize]) -> Self {
        if memory_samples.is_empty() {
            return Self {
                count: 0,
                avg_bytes: 0,
                max_bytes: 0,
                min_bytes: 0,
            };
        }

        let count = memory_samples.len();
        let sum: usize = memory_samples.iter().sum();
        let avg_bytes = sum / count;
        let max_bytes = *memory_samples.iter().max().unwrap();
        let min_bytes = *memory_samples.iter().min().unwrap();

        Self {
            count,
            avg_bytes,
            max_bytes,
            min_bytes,
        }
    }
}

/// Complete metrics report containing all collected performance data
///
/// Aggregates all performance metrics collected during a MAOS session into
/// a comprehensive report suitable for monitoring, analysis, and debugging.
///
/// # Usage
///
/// This report is typically generated at session end or periodically for
/// long-running sessions. It provides a complete view of system performance
/// across all operations and can be exported for external analysis.
///
/// # Example
///
/// ```rust
/// use maos_core::metrics::report::MetricsReport;
///
/// let mut report = MetricsReport::new();
/// // Metrics would be populated by the collector during operation
///
/// println!("Report generated at: {}", report.generated_at);
/// println!("Operations monitored: {}", report.execution_stats.len());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsReport {
    /// Execution time statistics grouped by operation name
    ///
    /// Maps operation names (e.g., "pre_tool_use", "post_tool_use") to their
    /// respective execution time statistics. Use to identify slow operations.
    pub execution_stats: HashMap<String, ExecutionStats>,

    /// Memory usage statistics grouped by operation name
    ///
    /// Maps operation names to their memory consumption patterns.
    /// Critical for identifying memory leaks and resource-intensive operations.
    pub memory_stats: HashMap<String, MemoryStats>,

    /// Error occurrence counts grouped by error type or operation name
    ///
    /// Tracks how frequently different types of errors occur.
    /// Use for reliability monitoring and identifying problematic operations.
    pub error_counts: HashMap<String, usize>,

    /// UTC timestamp indicating when this report was generated
    ///
    /// Enables temporal analysis and correlation with external events.
    /// All timestamps are in UTC for consistency across time zones.
    pub generated_at: DateTime<Utc>,
}

impl MetricsReport {
    /// Create a new empty metrics report
    pub fn new() -> Self {
        Self {
            execution_stats: HashMap::new(),
            memory_stats: HashMap::new(),
            error_counts: HashMap::new(),
            generated_at: Utc::now(),
        }
    }
}

impl Default for MetricsReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate percentile using linear interpolation
///
/// This function calculates the percentile value using the nearest-rank method
/// with linear interpolation between ranks when the index falls between integers.
///
/// # Arguments
///
/// * `sorted_values` - A sorted slice of f64 values
/// * `percentile` - The percentile to calculate (0.0 to 100.0)
///
/// # Returns
///
/// The calculated percentile value
fn calculate_percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    if sorted_values.len() == 1 {
        return sorted_values[0];
    }

    let n = sorted_values.len();

    // Handle boundary cases explicitly
    if percentile <= 0.0 {
        return sorted_values[0];
    }
    if percentile >= 100.0 {
        return sorted_values[n - 1];
    }

    // Use the nearest-rank method for percentile calculation
    // For 1-100 values, p50 should be value at index 49 (0-based), which is 50
    let index = ((percentile / 100.0) * n as f64).ceil() as isize - 1;
    let clamped_index = index.max(0).min((n - 1) as isize) as usize;

    sorted_values[clamped_index]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_execution_stats_calculation() {
        let durations = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(15),
        ];

        let stats = ExecutionStats::from_durations(&durations);

        assert_eq!(stats.count, 3);
        assert_eq!(stats.avg_ms, 15.0);
        assert_eq!(stats.max_ms, 20.0);
        assert_eq!(stats.min_ms, 10.0);
    }

    #[test]
    fn test_percentile_calculation() {
        // Test with 1-100ms values for predictable percentiles
        let values: Vec<f64> = (1..=100).map(|i| i as f64).collect();

        assert_eq!(calculate_percentile(&values, 50.0), 50.0);
        assert_eq!(calculate_percentile(&values, 95.0), 95.0);
        assert_eq!(calculate_percentile(&values, 99.0), 99.0);
    }

    #[test]
    fn test_memory_stats_calculation() {
        let samples = vec![1024, 2048, 1536];
        let stats = MemoryStats::from_samples(&samples);

        assert_eq!(stats.count, 3);
        assert_eq!(stats.avg_bytes, 1536); // (1024 + 2048 + 1536) / 3
        assert_eq!(stats.max_bytes, 2048);
        assert_eq!(stats.min_bytes, 1024);
    }

    #[test]
    fn test_empty_stats() {
        let empty_durations: Vec<Duration> = vec![];
        let stats = ExecutionStats::from_durations(&empty_durations);
        assert_eq!(stats.count, 0);
        assert_eq!(stats.avg_ms, 0.0);

        let empty_memory: Vec<usize> = vec![];
        let mem_stats = MemoryStats::from_samples(&empty_memory);
        assert_eq!(mem_stats.count, 0);
        assert_eq!(mem_stats.avg_bytes, 0);
    }
}
