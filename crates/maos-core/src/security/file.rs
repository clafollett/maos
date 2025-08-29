//! Protected file access validation
//!
//! Prevents access to sensitive files like .env containing secrets

use crate::error::{MaosError, Result, SecurityError};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Protected file patterns that should be blocked
static PROTECTED_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Environment files (but not examples or templates)
        Regex::new(r"\.env$").unwrap(),
        Regex::new(r"\.env\.local$").unwrap(),
        Regex::new(r"\.env\.production$").unwrap(),
        Regex::new(r"\.env\.staging$").unwrap(),
        Regex::new(r"\.env\.development$").unwrap(),
        Regex::new(r"\.env\.test$").unwrap(),
        // Key files
        Regex::new(r".*\.key$").unwrap(),
        Regex::new(r".*\.pem$").unwrap(),
        Regex::new(r".*\.p12$").unwrap(),
        Regex::new(r".*\.pfx$").unwrap(),
        // Config files with secrets
        Regex::new(r"config/secrets\.yml$").unwrap(),
        Regex::new(r".*\.credentials$").unwrap(),
        // SSH keys
        Regex::new(r"id_rsa$").unwrap(),
        Regex::new(r"id_dsa$").unwrap(),
        Regex::new(r"id_ecdsa$").unwrap(),
        Regex::new(r"id_ed25519$").unwrap(),
    ]
});

/// Allowed file patterns (exceptions to protected files)
static ALLOWED_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"\.env\.example$").unwrap(),
        Regex::new(r"\.env\.sample$").unwrap(),
        Regex::new(r"\.env\.template$").unwrap(),
        Regex::new(r"stack\.env$").unwrap(), // MAOS-specific
    ]
});

/// Validate file access against protection rules
///
/// # Security
///
/// Prevents access to files containing sensitive data like API keys, passwords,
/// and other secrets. Allows access to template/example files.
///
/// # Parameters
///
/// * `file_path` - Path to the file being accessed
/// * `tool_name` - Name of the tool attempting access (for error messages)
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use maos_core::security::file::validate_file_access;
///
/// // Protected file - blocked
/// assert!(validate_file_access(Path::new(".env"), "Read").is_err());
///
/// // Template file - allowed
/// assert!(validate_file_access(Path::new(".env.example"), "Read").is_ok());
/// ```
///
/// # Errors
///
/// Returns [`MaosError::Security`] if file access should be blocked.
pub fn validate_file_access(file_path: &Path, tool_name: &str) -> Result<()> {
    let path_str = file_path.to_string_lossy();

    // Check if file is explicitly allowed
    for allowed in ALLOWED_PATTERNS.iter() {
        if allowed.is_match(&path_str) {
            return Ok(());
        }
    }

    // Check if file is protected
    for protected in PROTECTED_PATTERNS.iter() {
        if protected.is_match(&path_str) {
            return Err(MaosError::Security(SecurityError::PolicyViolation {
                policy: format!(
                    "{} access to protected file '{}' is blocked to prevent exposure of secrets",
                    tool_name,
                    file_path.display()
                ),
            }));
        }
    }

    Ok(())
}

/// Check if a path refers to an environment file
pub fn is_env_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // Check if it's an allowed exception first
    for allowed in ALLOWED_PATTERNS.iter() {
        if allowed.is_match(&path_str) {
            return false; // It's allowed, not a protected env file
        }
    }

    // Check if it matches .env patterns
    path_str.contains(".env") && !path_str.ends_with(".example") && !path_str.ends_with(".sample")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_protected_files_blocked() {
        let protected = vec![
            ".env",
            ".env.production",
            ".env.local",
            "private.key",
            "cert.pem",
            "config/secrets.yml",
            "id_rsa",
            "server.credentials",
        ];

        for file in protected {
            assert!(
                validate_file_access(&PathBuf::from(file), "Read").is_err(),
                "File '{file}' should be protected"
            );
        }
    }

    #[test]
    fn test_allowed_files() {
        let allowed = vec![
            ".env.example",
            ".env.sample",
            ".env.template",
            "stack.env",
            "readme.md",
            "config.json",
            "data.txt",
        ];

        for file in allowed {
            assert!(
                validate_file_access(&PathBuf::from(file), "Read").is_ok(),
                "File '{file}' should be allowed"
            );
        }
    }

    #[test]
    fn test_is_env_file() {
        assert!(is_env_file(&PathBuf::from(".env")));
        assert!(is_env_file(&PathBuf::from(".env.production")));

        assert!(!is_env_file(&PathBuf::from(".env.example")));
        assert!(!is_env_file(&PathBuf::from("stack.env")));
        assert!(!is_env_file(&PathBuf::from("config.json")));
    }
}
