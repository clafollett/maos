//! Tests for PathValidationError enum
//!
//! These tests are written FIRST (RED phase of TDD) to define the behavior
//! we want from our PathValidationError enum before implementing it.

use maos_core::error::{MaosError, PathValidationError};
use std::path::PathBuf;

#[cfg(test)]
mod path_validation_error_tests {
    use super::*;

    #[test]
    fn test_path_traversal_error_display() {
        let error = PathValidationError::PathTraversal {
            path: PathBuf::from("../../../etc/passwd"),
        };
        let display = format!("{}", error);
        assert!(display.contains("Path traversal attempt blocked"));
        assert!(display.contains("../../../etc/passwd"));
    }

    #[test]
    fn test_outside_workspace_error_display() {
        let error = PathValidationError::OutsideWorkspace {
            path: PathBuf::from("/etc/passwd"),
            workspace: PathBuf::from("/workspace"),
        };
        let display = format!("{}", error);
        assert!(display.contains("Path outside workspace"));
        assert!(display.contains("/etc/passwd"));
        assert!(display.contains("/workspace"));
    }

    #[test]
    fn test_blocked_path_error_display() {
        let error = PathValidationError::BlockedPath(PathBuf::from("/workspace/.git/hooks"));
        let display = format!("{}", error);
        assert!(display.contains("Blocked path pattern"));
        assert!(display.contains(".git/hooks"));
    }

    #[test]
    fn test_canonicalization_failed_error_display() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "No such file");
        let error =
            PathValidationError::CanonicalizationFailed(PathBuf::from("/nonexistent"), io_error);
        let display = format!("{}", error);
        assert!(display.contains("Canonicalization failed"));
        assert!(display.contains("/nonexistent"));
    }

    #[test]
    fn test_invalid_workspace_error_display() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "No such directory");
        let error = PathValidationError::InvalidWorkspace(PathBuf::from("/invalid"), io_error);
        let display = format!("{}", error);
        assert!(display.contains("Invalid workspace"));
        assert!(display.contains("/invalid"));
    }

    #[test]
    fn test_invalid_component_error_display() {
        let error = PathValidationError::InvalidComponent("..".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Invalid path component"));
        assert!(display.contains(".."));
    }

    #[test]
    fn test_path_validation_error_converts_to_maos_error() {
        let error = PathValidationError::PathTraversal {
            path: PathBuf::from("../../../etc/passwd"),
        };
        let maos_error: MaosError = error.into();

        // Should convert to MaosError::PathValidation variant
        match maos_error {
            MaosError::PathValidation(_) => (),
            _ => panic!("PathValidationError should convert to MaosError::PathValidation"),
        }
    }
}
