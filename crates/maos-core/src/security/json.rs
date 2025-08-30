//! JSON structure validation for DoS protection

use crate::error::{MaosError, Result, SecurityError};

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
/// use maos_core::security::json::validate_json_structure;
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
pub fn validate_json_structure(json_bytes: &[u8], max_depth: u32, max_size: usize) -> Result<()> {
    // Size check
    if json_bytes.len() > max_size {
        return Err(MaosError::Security(SecurityError::PolicyViolation {
            policy: format!("JSON size {} exceeds limit {}", json_bytes.len(), max_size),
        }));
    }

    // Depth check
    let depth = calculate_json_depth(json_bytes)?;
    if depth > max_depth {
        return Err(MaosError::Security(SecurityError::PolicyViolation {
            policy: format!("JSON nesting depth {depth} exceeds maximum {max_depth}"),
        }));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_size_validation() {
        let small_json = br#"{"key": "value"}"#;
        assert!(validate_json_structure(small_json, 10, 100).is_ok());

        let large_json = br#"{"key": "very long value that exceeds size limit"}"#;
        assert!(validate_json_structure(large_json, 10, 20).is_err());
    }

    #[test]
    fn test_json_depth_validation() {
        let shallow_json = br#"{"a": {"b": "value"}}"#;
        assert!(validate_json_structure(shallow_json, 2, 1024).is_ok());

        let deep_json = br#"{"a": {"b": {"c": {"d": "too deep"}}}}"#;
        assert!(validate_json_structure(deep_json, 2, 1024).is_err());
    }

    #[test]
    fn test_json_depth_calculation() {
        let json1 = br#"{"a": "b"}"#;
        assert_eq!(calculate_json_depth(json1).unwrap(), 1);

        let json2 = br#"{"a": {"b": "c"}}"#;
        assert_eq!(calculate_json_depth(json2).unwrap(), 2);

        let json3 = br#"[1, [2, [3, [4]]]]"#;
        assert_eq!(calculate_json_depth(json3).unwrap(), 4);

        // String with braces shouldn't affect depth
        let json4 = br#"{"text": "{ nested brace in string }"}"#;
        assert_eq!(calculate_json_depth(json4).unwrap(), 1);
    }
}
