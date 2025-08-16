//! JSON Input/Output processing for Claude Code hooks
//!
//! This module provides high-performance JSON processing for stdin/stdout
//! communication with Claude Code hooks, supporting all 8 hook event types.

pub mod messages;
pub mod processor;

pub use messages::HookInput;
pub use processor::StdinProcessor;

#[cfg(test)]
mod tests {
    mod dos_protection_tests;
    mod error_handling_tests;
    mod memory_dos_tests;
    mod security_tests;
    mod type_safety_tests;
}
