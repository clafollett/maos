//! Handler for stop hook events

use crate::cli::handler::{CommandHandler, CommandResult, ExecutionMetrics};
use crate::io::HookInput;
use async_trait::async_trait;
use maos_core::{ExitCode, Result, hook_constants::STOP};

/// Handler for session stop hook events from Claude Code
///
/// This handler processes events when Claude Code sessions terminate, following
/// the official Claude Code hook specification. Currently provides basic stop
/// event processing and placeholder logic. Full implementation planned for
/// PRD-06 will include:
///
/// - Session state cleanup and resource deallocation
/// - Agent coordination termination and synchronization
/// - Chat transcript export and archival (when stop_hook_active is true)
/// - Metrics finalization and reporting
///
/// # Hook Event
///
/// Responds to `stop` events with optional `stop_hook_active` field.
///
/// # Official Claude Code JSON Structure
///
/// ```json
/// {
///   "session_id": "abc123",
///   "transcript_path": "~/.claude/projects/.../transcript.jsonl",
///   "hook_event_name": "stop",
///   "stop_hook_active": true
/// }
/// ```
///
/// # Example
///
/// ```rust,no_run
/// use maos::cli::handlers::StopHandler;
/// use maos::cli::handler::CommandHandler;
/// use maos::io::HookInput;
///
/// # async fn example() -> maos_core::Result<()> {
/// let handler = StopHandler;
/// let input = HookInput {
///     hook_event_name: "stop".to_string(),
///     stop_hook_active: Some(true),
///     session_id: "session-123".to_string(),
///     ..Default::default()
/// };
///
/// let result = handler.execute(input).await?;
/// assert_eq!(result.exit_code, maos_core::ExitCode::Success);
/// # Ok(())
/// # }
/// ```
pub struct StopHandler;

#[async_trait]
impl CommandHandler for StopHandler {
    /// Execute session stop processing
    ///
    /// Currently processes the stop event and checks the `stop_hook_active` flag
    /// from Claude Code. Future PRD-06 implementation will add session cleanup,
    /// agent coordination termination, and chat transcript export.
    ///
    /// # Arguments
    ///
    /// * `input` - Hook input from Claude Code containing stop event details
    ///
    /// # Returns
    ///
    /// Returns [`CommandResult`] with success status and stop processing confirmation.
    ///
    /// # Errors
    ///
    /// Currently always succeeds. Future implementation may return errors for
    /// session cleanup failures or resource deallocation issues.
    ///
    /// [`CommandResult`]: crate::cli::handler::CommandResult
    async fn execute(&self, input: HookInput) -> Result<CommandResult> {
        // TODO: (Implemented in PRD-06) Implement actual stop logic - see PRD-06 for session cleanup,
        // agent coordination termination, and chat transcript export handling.
        // For now, just a stub that validates and returns success

        let hook_active = input.stop_hook_active.unwrap_or(false);
        let status = if hook_active { "active" } else { "inactive" };

        Ok(CommandResult {
            exit_code: ExitCode::Success,
            output: Some(format!("Stop hook executed with status: {}", status)),
            metrics: ExecutionMetrics::default(),
        })
    }

    /// Returns the hook event name constant
    ///
    /// # Returns
    ///
    /// Returns `"stop"` as defined in [`maos_core::hook_constants`]
    ///
    /// [`maos_core::hook_constants`]: maos_core::hook_constants
    fn name(&self) -> &'static str {
        STOP
    }

    /// Validates that hook input matches stop event
    ///
    /// Ensures the `hook_event_name` field matches the expected "stop" value
    /// according to the official Claude Code hook specification.
    ///
    /// # Arguments
    ///
    /// * `input` - Hook input to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if validation passes.
    ///
    /// # Errors
    ///
    /// Returns [`MaosError::InvalidInput`] if `hook_event_name` doesn't match "stop".
    ///
    /// [`MaosError::InvalidInput`]: maos_core::MaosError::InvalidInput
    fn validate_input(&self, input: &HookInput) -> Result<()> {
        // Ensure hook_event_name matches
        if input.hook_event_name != STOP {
            return Err(maos_core::MaosError::InvalidInput {
                message: format!("Expected stop hook, got {}", input.hook_event_name),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::HookInput;
    use maos_core::ExitCode;

    // TODO: (Implemented in PRD-06) Expand tests for full business logic implementation including
    // session cleanup, agent coordination termination, and chat transcript export

    fn create_valid_hook_input() -> HookInput {
        HookInput {
            hook_event_name: STOP.to_string(),
            session_id: "test-session-stop".to_string(),
            stop_hook_active: Some(true),
            ..Default::default()
        }
    }

    fn create_hook_input_inactive() -> HookInput {
        HookInput {
            hook_event_name: STOP.to_string(),
            session_id: "test-session-stop".to_string(),
            stop_hook_active: Some(false),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_handler_name_returns_correct_constant() {
        let handler = StopHandler;
        assert_eq!(handler.name(), STOP);
    }

    #[tokio::test]
    async fn test_execute_success_with_active_hook() {
        let handler = StopHandler;
        let input = create_valid_hook_input();

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("active"));
    }

    #[tokio::test]
    async fn test_execute_success_with_inactive_hook() {
        let handler = StopHandler;
        let input = create_hook_input_inactive();

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("inactive"));
    }

    #[tokio::test]
    async fn test_execute_success_with_missing_stop_hook_active() {
        let handler = StopHandler;
        let mut input = create_valid_hook_input();
        input.stop_hook_active = None; // Test default behavior

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("inactive")); // Should default to false
    }

    #[tokio::test]
    async fn test_validate_input_success() {
        let handler = StopHandler;
        let input = create_valid_hook_input();

        let result = handler.validate_input(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_input_wrong_hook_event() {
        let handler = StopHandler;
        let mut input = create_valid_hook_input();
        input.hook_event_name = "notification".to_string(); // Different event to test validation

        let result = handler.validate_input(&input);

        assert!(result.is_err());
        match result.unwrap_err() {
            maos_core::MaosError::InvalidInput { message } => {
                assert!(message.contains("Expected stop"));
                assert!(message.contains("notification"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_stop_hook_active_field_processing() {
        let handler = StopHandler;

        // Test explicit true
        let mut input = create_valid_hook_input();
        input.stop_hook_active = Some(true);
        let result = handler.execute(input).await.unwrap();
        assert!(result.output.unwrap().contains("active"));

        // Test explicit false
        let mut input = create_hook_input_inactive();
        input.stop_hook_active = Some(false);
        let result = handler.execute(input).await.unwrap();
        assert!(result.output.unwrap().contains("inactive"));
    }
}
