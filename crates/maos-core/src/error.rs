//! Error handling for MAOS
//!
//! This module provides comprehensive error types with thiserror,
//! exit code mappings, and error context utilities.
//!
//! Implemented for issue #39: foundational error types and exit code mapping.
//! Hook test: automatic cargo fmt and clippy integration.

use thiserror::Error;

/// Convenient result alias for MAOS operations.
///
/// This is the primary `Result` used across MAOS crates.
///
/// # Examples
///
/// ```
/// use maos_core::error::{Result, MaosError};
///
/// fn do_work(ok: bool) -> Result<()> {
///     if ok { Ok(()) } else { Err(MaosError::InvalidInput { message: "bad".into() }) }
/// }
///
/// assert!(do_work(true).is_ok());
/// assert!(do_work(false).is_err());
/// ```
pub type Result<T> = std::result::Result<T, MaosError>;
/// Alias identical to [`Result<T>`] for readability in some contexts.
pub type MaosResult<T> = Result<T>;
/// Result specialized for configuration-related operations.
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
/// Result specialized for validation operations.
pub type ValidationResult<T> = std::result::Result<T, ValidationError>;

/// Root error type for all MAOS operations.
///
/// This error type provides consistent, actionable messages and integrates with
/// standard exit code mapping via [`ExitCode`].
///
/// Variants cover configuration, sessions, security, filesystem, git, JSON/IO
/// processing, input validation, timeouts, explicit blocking, and contextual
/// error wrapping.
///
/// # Exit Code Mapping
///
/// - `Anyhow` variants map to `InternalError` (99) indicating unexpected failures
/// - `Context` variants preserve the underlying `MaosError`'s exit code when possible
/// - Non-`MaosError` sources in `Context` default to `GeneralError` (1)
#[derive(Error, Debug)]
pub enum MaosError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Session error: {0}")]
    Session(#[from] SessionError),

    #[error("Security validation failed: {0}")]
    Security(#[from] SecurityError),

    #[error("File system error: {0}")]
    FileSystem(#[from] FileSystemError),

    #[error("Git operation failed: {0}")]
    Git(#[from] GitError),

    #[error("JSON processing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Resource limit exceeded: {resource} limit={limit}, actual={actual} - {message}")]
    ResourceLimit {
        resource: String,
        limit: u64,
        actual: u64,
        message: String,
    },

    #[error("Operation timeout: {operation} took longer than {timeout_ms}ms")]
    Timeout { operation: String, timeout_ms: u64 },

    #[error("Blocking error: {reason}")]
    Blocking { reason: String },

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Path validation failed: {0}")]
    PathValidation(#[from] PathValidationError),

    #[error("{message}: {source}")]
    Context {
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Wraps arbitrary errors from external libraries
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

/// Standard exit codes for MAOS operations.
///
/// These exit codes provide semantic meaning to Claude Code and other tools
/// about the nature of errors that occur during hook execution.
///
/// # Exit Code Reference
///
/// | Code | Name           | Meaning                                      | Claude Code Behavior |
/// |------|----------------|----------------------------------------------|---------------------|
/// | 0    | Success        | Operation completed successfully            | Continue normally   |
/// | 1    | GeneralError   | General error (parsing, validation, etc.)   | Log and continue    |
/// | 2    | BlockingError  | Security violation - block tool execution   | **Block tool call** |
/// | 3    | ConfigError    | Configuration missing or invalid            | May retry           |
/// | 4    | SecurityError  | Security check failed                       | Log warning         |
/// | 5    | TimeoutError   | Operation exceeded time limit               | May retry           |
/// | 6    | ResourceError  | Resource limit exceeded (memory, etc.)      | Reduce load         |
/// | 99   | InternalError  | Unexpected internal error                   | Report bug          |
///
/// # Examples
/// ```
/// use maos_core::error::{MaosError, ExitCode};
/// let err = MaosError::Timeout { operation: "op".into(), timeout_ms: 5 };
/// let code: ExitCode = (&err).into();
/// assert_eq!(code, ExitCode::TimeoutError);
///
/// // Convert to process exit code
/// let process_code = std::process::ExitCode::from(code);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    /// Operation completed successfully
    Success = 0,
    /// General error (validation, parsing, I/O)
    GeneralError = 1,
    /// Security violation - Claude Code should block the tool execution
    BlockingError = 2,
    /// Configuration error (missing config, invalid values)
    ConfigError = 3,
    /// Security check failed (but not necessarily blocking)
    SecurityError = 4,
    /// Operation timeout exceeded
    TimeoutError = 5,
    /// Resource limit exceeded (memory, CPU, etc.)
    ResourceError = 6,
    /// Unexpected internal error (likely a bug)
    InternalError = 99,
}

impl From<&MaosError> for ExitCode {
    fn from(error: &MaosError) -> Self {
        match error {
            MaosError::Security(_) => ExitCode::SecurityError,
            MaosError::Config(_) => ExitCode::ConfigError,
            MaosError::ResourceLimit { .. } => ExitCode::ResourceError,
            MaosError::Timeout { .. } => ExitCode::TimeoutError,
            MaosError::Blocking { .. } => ExitCode::BlockingError,
            // PathValidation errors should block tool execution
            MaosError::PathValidation(_) => ExitCode::BlockingError,
            MaosError::Anyhow(_) => ExitCode::InternalError, // Unexpected errors map to 99
            MaosError::Context { source, .. } => {
                // Try to extract the underlying MaosError if possible
                if let Some(maos_err) = source.downcast_ref::<MaosError>() {
                    // Recursively get exit code from wrapped MaosError
                    ExitCode::from(maos_err)
                } else {
                    // Non-MaosError sources default to GeneralError
                    ExitCode::GeneralError
                }
            }
            _ => ExitCode::GeneralError,
        }
    }
}

/// Convert ExitCode to std::process::ExitCode for CLI exit.
///
/// This enables seamless integration with the Rust standard library's
/// process exit code handling.
///
/// # Examples
///
/// ```
/// use maos_core::ExitCode;
/// use std::process;
///
/// let code = ExitCode::BlockingError;
/// let process_code: process::ExitCode = code.into();
/// // Process will exit with code 2
/// ```
impl From<ExitCode> for std::process::ExitCode {
    fn from(code: ExitCode) -> Self {
        std::process::ExitCode::from(code as u8)
    }
}

// Domain-specific error types with structured information

/// Configuration-related errors with specific variants
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    #[error("Invalid configuration format: {reason}")]
    InvalidFormat { reason: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid value for {field}: {value} - {reason}")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
    },

    #[error("Permission denied accessing config: {path}")]
    PermissionDenied { path: String },

    #[error("{0}")]
    Other(String),
}

/// Session management errors
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found: {id}")]
    NotFound { id: String },

    #[error("Session already exists: {id}")]
    AlreadyExists { id: String },

    #[error("Session expired: {id} (expired at {expired_at})")]
    Expired { id: String, expired_at: String },

    #[error("Invalid session state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Session lock failed: {reason}")]
    LockFailed { reason: String },

    #[error("{0}")]
    Other(String),
}

/// Security validation errors
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Unauthorized access to resource: {resource}")]
    Unauthorized { resource: String },

    #[error("Path traversal attempt detected: {path}")]
    PathTraversal { path: String },

    #[error("Invalid permissions on {path}: expected {expected}, got {actual}")]
    InvalidPermissions {
        path: String,
        expected: String,
        actual: String,
    },

    #[error("Suspicious command detected: {command}")]
    SuspiciousCommand { command: String },

    #[error("Security policy violation: {policy}")]
    PolicyViolation { policy: String },

    #[error("{0}")]
    Other(String),
}

/// File system operation errors
#[derive(Debug, Error)]
pub enum FileSystemError {
    #[error("File not found: {path}")]
    NotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Path already exists: {path}")]
    AlreadyExists { path: String },

    #[error("Not a directory: {path}")]
    NotADirectory { path: String },

    #[error("Not a file: {path}")]
    NotAFile { path: String },

    #[error("Disk space exhausted")]
    NoSpace,

    #[error("Too many open files")]
    TooManyOpenFiles,

    #[error("{0}")]
    Other(String),
}

/// Git operation errors
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Repository not found at {path}")]
    RepoNotFound { path: String },

    #[error("Not a git repository (or any of the parent directories)")]
    NotARepository,

    #[error("Branch not found: {branch}")]
    BranchNotFound { branch: String },

    #[error("Merge conflict in {files:?}")]
    MergeConflict { files: Vec<String> },

    #[error("Uncommitted changes in working directory")]
    UncommittedChanges,

    #[error("Remote operation failed: {reason}")]
    RemoteError { reason: String },

    #[error("Invalid ref: {ref_name}")]
    InvalidRef { ref_name: String },

    #[error("{0}")]
    Other(String),
}

/// Input validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },

    #[error("Invalid format for {field}: expected {expected}, got {actual}")]
    InvalidFormat {
        field: String,
        expected: String,
        actual: String,
    },

    #[error("Value out of range for {field}: {value} (expected {min}..{max})")]
    OutOfRange {
        field: String,
        value: String,
        min: String,
        max: String,
    },

    #[error("Invalid length for {field}: {actual} (expected {expected})")]
    InvalidLength {
        field: String,
        actual: usize,
        expected: String,
    },

    #[error("Pattern validation failed for {field}: {pattern}")]
    PatternMismatch { field: String, pattern: String },

    #[error("{0}")]
    Other(String),
}

