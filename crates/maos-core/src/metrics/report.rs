//! Metrics reporting structures and statistics calculations
//!
//! This module defines the data structures used to represent collected metrics
//! and provides efficient percentile calculations for performance analysis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Statistics for execution times of operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// Number of samples collected
    pub count: usize,
    /// Average execution time in milliseconds
    pub avg_ms: f64,
    /// Maximum execution time in milliseconds
    pub max_ms: f64,
    /// Minimum execution time in milliseconds
    pub min_ms: f64,
    /// 50th percentile (median) in milliseconds
    pub p50_ms: f64,
    /// 95th percentile in milliseconds
    pub p95_ms: f64,
    /// 99th percentile in milliseconds
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Number of samples collected
    pub count: usize,
    /// Average memory usage in bytes
    pub avg_bytes: usize,
    /// Maximum memory usage in bytes
    pub max_bytes: usize,
    /// Minimum memory usage in bytes
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsReport {
    /// Execution time statistics by operation name
    pub execution_stats: HashMap<String, ExecutionStats>,
    /// Memory usage statistics by operation name
    pub memory_stats: HashMap<String, MemoryStats>,
    /// Error counts by error type/operation name
    pub error_counts: HashMap<String, usize>,
    /// Timestamp when this report was generated
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

    // Use the nearest-rank method for percentile calculation
    // For 1-100 values, p50 should be value at index 49 (0-based), which is 50
    let n = sorted_values.len();
    let index = ((percentile / 100.0) * n as f64).ceil() as usize - 1;
    let clamped_index = index.min(n - 1);

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
