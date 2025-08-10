//! Logging initialization and setup

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use super::config::{LogFormat, LogLevel, LogOutput, LoggingConfig};
use crate::Result;

/// Initialize the global tracing subscriber based on configuration
pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    // Convert our LogLevel to tracing level filter
    let level_filter = match config.level {
        LogLevel::Trace => "trace",
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    };

    // Create env filter
    let env_filter = EnvFilter::try_new(level_filter).map_err(|e| {
        crate::MaosError::Config(crate::ConfigError::InvalidValue {
            field: "log_level".to_string(),
            value: level_filter.to_string(),
            reason: format!("Invalid log level: {}", e),
        })
    })?;

    // Create subscriber based on output and format configuration
    match (config.output, config.format) {
        (LogOutput::Stdout, LogFormat::Json) => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json())
                .try_init()
                .map_err(|e| {
                    crate::MaosError::Config(crate::ConfigError::InvalidValue {
                        field: "logging".to_string(),
                        value: "subscriber".to_string(),
                        reason: format!("Failed to initialize tracing subscriber: {}", e),
                    })
                })?;
        }
        (LogOutput::Stdout, LogFormat::Pretty) => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().pretty())
                .try_init()
                .map_err(|e| {
                    crate::MaosError::Config(crate::ConfigError::InvalidValue {
                        field: "logging".to_string(),
                        value: "subscriber".to_string(),
                        reason: format!("Failed to initialize tracing subscriber: {}", e),
                    })
                })?;
        }
        (LogOutput::Stdout, LogFormat::Plain) => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer())
                .try_init()
                .map_err(|e| {
                    crate::MaosError::Config(crate::ConfigError::InvalidValue {
                        field: "logging".to_string(),
                        value: "subscriber".to_string(),
                        reason: format!("Failed to initialize tracing subscriber: {}", e),
                    })
                })?;
        }
        (LogOutput::SessionFile, _) | (LogOutput::Both, _) => {
            // For now, just use stdout. File-based logging would require
            // more complex setup with the session logger
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer())
                .try_init()
                .map_err(|e| {
                    crate::MaosError::Config(crate::ConfigError::InvalidValue {
                        field: "logging".to_string(),
                        value: "subscriber".to_string(),
                        reason: format!("Failed to initialize tracing subscriber: {}", e),
                    })
                })?;
        }
    }

    Ok(())
}
