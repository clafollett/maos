//! Claude Code hook event constants and types
//!
//! This module provides a single source of truth for all Claude Code hook event names
//! to prevent string duplication and typos across the codebase.

use std::fmt::{self, Display};

/// Claude Code hook event name constants
///
/// These constants represent the exact strings that Claude Code sends in the
/// `hook_event_name` field of JSON messages. They must match Claude Code's
/// implementation exactly.
pub mod event_constants {
    /// Pre-tool execution hook event
    pub const PRE_TOOL_USE: &str = "pre_tool_use";

    /// Post-tool execution hook event  
    pub const POST_TOOL_USE: &str = "post_tool_use";

    /// Notification display hook event
    pub const NOTIFICATION: &str = "notification";

    /// Session stop hook event (includes --chat mode)
    pub const STOP: &str = "stop";

    /// Subagent completion hook event
    pub const SUBAGENT_STOP: &str = "subagent_stop";

    /// User prompt submission hook event
    pub const USER_PROMPT_SUBMIT: &str = "user_prompt_submit";

    /// Pre-conversation compaction hook event
    pub const PRE_COMPACT: &str = "pre_compact";

    /// Session start hook event
    pub const SESSION_START: &str = "session_start";

    /// All valid hook event names as a slice
    pub const ALL_EVENTS: &[&str] = &[
        PRE_TOOL_USE,
        POST_TOOL_USE,
        NOTIFICATION,
        STOP,
        SUBAGENT_STOP,
        USER_PROMPT_SUBMIT,
        PRE_COMPACT,
        SESSION_START,
    ];
}

/// Category constants for hook event classification
///
/// These constants represent the logical groupings of hook events
/// for metrics, logging, and organizational purposes.
pub mod category_constants {
    /// Tool-related hook events (pre/post tool execution)
    pub const TOOL_HOOKS: &str = "tool-hooks";

    /// Notification display events
    pub const NOTIFICATIONS: &str = "notifications";

    /// Session lifecycle events (start, stop, etc.)
    pub const LIFECYCLE: &str = "lifecycle";

    /// User input and interaction events
    pub const USER_INPUT: &str = "user-input";

    /// Maintenance and system events (compaction, etc.)
    pub const MAINTENANCE: &str = "maintenance";

    /// All valid category names as a slice
    pub const ALL_CATEGORIES: &[&str] = &[
        TOOL_HOOKS,
        NOTIFICATIONS,
        LIFECYCLE,
        USER_INPUT,
        MAINTENANCE,
    ];
}

/// Strongly-typed enum for Claude Code hook events
///
/// This enum provides type safety and ensures all hook events are handled.
/// Use this instead of raw strings when possible.
///
/// 🔥 TYPE SAFETY ENHANCEMENT: Supports serialization and string conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HookEvent {
    /// Pre-tool execution hook
    PreToolUse,
    /// Post-tool execution hook
    PostToolUse,
    /// Notification display hook
    Notification,
    /// Session stop hook
    Stop,
    /// Subagent completion hook
    SubagentStop,
    /// User prompt submission hook
    UserPromptSubmit,
    /// Pre-conversation compaction hook
    PreCompact,
    /// Session start hook
    SessionStart,
}

