//! Configuration types for the logging system

use serde::{Deserialize, Serialize};

use crate::constants::{LOG_FILE_PATTERN, MAX_LOG_FILE_SIZE, MAX_LOG_FILES_PER_SESSION};

/// Log levels with proper ordering (higher number = higher severity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum LogLevel {
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
}

/// Output format for log entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Plain,
    Pretty,
}

/// Where to output logs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogOutput {
    Stdout,
    SessionFile,
    Both,
}

/// Configuration for log file rotation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollingLogConfig {
    /// Maximum file size in bytes before rotation
    pub max_file_size_bytes: usize,
    /// Maximum number of rotated files to keep per session
    pub max_files_per_session: usize,
    /// Whether to compress rotated files
    pub compress_on_roll: bool,
    /// File name pattern with {session_id} placeholder
    pub file_pattern: String,
}

impl Default for RollingLogConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: MAX_LOG_FILE_SIZE,
            max_files_per_session: MAX_LOG_FILES_PER_SESSION,
            compress_on_roll: true,
            file_pattern: LOG_FILE_PATTERN.to_string(),
        }
    }
}

/// Main logging configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level threshold
    pub level: LogLevel,
    /// Output format
    pub format: LogFormat,
    /// Output destination
    pub output: LogOutput,
    /// Enable performance-related logs
    pub enable_performance_logs: bool,
    /// Enable security-related logs
    pub enable_security_logs: bool,
    /// Rolling log configuration
    pub rolling: RollingLogConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_serialization() {
        let config = LoggingConfig {
            level: LogLevel::Debug,
            format: LogFormat::Json,
            output: LogOutput::Both,
            enable_performance_logs: true,
            enable_security_logs: false,
            rolling: RollingLogConfig::default(),
        };

        // Test serialization
        let json = serde_json::to_string(&config).expect("Failed to serialize");
        assert!(json.contains("\"level\":\"debug\""));
        assert!(json.contains("\"format\":\"json\""));

        // Test deserialization
        let deserialized: LoggingConfig =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert!(matches!(deserialized.level, LogLevel::Debug));
        assert!(matches!(deserialized.format, LogFormat::Json));
    }

    #[test]
    fn test_log_level_ordering() {
        // Test that log levels have correct severity ordering
        assert!(LogLevel::Error as u8 > LogLevel::Warn as u8);
        assert!(LogLevel::Warn as u8 > LogLevel::Info as u8);
        assert!(LogLevel::Info as u8 > LogLevel::Debug as u8);
        assert!(LogLevel::Debug as u8 > LogLevel::Trace as u8);
    }

    #[test]
    fn test_default_rolling_config() {
        let config = RollingLogConfig::default();

        // Should use constants from constants module
        assert_eq!(
            config.max_file_size_bytes,
            crate::constants::MAX_LOG_FILE_SIZE
        );
        assert_eq!(config.max_files_per_session, 10);
        assert!(config.compress_on_roll);
        assert_eq!(config.file_pattern, "session-{session_id}.log");
    }
}
