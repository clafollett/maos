//! MAOS (Multi-Agent Orchestration System)
//!
//! A high-performance Rust library for Claude Code hook processing and CLI operations.
//! Provides JSON I/O processing, command-line interface, and orchestration capabilities.

pub mod cli;
pub mod io;
pub mod security;

pub use cli::{Cli, Commands};