impl HookEvent {
    /// Get all hook events
    pub const fn all() -> &'static [HookEvent] {
        &[
            HookEvent::PreToolUse,
            HookEvent::PostToolUse,
            HookEvent::Notification,
            HookEvent::Stop,
            HookEvent::SubagentStop,
            HookEvent::UserPromptSubmit,
            HookEvent::PreCompact,
            HookEvent::SessionStart,
        ]
    }

    /// Parse hook event from string (fallible)
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            event_constants::PRE_TOOL_USE => Some(HookEvent::PreToolUse),
            event_constants::POST_TOOL_USE => Some(HookEvent::PostToolUse),
            event_constants::NOTIFICATION => Some(HookEvent::Notification),
            event_constants::STOP => Some(HookEvent::Stop),
            event_constants::SUBAGENT_STOP => Some(HookEvent::SubagentStop),
            event_constants::USER_PROMPT_SUBMIT => Some(HookEvent::UserPromptSubmit),
            event_constants::PRE_COMPACT => Some(HookEvent::PreCompact),
            event_constants::SESSION_START => Some(HookEvent::SessionStart),
            _ => None,
        }
    }

    /// Get the string representation
    ///
    /// This method provides direct access to the underlying string constant.
    /// For most use cases, prefer using the `Display` trait via `.to_string()`
    /// or string formatting, which provides the same result with automatic
    /// integration into Rust's formatting ecosystem.
    pub const fn as_str(&self) -> &'static str {
        match self {
            HookEvent::PreToolUse => event_constants::PRE_TOOL_USE,
            HookEvent::PostToolUse => event_constants::POST_TOOL_USE,
            HookEvent::Notification => event_constants::NOTIFICATION,
            HookEvent::Stop => event_constants::STOP,
            HookEvent::SubagentStop => event_constants::SUBAGENT_STOP,
            HookEvent::UserPromptSubmit => event_constants::USER_PROMPT_SUBMIT,
            HookEvent::PreCompact => event_constants::PRE_COMPACT,
            HookEvent::SessionStart => event_constants::SESSION_START,
        }
    }

    /// Check if this is a tool-related hook
    pub const fn is_tool_hook(&self) -> bool {
        matches!(self, HookEvent::PreToolUse | HookEvent::PostToolUse)
    }

    /// Check if this is a lifecycle hook
    pub const fn is_lifecycle_hook(&self) -> bool {
        matches!(
            self,
            HookEvent::Stop | HookEvent::SubagentStop | HookEvent::SessionStart
        )
    }

    /// Get the category for metrics/logging
    pub const fn category(&self) -> &'static str {
        match self {
            HookEvent::PreToolUse | HookEvent::PostToolUse => category_constants::TOOL_HOOKS,
            HookEvent::Notification => category_constants::NOTIFICATIONS,
            HookEvent::Stop | HookEvent::SubagentStop | HookEvent::SessionStart => {
                category_constants::LIFECYCLE
            }
            HookEvent::UserPromptSubmit => category_constants::USER_INPUT,
            HookEvent::PreCompact => category_constants::MAINTENANCE,
        }
    }
}

impl Display for HookEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            HookEvent::PreToolUse => event_constants::PRE_TOOL_USE,
            HookEvent::PostToolUse => event_constants::POST_TOOL_USE,
            HookEvent::Notification => event_constants::NOTIFICATION,
            HookEvent::Stop => event_constants::STOP,
            HookEvent::SubagentStop => event_constants::SUBAGENT_STOP,
            HookEvent::UserPromptSubmit => event_constants::USER_PROMPT_SUBMIT,
            HookEvent::PreCompact => event_constants::PRE_COMPACT,
            HookEvent::SessionStart => event_constants::SESSION_START,
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for HookEvent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        HookEvent::try_from_str(s).ok_or_else(|| format!("Invalid hook event: {s}"))
    }
}

/// 🔥 TYPE SAFETY: Enable conversion from string references
impl TryFrom<&str> for HookEvent {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// 🔥 TYPE SAFETY: Enable conversion from owned strings
impl TryFrom<String> for HookEvent {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_constants_are_valid() {
        for &event_str in event_constants::ALL_EVENTS {
            assert!(HookEvent::try_from_str(event_str).is_some());
        }
    }

    #[test]
    fn test_enum_string_roundtrip() {
        for event in HookEvent::all() {
            let string = event.as_str();
            let parsed = HookEvent::try_from_str(string).unwrap();
            assert_eq!(*event, parsed);
        }
    }

    #[test]
    fn test_display_trait() {
        // Test Display trait implementation
        assert_eq!(
            HookEvent::PreToolUse.to_string(),
            event_constants::PRE_TOOL_USE
        );
        assert_eq!(
            HookEvent::Notification.to_string(),
            event_constants::NOTIFICATION
        );

        // Test formatting integration
        assert_eq!(
            format!("{}", HookEvent::PreToolUse),
            event_constants::PRE_TOOL_USE
        );
        assert_eq!(
            format!("{}", HookEvent::SessionStart),
            event_constants::SESSION_START
        );

        // Test Debug trait (automatically derived from Display)
        assert_eq!(format!("{:?}", HookEvent::PreToolUse), "PreToolUse");
    }

    #[test]
    fn test_as_str_vs_display_equivalence() {
        // as_str() and Display should return the same string
        for event in HookEvent::all() {
            assert_eq!(event.as_str(), event.to_string());
            assert_eq!(event.as_str(), format!("{event}"));
        }
    }

    #[test]
    fn test_categorization() {
        assert!(HookEvent::PreToolUse.is_tool_hook());
        assert!(!HookEvent::Notification.is_tool_hook());

        assert!(HookEvent::SessionStart.is_lifecycle_hook());
        assert!(!HookEvent::PreToolUse.is_lifecycle_hook());

        assert_eq!(
            HookEvent::PreToolUse.category(),
            category_constants::TOOL_HOOKS
        );
        assert_eq!(
            HookEvent::Notification.category(),
            category_constants::NOTIFICATIONS
        );
    }

    #[test]
    fn test_invalid_event() {
        assert!(HookEvent::try_from_str("invalid_event").is_none());
        assert!(HookEvent::try_from_str("").is_none());
    }
}
