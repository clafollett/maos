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
