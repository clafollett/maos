//! Agent management types for MAOS
//!
//! This module provides types for managing agents within MAOS sessions.
//! MAOS is completely agent-agnostic - users can define any agent type they want
//! in Claude Code without MAOS knowing about specific implementations.
//!
//! # Design Philosophy
//!
//! MAOS treats agents as opaque entities identified by user-defined strings.
//! This allows maximum flexibility - users can create agents like "frontend-engineer",
//! "code-reviewer", "test-writer", or any custom type without modifying MAOS.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Agent type - flexible string to support any user-defined agent
///
/// IMPORTANT: MAOS is agent-agnostic and does not know about specific agent types.
/// Users can create any agent type in Claude Code without MAOS awareness.
///
/// # Example
///
/// ```
/// use maos_core::AgentType;
///
/// let agent_type: AgentType = "backend-engineer".to_string();
/// let custom_type: AgentType = "my-custom-agent".to_string();
/// ```
pub type AgentType = String;

/// Unique identifier for an agent
///
/// Agent IDs follow the format: `agent_{uuid}`
/// where uuid is a v4 UUID.
///
/// # Example
///
/// ```
/// use maos_core::AgentId;
///
/// let id = AgentId::generate();
/// assert!(id.is_valid());
/// assert!(id.as_str().starts_with("agent_"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentId(String);

// Use the macro to implement common ID functionality
crate::impl_id_type!(AgentId, "agent");

/// Agent capabilities configuration (loaded from user config)
///
/// Defines what an agent can do, which tools it can use, and where it can work.
/// This is typically loaded from user configuration files to customize agent behavior.
///
/// # Example
///
/// ```
/// use maos_core::AgentCapabilities;
/// use std::collections::HashMap;
/// use std::path::PathBuf;
///
/// let capabilities = AgentCapabilities {
///     agent_type: "database-engineer".to_string(),
///     capabilities: vec!["sql".to_string(), "schema-design".to_string()],
///     tool_denylist: vec!["Write".to_string()], // Can't use Write tool
///     workspace_paths: vec![PathBuf::from("/workspace/db")],
///     environment_vars: HashMap::from([
///         ("DATABASE_URL".to_string(), "postgres://localhost".to_string())
///     ]),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// The type of agent these capabilities apply to
    pub agent_type: AgentType,
    /// User-defined capability strings (e.g., "database", "testing")
    pub capabilities: Vec<String>,
    /// Tool names this agent cannot use (e.g., "Write", "Bash")
    pub tool_denylist: Vec<String>,
    /// Filesystem paths this agent is allowed to access
    pub workspace_paths: Vec<PathBuf>,
    /// Environment variables to set for this agent
    pub environment_vars: HashMap<String, String>,
}

/// Agent information and state
///
/// Complete metadata about an agent instance, including its identity,
/// current status, and workspace location.
///
/// # Example
///
/// ```
/// use maos_core::{AgentInfo, AgentId, AgentStatus, SessionId};
/// use chrono::Utc;
/// use std::path::PathBuf;
///
/// let info = AgentInfo {
///     id: AgentId::generate(),
///     agent_type: "code-reviewer".to_string(),
///     session_id: SessionId::generate(),
///     workspace_path: PathBuf::from("/tmp/agent-workspace"),
///     status: AgentStatus::Active,
///     created_at: Utc::now(),
///     last_activity: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Unique identifier for this agent instance
    pub id: AgentId,
    /// User-defined agent type string
    pub agent_type: AgentType,
    /// Session this agent belongs to
    pub session_id: crate::types::session::SessionId,
    /// Filesystem path where this agent operates
    pub workspace_path: PathBuf,
    /// Current status of the agent
    pub status: AgentStatus,
    /// When this agent was created
    pub created_at: DateTime<Utc>,
    /// Last time this agent performed any action
    pub last_activity: DateTime<Utc>,
}

