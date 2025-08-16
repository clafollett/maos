//! Handler registry for managing command handlers

use crate::cli::{Commands, handler::CommandHandler};
use maos_core::config::MaosConfig;
use maos_core::{MaosError, Result};
use std::collections::HashMap;

/// Type alias for command handler storage
type HandlerMap = HashMap<String, Box<dyn CommandHandler>>;

/// Registry for command handlers with lazy initialization
pub struct HandlerRegistry {
    #[cfg(test)]
    pub handlers: HandlerMap,
    #[cfg(not(test))]
    handlers: HandlerMap,
}

impl HandlerRegistry {
    /// Build handler registry with all command handlers
    pub async fn build(_config: &MaosConfig) -> Result<Self> {
        let mut handlers = HashMap::new();

        // Register all handler implementations using constants
        use crate::cli::handlers::*;
        use maos_core::hook_constants::*;

        handlers.insert(
            PRE_TOOL_USE.to_string(),
            Box::new(PreToolUseHandler) as Box<dyn CommandHandler>,
        );
        handlers.insert(
            POST_TOOL_USE.to_string(),
            Box::new(PostToolUseHandler) as Box<dyn CommandHandler>,
        );
        handlers.insert(
            NOTIFICATION.to_string(),
            Box::new(NotificationHandler) as Box<dyn CommandHandler>,
        );
        handlers.insert(
            STOP.to_string(),
            Box::new(StopHandler) as Box<dyn CommandHandler>,
        );
        handlers.insert(
            SUBAGENT_STOP.to_string(),
            Box::new(SubagentStopHandler) as Box<dyn CommandHandler>,
        );
        handlers.insert(
            USER_PROMPT_SUBMIT.to_string(),
            Box::new(UserPromptSubmitHandler) as Box<dyn CommandHandler>,
        );
        handlers.insert(
            PRE_COMPACT.to_string(),
            Box::new(PreCompactHandler) as Box<dyn CommandHandler>,
        );
        handlers.insert(
            SESSION_START.to_string(),
            Box::new(SessionStartHandler) as Box<dyn CommandHandler>,
        );

        Ok(Self { handlers })
    }

    /// Get handler for specific command
    pub fn get_handler(&self, command: &Commands) -> Result<&dyn CommandHandler> {
        let key = command.hook_event_name();

        self.handlers
            .get(key)
            .map(|h| h.as_ref())
            .ok_or_else(|| MaosError::InvalidInput {
                message: format!("No handler found for command: {}", key),
            })
    }

    /// Register a handler (useful for testing)
    pub fn register(&mut self, key: String, handler: Box<dyn CommandHandler>) {
        self.handlers.insert(key, handler);
    }

    /// Get number of registered handlers
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::handler::{CommandResult, ExecutionMetrics};
    use crate::io::HookInput;
    use async_trait::async_trait;
    use maos_core::ExitCode;

    struct TestHandler {
        name: &'static str,
    }

    #[async_trait]
    impl CommandHandler for TestHandler {
        async fn execute(&self, _input: HookInput) -> Result<CommandResult> {
            Ok(CommandResult {
                exit_code: ExitCode::Success,
                output: None,
                metrics: ExecutionMetrics::default(),
            })
        }

        fn name(&self) -> &'static str {
            self.name
        }
    }

    #[tokio::test]
    async fn test_registry_build() {
        let config = MaosConfig::default();
        let registry = HandlerRegistry::build(&config).await.unwrap();

        // Should have all 8 handlers registered
        assert_eq!(registry.len(), 8);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_registry_get_handler() {
        let mut registry = HandlerRegistry {
            handlers: HashMap::new(),
        };

        // Register a test handler
        use maos_core::hook_constants::PRE_TOOL_USE;
        registry.register(
            PRE_TOOL_USE.to_string(),
            Box::new(TestHandler {
                name: "pre_tool_handler",
            }),
        );

        // Should find handler for PreToolUse command
        let command = Commands::PreToolUse;
        let handler = registry.get_handler(&command).unwrap();
        assert_eq!(handler.name(), "pre_tool_handler");

        // Should fail for unregistered command
        let command = Commands::PostToolUse;
        assert!(registry.get_handler(&command).is_err());
    }

    #[test]
    fn test_registry_lazy_initialization() {
        let mut registry = HandlerRegistry {
            handlers: HashMap::new(),
        };

        assert_eq!(registry.len(), 0);

        // Simulate lazy initialization
        use maos_core::hook_constants::NOTIFICATION;
        registry.register(
            NOTIFICATION.to_string(),
            Box::new(TestHandler {
                name: "notify_handler",
            }),
        );

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_registry_handler_lookup_performance() {
        use std::time::Instant;

        let mut registry = HandlerRegistry {
            handlers: HashMap::new(),
        };

        // Register all 8 handler types
        use maos_core::hook_constants::*;
        let handler_names = [
            PRE_TOOL_USE,
            POST_TOOL_USE,
            NOTIFICATION,
            STOP,
            SUBAGENT_STOP,
            USER_PROMPT_SUBMIT,
            PRE_COMPACT,
            SESSION_START,
        ];

        for name in &handler_names {
            registry.register(name.to_string(), Box::new(TestHandler { name }));
        }

        // Test lookup performance
        let command = Commands::PreToolUse;
        let start = Instant::now();

        for _ in 0..1000 {
            let _ = registry.get_handler(&command);
        }

        let elapsed = start.elapsed();
        let avg_lookup = elapsed / 1000;

        // Should be O(1) - well under 1 microsecond
        assert!(
            avg_lookup.as_nanos() < 1000,
            "Lookup too slow: {:?}",
            avg_lookup
        );
    }
}
