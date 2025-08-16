//! Handler registry for managing command handlers

use crate::cli::{Commands, handler::CommandHandler};
use dashmap::DashMap;
use maos_core::config::MaosConfig;
use maos_core::{MaosError, Result};

/// Type alias for thread-safe command handler storage
///
/// 🔥 CRITICAL FIX: Using DashMap instead of HashMap for thread safety
/// HashMap would cause data races in concurrent access scenarios
type HandlerMap = DashMap<String, Box<dyn CommandHandler>>;

/// Type alias for DashMap reference to reduce type complexity
type HandlerRef<'a> = dashmap::mapref::one::Ref<'a, String, Box<dyn CommandHandler>>;

/// Registry for command handlers with thread-safe concurrent access
///
/// 🔥 CRITICAL FIX: No more conditional compilation anti-pattern
/// Using DashMap provides both thread safety and testing access
pub struct HandlerRegistry {
    handlers: HandlerMap,
}

impl HandlerRegistry {
    /// Build handler registry with all command handlers
    pub async fn build(_config: &MaosConfig) -> Result<Self> {
        let handlers = DashMap::new();

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

    /// Get handler for specific command (thread-safe concurrent access)
    ///
    /// 🔥 CRITICAL FIX: Now uses DashMap for safe concurrent access
    /// Returns a reference that's safe to use from multiple threads
    pub fn get_handler(&self, command: &Commands) -> Result<HandlerRef<'_>> {
        let key = command.hook_event_name();

        self.handlers
            .get(key)
            .ok_or_else(|| MaosError::InvalidInput {
                message: format!("No handler found for command: {key}"),
            })
    }

    /// Register a handler (useful for testing)
    ///
    /// 🔥 CRITICAL FIX: No longer needs &mut self - DashMap supports concurrent writes
    pub fn register(&self, key: String, handler: Box<dyn CommandHandler>) {
        self.handlers.insert(key, handler);
    }

    /// Get the total number of registered command handlers
    ///
    /// # Returns
    ///
    /// The count of handlers currently stored in the registry. In a fully
    /// initialized registry, this should be 8 (one for each Claude Code hook event).
    ///
    /// # Thread Safety
    ///
    /// This method is thread-safe and can be called concurrently from multiple
    /// threads without any synchronization overhead.
    ///
    /// # Example
    ///
    /// ```rust
    /// use maos::cli::HandlerRegistry;
    /// use maos_core::config::MaosConfig;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = MaosConfig::default();
    ///     let registry = HandlerRegistry::build(&config).await.unwrap();
    ///     
    ///     assert_eq!(registry.len(), 8); // All 8 hook handlers registered
    /// }
    /// ```
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Check if the handler registry contains no registered handlers
    ///
    /// # Returns
    ///
    /// `true` if the registry contains zero handlers, `false` otherwise.
    /// A properly initialized registry should never be empty.
    ///
    /// # Thread Safety
    ///
    /// This method is thread-safe and can be called concurrently from multiple
    /// threads without any synchronization overhead.
    ///
    /// # Example
    ///
    /// ```rust
    /// use maos::cli::HandlerRegistry;
    /// use maos_core::config::MaosConfig;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = MaosConfig::default();
    ///     let registry = HandlerRegistry::build(&config).await.unwrap();
    ///     
    ///     assert!(!registry.is_empty()); // Should have handlers
    /// }
    /// ```
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
        let registry = HandlerRegistry {
            handlers: DashMap::new(),
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
        let handler_ref = registry.get_handler(&command).unwrap();
        assert_eq!(handler_ref.name(), "pre_tool_handler");

        // Should fail for unregistered command
        let command = Commands::PostToolUse;
        assert!(registry.get_handler(&command).is_err());
    }

    #[test]
    fn test_registry_lazy_initialization() {
        let registry = HandlerRegistry {
            handlers: DashMap::new(),
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

        let registry = HandlerRegistry {
            handlers: DashMap::new(),
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
            "Lookup too slow: {avg_lookup:?}"
        );
    }
}
