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
    ///
    /// # Security
    ///
    /// Prevents "../../../etc/passwd" style directory traversal attacks by rejecting
    /// paths containing ".." sequences and suspicious drive specifiers on Windows.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    /// use maos::security::validators::validate_path_safety;
    ///
    /// // Safe path - accepted
    /// assert!(validate_path_safety(Path::new("safe/path/file.txt")).is_ok());
    ///
    /// // Traversal attack - rejected
    /// assert!(validate_path_safety(Path::new("../../../etc/passwd")).is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`MaosError::Security`] if path traversal patterns are detected.
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

        // Check for drive specifier attacks (consistent across all platforms)
        // This prevents both Windows drive attacks and similar colon-based attacks
        if path_str.contains(':') {
            // Check for Windows drive patterns (C:, D:/, etc.) - block these on all platforms for consistency
            if path_str.matches(':').count() == 1
                && path_str.chars().nth(1) == Some(':')
                && path_str
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic())
            {
                return Err(MaosError::Security(
                    maos_core::error::SecurityError::SuspiciousCommand {
                        command: format!("Windows drive specifier not allowed: {path_str}"),
                    },
                ));
            }
            // Also block relative paths with colons (original logic)
            if !path.is_absolute() {
                return Err(MaosError::Security(
                    maos_core::error::SecurityError::SuspiciousCommand {
                        command: format!("Relative path contains drive specifier: {path_str}"),
                    },
                ));
            }
        }

        // Check for UNC path attacks (\\server\share\file format)
        // UNC paths can be used to access network shares or device paths maliciously
        if path_str.starts_with("\\\\") || path_str.starts_with("//") {
            return Err(MaosError::Security(
                maos_core::error::SecurityError::SuspiciousCommand {
                    command: format!("UNC path not allowed: {path_str}"),
                },
            ));
        }

        Ok(())
    }

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
    /// use maos::security::validators::validate_resource_usage;
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
                message: format!(
                    "Execution time {execution_time_ms}ms exceeds limit {time_limit}ms"
                ),
            });
        }

        Ok(())
    }

    /// Validate JSON structure for DoS protection
    ///
    /// # Security
    ///
    /// Prevents JSON bomb attacks by enforcing size and depth limits. Protects against:
    /// - Large payloads causing memory exhaustion
    /// - Deeply nested structures causing stack overflow
    /// - Malformed JSON causing parser abuse
    ///
    /// # Parameters
    ///
    /// * `json_bytes` - Raw JSON bytes to validate
    /// * `max_depth` - Maximum allowed nesting depth (typically 10-100)
    /// * `max_size` - Maximum allowed size in bytes (typically 1-10MB)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use maos::security::validators::validate_json_structure;
    ///
    /// let safe_json = br#"{"level1": {"level2": "value"}}"#;
    /// assert!(validate_json_structure(safe_json, 5, 1024).is_ok());
    ///
    /// let deep_json = br#"{"a":{"b":{"c":{"d":{"e":"deep"}}}}}"#;
    /// assert!(validate_json_structure(deep_json, 3, 1024).is_err()); // Too deep
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`MaosError::Security`] if JSON structure violates safety policies.
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
                    policy: format!("JSON depth {depth} exceeds limit {max_depth}"),
                },
            ));
        }

        Ok(())
    }

    /// Calculate JSON nesting depth efficiently
    ///
    /// Fast single-pass parser that tracks brace/bracket nesting without full JSON parsing.
    /// Properly handles string escaping to avoid false positives from quoted braces.
    ///
    /// # Implementation Notes
    ///
    /// - Tracks `{` and `[` for depth increases
    /// - Tracks `}` and `]` for depth decreases  
    /// - Ignores braces/brackets inside quoted strings
    /// - Handles escape sequences properly (`\"`, `\\`, etc.)
    ///
    /// # Returns
    ///
    /// Maximum nesting depth found in the JSON structure.
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
