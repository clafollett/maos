//! Command dispatcher for routing commands to handlers

use crate::cli::{Commands, handler::CommandResult, registry::HandlerRegistry};
use crate::io::{HookInput, StdinProcessor};
use maos_core::config::MaosConfig;
use maos_core::{MaosError, PerformanceMetrics, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use async_trait::async_trait;

/// Type alias for async-safe input provider to reduce type complexity
/// Uses tokio::sync::Mutex for proper async compatibility
type ThreadSafeInputProvider = Arc<Mutex<Box<dyn InputProvider>>>;

/// 🔥 CRITICAL FIX: Always available trait for input providers (clean architecture)
///
/// This trait abstracts input reading to allow proper dependency injection
/// instead of conditional compilation anti-patterns.
#[async_trait]
pub trait InputProvider: Send + Sync {
    async fn read_hook_input(&mut self) -> Result<HookInput>;
}

/// Production implementation using actual stdin
#[async_trait]
impl InputProvider for StdinProcessor {
    async fn read_hook_input(&mut self) -> Result<HookInput> {
        self.read_hook_input().await
    }
}

/// Command dispatcher that routes commands to appropriate handlers
///
/// 🔥 CRITICAL FIX: Immutable dispatcher with thread-safe input provider
/// Clean architecture with dependency injection and no mutable state
pub struct CommandDispatcher {
    pub config: Arc<MaosConfig>,
    metrics: Arc<PerformanceMetrics>,
    pub registry: HandlerRegistry,
    input_provider: ThreadSafeInputProvider,
}

impl CommandDispatcher {
    /// Create a new command dispatcher with production stdin processor
    ///
    /// 🔥 CRITICAL FIX: Clean constructor using dependency injection
    pub async fn new(config: Arc<MaosConfig>, metrics: Arc<PerformanceMetrics>) -> Result<Self> {
        let registry = HandlerRegistry::build(&config).await?;
        let stdin_processor = StdinProcessor::new(config.hooks.clone());

        Ok(Self {
            config,
            metrics,
            registry,
            input_provider: Arc::new(Mutex::new(Box::new(stdin_processor))),
        })
    }

    /// Create dispatcher with custom input provider
    ///
    /// 🔥 CRITICAL FIX: Always available for clean dependency injection
    /// This enables both testing with mocks and custom input sources
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
            input_provider: Arc::new(Mutex::new(input_provider)),
        })
    }

    /// Dispatch command to appropriate handler
    ///
    /// 🔥 CRITICAL FIX: Now immutable - no &mut self needed for thread safety
    /// ✅ STDOUT CONTROL REMOVED: Returns full CommandResult with optional output
    pub async fn dispatch(&self, command: Commands) -> Result<CommandResult> {
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

        // Execute handler with timeout protection
        let handler_start = Instant::now();
        let execution_timeout = Duration::from_millis(self.config.system.max_execution_time_ms);

        // 🛡️ RESOURCE LIMIT: Enforce maximum execution time to prevent runaway handlers
        let mut result = match tokio::time::timeout(execution_timeout, handler.execute(input)).await
        {
            Ok(result) => result?,
            Err(_timeout) => {
                return Err(MaosError::ResourceLimit {
                    resource: "execution_time".to_string(),
                    limit: self.config.system.max_execution_time_ms,
                    actual: execution_timeout.as_millis() as u64,
                    message: format!(
                        "Handler execution exceeded maximum time limit of {}ms",
                        self.config.system.max_execution_time_ms
                    ),
                });
            }
        };
        let handler_time = handler_start.elapsed();

        // 🛡️ RESOURCE LIMIT: Check memory usage after handler execution
        let memory_usage = StdinProcessor::get_memory_usage();
        let memory_limit_mb = self.config.hooks.max_input_size_mb; // Reuse input limit for memory
        let memory_limit_bytes = (memory_limit_mb * 1024 * 1024) as usize;

        if memory_usage > memory_limit_bytes {
            tracing::warn!(
                "High memory usage detected after handler execution: {} bytes ({}% of {}MB limit)",
                memory_usage,
                (memory_usage * 100) / memory_limit_bytes,
                memory_limit_mb
            );
            // Note: We log but don't fail here since memory cleanup is async
            // Future enhancement could add stricter memory enforcement
        }

        // Update metrics
        result.metrics.validation_time = validation_time;
        result.metrics.handler_time = handler_time;
        result.metrics.total_time = start_time.elapsed();

        // Record performance metrics
        self.record_metrics(handler.name(), result.metrics.total_time);

        // ✅ STDOUT CONTROL REMOVED: Return full result to caller
        // Caller can extract output and decide how to handle it (stdout, logging, etc.)
        if let Some(ref output) = result.output {
            tracing::debug!("Handler produced output: {} chars", output.len());
        }

        Ok(result)
    }

    /// Read input using dependency-injected provider
    ///
    /// 🔥 CRITICAL FIX: Async-safe access with tokio::sync::Mutex
    async fn read_input(&self) -> Result<HookInput> {
        let mut guard = self.input_provider.lock().await;
        guard.read_hook_input().await
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
    use maos_core::{ExitCode, MaosError};

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
        let dispatcher =
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
        let result = dispatcher.dispatch(command).await.unwrap();
        assert_eq!(result.exit_code, ExitCode::Success);
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

        let dispatcher =
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

        let dispatcher = CommandDispatcher::new_with_input_provider(
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

        let dispatcher =
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

        let dispatcher =
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

    #[tokio::test]
    async fn test_execution_timeout_enforcement() {
        // 🛡️ RESOURCE LIMIT TEST: Handlers exceeding max_execution_time_ms are terminated
        let mut config = MaosConfig::default();
        config.system.max_execution_time_ms = 100; // Very short timeout
        let config = Arc::new(config);
        let metrics = Arc::new(PerformanceMetrics::new());

        let mock_input = HookInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/test.jsonl".into(),
            cwd: "/tmp".into(),
            hook_event_name: maos_core::hook_constants::SESSION_START.to_string(),
            ..Default::default()
        };

        // Create a handler that takes longer than the timeout
        struct SlowHandler;
        #[async_trait]
        impl CommandHandler for SlowHandler {
            async fn execute(&self, _input: HookInput) -> Result<CommandResult> {
                // Sleep for longer than the 100ms timeout
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(CommandResult {
                    exit_code: ExitCode::Success,
                    output: Some("Should not reach here".to_string()),
                    metrics: ExecutionMetrics::default(),
                })
            }
            fn name(&self) -> &'static str {
                "slow_handler"
            }
        }

        let input_provider = Box::new(MockInputProvider {
            input: mock_input,
            should_fail: false,
        });

        let dispatcher = CommandDispatcher::new_with_input_provider(
            config.clone(),
            metrics.clone(),
            input_provider,
        )
        .await
        .unwrap();

        // Replace with slow handler
        dispatcher.registry.register(
            maos_core::hook_constants::SESSION_START.to_string(),
            Box::new(SlowHandler),
        );

        // Execute should timeout and return ResourceLimit error
        let command = Commands::SessionStart;
        let result = dispatcher.dispatch(command).await;

        assert!(result.is_err());
        if let Err(MaosError::ResourceLimit {
            resource,
            limit,
            message,
            ..
        }) = result
        {
            assert_eq!(resource, "execution_time");
            assert_eq!(limit, 100);
            assert!(message.contains("exceeded maximum time limit"));
        } else {
            panic!("Expected ResourceLimit error, got: {:?}", result);
        }
    }
}
