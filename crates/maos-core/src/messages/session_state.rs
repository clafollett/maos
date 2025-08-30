//! Session state file formats for persistence
//!
//! This module provides schemas for persisting session state, agent coordination,
//! and file locking across the MAOS system. These structures enable proper
//! session management that was missing in the Python implementation.

use crate::constants::*;
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
        let root = PathBuf::from(MAOS_ROOT_DIR)
            .join(SESSIONS_DIR_NAME)
            .join(session_id.as_str());

        Ok(Self { root })
    }

    /// Get path to session.json
    pub fn session_file_path(&self) -> PathBuf {
        self.root.join(SESSION_FILE_NAME)
    }

    /// Get path to agents.json
    pub fn agents_file_path(&self) -> PathBuf {
        self.root.join(AGENTS_FILE_NAME)
    }

    /// Get path to locks.json
    pub fn locks_file_path(&self) -> PathBuf {
        self.root.join(LOCKS_FILE_NAME)
    }

    /// Get path to progress.json
    pub fn progress_file_path(&self) -> PathBuf {
        self.root.join(PROGRESS_FILE_NAME)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SessionId;
    use serde_json::json;
    use std::str::FromStr;

    #[test]
    fn test_session_file_schema() {
        // Test the main session state file format
        let session_json = json!({
            "session_id": "sess_12345678-1234-1234-1234-123456789012",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:01:00Z",
            "status": "active",
            "workspace_root": "/workspace",
            "transcript_path": "/tmp/transcript.txt",
            "metadata": {
                "user": "test_user",
                "project": "maos",
                "environment": "development"
            }
        });

        let session: SessionFile = serde_json::from_value(session_json.clone()).unwrap();
        assert_eq!(
            session.session_id.as_str(),
            "sess_12345678-1234-1234-1234-123456789012"
        );
        assert_eq!(session.status, SessionStatus::Active);

        // Should round-trip serialize
        let serialized = serde_json::to_value(&session).unwrap();
        assert_eq!(serialized["status"], "active");
    }

    #[test]
    fn test_agents_file_schema() {
        // Test agent coordination file format
        let agents_json = json!({
            "session_id": "sess_12345678-1234-1234-1234-123456789012",
            "agents": [
                {
                    "agent_id": "agent_12345678-1234-1234-1234-123456789012",
                    "agent_type": "backend-engineer",
                    "status": "active",
                    "workspace": "/worktrees/backend-sess_123",
                    "started_at": "2024-01-01T00:00:00Z",
                    "parent_agent": null,
                    "current_task": "Implementing API endpoint"
                },
                {
                    "agent_id": "agent_87654321-4321-4321-4321-210987654321",
                    "agent_type": "qa-engineer",
                    "status": "pending",
                    "workspace": null,
                    "started_at": "2024-01-01T00:00:30Z",
                    "parent_agent": "agent_12345678-1234-1234-1234-123456789012",
                    "current_task": null
                }
            ]
        });

        let agents_file: AgentsFile = serde_json::from_value(agents_json).unwrap();
        assert_eq!(agents_file.agents.len(), 2);
        assert_eq!(agents_file.agents[0].agent_type, "backend-engineer");
        assert_eq!(agents_file.agents[0].status, AgentStatus::Active);
        assert_eq!(agents_file.agents[1].status, AgentStatus::Pending);

        // Verify parent agent relationship
        assert!(agents_file.agents[0].parent_agent.is_none());
        assert_eq!(
            agents_file.agents[1]
                .parent_agent
                .as_ref()
                .unwrap()
                .as_str(),
            "agent_12345678-1234-1234-1234-123456789012"
        );
    }

    #[test]
    fn test_locks_file_schema() {
        // Test file locking coordination
        let locks_json = json!({
            "session_id": "sess_12345678-1234-1234-1234-123456789012",
            "locks": [
                {
                    "file_path": "/workspace/src/main.rs",
                    "agent_id": "agent_12345678-1234-1234-1234-123456789012",
                    "lock_type": "exclusive",
                    "operation": "Edit",
                    "acquired_at": "2024-01-01T00:00:00Z"
                },
                {
                    "file_path": "/workspace/Cargo.toml",
                    "agent_id": "agent_87654321-4321-4321-4321-210987654321",
                    "lock_type": "shared",
                    "operation": "Read",
                    "acquired_at": "2024-01-01T00:00:30Z"
                }
            ]
        });

        let locks_file: LocksFile = serde_json::from_value(locks_json).unwrap();
        assert_eq!(locks_file.locks.len(), 2);
        assert_eq!(locks_file.locks[0].lock_type, LockType::Exclusive);
        assert_eq!(locks_file.locks[1].lock_type, LockType::Shared);
    }

    #[test]
    fn test_session_directory_structure() {
        let session_id = SessionId::from_str("sess_12345678-1234-1234-1234-123456789012").unwrap();
        let session_dir = SessionDirectory::new(&session_id).unwrap();

        // Should create proper directory structure
        assert!(session_dir.session_file_path().ends_with(SESSION_FILE_NAME));
        assert!(session_dir.agents_file_path().ends_with(AGENTS_FILE_NAME));
        assert!(session_dir.locks_file_path().ends_with(LOCKS_FILE_NAME));
        assert!(
            session_dir
                .progress_file_path()
                .ends_with(PROGRESS_FILE_NAME)
        );
    }
}