/// Path validation and security errors
///
/// These errors are designed with security-first principles:
/// - Fail closed by default (deny access, require explicit allows)
/// - Minimal information leakage in error messages
/// - Clear indication of security violations without revealing sensitive paths
#[derive(Debug, Error)]
pub enum PathValidationError {
    /// Path traversal attempt detected (e.g., "../../../etc/passwd")
    /// 🔒 SECURITY FIX: No path information leaked
    #[error("Path traversal attempt blocked")]
    PathTraversal { path: std::path::PathBuf },

    /// Path is outside the allowed workspace boundary
    /// 🔒 SECURITY FIX: No sensitive path information leaked
    #[error("Path outside allowed workspace boundary")]
    OutsideWorkspace {
        path: std::path::PathBuf,
        workspace: std::path::PathBuf,
    },

    /// Path matches a blocked pattern (e.g., .git/hooks, .ssh)
    /// 🔒 SECURITY FIX: No specific path leaked to prevent information disclosure
    #[error("Access to path blocked by security policy")]
    BlockedPath(std::path::PathBuf),

    /// Failed to canonicalize path (resolve symlinks and relative paths)
    /// 🔒 SECURITY FIX: Generic error message to prevent path disclosure
    #[error("Path canonicalization failed")]
    CanonicalizationFailed(std::path::PathBuf, #[source] std::io::Error),

    /// Workspace root path is invalid or inaccessible
    /// 🔒 SECURITY FIX: No workspace path leaked
    #[error("Invalid or inaccessible workspace")]
    InvalidWorkspace(std::path::PathBuf, #[source] std::io::Error),

    /// Path component contains invalid characters or patterns
    #[error("Invalid path component: {0}")]
    InvalidComponent(String),
}

/// Error context extension trait to attach additional context during propagation.
///
/// This helps preserve actionable context while bubbling errors upwards.
///
/// # Exit Code Preservation
///
/// Exit codes are best preserved when the underlying error is a `MaosError`.
/// For optimal exit code fidelity, convert errors to `MaosError` using
/// `.into_maos_error()` before applying `.with_context()`.
///
/// # Examples
///
/// Basic usage:
/// ```
/// use maos_core::error::{ErrorContext, Result, MaosError};
///
/// fn parse() -> Result<()> {
///     Err(MaosError::InvalidInput { message: "bad".into() })
///         .with_context(|| "while parsing input".to_string())
/// }
///
/// let err = parse().unwrap_err();
/// let s = format!("{err}");
/// assert!(s.contains("while parsing input"));
/// ```
///
/// Preserving exit codes:
/// ```
/// use maos_core::error::{ErrorContext, IntoMaosError, Result, ValidationError};
///
/// fn validate() -> Result<()> {
///     Err(ValidationError::RequiredFieldMissing { field: "name".into() })
///         .into_maos_error()  // Convert to MaosError first
///         .with_context(|| "during config validation".to_string())
/// }
/// ```
pub trait ErrorContext<T> {
    fn with_context<F>(self, f: F) -> MaosResult<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<F>(self, f: F) -> MaosResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| MaosError::Context {
            message: f(),
            source: Box::new(e),
        })
    }
}

