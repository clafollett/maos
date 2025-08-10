//! Session state file formats for persistence
//!
//! This module provides schemas for persisting session state, agent coordination,
//! and file locking across the MAOS system. These structures enable proper
//! session management that was missing in the Python implementation.

use crate::{AgentId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Session status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is currently active
    Active,
    /// Session is paused/suspended
    Paused,
    /// Session has completed
    Completed,
    /// Session was terminated with error
    Failed,
}

/// Main session state file
///
/// This is the primary session configuration and state tracking file.
/// Located at: `.maos/sessions/{session_id}/session.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFile {
    /// Unique session identifier
    pub session_id: SessionId,

    /// When this session was created
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Current session status
    pub status: SessionStatus,

    /// Root workspace directory for this session
    pub workspace_root: PathBuf,

    /// Path to transcript file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<PathBuf>,

    /// Session metadata (user, project, environment, etc)
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Agent status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// Agent is pending activation
    Pending,
    /// Agent is currently active
    Active,
    /// Agent is paused
    Paused,
    /// Agent completed successfully
    Completed,
    /// Agent failed
    Failed,
}

/// Individual agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Unique agent identifier
    pub agent_id: AgentId,

    /// Agent type (backend-engineer, qa-engineer, etc)
    pub agent_type: String,

    /// Current agent status
    pub status: AgentStatus,

    /// Agent's workspace directory (if created)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<PathBuf>,

    /// When agent was started
    pub started_at: DateTime<Utc>,

    /// Parent agent that spawned this one (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_agent: Option<AgentId>,

    /// Current task description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_task: Option<String>,
}

/// Agents coordination file
///
/// Tracks all agents active in a session.
/// Located at: `.maos/sessions/{session_id}/agents.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsFile {
    /// Session this belongs to
    pub session_id: SessionId,

    /// List of agents in this session
    pub agents: Vec<AgentInfo>,
}

/// Lock type for file coordination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockType {
    /// Exclusive write lock - only one agent can hold
    Exclusive,
    /// Shared read lock - multiple agents can hold
    Shared,
}

/// Individual file lock entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLock {
    /// Path to locked file
    pub file_path: PathBuf,

    /// Agent holding the lock
    pub agent_id: AgentId,

    /// Type of lock
    pub lock_type: LockType,

    /// Operation being performed
    pub operation: String,

    /// When lock was acquired
    pub acquired_at: DateTime<Utc>,
}

/// File locks coordination file
///
/// Manages file locking across agents to prevent conflicts.
/// Located at: `.maos/sessions/{session_id}/locks.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocksFile {
    /// Session this belongs to
    pub session_id: SessionId,

    /// Active file locks
    pub locks: Vec<FileLock>,
}

/// Helper for session directory structure
pub struct SessionDirectory {
    root: PathBuf,
}

impl SessionDirectory {
    /// Create new session directory helper
    pub fn new(session_id: &SessionId) -> Result<Self, std::io::Error> {
        let root = PathBuf::from(".maos")
            .join("sessions")
            .join(session_id.as_str());

        Ok(Self { root })
    }

    /// Get path to session.json
    pub fn session_file_path(&self) -> PathBuf {
        self.root.join("session.json")
    }

    /// Get path to agents.json
    pub fn agents_file_path(&self) -> PathBuf {
        self.root.join("agents.json")
    }

    /// Get path to locks.json
    pub fn locks_file_path(&self) -> PathBuf {
        self.root.join("locks.json")
    }

    /// Get path to progress.json
    pub fn progress_file_path(&self) -> PathBuf {
        self.root.join("progress.json")
    }
}
