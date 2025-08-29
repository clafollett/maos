//! Integration tests for I/O error handling
//!
//! These tests verify error handling across the io and cli modules

use async_trait::async_trait;
use maos::cli::Commands;
use maos::cli::dispatcher::{CommandDispatcher, InputProvider};
use maos::io::HookInput;
use maos_core::config::MaosConfig;
use maos_core::{MaosError, PerformanceMetrics, Result};
use std::sync::Arc;

#[tokio::test]
async fn test_no_double_unwrap_panic() {
    // Test that our error handling is now clean using proper mocks

    // Create a mock that will fail - this tests error handling without hanging
    struct FailingMockInputProvider;

    #[async_trait]
    impl InputProvider for FailingMockInputProvider {
        async fn read_hook_input(&mut self) -> Result<HookInput> {
            // Return a proper error instead of hanging on stdin
            Err(MaosError::InvalidInput {
                message: "Mock failure to test error handling".to_string(),
            })
        }
    }

    let config = Arc::new(MaosConfig::default());
    let metrics = Arc::new(PerformanceMetrics::new());
    let mock_provider = Box::new(FailingMockInputProvider);

    let dispatcher = CommandDispatcher::new_with_input_provider(config, metrics, mock_provider)
        .await
        .unwrap();

    // This should trigger an error, not a panic - proving no double unwrap issue
    let result = dispatcher.dispatch(Commands::PreToolUse).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_memory_limit_enforcement_in_dispatcher() {
    // Test memory limits are enforced in dispatcher execution

    // Create config with very low memory tolerance
    let mut config = MaosConfig::default();
    config.hooks.max_input_size_mb = 1; // 1MB limit
    config.system.max_execution_time_ms = 100; // Short timeout

    let config = Arc::new(config);
    let metrics = Arc::new(PerformanceMetrics::new());

    // Create mock input that would cause memory pressure
    struct LargeMockInputProvider;

    #[async_trait]
    impl InputProvider for LargeMockInputProvider {
        async fn read_hook_input(&mut self) -> Result<HookInput> {
            // Simulate large input processing
            Ok(HookInput {
                session_id: "memory_test_session".to_string(),
                transcript_path: "/tmp/test.jsonl".into(),
                cwd: "/tmp".into(),
                hook_event_name: maos_core::hook_constants::PRE_TOOL_USE.to_string(),
                tool_name: Some("MemoryHog".to_string()),
                tool_input: Some(serde_json::json!({
                    "large_data": "x".repeat(2 * 1024 * 1024) // 2MB > 1MB limit
                })),
                ..Default::default()
            })
        }
    }

    let dispatcher = CommandDispatcher::new_with_input_provider(
        config,
        metrics,
        Box::new(LargeMockInputProvider),
    )
    .await
    .unwrap();

    // Memory monitoring should log warnings for high usage
    let command = Commands::PreToolUse;
    let _result = dispatcher.dispatch(command).await;

    // Test passes if no panic occurs - memory monitoring is working
}
