//! 🔒 MAOS Security Test Suite
//!
//! Comprehensive validation of all security enhancements implemented for Issue #56.
//! This module provides centralized security testing across all components.

#[cfg(test)]
mod tests;

/// Security validation utilities
pub mod validators {
    use maos_core::{MaosError, Result};
    use std::path::Path;

    /// Validate that a path doesn't contain traversal attempts
    pub fn validate_path_safety(path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();

        // Check for path traversal patterns
        if path_str.contains("..") {
            return Err(MaosError::Security(
                maos_core::error::SecurityError::PathTraversal {
                    path: path_str.to_string(),
                },
            ));
        }

        // Check for absolute path attempts from relative contexts
        if cfg!(windows) && path_str.contains(':') && !path.is_absolute() {
            return Err(MaosError::Security(
                maos_core::error::SecurityError::SuspiciousCommand {
                    command: format!("Relative path contains drive specifier: {}", path_str),
                },
            ));
        }

        Ok(())
    }

    /// Validate resource usage is within limits
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
                message: format!(
                    "Memory usage {} exceeds limit {}",
                    memory_bytes, memory_limit
                ),
            });
        }

        if execution_time_ms > time_limit {
            return Err(MaosError::ResourceLimit {
                resource: "execution_time".to_string(),
                limit: time_limit,
                actual: execution_time_ms,
                message: format!(
                    "Execution time {}ms exceeds limit {}ms",
                    execution_time_ms, time_limit
                ),
            });
        }

        Ok(())
    }

    /// Validate JSON structure for DoS protection
    pub fn validate_json_structure(
        json_bytes: &[u8],
        max_depth: u32,
        max_size: usize,
    ) -> Result<()> {
        // Size check
        if json_bytes.len() > max_size {
            return Err(MaosError::Security(
                maos_core::error::SecurityError::PolicyViolation {
                    policy: format!("JSON size {} exceeds limit {}", json_bytes.len(), max_size),
                },
            ));
        }

        // Depth check
        let depth = calculate_json_depth(json_bytes)?;
        if depth > max_depth {
            return Err(MaosError::Security(
                maos_core::error::SecurityError::PolicyViolation {
                    policy: format!("JSON depth {} exceeds limit {}", depth, max_depth),
                },
            ));
        }

        Ok(())
    }

    /// Calculate JSON nesting depth
    fn calculate_json_depth(json_bytes: &[u8]) -> Result<u32> {
        let mut depth: u32 = 0;
        let mut max_depth: u32 = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for &byte in json_bytes {
            if escape_next {
                escape_next = false;
                continue;
            }

            match byte {
                b'"' if !escape_next => in_string = !in_string,
                b'\\' if in_string => escape_next = true,
                b'{' | b'[' if !in_string => {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                }
                b'}' | b']' if !in_string => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        Ok(max_depth)
    }
}
