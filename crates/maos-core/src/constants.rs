//! System-wide constants for MAOS
//!
//! This module centralizes all constant values used throughout the MAOS system,
//! providing a single source of truth for configuration values, timeouts,
//! performance targets, and naming conventions.
//!
//! # Design Philosophy
//!
//! Constants are centralized in this module to:
//! - Ensure consistency across the entire codebase
//! - Make system-wide configuration changes easier
//! - Provide clear documentation of system limits and defaults
//! - Enable compile-time optimization of frequently used values
//! - Reduce magic numbers scattered throughout the code
//!
//! All constants are organized into logical groups with clear relationships
//! between related values. Duration constants use const expressions to ensure
//! compile-time evaluation and zero runtime cost.
//!
//! # Examples
//!
//! ## Directory Structure Setup
//!
//! ```
//! use maos_core::constants::*;
//! use std::path::PathBuf;
//!
//! // Build standard MAOS directory structure
//! let config_dir = PathBuf::from(DEFAULT_CONFIG_DIR);
//! let sessions_dir = config_dir.join(SESSIONS_DIR_NAME);
//! let workspace_dir = config_dir.join(WORKSPACES_DIR_NAME);
//!
//! println!("Config: {}", config_dir.display());
//! println!("Sessions: {}", sessions_dir.display());
//! ```
//!
//! ## Using Timeout Constants
//!
//! ```no_run
//! use maos_core::constants::*;
//! use std::time::Duration;
//!
//! // All timeout constants are const Duration values
//! async fn with_timeout<F, T>(operation: F) -> Result<T, String>
//! where
//!     F: std::future::Future<Output = T>,
//! {
//!     // Use with tokio::time::timeout or similar async runtime
//!     todo!("Implement with async runtime")
//! }
//! ```
//!
//! ## Performance Target Validation
//!
//! ```
//! use maos_core::constants::*;
//!
//! fn validate_performance(execution_ms: u64, memory_mb: usize) -> bool {
//!     execution_ms <= MAX_EXECUTION_TIME_MS &&
//!     memory_mb <= MAX_MEMORY_USAGE_MB
//! }
//!
//! assert!(validate_performance(5, 3)); // Within limits
//! assert!(!validate_performance(15, 3)); // Too slow
//! ```

use std::time::Duration;

// =============================================================================
// Directory Structure Constants
// =============================================================================

/// Default configuration directory: ~/.maos
///
/// This is the root directory for all MAOS data, located in the user's home directory.
/// All sessions, workspaces, and configuration files are stored here.
///
/// # Example
///
/// ```rust
/// use maos_core::constants::DEFAULT_CONFIG_DIR;
/// use std::path::PathBuf;
///
/// let home = PathBuf::from("/home/username");
/// let maos_root = home.join(DEFAULT_CONFIG_DIR);
/// assert_eq!(maos_root, PathBuf::from("/home/username/.maos"));
/// ```
pub const DEFAULT_CONFIG_DIR: &str = ".maos";

/// Default configuration file name
pub const CONFIG_FILE_NAME: &str = "config.json";

/// Default session directory name
pub const SESSIONS_DIR_NAME: &str = "sessions";

/// Default workspace directory name
pub const WORKSPACES_DIR_NAME: &str = "workspaces";

/// Default logs directory name within session
pub const LOGS_DIR_NAME: &str = "logs";

// =============================================================================
// Performance Target Constants
// =============================================================================

/// Maximum execution time target in milliseconds
///
/// Operations exceeding this threshold should be logged as performance warnings.
/// The aggressive 10ms target ensures MAOS remains responsive for interactive use.
///
/// # Rationale
///
/// 10ms is well below the human perception threshold (~100ms), ensuring
/// MAOS operations feel instantaneous to users. This is especially critical
/// for hook operations that run synchronously.
///
/// # Example
///
/// ```rust
/// use maos_core::constants::MAX_EXECUTION_TIME_MS;
/// use std::time::Instant;
///
/// // Stub function for demonstration
/// fn perform_operation() {
///     // Simulate some work
///     std::thread::sleep(std::time::Duration::from_millis(5));
/// }
///
/// let start = Instant::now();
/// perform_operation();
/// let elapsed_ms = start.elapsed().as_millis() as u64;
///
/// if elapsed_ms > MAX_EXECUTION_TIME_MS {
///     println!(
///         "Operation exceeded target: {}ms > {}ms",
///         elapsed_ms,
///         MAX_EXECUTION_TIME_MS
///     );
/// }
/// ```
pub const MAX_EXECUTION_TIME_MS: u64 = 10;

