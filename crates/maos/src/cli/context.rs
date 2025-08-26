//! CLI context for dependency injection and shared resources

use crate::cli::{Commands, dispatcher::CommandDispatcher};
use maos_core::config::MaosConfig;
use maos_core::error::ConfigError;
use maos_core::{ExitCode, MaosError, PerformanceMetrics, Result};
use std::sync::{Arc, OnceLock};

/// Dependency container for CLI operations with lazy initialization
pub struct CliContext {
    /// Lazily loaded configuration
    config: OnceLock<Arc<MaosConfig>>,
    /// Lazily loaded metrics
    metrics: OnceLock<Arc<PerformanceMetrics>>,
    /// Lazily loaded dispatcher
    dispatcher: OnceLock<CommandDispatcher>,
}

impl CliContext {
    /// Build CLI context with lazy initialization
    pub async fn build() -> Result<Self> {
        Ok(Self {
            config: OnceLock::new(),
            metrics: OnceLock::new(),
            dispatcher: OnceLock::new(),
        })
    }

    /// Build CLI context with custom configuration (useful for testing)
    pub async fn build_with_config(config: MaosConfig) -> Result<Self> {
        let context = Self {
            config: OnceLock::new(),
            metrics: OnceLock::new(),
            dispatcher: OnceLock::new(),
        };
        // Pre-initialize with the provided config for testing
        let _ = context.config.set(Arc::new(config));
        Ok(context)
    }

    /// Build CLI context with test dispatcher (for unit testing)
    #[cfg(test)]
    pub(crate) async fn build_with_test_dispatcher(
        config: MaosConfig,
        dispatcher: CommandDispatcher,
    ) -> Result<Self> {
        let context = Self {
            config: OnceLock::new(),
            metrics: OnceLock::new(),
            dispatcher: OnceLock::new(),
        };
        let _ = context.config.set(Arc::new(config));
        let _ = context.dispatcher.set(dispatcher);
        Ok(context)
    }

    /// Execute a command and return the exit code with lazy initialization
    pub async fn execute(self, command: Commands) -> ExitCode {
        // Initialize dispatcher only when needed
        match self.get_dispatcher().await {
            Ok(dispatcher) => match dispatcher.dispatch(command).await {
                Ok(result) => {
                    if let Some(output) = result.output {
                        print!("{output}");
                    }
                    result.exit_code
                }
                Err(err) => {
                    tracing::error!("Command execution failed: {err:?}");
                    tracing::warn!("Check application logs for detailed error information");
                    ExitCode::from(&err)
                }
            },
            Err(err) => {
                tracing::error!("MAOS initialization failed: {err:?}");
                ExitCode::from(&err)
            }
        }
    }

    /// Get or initialize configuration
    /// Uses atomic initialization to prevent race conditions
    fn get_config(&self) -> Result<Arc<MaosConfig>> {
        // Fast path: already initialized
        if let Some(config) = self.config.get() {
            return Ok(config.clone());
        }

        // Slow path: initialize with proper synchronization
        let new_config = Arc::new(MaosConfig::load()?);

        // set() is atomic - only one thread will succeed
        match self.config.set(new_config.clone()) {
            Ok(()) => Ok(new_config),
            Err(_) => {
                // Another thread won the race, use their config
                Ok(self
                    .config
                    .get()
                    .expect("Config must be set after race")
                    .clone())
            }
        }
    }

    /// Get or initialize metrics
    fn get_metrics(&self) -> Arc<PerformanceMetrics> {
        self.metrics
            .get_or_init(|| Arc::new(PerformanceMetrics::new()))
            .clone()
    }

