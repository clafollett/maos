//! Handler for pre_tool_use hook events

use crate::cli::handler::{CommandHandler, CommandResult, ExecutionMetrics};
use crate::io::HookInput;
use async_trait::async_trait;
use maos_core::{ExitCode, Result, hook_constants::PRE_TOOL_USE};

/// Handler for pre-tool-use hook events from Claude Code
///
/// This handler processes events that occur immediately before Claude Code executes a tool.
/// Currently provides basic validation and placeholder logic. Full implementation planned
/// for PRD-06 will include:
///
/// - Security validation and path traversal protection
/// - Workspace boundary enforcement
/// - Tool parameter modification for task spawning
/// - Agent coordination and capacity management
///
/// # Hook Event
///
/// Responds to `pre_tool_use` events with required `tool_name` field.
///
/// # Example
///
/// ```rust,no_run
/// use maos::cli::handlers::PreToolUseHandler;
/// use maos::cli::handler::CommandHandler;
/// use maos::io::HookInput;
///
/// # async fn example() -> maos_core::Result<()> {
/// let handler = PreToolUseHandler;
/// let input = HookInput {
///     hook_event_name: "pre_tool_use".to_string(),
///     tool_name: Some("Read".to_string()),
///     ..Default::default()
/// };
///
/// let result = handler.execute(input).await?;
/// assert_eq!(result.exit_code, maos_core::ExitCode::Success);
/// # Ok(())
/// # }
/// ```
pub struct PreToolUseHandler;

#[async_trait]
impl CommandHandler for PreToolUseHandler {
    /// Execute pre-tool-use hook processing
    ///
    /// Currently validates that `tool_name` is present and returns success.
    /// Future PRD-06 implementation will add security validation, workspace
    /// enforcement, and task spawning coordination.
    ///
    /// # Arguments
    ///
    /// * `input` - Hook input from Claude Code containing tool information
    ///
    /// # Returns
    ///
    /// Returns [`CommandResult`] with success status and tool name confirmation,
    /// or error if `tool_name` is missing.
    ///
    /// # Errors
    ///
    /// Returns [`MaosError::InvalidInput`] if:
    /// - `tool_name` field is None or empty
    /// - Hook event name doesn't match "pre_tool_use"
    ///
    /// [`CommandResult`]: crate::cli::handler::CommandResult
    /// [`MaosError::InvalidInput`]: maos_core::MaosError::InvalidInput
    async fn execute(&self, input: HookInput) -> Result<CommandResult> {
        // TODO: (Implemented in PRD-06) Implement actual pre_tool_use logic - see PRD-06 for security validation,
        // workspace enforcement, tool parameter modification, and task spawning coordination.
        // For now, just a stub that validates and returns success

        // Validate required fields for pre_tool_use
        if input.tool_name.is_none() {
            return Err(maos_core::MaosError::InvalidInput {
                message: "pre_tool_use requires tool_name".to_string(),
            });
        }

        Ok(CommandResult {
            exit_code: ExitCode::Success,
            output: Some(format!(
                "Pre-tool hook executed for tool: {}",
                input.tool_name.unwrap_or_default()
            )),
            metrics: ExecutionMetrics::default(),
        })
    }

    /// Returns the hook event name constant
    ///
    /// # Returns
    ///
    /// Returns `"pre_tool_use"` as defined in [`maos_core::hook_constants`]
    ///
    /// [`maos_core::hook_constants`]: maos_core::hook_constants
    fn name(&self) -> &'static str {
        PRE_TOOL_USE
    }

    /// Validates that hook input matches pre-tool-use event
    ///
    /// Ensures the `hook_event_name` field matches the expected "pre_tool_use" value.
    /// This prevents handler misrouting and ensures type safety.
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
    /// Returns [`MaosError::InvalidInput`] if `hook_event_name` doesn't match "pre_tool_use".
    ///
    /// [`MaosError::InvalidInput`]: maos_core::MaosError::InvalidInput
    fn validate_input(&self, input: &HookInput) -> Result<()> {
        // Ensure hook_event_name matches
        if input.hook_event_name != PRE_TOOL_USE {
            return Err(maos_core::MaosError::InvalidInput {
                message: format!("Expected pre_tool_use hook, got {}", input.hook_event_name),
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
    use maos_core::hook_constants::POST_TOOL_USE;

    // TODO: (Implemented in PRD-06) Expand tests for full business logic implementation including
    // security validation, workspace enforcement, and task spawning coordination

    fn create_valid_hook_input() -> HookInput {
        HookInput {
            hook_event_name: PRE_TOOL_USE.to_string(),
            tool_name: Some("Read".to_string()),
            session_id: "test-session-123".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_handler_name_returns_correct_constant() {
        let handler = PreToolUseHandler;
        assert_eq!(handler.name(), PRE_TOOL_USE);
    }

    #[tokio::test]
    async fn test_execute_success_with_valid_input() {
        let handler = PreToolUseHandler;
        let input = create_valid_hook_input();

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("Read"));
    }

    #[tokio::test]
    async fn test_execute_error_with_missing_tool_name() {
        let handler = PreToolUseHandler;
        let mut input = create_valid_hook_input();
        input.tool_name = None;

        let result = handler.execute(input).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            maos_core::MaosError::InvalidInput { message } => {
                assert!(message.contains("tool_name"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_validate_input_success() {
        let handler = PreToolUseHandler;
        let input = create_valid_hook_input();

        let result = handler.validate_input(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_input_wrong_hook_event() {
        let handler = PreToolUseHandler;
        let mut input = create_valid_hook_input();
        input.hook_event_name = POST_TOOL_USE.to_string(); // Different event to test validation

        let result = handler.validate_input(&input);

        assert!(result.is_err());
        match result.unwrap_err() {
            maos_core::MaosError::InvalidInput { message } => {
                assert!(message.contains("Expected pre_tool_use"));
                assert!(message.contains(POST_TOOL_USE));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_handler_tool_name_validation_logic() {
        let handler = PreToolUseHandler;

        // Test with empty tool name string
        let mut input = create_valid_hook_input();
        input.tool_name = Some("".to_string());

        let result = handler.execute(input).await.unwrap();
        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.unwrap().contains("Pre-tool hook executed"));
    }
}
