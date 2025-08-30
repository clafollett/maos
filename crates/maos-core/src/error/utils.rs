//! Error utility functions to reduce duplication
//!
//! This module provides helper functions for creating common error types,
//! reducing boilerplate and ensuring consistent error messages.

use super::{MaosError, SecurityError};

/// Create a path traversal security error
pub fn path_traversal_error(path: impl Into<String>) -> MaosError {
    MaosError::Security(SecurityError::PathTraversal { path: path.into() })
}

/// Create a suspicious command security error
pub fn suspicious_command_error(command: impl Into<String>) -> MaosError {
    MaosError::Security(SecurityError::SuspiciousCommand {
        command: command.into(),
    })
}

/// Create a policy violation security error
pub fn policy_violation_error(policy: impl Into<String>) -> MaosError {
    MaosError::Security(SecurityError::PolicyViolation {
        policy: policy.into(),
    })
}

/// Create an unauthorized security error
pub fn unauthorized_error(resource: impl Into<String>) -> MaosError {
    MaosError::Security(SecurityError::Unauthorized {
        resource: resource.into(),
    })
}

/// Helper to create security errors with consistent formatting
pub mod messages {
    /// Standard message for blocked environment file access
    pub const ENV_FILE_BLOCKED: &str = "Access to environment files is restricted for security";

    /// Standard message for blocked credential file access
    pub const CREDENTIALS_BLOCKED: &str = "Access to credential files is restricted";

    /// Standard message for blocked SSH key access
    pub const SSH_KEY_BLOCKED: &str = "Access to SSH keys is restricted";

    /// Standard message for path traversal attempts
    pub const PATH_TRAVERSAL_DETECTED: &str = "Path traversal attempt detected";

    /// Standard message for dangerous command patterns
    pub const DANGEROUS_COMMAND: &str = "Command contains dangerous patterns";

    /// Standard message for excessive JSON depth
    pub const JSON_DEPTH_EXCEEDED: &str = "JSON structure exceeds maximum allowed depth";

    /// Standard message for excessive JSON size
    pub const JSON_SIZE_EXCEEDED: &str = "JSON structure exceeds maximum allowed size";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation_helpers() {
        let err = path_traversal_error("../etc/passwd");
        match err {
            MaosError::Security(SecurityError::PathTraversal { path }) => {
                assert_eq!(path, "../etc/passwd");
            }
            _ => panic!("Wrong error type"),
        }

        let err = suspicious_command_error("rm -rf /");
        match err {
            MaosError::Security(SecurityError::SuspiciousCommand { command }) => {
                assert_eq!(command, "rm -rf /");
            }
            _ => panic!("Wrong error type"),
        }

        let err = policy_violation_error("Test violation");
        match err {
            MaosError::Security(SecurityError::PolicyViolation { policy }) => {
                assert_eq!(policy, "Test violation");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_standard_messages() {
        assert!(messages::ENV_FILE_BLOCKED.contains("environment"));
        assert!(messages::CREDENTIALS_BLOCKED.contains("credential"));
        assert!(messages::SSH_KEY_BLOCKED.contains("SSH"));
        assert!(messages::PATH_TRAVERSAL_DETECTED.contains("traversal"));
        assert!(messages::DANGEROUS_COMMAND.contains("dangerous"));
        assert!(messages::JSON_DEPTH_EXCEEDED.contains("depth"));
        assert!(messages::JSON_SIZE_EXCEEDED.contains("size"));
    }
}
