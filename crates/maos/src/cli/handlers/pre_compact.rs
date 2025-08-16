//! Handler for pre_compact hook events

use crate::cli::handler::{CommandHandler, CommandResult, ExecutionMetrics};
use crate::io::HookInput;
use async_trait::async_trait;
use maos_core::{ExitCode, Result, hook_constants::PRE_COMPACT};

/// Handler for pre-conversation compaction hook events from Claude Code
///
/// This handler processes events that occur before Claude Code compacts the
/// conversation history. Currently provides basic validation and placeholder logic.
/// Full implementation planned for PRD-06 will include:
///
/// - Conversation analysis and critical context identification
/// - Context preservation strategies and priority ranking
/// - Compaction coordination with active agents
/// - Memory optimization and performance tuning
///
/// # Hook Event
///
/// Responds to `pre_compact` events with optional `trigger` and `custom_instructions`
/// fields indicating the compaction reason and any user-provided directives.
///
/// # Example
///
/// ```rust,no_run
/// use maos::cli::handlers::PreCompactHandler;
/// use maos::cli::handler::CommandHandler;
/// use maos::io::HookInput;
///
/// # async fn example() -> maos_core::Result<()> {
/// let handler = PreCompactHandler;
/// let input = HookInput {
///     hook_event_name: "pre_compact".to_string(),
///     trigger: Some("auto".to_string()),
///     custom_instructions: Some("Preserve task context".to_string()),
///     ..Default::default()
/// };
///
/// let result = handler.execute(input).await?;
/// assert_eq!(result.exit_code, maos_core::ExitCode::Success);
/// # Ok(())
/// # }
/// ```
pub struct PreCompactHandler;

#[async_trait]
impl CommandHandler for PreCompactHandler {
    /// Execute pre-compaction processing
    ///
    /// Currently returns success status. Future PRD-06 implementation
    /// will add conversation analysis, context preservation, and
    /// compaction coordination.
    ///
    /// # Arguments
    ///
    /// * `_input` - Hook input from Claude Code (currently unused)
    ///
    /// # Returns
    ///
    /// Returns [`CommandResult`] with success status.
    async fn execute(&self, _input: HookInput) -> Result<CommandResult> {
        // TODO:  (Implemented in PRD-06) Implement actual pre_compact logic - see PRD-06 for conversation
        // analysis, critical context preservation, and compaction coordination.
        // For now, just a stub that validates and returns success

        Ok(CommandResult {
            exit_code: ExitCode::Success,
            output: Some("Pre-compact hook executed".to_string()),
            metrics: ExecutionMetrics::default(),
        })
    }

    /// Returns the hook event name constant
    ///
    /// # Returns
    ///
    /// Returns `"pre_compact"` as defined in [`maos_core::hook_constants`]
    fn name(&self) -> &'static str {
        PRE_COMPACT
    }

    /// Validates that hook input matches pre_compact event
    ///
    /// Ensures the `hook_event_name` field matches the expected "pre_compact" value.
    ///
    /// # Arguments
    ///
    /// * `input` - Hook input to validate
    ///
    /// # Errors
    ///
    /// Returns [`MaosError::InvalidInput`] if `hook_event_name` doesn't match "pre_compact".
    fn validate_input(&self, input: &HookInput) -> Result<()> {
        // Ensure hook_event_name matches
        if input.hook_event_name != PRE_COMPACT {
            return Err(maos_core::MaosError::InvalidInput {
                message: format!("Expected pre_compact hook, got {}", input.hook_event_name),
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
    // conversation analysis, context preservation, and compaction coordination

    fn create_valid_hook_input() -> HookInput {
        HookInput {
            hook_event_name: PRE_COMPACT.to_string(),
            session_id: "test-session-compact".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_handler_name_returns_correct_constant() {
        let handler = PreCompactHandler;
        assert_eq!(handler.name(), PRE_COMPACT);
    }

    #[tokio::test]
    async fn test_execute_success_with_valid_input() {
        let handler = PreCompactHandler;
        let input = create_valid_hook_input();

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("Pre-compact"));
    }

    #[tokio::test]
    async fn test_validate_input_success() {
        let handler = PreCompactHandler;
        let input = create_valid_hook_input();

        let result = handler.validate_input(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_input_wrong_hook_event() {
        let handler = PreCompactHandler;
        let mut input = create_valid_hook_input();
        input.hook_event_name = "invalid_event".to_string(); // Different event to test validation

        let result = handler.validate_input(&input);

        assert!(result.is_err());
        match result.unwrap_err() {
            maos_core::MaosError::InvalidInput { message } => {
                assert!(message.contains("Expected pre_compact"));
                assert!(message.contains("invalid_event"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }
}