/// Maximum memory usage target in megabytes
pub const MAX_MEMORY_USAGE_MB: usize = 5;

/// Maximum binary size target in megabytes
pub const MAX_BINARY_SIZE_MB: usize = 10;

// =============================================================================
// Timeout Constants
// =============================================================================

/// Default timeout for general operations (5 seconds)
///
/// Most MAOS operations should complete within this timeout.
/// Operations that may take longer (like TTS) should use specialized timeouts.
///
/// # Usage
///
/// This is a const Duration, usable in const contexts and with async timeouts.
///
/// # Example
///
/// ```no_run
/// use maos_core::constants::DEFAULT_OPERATION_TIMEOUT;
/// use std::time::Duration;
///
/// // Example showing how the timeout constant would be used
/// // (requires async runtime like tokio in practice)
/// #[derive(Debug)]
/// enum Error {
///     OperationTimeout,
/// }
///
/// struct Data;
///
/// async fn fetch_data() -> Result<Data, Error> {
///     Ok(Data)
/// }
///
/// async fn fetch_with_timeout() -> Result<Data, Error> {
///     // In practice, would use tokio::time::timeout
///     // timeout(DEFAULT_OPERATION_TIMEOUT, fetch_data()).await
///     //     .map_err(|_| Error::OperationTimeout)?
///     fetch_data().await
/// }
/// ```
pub const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_millis(5000);

/// Timeout for acquiring file locks
pub const FILE_LOCK_TIMEOUT: Duration = Duration::from_millis(1000);

/// Timeout for Text-to-Speech operations
pub const TTS_TIMEOUT: Duration = Duration::from_millis(10000);

// =============================================================================
// File Naming Patterns
// =============================================================================

/// Session metadata file name
pub const SESSION_FILE_NAME: &str = "session.json";

/// Agents registry file name
pub const AGENTS_FILE_NAME: &str = "agents.json";

/// File locks registry file name
pub const LOCKS_FILE_NAME: &str = "locks.json";

/// Progress tracking file name
pub const PROGRESS_FILE_NAME: &str = "progress.json";

/// Timeline events file name
pub const TIMELINE_FILE_NAME: &str = "timeline.json";

/// Metrics collection file name
pub const METRICS_FILE_NAME: &str = "metrics.json";

// =============================================================================
// Logging Constants
// =============================================================================

/// Log file name pattern for session logs
///
/// Uses `{session_id}` as a placeholder for the session identifier.
/// This pattern ensures each session has isolated log files.
///
/// # Format
///
/// - Base: `session-{session_id}.log`
/// - Rotated: `session-{session_id}.log.1`, `.2`, etc.
/// - Compressed: `session-{session_id}.log.1.gz`
///
/// # Example
///
/// ```rust
/// use maos_core::constants::LOG_FILE_PATTERN;
/// use maos_core::SessionId;
///
/// let session_id = SessionId::generate();
/// let log_file = LOG_FILE_PATTERN.replace("{session_id}", session_id.as_str());
/// // Results in: session-sess_abc123def456.log
///
/// // For rotated logs
/// let rotated = format!("{}.1", log_file);
/// let compressed = format!("{}.gz", rotated);
/// ```
pub const LOG_FILE_PATTERN: &str = "session-{session_id}.log";

/// Maximum size per log file before rotation (10MB)
///
/// When a log file reaches this size, it's rotated to a numbered backup.
/// This prevents unbounded log growth while preserving session history.
///
/// # Rationale
///
/// 10MB provides a good balance:
/// - Large enough to capture significant session activity
/// - Small enough to be easily transmitted/analyzed
/// - Can be compressed to ~1MB with gzip
///
/// # Example
///
/// ```rust
/// use maos_core::constants::MAX_LOG_FILE_SIZE;
/// use std::fs::metadata;
/// use std::io;
///
/// // Stub function for demonstration
/// fn rotate_log_file(path: &str) -> io::Result<()> {
///     println!("Rotating log file: {}", path);
///     Ok(())
/// }
///
/// fn check_log_size() -> io::Result<()> {
///     let log_file = "session.log";
///     // In practice, this would check an actual file
///     let size = 15 * 1024 * 1024; // Simulate 15MB file
///
///     if size >= MAX_LOG_FILE_SIZE {
///         // Trigger log rotation
///         rotate_log_file(log_file)?;
///     }
///     Ok(())
/// }
///
/// # check_log_size().ok();
/// ```
pub const MAX_LOG_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of rolled log files to keep per session
pub const MAX_LOG_FILES_PER_SESSION: usize = 10;

