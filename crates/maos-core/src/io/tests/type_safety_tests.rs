//! 🔥 TYPE SAFETY TESTS for Hook Event Enum Integration
//!
//! These tests validate that the strongly-typed HookEvent enum provides
//! compile-time safety and prevents runtime errors from typos or invalid events.

use crate::hook_events::HookEvent;
use crate::hook_events::*;
use crate::io::HookInput;
use serde_json::json;
use std::path::PathBuf;

#[test]
fn test_hook_event_string_conversions() {
    // 🔥 TYPE SAFETY: Test all conversions work correctly

    // Test as_str() method
    assert_eq!(
        HookEvent::PreToolUse.as_str(),
        event_constants::PRE_TOOL_USE
    );
    assert_eq!(
        HookEvent::PostToolUse.as_str(),
        event_constants::POST_TOOL_USE
    );
    assert_eq!(
        HookEvent::Notification.as_str(),
        event_constants::NOTIFICATION
    );
    assert_eq!(HookEvent::Stop.as_str(), event_constants::STOP);
    assert_eq!(
        HookEvent::SubagentStop.as_str(),
        event_constants::SUBAGENT_STOP
    );
    assert_eq!(
        HookEvent::UserPromptSubmit.as_str(),
        event_constants::USER_PROMPT_SUBMIT
    );
    assert_eq!(HookEvent::PreCompact.as_str(), event_constants::PRE_COMPACT);
    assert_eq!(
        HookEvent::SessionStart.as_str(),
        event_constants::SESSION_START
    );
}

#[test]
fn test_hook_event_try_from_string() {
    // 🎯 TYPE SAFETY: Test TryFrom<&str> implementation

    assert_eq!(
        HookEvent::try_from(event_constants::PRE_TOOL_USE).unwrap(),
        HookEvent::PreToolUse
    );
    assert_eq!(
        HookEvent::try_from(event_constants::POST_TOOL_USE).unwrap(),
        HookEvent::PostToolUse
    );
    assert_eq!(
        HookEvent::try_from(event_constants::NOTIFICATION).unwrap(),
        HookEvent::Notification
    );
    assert_eq!(
        HookEvent::try_from(event_constants::STOP).unwrap(),
        HookEvent::Stop
    );
    assert_eq!(
        HookEvent::try_from(event_constants::SUBAGENT_STOP).unwrap(),
        HookEvent::SubagentStop
    );
    assert_eq!(
        HookEvent::try_from(event_constants::USER_PROMPT_SUBMIT).unwrap(),
        HookEvent::UserPromptSubmit
    );
    assert_eq!(
        HookEvent::try_from(event_constants::PRE_COMPACT).unwrap(),
        HookEvent::PreCompact
    );
    assert_eq!(
        HookEvent::try_from(event_constants::SESSION_START).unwrap(),
        HookEvent::SessionStart
    );

    // Test invalid event name
    assert!(HookEvent::try_from("invalid_event").is_err());
}

#[test]
fn test_hook_event_try_from_owned_string() {
    // 🎯 TYPE SAFETY: Test TryFrom<String> implementation

    let event_name = event_constants::PRE_TOOL_USE.to_string();
    assert_eq!(
        HookEvent::try_from(event_name).unwrap(),
        HookEvent::PreToolUse
    );

    let invalid_name = "typo_event".to_string();
    assert!(HookEvent::try_from(invalid_name).is_err());
}

#[test]
fn test_hook_event_parse() {
    // 🎯 TYPE SAFETY: Test FromStr implementation

    assert_eq!(
        event_constants::PRE_TOOL_USE.parse::<HookEvent>().unwrap(),
        HookEvent::PreToolUse
    );
    assert_eq!(
        event_constants::NOTIFICATION.parse::<HookEvent>().unwrap(),
        HookEvent::Notification
    );

    assert!("unknown_event".parse::<HookEvent>().is_err());
}

#[test]
fn test_hook_event_display() {
    // 🎯 TYPE SAFETY: Test Display trait implementation

    assert_eq!(
        format!("{}", HookEvent::PreToolUse),
        event_constants::PRE_TOOL_USE
    );
    assert_eq!(
        format!("{}", HookEvent::Notification),
        event_constants::NOTIFICATION
    );
    assert_eq!(HookEvent::Stop.to_string(), event_constants::STOP);
}

#[test]
fn test_hook_event_roundtrip_all_variants() {
    // 🔄 TYPE SAFETY: Ensure all enum variants roundtrip correctly

    for &event in HookEvent::all() {
        let string = event.as_str();
        let parsed = HookEvent::try_from(string).unwrap();
        assert_eq!(event, parsed);

        // Test via Display trait too
        let display_string = event.to_string();
        let parsed_display = display_string.parse::<HookEvent>().unwrap();
        assert_eq!(event, parsed_display);
    }
}

