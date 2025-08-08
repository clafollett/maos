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
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(format!(
                "invalid log level '{s}', expected one of: trace, debug, info, warn, error"
            )),
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

    /// Helper function for parsing environment variables with error mapping
    fn parse_env_var<T: std::str::FromStr>(val: &str, field: &str, reason: &str) -> Result<T> {
        val.parse().map_err(|_| {
            ConfigError::InvalidValue {
                field: field.into(),
                value: val.to_string(),
                reason: reason.into(),
            }
            .into()
        })
    }

    /// Load configuration from a JSON string
    pub fn load_from_str(&self, json: &str) -> Result<MaosConfig> {
        // Parse JSON into a partial config
        let partial: MaosConfig = serde_json::from_str(json)?;

        // Merge with defaults (simple approach - could be more sophisticated)
        let mut config = MaosConfig::default();
        self.merge_configs(&mut config, partial);

        // Validate
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from any reader providing JSON bytes
    pub fn load_from_reader<R: Read>(&self, mut reader: R) -> Result<MaosConfig> {
        // Read and parse JSON
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;

        // Use the string loader
        self.load_from_str(&buf)
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

    /// Merge a partial config into a base config
    /// This replaces only the fields that are present in the partial config
    fn merge_configs(&self, base: &mut MaosConfig, partial: MaosConfig) {
        // For now, this is a simple field-by-field replacement
        // In a more sophisticated implementation, we could check if fields
        // are "default" and only override non-default values

        // System config - check if values differ from defaults to avoid overwriting
        if partial.system.max_execution_time_ms != default_max_execution_time() {
            base.system.max_execution_time_ms = partial.system.max_execution_time_ms;
        }
        if partial.system.workspace_root != default_workspace_root() {
            base.system.workspace_root = partial.system.workspace_root;
        }
        if partial.system.enable_metrics != default_true() {
            base.system.enable_metrics = partial.system.enable_metrics;
        }

        // Security config
        if partial.security.enable_validation != default_true() {
            base.security.enable_validation = partial.security.enable_validation;
        }
        if partial.security.allowed_tools != default_allowed_tools() {
            base.security.allowed_tools = partial.security.allowed_tools;
        }
        if !partial.security.blocked_paths.is_empty() {
            base.security.blocked_paths = partial.security.blocked_paths;
        }

        // TTS config
        if partial.tts.provider != default_tts_provider() {
            base.tts.provider = partial.tts.provider;
        }
        if partial.tts.voice != default_voice() {
            base.tts.voice = partial.tts.voice;
        }
        if partial.tts.rate != default_tts_rate() {
            base.tts.rate = partial.tts.rate;
        }

        // Session config
        if partial.session.max_agents != default_max_agents() {
            base.session.max_agents = partial.session.max_agents;
        }
        if partial.session.timeout_minutes != default_timeout_minutes() {
            base.session.timeout_minutes = partial.session.timeout_minutes;
        }
        if partial.session.auto_cleanup != default_true() {
            base.session.auto_cleanup = partial.session.auto_cleanup;
        }

        // Worktree config
        if partial.worktree.prefix != default_worktree_prefix() {
            base.worktree.prefix = partial.worktree.prefix;
        }
        if partial.worktree.auto_cleanup != default_true() {
            base.worktree.auto_cleanup = partial.worktree.auto_cleanup;
        }
        if partial.worktree.max_worktrees != default_max_worktrees() {
            base.worktree.max_worktrees = partial.worktree.max_worktrees;
        }

        // Logging config
        if partial.logging.level != default_log_level() {
            base.logging.level = partial.logging.level;
        }
        if partial.logging.format != default_log_format() {
            base.logging.format = partial.logging.format;
        }
        if partial.logging.output != default_log_output() {
            base.logging.output = partial.logging.output;
        }
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(
        &self,
        config: &mut MaosConfig,
        env_vars: HashMap<String, String>,
    ) -> Result<()> {
        // System overrides
        if let Some(val) = env_vars.get("MAOS_SYSTEM_MAX_EXECUTION_TIME_MS") {
            config.system.max_execution_time_ms = Self::parse_env_var(
                val,
                "MAOS_SYSTEM_MAX_EXECUTION_TIME_MS",
                "must be a valid number",
            )?;
        }

        if let Some(val) = env_vars.get("MAOS_SYSTEM_WORKSPACE_ROOT") {
            config.system.workspace_root = PathBuf::from(val);
        }

        // Security overrides
        if let Some(val) = env_vars.get("MAOS_SECURITY_ENABLE_VALIDATION") {
            config.security.enable_validation = Self::parse_env_var(
                val,
                "MAOS_SECURITY_ENABLE_VALIDATION",
                "must be true or false",
            )?;
        }

        // TTS overrides
        if let Some(val) = env_vars.get("MAOS_TTS_PROVIDER") {
            config.tts.provider = val.clone();
        }
        if let Some(val) = env_vars.get("MAOS_TTS_VOICE") {
            config.tts.voice = val.clone();
        }
        if let Some(val) = env_vars.get("MAOS_TTS_RATE") {
            config.tts.rate = Self::parse_env_var(val, "MAOS_TTS_RATE", "must be a valid number")?;
        }

        // Logging overrides
        if let Some(val) = env_vars.get("MAOS_LOGGING_LEVEL") {
            config.logging.level =
                val.parse()
                    .map_err(|err: String| ConfigError::InvalidValue {
                        field: "MAOS_LOGGING_LEVEL".into(),
                        value: val.clone(),
                        reason: err,
                    })?;
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
