//! Path safety validation
//!
//! Core path validation to prevent traversal attacks and malicious paths

use crate::error::{MaosError, Result, SecurityError};
use std::path::Path;

/// Check if a string matches a Windows drive pattern (e.g., C:, D:/, E:\)
///
/// Returns true for patterns like:
/// - C: (single letter followed by colon)
/// - D:/ (with forward slash)
/// - E:\ (with backslash)
fn is_windows_drive_pattern(path_str: &str) -> bool {
    path_str.matches(':').count() == 1
        && path_str.chars().nth(1) == Some(':')
        && path_str
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
}

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
/// use maos_core::security::path::validate_path_safety;
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
        return Err(MaosError::Security(SecurityError::PathTraversal {
            path: path_str.to_string(),
        }));
    }

    // Check for drive specifier attacks (consistent across all platforms)
    // This prevents both Windows drive attacks and similar colon-based attacks
    if path_str.contains(':') {
        // Check for Windows drive patterns (C:, D:/, etc.) - block these on all platforms for consistency
        if is_windows_drive_pattern(&path_str) {
            return Err(MaosError::Security(SecurityError::SuspiciousCommand {
                command: format!("Windows drive specifier not allowed: {path_str}"),
            }));
        }
        // Also block relative paths with colons (original logic)
        if !path.is_absolute() {
            return Err(MaosError::Security(SecurityError::SuspiciousCommand {
                command: format!("Relative path contains drive specifier: {path_str}"),
            }));
        }
    }

    // Check for UNC path attacks (\\server\share\file format)
    // UNC paths can be used to access network shares or device paths maliciously
    if path_str.starts_with("\\\\") || path_str.starts_with("//") {
        return Err(MaosError::Security(SecurityError::SuspiciousCommand {
            command: format!("UNC path not allowed: {path_str}"),
        }));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_traversal_detection() {
        // Should detect classic traversal attempts
        assert!(validate_path_safety(&PathBuf::from("../../../etc/passwd")).is_err());
        assert!(validate_path_safety(&PathBuf::from("./../../secrets")).is_err());
        assert!(validate_path_safety(&PathBuf::from("data/../../../root")).is_err());

        // Should allow safe paths
        assert!(validate_path_safety(&PathBuf::from("./data/hooks")).is_ok());
        assert!(validate_path_safety(&PathBuf::from("relative/path")).is_ok());
        assert!(validate_path_safety(&PathBuf::from("/absolute/safe/path")).is_ok());
    }

    #[test]
    fn test_drive_specifier_and_unc_attacks() {
        // Drive specifier attacks should be blocked on ALL platforms (consistent security)
        assert!(validate_path_safety(&PathBuf::from("C:/windows/system32")).is_err());
        assert!(validate_path_safety(&PathBuf::from("D:\\sensitive")).is_err());
        assert!(validate_path_safety(&PathBuf::from("E:malicious.exe")).is_err());

        // UNC path attacks should be blocked on ALL platforms
        assert!(validate_path_safety(&PathBuf::from("\\\\server\\share\\file")).is_err());
        assert!(validate_path_safety(&PathBuf::from("//malicious-server/steal-data")).is_err());

        // But legitimate absolute paths should still be allowed
        assert!(validate_path_safety(&PathBuf::from("/absolute/unix/path")).is_ok());
        assert!(validate_path_safety(&PathBuf::from("/usr/local/bin")).is_ok());
    }
}