/// Additional trait for converting any error into MaosError with context
pub trait IntoMaosError<T> {
    fn into_maos_error(self) -> MaosResult<T>;
}

impl<T, E> IntoMaosError<T> for std::result::Result<T, E>
where
    E: Into<MaosError>,
{
    fn into_maos_error(self) -> MaosResult<T> {
        self.map_err(Into::into)
    }
}

/// Convert a MaosError to an appropriate ExitCode for CLI operations.
///
/// This function provides a single source of truth for error-to-exit-code mapping,
/// ensuring consistent behavior across all handlers and the main CLI.
///
/// # Context Error Unwrapping
///
/// The exit code mapping correctly handles nested errors through context unwrapping:
///
/// ```
/// use maos_core::{MaosError, ExitCode, error_to_exit_code, PathValidationError};
///
/// // Original security error
/// let security_err = MaosError::PathValidation(
///     PathValidationError::PathTraversal { path: "/etc/passwd".into() }
/// );
///
/// // Exit code is correctly mapped
/// assert_eq!(error_to_exit_code(&security_err), ExitCode::BlockingError);
/// ```
///
/// # Examples
///
/// ```
/// use maos_core::error::{error_to_exit_code, MaosError, ExitCode};
///
/// let err = MaosError::Timeout { operation: "test".into(), timeout_ms: 100 };
/// assert_eq!(error_to_exit_code(&err), ExitCode::TimeoutError);
/// ```
///
/// # Note on const fn
///
/// This function cannot be made `const` because it relies on the `From<&MaosError>` trait
/// implementation which involves dynamic dispatch for Context error unwrapping and
/// `downcast_ref` operations that are not available in const contexts.
pub fn error_to_exit_code(error: &MaosError) -> ExitCode {
    ExitCode::from(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_to_process_exit_code() {
        // RED TEST: This will fail until we implement From<ExitCode> for std::process::ExitCode
        assert_eq!(
            std::process::ExitCode::from(ExitCode::Success),
            std::process::ExitCode::from(0)
        );
        assert_eq!(
            std::process::ExitCode::from(ExitCode::GeneralError),
            std::process::ExitCode::from(1)
        );
        assert_eq!(
            std::process::ExitCode::from(ExitCode::BlockingError),
            std::process::ExitCode::from(2)
        );
        assert_eq!(
            std::process::ExitCode::from(ExitCode::ConfigError),
            std::process::ExitCode::from(3)
        );
        assert_eq!(
            std::process::ExitCode::from(ExitCode::SecurityError),
            std::process::ExitCode::from(4)
        );
        assert_eq!(
            std::process::ExitCode::from(ExitCode::TimeoutError),
            std::process::ExitCode::from(5)
        );
        assert_eq!(
            std::process::ExitCode::from(ExitCode::ResourceError),
            std::process::ExitCode::from(6)
        );
        assert_eq!(
            std::process::ExitCode::from(ExitCode::InternalError),
            std::process::ExitCode::from(99)
        );
    }

    #[test]
    fn test_path_validation_errors_map_to_blocking() {
        // Test that all PathValidationError variants map to BlockingError
        let errors = vec![
            MaosError::PathValidation(PathValidationError::PathTraversal {
                path: "/etc/passwd".into(),
            }),
            MaosError::PathValidation(PathValidationError::OutsideWorkspace {
                path: "/tmp/bad".into(),
                workspace: "/workspace".into(),
            }),
            MaosError::PathValidation(PathValidationError::BlockedPath("/etc/ssh".into())),
        ];

        for err in errors {
            assert_eq!(
                error_to_exit_code(&err),
                ExitCode::BlockingError,
                "PathValidation error should map to BlockingError"
            );
        }
    }

    #[test]
    fn test_security_errors_map_correctly() {
        // Security errors should map to SecurityError exit code
        let err = MaosError::Security(SecurityError::PathTraversal {
            path: "../etc".into(),
        });
        assert_eq!(error_to_exit_code(&err), ExitCode::SecurityError);

        let err = MaosError::Security(SecurityError::Unauthorized {
            resource: "admin".into(),
        });
        assert_eq!(error_to_exit_code(&err), ExitCode::SecurityError);
    }

    #[test]
    fn test_context_error_unwrapping() {
        // Test that Context errors properly unwrap to get the underlying exit code
        let inner = MaosError::Timeout {
            operation: "test".into(),
            timeout_ms: 100,
        };

        let wrapped = MaosError::Context {
            message: "During operation".into(),
            source: Box::new(inner),
        };

        assert_eq!(
            error_to_exit_code(&wrapped),
            ExitCode::TimeoutError,
            "Context should unwrap to inner MaosError's exit code"
        );
    }

    #[test]
    fn test_all_error_variants_have_exit_codes() {
        // Comprehensive test for all MaosError variants
        let test_cases = vec![
            (
                MaosError::Config(ConfigError::FileNotFound {
                    path: "test".into(),
                }),
                ExitCode::ConfigError,
            ),
            (
                MaosError::Session(SessionError::NotFound { id: "123".into() }),
                ExitCode::GeneralError,
            ),
            (
                MaosError::Security(SecurityError::Unauthorized {
                    resource: "test".into(),
                }),
                ExitCode::SecurityError,
            ),
            (
                MaosError::FileSystem(FileSystemError::NotFound {
                    path: "test".into(),
                }),
                ExitCode::GeneralError,
            ),
            (
                MaosError::Git(GitError::NotARepository),
                ExitCode::GeneralError,
            ),
            (
                MaosError::Json(serde_json::from_str::<String>("invalid").unwrap_err()),
                ExitCode::GeneralError,
            ),
            (
                MaosError::InvalidInput {
                    message: "test".into(),
                },
                ExitCode::GeneralError,
            ),
            (
                MaosError::ResourceLimit {
                    resource: "memory".into(),
                    limit: 100,
                    actual: 200,
                    message: "too much".into(),
                },
                ExitCode::ResourceError,
            ),
            (
                MaosError::Timeout {
                    operation: "test".into(),
                    timeout_ms: 100,
                },
                ExitCode::TimeoutError,
            ),
            (
                MaosError::Blocking {
                    reason: "security".into(),
                },
                ExitCode::BlockingError,
            ),
            (
                MaosError::Validation(ValidationError::RequiredFieldMissing {
                    field: "name".into(),
                }),
                ExitCode::GeneralError,
            ),
        ];

        for (error, expected_code) in test_cases {
            assert_eq!(
                error_to_exit_code(&error),
                expected_code,
                "Error {error:?} should map to {expected_code:?}"
            );
        }
    }

    #[test]
    fn test_error_to_exit_code_helper_function() {
        // Test the helper function works identically to From trait
        let err = MaosError::Blocking {
            reason: "test".into(),
        };
        assert_eq!(error_to_exit_code(&err), ExitCode::from(&err));

        let err = MaosError::Config(ConfigError::MissingField {
            field: "api_key".into(),
        });
        assert_eq!(error_to_exit_code(&err), ExitCode::from(&err));
    }

    #[test]
    fn test_invalid_input_error_and_exit_code() {
        let err = MaosError::InvalidInput {
            message: "bad arg".into(),
        };
        let code: ExitCode = (&err).into();
        assert_eq!(code as i32, ExitCode::GeneralError as i32);
        let _res: Result<()> = Err(err);
    }

    #[test]
    fn test_blocking_exit_code_mapping() {
        let err = MaosError::Blocking {
            reason: "policy".into(),
        };
        let code: ExitCode = (&err).into();
        assert_eq!(code, ExitCode::BlockingError);
    }

    #[test]
    fn test_validation_error_variants() {
        // Test structured validation errors
        let val = ValidationError::RequiredFieldMissing {
            field: "username".into(),
        };
        let err: MaosError = val.into();
        let code: ExitCode = (&err).into();
        assert_eq!(code, ExitCode::GeneralError);

        let val2 = ValidationError::OutOfRange {
            field: "age".into(),
            value: "150".into(),
            min: "0".into(),
            max: "120".into(),
        };
        let display = format!("{val2}");
        assert!(display.contains("out of range"));
        assert!(display.contains("age"));
    }

    #[test]
    fn test_error_context_preserved() {
        fn might_fail() -> Result<()> {
            Err(ValidationError::RequiredFieldMissing {
                field: "name".into(),
            })
            .into_maos_error()
            .with_context(|| "while parsing config".to_string())
        }

        let err = might_fail().unwrap_err();
        let s = format!("{err}");
        assert!(s.contains("while parsing config"));
        assert!(s.contains("Required field missing"));
    }

    #[test]
    fn test_nested_context_exit_codes() {
        // Test that nested contexts preserve the original error's exit code
        let sec_err = SecurityError::Unauthorized {
            resource: "secret".into(),
        };
        // First convert to MaosError, then wrap in context
        let maos_err: MaosError = sec_err.into();
        let with_context = MaosError::Context {
            message: "during startup".into(),
            source: Box::new(maos_err) as Box<dyn std::error::Error + Send + Sync>,
        };

        // Should still map to SecurityError exit code
        let code: ExitCode = (&with_context).into();
        assert_eq!(code, ExitCode::SecurityError);
    }

    #[test]
    fn test_anyhow_integration() {
        // Test that we can wrap anyhow errors
        let anyhow_err = anyhow::anyhow!("external library error");
        let maos_err: MaosError = anyhow_err.into();
        let display = format!("{maos_err}");
        assert!(display.contains("external library error"));

        // Anyhow errors should map to InternalError (exit code 99)
        let code: ExitCode = (&maos_err).into();
        assert_eq!(code, ExitCode::InternalError);
    }

    #[test]
    fn test_timeout_exit_code_mapping() {
        let err = MaosError::Timeout {
            operation: "op".into(),
            timeout_ms: 1234,
        };
        let code: ExitCode = (&err).into();
        assert_eq!(code, ExitCode::TimeoutError);
    }

    #[test]
    fn test_from_std_io_error() {
        let io_err = std::io::Error::other("oops");
        let err: MaosError = io_err.into();
        // Default mapping for IO should be GeneralError
        let code: ExitCode = (&err).into();
        assert_eq!(code, ExitCode::GeneralError);
    }

    #[test]
    fn test_config_error_variants() {
        let cfg_err = ConfigError::FileNotFound {
            path: "/etc/maos/config.toml".into(),
        };
        let err: MaosError = cfg_err.into();
        let code: ExitCode = (&err).into();
        assert_eq!(code, ExitCode::ConfigError);

        let cfg_err2 = ConfigError::InvalidValue {
            field: "timeout".into(),
            value: "-1".into(),
            reason: "must be positive".into(),
        };
        let display = format!("{cfg_err2}");
        assert!(display.contains("timeout"));
        assert!(display.contains("-1"));
    }

    #[test]
    fn test_session_error_variants() {
        let sess_err = SessionError::NotFound {
            id: "sess_123".into(),
        };
        let display = format!("{sess_err}");
        assert!(display.contains("not found"));
        assert!(display.contains("sess_123"));

        let sess_err2 = SessionError::InvalidStateTransition {
            from: "active".into(),
            to: "expired".into(),
        };
        let err: MaosError = sess_err2.into();
        let display = format!("{err}");
        assert!(display.contains("state transition"));
    }

    #[test]
    fn test_security_error_variants() {
        let sec_err = SecurityError::PathTraversal {
            path: "../../../etc/passwd".into(),
        };
        let err: MaosError = sec_err.into();
        let code: ExitCode = (&err).into();
        assert_eq!(code, ExitCode::SecurityError);

        let sec_err2 = SecurityError::Unauthorized {
            resource: "/admin/panel".into(),
        };
        let display = format!("{sec_err2}");
        assert!(display.contains("Unauthorized"));
    }

    #[test]
    fn test_filesystem_error_variants() {
        let fs_err = FileSystemError::NotFound {
            path: "/tmp/missing.txt".into(),
        };
        let display = format!("{fs_err}");
        assert!(display.contains("not found"));

        let fs_err2 = FileSystemError::NoSpace;
        let display = format!("{fs_err2}");
        assert!(display.contains("space"));
    }

    #[test]
    fn test_git_error_variants() {
        let git_err = GitError::NotARepository;
        let display = format!("{git_err}");
        assert!(display.to_lowercase().contains("not a git repository"));

        let git_err2 = GitError::MergeConflict {
            files: vec!["file1.rs".into(), "file2.rs".into()],
        };
        let display = format!("{git_err2}");
        assert!(display.contains("conflict"));
        assert!(display.contains("file1.rs"));
    }

    #[test]
    fn test_context_with_anyhow_error() {
        // Test that Context wrapping an anyhow error maps to InternalError
        let anyhow_err = anyhow::anyhow!("unexpected failure");
        // First convert to MaosError::Anyhow, then wrap in context
        let maos_err: MaosError = anyhow_err.into();
        let with_context = MaosError::Context {
            message: "during initialization".into(),
            source: Box::new(maos_err) as Box<dyn std::error::Error + Send + Sync>,
        };

        // Context wrapping non-MaosError should default to InternalError for anyhow
        let code: ExitCode = (&with_context).into();
        assert_eq!(code, ExitCode::InternalError);

        let display = format!("{with_context}");
        assert!(display.contains("during initialization"));
        assert!(display.contains("unexpected failure"));
    }

    #[test]
    fn test_context_with_std_error() {
        // Test that Context wrapping a standard error maps to GeneralError
        let io_err = std::io::Error::other("io problem");
        let with_context = MaosError::Context {
            message: "while reading file".into(),
            source: Box::new(io_err),
        };

        // Context wrapping non-MaosError std errors should default to GeneralError
        let code: ExitCode = (&with_context).into();
        assert_eq!(code, ExitCode::GeneralError);
    }

    // ============================================================================
    // CRITICAL SECURITY TESTS - DO NOT MODIFY WITHOUT SECURITY REVIEW
    // ============================================================================
    //
    // These tests protect the security-critical exit code mappings that Claude Code
    // depends on for blocking tool execution. Exit code 2 (BlockingError) MUST be
    // returned for all PathValidation errors to prevent security bypasses.
    //
    // PathValidation → BlockingError mapping is CRITICAL for:
    // - Preventing path traversal attacks
    // - Blocking access to sensitive files
    // - Enforcing workspace boundaries
    //
    // NEVER change these mappings without a thorough security review!
    // ============================================================================

    #[test]
    fn test_critical_security_exit_code_mappings_never_change() {
        // This test MUST NEVER fail or be modified without security review
        // Exit code 2 is required by Claude Code to block tool execution

        // Direct PathValidation errors MUST map to BlockingError
        assert_eq!(
            ExitCode::BlockingError as i32,
            2,
            "SECURITY CRITICAL: BlockingError must always be exit code 2"
        );

        // All PathValidation variants MUST map to BlockingError
        let path_errors = vec![
            MaosError::PathValidation(PathValidationError::PathTraversal {
                path: "/etc/passwd".into(),
            }),
            MaosError::PathValidation(PathValidationError::OutsideWorkspace {
                path: "/tmp/bad".into(),
                workspace: "/workspace".into(),
            }),
            MaosError::PathValidation(PathValidationError::BlockedPath("/etc/ssh".into())),
            MaosError::PathValidation(PathValidationError::InvalidComponent("..".into())),
        ];

        for err in path_errors {
            assert_eq!(
                error_to_exit_code(&err),
                ExitCode::BlockingError,
                "SECURITY CRITICAL: PathValidation error {err:?} must map to BlockingError (exit code 2)"
            );

            // Also verify the numeric value
            let exit_code = error_to_exit_code(&err);
            assert_eq!(
                exit_code as i32, 2,
                "SECURITY CRITICAL: PathValidation must return exit code 2, got {}",
                exit_code as i32
            );
        }

        // Blocking errors must also remain exit code 2
        let blocking_err = MaosError::Blocking {
            reason: "security violation".into(),
        };
        assert_eq!(
            error_to_exit_code(&blocking_err),
            ExitCode::BlockingError,
            "SECURITY CRITICAL: Blocking error must map to BlockingError"
        );
    }

    #[test]
    fn test_security_exit_codes_through_context_wrapping() {
        // Ensure security exit codes are preserved through Context wrapping
        let inner = MaosError::PathValidation(PathValidationError::PathTraversal {
            path: "/etc/passwd".into(),
        });

        let wrapped = MaosError::Context {
            message: "During file access".into(),
            source: Box::new(inner),
        };

        assert_eq!(
            error_to_exit_code(&wrapped),
            ExitCode::BlockingError,
            "SECURITY CRITICAL: Context-wrapped PathValidation must preserve BlockingError exit code"
        );

        // Verify numeric value
        assert_eq!(
            error_to_exit_code(&wrapped) as i32,
            2,
            "SECURITY CRITICAL: Context-wrapped security error must return exit code 2"
        );
    }
}
