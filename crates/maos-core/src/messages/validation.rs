//! JSON schema validation for message types
//!
//! This module provides JSON schema validation for all MAOS message types,
//! ensuring data integrity and format compliance across the system.

use crate::messages::{HookInput, HookResponse, SessionFile};
use serde_json::Value;
use thiserror::Error;

/// Schema validation errors
#[derive(Error, Debug)]
pub enum SchemaError {
    /// Schema compilation failed
    #[error("Schema compilation failed: {0}")]
    CompilationFailed(String),

    /// Validation failed with details
    #[error("Validation failed for {schema}: {}", errors.join(", "))]
    ValidationFailed {
        /// Schema name that failed
        schema: String,
        /// List of validation errors
        errors: Vec<String>,
    },

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Schema validator with cached compiled schemas
///
/// Provides runtime validation of JSON messages against their schemas.
/// This ensures compatibility between Rust and Python implementations.
///
/// # Example
///
/// ```
/// use maos_core::messages::SchemaValidator;
/// use serde_json::json;
///
/// let validator = SchemaValidator::new();
///
/// let hook_input = json!({
///     "session_id": "sess_12345678-1234-1234-1234-123456789012",
///     "transcript_path": "/tmp/transcript",
///     "cwd": "/workspace",
///     "hook_event_name": "PreToolUse",
///     "tool_name": "Edit",
///     "tool_input": { "file_path": "test.rs" }
/// });
///
/// assert!(validator.validate_hook_input(&hook_input).is_ok());
/// ```
pub struct SchemaValidator {
    // In a full implementation, we would cache compiled JSON schemas here
    // For now, we'll do structural validation
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaValidator {
    /// Create new validator with compiled schemas
    pub fn new() -> Self {
        Self {}
    }

    /// Validate a hook input message
    ///
    /// Checks that the JSON conforms to either the legacy or documented format.
    pub fn validate_hook_input(&self, value: &Value) -> Result<(), SchemaError> {
        // Try to deserialize as HookInput - this validates the structure
        let _input: HookInput =
            serde_json::from_value(value.clone()).map_err(|e| SchemaError::ValidationFailed {
                schema: "HookInput".to_string(),
                errors: vec![e.to_string()],
            })?;

        Ok(())
    }

    /// Validate a hook response message
    pub fn validate_hook_response(&self, value: &Value) -> Result<(), SchemaError> {
        // Check for required 'action' field
        if value.get("action").is_none() {
            return Err(SchemaError::ValidationFailed {
                schema: "HookResponse".to_string(),
                errors: vec!["Missing required field: action".to_string()],
            });
        }

        // Try to deserialize as HookResponse
        let _response: HookResponse =
            serde_json::from_value(value.clone()).map_err(|e| SchemaError::ValidationFailed {
                schema: "HookResponse".to_string(),
                errors: vec![e.to_string()],
            })?;

        Ok(())
    }

    /// Validate a session file
    pub fn validate_session_file(&self, value: &Value) -> Result<(), SchemaError> {
        // Check for required fields
        let required_fields = [
            "session_id",
            "created_at",
            "updated_at",
            "status",
            "workspace_root",
        ];
        let mut missing_fields = Vec::new();

        for field in &required_fields {
            if value.get(field).is_none() {
                missing_fields.push(format!("Missing required field: {field}"));
            }
        }

        if !missing_fields.is_empty() {
            return Err(SchemaError::ValidationFailed {
                schema: "SessionFile".to_string(),
                errors: missing_fields,
            });
        }

        // Validate status enum value
        match value.get("status").and_then(|v| v.as_str()) {
            Some(status) if !["active", "paused", "completed", "failed"].contains(&status) => {
                return Err(SchemaError::ValidationFailed {
                    schema: "SessionFile".to_string(),
                    errors: vec![format!("Invalid status value: {}", status)],
                });
            }
            _ => {}
        }

        // Try to deserialize to validate full structure
        let _session: SessionFile =
            serde_json::from_value(value.clone()).map_err(|e| SchemaError::ValidationFailed {
                schema: "SessionFile".to_string(),
                errors: vec![e.to_string()],
            })?;

        Ok(())
    }

    /// Validate an agents file
    pub fn validate_agents_file(&self, value: &Value) -> Result<(), SchemaError> {
        // Check for required fields
        if value.get("session_id").is_none() {
            return Err(SchemaError::ValidationFailed {
                schema: "AgentsFile".to_string(),
                errors: vec!["Missing required field: session_id".to_string()],
            });
        }

        if value.get("agents").and_then(|v| v.as_array()).is_none() {
            return Err(SchemaError::ValidationFailed {
                schema: "AgentsFile".to_string(),
                errors: vec!["Missing or invalid field: agents (must be array)".to_string()],
            });
        }

        Ok(())
    }

    /// Validate a locks file
    pub fn validate_locks_file(&self, value: &Value) -> Result<(), SchemaError> {
        // Check for required fields
        if value.get("session_id").is_none() {
            return Err(SchemaError::ValidationFailed {
                schema: "LocksFile".to_string(),
                errors: vec!["Missing required field: session_id".to_string()],
            });
        }

        if value.get("locks").and_then(|v| v.as_array()).is_none() {
            return Err(SchemaError::ValidationFailed {
                schema: "LocksFile".to_string(),
                errors: vec!["Missing or invalid field: locks (must be array)".to_string()],
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_validator_hook_input() {
        let validator = SchemaValidator::new();

        // Valid PreToolUse event
        let valid_pre_tool = json!({
            "session_id": "sess_12345678-1234-1234-1234-123456789012",
            "transcript_path": "/tmp/transcript",
            "cwd": "/workspace",
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {
                "command": "cargo test"
            }
        });
        assert!(validator.validate_hook_input(&valid_pre_tool).is_ok());

        // Valid UserPromptSubmit event
        let valid_prompt = json!({
            "session_id": "sess_12345678-1234-1234-1234-123456789012",
            "transcript_path": "/tmp/transcript",
            "cwd": "/workspace",
            "hook_event_name": "UserPromptSubmit",
            "prompt": "Help me with this code"
        });
        assert!(validator.validate_hook_input(&valid_prompt).is_ok());

        // Invalid - missing required fields
        let invalid = json!({
            "some_field": "value"
        });
        assert!(validator.validate_hook_input(&invalid).is_err());
    }

    #[test]
    fn test_schema_validator_hook_response() {
        let validator = SchemaValidator::new();

        // Valid Allow response
        let valid_allow = json!({
            "action": "Allow"
        });
        assert!(validator.validate_hook_response(&valid_allow).is_ok());

        // Valid Block response
        let valid_block = json!({
            "action": "Block",
            "data": {
                "reason": "Security violation"
            }
        });
        assert!(validator.validate_hook_response(&valid_block).is_ok());

        // Invalid - missing action
        let invalid = json!({
            "data": {
                "reason": "test"
            }
        });
        assert!(validator.validate_hook_response(&invalid).is_err());
    }

    #[test]
    fn test_schema_validator_session_file() {
        let validator = SchemaValidator::new();

        // Valid session file
        let valid = json!({
            "session_id": "sess_12345678-1234-1234-1234-123456789012",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:01:00Z",
            "status": "active",
            "workspace_root": "/workspace"
        });
        assert!(validator.validate_session_file(&valid).is_ok());

        // Invalid - bad status value
        let invalid_status = json!({
            "session_id": "sess_12345678-1234-1234-1234-123456789012",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:01:00Z",
            "status": "invalid_status",
            "workspace_root": "/workspace"
        });
        assert!(validator.validate_session_file(&invalid_status).is_err());
    }

    #[test]
    fn test_schema_error_display() {
        let error = SchemaError::ValidationFailed {
            schema: "HookInput".to_string(),
            errors: vec!["Missing field: tool_name".to_string()],
        };

        let display = format!("{error}");
        assert!(display.contains("HookInput"));
        assert!(display.contains("Missing field: tool_name"));
    }
}
