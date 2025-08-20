//! Command handler trait and result types

use crate::io::HookInput;
use async_trait::async_trait;
use maos_core::{ExitCode, Result};
use std::time::Duration;
use tracing::{Level, event};

/// Result returned by command handlers
#[derive(Debug)]
pub struct CommandResult {
    /// Exit code to return to the shell
    pub exit_code: ExitCode,
    /// Optional output to write to stdout
    pub output: Option<String>,
    /// Execution metrics for performance tracking
    pub metrics: ExecutionMetrics,
}

impl CommandResult {
    /// Create a successful result with no output.
    ///
    /// # Examples
    ///
    /// ```
    /// use maos::cli::handler::CommandResult;
    ///
    /// let result = CommandResult::success();
    /// assert_eq!(result.exit_code, maos_core::ExitCode::Success);
    /// ```
    pub fn success() -> Self {
        Self {
            exit_code: ExitCode::Success,
            output: None,
            metrics: ExecutionMetrics::default(),
        }
    }

    /// Create a blocking error result (exit code 2).
    ///
    /// Used for security violations that should block tool execution.
    /// Claude Code will prevent the tool from running when this is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use maos::cli::handler::CommandResult;
    ///
    /// let result = CommandResult::blocking_error("Path traversal detected".into());
    /// assert_eq!(result.exit_code, maos_core::ExitCode::BlockingError);
    /// ```
    pub fn blocking_error(reason: String) -> Self {
        Self {
            exit_code: ExitCode::BlockingError,
            output: Some(reason),
            metrics: ExecutionMetrics::default(),
        }
    }

    /// Create a configuration error result (exit code 3).
    ///
    /// Used when configuration is missing or invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use maos::cli::handler::CommandResult;
    ///
    /// let result = CommandResult::config_error("Missing API key".into());
    /// assert_eq!(result.exit_code, maos_core::ExitCode::ConfigError);
    /// ```
    pub fn config_error(reason: String) -> Self {
        Self {
            exit_code: ExitCode::ConfigError,
            output: Some(reason),
            metrics: ExecutionMetrics::default(),
        }
    }

    /// Create a result from a MaosError.
    ///
    /// Automatically maps the error to the appropriate exit code
    /// and formats the error message for output. Security-sensitive errors
    /// have their messages sanitized to prevent information leakage.
    ///
    /// # Examples
    ///
    /// ```
    /// use maos::cli::handler::CommandResult;
    /// use maos_core::MaosError;
    ///
    /// let err = MaosError::Timeout {
    ///     operation: "test".into(),
    ///     timeout_ms: 100
    /// };
    /// let result = CommandResult::from_error(&err);
    /// assert_eq!(result.exit_code, maos_core::ExitCode::TimeoutError);
    /// ```
    pub fn from_error(error: &maos_core::MaosError) -> Self {
        Self {
            exit_code: maos_core::error_to_exit_code(error),
            output: Some(sanitize_error_message(error)),
            metrics: ExecutionMetrics::default(),
        }
    }

    /// Builder method to add output to the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use maos::cli::handler::CommandResult;
    ///
    /// let result = CommandResult::success()
    ///     .with_output("Operation completed".into());
    /// assert_eq!(result.output, Some("Operation completed".into()));
    /// ```
    pub fn with_output(mut self, output: String) -> Self {
        self.output = Some(output);
        self
    }

    /// Builder method to set execution metrics.
    ///
    /// # Examples
    ///
    /// ```
    /// use maos::cli::handler::{CommandResult, ExecutionMetrics};
    /// use std::time::Duration;
    ///
    /// let metrics = ExecutionMetrics {
    ///     validation_time: Duration::from_millis(5),
    ///     handler_time: Duration::from_millis(50),
    ///     total_time: Duration::from_millis(55),
    /// };
    ///
    /// let result = CommandResult::success()
    ///     .with_metrics(metrics.clone());
    /// assert_eq!(result.metrics.total_time, Duration::from_millis(55));
    /// ```
    pub fn with_metrics(mut self, metrics: ExecutionMetrics) -> Self {
        self.metrics = metrics;
        self
    }
}

