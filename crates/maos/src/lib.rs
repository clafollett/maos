//! MAOS (Multi-Agent Orchestration System)
//!
//! A high-performance Rust library for Claude Code hook processing and CLI operations.
//! Provides JSON I/O processing, command-line interface, and orchestration capabilities.

/// Command-line interface module
pub mod cli;

/// JSON input/output processing for Claude Code hooks
pub mod io;

pub use cli::{Cli, Commands};
