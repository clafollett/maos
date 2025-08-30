//! Path safety validation - Legacy compatibility layer
//!
//! This module now delegates to the unified PathSecurityValidator
//! to avoid code duplication.

use super::path_validator::PathSecurityValidator;
use crate::Result;
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
    // Delegate to the unified path security validator
    PathSecurityValidator::validate_all_security_aspects(path)
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
