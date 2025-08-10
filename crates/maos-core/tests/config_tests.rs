use maos_core::config::{ConfigLoader, LogLevel, MaosConfig};
use maos_core::error::Result;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_config_defaults_load() -> Result<()> {
    // Load defaults when no files/env present
    let cfg = MaosConfig::default();

    // System defaults
    assert_eq!(cfg.system.max_execution_time_ms, 60_000);
    assert_eq!(cfg.system.workspace_root, PathBuf::from("/tmp/maos"));
    assert!(cfg.system.enable_metrics);

    // Security defaults
    assert!(cfg.security.enable_validation);
    assert_eq!(cfg.security.allowed_tools, vec!["*"]);
    assert!(cfg.security.blocked_paths.is_empty());

    // TTS defaults
    assert!(cfg.tts.enabled);
    assert_eq!(cfg.tts.provider, "pyttsx3");
    assert_eq!(cfg.tts.text_length_limit, 2000);
    assert_eq!(cfg.tts.timeout, 120);
    assert_eq!(cfg.tts.voices.macos.voice, "Alex");
    assert_eq!(cfg.tts.voices.macos.rate, 190);
    assert_eq!(cfg.tts.voices.macos.quality, 127);
    assert_eq!(cfg.tts.voices.pyttsx3.rate, 190);
    assert_eq!(cfg.tts.voices.pyttsx3.volume, 0.9);
    assert!(!cfg.tts.responses.enabled);
    assert!(cfg.tts.completion.enabled);
    assert!(cfg.tts.notifications.enabled);

    // Session defaults
    assert_eq!(cfg.session.max_agents, 20);
    assert_eq!(cfg.session.timeout_minutes, 60);
    assert!(cfg.session.auto_cleanup);

    // Worktree defaults
    assert_eq!(cfg.worktree.prefix, "maos-agent");
    assert!(cfg.worktree.auto_cleanup);
    assert_eq!(cfg.worktree.max_worktrees, 50);

    // Logging defaults
    assert_eq!(cfg.logging.level, LogLevel::Info);
    assert_eq!(cfg.logging.format, "json");
    assert_eq!(cfg.logging.output, "session_file");

    Ok(())
}

#[test]
fn test_config_with_env_overrides() {
    // Test that we can apply environment overrides using a test-friendly method
    let mut env_vars = HashMap::new();
    env_vars.insert(
        "MAOS_SYSTEM_MAX_EXECUTION_TIME_MS".to_string(),
        "5000".to_string(),
    );
    env_vars.insert(
        "MAOS_SYSTEM_WORKSPACE_ROOT".to_string(),
        "/custom/path".to_string(),
    );
    env_vars.insert(
        "MAOS_SECURITY_ENABLE_VALIDATION".to_string(),
        "false".to_string(),
    );
    // Note: TTS provider is now config-only, no env var overrides
    env_vars.insert("MAOS_LOGGING_LEVEL".to_string(), "debug".to_string());
    env_vars.insert("ELEVENLABS_API_KEY".to_string(), "test-key-123".to_string());
    env_vars.insert("OPENAI_API_KEY".to_string(), "test-openai-456".to_string());

    // Use a test-specific loader that accepts env vars
    let loader = ConfigLoader::new();
    let cfg = loader.load_with_env(env_vars).unwrap();

    assert_eq!(cfg.system.max_execution_time_ms, 5000);
    assert_eq!(cfg.system.workspace_root, PathBuf::from("/custom/path"));
    assert!(!cfg.security.enable_validation);
    // TTS provider should remain default (no env override)
    assert_eq!(cfg.tts.provider, "pyttsx3");
    assert_eq!(cfg.logging.level, LogLevel::Debug);

    // API keys should be loaded from environment
    assert_eq!(
        cfg.tts.voices.elevenlabs.api_key,
        Some("test-key-123".to_string())
    );
    assert_eq!(
        cfg.tts.voices.openai.api_key,
        Some("test-openai-456".to_string())
    );
}

