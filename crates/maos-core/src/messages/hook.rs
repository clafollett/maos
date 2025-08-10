//! Hook message types for Claude Code integration
//!
//! This module provides comprehensive message types for Claude Code hook integration,
//! handling all hook events including tool usage, user prompts, and subagent lifecycle.
//!
//! # Architecture
//!
//! MAOS hooks intercept Claude Code events to provide:
//! - Tool execution monitoring and validation
//! - Session state management
//! - Multi-agent coordination
//! - Security enforcement via path constraints
//!
//! # Event Flow
//!
//! ```text
//! Claude Code → Hook (stdin) → MAOS Processing → Response (exit code)
//! ```
//!
//! # Hook Events
//!
//! - `PreToolUse`: Before tool execution (can block with exit 2)
//! - `PostToolUse`: After tool execution (logging only)
//! - `UserPromptSubmit`: User sends a prompt
//! - `SubagentStart`: Subagent spawned
//! - `SubagentStop`: Subagent terminated
//!
//! # Example
//!
//! ```no_run
//! use maos_core::messages::{HookInput, HookResponse, HookEventName};
//! use std::io;
//! use serde_json;
//!
//! # fn is_dangerous(_input: &HookInput) -> bool { false }
//! // Read hook input from stdin (as Claude Code sends it)
//! let input: HookInput = serde_json::from_reader(io::stdin())?;
//!
//! // Process based on event type
//! match input.hook_event_name {
//!     HookEventName::PreToolUse => {
//!         // Validate tool usage
//!         if is_dangerous(&input) {
//!             let response = HookResponse::Block {
//!                 reason: "Security violation".to_string()
//!             };
//!             std::process::exit(response.to_exit_code());
//!         }
//!     }
//!     _ => {}
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::{AgentId, Result, SessionId, ToolCall, ToolCallId, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Hook event types sent by Claude Code
///
/// These events represent different lifecycle points in Claude Code's execution
/// where hooks can intercept and modify behavior.
///
/// # Event Timing
///
/// - `PreToolUse`: Fired BEFORE a tool executes (can prevent execution)
/// - `PostToolUse`: Fired AFTER a tool completes (logging only)
/// - `UserPromptSubmit`: Fired when user sends a message
/// - `SubagentStart`: Fired when spawning a subagent
/// - `SubagentStop`: Fired when a subagent terminates
/// - `PreCompact`: Fired before compacting conversation history
/// - `SessionStart`: Fired when a new session begins
/// - `Stop`: Fired when session is ending
/// - `Notification`: Fired for system notifications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookEventName {
    /// Before tool execution - can block with exit code 2
    PreToolUse,
    /// After tool execution - for logging and metrics
    PostToolUse,
    /// User submitted a prompt to Claude
    UserPromptSubmit,
    /// A subagent is starting
    SubagentStart,
    /// A subagent has stopped
    SubagentStop,
    /// Before compacting conversation history
    PreCompact,
    /// Session is starting
    SessionStart,
    /// Session is ending
    Stop,
    /// System notification event
    Notification,
}

/// Hook input from Claude Code
///
/// This is the standardized format sent by Claude Code to hooks via stdin.
/// All hooks receive this JSON structure and must parse it to determine
/// the event type and appropriate response.
///
/// # Format
///
/// Claude Code sends this as JSON to the hook's stdin. The hook must:
/// 1. Parse the JSON from stdin
/// 2. Process based on `hook_event_name`
/// 3. Exit with appropriate code (0=allow, 2=block)
///
/// # Example
///
/// ```json
/// {
///   "session_id": "sess_12345678-1234-1234-1234-123456789012",
///   "transcript_path": "/path/to/transcript.jsonl",
///   "cwd": "/current/working/directory",
///   "hook_event_name": "PreToolUse",
///   "tool_name": "Bash",
///   "tool_input": {
///     "command": "cargo test"
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInput {
    /// Session ID from Claude Code
    pub session_id: String,

    /// Path to the transcript file
    pub transcript_path: PathBuf,

    /// Current working directory
    pub cwd: PathBuf,

    /// The hook event type
    pub hook_event_name: HookEventName,

    /// Name of the tool being called (for tool events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    /// Tool input parameters (for tool events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<Value>,

    /// Tool response (for PostToolUse)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_response: Option<Value>,

    /// User prompt (for UserPromptSubmit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

impl HookInput {
    /// Check if this is a tool-related event
    pub fn is_tool_event(&self) -> bool {
        matches!(
            self.hook_event_name,
            HookEventName::PreToolUse | HookEventName::PostToolUse
        )
    }

    /// Get the tool name (returns empty string if not a tool event)
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_deref().unwrap_or("")
    }

    /// Get the tool input (returns null if not a tool event)
    pub fn tool_input(&self) -> &Value {
        static NULL: Value = Value::Null;
        self.tool_input.as_ref().unwrap_or(&NULL)
    }

    /// Get the tool response if present (PostToolUse only)
    pub fn tool_response(&self) -> Option<&Value> {
        self.tool_response.as_ref()
    }

    /// Get the user prompt (UserPromptSubmit only)
    pub fn user_prompt(&self) -> Option<&str> {
        self.prompt.as_deref()
    }
}

