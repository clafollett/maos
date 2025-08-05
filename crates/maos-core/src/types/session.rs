//! Session management types for MAOS
//!
//! This module provides types for managing MAOS sessions, including unique
//! identifiers, session state, and metadata tracking.

use chrono::{DateTime, Utc};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique identifier for a MAOS session
///
/// Session IDs follow the format: `sess_{timestamp}_{random}`
/// where timestamp is YYYYMMDDHHMMss and random is a 6-character nanoid.
///
/// # Example
///
/// ```
/// use maos_core::SessionId;
///
/// let id = SessionId::generate();
/// assert!(id.is_valid());
/// assert!(id.as_str().starts_with("sess_"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Generate a new unique session ID
    pub fn generate() -> Self {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let random = nanoid!(6); // 6 character random suffix
        Self(format!("sess_{timestamp}_{random}"))
    }

    /// Check if the session ID format is valid
    pub fn is_valid(&self) -> bool {
        let parts: Vec<&str> = self.0.split('_').collect();
        parts.len() == 3 && parts[0] == "sess" && !parts[1].is_empty() && !parts[2].is_empty()
    }

    /// Get the ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Session metadata and state
///
/// Represents a complete MAOS session with all its metadata, including
/// the workspace location, active agents, and current status.
///
/// # Example
///
/// ```
/// use maos_core::{Session, SessionId, SessionStatus};
/// use chrono::Utc;
/// use std::path::PathBuf;
///
/// let session = Session {
///     id: SessionId::generate(),
///     created_at: Utc::now(),
///     last_activity: Utc::now(),
///     status: SessionStatus::Active,
///     workspace_root: PathBuf::from("/tmp/maos-workspace"),
///     active_agents: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier for this session
    pub id: SessionId,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// Last time any activity occurred in this session
    pub last_activity: DateTime<Utc>,
    /// Current status of the session
    pub status: SessionStatus,
    /// Root directory for all session workspaces
    pub workspace_root: PathBuf,
    /// List of active agent IDs (TODO: Will be Vec<AgentInfo> later)
    pub active_agents: Vec<String>,
}

/// Status of a MAOS session
///
/// Sessions progress through different states during their lifecycle:
/// - `Active`: Currently running with agents working
/// - `Paused`: Temporarily suspended but can be resumed
/// - `Completed`: Successfully finished all tasks
/// - `Failed`: Terminated due to an error
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is actively running
    Active,
    /// Session is paused and can be resumed
    Paused,
    /// Session completed successfully
    Completed,
    /// Session failed with an error
    Failed {
        /// Human-readable failure reason
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_session_id_generation() {
        // This will fail - no generate() method yet
        let id = SessionId::generate();
        assert!(id.is_valid());

        // IDs should be unique
        let id2 = SessionId::generate();
        assert_ne!(id, id2);
    }

    #[test]
    fn test_session_id_format() {
        let id = SessionId::generate();
        let id_str = id.as_str();

        // Should start with "sess_"
        assert!(id_str.starts_with("sess_"));

        // Should have timestamp and random parts
        let parts: Vec<&str> = id_str.split('_').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_session_id_validation() {
        // Valid ID
        let valid = SessionId("sess_20240805_abc123".to_string());
        assert!(valid.is_valid());

        // Invalid IDs
        let invalid = SessionId("invalid".to_string());
        assert!(!invalid.is_valid());

        let empty = SessionId("".to_string());
        assert!(!empty.is_valid());
    }

    #[test]
    fn test_session_creation() {
        // RED: This test will fail - Session doesn't have fields yet
        let session = Session {
            id: SessionId::generate(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            status: SessionStatus::Active,
            workspace_root: PathBuf::from("/tmp/maos-test"),
            active_agents: vec![],
        };

        assert!(session.id.is_valid());
        assert!(matches!(session.status, SessionStatus::Active));
    }

    #[test]
    fn test_session_status_serialization() {
        // Test all status variants serialize correctly
        let statuses = vec![
            SessionStatus::Active,
            SessionStatus::Paused,
            SessionStatus::Completed,
            SessionStatus::Failed {
                reason: "test error".to_string(),
            },
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: SessionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }
}
