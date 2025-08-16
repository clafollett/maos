use clap::{Parser, Subcommand};

/// Multi-Agent Orchestration System CLI
#[derive(Parser, Debug)]
#[command(name = "maos")]
#[command(about = "Multi-Agent Orchestration System")]
#[command(version)]
pub struct Cli {
    /// The hook command to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available hook commands matching Claude Code's 8 hook events
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Process pre-tool-use hook (runs before tool execution)
    #[command(name = "pre-tool-use")]
    PreToolUse,

    /// Process post-tool-use hook (runs after tool completion)
    #[command(name = "post-tool-use")]
    PostToolUse,

    /// Handle notification messages from Claude Code
    Notify,

    /// Process session stop events (main agent finished)
    Stop {
        /// Export chat transcript to logs
        #[arg(long)]
        chat: bool,
    },

    /// Handle subagent stop events (Task tool completed)
    #[command(name = "subagent-stop")]
    SubagentStop,

    /// Process user prompt submissions
    #[command(name = "user-prompt-submit")]
    UserPromptSubmit {
        /// Validate prompt before processing
        #[arg(long)]
        validate: bool,
    },

    /// Handle pre-compact events (before conversation compaction)
    #[command(name = "pre-compact")]
    PreCompact,

    /// Handle session start events (new or resumed session)
    #[command(name = "session-start")]
    SessionStart,
}

impl Commands {
    /// Returns the Claude Code hook event name for this command
    pub fn hook_event_name(&self) -> &'static str {
        use maos_core::hook_constants::*;
        match self {
            Commands::PreToolUse => PRE_TOOL_USE,
            Commands::PostToolUse => POST_TOOL_USE,
            Commands::Notify => NOTIFICATION,
            Commands::Stop { .. } => STOP,
            Commands::SubagentStop => SUBAGENT_STOP,
            Commands::UserPromptSubmit { .. } => USER_PROMPT_SUBMIT,
            Commands::PreCompact => PRE_COMPACT,
            Commands::SessionStart => SESSION_START,
        }
    }

    /// Returns true if this command expects JSON input on stdin
    pub fn expects_stdin(&self) -> bool {
        true // All Claude Code hooks receive JSON via stdin
    }

    /// Returns the category of this command for logging/metrics
    pub fn category(&self) -> &'static str {
        use maos_core::category_constants::*;
        match self {
            Commands::PreToolUse | Commands::PostToolUse => TOOL_HOOKS,
            Commands::Notify => NOTIFICATIONS,
            Commands::Stop { .. } | Commands::SubagentStop | Commands::SessionStart => LIFECYCLE,
            Commands::UserPromptSubmit { .. } => USER_INPUT,
            Commands::PreCompact => MAINTENANCE,
        }
    }

    /// Returns true if this is a lifecycle hook
    pub fn is_lifecycle_hook(&self) -> bool {
        matches!(
            self,
            Commands::Stop { .. } | Commands::SubagentStop | Commands::SessionStart
        )
    }

    /// Returns true if this is a tool-related hook
    pub fn is_tool_hook(&self) -> bool {
        matches!(self, Commands::PreToolUse | Commands::PostToolUse)
    }
}

impl std::fmt::Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Commands::PreToolUse => write!(f, "pre-tool-use"),
            Commands::PostToolUse => write!(f, "post-tool-use"),
            Commands::Notify => write!(f, "notify"),
            Commands::Stop { .. } => write!(f, "stop"),
            Commands::SubagentStop => write!(f, "subagent-stop"),
            Commands::UserPromptSubmit { .. } => write!(f, "user-prompt-submit"),
            Commands::PreCompact => write!(f, "pre-compact"),
            Commands::SessionStart => write!(f, "session-start"),
        }
    }
}