/// Session context for MAOS operations
///
/// Provides comprehensive session state including agent tracking,
/// workspace isolation, and multi-agent coordination. This context
/// is extracted from hook inputs and used throughout MAOS for
/// security and orchestration decisions.
///
/// # Key Responsibilities
///
/// - **Agent Tracking**: Monitor active agents and relationships
/// - **Workspace Isolation**: Enforce git worktree boundaries
/// - **Session State**: Maintain consistent state across hooks
/// - **Security Context**: Provide path constraints and validation
///
/// # Example
///
/// ```
/// use maos_core::messages::{SessionContext, HookInput, HookEventName};
/// use std::path::PathBuf;
///
/// let hook_input = HookInput {
///     session_id: "sess_12345678-1234-1234-1234-123456789012".to_string(),
///     transcript_path: PathBuf::from("/tmp/transcript"),
///     cwd: PathBuf::from("/workspace"),
///     hook_event_name: HookEventName::PreToolUse,
///     tool_name: Some("Bash".to_string()),
///     tool_input: Some(serde_json::json!({"command": "ls"})),
///     tool_response: None,
///     prompt: None,
/// };
/// let context = SessionContext::from_hook_input(&hook_input)?;
///
/// // Use context for security checks
/// if context.cwd.starts_with(&context.workspace_root) {
///     // Tool operating within workspace bounds
/// }
/// # Ok::<(), maos_core::MaosError>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Unique session identifier
    pub session_id: SessionId,

    /// Current agent ID (if operating within an agent)
    pub agent_id: Option<AgentId>,

    /// Type of agent (backend-engineer, qa-engineer, etc)
    pub agent_type: Option<String>,

    /// Root workspace directory for this session
    pub workspace_root: PathBuf,

    /// List of currently active agents in session
    pub active_agents: Vec<AgentId>,

    /// Parent agent that spawned this context
    pub parent_agent: Option<AgentId>,

    /// Current working directory
    pub cwd: PathBuf,

    /// Path to session transcript file
    pub transcript_path: Option<PathBuf>,
}

impl SessionContext {
    /// Create context from hook input
    pub fn from_hook_input(input: &HookInput) -> Result<Self> {
        let session_id = SessionId::from_str(&input.session_id)
            .map_err(|e| crate::MaosError::InvalidInput { message: e })?;

        // Try to extract agent_id from environment or tool input
        let agent_id = if let Some(tool_input) = &input.tool_input {
            tool_input
                .get("agent_id")
                .and_then(|v| v.as_str())
                .and_then(|s| AgentId::from_str(s).ok())
        } else {
            None
        };

        Ok(Self {
            session_id,
            agent_id,
            agent_type: None,
            workspace_root: input
                .cwd
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/workspace")),
            active_agents: Vec::new(),
            parent_agent: None,
            cwd: input.cwd.clone(),
            transcript_path: Some(input.transcript_path.clone()),
        })
    }
}

/// Path constraints for workspace isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConstraint {
    pub allowed_paths: Vec<PathBuf>,
    pub blocked_patterns: Vec<String>,
    pub max_depth: Option<usize>,
}

impl PathConstraint {
    /// Check if a path is allowed by this constraint
    pub fn is_allowed(&self, path: &Path) -> bool {
        // Check if path starts with any allowed path
        let mut is_within_allowed = false;
        for allowed in &self.allowed_paths {
            if path.starts_with(allowed) {
                is_within_allowed = true;
                break;
            }
        }

        if !is_within_allowed {
            return false;
        }

        // Check blocked patterns
        let path_str = path.to_string_lossy();
        for pattern in &self.blocked_patterns {
            if pattern.contains('*') {
                // Simple glob matching for patterns with wildcards
                if pattern.starts_with('*') && pattern.ends_with('*') {
                    // Pattern like *foo* - check if path contains the middle part
                    let middle = &pattern[1..pattern.len() - 1];
                    if path_str.contains(middle) {
                        return false;
                    }
                } else if let Some(suffix) = pattern.strip_prefix('*') {
                    // Pattern like *.log - check suffix
                    if path_str.ends_with(suffix) {
                        return false;
                    }
                } else if pattern.ends_with('*') {
                    // Pattern like test_* - check prefix
                    let prefix = &pattern[..pattern.len() - 1];
                    if path_str.starts_with(prefix) {
                        return false;
                    }
                } else {
                    // Pattern like test_*_backup - check prefix and suffix
                    let parts: Vec<&str> = pattern.splitn(2, '*').collect();
                    if parts.len() == 2 {
                        let prefix = parts[0];
                        let suffix = parts[1];
                        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if filename.starts_with(prefix) && filename.ends_with(suffix) {
                            return false;
                        }
                    }
                }
            } else if path_str.contains(pattern) {
                return false;
            }
        }

        // Check depth
        if let Some(max_depth) = self.max_depth {
            let depth = path.components().count();
            // Find the depth of the allowed path
            for allowed in &self.allowed_paths {
                if path.starts_with(allowed) {
                    let allowed_depth = allowed.components().count();
                    if depth - allowed_depth > max_depth {
                        return false;
                    }
                    break;
                }
            }
        }

        true
    }
}

