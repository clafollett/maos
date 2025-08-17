//! Handler for subagent_stop hook events

use crate::cli::handler::{CommandHandler, CommandResult, ExecutionMetrics};
use crate::io::HookInput;
use async_trait::async_trait;
use maos_core::{ExitCode, Result, hook_constants::SUBAGENT_STOP};

/// Handler for subagent completion hook events from Claude Code
///
/// This handler processes events when spawned sub-agents complete their tasks.
/// Currently provides basic validation and placeholder logic. Full implementation
/// planned for PRD-06 will include:
///
/// - Agent lifecycle management and status tracking
/// - Workspace cleanup and resource deallocation
/// - Coordination state updates and synchronization
/// - Result aggregation and parent agent notification
///
/// # Hook Event
///
/// Responds to `subagent_stop` events indicating sub-agent task completion,
/// with optional status information about the agent's final state.
///
/// # Example
///
/// ```rust,no_run
/// use maos::cli::handlers::SubagentStopHandler;
/// use maos::cli::handler::CommandHandler;
/// use maos::io::HookInput;
///
/// # async fn example() -> maos_core::Result<()> {
/// let handler = SubagentStopHandler;
/// let input = HookInput {
///     hook_event_name: "subagent_stop".to_string(),
///     session_id: "agent_session_456".to_string(),
///     ..Default::default()
/// };
///
/// let result = handler.execute(input).await?;
/// assert_eq!(result.exit_code, maos_core::ExitCode::Success);
/// # Ok(())
/// # }
/// ```
pub struct SubagentStopHandler;

#[async_trait]
impl CommandHandler for SubagentStopHandler {
    /// Execute subagent completion processing
    ///
    /// Currently returns success status. Future PRD-06 implementation
    /// will add agent lifecycle management, workspace cleanup, and
    /// coordination state updates.
    ///
    /// # Arguments
    ///
    /// * `_input` - Hook input from Claude Code (currently unused)
    ///
    /// # Returns
    ///
    /// Returns [`CommandResult`] with success status.
    async fn execute(&self, _input: HookInput) -> Result<CommandResult> {
        // NOTE: Full subagent_stop logic (agent lifecycle management, workspace cleanup,
        // and coordination state updates) is planned as a future enhancement in PRD-06.
        // The current implementation intentionally provides basic validation and success response.

        Ok(CommandResult {
            exit_code: ExitCode::Success,
            output: Some("Subagent stop hook executed".to_string()),
            metrics: ExecutionMetrics::default(),
        })
    }

    /// Returns the hook event name constant
    ///
    /// # Returns
    ///
    /// Returns `"subagent_stop"` as defined in [`maos_core::hook_constants`]
    fn name(&self) -> &'static str {
        SUBAGENT_STOP
    }

    /// Validates that hook input matches subagent_stop event
    ///
    /// Ensures the `hook_event_name` field matches the expected "subagent_stop" value.
    ///
    /// # Arguments
    ///
    /// * `input` - Hook input to validate
    ///
    /// # Errors
    ///
    /// Returns [`MaosError::InvalidInput`] if `hook_event_name` doesn't match "subagent_stop".
    fn validate_input(&self, input: &HookInput) -> Result<()> {
        // Ensure hook_event_name matches
        if input.hook_event_name != SUBAGENT_STOP {
            return Err(maos_core::MaosError::InvalidInput {
                message: format!("Expected subagent_stop hook, got {}", input.hook_event_name),
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
    // agent lifecycle management, workspace cleanup, and coordination state updates

    fn create_valid_hook_input() -> HookInput {
        HookInput {
            hook_event_name: SUBAGENT_STOP.to_string(),
            session_id: "test-session-subagent".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_handler_name_returns_correct_constant() {
        let handler = SubagentStopHandler;
        assert_eq!(handler.name(), SUBAGENT_STOP);
    }

    #[tokio::test]
    async fn test_execute_success_with_valid_input() {
        let handler = SubagentStopHandler;
        let input = create_valid_hook_input();

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("Subagent stop"));
    }

    #[tokio::test]
    async fn test_validate_input_success() {
        let handler = SubagentStopHandler;
        let input = create_valid_hook_input();

        let result = handler.validate_input(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_input_wrong_hook_event() {
        let handler = SubagentStopHandler;
        let mut input = create_valid_hook_input();
        input.hook_event_name = "session_start".to_string(); // Different event to test validation

        let result = handler.validate_input(&input);

        assert!(result.is_err());
        match result.unwrap_err() {
            maos_core::MaosError::InvalidInput { message } => {
                assert!(message.contains("Expected subagent_stop"));
                assert!(message.contains("session_start"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }
}
