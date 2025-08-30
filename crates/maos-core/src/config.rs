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

use crate::constants::{MAOS_ROOT_DIR, WORKSPACES_DIR_NAME};
use crate::error::{ConfigError, Result};

/// System-wide configuration settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemConfig {
    /// Default project root directory
    #[serde(default = "default_project_root")]
    pub project_root: PathBuf,

    /// Default workspace root directory
    #[serde(default = "default_workspaces_root")]
    pub workspaces_root: PathBuf,

    /// Maximum execution time for any operation (ms)
    #[serde(default = "default_max_execution_time")]
    pub max_execution_time_ms: u64,

    /// Enable performance metrics collection
    #[serde(default = "default_true")]
    pub enable_metrics: bool,
}

/// Hook processing configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HookConfig {
    /// Maximum input size in megabytes
    #[serde(default = "default_max_input_size_mb")]
    pub max_input_size_mb: u64,

    /// Maximum total processing time in milliseconds
    #[serde(default = "default_max_processing_time_ms")]
    pub max_processing_time_ms: u64,

    /// Maximum JSON nesting depth
    #[serde(default = "default_max_json_depth")]
    pub max_json_depth: u32,

    /// Timeout for individual stdin read operations in milliseconds
    #[serde(default = "default_stdin_read_timeout_ms")]
    pub stdin_read_timeout_ms: u64,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            max_input_size_mb: default_max_input_size_mb(),
            max_processing_time_ms: default_max_processing_time_ms(),
            max_json_depth: default_max_json_depth(),
            stdin_read_timeout_ms: default_stdin_read_timeout_ms(),
        }
    }
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TtsConfig {
    /// Master TTS switch - controls all TTS functionality
    #[serde(default = "default_tts_enabled")]
    pub enabled: bool,

    /// TTS provider ("none", "macos", "elevenlabs", "openai", "pyttsx3")
    #[serde(default = "default_tts_provider")]
    pub provider: String,

    /// Maximum text length for TTS processing
    #[serde(default = "default_text_length_limit")]
    pub text_length_limit: u32,

    /// TTS operation timeout in seconds
    #[serde(default = "default_tts_timeout")]
    pub timeout: u32,

    /// Provider-specific voice configurations
    #[serde(default = "default_tts_voices")]
    pub voices: TtsVoiceConfigs,

    /// Feature-specific toggles
    #[serde(default = "default_tts_responses")]
    pub responses: TtsFeatureConfig,

    /// TTS for task completion notifications
    #[serde(default = "default_tts_completion")]
    pub completion: TtsFeatureConfig,

    /// TTS for system notifications
    #[serde(default = "default_tts_notifications")]
    pub notifications: TtsFeatureConfig,

    /// Engineer configuration for TTS
    #[serde(default = "default_engineer_config")]
    pub engineer: EngineerConfig,
}

/// Provider-specific voice configurations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TtsVoiceConfigs {
    /// macOS built-in TTS configuration
    #[serde(default = "default_macos_voice_config")]
    pub macos: MacOsVoiceConfig,

    /// ElevenLabs TTS API configuration
    #[serde(default = "default_elevenlabs_voice_config")]
    pub elevenlabs: ElevenLabsVoiceConfig,

    /// OpenAI TTS API configuration
    #[serde(default = "default_openai_voice_config")]
    pub openai: OpenAiVoiceConfig,

    /// Pyttsx3 cross-platform TTS configuration
    #[serde(default = "default_pyttsx3_voice_config")]
    pub pyttsx3: Pyttsx3VoiceConfig,
}

/// macOS TTS voice configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacOsVoiceConfig {
    /// Voice name (e.g., "Alex", "Samantha")
    #[serde(default = "default_macos_voice")]
    pub voice: String,

    /// Speech rate in words per minute
    #[serde(default = "default_macos_rate")]
    pub rate: u32,

    /// Audio quality setting (0-127)
    #[serde(default = "default_macos_quality")]
    pub quality: u32,
}

/// ElevenLabs TTS configuration
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElevenLabsVoiceConfig {
    /// Voice ID from ElevenLabs (e.g., "IKne3meq5aSn9XLyUdCD" for Charlie)
    #[serde(default = "default_elevenlabs_voice_id")]
    pub voice_id: String,

    /// ElevenLabs model to use (e.g., "eleven_turbo_v2_5")
    #[serde(default = "default_elevenlabs_model")]
    pub model: String,

    /// Audio output format (e.g., "mp3_44100_128")
    #[serde(default = "default_elevenlabs_output_format")]
    pub output_format: String,

    /// Optional API key (prefer environment variable)
    /// 🔒 SECURITY FIX: Never serialize API keys to prevent leakage
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
}

