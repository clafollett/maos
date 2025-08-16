//! CLI context for dependency injection and shared resources

use crate::cli::{Commands, dispatcher::CommandDispatcher};
use maos_core::config::MaosConfig;
use maos_core::{ExitCode, PerformanceMetrics, Result};
use std::sync::Arc;

/// Dependency container for CLI operations
pub struct CliContext {
    /// Shared configuration
    pub config: Arc<MaosConfig>,
    /// Performance metrics collector
    pub metrics: Arc<PerformanceMetrics>,
    /// Command dispatcher
    pub dispatcher: CommandDispatcher,
}

impl CliContext {
    /// Build CLI context with configuration
    pub async fn build() -> Result<Self> {
        let config = Arc::new(MaosConfig::load()?);
        let metrics = Arc::new(PerformanceMetrics::new());
        let dispatcher = CommandDispatcher::new(config.clone(), metrics.clone()).await?;

        Ok(Self {
            config,
            metrics,
            dispatcher,
        })
    }

    /// Build CLI context with custom configuration (useful for testing)
    pub async fn build_with_config(config: MaosConfig) -> Result<Self> {
        let config = Arc::new(config);
        let metrics = Arc::new(PerformanceMetrics::new());
        let dispatcher = CommandDispatcher::new(config.clone(), metrics.clone()).await?;

        Ok(Self {
            config,
            metrics,
            dispatcher,
        })
    }

    /// Execute a command and return the exit code
    pub async fn execute(mut self, command: Commands) -> ExitCode {
        match self.dispatcher.dispatch(command).await {
            Ok(exit_code) => exit_code,
            Err(err) => {
                eprintln!("Error: {}", err);
                ExitCode::from(&err)
            }
        }
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &MaosConfig {
        &self.config
    }

    /// Get a reference to the metrics
    pub fn metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_build() {
        let context = CliContext::build().await.unwrap();

        // Should have initialized all components
        assert!(Arc::strong_count(&context.config) > 0);
        assert!(Arc::strong_count(&context.metrics) > 0);
    }

    #[tokio::test]
    async fn test_context_shared_resources() {
        let config = MaosConfig::default();
        let context = CliContext::build_with_config(config).await.unwrap();

        // Resources should be shared via Arc
        let config_ref1 = context.config.clone();
        let _config_ref2 = context.config.clone();
        // The count is 4 because: original + 2 clones + 1 in dispatcher
        assert_eq!(Arc::strong_count(&config_ref1), 4);

        let metrics_ref1 = context.metrics.clone();
        let _metrics_ref2 = context.metrics.clone();
        // The count is 4 because: original + 2 clones + 1 in dispatcher
        assert_eq!(Arc::strong_count(&metrics_ref1), 4);
    }

    #[tokio::test]
    async fn test_context_config_loading() {
        let mut custom_config = MaosConfig::default();
        custom_config.system.max_execution_time_ms = 5000;

        let context = CliContext::build_with_config(custom_config).await.unwrap();

        // Should use the provided configuration
        assert_eq!(context.config.system.max_execution_time_ms, 5000);
    }

    #[tokio::test]
    async fn test_context_handler_initialization() {
        let config = MaosConfig::default();
        let context = CliContext::build_with_config(config).await.unwrap();

        // Test that handlers are registered (registry should have 8 handlers)
        assert_eq!(context.dispatcher.registry.len(), 8);

        // Test that we can get a handler
        let command = Commands::PreCompact;
        let handler = context.dispatcher.registry.get_handler(&command).unwrap();
        assert_eq!(handler.name(), maos_core::hook_constants::PRE_COMPACT);
    }
}
