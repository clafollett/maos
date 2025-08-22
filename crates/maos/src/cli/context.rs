//! CLI context for dependency injection and shared resources

use crate::cli::{Commands, dispatcher::CommandDispatcher};
use maos_core::config::MaosConfig;
use maos_core::{ExitCode, PerformanceMetrics, Result};
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
    fn get_config(&self) -> Result<Arc<MaosConfig>> {
        if let Some(config) = self.config.get() {
            Ok(config.clone())
        } else {
            let config = Arc::new(MaosConfig::load()?);
            let _ = self.config.set(config.clone());
            Ok(config)
        }
    }

    /// Get or initialize metrics
    fn get_metrics(&self) -> Arc<PerformanceMetrics> {
        self.metrics
            .get_or_init(|| Arc::new(PerformanceMetrics::new()))
            .clone()
    }

    /// Get or initialize dispatcher
    async fn get_dispatcher(&self) -> Result<&CommandDispatcher> {
        if self.dispatcher.get().is_none() {
            let config = self.get_config()?;
            let metrics = self.get_metrics();
            let dispatcher = CommandDispatcher::new(config, metrics).await?;
            let _ = self.dispatcher.set(dispatcher);
        }
        Ok(self.dispatcher.get().unwrap())
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
        let context = CliContext::build_with_config(config).await.unwrap();

        // Dispatcher should be lazily initialized
        assert!(context.dispatcher.get().is_none());

        // After getting the dispatcher, it should be initialized
        let dispatcher = context.get_dispatcher().await.unwrap();
        assert_eq!(dispatcher.registry.len(), 8);

        // Test that we can get a handler
        let command = Commands::PreCompact;
        let handler = dispatcher.registry.get_handler(&command).unwrap();
        assert_eq!(handler.name(), maos_core::hook_constants::PRE_COMPACT);
    }
}
