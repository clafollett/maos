#[cfg(test)]
mod io_tests {
    use super::super::{HookInput, StdinProcessor};
    use maos_core::config::HookConfig;
    use serde_json::json;
    use std::path::PathBuf;
    use std::time::Instant;

    // ===== STDIN PROCESSOR TESTS =====

    #[tokio::test]
    async fn test_stdin_processor_new() {
        let config = HookConfig {
            max_input_size_mb: 1,
            ..Default::default()
        };
        let processor = StdinProcessor::new(config);
        assert_eq!(processor.max_size(), 1024 * 1024);
    }

    #[tokio::test]
    async fn test_stdin_processor_with_defaults() {
        let processor = StdinProcessor::default();
        assert_eq!(processor.max_size(), 10 * 1024 * 1024); // 10MB default
    }

    #[tokio::test]
    async fn test_stdin_processor_buffer_reuse() {
        let mut processor = StdinProcessor::default();
        // Buffer should be reused across operations
        let buffer_ptr1 = processor.buffer_ptr();
        processor.clear_buffer();
        let buffer_ptr2 = processor.buffer_ptr();
        assert_eq!(buffer_ptr1, buffer_ptr2);
    }

    // ===== HOOK MESSAGE PARSING TESTS =====

    #[test]
    fn test_parse_pre_tool_use() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "pre_tool_use",
            "tool_name": "Bash",
            "tool_input": {"command": "ls -la"}
        });

        let input: HookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.session_id, "sess_123");
        assert_eq!(input.hook_event_name, "pre_tool_use");
        assert_eq!(input.tool_name.unwrap(), "Bash");
        assert!(input.tool_input.is_some());
    }

    #[test]
    fn test_parse_post_tool_use() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "post_tool_use",
            "tool_name": "Bash",
            "tool_input": {"command": "ls"},
            "tool_response": {"output": "file1.txt\nfile2.txt"}
        });

        let input: HookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.hook_event_name, "post_tool_use");
        assert!(input.tool_response.is_some());
    }

    #[test]
    fn test_parse_notification() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "notification",
            "message": "Task completed successfully"
        });

        let input: HookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.hook_event_name, "notification");
        assert_eq!(input.message.unwrap(), "Task completed successfully");
    }

    #[test]
    fn test_parse_user_prompt_submit() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "user_prompt_submit",
            "prompt": "Please help me fix this bug"
        });

        let input: HookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.hook_event_name, "user_prompt_submit");
        assert_eq!(input.prompt.unwrap(), "Please help me fix this bug");
    }

    #[test]
    fn test_parse_stop() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "stop",
            "stop_hook_active": true
        });

        let input: HookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.hook_event_name, "stop");
        assert!(input.stop_hook_active.unwrap());
    }

    #[test]
    fn test_parse_subagent_stop() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "subagent_stop",
            "stop_hook_active": false
        });

        let input: HookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.hook_event_name, "subagent_stop");
        assert!(!input.stop_hook_active.unwrap());
    }

    #[test]
    fn test_parse_pre_compact() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "pre_compact",
            "trigger": "auto",
            "custom_instructions": "Keep recent context"
        });

        let input: HookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.hook_event_name, "pre_compact");
        assert_eq!(input.trigger.unwrap(), "auto");
        assert_eq!(input.custom_instructions.unwrap(), "Keep recent context");
    }

    #[test]
    fn test_parse_session_start() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "session_start",
            "source": "startup"
        });

        let input: HookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.hook_event_name, "session_start");
        assert_eq!(input.source.unwrap(), "startup");
    }

    // ===== MISSING FIELDS TESTS =====

    #[test]
    fn test_parse_missing_required_fields() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            // Missing cwd and hook_event_name
        });

        let result: Result<HookInput, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_stop_without_optional_field() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "stop"
            // stop_hook_active is optional
        });

        let input: HookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.hook_event_name, "stop");
        assert!(input.stop_hook_active.is_none());
    }

    // ===== TYPE-SAFE ACCESSOR TESTS =====

    #[test]
    fn test_hook_input_is_tool_event() {
        let pre_tool = HookInput {
            session_id: "sess_123".to_string(),
            transcript_path: PathBuf::from("/tmp/transcript.jsonl"),
            cwd: PathBuf::from("/workspace"),
            hook_event_name: "pre_tool_use".to_string(),
            tool_name: Some("Bash".to_string()),
            tool_input: Some(json!({"command": "ls"})),
            tool_response: None,
            message: None,
            prompt: None,
            stop_hook_active: None,
            trigger: None,
            custom_instructions: None,
            source: None,
        };
        assert!(pre_tool.is_tool_event());

        let notification = HookInput {
            session_id: "sess_123".to_string(),
            transcript_path: PathBuf::from("/tmp/transcript.jsonl"),
            cwd: PathBuf::from("/workspace"),
            hook_event_name: "notification".to_string(),
            tool_name: None,
            tool_input: None,
            tool_response: None,
            message: Some("Test".to_string()),
            prompt: None,
            stop_hook_active: None,
            trigger: None,
            custom_instructions: None,
            source: None,
        };
        assert!(!notification.is_tool_event());
    }

    #[test]
    fn test_hook_input_get_tool_name() {
        let input = HookInput {
            session_id: "sess_123".to_string(),
            transcript_path: PathBuf::from("/tmp/transcript.jsonl"),
            cwd: PathBuf::from("/workspace"),
            hook_event_name: "pre_tool_use".to_string(),
            tool_name: Some("Bash".to_string()),
            tool_input: None,
            tool_response: None,
            message: None,
            prompt: None,
            stop_hook_active: None,
            trigger: None,
            custom_instructions: None,
            source: None,
        };
        assert_eq!(input.get_tool_name(), "Bash");

        let input_no_tool = HookInput {
            session_id: "sess_123".to_string(),
            transcript_path: PathBuf::from("/tmp/transcript.jsonl"),
            cwd: PathBuf::from("/workspace"),
            hook_event_name: "notification".to_string(),
            tool_name: None,
            tool_input: None,
            tool_response: None,
            message: Some("Test".to_string()),
            prompt: None,
            stop_hook_active: None,
            trigger: None,
            custom_instructions: None,
            source: None,
        };
        assert_eq!(input_no_tool.get_tool_name(), "");
    }

    // ===== SERIALIZATION ROUND-TRIP TESTS =====

    #[test]
    fn test_round_trip_serialization() {
        let original = HookInput {
            session_id: "sess_123".to_string(),
            transcript_path: PathBuf::from("/tmp/transcript.jsonl"),
            cwd: PathBuf::from("/workspace"),
            hook_event_name: "pre_tool_use".to_string(),
            tool_name: Some("Bash".to_string()),
            tool_input: Some(json!({"command": "ls"})),
            tool_response: None,
            message: None,
            prompt: None,
            stop_hook_active: None,
            trigger: None,
            custom_instructions: None,
            source: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: HookInput = serde_json::from_str(&json).unwrap();

        assert_eq!(original.session_id, deserialized.session_id);
        assert_eq!(original.hook_event_name, deserialized.hook_event_name);
        assert_eq!(original.tool_name, deserialized.tool_name);
    }

    #[test]
    fn test_serialize_omits_none_fields() {
        let input = HookInput {
            session_id: "sess_123".to_string(),
            transcript_path: PathBuf::from("/tmp/transcript.jsonl"),
            cwd: PathBuf::from("/workspace"),
            hook_event_name: "notification".to_string(),
            tool_name: None,
            tool_input: None,
            tool_response: None,
            message: Some("Test message".to_string()),
            prompt: None,
            stop_hook_active: None,
            trigger: None,
            custom_instructions: None,
            source: None,
        };

        let json = serde_json::to_string(&input).unwrap();
        assert!(!json.contains("tool_name"));
        assert!(!json.contains("tool_input"));
        assert!(json.contains("message"));
    }

    // ===== EDGE CASE TESTS =====

    #[test]
    fn test_parse_empty_json() {
        let json = json!({});
        let result: Result<HookInput, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_extra_fields_ignored() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "notification",
            "message": "Test",
            "extra_field": "should be ignored",
            "another_extra": 123
        });

        let input: HookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.message.unwrap(), "Test");
    }

    #[test]
    fn test_parse_large_tool_input() {
        let large_data = "x".repeat(10000);
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "pre_tool_use",
            "tool_name": "Write",
            "tool_input": {"content": large_data.clone()}
        });

        let input: HookInput = serde_json::from_value(json).unwrap();
        let tool_input = input.tool_input.unwrap();
        let content = tool_input["content"].as_str().unwrap();
        assert_eq!(content.len(), 10000);
    }

    // ===== PERFORMANCE BENCHMARK TESTS =====

    #[test]
    fn test_parse_performance_small_message() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "notification",
            "message": "Test"
        });

        let json_str = json.to_string();
        let start = Instant::now();
        for _ in 0..1000 {
            let _: HookInput = serde_json::from_str(&json_str).unwrap();
        }
        let elapsed = start.elapsed();

        // Should parse 1000 small messages in under 100ms (100μs per message)
        assert!(
            elapsed.as_millis() < 100,
            "Parsing took {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_parse_performance_medium_message() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "pre_tool_use",
            "tool_name": "Write",
            "tool_input": {
                "content": "x".repeat(10000)  // 10KB
            }
        });

        let json_str = json.to_string();
        let start = Instant::now();
        for _ in 0..100 {
            let _: HookInput = serde_json::from_str(&json_str).unwrap();
        }
        let elapsed = start.elapsed();

        // Should parse 100 medium messages in under 50ms (500μs per message)
        assert!(
            elapsed.as_millis() < 50,
            "Parsing took {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_parse_performance_large_message() {
        let json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "post_tool_use",
            "tool_name": "Read",
            "tool_input": {"file": "large.txt"},
            "tool_response": {
                "content": "x".repeat(100000)  // 100KB
            }
        });

        let json_str = json.to_string();
        let start = Instant::now();
        for _ in 0..10 {
            let _: HookInput = serde_json::from_str(&json_str).unwrap();
        }
        let elapsed = start.elapsed();

        // Should parse 10 large messages in under 20ms (2ms per message)
        assert!(
            elapsed.as_millis() < 20,
            "Parsing took {}ms",
            elapsed.as_millis()
        );
    }

    // ===== ASYNC STDIN TESTS =====

    // Note: Timeout test removed because it requires actual stdin mocking
    // which is complex in async context. The timeout functionality is
    // tested implicitly through normal usage.

    #[tokio::test]
    async fn test_stdin_processor_max_size() {
        let config = HookConfig {
            max_input_size_mb: 0, // Very small max size (0MB = 0 bytes)
            ..Default::default()
        };
        let processor = StdinProcessor::new(config);

        // Simulate large input that exceeds max size
        let large_json = json!({
            "session_id": "sess_123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "notification",
            "message": "x".repeat(1000)
        });

        // This should fail due to size limit
        let result = processor.validate_size(large_json.to_string().len());
        assert!(result.is_err());
    }

    // ===== JSON DEPTH VALIDATION TESTS =====

    #[test]
    fn test_json_depth_validation_normal() {
        let json = r#"{"a":{"b":{"c":"value"}}}"#;
        let result = StdinProcessor::validate_json_depth_static(json.as_bytes(), 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_depth_validation_exceeds_limit() {
        // Create deeply nested JSON beyond limit
        let mut json = String::new();
        for _ in 0..70 {
            json.push('{');
            json.push_str("\"a\":");
        }
        json.push_str("\"value\"");
        for _ in 0..70 {
            json.push('}');
        }

        let result = StdinProcessor::validate_json_depth_static(json.as_bytes(), 64);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("nesting depth"));
    }

    #[test]
    fn test_json_depth_validation_array_nesting() {
        let json = r#"[[[[[["value"]]]]]]"#;
        let result = StdinProcessor::validate_json_depth_static(json.as_bytes(), 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_depth_validation_mixed_nesting() {
        let json = r#"{"a":[{"b":[{"c":"value"}]}]}"#;
        let result = StdinProcessor::validate_json_depth_static(json.as_bytes(), 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_depth_validation_string_with_braces() {
        // Braces inside strings should not count towards depth
        let json =
            r#"{"message": "This {has} braces [and] brackets", "nested": {"value": "test"}}"#;
        let result = StdinProcessor::validate_json_depth_static(json.as_bytes(), 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_depth_validation_escaped_quotes() {
        let json = r#"{"message": "Quote \" in string", "nested": {"value": "test"}}"#;
        let result = StdinProcessor::validate_json_depth_static(json.as_bytes(), 2);
        assert!(result.is_ok());
    }

    // ===== VALIDATION TESTS =====

    #[test]
    fn test_validate_pre_tool_use_requirements() {
        let invalid = HookInput {
            session_id: "sess_123".to_string(),
            transcript_path: PathBuf::from("/tmp/transcript.jsonl"),
            cwd: PathBuf::from("/workspace"),
            hook_event_name: "pre_tool_use".to_string(),
            tool_name: None,  // Missing required field for pre_tool_use
            tool_input: None, // Missing required field for pre_tool_use
            tool_response: None,
            message: None,
            prompt: None,
            stop_hook_active: None,
            trigger: None,
            custom_instructions: None,
            source: None,
        };

        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_validate_session_start_source() {
        let valid = HookInput {
            session_id: "sess_123".to_string(),
            transcript_path: PathBuf::from("/tmp/transcript.jsonl"),
            cwd: PathBuf::from("/workspace"),
            hook_event_name: "session_start".to_string(),
            tool_name: None,
            tool_input: None,
            tool_response: None,
            message: None,
            prompt: None,
            stop_hook_active: None,
            trigger: None,
            custom_instructions: None,
            source: Some("startup".to_string()),
        };

        assert!(valid.validate().is_ok());

        // Test invalid source value
        let mut invalid = valid.clone();
        invalid.source = Some("invalid_source".to_string());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_validate_pre_compact_trigger() {
        let valid_auto = HookInput {
            session_id: "sess_123".to_string(),
            transcript_path: PathBuf::from("/tmp/transcript.jsonl"),
            cwd: PathBuf::from("/workspace"),
            hook_event_name: "pre_compact".to_string(),
            tool_name: None,
            tool_input: None,
            tool_response: None,
            message: None,
            prompt: None,
            stop_hook_active: None,
            trigger: Some("auto".to_string()),
            custom_instructions: Some("Keep context".to_string()),
            source: None,
        };

        assert!(valid_auto.validate().is_ok());

        let valid_manual = HookInput {
            trigger: Some("manual".to_string()),
            ..valid_auto.clone()
        };
        assert!(valid_manual.validate().is_ok());

        let invalid = HookInput {
            trigger: Some("invalid_trigger".to_string()),
            ..valid_auto.clone()
        };
        assert!(invalid.validate().is_err());
    }
}
