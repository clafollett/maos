//! Message formats for Claude Code hook integration and session state persistence
//!
//! This module provides comprehensive message types for MAOS integration with Claude Code,
//! including hook messages, notifications, and session state management. It supports both
//! the legacy format used by our Python hooks and the documented Claude Code format.
//!
//! # Components
//!
//! - **Hook Messages**: Pre/post tool messages for Claude Code hooks
//! - **Notifications**: TTS-ready notification messages
//! - **Session State**: Persistent session state files
//! - **Validation**: JSON schema validation for all message types
//!
//! # Format Compatibility
//!
//! This module handles two different hook input formats:
//!
//! 1. **Legacy Format** (used by our Python hooks):
//!    ```json
//!    {
//!      "tool_name": "Edit",
//!      "tool_input": { "file_path": "..." },
//!      "metadata": { }
//!    }
//!    ```
//!
//! 2. **Documented Format** (from Claude Code documentation):
//!    ```json
//!    {
//!      "session_id": "sess_123",
//!      "transcript_path": "/path/to/transcript",
//!      "cwd": "/current/directory",
//!      "hook_event_name": "PreToolUse",
//!      "tool_name": "Edit",
//!      "tool_input": { "file_path": "..." }
//!    }
//!    ```
//!
//! # Example
//!
//! ```no_run
//! use maos_core::messages::{HookInput, HookResponse};
//! use serde_json::json;
//!
//! // Parse hook input (works with both formats)
//! let input = json!({
//!     "tool_name": "Edit",
//!     "tool_input": { "file_path": "test.rs" }
//! });
//! let hook_input: HookInput = serde_json::from_value(input)?;
//!
//! // Create response
//! let response = HookResponse::Allow;
//! println!("Exit code: {}", response.to_exit_code());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod hook;
pub mod notification;
pub mod session_state;
pub mod validation;

// Re-export commonly used types
pub use hook::{
    HookEventName, HookInput, HookOutput, HookResponse, PathConstraint, PostToolMessage,
    PreToolMessage, SessionContext,
};
pub use notification::{NotificationMessage, NotificationType, NotificationUrgency};
pub use session_state::{
    AgentInfo, AgentStatus, AgentsFile, FileLock, LockType, LocksFile, SessionDirectory,
    SessionFile, SessionStatus,
};
pub use validation::{SchemaError, SchemaValidator};
