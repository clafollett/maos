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
/// Use [`ExitCode::from`] with a reference to [`MaosError`] to consistently map
/// errors to process exit codes.
///
/// # Examples
/// ```
/// use maos_core::error::{MaosError, ExitCode};
/// let err = MaosError::Timeout { operation: "op".into(), timeout_ms: 5 };
/// let code: ExitCode = (&err).into();
/// assert_eq!(code, ExitCode::TimeoutError);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    BlockingError = 2,
    ConfigError = 3,
    SecurityError = 4,
    TimeoutError = 5,
    InternalError = 99,
}

impl From<&MaosError> for ExitCode {
    fn from(error: &MaosError) -> Self {
        match error {
            MaosError::Security(_) => ExitCode::SecurityError,
            MaosError::Config(_) => ExitCode::ConfigError,
            MaosError::Timeout { .. } => ExitCode::TimeoutError,
            MaosError::Blocking { .. } => ExitCode::BlockingError,
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
    #[error("Path traversal attempt blocked: {path}")]
    PathTraversal { path: std::path::PathBuf },

    /// Path is outside the allowed workspace boundary
    #[error("Path outside workspace: {path} not in {workspace}")]
    OutsideWorkspace {
        path: std::path::PathBuf,
        workspace: std::path::PathBuf,
    },

    /// Path matches a blocked pattern (e.g., .git/hooks, .ssh)
    #[error("Blocked path pattern: {0}")]
    BlockedPath(std::path::PathBuf),

    /// Failed to canonicalize path (resolve symlinks and relative paths)
    #[error("Canonicalization failed for {0}: {1}")]
    CanonicalizationFailed(std::path::PathBuf, #[source] std::io::Error),

    /// Workspace root path is invalid or inaccessible
    #[error("Invalid workspace: {0}")]
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
