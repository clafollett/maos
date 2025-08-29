//! Type safety tests for Commands and HookEvent integration
//!
//! These tests ensure type safety between the CLI Commands and HookEvent enums

use maos::cli::Commands;
use maos_core::hook_events::HookEvent;
use maos_core::io::HookInput;
use serde_json::json;
use std::path::PathBuf;

#[test]
fn test_commands_to_hook_event_conversion() {
    // 🔗 TYPE SAFETY: Test Commands -> HookEvent conversion

    assert_eq!(Commands::PreToolUse.to_hook_event(), HookEvent::PreToolUse);
    assert_eq!(
        Commands::PostToolUse.to_hook_event(),
        HookEvent::PostToolUse
    );
    assert_eq!(Commands::Notify.to_hook_event(), HookEvent::Notification);
    assert_eq!(
        Commands::Stop { chat: false }.to_hook_event(),
        HookEvent::Stop
    );
    assert_eq!(
        Commands::Stop { chat: true }.to_hook_event(),
        HookEvent::Stop
    );
    assert_eq!(
        Commands::SubagentStop.to_hook_event(),
        HookEvent::SubagentStop
    );
    assert_eq!(
        Commands::UserPromptSubmit { validate: false }.to_hook_event(),
        HookEvent::UserPromptSubmit
    );
    assert_eq!(
        Commands::UserPromptSubmit { validate: true }.to_hook_event(),
        HookEvent::UserPromptSubmit
    );
    assert_eq!(Commands::PreCompact.to_hook_event(), HookEvent::PreCompact);
    assert_eq!(
        Commands::SessionStart.to_hook_event(),
        HookEvent::SessionStart
    );
}

#[test]
fn test_commands_hook_event_name_consistency() {
    // 🔗 TYPE SAFETY: Ensure Commands.hook_event_name() uses enum conversion

    let commands = vec![
        Commands::PreToolUse,
        Commands::PostToolUse,
        Commands::Notify,
        Commands::Stop { chat: false },
        Commands::SubagentStop,
        Commands::UserPromptSubmit { validate: false },
        Commands::PreCompact,
        Commands::SessionStart,
    ];

    for command in commands {
        let string_name = command.hook_event_name();
        let enum_name = command.to_hook_event().as_str();
        assert_eq!(
            string_name, enum_name,
            "Inconsistency for command: {command:?}"
        );
    }
}

#[test]
fn test_typed_validation_with_valid_inputs() {
    // ✅ TYPE SAFETY: Test that typed validation works for valid inputs

    // Test PreToolUse with required fields
    let pre_tool_input = HookInput {
        session_id: "test_session".to_string(),
        transcript_path: PathBuf::from("/tmp/test.jsonl"),
        cwd: PathBuf::from("/tmp"),
        hook_event_name: HookEvent::PreToolUse.to_string(),
        tool_name: Some("TestTool".to_string()),
        tool_input: Some(json!({"test": "value"})),
        ..Default::default()
    };

    assert!(pre_tool_input.validate().is_ok());

    // Test Notification with required message
    let notification_input = HookInput {
        session_id: "test_session".to_string(),
        transcript_path: PathBuf::from("/tmp/test.jsonl"),
        cwd: PathBuf::from("/tmp"),
        hook_event_name: HookEvent::Notification.to_string(),
        message: Some("Test notification".to_string()),
        ..Default::default()
    };

    assert!(notification_input.validate().is_ok());
}