/// Sanitize error messages to prevent information leakage for security-sensitive errors.
///
/// This function ensures that PathValidation and Security errors return generic messages
/// that don't reveal sensitive path information or internal system details. Other error
/// types pass through unchanged. Security violations are logged for audit purposes.
///
/// # Security Rationale
/// Path validation errors often contain sensitive file system paths that could aid
/// attackers in reconnaissance. By sanitizing these messages, we maintain security
/// while still providing enough information for legitimate debugging.
///
/// # Security Logging
/// When security violations occur, structured logs are emitted for monitoring and
/// audit purposes. Logs include violation type and component but not sensitive details.
///
/// # Examples
///
/// ```
/// use maos::cli::handler::sanitize_error_message;
/// use maos_core::{MaosError, PathValidationError};
///
/// let path_error = MaosError::PathValidation(
///     PathValidationError::PathTraversal { path: "/etc/passwd".into() }
/// );
/// let sanitized = sanitize_error_message(&path_error);
/// assert_eq!(sanitized, "Path access denied for security reasons");
/// ```
pub fn sanitize_error_message(error: &maos_core::MaosError) -> String {
    match error {
        // Security-sensitive errors get generic messages and are logged
        maos_core::MaosError::PathValidation(path_err) => {
            let violation_type = match path_err {
                maos_core::PathValidationError::PathTraversal { .. } => "path_traversal",
                maos_core::PathValidationError::OutsideWorkspace { .. } => "workspace_escape",
                maos_core::PathValidationError::BlockedPath(_) => "blocked_path_access",
                maos_core::PathValidationError::InvalidComponent { .. } => "invalid_path_component",
                maos_core::PathValidationError::CanonicalizationFailed(..) => {
                    "path_canonicalization_failed"
                }
                maos_core::PathValidationError::InvalidWorkspace { .. } => "invalid_workspace",
            };

            event!(
                Level::WARN,
                session_id = "unknown",
                violation_type = violation_type,
                component = "path_validation",
                "Security violation: PathValidation error occurred"
            );

            "Path access denied for security reasons".to_string()
        }
        maos_core::MaosError::Security(sec_err) => {
            let violation_type = match sec_err {
                maos_core::SecurityError::Unauthorized { .. } => "unauthorized_access",
                maos_core::SecurityError::PathTraversal { .. } => "path_traversal",
                maos_core::SecurityError::InvalidPermissions { .. } => "invalid_permissions",
                maos_core::SecurityError::SuspiciousCommand { .. } => "suspicious_command",
                _ => "security_violation",
            };

            event!(
                Level::WARN,
                session_id = "unknown",
                violation_type = violation_type,
                component = "security",
                "Security violation: Security error occurred"
            );

            "Security validation failed".to_string()
        }
        // Context errors might wrap security errors, check recursively
        maos_core::MaosError::Context { source, message } => {
            if let Some(maos_err) = source.downcast_ref::<maos_core::MaosError>() {
                match maos_err {
                    maos_core::MaosError::PathValidation(_) => {
                        "Path access denied for security reasons".to_string()
                    }
                    maos_core::MaosError::Security(_) => "Security validation failed".to_string(),
                    _ => format!("{message}: {}", sanitize_error_message(maos_err)),
                }
            } else {
                // Non-MaosError source, use the context message but don't leak details
                message.clone()
            }
        }
        // All other errors pass through unchanged
        _ => format!("{error}"),
    }
}

/// Metrics collected during command execution
#[derive(Debug, Default, Clone)]
pub struct ExecutionMetrics {
    /// Time spent validating input
    pub validation_time: Duration,
    /// Time spent in handler logic
    pub handler_time: Duration,
    /// Total execution time
    pub total_time: Duration,
}

