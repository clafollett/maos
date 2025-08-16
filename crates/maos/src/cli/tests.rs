#[cfg(test)]
mod cli_tests {
    use crate::cli::{Cli, Commands};
    use clap::{CommandFactory, Parser};
    use maos_core::category_constants::*;

    // ===== PARSING TESTS =====

    #[test]
    fn test_parse_pre_tool_use() {
        let cli = Cli::try_parse_from(["maos", "pre-tool-use"]).unwrap();
        assert!(matches!(cli.command, Commands::PreToolUse));
    }

    #[test]
    fn test_parse_post_tool_use() {
        let cli = Cli::try_parse_from(["maos", "post-tool-use"]).unwrap();
        assert!(matches!(cli.command, Commands::PostToolUse));
    }

    #[test]
    fn test_parse_notify() {
        let cli = Cli::try_parse_from(["maos", "notify"]).unwrap();
        assert!(matches!(cli.command, Commands::Notify));
    }

    #[test]
    fn test_parse_stop_no_flag() {
        let cli = Cli::try_parse_from(["maos", maos_core::hook_constants::STOP]).unwrap();
        match cli.command {
            Commands::Stop { chat } => assert!(!chat),
            _ => panic!("Expected Stop command"),
        }
    }

    #[test]
    fn test_parse_stop_with_chat() {
        let cli = Cli::try_parse_from(["maos", maos_core::hook_constants::STOP, "--chat"]).unwrap();
        match cli.command {
            Commands::Stop { chat } => assert!(chat),
            _ => panic!("Expected Stop command"),
        }
    }

    #[test]
    fn test_parse_subagent_stop() {
        let cli = Cli::try_parse_from(["maos", "subagent-stop"]).unwrap();
        assert!(matches!(cli.command, Commands::SubagentStop));
    }

    #[test]
    fn test_parse_user_prompt_submit_no_flag() {
        let cli = Cli::try_parse_from(["maos", "user-prompt-submit"]).unwrap();
        match cli.command {
            Commands::UserPromptSubmit { validate } => assert!(!validate),
            _ => panic!("Expected UserPromptSubmit command"),
        }
    }

    #[test]
    fn test_parse_user_prompt_submit_with_validate() {
        let cli = Cli::try_parse_from(["maos", "user-prompt-submit", "--validate"]).unwrap();
        match cli.command {
            Commands::UserPromptSubmit { validate } => assert!(validate),
            _ => panic!("Expected UserPromptSubmit command"),
        }
    }

    #[test]
    fn test_parse_pre_compact() {
        let cli = Cli::try_parse_from(["maos", "pre-compact"]).unwrap();
        assert!(matches!(cli.command, Commands::PreCompact));
    }

    #[test]
    fn test_parse_session_start() {
        let cli = Cli::try_parse_from(["maos", "session-start"]).unwrap();
        assert!(matches!(cli.command, Commands::SessionStart));
    }

    // ===== ERROR HANDLING TESTS =====

