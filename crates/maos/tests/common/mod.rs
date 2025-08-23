//! Common test utilities and helpers for integration tests
//!
//! This module provides shared functionality for all integration tests

use serde_json::{Value, json};
use std::collections::HashMap;

/// Builder for creating test hook inputs with a fluent API
pub struct TestInputBuilder {
    session_id: String,
    transcript_path: String,
    cwd: String,
    hook_event_name: String,
    extra_fields: HashMap<String, Value>,
}

impl TestInputBuilder {
    /// Create a new builder with defaults
    pub fn new(hook_event: &str) -> Self {
        Self {
            session_id: "test-session".to_string(),
            transcript_path: "/tmp/transcript.jsonl".to_string(),
            cwd: "/tmp/test".to_string(),
            hook_event_name: hook_event.to_string(),
            extra_fields: HashMap::new(),
        }
    }

    /// Set the session ID
    pub fn session_id(mut self, id: &str) -> Self {
        self.session_id = id.to_string();
        self
    }

    /// Set the transcript path
    #[allow(dead_code)] // Used by exit_code_integration tests
    pub fn transcript_path(mut self, path: &str) -> Self {
        self.transcript_path = path.to_string();
        self
    }

    /// Set the current working directory
    #[allow(dead_code)] // Used by exit_code_integration tests
    pub fn cwd(mut self, cwd: &str) -> Self {
        self.cwd = cwd.to_string();
        self
    }

    /// Add an extra field
    pub fn with(mut self, key: &str, value: Value) -> Self {
        self.extra_fields.insert(key.to_string(), value);
        self
    }

    /// Add tool-specific fields for pre_tool_use
    pub fn with_tool(mut self, tool_name: &str, tool_input: Value) -> Self {
        self.extra_fields
            .insert("tool_name".to_string(), json!(tool_name));
        self.extra_fields
            .insert("tool_input".to_string(), tool_input);
        self
    }

    /// Add tool output for post_tool_use
    pub fn with_tool_output(mut self, tool_name: &str, output: &str, error: Option<&str>) -> Self {
        self.extra_fields
            .insert("tool_name".to_string(), json!(tool_name));
        self.extra_fields
            .insert("tool_output".to_string(), json!(output));
        self.extra_fields.insert(
            "tool_error".to_string(),
            error.map(|e| json!(e)).unwrap_or(Value::Null),
        );
        self
    }

    /// Add a message for notifications
    pub fn with_message(mut self, message: &str) -> Self {
        self.extra_fields
            .insert("message".to_string(), json!(message));
        self
    }

    /// Build the JSON string
    pub fn build(self) -> String {
        let mut result = json!({
            "session_id": self.session_id,
            "transcript_path": self.transcript_path,
            "cwd": self.cwd,
            "hook_event_name": self.hook_event_name,
        });

        if let Some(obj) = result.as_object_mut() {
            for (key, value) in self.extra_fields {
                obj.insert(key, value);
            }
        }

        result.to_string()
    }
}

/// Common test data generators
pub mod generators {
    use super::*;

    /// Generate a valid pre_tool_use message
    pub fn pre_tool_use(tool_name: &str) -> String {
        TestInputBuilder::new("pre_tool_use")
            .with_tool(tool_name, json!({"file_path": "/tmp/test.txt"}))
            .build()
    }

    /// Generate a valid post_tool_use message
    pub fn post_tool_use(tool_name: &str, output: &str) -> String {
        TestInputBuilder::new("post_tool_use")
            .with_tool_output(tool_name, output, None)
            .build()
    }

    /// Generate a notification message
    pub fn notification(message: &str) -> String {
        TestInputBuilder::new("notification")
            .with_message(message)
            .build()
    }

    /// Generate a session_start message
    #[allow(dead_code)] // Used by e2e_integration tests
    pub fn session_start() -> String {
        TestInputBuilder::new("session_start").build()
    }
}

/// Exit code constants for assertions
pub mod exit_codes {
    #[allow(dead_code)] // Used by multiple test files
    pub const SUCCESS: i32 = 0;
    #[allow(dead_code)] // Used by multiple test files
    pub const GENERAL_ERROR: i32 = 1;
    /// TODO: PRD-06 will implement actual blocking for dangerous commands
    #[allow(dead_code)]
    pub const BLOCKING_ERROR: i32 = 2;
    /// TODO: PRD-07 will enhance configuration validation
    #[allow(dead_code)]
    pub const CONFIG_ERROR: i32 = 3;
    /// TODO: PRD-06 will add security validation and path traversal blocking
    #[allow(dead_code)]
    pub const SECURITY_ERROR: i32 = 4;
}

/// Platform-specific test helpers
pub mod platform {
    /// Get platform-appropriate test path
    /// Used by cross_platform tests
    #[allow(dead_code)]
    pub fn test_path() -> String {
        if cfg!(windows) {
            "C:\\Temp\\test".to_string()
        } else {
            "/tmp/test".to_string()
        }
    }

    /// Get platform-appropriate transcript path
    /// Used by cross_platform tests
    #[allow(dead_code)]
    pub fn transcript_path() -> String {
        if cfg!(windows) {
            "C:\\Temp\\transcript.jsonl".to_string()
        } else {
            "/tmp/transcript.jsonl".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_builder() {
        let input = TestInputBuilder::new("test_event")
            .session_id("custom-session")
            .with("extra_field", json!("value"))
            .build();

        assert!(input.contains("\"session_id\":\"custom-session\""));
        assert!(input.contains("\"hook_event_name\":\"test_event\""));
        assert!(input.contains("\"extra_field\":\"value\""));
    }

    #[test]
    fn test_generators() {
        let pre_tool = generators::pre_tool_use("Read");
        assert!(pre_tool.contains("\"tool_name\":\"Read\""));

        let post_tool = generators::post_tool_use("Write", "Success");
        assert!(post_tool.contains("\"tool_output\":\"Success\""));

        let notification = generators::notification("Test message");
        assert!(notification.contains("\"message\":\"Test message\""));
    }
}