    /// Get or initialize dispatcher
    /// Uses atomic initialization to prevent race conditions
    async fn get_dispatcher(&self) -> Result<&CommandDispatcher> {
        // Check if already initialized first (fast path)
        if let Some(dispatcher) = self.dispatcher.get() {
            return Ok(dispatcher);
        }

        // Slow path: initialize with proper synchronization
        // Note: We can't use get_or_try_init directly because it doesn't support async closures
        // Instead, we use a double-checked locking pattern with OnceLock's atomic guarantees
        let config = self.get_config()?;
        let metrics = self.get_metrics();
        let new_dispatcher = CommandDispatcher::new(config, metrics).await?;

        // set() returns Ok if this thread won the race, Err if another thread already set it
        match self.dispatcher.set(new_dispatcher) {
            Ok(()) => {
                // We successfully initialized it
                self.dispatcher.get().ok_or_else(|| {
                    MaosError::Config(ConfigError::InvalidFormat {
                        reason: "Failed to initialize dispatcher".to_string(),
                    })
                })
            }
            Err(_) => {
                // Another thread initialized it, use theirs
                self.dispatcher.get().ok_or_else(|| {
                    MaosError::Config(ConfigError::InvalidFormat {
                        reason: "Failed to initialize dispatcher".to_string(),
                    })
                })
            }
        }
    }

    /// Get a reference to the configuration (for testing)
    pub fn config(&self) -> Result<Arc<MaosConfig>> {
        self.get_config()
    }