/// Status of an agent within MAOS
///
/// Agents progress through different states during their lifecycle:
/// - `Initializing`: Agent is being set up
/// - `Active`: Agent is currently working on tasks
/// - `Waiting`: Agent is idle, waiting for work
/// - `Completed`: Agent finished all its tasks
/// - `Failed`: Agent encountered an error
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// Agent is being initialized
    Initializing,
    /// Agent is actively working
    Active,
    /// Agent is waiting for tasks
    Waiting,
    /// Agent completed its work
    Completed,
    /// Agent failed with an error
    Failed {
        /// Description of what went wrong
        error: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::session::SessionId;
    use chrono::Utc;

    #[test]
    fn test_agent_id_generation() {
        let id = AgentId::generate();
        assert!(id.is_valid());

        // Should be unique
        let id2 = AgentId::generate();
        assert_ne!(id, id2);
    }

    #[test]
    fn test_agent_type_flexibility() {
        // AgentType should be a simple string alias
        let agent_type: AgentType = "frontend-engineer".to_string();
        assert_eq!(agent_type, "frontend-engineer");

        // Any string should be valid
        let custom_type: AgentType = "my-custom-agent-type".to_string();
        assert!(!custom_type.is_empty());
    }

    #[test]
    fn test_agent_capabilities() {
        let capabilities = AgentCapabilities {
            agent_type: "backend-engineer".to_string(),
            capabilities: vec!["database".to_string(), "api-design".to_string()],
            tool_denylist: vec!["Write".to_string()],
            workspace_paths: vec![PathBuf::from("/workspace/backend")],
            environment_vars: HashMap::from([("NODE_ENV".to_string(), "development".to_string())]),
        };

        assert_eq!(capabilities.agent_type, "backend-engineer");
        assert_eq!(capabilities.capabilities.len(), 2);
    }

    #[test]
    fn test_agent_info_creation() {
        let info = AgentInfo {
            id: AgentId::generate(),
            agent_type: "tester".to_string(),
            session_id: SessionId::generate(),
            workspace_path: PathBuf::from("/tmp/agent-workspace"),
            status: AgentStatus::Active,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };

        assert!(info.id.is_valid());
        assert_eq!(info.agent_type, "tester");
    }

    #[test]
    fn test_agent_status_variants() {
        let statuses = vec![
            AgentStatus::Initializing,
            AgentStatus::Active,
            AgentStatus::Waiting,
            AgentStatus::Completed,
            AgentStatus::Failed {
                error: "test error".to_string(),
            },
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: AgentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_agent_id_validation() {
        // Valid ID
        let valid = AgentId("agent_550e8400-e29b-41d4-a716-446655440000".to_string());
        assert!(valid.is_valid());

        // Invalid IDs - wrong prefix
        assert!(!AgentId("invalid".to_string()).is_valid());
        assert!(!AgentId("sess_550e8400-e29b-41d4-a716-446655440000".to_string()).is_valid());
        assert!(!AgentId("agents_550e8400-e29b-41d4-a716-446655440000".to_string()).is_valid());

        // Invalid IDs - wrong structure
        assert!(!AgentId("".to_string()).is_valid());
        assert!(!AgentId("agent".to_string()).is_valid());
        assert!(!AgentId("agent_".to_string()).is_valid());
        assert!(!AgentId("agent_invalid-uuid".to_string()).is_valid());

        // Invalid IDs - bad UUID
        assert!(!AgentId("agent_not-a-uuid".to_string()).is_valid());
        assert!(!AgentId("agent_550e8400-e29b-41d4-a716".to_string()).is_valid()); // Too short
        assert!(
            !AgentId("agent_550e8400-e29b-41d4-a716-446655440000-extra".to_string()).is_valid()
        ); // Too long
        assert!(!AgentId("agent_550e8400-e29b-41d4-a716-44665544000g".to_string()).is_valid()); // Invalid char
    }
}
