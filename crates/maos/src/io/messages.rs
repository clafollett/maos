//! Hook message types compatible with Claude Code JSON format

use maos_core::path::PathValidator;
use maos_core::{MaosError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

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
///     "hook_event_name": maos_core::hook_constants::PRE_TOOL_USE,
///     "tool_name": "Bash",
///     "tool_input": {"command": "ls"}
/// });
///
/// let input: HookInput = serde_json::from_value(json).unwrap();
/// assert_eq!(input.hook_event_name, maos_core::hook_constants::PRE_TOOL_USE);
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

impl Default for HookInput {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            transcript_path: PathBuf::new(),
            cwd: PathBuf::new(),
            hook_event_name: String::new(),
            tool_name: None,
            tool_input: None,
            tool_response: None,
            message: None,
            prompt: None,
            stop_hook_active: None,
            trigger: None,
            custom_instructions: None,
            source: None,
        }
    }
}

impl HookInput {
    /// Check if this is a tool-related event
    pub fn is_tool_event(&self) -> bool {
        matches!(
            self.hook_event_name.as_str(),
            maos_core::hook_constants::PRE_TOOL_USE | maos_core::hook_constants::POST_TOOL_USE
        )
    }

    /// Get the tool name (returns empty string if not a tool event)
    pub fn get_tool_name(&self) -> &str {
        self.tool_name.as_deref().unwrap_or("")
    }

