//! Command dispatcher for routing commands to handlers

use crate::cli::{Commands, registry::HandlerRegistry};
use crate::io::{HookInput, StdinProcessor};
use maos_core::config::MaosConfig;
use maos_core::{ExitCode, PerformanceMetrics, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(test)]
use async_trait::async_trait;

/// Trait for input providers (allows mocking in tests)
#[cfg(test)]
#[async_trait]
pub trait InputProvider: Send + Sync {
    async fn read_hook_input(&mut self) -> Result<HookInput>;
}

#[cfg(test)]
#[async_trait]
impl InputProvider for StdinProcessor {
    async fn read_hook_input(&mut self) -> Result<HookInput> {
        self.read_hook_input().await
    }
}

/// Command dispatcher that routes commands to appropriate handlers
pub struct CommandDispatcher {
    pub config: Arc<MaosConfig>,
    metrics: Arc<PerformanceMetrics>,
    pub registry: HandlerRegistry,
    #[cfg(test)]
    input_provider: Box<dyn InputProvider>,
    #[cfg(not(test))]
    stdin_processor: StdinProcessor,
}

impl CommandDispatcher {
    /// Create a new command dispatcher
    pub async fn new(config: Arc<MaosConfig>, metrics: Arc<PerformanceMetrics>) -> Result<Self> {
        let registry = HandlerRegistry::build(&config).await?;
        let stdin_processor = StdinProcessor::new(config.hooks.clone());

        Ok(Self {
            config,
            metrics,
            registry,
            #[cfg(test)]
            input_provider: Box::new(stdin_processor),
            #[cfg(not(test))]
            stdin_processor,
        })
    }

    /// Create dispatcher with mock input provider (for testing)
    #[cfg(test)]
    pub async fn new_with_input_provider(
        config: Arc<MaosConfig>,
        metrics: Arc<PerformanceMetrics>,
        input_provider: Box<dyn InputProvider>,
    ) -> Result<Self> {
        let registry = HandlerRegistry::build(&config).await?;

        Ok(Self {
            config,
            metrics,
            registry,
            input_provider,
        })
    }

    /// Dispatch command to appropriate handler
    pub async fn dispatch(&mut self, command: Commands) -> Result<ExitCode> {
        let start_time = Instant::now();

        // Read input if command expects it
        let input = if command.expects_stdin() {
            self.read_input().await?
        } else {
            HookInput::default()
        };

        // Get the appropriate handler
        let handler = self.registry.get_handler(&command)?;

        // Validate input
        let validation_start = Instant::now();
        handler.validate_input(&input)?;
        let validation_time = validation_start.elapsed();

        // Execute handler
        let handler_start = Instant::now();
        let mut result = handler.execute(input).await?;
        let handler_time = handler_start.elapsed();

        // Update metrics
        result.metrics.validation_time = validation_time;
        result.metrics.handler_time = handler_time;
        result.metrics.total_time = start_time.elapsed();

        // Record performance metrics
        self.record_metrics(handler.name(), result.metrics.total_time);

        // Output result if present
        if let Some(output) = result.output {
            println!("{}", output);
        }

        Ok(result.exit_code)
    }

    /// Read input from stdin
    async fn read_input(&mut self) -> Result<HookInput> {
        #[cfg(test)]
        {
            self.input_provider.read_hook_input().await
        }
        #[cfg(not(test))]
        {
            self.stdin_processor.read_hook_input().await
        }
    }

    /// Record metrics for the operation
    fn record_metrics(&self, handler_name: &str, duration: Duration) {
        self.metrics.record_execution_time(handler_name, duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::handler::{CommandHandler, CommandResult, ExecutionMetrics};
    use maos_core::MaosError;

    /// Mock input provider for testing
    struct MockInputProvider {
        input: HookInput,
        should_fail: bool,
    }

    #[async_trait]
    impl InputProvider for MockInputProvider {
        async fn read_hook_input(&mut self) -> Result<HookInput> {
            if self.should_fail {
                return Err(MaosError::InvalidInput {
                    message: "Mock input failure".to_string(),
                });
            }
            Ok(self.input.clone())
        }
    }

    struct TestHandler {
        name: &'static str,
        exit_code: ExitCode,
    }

    #[async_trait]
    impl CommandHandler for TestHandler {
        async fn execute(&self, _input: HookInput) -> Result<CommandResult> {
            Ok(CommandResult {
                exit_code: self.exit_code,
                output: Some(format!("Handled by {}", self.name)),
                metrics: ExecutionMetrics::default(),
            })
        }

        fn name(&self) -> &'static str {
            self.name
        }
    }

    #[tokio::test]
    async fn test_dispatcher_routing() {
        let config = Arc::new(MaosConfig::default());
        let metrics = Arc::new(PerformanceMetrics::new());

        // Create mock input
        let mock_input = HookInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/test.jsonl".into(),
            cwd: "/tmp".into(),
            hook_event_name: maos_core::hook_constants::PRE_TOOL_USE.to_string(),
            tool_name: Some("Bash".to_string()),
            tool_input: Some(serde_json::json!({"command": "ls"})),
            ..Default::default()
        };

        let input_provider = Box::new(MockInputProvider {
            input: mock_input,
            should_fail: false,
        });

        // Create dispatcher with mock input
        let mut dispatcher =
            CommandDispatcher::new_with_input_provider(config, metrics, input_provider)
                .await
                .unwrap();

        // Replace handler with test handler
        dispatcher.registry.register(
            maos_core::hook_constants::PRE_TOOL_USE.to_string(),
            Box::new(TestHandler {
                name: "pre_tool_handler",
                exit_code: ExitCode::Success,
            }),
        );

        // Dispatch should route to correct handler and read mock input
        let command = Commands::PreToolUse;
        let exit_code = dispatcher.dispatch(command).await.unwrap();
        assert_eq!(exit_code, ExitCode::Success);
    }