    /// Get a reference to the metrics (for testing)
    pub fn metrics(&self) -> Arc<PerformanceMetrics> {
        self.get_metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::dispatcher::{CommandDispatcher, InputProvider};
    use crate::io::HookInput;
    use async_trait::async_trait;

    /// Mock input provider for testing
    struct MockInputProvider {
        input: HookInput,
    }

    #[async_trait]
    impl InputProvider for MockInputProvider {
        async fn read_hook_input(&mut self) -> Result<HookInput> {
            Ok(self.input.clone())
        }
    }

    /// Create a test dispatcher with mock input
    async fn create_test_dispatcher(
        config: Arc<MaosConfig>,
        metrics: Arc<PerformanceMetrics>,
    ) -> CommandDispatcher {
        let mock_input = HookInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/test.jsonl".into(),
            cwd: "/tmp".into(),
            hook_event_name: "test_event".to_string(),
            ..Default::default()
        };

        let input_provider = Box::new(MockInputProvider { input: mock_input });

        CommandDispatcher::new_with_input_provider(config, metrics, input_provider)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_context_build() {
        let context = CliContext::build().await.unwrap();

        // Components should be lazy - not initialized yet
        assert!(context.config.get().is_none());
        assert!(context.metrics.get().is_none());
        assert!(context.dispatcher.get().is_none());
    }

    #[tokio::test]
    async fn test_context_shared_resources() {
        let config = MaosConfig::default();
        let context = CliContext::build_with_config(config).await.unwrap();

        // After accessing config, it should be initialized
        let config_ref1 = context.config().unwrap();
        let config_ref2 = context.config().unwrap();
        // Should be the same Arc instance
        assert!(Arc::ptr_eq(&config_ref1, &config_ref2));

        let metrics_ref1 = context.metrics();
        let metrics_ref2 = context.metrics();
        // Should be the same Arc instance
        assert!(Arc::ptr_eq(&metrics_ref1, &metrics_ref2));
    }

    #[tokio::test]
    async fn test_context_config_loading() {
        let mut custom_config = MaosConfig::default();
        custom_config.system.max_execution_time_ms = 5000;

        let context = CliContext::build_with_config(custom_config).await.unwrap();

        // Should use the provided configuration
        assert_eq!(context.config().unwrap().system.max_execution_time_ms, 5000);
    }

    #[tokio::test]
    async fn test_context_handler_initialization() {
        let config = MaosConfig::default();
        let config_arc = Arc::new(config.clone());
        let metrics = Arc::new(PerformanceMetrics::new());

        // Create a test dispatcher with mock input
        let dispatcher = create_test_dispatcher(config_arc.clone(), metrics.clone()).await;

        // Create context with the test dispatcher
        let context = CliContext::build_with_test_dispatcher(config, dispatcher)
            .await
            .unwrap();

        // Dispatcher should be pre-initialized in test
        assert!(context.dispatcher.get().is_some());

        // After getting the dispatcher, it should work
        let dispatcher = context.get_dispatcher().await.unwrap();
        assert_eq!(dispatcher.registry.len(), 8);

        // Test that we can get a handler
        let command = Commands::PreCompact;
        let handler = dispatcher.registry.get_handler(&command).unwrap();
        assert_eq!(handler.name(), maos_core::hook_constants::PRE_COMPACT);
    }

    #[tokio::test]
    async fn test_context_once_lock_consistency() {
        // Test that OnceLock ensures single initialization even under concurrent access
        let config = MaosConfig::default();
        let context = Arc::new(CliContext::build_with_config(config).await.unwrap());

        let mut handles = vec![];
        for _ in 0..10 {
            let ctx = context.clone();
            let handle = tokio::spawn(async move {
                // Multiple concurrent accesses should return the same instance
                let config = ctx.config().unwrap();
                let metrics = ctx.metrics();
                (config, metrics)
            });
            handles.push(handle);
        }

        let mut configs = vec![];
        let mut metrics = vec![];
        for handle in handles {
            let (cfg, met) = handle.await.unwrap();
            configs.push(cfg);
            metrics.push(met);
        }

        // All should be the same Arc instance
        for i in 1..configs.len() {
            assert!(Arc::ptr_eq(&configs[0], &configs[i]));
            assert!(Arc::ptr_eq(&metrics[0], &metrics[i]));
        }
    }

    #[tokio::test]
    async fn test_concurrent_dispatcher_initialization_stress() {
        // Stress test: many threads racing to initialize dispatcher
        const NUM_THREADS: usize = 100;
        const NUM_ITERATIONS: usize = 10;

        for iteration in 0..NUM_ITERATIONS {
            let config = MaosConfig::default();
            let config_arc = Arc::new(config.clone());
            let metrics = Arc::new(PerformanceMetrics::new());

            // Create context without dispatcher
            let context = Arc::new(CliContext::build().await.unwrap());

            // Pre-set config and metrics to isolate dispatcher initialization
            let _ = context.config.set(config_arc.clone());
            let _ = context.metrics.set(metrics.clone());

            // Use a barrier to ensure all tasks start at the same time
            let barrier = Arc::new(tokio::sync::Barrier::new(NUM_THREADS));

            let mut handles = vec![];
            for thread_id in 0..NUM_THREADS {
                let ctx = context.clone();
                let barrier = barrier.clone();
                let test_config = config_arc.clone();
                let test_metrics = metrics.clone();

                let handle = tokio::spawn(async move {
                    // Wait for all threads to be ready
                    barrier.wait().await;

                    // Try to create a test dispatcher simultaneously
                    // This simulates the race condition where multiple threads
                    // try to initialize the dispatcher at the same time
                    let mock_input = HookInput {
                        session_id: format!("stress_test_{thread_id}"),
                        transcript_path: "/tmp/test.jsonl".into(),
                        cwd: "/tmp".into(),
                        hook_event_name: "test_event".to_string(),
                        ..Default::default()
                    };

                    let input_provider = Box::new(MockInputProvider { input: mock_input });

                    let test_dispatcher = CommandDispatcher::new_with_input_provider(
                        test_config,
                        test_metrics,
                        input_provider,
                    )
                    .await
                    .unwrap();

                    // Race to set the dispatcher
                    let _ = ctx.dispatcher.set(test_dispatcher);

                    // Now try to get it - should always succeed
                    let dispatcher = ctx.get_dispatcher().await.unwrap();
                    dispatcher as *const _ as usize // Convert pointer to usize for Send
                });
                handles.push(handle);
            }

            let mut dispatcher_ptrs = vec![];
            for handle in handles {
                let ptr = handle.await.unwrap();
                dispatcher_ptrs.push(ptr);
            }

            // All threads should see the same dispatcher instance
            let first_ptr = dispatcher_ptrs[0];
            for ptr in &dispatcher_ptrs[1..] {
                assert_eq!(
                    first_ptr, *ptr,
                    "Iteration {iteration}: Different dispatcher instances detected - race condition!"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_context_error_propagation() {
        // Test that errors from command execution are properly handled
        let config = MaosConfig::default();
        let config_arc = Arc::new(config.clone());
        let metrics = Arc::new(PerformanceMetrics::new());

        // Create a test dispatcher with mock input but no handlers
        let dispatcher = create_test_dispatcher(config_arc, metrics).await;

        let context = CliContext::build_with_test_dispatcher(config, dispatcher)
            .await
            .unwrap();

        // Use a command that will fail (no handler registered)
        let exit_code = context.execute(Commands::Stop { chat: false }).await;

        // Should return an error exit code
        assert_ne!(exit_code, ExitCode::Success);
    }

    #[tokio::test]
    async fn test_context_custom_config_persistence() {
        // Test that custom config is properly retained
        let mut custom_config = MaosConfig::default();
        custom_config.system.max_execution_time_ms = 12345;
        custom_config.hooks.max_input_size_mb = 999;
        custom_config.security.allowed_tools = vec!["echo".to_string()];

        let context = CliContext::build_with_config(custom_config.clone())
            .await
            .unwrap();

        // Config should match what we provided
        let loaded_config = context.config().unwrap();
        assert_eq!(loaded_config.system.max_execution_time_ms, 12345);
        assert_eq!(loaded_config.hooks.max_input_size_mb, 999);
        assert_eq!(
            loaded_config.security.allowed_tools,
            vec!["echo".to_string()]
        );
    }

    #[tokio::test]
    async fn test_context_metrics_collection() {
        // Test that metrics are properly initialized and shared
        let context = CliContext::build().await.unwrap();

        // Get metrics and record some data
        let metrics = context.metrics();
        metrics.record_execution_time("test_handler", std::time::Duration::from_millis(100));

        // Get metrics again - should have the recorded data
        let metrics2 = context.metrics();
        assert!(Arc::ptr_eq(&metrics, &metrics2));

        let report = metrics2.export_metrics();
        assert!(report.execution_stats.contains_key("test_handler"));
    }

    #[tokio::test]
    async fn test_context_lazy_dispatcher_initialization() {
        // Test that components are truly lazy-initialized only when accessed
        // This test verifies memory efficiency and startup performance

        // Track initialization order and timing
        let start = std::time::Instant::now();
        let context = CliContext::build().await.unwrap();
        let build_time = start.elapsed();

        // Building context should be nearly instant (< 1ms) since nothing is initialized
        assert!(
            build_time.as_millis() < 10,
            "Context build took {build_time:?} - should be instant since it's lazy"
        );

        // Verify nothing is initialized yet
        assert!(
            context.config.get().is_none(),
            "Config should not be eagerly initialized"
        );
        assert!(
            context.metrics.get().is_none(),
            "Metrics should not be eagerly initialized"
        );
        assert!(
            context.dispatcher.get().is_none(),
            "Dispatcher should not be eagerly initialized"
        );

        // Test selective initialization - metrics only
        let metrics_start = std::time::Instant::now();
        let _ = context.metrics();
        let metrics_time = metrics_start.elapsed();

        assert!(
            context.config.get().is_none(),
            "Config should still be uninitialized"
        );
        assert!(
            context.metrics.get().is_some(),
            "Metrics should now be initialized"
        );
        assert!(
            context.dispatcher.get().is_none(),
            "Dispatcher should still be uninitialized"
        );
        assert!(
            metrics_time.as_micros() < 1000,
            "Metrics initialization should be fast (< 1ms)"
        );

        // Test selective initialization - config only
        let config_start = std::time::Instant::now();
        let _ = context.config().unwrap();
        let config_time = config_start.elapsed();

        assert!(
            context.config.get().is_some(),
            "Config should now be initialized"
        );
        assert!(
            context.dispatcher.get().is_none(),
            "Dispatcher should still be uninitialized"
        );

        // Config loading might take longer due to file I/O
        assert!(
            config_time.as_millis() < 100,
            "Config initialization took {config_time:?} - should be reasonably fast"
        );

        // Verify that accessing the same resource again is instant (no re-initialization)
        let second_access_start = std::time::Instant::now();
        let _ = context.config().unwrap();
        let _ = context.metrics();
        let second_access_time = second_access_start.elapsed();

        assert!(
            second_access_time.as_micros() < 100,
            "Subsequent accesses should be instant (< 100μs), took {second_access_time:?}"
        );
    }

    #[tokio::test]
    async fn test_context_execute_with_output() {
        use crate::cli::handler::{CommandHandler, CommandResult, ExecutionMetrics};
        use crate::io::HookInput;
        use async_trait::async_trait;

        struct TestHandler;

        #[async_trait]
        impl CommandHandler for TestHandler {
            async fn execute(&self, _input: HookInput) -> Result<CommandResult> {
                Ok(CommandResult {
                    exit_code: ExitCode::Success,
                    output: Some("Test output message".to_string()),
                    metrics: ExecutionMetrics::default(),
                })
            }
            fn name(&self) -> &'static str {
                "test_handler"
            }
        }

        let config = MaosConfig::default();
        let config_arc = Arc::new(config.clone());
        let metrics = Arc::new(PerformanceMetrics::new());

        // Create a test dispatcher with mock input
        let dispatcher = create_test_dispatcher(config_arc, metrics).await;

        // Register the test handler
        dispatcher.registry.register(
            maos_core::hook_constants::PRE_COMPACT.to_string(),
            Box::new(TestHandler),
        );

        let context = CliContext::build_with_test_dispatcher(config, dispatcher)
            .await
            .unwrap();

        // Execute should handle the output
        let exit_code = context.execute(Commands::PreCompact).await;
        assert_eq!(exit_code, ExitCode::Success);
    }

    #[tokio::test]
    async fn test_context_multiple_dispatcher_access() {
        // Test that multiple accesses to dispatcher return the same instance
        let config = MaosConfig::default();
        let config_arc = Arc::new(config.clone());
        let metrics = Arc::new(PerformanceMetrics::new());

        // Create a test dispatcher
        let dispatcher = create_test_dispatcher(config_arc, metrics).await;

        let context = CliContext::build_with_test_dispatcher(config, dispatcher)
            .await
            .unwrap();

        // Multiple calls to get_dispatcher should return the same instance
        let dispatcher1 = context.get_dispatcher().await.unwrap();
        let dispatcher2 = context.get_dispatcher().await.unwrap();

        // Both should be the same reference (pointers should be equal)
        assert!(std::ptr::eq(dispatcher1, dispatcher2));
    }

    #[tokio::test]
    async fn test_config_race_condition_protection() {
        // Test that our race condition fix for get_config() actually works
        const NUM_TASKS: usize = 50;

        let context = Arc::new(CliContext::build().await.unwrap());
        let barrier = Arc::new(tokio::sync::Barrier::new(NUM_TASKS));

        let mut handles = vec![];
        for _ in 0..NUM_TASKS {
            let ctx = context.clone();
            let barrier = barrier.clone();

            let handle = tokio::spawn(async move {
                // Wait for all tasks to start simultaneously
                barrier.wait().await;

                // All tasks race to initialize config
                let config = ctx.config().unwrap();
                Arc::as_ptr(&config) as usize // Convert to usize which is Send
            });
            handles.push(handle);
        }

        let mut config_ptrs = vec![];
        for handle in handles {
            let ptr = handle.await.unwrap();
            config_ptrs.push(ptr);
        }

        // All tasks should get the exact same Arc instance
        let first_ptr = config_ptrs[0];
        for ptr in &config_ptrs[1..] {
            assert_eq!(
                first_ptr, *ptr,
                "Different config instances detected - race condition in get_config()!"
            );
        }
    }
}