#[test]
fn test_hook_event_categories() {
    // 📂 TYPE SAFETY: Test category classification

    assert_eq!(
        HookEvent::PreToolUse.category(),
        category_constants::TOOL_HOOKS
    );
    assert_eq!(
        HookEvent::PostToolUse.category(),
        category_constants::TOOL_HOOKS
    );
    assert_eq!(
        HookEvent::Notification.category(),
        category_constants::NOTIFICATIONS
    );
    assert_eq!(HookEvent::Stop.category(), category_constants::LIFECYCLE);
    assert_eq!(
        HookEvent::SubagentStop.category(),
        category_constants::LIFECYCLE
    );
    assert_eq!(
        HookEvent::SessionStart.category(),
        category_constants::LIFECYCLE
    );
    assert_eq!(
        HookEvent::UserPromptSubmit.category(),
        category_constants::USER_INPUT
    );
    assert_eq!(
        HookEvent::PreCompact.category(),
        category_constants::MAINTENANCE
    );
}

#[test]
fn test_typed_validation_with_invalid_inputs() {
    // ❌ TYPE SAFETY: Test that typed validation catches missing required fields

    // Test PreToolUse missing tool_name
    let invalid_pre_tool = HookInput {
        session_id: "test_session".to_string(),
        transcript_path: PathBuf::from("/tmp/test.jsonl"),
        cwd: PathBuf::from("/tmp"),
        hook_event_name: HookEvent::PreToolUse.to_string(),
        tool_input: Some(json!({"test": "value"})),
        // tool_name is missing!
        ..Default::default()
    };

    assert!(invalid_pre_tool.validate().is_err());

    // Test Notification missing message
    let invalid_notification = HookInput {
        session_id: "test_session".to_string(),
        transcript_path: PathBuf::from("/tmp/test.jsonl"),
        cwd: PathBuf::from("/tmp"),
        hook_event_name: HookEvent::Notification.to_string(),
        // message is missing!
        ..Default::default()
    };

    assert!(invalid_notification.validate().is_err());
}

#[test]
fn test_unknown_hook_event_fallback() {
    // 🔄 TYPE SAFETY: Test that unknown events fall back to string validation

    let unknown_event_input = HookInput {
        session_id: "test_session".to_string(),
        transcript_path: PathBuf::from("/tmp/test.jsonl"),
        cwd: PathBuf::from("/tmp"),
        hook_event_name: "future_hook_event".to_string(), // Unknown event - intentionally hardcoded for fallback test
        ..Default::default()
    };

    // Should fail with unknown event error
    let result = unknown_event_input.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unknown hook event")
    );
}

#[test]
fn test_hook_event_serialization() {
    // 📤 TYPE SAFETY: Test serde serialization/deserialization

    let event = HookEvent::PreToolUse;

    // Test serialization
    let serialized = serde_json::to_string(&event).unwrap();
    assert_eq!(serialized, "\"PreToolUse\"");

    // Test deserialization
    let deserialized: HookEvent = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, event);
}

#[test]
fn test_hook_event_all_variants_present() {
    // 📋 TYPE SAFETY: Ensure all() method includes all enum variants

    let all_events = HookEvent::all();
    assert_eq!(
        all_events.len(),
        8,
        "HookEvent::all() should include all 8 variants"
    );

    // Check each variant is present
    assert!(all_events.contains(&HookEvent::PreToolUse));
    assert!(all_events.contains(&HookEvent::PostToolUse));
    assert!(all_events.contains(&HookEvent::Notification));
    assert!(all_events.contains(&HookEvent::Stop));
    assert!(all_events.contains(&HookEvent::SubagentStop));
    assert!(all_events.contains(&HookEvent::UserPromptSubmit));
    assert!(all_events.contains(&HookEvent::PreCompact));
    assert!(all_events.contains(&HookEvent::SessionStart));
}

#[test]
fn test_hook_event_hash_equality() {
    // #️⃣ TYPE SAFETY: Test Hash and PartialEq implementations
    use std::collections::HashSet;

    let mut event_set = HashSet::new();

    // Add all events
    for &event in HookEvent::all() {
        event_set.insert(event);
    }

    // Should have exactly 8 unique events
    assert_eq!(event_set.len(), 8);

    // Test equality
    assert_eq!(HookEvent::PreToolUse, HookEvent::PreToolUse);
    assert_ne!(HookEvent::PreToolUse, HookEvent::PostToolUse);
}

#[test]
fn test_typed_validation_coverage() {
    // 🎯 TYPE SAFETY: Ensure typed validation covers all enum variants
    // This test ensures no enum variant is forgotten in validate_typed_event

    for &event in HookEvent::all() {
        let hook_input = HookInput {
            session_id: "test_session".to_string(),
            transcript_path: PathBuf::from("/tmp/test.jsonl"),
            cwd: PathBuf::from("/tmp"),
            hook_event_name: event.as_str().to_string(),
            // Add some fields that might be required
            tool_name: Some("TestTool".to_string()),
            tool_input: Some(json!({"test": "value"})),
            tool_response: Some(json!({"result": "success"})),
            message: Some("Test message".to_string()),
            prompt: Some("Test prompt".to_string()),
            source: Some("startup".to_string()),
            ..Default::default()
        };

        // Should not panic - validates that all enum variants are handled
        let _result = hook_input.validate();
        // Note: We don't assert success because some events may still fail validation
        // for other reasons, but the important thing is no panic from unhandled enum
    }
}