#[test]
fn test_config_validation() {
    let mut cfg = MaosConfig::default();

    // Valid config should pass
    assert!(cfg.validate().is_ok());

    // Invalid execution time should fail
    cfg.system.max_execution_time_ms = 0;
    assert!(cfg.validate().is_err());
    cfg.system.max_execution_time_ms = 60_000;

    // Valid again
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_tts_api_key_fallback_logic() {
    let mut cfg = MaosConfig::default();
    cfg.tts.enabled = true;

    // Test 1: pyttsx3 provider (no API key needed)
    cfg.tts.provider = "pyttsx3".to_string();
    assert_eq!(cfg.get_active_tts_provider(), "pyttsx3");

    // Test 2: macos provider (no API key needed)
    cfg.tts.provider = "macos".to_string();
    assert_eq!(cfg.get_active_tts_provider(), "macos");

    // Test 3: API provider behavior depends on actual environment state
    cfg.tts.provider = "elevenlabs".to_string();
    cfg.tts.voices.elevenlabs.api_key = None; // Force config-only check

    // Check what environment provides for elevenlabs
    let has_elevenlabs_key = std::env::var("ELEVENLABS_API_KEY")
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false);
    let expected_provider = if has_elevenlabs_key {
        "elevenlabs"
    } else {
        "pyttsx3"
    };
    assert_eq!(cfg.get_active_tts_provider(), expected_provider);

    // Test 4: elevenlabs with empty API key in config (env still takes precedence)
    cfg.tts.voices.elevenlabs.api_key = Some("".to_string());
    assert_eq!(cfg.get_active_tts_provider(), expected_provider);

    // Test 5: elevenlabs with whitespace-only API key in config (env still takes precedence)
    cfg.tts.voices.elevenlabs.api_key = Some("   ".to_string());
    assert_eq!(cfg.get_active_tts_provider(), expected_provider);

    // Test 6: elevenlabs with valid API key in config (should use elevenlabs regardless of env)
    cfg.tts.voices.elevenlabs.api_key = Some("sk-test-key".to_string());
    assert_eq!(cfg.get_active_tts_provider(), "elevenlabs");

    // Test 7: openai behavior depends on actual environment state
    cfg.tts.provider = "openai".to_string();
    cfg.tts.voices.openai.api_key = None; // Force config-only check

    // Check what environment provides for openai
    let has_openai_key = std::env::var("OPENAI_API_KEY")
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false);
    let expected_openai_provider = if has_openai_key { "openai" } else { "pyttsx3" };
    assert_eq!(cfg.get_active_tts_provider(), expected_openai_provider);

    // Test 8: openai with valid API key in config (should use openai regardless of env)
    cfg.tts.voices.openai.api_key = Some("sk-openai-test".to_string());
    assert_eq!(cfg.get_active_tts_provider(), "openai");
}

#[test]
fn test_tts_feature_toggles() {
    let mut cfg = MaosConfig::default();

    // Master switch disabled - all features should be false
    cfg.tts.enabled = false;
    cfg.tts.responses.enabled = true;
    cfg.tts.completion.enabled = true;
    cfg.tts.notifications.enabled = true;

    assert!(!cfg.is_tts_enabled());
    assert!(!cfg.is_response_tts_enabled());
    assert!(!cfg.is_completion_tts_enabled());
    assert!(!cfg.is_notification_tts_enabled());

    // Master switch enabled - individual features control behavior
    cfg.tts.enabled = true;
    cfg.tts.responses.enabled = false;
    cfg.tts.completion.enabled = true;
    cfg.tts.notifications.enabled = false;

    assert!(cfg.is_tts_enabled());
    assert!(!cfg.is_response_tts_enabled()); // Individual switch off
    assert!(cfg.is_completion_tts_enabled()); // Individual switch on
    assert!(!cfg.is_notification_tts_enabled()); // Individual switch off
}

#[test]
fn test_tts_config_getters() {
    let cfg = MaosConfig::default();

    // Test defaults
    assert_eq!(cfg.get_text_length_limit(), 2000);
    assert_eq!(cfg.get_tts_timeout(), 120);

    // Test with custom values
    let mut custom_cfg = MaosConfig::default();
    custom_cfg.tts.text_length_limit = 5000;
    custom_cfg.tts.timeout = 300;

    assert_eq!(custom_cfg.get_text_length_limit(), 5000);
    assert_eq!(custom_cfg.get_tts_timeout(), 300);
}

