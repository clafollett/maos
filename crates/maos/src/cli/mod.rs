mod commands;
pub mod context;
pub mod dispatcher;
pub mod handler;
pub mod handlers;
pub mod registry;

pub use commands::{Cli, Commands};
pub use context::CliContext;
pub use dispatcher::CommandDispatcher;
pub use handler::{CommandHandler, CommandResult, ExecutionMetrics};
pub use registry::HandlerRegistry;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod thread_safety_tests;
