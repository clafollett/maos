//! Handler for session_start hook events

use crate::cli::handler::{CommandHandler, CommandResult, ExecutionMetrics};
use crate::io::HookInput;
use async_trait::async_trait;
use maos_core::{ExitCode, Result, hook_constants::SESSION_START};

/// Handler for session initialization hook events from Claude Code
///
/// This handler processes events when new Claude Code sessions begin or resume.
/// Currently provides basic validation and placeholder logic. Full implementation
/// planned for PRD-06 will include:
///
/// - Session state initialization and workspace setup
/// - Agent coordination bootstrap and capacity planning
/// - Configuration loading and environment validation
/// - Performance monitoring and metrics initialization
///
/// # Hook Event
///
/// Responds to `session_start` events with required `session_id` field and optional
/// `source` field indicating the session initialization type.
///
/// # Example
///
/// ```rust,no_run
/// use maos::cli::handlers::SessionStartHandler;
/// use maos::cli::handler::CommandHandler;
/// use maos::io::HookInput;
///
/// # async fn example() -> maos_core::Result<()> {
/// let handler = SessionStartHandler;
/// let input = HookInput {
///     hook_event_name: "session_start".to_string(),
///     session_id: "sess_abc123".to_string(),
///     source: Some("startup".to_string()),
///     ..Default::default()
/// };
///
/// let result = handler.execute(input).await?;
/// assert_eq!(result.exit_code, maos_core::ExitCode::Success);
/// # Ok(())
/// # }
/// ```
pub struct SessionStartHandler;

#[async_trait]
impl CommandHandler for SessionStartHandler {
    /// Execute session initialization processing
    ///
    /// Currently processes the session ID and returns success.
    /// Future PRD-06 implementation will add session setup,
    /// workspace initialization, and agent coordination bootstrap.
    ///
    /// # Arguments
    ///
    /// * `input` - Hook input from Claude Code containing session information
    ///
    /// # Returns
    ///
    /// Returns [`CommandResult`] with success status and session confirmation,
    /// including session ID in the output for tracking purposes.
    ///
    /// # Errors
    ///
    /// Currently always succeeds. Future implementation may return errors for
    /// workspace setup failures or configuration validation issues.
    ///
    /// [`CommandResult`]: crate::cli::handler::CommandResult
    async fn execute(&self, input: HookInput) -> Result<CommandResult> {
        // NOTE: Full session_start logic (session initialization, workspace setup,
        // and agent coordination bootstrap) is planned as a future enhancement in PRD-06.
        // The current implementation intentionally provides basic validation and success response.

        Ok(CommandResult {
            exit_code: ExitCode::Success,
            output: Some(format!("Session started: {}", input.session_id)),
            metrics: ExecutionMetrics::default(),
        })
    }

    /// Returns the hook event name constant
    ///
    /// # Returns
    ///
    /// Returns `"session_start"` as defined in [`maos_core::hook_constants`]
    ///
    /// [`maos_core::hook_constants`]: maos_core::hook_constants
    fn name(&self) -> &'static str {
        SESSION_START
    }

    /// Validates that hook input matches session_start event
    ///
    /// Ensures the `hook_event_name` field matches the expected "session_start" value.
    ///
    /// # Arguments
    ///
    /// * `input` - Hook input to validate
    ///
    /// # Errors
    ///
    /// Returns [`MaosError::InvalidInput`] if `hook_event_name` doesn't match "session_start".
    fn validate_input(&self, input: &HookInput) -> Result<()> {
        // Ensure hook_event_name matches
        if input.hook_event_name != SESSION_START {
            return Err(maos_core::MaosError::InvalidInput {
                message: format!("Expected session_start hook, got {}", input.hook_event_name),
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

    // NOTE: Additional tests for full business logic implementation will be added with PRD-06.
    // session initialization, workspace setup, and agent coordination bootstrap

    fn create_valid_hook_input() -> HookInput {
        HookInput {
            hook_event_name: SESSION_START.to_string(),
            session_id: "test-session-start-123".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_handler_name_returns_correct_constant() {
        let handler = SessionStartHandler;
        assert_eq!(handler.name(), SESSION_START);
    }

    #[tokio::test]
    async fn test_execute_success_with_valid_input() {
        let handler = SessionStartHandler;
        let input = create_valid_hook_input();

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("test-session-start-123"));
    }

    #[tokio::test]
    async fn test_validate_input_success() {
        let handler = SessionStartHandler;
        let input = create_valid_hook_input();

        let result = handler.validate_input(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_input_wrong_hook_event() {
        let handler = SessionStartHandler;
        let mut input = create_valid_hook_input();
        input.hook_event_name = "invalid_event".to_string(); // Different event to test validation

        let result = handler.validate_input(&input);

        assert!(result.is_err());
        match result.unwrap_err() {
            maos_core::MaosError::InvalidInput { message } => {
                assert!(message.contains("Expected session_start"));
                assert!(message.contains("invalid_event"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_session_id_processing() {
        let handler = SessionStartHandler;
        let mut input = create_valid_hook_input();
        input.session_id = "custom-session-id-999".to_string();

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.unwrap().contains("custom-session-id-999"));
    }
}