/// Trait for command handlers with async execution
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// Execute command with hook message input
    async fn execute(&self, input: HookInput) -> Result<CommandResult>;

    /// Get command name for logging/metrics
    fn name(&self) -> &'static str;

    /// Validate input before execution (optional)
    fn validate_input(&self, _input: &HookInput) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maos_core::MaosError;
    use tokio::time::Instant;

    // Mock handler for testing
    struct MockHandler {
        name: &'static str,
        should_fail: bool,
    }

    #[test]
    fn test_command_result_success_builder() {
        // RED TEST: This will fail until we implement CommandResult::success()
        let result = CommandResult::success();
        assert_eq!(result.exit_code, ExitCode::Success);
        assert_eq!(result.output, None);
    }

    #[test]
    fn test_command_result_blocking_error_builder() {
        // RED TEST: This will fail until we implement CommandResult::blocking_error()
        let reason = "Security violation detected".to_string();
        let result = CommandResult::blocking_error(reason.clone());
        assert_eq!(result.exit_code, ExitCode::BlockingError);
        assert_eq!(result.output, Some(reason));
    }

    #[test]
    fn test_command_result_config_error_builder() {
        // RED TEST: This will fail until we implement CommandResult::config_error()
        let reason = "Missing API key".to_string();
        let result = CommandResult::config_error(reason.clone());
        assert_eq!(result.exit_code, ExitCode::ConfigError);
        assert_eq!(result.output, Some(reason));
    }

    #[test]
    fn test_command_result_from_error_builder() {
        // RED TEST: This will fail until we implement CommandResult::from_error()
        let err = MaosError::Timeout {
            operation: "test".into(),
            timeout_ms: 100,
        };
        let result = CommandResult::from_error(&err);
        assert_eq!(result.exit_code, ExitCode::TimeoutError);
        assert!(result.output.is_some());
    }

    #[test]
    fn test_command_result_with_output_builder() {
        // RED TEST: This will fail until we implement CommandResult::with_output()
        let result = CommandResult::success().with_output("Operation completed".to_string());
        assert_eq!(result.exit_code, ExitCode::Success);
        assert_eq!(result.output, Some("Operation completed".to_string()));
    }

    #[test]
    fn test_command_result_with_metrics_builder() {
        // RED TEST: This will fail until we implement CommandResult::with_metrics()
        let metrics = ExecutionMetrics {
            validation_time: Duration::from_millis(10),
            handler_time: Duration::from_millis(50),
            total_time: Duration::from_millis(60),
        };
        let result = CommandResult::success().with_metrics(metrics.clone());
        assert_eq!(result.metrics.validation_time, metrics.validation_time);
        assert_eq!(result.metrics.handler_time, metrics.handler_time);
        assert_eq!(result.metrics.total_time, metrics.total_time);
    }

    #[async_trait]
    impl CommandHandler for MockHandler {
        async fn execute(&self, _input: HookInput) -> Result<CommandResult> {
            if self.should_fail {
                return Err(maos_core::MaosError::InvalidInput {
                    message: "Mock failure".to_string(),
                });
            }

            Ok(CommandResult {
                exit_code: ExitCode::Success,
                output: Some("Mock output".to_string()),
                metrics: ExecutionMetrics::default(),
            })
        }

        fn name(&self) -> &'static str {
            self.name
        }
    }

    #[tokio::test]
    async fn test_handler_trait_execute() {
        let handler = MockHandler {
            name: "test_handler",
            should_fail: false,
        };

        let input = HookInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/transcript.jsonl".into(),
            cwd: "/tmp".into(),
            hook_event_name: maos_core::hook_constants::PRE_TOOL_USE.to_string(),
            tool_name: Some("Bash".to_string()),
            tool_input: Some(serde_json::json!({"command": "ls"})),
            ..Default::default()
        };

        let result = handler.execute(input).await.unwrap();
        assert_eq!(result.exit_code, ExitCode::Success);
        assert_eq!(result.output, Some("Mock output".to_string()));
    }

    #[tokio::test]
    async fn test_handler_trait_error_propagation() {
        let handler = MockHandler {
            name: "failing_handler",
            should_fail: true,
        };

        let input = HookInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/transcript.jsonl".into(),
            cwd: "/tmp".into(),
            hook_event_name: maos_core::hook_constants::PRE_TOOL_USE.to_string(),
            ..Default::default()
        };

        let result = handler.execute(input).await;
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(matches!(err, maos_core::MaosError::InvalidInput { .. }));
        }
    }

    #[test]
    fn test_handler_trait_validation() {
        let handler = MockHandler {
            name: "test_handler",
            should_fail: false,
        };

        let input = HookInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/transcript.jsonl".into(),
            cwd: "/tmp".into(),
            hook_event_name: maos_core::hook_constants::PRE_TOOL_USE.to_string(),
            ..Default::default()
        };

        // Default validation should always pass
        assert!(handler.validate_input(&input).is_ok());
    }

    #[tokio::test]
    async fn test_handler_trait_metrics() {
        let handler = MockHandler {
            name: "metrics_handler",
            should_fail: false,
        };

        let input = HookInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/transcript.jsonl".into(),
            cwd: "/tmp".into(),
            hook_event_name: maos_core::hook_constants::NOTIFICATION.to_string(),
            message: Some("Test notification".to_string()),
            ..Default::default()
        };

        let start = Instant::now();
        let result = handler.execute(input).await.unwrap();
        let elapsed = start.elapsed();

        // Metrics should be present (even if default/zero)
        assert!(result.metrics.total_time <= elapsed);
    }

    #[test]
    fn test_sanitize_error_message_path_validation() {
        let path_error = MaosError::PathValidation(maos_core::PathValidationError::PathTraversal {
            path: "/etc/passwd".into(),
        });
        let sanitized = sanitize_error_message(&path_error);
        assert_eq!(sanitized, "Path access denied for security reasons");
        // Ensure no path information is leaked
        assert!(!sanitized.contains("/etc/passwd"));
    }

    #[test]
    fn test_sanitize_error_message_security() {
        let security_error = MaosError::Security(maos_core::SecurityError::Unauthorized {
            resource: "admin credentials database".to_string(),
        });
        let sanitized = sanitize_error_message(&security_error);
        assert_eq!(sanitized, "Security validation failed");
        // Ensure no detailed security information is leaked
        assert!(!sanitized.contains("admin"));
        assert!(!sanitized.contains("credentials"));
        assert!(!sanitized.contains("database"));
    }

    #[test]
    fn test_sanitize_error_message_context_with_path_validation() {
        let inner_error =
            MaosError::PathValidation(maos_core::PathValidationError::OutsideWorkspace {
                path: "/home/user/secret.txt".into(),
                workspace: "/workspace".into(),
            });
        let context_error = MaosError::Context {
            message: "During file operation".to_string(),
            source: Box::new(inner_error),
        };
        let sanitized = sanitize_error_message(&context_error);
        assert_eq!(sanitized, "Path access denied for security reasons");
        // Ensure no path information is leaked from nested errors
        assert!(!sanitized.contains("/home/user/secret.txt"));
        assert!(!sanitized.contains("/workspace"));
    }

    #[test]
    fn test_sanitize_error_message_other_errors_pass_through() {
        let timeout_error = MaosError::Timeout {
            operation: "test operation".to_string(),
            timeout_ms: 5000,
        };
        let sanitized = sanitize_error_message(&timeout_error);
        // Non-security errors should pass through unchanged
        assert!(sanitized.contains("test operation"));
        assert!(sanitized.contains("5000"));
    }

    #[test]
    fn test_from_error_uses_sanitized_message() {
        let path_error = MaosError::PathValidation(maos_core::PathValidationError::BlockedPath(
            "/etc/hosts".into(),
        ));
        let result = CommandResult::from_error(&path_error);

        assert_eq!(result.exit_code, ExitCode::BlockingError);
        assert_eq!(
            result.output,
            Some("Path access denied for security reasons".to_string())
        );
        // Ensure the sensitive path is not in the output
        assert!(!result.output.unwrap_or_default().contains("/etc/hosts"));
    }
}