    #[test]
    fn test_invalid_command_returns_error() {
        let result = Cli::try_parse_from(["maos", "invalid-command"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_command_returns_error() {
        let result = Cli::try_parse_from(["maos"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_flag_returns_error() {
        let result = Cli::try_parse_from(["maos", maos_core::hook_constants::STOP, "--invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_help_flag_returns_error() {
        // Clap exits with error on help
        let result = Cli::try_parse_from(["maos", "--help"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_version_flag_returns_error() {
        // Clap exits with error on version
        let result = Cli::try_parse_from(["maos", "--version"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_subcommand_help_returns_error() {
        let result = Cli::try_parse_from(["maos", "stop", "--help"]);
        assert!(result.is_err());
    }

    // ===== HOOK EVENT NAME TESTS =====

    #[test]
    fn test_hook_event_name_pre_tool_use() {
        assert_eq!(
            Commands::PreToolUse.hook_event_name(),
            maos_core::hook_constants::PRE_TOOL_USE
        );
    }

    #[test]
    fn test_hook_event_name_post_tool_use() {
        assert_eq!(
            Commands::PostToolUse.hook_event_name(),
            maos_core::hook_constants::POST_TOOL_USE
        );
    }

    #[test]
    fn test_hook_event_name_notification() {
        assert_eq!(
            Commands::Notify.hook_event_name(),
            maos_core::hook_constants::NOTIFICATION
        );
    }

    #[test]
    fn test_hook_event_name_stop() {
        assert_eq!(
            Commands::Stop { chat: false }.hook_event_name(),
            maos_core::hook_constants::STOP
        );
        assert_eq!(
            Commands::Stop { chat: true }.hook_event_name(),
            maos_core::hook_constants::STOP
        );
    }

    #[test]
    fn test_hook_event_name_subagent_stop() {
        assert_eq!(
            Commands::SubagentStop.hook_event_name(),
            maos_core::hook_constants::SUBAGENT_STOP
        );
    }

    #[test]
    fn test_hook_event_name_user_prompt_submit() {
        assert_eq!(
            Commands::UserPromptSubmit { validate: false }.hook_event_name(),
            maos_core::hook_constants::USER_PROMPT_SUBMIT
        );
        assert_eq!(
            Commands::UserPromptSubmit { validate: true }.hook_event_name(),
            maos_core::hook_constants::USER_PROMPT_SUBMIT
        );
    }

    #[test]
    fn test_hook_event_name_pre_compact() {
        assert_eq!(
            Commands::PreCompact.hook_event_name(),
            maos_core::hook_constants::PRE_COMPACT
        );
    }

    #[test]
    fn test_hook_event_name_session_start() {
        assert_eq!(
            Commands::SessionStart.hook_event_name(),
            maos_core::hook_constants::SESSION_START
        );
    }

    // ===== STDIN EXPECTATION TESTS =====

    #[test]
    fn test_all_commands_expect_stdin() {
        assert!(Commands::PreToolUse.expects_stdin());
        assert!(Commands::PostToolUse.expects_stdin());
        assert!(Commands::Notify.expects_stdin());
        assert!(Commands::Stop { chat: false }.expects_stdin());
        assert!(Commands::SubagentStop.expects_stdin());
        assert!(Commands::UserPromptSubmit { validate: false }.expects_stdin());
        assert!(Commands::PreCompact.expects_stdin());
        assert!(Commands::SessionStart.expects_stdin());
    }

    // ===== DISPLAY TRAIT TESTS =====

    #[test]
    fn test_display_pre_tool_use() {
        assert_eq!(format!("{}", Commands::PreToolUse), "pre-tool-use");
    }

    #[test]
    fn test_display_post_tool_use() {
        assert_eq!(format!("{}", Commands::PostToolUse), "post-tool-use");
    }

    #[test]
    fn test_display_notify() {
        assert_eq!(format!("{}", Commands::Notify), "notify");
    }

    #[test]
    fn test_display_stop() {
        assert_eq!(format!("{}", Commands::Stop { chat: false }), "stop");
        assert_eq!(format!("{}", Commands::Stop { chat: true }), "stop");
    }

    #[test]
    fn test_display_subagent_stop() {
        assert_eq!(format!("{}", Commands::SubagentStop), "subagent-stop");
    }

    #[test]
    fn test_display_user_prompt_submit() {
        assert_eq!(
            format!("{}", Commands::UserPromptSubmit { validate: false }),
            "user-prompt-submit"
        );
        assert_eq!(
            format!("{}", Commands::UserPromptSubmit { validate: true }),
            "user-prompt-submit"
        );
    }

    #[test]
    fn test_display_pre_compact() {
        assert_eq!(format!("{}", Commands::PreCompact), "pre-compact");
    }

    #[test]
    fn test_display_session_start() {
        assert_eq!(format!("{}", Commands::SessionStart), "session-start");
    }

    // ===== CATEGORY TESTS =====

    #[test]
    fn test_category_tool_hooks() {
        assert_eq!(Commands::PreToolUse.category(), TOOL_HOOKS);
        assert_eq!(Commands::PostToolUse.category(), TOOL_HOOKS);
    }

    #[test]
    fn test_category_notifications() {
        assert_eq!(Commands::Notify.category(), NOTIFICATIONS);
    }

    #[test]
    fn test_category_lifecycle() {
        assert_eq!(Commands::Stop { chat: false }.category(), LIFECYCLE);
        assert_eq!(Commands::SubagentStop.category(), LIFECYCLE);
        assert_eq!(Commands::SessionStart.category(), LIFECYCLE);
    }

    #[test]
    fn test_category_user_input() {
        assert_eq!(
            Commands::UserPromptSubmit { validate: false }.category(),
            USER_INPUT
        );
    }

    #[test]
    fn test_category_maintenance() {
        assert_eq!(Commands::PreCompact.category(), MAINTENANCE);
    }

    // ===== HOOK TYPE TESTS =====

    #[test]
    fn test_is_lifecycle_hook() {
        assert!(!Commands::PreToolUse.is_lifecycle_hook());
        assert!(!Commands::PostToolUse.is_lifecycle_hook());
        assert!(!Commands::Notify.is_lifecycle_hook());
        assert!(Commands::Stop { chat: false }.is_lifecycle_hook());
        assert!(Commands::SubagentStop.is_lifecycle_hook());
        assert!(!Commands::UserPromptSubmit { validate: false }.is_lifecycle_hook());
        assert!(!Commands::PreCompact.is_lifecycle_hook());
        assert!(Commands::SessionStart.is_lifecycle_hook());
    }

    #[test]
    fn test_is_tool_hook() {
        assert!(Commands::PreToolUse.is_tool_hook());
        assert!(Commands::PostToolUse.is_tool_hook());
        assert!(!Commands::Notify.is_tool_hook());
        assert!(!Commands::Stop { chat: false }.is_tool_hook());
        assert!(!Commands::SubagentStop.is_tool_hook());
        assert!(!Commands::UserPromptSubmit { validate: false }.is_tool_hook());
        assert!(!Commands::PreCompact.is_tool_hook());
        assert!(!Commands::SessionStart.is_tool_hook());
    }

    // ===== CLONE TRAIT TEST =====

    #[test]
    fn test_commands_are_cloneable() {
        let cmd = Commands::Stop { chat: true };
        let cloned = cmd.clone();
        match cloned {
            Commands::Stop { chat } => assert!(chat),
            _ => panic!("Clone failed"),
        }
    }

    // ===== DEBUG TRAIT TEST =====

    #[test]
    fn test_commands_have_debug() {
        let cmd = Commands::PreToolUse;
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("PreToolUse"));
    }

    // ===== CLI METADATA TESTS =====

    #[test]
    fn test_cli_has_correct_name() {
        let cmd = Cli::command();
        assert_eq!(cmd.get_name(), "maos");
    }

    #[test]
    fn test_cli_has_about_text() {
        let cmd = Cli::command();
        assert!(cmd.get_about().is_some());
        assert!(
            cmd.get_about()
                .unwrap()
                .to_string()
                .contains("Multi-Agent Orchestration System")
        );
    }

    #[test]
    fn test_cli_has_version() {
        let cmd = Cli::command();
        assert!(cmd.get_version().is_some());
    }

    // ===== PERFORMANCE TEST =====

    #[test]
    fn test_parsing_performance_under_1ms() {
        use std::time::Instant;

        let commands = vec![
            vec!["maos", "pre-tool-use"],
            vec!["maos", "post-tool-use"],
            vec!["maos", "notify"],
            vec!["maos", "stop", "--chat"],
            vec!["maos", "subagent-stop"],
            vec!["maos", "user-prompt-submit", "--validate"],
            vec!["maos", "pre-compact"],
            vec!["maos", "session-start"],
        ];

        // Warm up
        for cmd in &commands {
            let _ = Cli::try_parse_from(cmd.clone());
        }

        // Actual test
        let start = Instant::now();
        for _ in 0..100 {
            for cmd in &commands {
                let _ = Cli::try_parse_from(cmd.clone());
            }
        }
        let elapsed = start.elapsed();

        // 800 parses should take less than 800ms (< 1ms per parse)
        assert!(
            elapsed.as_millis() < 800,
            "Parsing took {}ms",
            elapsed.as_millis()
        );
    }
}