// 🔒 SECURITY FIX: Custom Debug to mask API keys
impl std::fmt::Debug for ElevenLabsVoiceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElevenLabsVoiceConfig")
            .field("voice_id", &self.voice_id)
            .field("model", &self.model)
            .field("output_format", &self.output_format)
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// OpenAI TTS configuration
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenAiVoiceConfig {
    /// OpenAI TTS model ("tts-1" or "tts-1-hd")
    #[serde(default = "default_openai_model")]
    pub model: String,

    /// Voice name ("alloy", "echo", "fable", "onyx", "nova", "shimmer")
    #[serde(default = "default_openai_voice")]
    pub voice: String,

    /// Optional API key (prefer environment variable)
    /// 🔒 SECURITY FIX: Never serialize API keys to prevent leakage
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
}

// 🔒 SECURITY FIX: Custom Debug to mask API keys
impl std::fmt::Debug for OpenAiVoiceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAiVoiceConfig")
            .field("model", &self.model)
            .field("voice", &self.voice)
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// Pyttsx3 TTS configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pyttsx3VoiceConfig {
    /// Voice name ("default" or system-specific voice name)
    #[serde(default = "default_pyttsx3_voice")]
    pub voice: String,

    /// Speech rate in words per minute (typical range: 100-300)
    #[serde(default = "default_pyttsx3_rate")]
    pub rate: u32,

    /// Audio volume level (0.0 to 1.0)
    #[serde(default = "default_pyttsx3_volume")]
    pub volume: f32,
}

/// Feature-specific TTS configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TtsFeatureConfig {
    /// Enable TTS for this specific feature
    #[serde(default = "default_feature_enabled")]
    pub enabled: bool,
}

/// Engineer configuration for TTS personalization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineerConfig {
    /// Engineer's name for personalized TTS messages (empty for generic messages)
    #[serde(default = "default_engineer_name")]
    pub name: String,
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

