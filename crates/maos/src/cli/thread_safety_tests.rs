//! 🔥 CRITICAL THREAD SAFETY TESTS for Issue #56 DashMap Fix
//!
//! These tests verify that the HashMap→DashMap fix prevents data races
//! and allows safe concurrent access to the handler registry.

use crate::cli::Commands;
use crate::cli::registry::HandlerRegistry;
use maos_core::{config::MaosConfig, hook_constants::PRE_TOOL_USE};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_concurrent_handler_access() {
    // 🚨 CRITICAL: Test that multiple threads can safely access registry
    let config = MaosConfig::default();
    let registry = Arc::new(HandlerRegistry::build(&config).await.unwrap());

    // Spawn multiple threads accessing registry concurrently
    let handles = (0..10)
        .map(|i| {
            let registry = Arc::clone(&registry);
            tokio::spawn(async move {
                for j in 0..100 {
                    let command = Commands::PreToolUse;
                    let handler = registry.get_handler(&command);
                    assert!(handler.is_ok(), "Thread {i} iteration {j} failed");

                    // Small delay to increase chance of race conditions if they existed
                    if j % 10 == 0 {
                        sleep(Duration::from_nanos(1)).await;
                    }
                }
            })
        })
        .collect::<Vec<_>>();

    // All tasks should complete without data races or panics
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_concurrent_registry_read_write() {
    // 🔥 CRITICAL: Test that reads and writes can happen concurrently
    let config = MaosConfig::default();
    let registry = Arc::new(HandlerRegistry::build(&config).await.unwrap());

    // Reader tasks - constantly accessing handlers
    let readers = (0..5)
        .map(|i| {
            let registry = Arc::clone(&registry);
            tokio::spawn(async move {
                for j in 0..50 {
                    let command = Commands::Notify;
                    let handler = registry.get_handler(&command);
                    assert!(handler.is_ok(), "Reader {i} iteration {j} failed");
                }
            })
        })
        .collect::<Vec<_>>();

    // Writer task - registering new handlers concurrently
    let writers = (0..2)
        .map(|i| {
            let registry = Arc::clone(&registry);
            tokio::spawn(async move {
                use crate::cli::handler::{CommandHandler, CommandResult, ExecutionMetrics};
                use crate::io::HookInput;
                use async_trait::async_trait;
                use maos_core::{ExitCode, Result};

                struct DynamicHandler;

                #[async_trait]
                impl CommandHandler for DynamicHandler {
                    async fn execute(&self, _input: HookInput) -> Result<CommandResult> {
                        Ok(CommandResult {
                            exit_code: ExitCode::Success,
                            output: None,
                            metrics: ExecutionMetrics::default(),
                        })
                    }

                    fn name(&self) -> &'static str {
                        "dynamic_handler"
                    }
                }

                for j in 0..25 {
                    let key = format!("dynamic_handler_{i}_{j}");
                    registry.register(key, Box::new(DynamicHandler));
                }
            })
        })
        .collect::<Vec<_>>();

    // Wait for all tasks to complete
    for reader in readers {
        reader.await.unwrap();
    }

    for writer in writers {
        writer.await.unwrap();
    }

    // Registry should have original handlers plus new ones
    assert!(registry.len() >= 8); // At least the original 8 handlers
}

#[test]
fn test_registry_is_send_sync() {
    // 🎯 Compile-time test: Verify registry implements Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<HandlerRegistry>();
}

#[tokio::test]
async fn test_handler_references_are_safe() {
    // 🛡️ Test that handler references from DashMap are safe to use
    let config = MaosConfig::default();
    let registry = Arc::new(HandlerRegistry::build(&config).await.unwrap());

    // Get multiple references concurrently
    let handles = (0..5)
        .map(|_| {
            let registry = Arc::clone(&registry);
            tokio::spawn(async move {
                let command = Commands::PreToolUse;
                let handler_ref = registry.get_handler(&command).unwrap();

                // Use the handler reference
                let name = handler_ref.name();
                assert_eq!(name, PRE_TOOL_USE);

                // Reference should remain valid
                drop(handler_ref);
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_no_deadlocks_under_stress() {
    // 🧨 Stress test: Verify no deadlocks under heavy concurrent load
    let config = MaosConfig::default();
    let registry = Arc::new(HandlerRegistry::build(&config).await.unwrap());

    let commands = vec![
        Commands::PreToolUse,
        Commands::PostToolUse,
        Commands::Notify,
        Commands::Stop { chat: false },
        Commands::SubagentStop,
        Commands::UserPromptSubmit { validate: false },
        Commands::PreCompact,
        Commands::SessionStart,
    ];

    // High-stress concurrent access
    let handles = (0..20)
        .map(|i| {
            let registry = Arc::clone(&registry);
            let commands = commands.clone();
            tokio::spawn(async move {
                for j in 0..200 {
                    let command = &commands[j % commands.len()];
                    let handler = registry.get_handler(command);
                    assert!(
                        handler.is_ok(),
                        "Stress test failed at thread {i} iteration {j}"
                    );

                    // Random micro-delays to increase scheduling unpredictability
                    if j % 17 == 0 {
                        sleep(Duration::from_nanos(j as u64 % 100)).await;
                    }
                }
            })
        })
        .collect::<Vec<_>>();

    // All tasks should complete within reasonable time (no deadlocks)
    let timeout = Duration::from_secs(10);
    let completion = async {
        for handle in handles {
            handle.await.unwrap();
        }
    };

    tokio::time::timeout(timeout, completion)
        .await
        .expect("Test timed out - possible deadlock detected!");
}