    /// Validate that required fields are present for the hook type
    /// 🔥 TYPE SAFETY ENHANCEMENT: Uses enum-based validation when possible
    pub fn validate(&self) -> Result<()> {
        // 🔥 TYPE SAFETY: Try enum-based validation first
        if let Ok(event) =
            maos_core::hook_events::HookEvent::try_from(self.hook_event_name.as_str())
        {
            return self.validate_typed_event(event);
        }

        // Fallback to string-based validation for unknown events
        match self.hook_event_name.as_str() {
            maos_core::hook_constants::PRE_TOOL_USE => {
                if self.tool_name.is_none() || self.tool_input.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "pre_tool_use requires tool_name and tool_input".to_string(),
                    });
                }
            }
            maos_core::hook_constants::POST_TOOL_USE => {
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
            maos_core::hook_constants::NOTIFICATION => {
                if self.message.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "notification requires message".to_string(),
                    });
                }
            }
            maos_core::hook_constants::USER_PROMPT_SUBMIT => {
                if self.prompt.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "user_prompt_submit requires prompt".to_string(),
                    });
                }
            }
            maos_core::hook_constants::PRE_COMPACT => {
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
                            "Invalid trigger value: {trigger}. Must be 'manual' or 'auto'"
                        ),
                    });
                }
            }
            maos_core::hook_constants::SESSION_START => {
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
                            "Invalid source value: {source}. Must be 'startup', 'resume', or 'clear'"
                        ),
                    });
                }
            }
            maos_core::hook_constants::STOP | maos_core::hook_constants::SUBAGENT_STOP => {
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

    /// Type-safe validation using strongly-typed HookEvent enum
    /// 🔥 TYPE SAFETY: Compile-time guaranteed complete coverage of all events
    fn validate_typed_event(&self, event: maos_core::hook_events::HookEvent) -> Result<()> {
        use maos_core::hook_events::HookEvent;

        match event {
            HookEvent::PreToolUse => {
                if self.tool_name.is_none() || self.tool_input.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "pre_tool_use requires tool_name and tool_input".to_string(),
                    });
                }
            }
            HookEvent::PostToolUse => {
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
            HookEvent::Notification => {
                if self.message.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "notification requires message".to_string(),
                    });
                }
            }
            HookEvent::UserPromptSubmit => {
                if self.prompt.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "user_prompt_submit requires prompt".to_string(),
                    });
                }
            }
            HookEvent::PreCompact => {
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
                            "Invalid trigger value: {trigger}. Must be 'manual' or 'auto'"
                        ),
                    });
                }
            }
            HookEvent::SessionStart => {
                if self.source.is_none() {
                    return Err(MaosError::InvalidInput {
                        message: "session_start requires source".to_string(),
                    });
                }

                // Validate source value with enum for type safety
                if let Some(source) = &self.source
                    && source != "startup"
                    && source != "resume"
                    && source != "clear"
                {
                    return Err(MaosError::InvalidInput {
                        message: format!(
                            "Invalid source value: {source}. Must be 'startup', 'resume', or 'clear'"
                        ),
                    });
                }
            }
            HookEvent::Stop | HookEvent::SubagentStop => {
                // stop_hook_active is optional for both
            }
        }

        Ok(())
    }

    /// 🔥 CRITICAL SECURITY FIX: Validate all path fields against workspace boundaries
    ///
    /// This method prevents path traversal attacks by ensuring that all path fields
    /// (transcript_path, cwd) are contained within the specified workspace directory.
    ///
    /// # Security
    ///
    /// Without this validation, malicious JSON input could access arbitrary files:
    /// ```json
    /// {
    ///   "transcript_path": "../../../etc/passwd",
    ///   "cwd": "/root/.ssh",
    ///   "hook_event_name": "pre_tool_use"
    /// }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `workspace` - The trusted workspace directory that all paths must be within
    ///
    /// # Errors
    ///
    /// Returns `MaosError::InvalidInput` if any path field attempts to escape
    /// the workspace boundary or contains malicious path components.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::{Path, PathBuf};
    /// use maos::io::HookInput;
    ///
    /// let input = HookInput {
    ///     transcript_path: PathBuf::from("/workspace/transcript.jsonl"),
    ///     cwd: PathBuf::from("/workspace/project"),
    ///     hook_event_name: "pre_tool_use".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// let workspace = Path::new("/workspace");
    /// input.validate_paths(workspace).unwrap(); // ✅ Safe - within workspace
    ///
    /// let malicious_input = HookInput {
    ///     transcript_path: PathBuf::from("../../../etc/passwd"),
    ///     cwd: PathBuf::from("/workspace/../../root"),
    ///     hook_event_name: "pre_tool_use".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// assert!(malicious_input.validate_paths(workspace).is_err()); // ❌ Blocked
    /// ```
    pub fn validate_paths(&self, workspace: &Path) -> Result<()> {
        // 🚨 SECURITY: Check for empty paths first
        if self.transcript_path.as_os_str().is_empty() {
            return Err(MaosError::InvalidInput {
                message: "Empty transcript path not allowed".to_string(),
            });
        }

        if self.cwd.as_os_str().is_empty() {
            return Err(MaosError::InvalidInput {
                message: "Empty working directory not allowed".to_string(),
            });
        }

        // 🚨 SECURITY: Check for URL schemes that shouldn't be in paths
        if let Some(transcript_str) = self.transcript_path.to_str()
            && transcript_str.contains("://")
        {
            return Err(MaosError::InvalidInput {
                message: "URL schemes not allowed in transcript path".to_string(),
            });
        }

        if let Some(cwd_str) = self.cwd.to_str()
            && cwd_str.contains("://")
        {
            return Err(MaosError::InvalidInput {
                message: "URL schemes not allowed in working directory".to_string(),
            });
        }

        let validator = PathValidator::new(
            vec![workspace.to_path_buf()],
            vec![], // No additional blocked patterns needed - workspace isolation is sufficient
        );

        // 🛡️ Validate transcript path is within workspace
        validator
            .validate_workspace_path(&self.transcript_path, workspace)
            .map_err(|_e| MaosError::InvalidInput {
                message:
                    "Invalid transcript path: Security violation detected (path traversal blocked)"
                        .to_string(),
            })?;

        // 🛡️ Validate CWD is within workspace
        validator.validate_workspace_path(&self.cwd, workspace)
            .map_err(|_e| MaosError::InvalidInput {
                message: "Invalid working directory: Security violation detected (path traversal blocked)".to_string(),
            })?;

        Ok(())
    }
}