    #[tokio::test]
    async fn test_dispatcher_metrics_collection() {
        let config = Arc::new(MaosConfig::default());
        let metrics = Arc::new(PerformanceMetrics::new());

        let mock_input = HookInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/test.jsonl".into(),
            cwd: "/tmp".into(),
            hook_event_name: maos_core::hook_constants::NOTIFICATION.to_string(),
            message: Some("Test notification".to_string()),
            ..Default::default()
        };

        let input_provider = Box::new(MockInputProvider {
            input: mock_input,
            should_fail: false,
        });

        let mut dispatcher =
            CommandDispatcher::new_with_input_provider(config, metrics.clone(), input_provider)
                .await
                .unwrap();

        dispatcher.registry.register(
            maos_core::hook_constants::NOTIFICATION.to_string(),
            Box::new(TestHandler {
                name: "notify_handler",
                exit_code: ExitCode::Success,
            }),
        );

        // Execute command through full dispatch flow
        let command = Commands::Notify;
        dispatcher.dispatch(command).await.unwrap();

        // Metrics should be recorded
        let report = metrics.export_metrics();
        assert!(!report.execution_stats.is_empty());
    }

    #[tokio::test]
    async fn test_dispatcher_error_handling() {
        let config = Arc::new(MaosConfig::default());
        let metrics = Arc::new(PerformanceMetrics::new());

        // Test input provider failure
        let input_provider = Box::new(MockInputProvider {
            input: HookInput::default(),
            should_fail: true,
        });

        let mut dispatcher = CommandDispatcher::new_with_input_provider(
            config.clone(),
            metrics.clone(),
            input_provider,
        )
        .await
        .unwrap();

        let command = Commands::PreToolUse;
        let result = dispatcher.dispatch(command).await;
        assert!(result.is_err());

        // Test missing handler
        let input_provider = Box::new(MockInputProvider {
            input: HookInput::default(),
            should_fail: false,
        });

        let mut dispatcher =
            CommandDispatcher::new_with_input_provider(config, metrics, input_provider)
                .await
                .unwrap();

        let command = Commands::Stop { chat: false };
        let result = dispatcher.dispatch(command).await;
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, MaosError::InvalidInput { .. }));
        }
    }

    #[tokio::test]
    async fn test_dispatcher_async_execution() {
        use tokio::time::sleep;

        struct AsyncHandler;

        #[async_trait]
        impl CommandHandler for AsyncHandler {
            async fn execute(&self, _input: HookInput) -> Result<CommandResult> {
                // Simulate async work
                sleep(Duration::from_millis(10)).await;

                Ok(CommandResult {
                    exit_code: ExitCode::Success,
                    output: None,
                    metrics: ExecutionMetrics::default(),
                })
            }

            fn name(&self) -> &'static str {
                "async_handler"
            }
        }

        let config = Arc::new(MaosConfig::default());
        let metrics = Arc::new(PerformanceMetrics::new());

        let mock_input = HookInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/test.jsonl".into(),
            cwd: "/tmp".into(),
            hook_event_name: maos_core::hook_constants::SESSION_START.to_string(),
            ..Default::default()
        };

        let input_provider = Box::new(MockInputProvider {
            input: mock_input,
            should_fail: false,
        });

        let mut dispatcher =
            CommandDispatcher::new_with_input_provider(config, metrics, input_provider)
                .await
                .unwrap();

        dispatcher.registry.register(
            maos_core::hook_constants::SESSION_START.to_string(),
            Box::new(AsyncHandler),
        );

        let start = Instant::now();
        let command = Commands::SessionStart;
        dispatcher.dispatch(command).await.unwrap();
        let elapsed = start.elapsed();

        // Should have taken at least 10ms due to async sleep
        assert!(elapsed >= Duration::from_millis(10));
    }
}
