//! Size-related constants to eliminate magic numbers
//!
//! This module centralizes all size calculations and memory constants
//! to prevent duplication and magic numbers throughout the codebase.

/// Number of bytes in a kilobyte
pub const BYTES_PER_KB: usize = 1024;

/// Number of bytes in a megabyte
pub const BYTES_PER_MB: usize = 1024 * 1024;

/// Number of bytes in a gigabyte  
pub const BYTES_PER_GB: usize = 1024 * 1024 * 1024;

/// Default buffer size in kilobytes
pub const DEFAULT_BUFFER_KB: usize = 8;

/// Default buffer size in bytes
pub const DEFAULT_BUFFER_SIZE: usize = DEFAULT_BUFFER_KB * BYTES_PER_KB;

/// Memory growth warning threshold in megabytes
pub const MEMORY_WARNING_THRESHOLD_MB: usize = 50;

/// Memory growth warning threshold in bytes
pub const MEMORY_WARNING_THRESHOLD: usize = MEMORY_WARNING_THRESHOLD_MB * BYTES_PER_MB;

/// Maximum input size in megabytes (default)
pub const MAX_INPUT_SIZE_MB: usize = 10;

/// Maximum input size in bytes (default)
pub const MAX_INPUT_SIZE: usize = MAX_INPUT_SIZE_MB * BYTES_PER_MB;

/// Maximum log file size in megabytes
pub const MAX_LOG_FILE_SIZE_MB: usize = 10;

/// Maximum log file size in bytes
pub const MAX_LOG_FILE_SIZE: usize = MAX_LOG_FILE_SIZE_MB * BYTES_PER_MB;

/// Warning threshold for input size (percentage of max)
pub const INPUT_SIZE_WARNING_PERCENT: usize = 50;

/// Test-specific memory limits
pub mod test_sizes {
    use super::*;

    /// Small test size (1MB)
    pub const SMALL_TEST_SIZE: usize = BYTES_PER_MB;

    /// Medium test size (5MB)
    pub const MEDIUM_TEST_SIZE: usize = 5 * BYTES_PER_MB;

    /// Large test size (10MB)
    pub const LARGE_TEST_SIZE: usize = 10 * BYTES_PER_MB;

    /// Attack simulation size (for DoS tests)
    pub const ATTACK_SIZE: usize = 2 * MAX_INPUT_SIZE;
}
