//! Command-line interface module for MAOS
//!
//! This module provides the complete CLI infrastructure for processing Claude Code hooks,
//! including command parsing, handler dispatch, and execution context management.

mod commands;
pub mod context;
pub mod dispatcher;
pub mod handler;
pub mod handlers;
pub mod registry;

/// Command-line interface parser and command definitions
pub use commands::{Cli, Commands};

/// CLI dependency injection context for shared resources
pub use context::CliContext;

/// Command dispatcher for routing commands to appropriate handlers
pub use dispatcher::CommandDispatcher;

/// Core traits and types for command handler implementations
pub use handler::{CommandHandler, CommandResult, ExecutionMetrics};

/// Thread-safe registry for managing command handlers
pub use registry::HandlerRegistry;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod thread_safety_tests;