// =============================================================================
// Additional System Constants
// =============================================================================

/// Default number of worker threads for parallel operations
///
/// Controls the default parallelism level for multi-agent orchestration.
/// Can be overridden via configuration for systems with different core counts.
///
/// # Rationale
///
/// 4 threads provides good parallelism on modern systems (4+ cores)
/// while avoiding excessive context switching on lower-end hardware.
///
/// # Example
///
/// ```no_run
/// use maos_core::constants::DEFAULT_WORKER_THREADS;
///
/// // Example showing how the worker thread constant would be used
/// // (requires tokio runtime in practice)
/// fn configure_runtime() -> Result<(), Box<dyn std::error::Error>> {
///     println!("Configuring runtime with {} worker threads", DEFAULT_WORKER_THREADS);
///     
///     // In practice, would use tokio::runtime::Builder:
///     // let runtime = tokio::runtime::Builder::new_multi_thread()
///     //     .worker_threads(DEFAULT_WORKER_THREADS)
///     //     .enable_all()
///     //     .build()?;
///     
///     Ok(())
/// }
///
/// # configure_runtime().ok();
/// ```
pub const DEFAULT_WORKER_THREADS: usize = 4;

/// Maximum number of concurrent agents per session
///
/// Limits the number of Claude sub-agents that can run simultaneously
/// within a single MAOS session to prevent resource exhaustion.
///
/// # Rationale
///
/// 10 concurrent agents allows substantial parallelism while:
/// - Preventing API rate limit issues
/// - Avoiding excessive memory usage
/// - Maintaining manageable coordination complexity
///
/// # Example
///
/// ```no_run
/// use maos_core::constants::MAX_CONCURRENT_AGENTS;
/// use std::sync::Arc;
///
/// // Example showing how the agent limit would be enforced
/// // (requires async runtime and tokio semaphore in practice)
///
/// async fn manage_agents() -> Result<(), Box<dyn std::error::Error>> {
///     println!("Managing up to {} concurrent agents", MAX_CONCURRENT_AGENTS);
///     
///     // In practice, would use tokio::sync::Semaphore:
///     // let agent_limiter = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_AGENTS));
///     // let permit = agent_limiter.acquire().await?;
///     // spawn_agent().await?;
///     // drop(permit);
///     
///     Ok(())
/// }
/// ```
pub const MAX_CONCURRENT_AGENTS: usize = 10;

/// Default buffer size for I/O operations (64KB)
pub const DEFAULT_BUFFER_SIZE: usize = 64 * 1024;

/// Maximum retries for transient failures
pub const MAX_RETRY_ATTEMPTS: u32 = 3;

/// Delay between retry attempts
pub const RETRY_DELAY: Duration = Duration::from_millis(100);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_constants_compile_time() {
        // Verify Duration constants are const-evaluable
        const _TEST_TIMEOUT: Duration = DEFAULT_OPERATION_TIMEOUT;
        const _TEST_LOCK: Duration = FILE_LOCK_TIMEOUT;
        const _TEST_TTS: Duration = TTS_TIMEOUT;
    }

    #[test]
    fn test_string_constants_compile_time() {
        // Verify string constants are const-evaluable
        const _TEST_DIR: &str = DEFAULT_CONFIG_DIR;
        const _TEST_FILE: &str = SESSION_FILE_NAME;
        const _TEST_PATTERN: &str = LOG_FILE_PATTERN;
    }

    #[test]
    fn test_numeric_constants_compile_time() {
        // Verify numeric constants are const-evaluable
        const _TEST_MS: u64 = MAX_EXECUTION_TIME_MS;
        const _TEST_MB: usize = MAX_MEMORY_USAGE_MB;
        const _TEST_SIZE: usize = MAX_LOG_FILE_SIZE;
    }
}