/// Pre-tool message for MAOS processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreToolMessage {
    pub tool_call: ToolCall,
    pub session_context: SessionContext,
    pub workspace_constraints: Vec<PathConstraint>,
}

impl PreToolMessage {
    /// Create from hook input
    pub fn from_hook_input(input: HookInput) -> Result<Self> {
        if !input.is_tool_event() {
            return Err(crate::MaosError::InvalidInput {
                message: "PreToolMessage requires a tool event".to_string(),
            });
        }

        let session_context = SessionContext::from_hook_input(&input)?;

        let tool_call = ToolCall {
            id: ToolCallId::generate(),
            tool_name: input.tool_name.unwrap_or_default(),
            parameters: input.tool_input.unwrap_or(Value::Null),
            timestamp: chrono::Utc::now(),
            session_id: Some(session_context.session_id.clone()),
            agent_id: session_context.agent_id.clone(),
        };

        Ok(Self {
            tool_call,
            session_context,
            workspace_constraints: Vec::new(),
        })
    }
}

/// Post-tool message for MAOS processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToolMessage {
    pub tool_call: ToolCall,
    pub tool_result: ToolResult,
    pub session_context: SessionContext,
}

impl PostToolMessage {
    /// Create from hook input
    pub fn from_hook_input(input: HookInput) -> Result<Self> {
        if input.hook_event_name != HookEventName::PostToolUse {
            return Err(crate::MaosError::InvalidInput {
                message: "PostToolMessage requires PostToolUse event".to_string(),
            });
        }

        let session_context = SessionContext::from_hook_input(&input)?;

        let tool_call = ToolCall {
            id: ToolCallId::generate(),
            tool_name: input.tool_name.unwrap_or_default(),
            parameters: input.tool_input.unwrap_or(Value::Null),
            timestamp: chrono::Utc::now(),
            session_id: Some(session_context.session_id.clone()),
            agent_id: session_context.agent_id.clone(),
        };

        let tool_response = input.tool_response.as_ref();
        let success = tool_response
            .and_then(|r| r.get("success"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let output = tool_response
            .and_then(|r| r.get("output"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let tool_result = ToolResult {
            tool_call_id: tool_call.id.clone(),
            success,
            output,
            error: None,
            execution_time_ms: 0,
            timestamp: chrono::Utc::now(),
        };

        Ok(Self {
            tool_call,
            tool_result,
            session_context,
        })
    }
}

/// Hook output that gets displayed to the user
///
/// According to Claude Code documentation, hooks can output to stdout/stderr
/// and this output is shown to the user. This struct captures both streams
/// along with the exit code to provide complete hook execution context.
///
/// # Example
///
/// ```
/// use maos_core::messages::{HookOutput, HookResponse};
///
/// let output = HookOutput {
///     stdout: Some("✅ Tool validated successfully".to_string()),
///     stderr: None,
///     exit_code: 0,
///     response: HookResponse::Allow,
/// };
///
/// // Display to user if there's output
/// if let Some(stdout) = &output.stdout {
///     println!("{}", stdout);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookOutput {
    /// Standard output from the hook (displayed to user)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,

    /// Standard error from the hook (displayed to user)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,

    /// Exit code from the hook (0 = allow, 2 = block)
    pub exit_code: i32,

    /// Parsed response based on exit code
    pub response: HookResponse,
}

impl HookOutput {
    /// Create output from hook execution
    pub fn from_execution(stdout: Option<String>, stderr: Option<String>, exit_code: i32) -> Self {
        let response = match exit_code {
            0 => HookResponse::Allow,
            2 => HookResponse::Block {
                reason: stderr
                    .clone()
                    .unwrap_or_else(|| "Hook blocked execution".to_string()),
            },
            _ => HookResponse::Allow, // Default to allow for unknown codes
        };

        Self {
            stdout,
            stderr,
            exit_code,
            response,
        }
    }

    /// Check if there's any output to display
    pub fn has_output(&self) -> bool {
        self.stdout.is_some() || self.stderr.is_some()
    }

    /// Get combined output for display
    pub fn display_output(&self) -> Option<String> {
        match (&self.stdout, &self.stderr) {
            (Some(out), Some(err)) => Some(format!("{out}\n{err}")),
            (Some(out), None) => Some(out.clone()),
            (None, Some(err)) => Some(err.clone()),
            (None, None) => None,
        }
    }
}

/// Hook response format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "data")]
pub enum HookResponse {
    /// Allow tool execution (exit 0)
    Allow,

    /// Block tool execution (exit 2)
    Block { reason: String },

    /// Modify tool parameters (not yet supported by Claude Code)
    Modify { parameters: Value },

    /// Redirect to different tool (future feature)
    Redirect {
        tool_name: String,
        parameters: Value,
    },
}

impl HookResponse {
    /// Convert to exit code for hook script
    pub fn to_exit_code(&self) -> i32 {
        match self {
            HookResponse::Allow => 0,
            HookResponse::Block { .. } => 2,
            _ => 0, // Unsupported actions default to allow
        }
    }
}