#[test]
fn test_api_key_cascading_resolution() {
    // This test verifies cascading resolution: env vars → config
    let mut cfg = MaosConfig::default();

    // Helper to get current env state
    let get_env_key = |var: &str| -> Option<String> {
        std::env::var(var).ok().and_then(|k| {
            let trimmed = k.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    };

    // Test 1: Check current environment state (no config keys set)
    let env_elevenlabs = get_env_key("ELEVENLABS_API_KEY");
    let env_openai = get_env_key("OPENAI_API_KEY");

    // Should return env vars if present, None otherwise
    assert_eq!(cfg.get_api_key("elevenlabs"), env_elevenlabs);
    assert_eq!(cfg.get_api_key("openai"), env_openai);
    assert_eq!(cfg.get_api_key("unknown"), None);

    // Test 2: Config keys set - env still takes precedence
    cfg.tts.voices.elevenlabs.api_key = Some("sk-config-key".to_string());
    cfg.tts.voices.openai.api_key = Some("sk-openai-config".to_string());

    let expected_elevenlabs = env_elevenlabs.unwrap_or("sk-config-key".to_string());
    let expected_openai = env_openai.unwrap_or("sk-openai-config".to_string());

    assert_eq!(cfg.get_api_key("elevenlabs"), Some(expected_elevenlabs));
    assert_eq!(cfg.get_api_key("openai"), Some(expected_openai));

    // Test 3: Empty/whitespace config keys - should fall back to env (or None)
    cfg.tts.voices.elevenlabs.api_key = Some("".to_string());
    cfg.tts.voices.openai.api_key = Some("   ".to_string());

    // Should still return env vars if present, None otherwise
    assert_eq!(
        cfg.get_api_key("elevenlabs"),
        get_env_key("ELEVENLABS_API_KEY")
    );
    assert_eq!(cfg.get_api_key("openai"), get_env_key("OPENAI_API_KEY"));
}

#[test]
fn test_invalid_logging_level_from_json_fails() {
    let json = r#"{
        "logging": { "level": "invalid" }
    }"#;
    let loader = ConfigLoader::new();
    let res = loader.load_from_str(json);
    assert!(res.is_err());
}

#[test]
fn test_invalid_logging_level_from_env_fails() {
    let loader = ConfigLoader::new();
    let mut env_vars = HashMap::new();
    env_vars.insert("MAOS_LOGGING_LEVEL".to_string(), "notalevel".to_string());
    let res = loader.load_with_env(env_vars);
    assert!(res.is_err());
}

#[test]
fn test_invalid_max_execution_time_from_env_fails() {
    let loader = ConfigLoader::new();
    let mut env_vars = HashMap::new();
    env_vars.insert(
        "MAOS_SYSTEM_MAX_EXECUTION_TIME_MS".to_string(),
        "not_a_number".to_string(),
    );
    let res = loader.load_with_env(env_vars);
    assert!(res.is_err());
}

#[test]
fn test_config_serialization() {
    let cfg = MaosConfig::default();

    // Should serialize to JSON
    let json = serde_json::to_string_pretty(&cfg).unwrap();
    assert!(json.contains("\"system\""));
    assert!(json.contains("\"security\""));
    assert!(json.contains("\"tts\""));

    // Should deserialize back
    let cfg2: MaosConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(
        cfg.system.max_execution_time_ms,
        cfg2.system.max_execution_time_ms
    );
}

#[test]
fn test_config_from_json_string() {
    let json = r#"{
        "system": {
            "max_execution_time_ms": 30000,
            "workspace_root": "/test/path",
            "enable_metrics": false
        },
        "security": {
            "enable_validation": true,
            "allowed_tools": ["bash", "python"],
            "blocked_paths": [".git", "secrets"]
        },
        "tts": {
            "enabled": true,
            "provider": "pyttsx3",
            "text_length_limit": 1000,
            "timeout": 30,
            "voices": {
                "macos": {
                    "voice": "Daniel",
                    "rate": 180,
                    "quality": 100
                },
                "pyttsx3": {
                    "voice": "custom",
                    "rate": 150,
                    "volume": 0.8
                }
            },
            "responses": {
                "enabled": true
            },
            "completion": {
                "enabled": false
            },
            "notifications": {
                "enabled": true
            }
        },
        "session": {
            "max_agents": 10,
            "timeout_minutes": 30,
            "auto_cleanup": false
        },
        "worktree": {
            "prefix": "test",
            "auto_cleanup": false,
            "max_worktrees": 25
        },
        "logging": {
            "level": "debug",
            "format": "text",
            "output": "stdout"
        }
    }"#;

    let loader = ConfigLoader::new();
    let cfg = loader.load_from_str(json).unwrap();

    assert_eq!(cfg.system.max_execution_time_ms, 30000);
    assert_eq!(cfg.security.allowed_tools, vec!["bash", "python"]);
    assert!(cfg.tts.enabled);
    assert_eq!(cfg.tts.provider, "pyttsx3");
    assert_eq!(cfg.tts.text_length_limit, 1000);
    assert_eq!(cfg.tts.timeout, 30);
    assert_eq!(cfg.tts.voices.macos.voice, "Daniel");
    assert_eq!(cfg.tts.voices.macos.rate, 180);
    assert_eq!(cfg.tts.voices.macos.quality, 100);
    assert_eq!(cfg.tts.voices.pyttsx3.voice, "custom");
    assert_eq!(cfg.tts.voices.pyttsx3.rate, 150);
    assert_eq!(cfg.tts.voices.pyttsx3.volume, 0.8);
    assert!(cfg.tts.responses.enabled);
    assert!(!cfg.tts.completion.enabled);
    assert!(cfg.tts.notifications.enabled);
}

#[test]
fn test_partial_config_merge() {
    // Test that partial configs merge correctly with defaults
    let partial_json = r#"{
        "system": {
            "max_execution_time_ms": 15000
        },
        "logging": {
            "level": "debug"
        }
    }"#;

    let loader = ConfigLoader::new();
    let cfg = loader.load_from_str(partial_json).unwrap();

    // Overridden values
    assert_eq!(cfg.system.max_execution_time_ms, 15000);
    assert_eq!(cfg.logging.level, LogLevel::Debug);

    // Default values should still be present
    assert_eq!(cfg.system.workspace_root, PathBuf::from("/tmp/maos"));
    assert_eq!(cfg.logging.format, "json");
    assert!(cfg.security.enable_validation);
}
