//! Configuration management for MAOS.
//!
//! This module provides a flexible configuration system supporting:
//! - Default values
//! - JSON file configuration
//! - Environment variable overrides
//! - Command-line argument overrides (future)
//!
//! # Example
//! ```
//! use maos_core::config::{MaosConfig, LogLevel};
//!
//! let cfg = MaosConfig::default();
//! assert_eq!(cfg.logging.level, LogLevel::Info);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{ConfigError, Result};

/// System-wide configuration settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemConfig {
    /// Maximum execution time for any operation (ms)
    #[serde(default = "default_max_execution_time")]
    pub max_execution_time_ms: u64,

    /// Default workspace root directory
    #[serde(default = "default_workspace_root")]
    pub workspace_root: PathBuf,

    /// Enable performance metrics collection
    #[serde(default = "default_true")]
    pub enable_metrics: bool,
}

/// Security validation configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityConfig {
    /// Enable security validation checks
    #[serde(default = "default_true")]
    pub enable_validation: bool,

    /// List of allowed tools ("*" for all)
    #[serde(default = "default_allowed_tools")]
    pub allowed_tools: Vec<String>,

    /// Paths that should be blocked
    #[serde(default)]
    pub blocked_paths: Vec<String>,
}

/// TTS (Text-to-Speech) configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TtsConfig {
    /// TTS provider ("none", "say", "espeak", etc.)
    #[serde(default = "default_tts_provider")]
    pub provider: String,

    /// Voice name
    #[serde(default = "default_voice")]
    pub voice: String,

    /// Speech rate (words per minute)
    #[serde(default = "default_tts_rate")]
    pub rate: u32,
}

/// Session management configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionConfig {
    /// Maximum number of concurrent agents
    #[serde(default = "default_max_agents")]
    pub max_agents: u32,

    /// Session timeout in minutes
    #[serde(default = "default_timeout_minutes")]
    pub timeout_minutes: u32,

    /// Automatically cleanup sessions on completion
    #[serde(default = "default_true")]
    pub auto_cleanup: bool,
}

/// Git worktree configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorktreeConfig {
    /// Prefix for worktree names
    #[serde(default = "default_worktree_prefix")]
    pub prefix: String,

    /// Automatically cleanup worktrees
    #[serde(default = "default_true")]
    pub auto_cleanup: bool,

    /// Maximum number of worktrees
    #[serde(default = "default_max_worktrees")]
    pub max_worktrees: u32,
}

/// Logging level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for LogLevel {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(()),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: LogLevel,

    /// Log format ("json" or "text")
    #[serde(default = "default_log_format")]
    pub format: String,

    /// Log output ("stdout", "stderr", "session_file")
    #[serde(default = "default_log_output")]
    pub output: String,
}

/// Root MAOS configuration structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct MaosConfig {
    /// System-wide settings
    pub system: SystemConfig,

    /// Security validation settings
    pub security: SecurityConfig,

    /// TTS provider settings
    pub tts: TtsConfig,

    /// Session management settings
    pub session: SessionConfig,

    /// Git worktree settings
    pub worktree: WorktreeConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

// Keep backward compatibility alias
pub type Config = MaosConfig;

impl Default for MaosConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                max_execution_time_ms: default_max_execution_time(),
                workspace_root: default_workspace_root(),
                enable_metrics: default_true(),
            },
            security: SecurityConfig {
                enable_validation: default_true(),
                allowed_tools: default_allowed_tools(),
                blocked_paths: Vec::new(),
            },
            tts: TtsConfig {
                provider: default_tts_provider(),
                voice: default_voice(),
                rate: default_tts_rate(),
            },
            session: SessionConfig {
                max_agents: default_max_agents(),
                timeout_minutes: default_timeout_minutes(),
                auto_cleanup: default_true(),
            },
            worktree: WorktreeConfig {
                prefix: default_worktree_prefix(),
                auto_cleanup: default_true(),
                max_worktrees: default_max_worktrees(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
                format: default_log_format(),
                output: default_log_output(),
            },
        }
    }
}

impl MaosConfig {
    /// Load configuration (currently just returns defaults)
    pub fn load() -> Result<Self> {
        Ok(Self::default())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate execution time
        if self.system.max_execution_time_ms == 0 {
            return Err(ConfigError::InvalidValue {
                field: "max_execution_time_ms".into(),
                value: "0".into(),
                reason: "must be greater than 0".into(),
            }
            .into());
        }

        Ok(())
    }
}

