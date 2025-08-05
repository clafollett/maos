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
use nanoid::nanoid;
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
/// Agent IDs follow the format: `agent_{timestamp}_{random}`
/// where timestamp is YYYYMMDDHHMMss and random is a 6-character nanoid.
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
pub struct AgentId(String);

impl AgentId {
    /// Generate a new unique agent ID
    pub fn generate() -> Self {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let random = nanoid!(6);
        Self(format!("agent_{timestamp}_{random}"))
    }

    /// Check if the agent ID format is valid
    pub fn is_valid(&self) -> bool {
        let parts: Vec<&str> = self.0.split('_').collect();
        parts.len() == 3 && parts[0] == "agent" && !parts[1].is_empty() && !parts[2].is_empty()
    }

    /// Get the ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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
///     tool_restrictions: vec!["Write".to_string()], // Can't use Write tool
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
    pub tool_restrictions: Vec<String>,
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
        // RED: AgentId needs generate() method
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
            tool_restrictions: vec!["Write".to_string()],
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
}
