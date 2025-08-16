//! Handler for notification hook events

use crate::cli::handler::{CommandHandler, CommandResult, ExecutionMetrics};
use crate::io::HookInput;
use async_trait::async_trait;
use maos_core::{ExitCode, Result, hook_constants::NOTIFICATION};

/// Handler for notification display hook events from Claude Code
///
/// This handler processes notification messages that should be displayed to the user.
/// Currently provides basic message processing and placeholder logic.
/// Full implementation planned for PRD-06 will include:
///
/// - Text-to-Speech (TTS) integration for audio notifications
/// - Notification urgency level handling and prioritization
/// - Message formatting and templating system
/// - Integration with system notification frameworks
///
/// # Hook Event
///
/// Responds to `notification` events with optional `message` field containing
/// the notification content to display to the user.
///
/// # Example
///
/// ```rust,no_run
/// use maos::cli::handlers::NotificationHandler;
/// use maos::cli::handler::CommandHandler;
/// use maos::io::HookInput;
///
/// # async fn example() -> maos_core::Result<()> {
/// let handler = NotificationHandler;
/// let input = HookInput {
///     hook_event_name: "notification".to_string(),
///     message: Some("Build completed successfully!".to_string()),
///     ..Default::default()
/// };
///
/// let result = handler.execute(input).await?;
/// assert_eq!(result.exit_code, maos_core::ExitCode::Success);
/// # Ok(())
/// # }
/// ```
pub struct NotificationHandler;

#[async_trait]
impl CommandHandler for NotificationHandler {
    /// Execute notification display processing
    ///
    /// Currently processes the notification message and returns success.
    /// Future PRD-06 implementation will add TTS integration, urgency
    /// handling, and system notification display.
    ///
    /// # Arguments
    ///
    /// * `input` - Hook input from Claude Code containing notification message
    ///
    /// # Returns
    ///
    /// Returns [`CommandResult`] with success status and processed message,
    /// including fallback handling for missing messages.
    ///
    /// # Errors
    ///
    /// Currently always succeeds. Future implementation may return errors for
    /// TTS failures or system notification delivery issues.
    ///
    /// [`CommandResult`]: crate::cli::handler::CommandResult
    async fn execute(&self, input: HookInput) -> Result<CommandResult> {
        // TODO: (Implemented in PRD-06) Implement actual notification logic - see PRD-06 for TTS integration,
        // notification urgency handling, and message formatting.
        // For now, just a stub that validates and returns success

        // Process notification message
        let message = input
            .message
            .unwrap_or_else(|| "No message provided".to_string());

        Ok(CommandResult {
            exit_code: ExitCode::Success,
            output: Some(format!("Notification processed: {message}")),
            metrics: ExecutionMetrics::default(),
        })
    }

    /// Returns the hook event name constant
    ///
    /// # Returns
    ///
    /// Returns `"notification"` as defined in [`maos_core::hook_constants`]
    ///
    /// [`maos_core::hook_constants`]: maos_core::hook_constants
    fn name(&self) -> &'static str {
        NOTIFICATION
    }

    /// Validates that hook input matches notification event
    ///
    /// Ensures the `hook_event_name` field matches the expected "notification" value.
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
    /// Returns [`MaosError::InvalidInput`] if `hook_event_name` doesn't match "notification".
    ///
    /// [`MaosError::InvalidInput`]: maos_core::MaosError::InvalidInput
    fn validate_input(&self, input: &HookInput) -> Result<()> {
        // Ensure hook_event_name matches
        if input.hook_event_name != NOTIFICATION {
            return Err(maos_core::MaosError::InvalidInput {
                message: format!("Expected notification hook, got {}", input.hook_event_name),
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
    use maos_core::hook_constants::STOP;

    // TODO: (Implemented in PRD-06) Expand tests for full business logic implementation including
    // TTS integration, urgency handling, and system notification display

    fn create_valid_hook_input() -> HookInput {
        HookInput {
            hook_event_name: NOTIFICATION.to_string(),
            message: Some("Test notification message".to_string()),
            session_id: "test-session-789".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_handler_name_returns_correct_constant() {
        let handler = NotificationHandler;
        assert_eq!(handler.name(), NOTIFICATION);
    }

    #[tokio::test]
    async fn test_execute_success_with_valid_input() {
        let handler = NotificationHandler;
        let input = create_valid_hook_input();

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("Test notification message"));
    }

    #[tokio::test]
    async fn test_execute_success_with_empty_message() {
        let handler = NotificationHandler;
        let mut input = create_valid_hook_input();
        input.message = None;

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("No message provided"));
    }

    #[tokio::test]
    async fn test_validate_input_success() {
        let handler = NotificationHandler;
        let input = create_valid_hook_input();

        let result = handler.validate_input(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_input_wrong_hook_event() {
        let handler = NotificationHandler;
        let mut input = create_valid_hook_input();
        input.hook_event_name = STOP.to_string(); // Different event to test validation

        let result = handler.validate_input(&input);

        assert!(result.is_err());
        match result.unwrap_err() {
            maos_core::MaosError::InvalidInput { message } => {
                assert!(message.contains("Expected notification"));
                assert!(message.contains(STOP));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_message_processing_with_special_characters() {
        let handler = NotificationHandler;
        let mut input = create_valid_hook_input();
        input.message = Some("Test with émojis 🎉 and special chars: <>&\"'".to_string());

        let result = handler.execute(input).await.unwrap();

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.output.unwrap().contains("émojis 🎉"));
    }
}
