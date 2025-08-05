//! Session management types for MAOS
//!
//! This module provides types for managing MAOS sessions, including unique
//! identifiers, session state, and metadata tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique identifier for a MAOS session
///
/// Session IDs follow the format: `sess_{uuid}`
/// where uuid is a v4 UUID.
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
#[serde(transparent)]
pub struct SessionId(String);

// Use the macro to implement common ID functionality
crate::impl_id_type!(SessionId, "sess");

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
    /// List of active agent IDs in this session
    pub active_agents: Vec<crate::types::agent::AgentId>,
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

        // Should have prefix and UUID parts
        let parts: Vec<&str> = id_str.splitn(2, '_').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "sess");

        // Validate UUID format
        assert!(uuid::Uuid::parse_str(parts[1]).is_ok());
    }

    #[test]
    fn test_session_id_validation() {
        // Valid ID
        let valid = SessionId("sess_550e8400-e29b-41d4-a716-446655440000".to_string());
        assert!(valid.is_valid());

        // Invalid IDs - wrong prefix
        assert!(!SessionId("invalid".to_string()).is_valid());
        assert!(!SessionId("session_550e8400-e29b-41d4-a716-446655440000".to_string()).is_valid());
        assert!(!SessionId("agent_550e8400-e29b-41d4-a716-446655440000".to_string()).is_valid());

        // Invalid IDs - wrong structure
        assert!(!SessionId("".to_string()).is_valid());
        assert!(!SessionId("sess".to_string()).is_valid());
        assert!(!SessionId("sess_".to_string()).is_valid());
        assert!(!SessionId("sess_invalid-uuid".to_string()).is_valid());

        // Invalid IDs - bad UUID
        assert!(!SessionId("sess_not-a-uuid".to_string()).is_valid());
        assert!(!SessionId("sess_550e8400-e29b-41d4-a716".to_string()).is_valid()); // Too short
        assert!(
            !SessionId("sess_550e8400-e29b-41d4-a716-446655440000-extra".to_string()).is_valid()
        ); // Too long
        assert!(!SessionId("sess_550e8400-e29b-41d4-a716-44665544000g".to_string()).is_valid()); // Invalid char
    }

    #[test]
    fn test_session_creation() {
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
