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
//! let config_dir = PathBuf::from(MAOS_ROOT_DIR);
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

/// MAOS root directory name within a project
pub const MAOS_ROOT_DIR: &str = ".maos";

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
pub const MAX_EXECUTION_TIME_MS: u64 = 10;

/// Maximum memory usage target in megabytes
pub const MAX_MEMORY_USAGE_MB: usize = 5;

/// Maximum binary size target in megabytes
pub const MAX_BINARY_SIZE_MB: usize = 10;

// =============================================================================
// Timeout Constants
// =============================================================================

/// Default timeout for general operations (5 seconds)
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
pub const LOG_FILE_PATTERN: &str = "session-{session_id}.log";

/// Maximum size per log file before rotation (10MB)
pub const MAX_LOG_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of rolled log files to keep per session
pub const MAX_LOG_FILES_PER_SESSION: usize = 10;

// =============================================================================
// Additional System Constants
// =============================================================================

/// Default number of worker threads for parallel operations
pub const DEFAULT_WORKER_THREADS: usize = 4;

/// Maximum number of concurrent agents per session
pub const MAX_CONCURRENT_AGENTS: usize = 10;

/// Default buffer size for I/O operations (64KB)
pub const DEFAULT_BUFFER_SIZE: usize = 64 * 1024;

/// Maximum retries for transient failures
pub const MAX_RETRY_ATTEMPTS: u32 = 3;

/// Delay between retry attempts
pub const RETRY_DELAY: Duration = Duration::from_millis(100);
