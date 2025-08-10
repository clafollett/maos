//! Core types and logic for MAOS (Multi-Agent Orchestration System)
//!
//! This crate provides the foundational types, error handling, and shared
//! utilities that all other MAOS components depend on. It ensures consistency,
//! type safety, and performance across the entire system.
//!
//! # Core Components
//!
//! - **Types**: Domain models for sessions, agents, and tool interactions
//! - **Error Handling**: Comprehensive error types with clear exit codes
//! - **Configuration**: Flexible configuration management (coming soon)
//! - **Path Utilities**: Secure path validation and manipulation (coming soon)
//!
//! # Example
//!
//! ```no_run
//! use maos_core::{SessionId, Session, SessionStatus, AgentId};
//! use chrono::Utc;
//! use std::path::PathBuf;
//!
//! // Create a new session
//! let session = Session {
//!     id: SessionId::generate(),
//!     created_at: Utc::now(),
//!     last_activity: Utc::now(),
//!     status: SessionStatus::Active,
//!     workspace_root: PathBuf::from("/tmp/maos"),
//!     active_agents: vec![],
//! };
//!
//! // Create a new agent
//! let agent_id = AgentId::generate();
//! assert!(agent_id.is_valid());
//! ```

#[macro_use]
pub mod types;
pub mod config;
pub mod constants;
pub mod error;
pub mod logging;
pub mod metrics;
pub mod path;

// Re-export commonly used types
pub use types::{
    agent::{AgentCapabilities, AgentId, AgentInfo, AgentStatus, AgentType},
    session::{Session, SessionId, SessionStatus},
    tool::{ToolCall, ToolCallId, ToolResult},
};

// Re-export error types
pub use error::{
    ConfigError, ErrorContext, ExitCode, FileSystemError, GitError, IntoMaosError, MaosError,
    PathValidationError, Result, SecurityError, SessionError, ValidationError,
};

// Re-export metrics types
pub use metrics::{ExecutionStats, MemoryStats, MetricsReport, PerformanceMetrics};
