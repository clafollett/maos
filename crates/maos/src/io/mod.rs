//! JSON Input/Output processing for Claude Code hooks
//!
//! This module provides high-performance JSON processing for stdin/stdout
//! communication with Claude Code hooks, supporting all 8 hook event types.

pub mod messages;
pub mod processor;

#[cfg(test)]
mod tests;

pub use messages::HookInput;
pub use processor::StdinProcessor;
