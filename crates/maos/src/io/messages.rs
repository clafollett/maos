//! Hook message types compatible with Claude Code JSON format

use maos_core::{MaosError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Hook input message from Claude Code
///
/// This struct represents the JSON payload sent by Claude Code to hooks via stdin.
/// It contains both required base fields and optional event-specific fields.
///
/// # Claude Code Compatibility
///
/// This structure exactly matches Claude Code's hook input format with:
/// - Base fields: session_id, transcript_path, cwd, hook_event_name
/// - Event-specific optional fields for each of the 8 hook types
///
/// # Example
///
/// ```
/// use maos::io::HookInput;
/// use serde_json::json;
///
/// let json = json!({
///     "session_id": "sess_123",
///     "transcript_path": "/tmp/transcript.jsonl",
///     "cwd": "/workspace",
///     "hook_event_name": "pre_tool_use",
///     "tool_name": "Bash",
///     "tool_input": {"command": "ls"}
/// });
///
/// let input: HookInput = serde_json::from_value(json).unwrap();
/// assert_eq!(input.hook_event_name, "pre_tool_use");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInput {
    /// Unique session identifier from Claude Code
    pub session_id: String,

    /// Path to the conversation transcript file
    pub transcript_path: PathBuf,

    /// Current working directory
    pub cwd: PathBuf,

    /// Hook event type (snake_case: pre_tool_use, post_tool_use, etc.)
    pub hook_event_name: String,

    // ===== Tool-related fields (PreToolUse, PostToolUse) =====
    /// Name of the tool being called
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    /// Tool input parameters (tool-specific JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<Value>,

    /// Tool execution result (PostToolUse only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_response: Option<Value>,

    // ===== Notification field =====
    /// Notification message content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    // ===== UserPromptSubmit field =====
    /// User's prompt text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    // ===== Stop/SubagentStop field =====
    /// Whether stop hook is active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_hook_active: Option<bool>,

    // ===== PreCompact fields =====
    /// Compaction trigger type ("manual" or "auto")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,

    /// Custom instructions for compaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_instructions: Option<String>,

    // ===== SessionStart field =====
    /// Session source ("startup", "resume", or "clear")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl HookInput {
    /// Check if this is a tool-related event
    pub fn is_tool_event(&self) -> bool {
        matches!(
            self.hook_event_name.as_str(),
            "pre_tool_use" | "post_tool_use"
        )
    }

    /// Get the tool name (returns empty string if not a tool event)
    pub fn get_tool_name(&self) -> &str {
        self.tool_name.as_deref().unwrap_or("")
    }

    /// Validate that required fields are present for the hook type
    pub fn validate(&self) -> Result<()> {
        match self.hook_event_name.as_str() {
            "pre_tool_use" => {
                if self.tool_name.is_none() || self.tool_input.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "pre_tool_use requires tool_name and tool_input".to_string(),
                    });
                }
            }
            "post_tool_use" => {
                if self.tool_name.is_none()
                    || self.tool_input.is_none()
                    || self.tool_response.is_none()
                {
                    return Err(MaosError::InvalidInput {
                        message: "post_tool_use requires tool_name, tool_input, and tool_response"
                            .to_string(),
                    });
                }
            }
            "notification" => {
                if self.message.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "notification requires message".to_string(),
                    });
                }
            }
            "user_prompt_submit" => {
                if self.prompt.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "user_prompt_submit requires prompt".to_string(),
                    });
                }
            }
            "pre_compact" => {
                if self.trigger.is_none() || self.custom_instructions.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "pre_compact requires trigger and custom_instructions".to_string(),
                    });
                }

                // Validate trigger value
                if let Some(trigger) = &self.trigger
                    && trigger != "manual"
                    && trigger != "auto"
                {
                    return Err(MaosError::InvalidInput {
                        message: format!(
                            "Invalid trigger value: {}. Must be 'manual' or 'auto'",
                            trigger
                        ),
                    });
                }
            }
            "session_start" => {
                if self.source.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "session_start requires source".to_string(),
                    });
                }

                // Validate source value
                if let Some(source) = &self.source
                    && source != "startup"
                    && source != "resume"
                    && source != "clear"
                {
                    return Err(MaosError::InvalidInput {
                        message: format!(
                            "Invalid source value: {}. Must be 'startup', 'resume', or 'clear'",
                            source
                        ),
                    });
                }
            }
            "stop" | "subagent_stop" => {
                // stop_hook_active is optional
            }
            _ => {
                return Err(MaosError::InvalidInput {
                    message: format!("Unknown hook event: {}", self.hook_event_name),
                });
            }
        }

        Ok(())
    }
}
