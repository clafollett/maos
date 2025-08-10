//! Performance metrics collection and reporting system
//!
//! This module provides comprehensive metrics collection for MAOS operations,
//! including execution timing, memory usage tracking, and error counting.
//! All operations are thread-safe using parking_lot::RwLock.
//!
//! # Examples
//!
//! ```no_run
//! use maos_core::metrics::PerformanceMetrics;
//! use maos_core::timed_operation;
//! use std::time::Duration;
//!
//! let metrics = PerformanceMetrics::new();
//!
//! // Manual timing
//! metrics.record_execution_time("my_op", Duration::from_millis(42));
//!
//! // Using the macro for automatic timing
//! let result = timed_operation!(metrics, "complex_op", {
//!     // Your code here
//!     42
//! });
//! ```

pub mod collector;
pub mod report;

// Re-exports
pub use collector::PerformanceMetrics;
pub use report::{ExecutionStats, MemoryStats, MetricsReport};

/// Macro for timing operations automatically
///
/// This macro executes the given code block, measures its execution time,
/// and records it in the provided metrics collector.
///
/// # Arguments
///
/// * `$metrics` - A reference to a PerformanceMetrics instance
/// * `$operation` - The operation name (string or &str)
/// * `$code` - The code block to execute and time
///
/// # Returns
///
/// The result of executing the code block
///
/// # Example
///
/// ```no_run
/// use maos_core::metrics::PerformanceMetrics;
/// use maos_core::timed_operation;
/// use std::thread;
/// use std::time::Duration;
///
/// let metrics = PerformanceMetrics::new();
/// let result = timed_operation!(metrics, "sleep_test", {
///     thread::sleep(Duration::from_millis(10));
///     "done"
/// });
/// assert_eq!(result, "done");
/// ```
#[macro_export]
macro_rules! timed_operation {
    ($metrics:expr, $operation:expr, $code:block) => {{
        let start = std::time::Instant::now();
        let result = $code;
        let duration = start.elapsed();
        $metrics.record_execution_time($operation, duration);
        result
    }};
}
