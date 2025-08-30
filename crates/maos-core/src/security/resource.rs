//! Resource usage validation for DoS protection

use crate::error::{MaosError, Result};

/// Validate resource usage is within limits
///
/// # Security
///
/// Prevents Denial-of-Service attacks by enforcing memory and execution time limits.
/// Critical for preventing resource exhaustion attacks that could make the system
/// unresponsive or crash.
///
/// # Parameters
///
/// * `memory_bytes` - Current memory usage in bytes
/// * `execution_time_ms` - Current execution time in milliseconds  
/// * `memory_limit` - Maximum allowed memory in bytes
/// * `time_limit` - Maximum allowed execution time in milliseconds
///
/// # Examples
///
/// ```rust
/// use maos_core::security::resource::validate_resource_usage;
///
/// // Within limits - accepted
/// assert!(validate_resource_usage(1024, 100, 2048, 1000).is_ok());
///
/// // Exceeds memory limit - rejected
/// assert!(validate_resource_usage(4096, 100, 2048, 1000).is_err());
/// ```
///
/// # Errors
///
/// Returns [`MaosError::ResourceLimit`] if any resource limit is exceeded.
pub fn validate_resource_usage(
    memory_bytes: u64,
    execution_time_ms: u64,
    memory_limit: u64,
    time_limit: u64,
) -> Result<()> {
    if memory_bytes > memory_limit {
        return Err(MaosError::ResourceLimit {
            resource: "memory".to_string(),
            limit: memory_limit,
            actual: memory_bytes,
            message: format!("Memory usage {memory_bytes} exceeds limit {memory_limit}"),
        });
    }

    if execution_time_ms > time_limit {
        return Err(MaosError::ResourceLimit {
            resource: "execution_time".to_string(),
            limit: time_limit,
            actual: execution_time_ms,
            message: format!("Execution time {execution_time_ms}ms exceeds limit {time_limit}ms"),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_limits() {
        // Within limits should pass
        assert!(
            validate_resource_usage(
                512 * crate::constants::BYTES_PER_MB as u64, // 512MB
                1000,                                        // 1 second
                crate::constants::BYTES_PER_GB as u64,       // 1GB limit
                5000                                         // 5 second limit
            )
            .is_ok()
        );

        // Over memory limit should fail
        assert!(
            validate_resource_usage(
                2 * crate::constants::BYTES_PER_GB as u64, // 2GB
                1000,                                      // 1 second
                crate::constants::BYTES_PER_GB as u64,     // 1GB limit
                5000                                       // 5 second limit
            )
            .is_err()
        );
    }

    #[test]
    fn test_execution_time_limits() {
        // Within time limit should pass
        assert!(
            validate_resource_usage(
                512 * crate::constants::BYTES_PER_MB as u64, // 512MB
                3000,                                        // 3 seconds
                crate::constants::BYTES_PER_GB as u64,       // 1GB limit
                5000                                         // 5 second limit
            )
            .is_ok()
        );

        // Over time limit should fail
        assert!(
            validate_resource_usage(
                512 * crate::constants::BYTES_PER_MB as u64, // 512MB
                6000,                                        // 6 seconds
                crate::constants::BYTES_PER_GB as u64,       // 1GB limit
                5000                                         // 5 second limit
            )
            .is_err()
        );
    }
}
