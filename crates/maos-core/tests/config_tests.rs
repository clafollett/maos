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
    assert_eq!(cfg.tts.provider, "none");
    assert_eq!(cfg.tts.voice, "default");
    assert_eq!(cfg.tts.rate, 200);

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
    env_vars.insert("MAOS_TTS_PROVIDER".to_string(), "say".to_string());
    env_vars.insert("MAOS_LOGGING_LEVEL".to_string(), "debug".to_string());

    // Use a test-specific loader that accepts env vars
    let loader = ConfigLoader::new();
    let cfg = loader.load_with_env(env_vars).unwrap();

    assert_eq!(cfg.system.max_execution_time_ms, 5000);
    assert_eq!(cfg.system.workspace_root, PathBuf::from("/custom/path"));
    assert!(!cfg.security.enable_validation);
    assert_eq!(cfg.tts.provider, "say");
    assert_eq!(cfg.logging.level, LogLevel::Debug);
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
fn test_invalid_tts_rate_from_env_fails() {
    let loader = ConfigLoader::new();
    let mut env_vars = HashMap::new();
    env_vars.insert("MAOS_TTS_RATE".to_string(), "fast".to_string());
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
            "provider": "espeak",
            "voice": "en-us",
            "rate": 150
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
    assert_eq!(cfg.tts.voice, "en-us");
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
