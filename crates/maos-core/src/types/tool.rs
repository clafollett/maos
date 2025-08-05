//! Tool interaction types for MAOS
//!
//! This module provides types for tracking tool calls made by Claude Code
//! and their results. These types are used by MAOS hooks to intercept,
//! validate, and monitor tool usage.

use crate::types::{agent::AgentId, session::SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Unique identifier for a tool call
///
/// Tool call IDs follow the format: `tool_{uuid}`
/// where uuid is a v4 UUID.
///
/// # Example
///
/// ```
/// use maos_core::ToolCallId;
///
/// let id = ToolCallId::generate();
/// assert!(id.is_valid());
/// assert!(id.as_str().starts_with("tool_"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ToolCallId(String);

// Use the macro to implement common ID functionality
crate::impl_id_type!(ToolCallId, "tool");

/// Tool call metadata from Claude Code hooks
///
/// Represents a request to execute a tool (e.g., Bash, Read, Write) with
/// all the context needed for validation and tracking.
///
/// # Example
///
/// ```
/// use maos_core::{ToolCall, ToolCallId, SessionId, AgentId};
/// use chrono::Utc;
/// use serde_json::json;
///
/// let tool_call = ToolCall {
///     id: ToolCallId::generate(),
///     tool_name: "Bash".to_string(),
///     parameters: json!({
///         "command": "cargo test",
///         "timeout": 30000
///     }),
///     timestamp: Utc::now(),
///     session_id: Some(SessionId::generate()),
///     agent_id: Some(AgentId::generate()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: ToolCallId,
    /// Name of the tool being called (e.g., "Bash", "Read", "Write")
    pub tool_name: String,
    /// Tool-specific parameters as JSON
    pub parameters: Value,
    /// When this tool was called
    pub timestamp: DateTime<Utc>,
    /// Session this tool call belongs to (if within a session)
    pub session_id: Option<SessionId>,
    /// Agent that made this tool call (if made by an agent)
    pub agent_id: Option<AgentId>,
}

/// Tool execution result
///
/// Contains the outcome of a tool execution, including success status,
/// output, errors, and timing information.
///
/// # Example
///
/// ```
/// use maos_core::{ToolResult, ToolCallId};
/// use chrono::Utc;
///
/// let result = ToolResult {
///     tool_call_id: ToolCallId::generate(),
///     success: true,
///     output: Some("Tests passed: 42/42".to_string()),
///     error: None,
///     execution_time_ms: 1250,
///     timestamp: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// ID of the tool call this result belongs to
    pub tool_call_id: ToolCallId,
    /// Whether the tool execution succeeded
    pub success: bool,
    /// Tool output (stdout for commands, file content for reads, etc.)
    pub output: Option<String>,
    /// Error message if the tool failed
    pub error: Option<String>,
    /// How long the tool took to execute in milliseconds
    pub execution_time_ms: u64,
    /// When this result was generated
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{agent::AgentId, session::SessionId};

    #[test]
    fn test_tool_call_creation() {
        let tool_call = ToolCall {
            id: ToolCallId::generate(),
            tool_name: "Bash".to_string(),
            parameters: serde_json::json!({
                "command": "ls -la",
                "timeout": 5000
            }),
            timestamp: Utc::now(),
            session_id: Some(SessionId::generate()),
            agent_id: Some(AgentId::generate()),
        };

        assert_eq!(tool_call.tool_name, "Bash");
        assert!(tool_call.id.is_valid());
    }

    #[test]
    fn test_tool_result_creation() {
        let tool_call_id = ToolCallId::generate();
        let result = ToolResult {
            tool_call_id: tool_call_id.clone(),
            success: true,
            output: Some("file1.txt\nfile2.txt".to_string()),
            error: None,
            execution_time_ms: 150,
            timestamp: Utc::now(),
        };

        assert!(result.success);
        assert_eq!(result.execution_time_ms, 150);
        assert!(result.error.is_none());
        assert_eq!(result.tool_call_id, tool_call_id);
    }

    #[test]
    fn test_tool_call_serialization() {
        let tool_call = ToolCall {
            id: ToolCallId::generate(),
            tool_name: "Read".to_string(),
            parameters: serde_json::json!({
                "file_path": "/tmp/test.txt"
            }),
            timestamp: Utc::now(),
            session_id: None,
            agent_id: None,
        };

        let json = serde_json::to_string(&tool_call).unwrap();
        let deserialized: ToolCall = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, tool_call.id);
        assert_eq!(deserialized.tool_name, tool_call.tool_name);
    }

    #[test]
    fn test_tool_call_id_validation() {
        // Valid ID
        let valid = ToolCallId("tool_550e8400-e29b-41d4-a716-446655440000".to_string());
        assert!(valid.is_valid());

        // Invalid IDs - wrong prefix
        assert!(!ToolCallId("invalid".to_string()).is_valid());
        assert!(!ToolCallId("sess_550e8400-e29b-41d4-a716-446655440000".to_string()).is_valid());
        assert!(!ToolCallId("tools_550e8400-e29b-41d4-a716-446655440000".to_string()).is_valid());

        // Invalid IDs - wrong structure
        assert!(!ToolCallId("".to_string()).is_valid());
        assert!(!ToolCallId("tool".to_string()).is_valid());
        assert!(!ToolCallId("tool_".to_string()).is_valid());
        assert!(!ToolCallId("tool_invalid-uuid".to_string()).is_valid());

        // Invalid IDs - bad UUID
        assert!(!ToolCallId("tool_not-a-uuid".to_string()).is_valid());
        assert!(!ToolCallId("tool_550e8400-e29b-41d4-a716".to_string()).is_valid()); // Too short
        assert!(
            !ToolCallId("tool_550e8400-e29b-41d4-a716-446655440000-extra".to_string()).is_valid()
        ); // Too long
        assert!(!ToolCallId("tool_550e8400-e29b-41d4-a716-44665544000g".to_string()).is_valid()); // Invalid char
    }
}
