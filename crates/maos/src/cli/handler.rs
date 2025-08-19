//! Command handler trait and result types

use crate::io::HookInput;
use async_trait::async_trait;
use maos_core::{ExitCode, Result};
use std::time::Duration;

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

/// Metrics collected during command execution
#[derive(Debug, Default)]
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
    use tokio::time::Instant;

    // Mock handler for testing
    struct MockHandler {
        name: &'static str,
        should_fail: bool,
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
}
