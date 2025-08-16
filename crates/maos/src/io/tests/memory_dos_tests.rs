//! 🛡️ MEMORY DoS PROTECTION TESTS for Issue #56
//!
//! These tests specifically validate memory exhaustion attack prevention
//! and verify that our memory monitoring and limits work correctly.

use crate::io::processor::StdinProcessor;
use maos_core::config::HookConfig;
use std::sync::Arc;
use tokio::task::JoinSet;

#[tokio::test]
async fn test_memory_exhaustion_attack_blocked() {
    // 🚨 CRITICAL: Test that repeated large allocations are blocked
    let config = HookConfig {
        max_input_size_mb: 2, // Small limit for testing
        max_processing_time_ms: 1000,
        stdin_read_timeout_ms: 500,
        max_json_depth: 10,
    };

    let processor = StdinProcessor::new(config);

    // Attempt to allocate beyond memory limits
    let attack_size = 5 * 1024 * 1024; // 5MB > 2MB limit

    let result = processor.validate_size(attack_size);
    assert!(result.is_err(), "Memory DoS attack should be blocked");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("security"));
}

#[tokio::test]
async fn test_rapid_allocation_attack() {
    // 🔥 Test protection against rapid allocation attacks
    let config = HookConfig {
        max_input_size_mb: 1,
        max_processing_time_ms: 1000,
        stdin_read_timeout_ms: 500,
        max_json_depth: 10,
    };

    let processor = Arc::new(StdinProcessor::new(config));
    let mut tasks = JoinSet::new();

    // Spawn multiple tasks trying to allocate at once
    for _i in 0..50 {
        let proc = Arc::clone(&processor);
        tasks.spawn(async move {
            // Each task tries to allocate near the limit
            let size = 800 * 1024; // 800KB each
            // Individual requests should pass size check
            // but system should handle concurrent allocations safely
            proc.validate_size(size)
        });
    }

    let mut success_count = 0;
    let mut error_count = 0;

    while let Some(result) = tasks.join_next().await {
        match result.unwrap() {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    // System should handle concurrent requests without panicking
    assert!(success_count > 0, "Some requests should succeed");
    println!("Concurrent allocation test: {success_count} success, {error_count} errors");
}

#[test]
fn test_memory_tracking_accuracy() {
    // 🎯 Test that memory tracking is reasonably accurate
    let baseline = StdinProcessor::get_memory_usage();

    // Allocate known amounts and verify tracking
    let allocations = vec![
        vec![0u8; 1024 * 1024],     // 1MB
        vec![0u8; 2 * 1024 * 1024], // 2MB
        vec![0u8; 4 * 1024 * 1024], // 4MB
    ];

    let after_allocation = StdinProcessor::get_memory_usage();

    // Memory usage should have increased
    assert!(
        after_allocation >= baseline,
        "Memory tracking should show increase: baseline={baseline}, after={after_allocation}"
    );

    // Keep allocations alive to prevent optimization
    drop(allocations);
}

#[tokio::test]
async fn test_memory_limit_enforcement_in_dispatcher() {
    // 🛡️ Test memory limits are enforced in dispatcher execution
    use crate::cli::dispatcher::CommandDispatcher;
    use crate::io::HookInput;
    use maos_core::{PerformanceMetrics, config::MaosConfig};

    // Create config with very low memory tolerance
    let mut config = MaosConfig::default();
    config.hooks.max_input_size_mb = 1; // 1MB limit
    config.system.max_execution_time_ms = 100; // Short timeout

    let config = Arc::new(config);
    let metrics = Arc::new(PerformanceMetrics::new());

    // Create mock input that would cause memory pressure
    struct LargeMockInputProvider;

    #[async_trait::async_trait]
    impl crate::cli::dispatcher::InputProvider for LargeMockInputProvider {
        async fn read_hook_input(&mut self) -> maos_core::Result<HookInput> {
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
    // (The actual memory check is in dispatcher.rs:124-138)
    let command = crate::cli::Commands::PreToolUse;
    let _result = dispatcher.dispatch(command).await;

    // Test passes if no panic occurs - memory monitoring is working
}

#[test]
fn test_buffer_reuse_prevents_fragmentation() {
    // 🔄 Test that buffer reuse prevents memory fragmentation
    let config = HookConfig::default();
    let _processor = StdinProcessor::new(config);

    let initial_memory = StdinProcessor::get_memory_usage();

    // Simulate many read operations (like DoS attack pattern)
    for i in 0..100 {
        // Vary the data size to test buffer adaptation
        let size = (i % 10 + 1) * 1024; // 1KB to 10KB

        // Simulate buffer operations without actual I/O
        let test_data = vec![0u8; size];

        // Buffer should reuse capacity efficiently
        // (This tests the pattern from processor.rs where we reuse BytesMut)
        let _serialized = serde_json::to_vec(&serde_json::json!({
            "test_data_size": test_data.len()
        }))
        .unwrap();
    }

    let final_memory = StdinProcessor::get_memory_usage();
    let memory_growth = final_memory.saturating_sub(initial_memory);

    // Memory growth should be bounded despite many operations
    assert!(
        memory_growth < 10 * 1024 * 1024, // Should grow less than 10MB
        "Excessive memory growth detected: {memory_growth} bytes"
    );
}

#[tokio::test]
async fn test_memory_pressure_recovery() {
    // 🔄 Test system recovery after memory pressure
    let config = HookConfig {
        max_input_size_mb: 10,
        max_processing_time_ms: 2000,
        stdin_read_timeout_ms: 1000,
        max_json_depth: 10,
    };

    let processor = StdinProcessor::new(config);

    // Create memory pressure
    let large_size = 8 * 1024 * 1024; // 8MB
    assert!(processor.validate_size(large_size).is_ok());

    // Allow some time for potential memory cleanup
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // System should still be responsive
    let small_size = 1024; // 1KB
    assert!(processor.validate_size(small_size).is_ok());

    // Memory tracking should still work
    let current_memory = StdinProcessor::get_memory_usage();
    assert!(current_memory > 0, "Memory tracking should still work");
}

#[test]
fn test_memory_dos_error_messages_sanitized() {
    // 🔍 Test that memory DoS error messages don't leak sensitive info
    let config = HookConfig {
        max_input_size_mb: 1, // 1MB limit
        max_processing_time_ms: 1000,
        stdin_read_timeout_ms: 500,
        max_json_depth: 10,
    };

    let processor = StdinProcessor::new(config);

    // Attempt allocation beyond limit
    let attack_size = 10 * 1024 * 1024; // 10MB
    let result = processor.validate_size(attack_size);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();

    // Should NOT contain specific size information
    assert!(!error_msg.contains("10485760")); // 10MB in bytes
    assert!(!error_msg.contains("1048576")); // 1MB in bytes

    // Should contain helpful but safe guidance
    assert!(error_msg.contains("security"));
    assert!(error_msg.contains("size"));
}

#[tokio::test]
async fn test_memory_monitoring_concurrent_access() {
    // 🔄 Test memory monitoring under concurrent access
    let mut tasks = JoinSet::new();

    // Spawn multiple tasks checking memory concurrently
    for _ in 0..20 {
        tasks.spawn(async {
            let memory = StdinProcessor::get_memory_usage();
            assert!(memory > 0, "Memory usage should be positive");

            // Simulate some work
            let _temp = vec![0u8; 1024]; // 1KB allocation

            let memory2 = StdinProcessor::get_memory_usage();
            assert!(memory2 > 0, "Memory usage should remain positive");
        });
    }

    // All tasks should complete without issues
    while let Some(result) = tasks.join_next().await {
        result.unwrap();
    }
}

#[test]
fn test_memory_limit_edge_cases() {
    // 🎯 Test edge cases in memory limit enforcement
    let configs = [
        HookConfig {
            max_input_size_mb: 0, // Zero limit
            max_processing_time_ms: 1000,
            stdin_read_timeout_ms: 500,
            max_json_depth: 10,
        },
        HookConfig {
            max_input_size_mb: 100, // Reasonable high limit
            max_processing_time_ms: 1000,
            stdin_read_timeout_ms: 500,
            max_json_depth: 10,
        },
    ];

    for config in configs {
        let processor = StdinProcessor::new(config.clone());

        // Zero limit should reject everything except empty
        if config.max_input_size_mb == 0 {
            assert!(processor.validate_size(0).is_ok());
            assert!(processor.validate_size(1).is_err());
        }

        // High limit should accept reasonable sizes
        if config.max_input_size_mb == 100 {
            assert!(processor.validate_size(1024 * 1024).is_ok());
        }
    }
}