/// Configuration loader with support for multiple sources
#[derive(Default)]
pub struct ConfigLoader {}

impl ConfigLoader {
    /// Create a new config loader
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a JSON string
    pub fn load_from_str(&self, json: &str) -> Result<MaosConfig> {
        // Start with defaults
        let mut config = MaosConfig::default();

        // Parse and merge the JSON
        let partial: serde_json::Value = serde_json::from_str(json)?;
        self.merge_json(&mut config, partial)?;

        // Validate
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from any reader providing JSON bytes
    pub fn load_from_reader<R: Read>(&self, mut reader: R) -> Result<MaosConfig> {
        // Start with defaults
        let mut config = MaosConfig::default();

        // Read and parse JSON
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        let partial: serde_json::Value = serde_json::from_str(&buf)?;
        self.merge_json(&mut config, partial)?;

        // Validate
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from a file path containing JSON
    pub fn load_from_path(&self, path: &Path) -> Result<MaosConfig> {
        let file = File::open(path)?;
        self.load_from_reader(file)
    }

    /// Load configuration with environment variable overrides
    pub fn load_with_env(&self, env_vars: HashMap<String, String>) -> Result<MaosConfig> {
        // Start with defaults
        let mut config = MaosConfig::default();

        // Apply environment overrides
        self.apply_env_overrides(&mut config, env_vars)?;

        // Validate
        config.validate()?;

        Ok(config)
    }

    /// Merge JSON values into config
    fn merge_json(&self, config: &mut MaosConfig, value: serde_json::Value) -> Result<()> {
        // This is a simple implementation - could be more sophisticated
        if let serde_json::Value::Object(map) = value {
            // System config
            if let Some(system) = map.get("system") {
                if let Some(val) = system.get("max_execution_time_ms")
                    && let Some(ms) = val.as_u64()
                {
                    config.system.max_execution_time_ms = ms;
                }
                if let Some(val) = system.get("workspace_root")
                    && let Some(path) = val.as_str()
                {
                    config.system.workspace_root = PathBuf::from(path);
                }
                if let Some(val) = system.get("enable_metrics")
                    && let Some(enabled) = val.as_bool()
                {
                    config.system.enable_metrics = enabled;
                }
            }

            // Security config
            if let Some(security) = map.get("security") {
                if let Some(val) = security.get("enable_validation")
                    && let Some(enabled) = val.as_bool()
                {
                    config.security.enable_validation = enabled;
                }
                if let Some(val) = security.get("allowed_tools")
                    && let Some(arr) = val.as_array()
                {
                    config.security.allowed_tools = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
                if let Some(val) = security.get("blocked_paths")
                    && let Some(arr) = val.as_array()
                {
                    config.security.blocked_paths = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
            }

            // TTS config
            if let Some(tts) = map.get("tts") {
                if let Some(val) = tts.get("provider")
                    && let Some(provider) = val.as_str()
                {
                    config.tts.provider = provider.to_string();
                }
                if let Some(val) = tts.get("voice")
                    && let Some(voice) = val.as_str()
                {
                    config.tts.voice = voice.to_string();
                }
                if let Some(val) = tts.get("rate")
                    && let Some(rate) = val.as_u64()
                {
                    config.tts.rate = rate as u32;
                }
            }

            // Session config
            if let Some(session) = map.get("session") {
                if let Some(val) = session.get("max_agents")
                    && let Some(max) = val.as_u64()
                {
                    config.session.max_agents = max as u32;
                }
                if let Some(val) = session.get("timeout_minutes")
                    && let Some(timeout) = val.as_u64()
                {
                    config.session.timeout_minutes = timeout as u32;
                }
                if let Some(val) = session.get("auto_cleanup")
                    && let Some(cleanup) = val.as_bool()
                {
                    config.session.auto_cleanup = cleanup;
                }
            }

            // Worktree config
            if let Some(worktree) = map.get("worktree") {
                if let Some(val) = worktree.get("prefix")
                    && let Some(prefix) = val.as_str()
                {
                    config.worktree.prefix = prefix.to_string();
                }
                if let Some(val) = worktree.get("auto_cleanup")
                    && let Some(cleanup) = val.as_bool()
                {
                    config.worktree.auto_cleanup = cleanup;
                }
                if let Some(val) = worktree.get("max_worktrees")
                    && let Some(max) = val.as_u64()
                {
                    config.worktree.max_worktrees = max as u32;
                }
            }

            // Logging config
            if let Some(logging) = map.get("logging") {
                if let Some(val) = logging.get("level")
                    && let Some(level_str) = val.as_str()
                {
                    match level_str.parse::<LogLevel>() {
                        Ok(level) => config.logging.level = level,
                        Err(_) => {
                            return Err(ConfigError::InvalidValue {
                                field: "logging.level".into(),
                                value: level_str.to_string(),
                                reason: "must be one of: trace, debug, info, warn, error"
                                    .to_string(),
                            }
                            .into());
                        }
                    }
                }
                if let Some(val) = logging.get("format")
                    && let Some(format) = val.as_str()
                {
                    config.logging.format = format.to_string();
                }
                if let Some(val) = logging.get("output")
                    && let Some(output) = val.as_str()
                {
                    config.logging.output = output.to_string();
                }
            }
        }

        Ok(())
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(
        &self,
        config: &mut MaosConfig,
        env_vars: HashMap<String, String>,
    ) -> Result<()> {
        // System overrides
        if let Some(val) = env_vars.get("MAOS_SYSTEM_MAX_EXECUTION_TIME_MS") {
            config.system.max_execution_time_ms =
                val.parse().map_err(|_| ConfigError::InvalidValue {
                    field: "MAOS_SYSTEM_MAX_EXECUTION_TIME_MS".into(),
                    value: val.clone(),
                    reason: "must be a valid number".into(),
                })?;
        }

        if let Some(val) = env_vars.get("MAOS_SYSTEM_WORKSPACE_ROOT") {
            config.system.workspace_root = PathBuf::from(val);
        }

        // Security overrides
        if let Some(val) = env_vars.get("MAOS_SECURITY_ENABLE_VALIDATION") {
            config.security.enable_validation =
                val.parse().map_err(|_| ConfigError::InvalidValue {
                    field: "MAOS_SECURITY_ENABLE_VALIDATION".into(),
                    value: val.clone(),
                    reason: "must be true or false".into(),
                })?;
        }

        // TTS overrides
        if let Some(val) = env_vars.get("MAOS_TTS_PROVIDER") {
            config.tts.provider = val.clone();
        }
        if let Some(val) = env_vars.get("MAOS_TTS_VOICE") {
            config.tts.voice = val.clone();
        }
        if let Some(val) = env_vars.get("MAOS_TTS_RATE") {
            config.tts.rate = val.parse().map_err(|_| ConfigError::InvalidValue {
                field: "MAOS_TTS_RATE".into(),
                value: val.clone(),
                reason: "must be a valid number".into(),
            })?;
        }

        // Logging overrides
        if let Some(val) = env_vars.get("MAOS_LOGGING_LEVEL") {
            match val.parse::<LogLevel>() {
                Ok(level) => config.logging.level = level,
                Err(_) => {
                    return Err(ConfigError::InvalidValue {
                        field: "MAOS_LOGGING_LEVEL".into(),
                        value: val.clone(),
                        reason: "must be one of: trace, debug, info, warn, error".to_string(),
                    }
                    .into());
                }
            }
        }
        if let Some(val) = env_vars.get("MAOS_LOGGING_FORMAT") {
            config.logging.format = val.clone();
        }
        if let Some(val) = env_vars.get("MAOS_LOGGING_OUTPUT") {
            config.logging.output = val.clone();
        }

        Ok(())
    }
}

// Default value functions
fn default_max_execution_time() -> u64 {
    60_000
}
fn default_workspace_root() -> PathBuf {
    PathBuf::from("/tmp/maos")
}
fn default_true() -> bool {
    true
}
fn default_allowed_tools() -> Vec<String> {
    vec!["*".to_string()]
}
fn default_tts_provider() -> String {
    "none".to_string()
}
fn default_voice() -> String {
    "default".to_string()
}
fn default_tts_rate() -> u32 {
    200
}
fn default_max_agents() -> u32 {
    20
}
fn default_timeout_minutes() -> u32 {
    60
}
fn default_worktree_prefix() -> String {
    "maos-agent".to_string()
}
fn default_max_worktrees() -> u32 {
    50
}
fn default_log_level() -> LogLevel {
    LogLevel::Info
}
fn default_log_format() -> String {
    "json".to_string()
}
fn default_log_output() -> String {
    "session_file".to_string()
}
