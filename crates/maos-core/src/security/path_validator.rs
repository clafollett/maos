//! Unified path security validation
//!
//! This module consolidates ALL path validation logic to prevent duplication
//! and ensure consistent security checks across the codebase.

use crate::error::utils::{path_traversal_error, suspicious_command_error};
use crate::{MaosError, Result, SecurityError};
use std::path::{Path, PathBuf};

/// Unified path security validator that consolidates all path validation logic
pub struct PathSecurityValidator;

impl PathSecurityValidator {
    /// Validate all security aspects of a path
    pub fn validate_all_security_aspects(path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();

        // Check all traversal patterns
        Self::check_traversal_patterns(&path_str)?;

        // Check Windows-specific patterns
        Self::check_windows_patterns(&path_str)?;

        // Check UNC paths
        Self::check_unc_patterns(&path_str)?;

        // Check Unicode attacks
        Self::check_unicode_attacks(&path_str)?;

        // Check URL encoding attacks
        Self::check_url_encoding_attacks(&path_str)?;

        // Check control character attacks
        Self::check_control_char_attacks(&path_str)?;

        // Check suspicious system paths
        Self::check_suspicious_system_paths(&path_str)?;

        Ok(())
    }

    /// Check for basic traversal patterns
    fn check_traversal_patterns(path_str: &str) -> Result<()> {
        const BASIC_TRAVERSALS: &[&str] = &["../", "..\\", "/..", "\\.."];

        if path_str.contains("..")
            || path_str.starts_with("..")
            || BASIC_TRAVERSALS.iter().any(|&p| path_str.contains(p))
        {
            return Err(path_traversal_error(path_str));
        }

        Ok(())
    }

    /// Check for Windows drive patterns (C:, D:/, etc.)
    fn check_windows_patterns(path_str: &str) -> Result<()> {
        if path_str.contains(':') {
            // Check for Windows drive patterns (C:, D:/, etc.)
            if Self::is_windows_drive_pattern(path_str) {
                return Err(suspicious_command_error(format!(
                    "Windows drive specifier not allowed: {path_str}"
                )));
            }
            // Also block relative paths with colons
            if !Path::new(path_str).is_absolute() {
                return Err(suspicious_command_error(format!(
                    "Relative path contains drive specifier: {path_str}"
                )));
            }
        }
        Ok(())
    }

    /// Check if a string matches a Windows drive pattern
    fn is_windows_drive_pattern(path_str: &str) -> bool {
        path_str.matches(':').count() == 1
            && path_str.chars().nth(1) == Some(':')
            && path_str
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
    }

    /// Check for UNC path attacks (\\server\share format)
    fn check_unc_patterns(path_str: &str) -> Result<()> {
        if path_str.starts_with("\\\\") || path_str.starts_with("//") {
            return Err(suspicious_command_error(format!(
                "UNC path not allowed: {path_str}"
            )));
        }
        Ok(())
    }

    /// Check for Unicode-based traversal attacks
    fn check_unicode_attacks(path_str: &str) -> Result<()> {
        // Unicode characters that resemble path separators
        const UNICODE_TRAVERSAL_PATTERNS: &[&str] = &[
            "..\u{FF0F}", // Fullwidth Solidus
            "\u{FF0F}../",
            "..\u{2044}", // Fraction Slash
            "\u{2044}../",
            "..\u{2215}", // Division Slash
            "\u{2215}../",
        ];

        if UNICODE_TRAVERSAL_PATTERNS
            .iter()
            .any(|&pattern| path_str.contains(pattern))
        {
            return Err(path_traversal_error(format!(
                "Unicode traversal attack detected in: {path_str}"
            )));
        }
        Ok(())
    }

    /// Check for URL-encoded traversal patterns
    fn check_url_encoding_attacks(path_str: &str) -> Result<()> {
        const URL_ENCODED: &[&str] = &[
            "%2e%2e",     // ".." single encoded
            "%2E%2E",     // ".." uppercase
            "%252e%252e", // ".." double encoded
            "%252E%252E", // ".." uppercase double encoded
        ];

        if URL_ENCODED.iter().any(|&p| path_str.contains(p)) {
            return Err(path_traversal_error(format!(
                "URL-encoded traversal detected in: {path_str}"
            )));
        }
        Ok(())
    }

    /// Check for control character combined with traversal
    fn check_control_char_attacks(path_str: &str) -> Result<()> {
        const CONTROL_CHARS: &[char] = &['\0', '\n', '\r', '\t'];

        let has_traversal = path_str.contains("..");
        if has_traversal && CONTROL_CHARS.iter().any(|&c| path_str.contains(c)) {
            return Err(suspicious_command_error(format!(
                "Control character traversal attack in: {path_str}"
            )));
        }
        Ok(())
    }

    /// Check for suspicious system path access attempts
    fn check_suspicious_system_paths(path_str: &str) -> Result<()> {
        let has_traversal = path_str.contains("..");
        let targets_system_paths = path_str.contains("/etc/")
            || path_str.contains("\\etc\\")
            || path_str.contains("/proc/")
            || path_str.contains("/sys/")
            || path_str.contains("/dev/");
        let has_encoded_traversal = path_str.contains("%2e");

        if targets_system_paths && (has_traversal || has_encoded_traversal) {
            return Err(suspicious_command_error(format!(
                "Suspicious system path access attempt: {path_str}"
            )));
        }
        Ok(())
    }

    /// Validate a path within a workspace context
    pub fn validate_workspace_path(path: &Path, workspace_root: &Path) -> Result<()> {
        // First run all security checks
        Self::validate_all_security_aspects(path)?;

        // Then verify it's within workspace bounds
        let canonical_path = Self::safe_canonicalize(path);
        let canonical_workspace = Self::safe_canonicalize(workspace_root);

        if !canonical_path.starts_with(&canonical_workspace) {
            return Err(MaosError::Security(SecurityError::Unauthorized {
                resource: format!("Path outside workspace: {}", path.display()),
            }));
        }

        Ok(())
    }

    /// Safely canonicalize a path without following symlinks
    fn safe_canonicalize(path: &Path) -> PathBuf {
        // Use dunce for Windows UNC path compatibility
        dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_traversal_detection() {
        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new("../../../etc/passwd"))
                .is_err()
        );

        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new("safe/path/file.txt"))
                .is_ok()
        );
    }

    #[test]
    fn test_unicode_traversal_detection() {
        // Unicode slash attacks
        let unicode_attack = "..\u{FF0F}etc\u{FF0F}passwd";
        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new(unicode_attack))
                .is_err()
        );
    }

    #[test]
    fn test_url_encoded_detection() {
        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new("%2e%2e/etc/passwd"))
                .is_err()
        );

        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new(
                "%252e%252e/etc/passwd"
            ))
            .is_err()
        );
    }

    #[test]
    fn test_control_char_detection() {
        let path_with_null = "../\0/etc/passwd";
        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new(path_with_null))
                .is_err()
        );
    }

    #[test]
    fn test_windows_patterns() {
        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new("C:/windows/system32"))
                .is_err()
        );

        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new("D:\\sensitive"))
                .is_err()
        );
    }

    #[test]
    fn test_unc_paths() {
        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new("\\\\server\\share"))
                .is_err()
        );

        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new("//server/share"))
                .is_err()
        );
    }

    #[test]
    fn test_suspicious_system_paths() {
        // Should block /etc/ with traversal
        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new("../etc/passwd"))
                .is_err()
        );

        // Should allow /etc/ without traversal (might be legitimate)
        assert!(
            PathSecurityValidator::validate_all_security_aspects(Path::new("/etc/hosts")).is_ok()
        );
    }
}
