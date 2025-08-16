//! Command handler implementations for Claude Code hooks
//!
//! This module provides stub implementations for all 8 Claude Code hook events.
//! Each handler implements the [`CommandHandler`] trait and provides basic validation
//! and placeholder logic. Full business logic implementation is planned for PRD-06.
//!
//! # Architecture
//!
//! All handlers follow a consistent pattern:
//! - [`CommandHandler::execute`] - Main hook processing logic
//! - [`CommandHandler::name`] - Returns the hook event constant
//! - [`CommandHandler::validate_input`] - Validates hook event name matches
//!
//! # Hook Events
//!
//! - [`PreToolUseHandler`] - Processes pre-tool execution events
//! - [`PostToolUseHandler`] - Processes post-tool execution events  
//! - [`NotificationHandler`] - Handles notification display events
//! - [`StopHandler`] - Manages session termination events
//! - [`SubagentStopHandler`] - Handles sub-agent completion events
//! - [`UserPromptSubmitHandler`] - Processes user prompt submission events
//! - [`PreCompactHandler`] - Manages pre-conversation compaction events
//! - [`SessionStartHandler`] - Handles session initialization events
//!
//! [`CommandHandler`]: crate::cli::handler::CommandHandler

pub mod notification;
pub mod post_tool_use;
pub mod pre_compact;
pub mod pre_tool_use;
pub mod session_start;
pub mod stop;
pub mod subagent_stop;
pub mod user_prompt_submit;

// Re-export all handlers
pub use notification::NotificationHandler;
pub use post_tool_use::PostToolUseHandler;
pub use pre_compact::PreCompactHandler;
pub use pre_tool_use::PreToolUseHandler;
pub use session_start::SessionStartHandler;
pub use stop::StopHandler;
pub use subagent_stop::SubagentStopHandler;
pub use user_prompt_submit::UserPromptSubmitHandler;
