//! Handler for user_prompt_submit hook events

use crate::cli::handler::{CommandHandler, CommandResult, ExecutionMetrics};
use crate::io::HookInput;
use async_trait::async_trait;
use maos_core::{ExitCode, Result, hook_constants::USER_PROMPT_SUBMIT};

/// Handler for user prompt submission hook events from Claude Code
///
/// This handler processes events when users submit prompts to Claude Code.
/// Currently provides basic message processing and placeholder logic.
/// Full implementation planned for PRD-06 will include:
///
/// - Prompt validation and security analysis
/// - Context analysis and intent detection
/// - Intelligent task routing and agent selection
/// - User interaction pattern analysis
///
/// # Hook Event
///
/// Responds to `user_prompt_submit` events with optional `message` field containing
/// the user's prompt text for analysis and processing.
///
/// # Example
///
/// ```rust,no_run
/// use maos::cli::handlers::UserPromptSubmitHandler;
/// use maos::cli::handler::CommandHandler;
/// use maos::io::HookInput;
///
/// # async fn example() -> maos_core::Result<()> {
/// let handler = UserPromptSubmitHandler;
/// let input = HookInput {
///     hook_event_name: "user_prompt_submit".to_string(),
///     message: Some("Please analyze this code for security issues".to_string()),
///     ..Default::default()
/// };
///
/// let result = handler.execute(input).await?;
/// assert_eq!(result.exit_code, maos_core::ExitCode::Success);
/// # Ok(())
/// # }
/// ```
pub struct UserPromptSubmitHandler;

#[async_trait]
impl CommandHandler for UserPromptSubmitHandler {
    /// Execute user prompt submission processing
    ///
    /// Currently processes the prompt message and returns success.
    /// Future PRD-06 implementation will add prompt validation,
    /// context analysis, and intelligent task routing.
    ///
    /// # Arguments
    ///
    /// * `input` - Hook input from Claude Code containing user prompt
    ///
    /// # Returns
    ///
    /// Returns [`CommandResult`] with success status and processed prompt,
    /// including fallback handling for missing prompts.
    ///
    /// # Errors
    ///
    /// Currently always succeeds. Future implementation may return errors for
    /// prompt validation failures or security analysis rejections.
    ///
    /// [`CommandResult`]: crate::cli::handler::CommandResult
    async fn execute(&self, input: HookInput) -> Result<CommandResult> {
        // TODO: (Implemented in PRD-06) Implement actual user_prompt_submit logic - see PRD-06 for prompt
        // validation, context analysis, and intelligent task routing.
        // For now, just a stub that validates and returns success

        let message = input
            .message
            .unwrap_or_else(|| "No prompt provided".to_string());

        Ok(CommandResult {
            exit_code: ExitCode::Success,
            output: Some(format!("User prompt processed: {message}")),
            metrics: ExecutionMetrics::default(),
        })
    }

    /// Returns the hook event name constant
    ///
    /// # Returns
    ///
    /// Returns `"user_prompt_submit"` as defined in [`maos_core::hook_constants`]
    ///
    /// [`maos_core::hook_constants`]: maos_core::hook_constants
    fn name(&self) -> &'static str {
        USER_PROMPT_SUBMIT
    }

    /// Validates that hook input matches user_prompt_submit event
    ///
    /// Ensures the `hook_event_name` field matches the expected "user_prompt_submit" value.
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
    /// Returns [`MaosError::InvalidInput`] if `hook_event_name` doesn't match "user_prompt_submit".
    ///
    /// [`MaosError::InvalidInput`]: maos_core::MaosError::InvalidInput
    fn validate_input(&self, input: &HookInput) -> Result<()> {
        // Ensure hook_event_name matches
        if input.hook_event_name != USER_PROMPT_SUBMIT {
            return Err(maos_core::MaosError::InvalidInput {
                message: format!(
                    "Expected user_prompt_submit hook, got {}",
                    input.hook_event_name
                ),
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
    // prompt validation, context analysis, and intelligent task routing

    fn create_valid_hook_input() -> HookInput {
        HookInput {
            hook_event_name: USER_PROMPT_SUBMIT.to_string(),
            message: Some("Test user prompt message".to_string()),
            session_id: "test-session-prompt".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_handler_name_returns_correct_constant() {
        let handler = UserPromptSubmitHandler;
        assert_eq!(handler.name(), USER_PROMPT_SUBMIT);
    }

    #[tokio::test]
    async fn test_execute_success_with_valid_input() {
        let handler = UserPromptSubmitHandler;
        let input = create_valid_hook_input();

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("Test user prompt message"));
    }

    #[tokio::test]
    async fn test_execute_success_with_empty_message() {
        let handler = UserPromptSubmitHandler;
        let mut input = create_valid_hook_input();
        input.message = None;

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("No prompt provided"));
    }

    #[tokio::test]
    async fn test_validate_input_success() {
        let handler = UserPromptSubmitHandler;
        let input = create_valid_hook_input();

        let result = handler.validate_input(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_input_wrong_hook_event() {
        let handler = UserPromptSubmitHandler;
        let mut input = create_valid_hook_input();
        input.hook_event_name = "pre_compact".to_string(); // Different event to test validation

        let result = handler.validate_input(&input);

        assert!(result.is_err());
        match result.unwrap_err() {
            maos_core::MaosError::InvalidInput { message } => {
                assert!(message.contains("Expected user_prompt_submit"));
                assert!(message.contains("pre_compact"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }
}