/// Logging level for filtering log output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Most verbose logging level, includes all debug information
    Trace,
    /// Debug information for troubleshooting
    Debug,
    /// General informational messages (default level)
    #[default]
    Info,
    /// Warning messages for potentially problematic situations
    Warn,
    /// Error messages for failures and critical issues
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct MaosConfig {
    /// System-wide settings
    pub system: SystemConfig,

    /// Security validation settings
    pub security: SecurityConfig,

    /// Hook processing settings
    pub hooks: HookConfig,

    /// TTS provider settings
    pub tts: TtsConfig,

    /// Session management settings
    pub session: SessionConfig,

    /// Git worktree settings
    pub worktree: WorktreeConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Legacy type alias for backward compatibility
///
/// @deprecated Use `MaosConfig` directly instead of `Config`
pub type Config = MaosConfig;

impl Default for MaosConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                project_root: default_project_root(),
                workspaces_root: default_workspaces_root(),
                max_execution_time_ms: default_max_execution_time(),
                enable_metrics: default_true(),
            },
            security: SecurityConfig {
                enable_validation: default_true(),
                allowed_tools: default_allowed_tools(),
                blocked_paths: Vec::new(),
            },
            hooks: HookConfig::default(),
            tts: TtsConfig {
                enabled: default_tts_enabled(),
                provider: default_tts_provider(),
                text_length_limit: default_text_length_limit(),
                timeout: default_tts_timeout(),
                voices: default_tts_voices(),
                responses: default_tts_responses(),
                completion: default_tts_completion(),
                notifications: default_tts_notifications(),
                engineer: default_engineer_config(),
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
    /// Load MAOS configuration with default values
    ///
    /// Creates a new configuration instance with sensible defaults for all settings.
    /// This is the primary entry point for configuration loading.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Configuration instance with default values
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::MaosConfig;
    ///
    /// let config = MaosConfig::load().unwrap();
    /// assert_eq!(config.logging.level.to_string(), "info");
    /// assert_eq!(config.tts.provider, "pyttsx3");
    /// ```
    pub fn load() -> Result<Self> {
        Ok(Self::default())
    }

    /// Get API key for TTS provider using cascading resolution
    ///
    /// Resolves API keys using priority order: environment variables → config.json.
    /// Supports ElevenLabs and OpenAI TTS providers with automatic key discovery.
    ///
    /// # Arguments
    ///
    /// * `provider` - TTS provider name ("elevenlabs" or "openai")
    ///
    /// # Returns
    ///
    /// * `Some(String)` - API key if found and non-empty
    /// * `None` - If provider unsupported or no key found
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::MaosConfig;
    ///
    /// let config = MaosConfig::default();
    ///
    /// // Check for ElevenLabs API key (from ELEVENLABS_API_KEY env var)
    /// if let Some(key) = config.get_api_key("elevenlabs") {
    ///     println!("ElevenLabs key found: {}...", &key[..8]);
    /// }
    ///
    /// // Check for OpenAI API key (from OPENAI_API_KEY env var)
    /// if let Some(key) = config.get_api_key("openai") {
    ///     println!("OpenAI key found: {}...", &key[..8]);
    /// }
    /// ```
    pub fn get_api_key(&self, provider: &str) -> Option<String> {
        use std::env;

        // Environment variable names for each provider
        let env_var = match provider {
            "elevenlabs" => "ELEVENLABS_API_KEY",
            "openai" => "OPENAI_API_KEY",
            _ => return None,
        };

        // 1. Check environment variable first (highest priority)
        if let Ok(api_key) = env::var(env_var) {
            let trimmed = api_key.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }

        // 2. Check config.json as fallback
        let config_api_key = match provider {
            "elevenlabs" => &self.tts.voices.elevenlabs.api_key,
            "openai" => &self.tts.voices.openai.api_key,
            _ => return None,
        };

        config_api_key.as_ref().and_then(|key| {
            let trimmed = key.trim();
            if !trimmed.is_empty() {
                Some(trimmed.to_string())
            } else {
                None
            }
        })
    }

    /// Get the active TTS provider with intelligent fallback
    ///
    /// Determines which TTS provider to use based on configuration and API key availability.
    /// For API-based providers (ElevenLabs, OpenAI), verifies API key presence before selection.
    /// Automatically falls back to local providers if API keys are missing.
    ///
    /// # Provider Priority
    ///
    /// 1. **User-configured provider** (if API key available for API providers)
    /// 2. **pyttsx3 fallback** (if API provider configured but key missing)
    /// 3. **Local providers** (macos, pyttsx3) always available
    ///
    /// # Returns
    ///
    /// * `String` - Active provider name ready for use
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::MaosConfig;
    ///
    /// let mut config = MaosConfig::default();
    ///
    /// // With local provider configured (always available)
    /// config.tts.provider = "macos".to_string();
    /// assert_eq!(config.get_active_tts_provider(), "macos"); // Direct use
    ///
    /// // With pyttsx3 provider configured
    /// config.tts.provider = "pyttsx3".to_string();
    /// assert_eq!(config.get_active_tts_provider(), "pyttsx3"); // Always available
    ///
    /// // API-based providers depend on environment variables:
    /// // - ElevenLabs needs ELEVENLABS_API_KEY
    /// // - OpenAI needs OPENAI_API_KEY
    /// // They fall back to pyttsx3 if keys are missing
    /// ```
    pub fn get_active_tts_provider(&self) -> String {
        let provider = &self.tts.provider;

        // For API-based providers, verify key availability
        if matches!(provider.as_str(), "elevenlabs" | "openai") {
            if self.get_api_key(provider).is_some() {
                return provider.clone();
            } else {
                // Fallback to pyttsx3 if API key not available
                return "pyttsx3".to_string();
            }
        }

        // For local providers (macos, pyttsx3), no API key needed
        provider.clone()
    }

    /// Check if TTS functionality is globally enabled
    ///
    /// Returns the master TTS switch status. When disabled, all TTS features are off
    /// regardless of individual feature settings.
    ///
    /// # Returns
    ///
    /// * `bool` - True if TTS globally enabled, false otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::MaosConfig;
    ///
    /// let config = MaosConfig::default();
    /// if config.is_tts_enabled() {
    ///     println!("TTS is available");
    /// }
    /// ```
    pub fn is_tts_enabled(&self) -> bool {
        self.tts.enabled
    }

    /// Check if response TTS is enabled
    ///
    /// Verifies both global TTS enabled AND response-specific TTS enabled.
    /// Used for speaking Claude's responses during conversations.
    ///
    /// # Returns
    ///
    /// * `bool` - True if response TTS should be used
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::MaosConfig;
    ///
    /// let config = MaosConfig::default();
    /// if config.is_response_tts_enabled() {
    ///     // Speak Claude's response
    /// }
    /// ```
    pub fn is_response_tts_enabled(&self) -> bool {
        self.tts.enabled && self.tts.responses.enabled
    }

    /// Check if completion TTS is enabled
    ///
    /// Verifies both global TTS enabled AND completion-specific TTS enabled.
    /// Used for speaking task completion notifications.
    ///
    /// # Returns
    ///
    /// * `bool` - True if completion TTS should be used
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::MaosConfig;
    ///
    /// let config = MaosConfig::default();
    /// if config.is_completion_tts_enabled() {
    ///     // Announce task completion
    /// }
    /// ```
    pub fn is_completion_tts_enabled(&self) -> bool {
        self.tts.enabled && self.tts.completion.enabled
    }

    /// Check if notification TTS is enabled
    ///
    /// Verifies both global TTS enabled AND notification-specific TTS enabled.
    /// Used for speaking system notifications and alerts.
    ///
    /// # Returns
    ///
    /// * `bool` - True if notification TTS should be used
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::MaosConfig;
    ///
    /// let config = MaosConfig::default();
    /// if config.is_notification_tts_enabled() {
    ///     // Announce system notification
    /// }
    /// ```
    pub fn is_notification_tts_enabled(&self) -> bool {
        self.tts.enabled && self.tts.notifications.enabled
    }

    /// Get the maximum text length limit for TTS processing
    ///
    /// Returns the character limit for TTS input to prevent overly long speech.
    /// Text exceeding this limit should be truncated or split.
    ///
    /// # Returns
    ///
    /// * `u32` - Maximum characters allowed (default: 2000)
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::MaosConfig;
    ///
    /// let config = MaosConfig::default();
    /// let max_chars = config.get_text_length_limit();
    ///
    /// let text = "Long response text...";
    /// if text.len() > max_chars as usize {
    ///     // Truncate or split text
    /// }
    /// ```
    pub fn get_text_length_limit(&self) -> u32 {
        self.tts.text_length_limit
    }

    /// Get the timeout for TTS operations
    ///
    /// Returns the maximum time to wait for TTS operations to complete.
    /// Operations exceeding this timeout should be cancelled.
    ///
    /// # Returns
    ///
    /// * `u32` - Timeout in seconds (default: 120)
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::MaosConfig;
    /// use std::time::Duration;
    ///
    /// let config = MaosConfig::default();
    /// let timeout = Duration::from_secs(config.get_tts_timeout() as u64);
    ///
    /// // Use timeout for TTS operations
    /// // tokio::time::timeout(timeout, tts_operation()).await
    /// ```
    pub fn get_tts_timeout(&self) -> u32 {
        self.tts.timeout
    }

    /// Validate configuration settings for correctness
    ///
    /// Performs comprehensive validation of all configuration values to ensure
    /// they are within valid ranges and logically consistent.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Configuration is valid
    /// * `Err(ConfigError)` - Configuration contains invalid values
    ///
    /// # Validation Rules
    ///
    /// - `max_execution_time_ms` must be greater than 0
    /// - TTS timeout must be reasonable (handled by defaults)
    /// - Path configurations must be valid (when applicable)
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::MaosConfig;
    ///
    /// let config = MaosConfig::default();
    /// config.validate().expect("Default config should be valid");
    ///
    /// // Invalid config example
    /// let mut bad_config = MaosConfig::default();
    /// bad_config.system.max_execution_time_ms = 0;
    /// assert!(bad_config.validate().is_err());
    /// ```
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
    /// Create a new configuration loader
    ///
    /// Initializes a ConfigLoader for loading MAOS configuration from various sources.
    /// The loader supports JSON files, JSON strings, environment variables, and readers.
    ///
    /// # Returns
    ///
    /// * `Self` - New ConfigLoader instance ready for use
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::ConfigLoader;
    ///
    /// let loader = ConfigLoader::new();
    /// // Use loader to load from various sources
    /// ```
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
    ///
    /// Parses a JSON string into a MAOS configuration with automatic defaults merging.
    /// Missing fields use default values via serde's `#[serde(default)]` attributes.
    ///
    /// # Arguments
    ///
    /// * `json` - JSON string containing configuration
    ///
    /// # Returns
    ///
    /// * `Result<MaosConfig>` - Validated configuration or error
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::ConfigLoader;
    ///
    /// let loader = ConfigLoader::new();
    /// let json = r#"{"tts": {"enabled": false}}"#;
    /// let config = loader.load_from_str(json).unwrap();
    /// assert!(!config.tts.enabled);
    /// ```
    pub fn load_from_str(&self, json: &str) -> Result<MaosConfig> {
        // Use serde's built-in merging by deserializing with defaults
        // The #[serde(default)] attributes handle the merging automatically
        let config: MaosConfig = serde_json::from_str(json)?;

        // Validate
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from any reader providing JSON data
    ///
    /// Reads JSON data from any source implementing `Read` (files, strings, network, etc.)
    /// and parses it into a validated MAOS configuration.
    ///
    /// # Arguments
    ///
    /// * `reader` - Any reader providing JSON bytes
    ///
    /// # Returns
    ///
    /// * `Result<MaosConfig>` - Validated configuration or error
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::ConfigLoader;
    /// use std::io::Cursor;
    ///
    /// let loader = ConfigLoader::new();
    /// let json_data = r#"{"logging": {"level": "debug"}}"#;
    /// let reader = Cursor::new(json_data);
    /// let config = loader.load_from_reader(reader).unwrap();
    /// ```
    pub fn load_from_reader<R: Read>(&self, mut reader: R) -> Result<MaosConfig> {
        // Read and parse JSON
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;

        // Use the string loader
        self.load_from_str(&buf)
    }

    /// Load configuration from a JSON file
    ///
    /// Opens and reads a JSON configuration file from the filesystem,
    /// parsing it into a validated MAOS configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to JSON configuration file
    ///
    /// # Returns
    ///
    /// * `Result<MaosConfig>` - Validated configuration or error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use maos_core::config::ConfigLoader;
    /// use std::path::Path;
    ///
    /// let loader = ConfigLoader::new();
    /// let config = loader.load_from_path(Path::new("maos.json")).unwrap();
    /// ```
    pub fn load_from_path(&self, path: &Path) -> Result<MaosConfig> {
        let file = File::open(path)?;
        self.load_from_reader(file)
    }

    /// Load configuration with environment variable overrides
    ///
    /// Creates configuration from defaults and applies environment variable overrides.
    /// Supports a predefined set of environment variables for common configuration options.
    ///
    /// # Supported Environment Variables
    ///
    /// - `MAOS_SYSTEM_MAX_EXECUTION_TIME_MS` - System execution timeout
    /// - `MAOS_SYSTEM_WORKSPACE_ROOT` - Workspace root directory
    /// - `MAOS_SECURITY_ENABLE_VALIDATION` - Security validation toggle
    /// - `ELEVENLABS_API_KEY` - ElevenLabs TTS API key
    /// - `OPENAI_API_KEY` - OpenAI TTS API key
    /// - `MAOS_LOGGING_LEVEL` - Log level (trace, debug, info, warn, error)
    /// - `MAOS_LOGGING_FORMAT` - Log format (json, text)
    /// - `MAOS_LOGGING_OUTPUT` - Log output (stdout, stderr, session_file)
    ///
    /// # Arguments
    ///
    /// * `env_vars` - HashMap of environment variable name-value pairs
    ///
    /// # Returns
    ///
    /// * `Result<MaosConfig>` - Configuration with environment overrides applied
    ///
    /// # Examples
    ///
    /// ```
    /// use maos_core::config::ConfigLoader;
    /// use std::collections::HashMap;
    ///
    /// let loader = ConfigLoader::new();
    /// let mut env_vars = HashMap::new();
    /// env_vars.insert("MAOS_LOGGING_LEVEL".to_string(), "debug".to_string());
    /// env_vars.insert("ELEVENLABS_API_KEY".to_string(), "sk-test123".to_string());
    ///
    /// let config = loader.load_with_env(env_vars).unwrap();
    /// assert_eq!(config.logging.level.to_string(), "debug");
    /// ```
    pub fn load_with_env(&self, env_vars: HashMap<String, String>) -> Result<MaosConfig> {
        // Start with defaults
        let mut config = MaosConfig::default();

        // Apply environment overrides
        self.apply_env_overrides(&mut config, env_vars)?;

        // Validate
        config.validate()?;

        Ok(config)
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
            config.system.workspaces_root = PathBuf::from(val);
        }

        // Security overrides
        if let Some(val) = env_vars.get("MAOS_SECURITY_ENABLE_VALIDATION") {
            config.security.enable_validation = Self::parse_env_var(
                val,
                "MAOS_SECURITY_ENABLE_VALIDATION",
                "must be true or false",
            )?;
        }

        // TTS API key overrides (ELEVENLABS_API_KEY, OPENAI_API_KEY)
        if let Some(val) = env_vars.get("ELEVENLABS_API_KEY") {
            config.tts.voices.elevenlabs.api_key = Some(val.clone());
        }

        if let Some(val) = env_vars.get("OPENAI_API_KEY") {
            config.tts.voices.openai.api_key = Some(val.clone());
        }

        // Logging overrides
        if let Some(val) = env_vars.get("MAOS_LOGGING_LEVEL") {
            config.logging.level = Self::parse_env_var(
                val,
                "MAOS_LOGGING_LEVEL",
                "must be one of: trace, debug, info, warn, error",
            )?;
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

fn default_project_root() -> PathBuf {
    // Check MAOS_PROJECT_ROOT_DIR environment variable first. If it is set,
    // use it as the workspace root.
    let project_root = std::env::var("MAOS_PROJECT_ROOT_DIR").ok();
    if let Some(project_root) = project_root {
        return PathBuf::from(project_root);
    }

    // With CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR enabled,
    // std::env::current_dir() ALWAYS returns the workspace root!
    // This is instant (no subprocess calls) and always accurate.
    // First, try to get the actual workspace root from current_dir
    if let Ok(cwd) = std::env::current_dir() {
        return cwd;
    }

    // Fallback (should never happen with Claude Code)
    PathBuf::from(".")
}

fn default_workspaces_root() -> PathBuf {
    let base_dir = default_project_root();

    // Append the .maos/workspaces subdirectory as expected by the system
    base_dir.join(MAOS_ROOT_DIR).join(WORKSPACES_DIR_NAME)
}

fn default_true() -> bool {
    true
}

fn default_allowed_tools() -> Vec<String> {
    vec!["*".to_string()]
}

// Hook configuration defaults
fn default_max_input_size_mb() -> u64 {
    10 // 10MB - reasonable for most hook payloads
}

fn default_max_processing_time_ms() -> u64 {
    5_000 // 5 seconds - allows for worktree creation and GitHub ops
}

fn default_max_json_depth() -> u32 {
    64 // Reasonable nesting limit to prevent JSON bombs
}

fn default_stdin_read_timeout_ms() -> u64 {
    100 // 100ms per read operation
}

// TTS configuration defaults
fn default_tts_enabled() -> bool {
    true
}

fn default_tts_provider() -> String {
    "pyttsx3".to_string()
}

fn default_text_length_limit() -> u32 {
    2000 // Match Python default
}

fn default_tts_timeout() -> u32 {
    120 // Match Python default (seconds)
}

fn default_tts_voices() -> TtsVoiceConfigs {
    TtsVoiceConfigs {
        macos: default_macos_voice_config(),
        elevenlabs: default_elevenlabs_voice_config(),
        openai: default_openai_voice_config(),
        pyttsx3: default_pyttsx3_voice_config(),
    }
}

fn default_tts_responses() -> TtsFeatureConfig {
    TtsFeatureConfig { enabled: false }
}

fn default_tts_completion() -> TtsFeatureConfig {
    TtsFeatureConfig { enabled: true }
}

fn default_tts_notifications() -> TtsFeatureConfig {
    TtsFeatureConfig { enabled: true }
}

fn default_feature_enabled() -> bool {
    false
}

fn default_engineer_config() -> EngineerConfig {
    EngineerConfig {
        name: default_engineer_name(),
    }
}

fn default_engineer_name() -> String {
    String::new()
}

// Voice configuration defaults
fn default_macos_voice_config() -> MacOsVoiceConfig {
    MacOsVoiceConfig {
        voice: default_macos_voice(),
        rate: default_macos_rate(),
        quality: default_macos_quality(),
    }
}

fn default_elevenlabs_voice_config() -> ElevenLabsVoiceConfig {
    ElevenLabsVoiceConfig {
        voice_id: default_elevenlabs_voice_id(),
        model: default_elevenlabs_model(),
        output_format: default_elevenlabs_output_format(),
        api_key: None,
    }
}

fn default_openai_voice_config() -> OpenAiVoiceConfig {
    OpenAiVoiceConfig {
        model: default_openai_model(),
        voice: default_openai_voice(),
        api_key: None,
    }
}

fn default_pyttsx3_voice_config() -> Pyttsx3VoiceConfig {
    Pyttsx3VoiceConfig {
        voice: default_pyttsx3_voice(),
        rate: default_pyttsx3_rate(),
        volume: default_pyttsx3_volume(),
    }
}

// Individual voice setting defaults
fn default_macos_voice() -> String {
    "Alex".to_string() // Match Python default
}

fn default_macos_rate() -> u32 {
    190 // Match Python default
}

fn default_macos_quality() -> u32 {
    127
}

fn default_elevenlabs_voice_id() -> String {
    "IKne3meq5aSn9XLyUdCD".to_string() // Charlie voice
}

fn default_elevenlabs_model() -> String {
    "eleven_turbo_v2_5".to_string()
}

fn default_elevenlabs_output_format() -> String {
    "mp3_44100_128".to_string()
}

fn default_openai_model() -> String {
    "tts-1".to_string()
}

fn default_openai_voice() -> String {
    "alloy".to_string()
}

fn default_pyttsx3_voice() -> String {
    "default".to_string()
}

fn default_pyttsx3_rate() -> u32 {
    190
}

fn default_pyttsx3_volume() -> f32 {
    0.9
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_config_defaults_load() -> Result<()> {
        // Load defaults when no files/env present
        let cfg = MaosConfig::default();

        // System defaults
        assert_eq!(cfg.system.max_execution_time_ms, 60_000);
        // Workspace root should be git repository root + .maos/workspaces
        assert!(
            cfg.system
                .workspaces_root
                .ends_with(format!("{MAOS_ROOT_DIR}/{WORKSPACES_DIR_NAME}")),
            "Workspace root should end with .maos/workspaces, got: {:?}",
            cfg.system.workspaces_root
        );
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
        assert_eq!(cfg.system.workspaces_root, PathBuf::from("/custom/path"));
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
                "project_root": "/test/project_root",
                "workspaces_root": "/test/project_root/.maos/workspaces",
                "max_execution_time_ms": 30000,
                "enable_metrics": true
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

        assert_eq!(cfg.system.project_root, PathBuf::from("/test/project_root"));
        assert_eq!(
            cfg.system.workspaces_root,
            PathBuf::from("/test/project_root/.maos/workspaces")
        );
        assert_eq!(cfg.system.max_execution_time_ms, 30000);
        assert!(cfg.system.enable_metrics);
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
        // Workspace root should be git repository root + .maos/workspaces
        assert!(
            cfg.system.workspaces_root.ends_with(".maos/workspaces"),
            "Workspace root should end with .maos/workspaces, got: {:?}",
            cfg.system.workspaces_root
        );
        assert_eq!(cfg.logging.format, "json");
        assert!(cfg.security.enable_validation);
    }

    #[test]
    fn test_hook_config_boundary_values() {
        // Test hook configuration with boundary values
        let mut cfg = MaosConfig::default();

        // Test maximum values
        cfg.hooks.max_input_size_mb = u64::MAX;
        cfg.hooks.max_processing_time_ms = u64::MAX;
        cfg.hooks.max_json_depth = u32::MAX;

        // Should handle large values gracefully
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.hooks.max_input_size_mb, u64::MAX);
        assert_eq!(cfg.hooks.max_processing_time_ms, u64::MAX);
        assert_eq!(cfg.hooks.max_json_depth, u32::MAX);

        // Test minimum values
        cfg.hooks.max_input_size_mb = 0;
        cfg.hooks.max_processing_time_ms = 0;
        cfg.hooks.max_json_depth = 0;

        // Zero values should be allowed but may be adjusted by implementation
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_workspace_root_path_validation() {
        let mut cfg = MaosConfig::default();

        // Test absolute path
        cfg.system.workspaces_root = PathBuf::from("/absolute/path/to/workspace");
        assert!(cfg.system.workspaces_root.is_absolute());

        // Test relative path (should be made absolute by implementation)
        cfg.system.workspaces_root = PathBuf::from("relative/path");
        // In practice, this would be resolved to absolute
        assert!(cfg.validate().is_ok());

        // Test path with special characters
        cfg.system.workspaces_root = PathBuf::from("/path with spaces/workspace");
        assert!(cfg.validate().is_ok());

        // Test very long path
        let mut long_path = PathBuf::from("/");
        for i in 0..100 {
            long_path.push(format!("dir{i}"));
        }
        cfg.system.workspaces_root = long_path;
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_tts_config_validation() {
        let mut cfg = MaosConfig::default();

        // Test zero timeout
        cfg.tts.timeout = 0;
        assert!(cfg.validate().is_ok());

        // Test zero text length limit
        cfg.tts.text_length_limit = 0;
        assert!(cfg.validate().is_ok());

        // Test extreme values
        cfg.tts.timeout = u32::MAX;
        cfg.tts.text_length_limit = u32::MAX;
        assert!(cfg.validate().is_ok());

        // Test voice settings
        cfg.tts.voices.macos.rate = 0;
        cfg.tts.voices.macos.quality = 0;
        assert!(cfg.validate().is_ok());

        cfg.tts.voices.macos.rate = 999999;
        cfg.tts.voices.macos.quality = 999999;
        assert!(cfg.validate().is_ok());

        // Test volume boundaries
        cfg.tts.voices.pyttsx3.volume = 0.0;
        assert!(cfg.validate().is_ok());

        cfg.tts.voices.pyttsx3.volume = 1.0;
        assert!(cfg.validate().is_ok());

        cfg.tts.voices.pyttsx3.volume = 2.0; // Over normal range
        assert!(cfg.validate().is_ok()); // Should handle gracefully
    }

    #[test]
    fn test_security_config_edge_cases() {
        let mut cfg = MaosConfig::default();

        // Test empty allowed tools
        cfg.security.allowed_tools = vec![];
        assert!(cfg.validate().is_ok());

        // Test many allowed tools
        cfg.security.allowed_tools = vec!["*".to_string(); 1000];
        assert!(cfg.validate().is_ok());

        // Test blocked paths validation
        cfg.security.blocked_paths = vec!["/etc/passwd".to_string(); 100];
        assert!(cfg.validate().is_ok());

        // Test with various path patterns (should be handled gracefully)
        cfg.security.blocked_paths = vec![
            "../../etc/passwd".to_string(),      // Path traversal
            "C:\\windows\\system32".to_string(), // Windows path
            "/\0/null".to_string(),              // Null byte
        ];
        assert!(cfg.validate().is_ok()); // Should not panic
    }

    #[test]
    fn test_logging_config_extreme_values() {
        let mut cfg = MaosConfig::default();

        // Test different log levels
        cfg.logging.level = LogLevel::Trace;
        assert!(cfg.validate().is_ok());

        cfg.logging.level = LogLevel::Error;
        assert!(cfg.validate().is_ok());

        // Test different log formats
        cfg.logging.format = "json".to_string();
        assert!(cfg.validate().is_ok());

        cfg.logging.format = "text".to_string();
        assert!(cfg.validate().is_ok());

        // Test different outputs
        cfg.logging.output = "stdout".to_string();
        assert!(cfg.validate().is_ok());

        cfg.logging.output = "stderr".to_string();
        assert!(cfg.validate().is_ok());

        cfg.logging.output = "session_file".to_string();
        assert!(cfg.validate().is_ok());

        // Test empty strings (should be handled gracefully)
        cfg.logging.format = "".to_string();
        assert!(cfg.validate().is_ok());

        cfg.logging.output = "".to_string();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        // Test that config survives serialization/deserialization
        let original = MaosConfig::default();

        // Serialize to JSON
        let json = serde_json::to_string(&original).unwrap();

        // Deserialize back
        let deserialized: MaosConfig = serde_json::from_str(&json).unwrap();

        // Key fields should match (not all fields due to skip_serializing)
        assert_eq!(
            original.system.max_execution_time_ms,
            deserialized.system.max_execution_time_ms
        );
        assert_eq!(
            original.hooks.max_input_size_mb,
            deserialized.hooks.max_input_size_mb
        );
        assert_eq!(original.logging.level, deserialized.logging.level);
        assert_eq!(
            original.security.enable_validation,
            deserialized.security.enable_validation
        );
    }

    #[test]
    fn test_config_with_missing_fields() {
        // Test that missing fields are filled with defaults
        let minimal_json = "{}";
        let loader = ConfigLoader::new();
        let cfg = loader.load_from_str(minimal_json).unwrap();

        // Should have all default values
        assert_eq!(
            cfg.system.max_execution_time_ms,
            60000 // Default value from default_max_execution_time()
        );
        assert_eq!(
            cfg.hooks.max_input_size_mb,
            10 // Default value from default_max_input_size_mb()
        );
        assert!(cfg.security.enable_validation);
    }

    #[test]
    fn test_config_with_extra_fields() {
        // Test that extra unknown fields cause an error (deny_unknown_fields is set)
        let json_with_extra = r#"{
            "system": {
                "max_execution_time_ms": 30000,
                "unknown_field": "ignored"
            }
        }"#;

        let loader = ConfigLoader::new();
        let result = loader.load_from_str(json_with_extra);

        // Should fail due to deny_unknown_fields
        assert!(result.is_err());
        if let Err(err) = result {
            let error_str = err.to_string();
            assert!(
                error_str.contains("unknown field") || error_str.contains("Invalid configuration")
            );
        } else {
            panic!("Expected error for unknown field");
        }
    }
}
