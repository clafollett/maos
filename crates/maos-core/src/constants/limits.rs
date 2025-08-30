//! Limit and timeout constants to eliminate magic numbers
//!
//! This module centralizes all timeout values, depth limits, and other
//! numeric limits to prevent duplication throughout the codebase.

/// Default operation timeout in milliseconds
pub const DEFAULT_TIMEOUT_MS: u64 = 5000;

/// Stdin read timeout in milliseconds
pub const STDIN_TIMEOUT_MS: u64 = 500;

/// File lock timeout in milliseconds
pub const FILE_LOCK_TIMEOUT_MS: u64 = 1000;

/// Validation timeout in milliseconds (for tests)
pub const VALIDATION_TIMEOUT_MS: u64 = 1000;

/// Default JSON depth limit for production
pub const JSON_DEPTH_DEFAULT: u32 = 64;

/// JSON depth limit for tests (more restrictive)
pub const JSON_DEPTH_TEST: u32 = 10;

/// Maximum path length in characters
pub const MAX_PATH_LENGTH: usize = 4096;

/// Default number of test iterations
pub const DEFAULT_TEST_ITERATIONS: usize = 100;

/// Default speech rate in words per minute
pub const DEFAULT_SPEECH_RATE_WPM: u32 = 190;

/// Maximum number of agents (default)
pub const MAX_AGENTS_DEFAULT: usize = 20;

/// Maximum number of worktrees (default)
pub const MAX_WORKTREES_DEFAULT: usize = 50;

/// Performance test iteration count
pub const PERF_TEST_ITERATIONS: usize = 1000;

/// Thread pool size for concurrent tests
pub const TEST_THREAD_POOL_SIZE: usize = 10;

/// Retry attempts for flaky operations
pub const MAX_RETRY_ATTEMPTS: usize = 3;

/// Rate limiting
pub mod rate_limits {
    /// Maximum requests per second
    pub const MAX_REQUESTS_PER_SECOND: usize = 100;

    /// Burst allowance
    pub const BURST_ALLOWANCE: usize = 10;
}

/// Test-specific limits
pub mod test_limits {

    /// Quick test timeout (for unit tests)
    pub const QUICK_TEST_TIMEOUT_MS: u64 = 100;

    /// Integration test timeout
    pub const INTEGRATION_TEST_TIMEOUT_MS: u64 = 10_000;

    /// Stress test iterations
    pub const STRESS_TEST_ITERATIONS: usize = 10_000;
}
