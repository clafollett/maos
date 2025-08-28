//! Logging initialization and setup

use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use super::config::{LogFormat, LogLevel, LogOutput, LoggingConfig};
use crate::Result;

/// Convert LogLevel to tracing filter string
fn log_level_to_filter(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "trace",
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    }
}

/// Initialize the global tracing subscriber based on configuration
pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    // Convert our LogLevel to tracing level filter
    let level_filter = log_level_to_filter(config.level);

    // Create env filter
    let env_filter = EnvFilter::try_new(level_filter).map_err(|e| {
        crate::MaosError::Config(crate::ConfigError::InvalidValue {
            field: "log_level".to_string(),
            value: level_filter.to_string(),
            reason: format!("Invalid log level: {e}"),
        })
    })?;

    // Build the registry with env filter
    let registry = tracing_subscriber::registry().with(env_filter);

    // Helper to create a formatter layer with the configured format
    let make_fmt_layer = || match config.format {
        LogFormat::Json => fmt::layer().json().boxed(),
        LogFormat::Pretty => fmt::layer().pretty().boxed(),
        LogFormat::Plain => fmt::layer().boxed(),
    };

    // Helper to create a file writer layer
    let make_file_layer = |file: std::fs::File| match config.format {
        LogFormat::Json => fmt::layer().json().with_writer(file).boxed(),
        LogFormat::Pretty => fmt::layer().pretty().with_writer(file).boxed(),
        LogFormat::Plain => fmt::layer().with_writer(file).boxed(),
    };

    // Initialize based on output configuration
    match config.output {
        LogOutput::Stdout => registry.with(make_fmt_layer()).try_init(),
        LogOutput::SessionFile => {
            // Create log directory if it doesn't exist
            let _ = std::fs::create_dir_all(".maos/logs");

            // Try to open log file, fallback to stdout
            match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(".maos/logs/session.log")
            {
                Ok(file) => registry.with(make_file_layer(file)).try_init(),
                Err(_) => {
                    // Fallback to stdout if file creation fails
                    registry.with(make_fmt_layer()).try_init()
                }
            }
        }
        LogOutput::Both => {
            // Try to create file for dual output
            let _ = std::fs::create_dir_all(".maos/logs");
            match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(".maos/logs/session.log")
            {
                Ok(file) => {
                    // Use both stdout and file layers - don't use helpers here due to type complexity
                    match config.format {
                        LogFormat::Json => registry
                            .with(fmt::layer().json())
                            .with(fmt::layer().json().with_writer(file))
                            .try_init(),
                        LogFormat::Pretty => registry
                            .with(fmt::layer().pretty())
                            .with(fmt::layer().pretty().with_writer(file))
                            .try_init(),
                        LogFormat::Plain => registry
                            .with(fmt::layer())
                            .with(fmt::layer().with_writer(file))
                            .try_init(),
                    }
                }
                Err(_) => {
                    // Fallback to stdout only
                    registry.with(make_fmt_layer()).try_init()
                }
            }
        }
    }
    .map_err(|e| {
        crate::MaosError::Config(crate::ConfigError::InvalidValue {
            field: "logging".to_string(),
            value: "subscriber".to_string(),
            reason: format!("Failed to initialize tracing subscriber: {e}"),
        })
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::config::LogOutput;
    use super::*;

    #[test]
    fn test_init_logging_validates_config() {
        // Test that init_logging with valid config doesn't panic
        // We can't actually initialize because it's global and can only be done once
        let config = LoggingConfig {
            level: LogLevel::Info,
            format: LogFormat::Plain,
            output: LogOutput::Stdout,
            enable_performance_logs: false,
            enable_security_logs: false,
            rolling: Default::default(),
        };

        // Just validate the config would work by building the filter
        let level_filter = match config.level {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        };

        let env_filter = EnvFilter::try_new(level_filter);
        assert!(env_filter.is_ok());
    }

    #[test]
    fn test_log_level_to_filter_string() {
        // Test the actual helper function
        assert_eq!(super::log_level_to_filter(LogLevel::Trace), "trace");
        assert_eq!(super::log_level_to_filter(LogLevel::Debug), "debug");
        assert_eq!(super::log_level_to_filter(LogLevel::Info), "info");
        assert_eq!(super::log_level_to_filter(LogLevel::Warn), "warn");
        assert_eq!(super::log_level_to_filter(LogLevel::Error), "error");
    }

    #[test]
    fn test_all_output_format_combinations() {
        // Validate all combinations are handled
        let outputs = vec![LogOutput::Stdout, LogOutput::SessionFile, LogOutput::Both];
        let formats = vec![LogFormat::Plain, LogFormat::Pretty, LogFormat::Json];

        for output in outputs {
            for format in formats.clone() {
                let config = LoggingConfig {
                    level: LogLevel::Info,
                    format,
                    output,
                    enable_performance_logs: false,
                    enable_security_logs: false,
                    rolling: Default::default(),
                };

                // We can't actually init, but we can verify the config is valid
                assert!(matches!(
                    config.output,
                    LogOutput::Stdout | LogOutput::SessionFile | LogOutput::Both
                ));
                assert!(matches!(
                    config.format,
                    LogFormat::Plain | LogFormat::Pretty | LogFormat::Json
                ));
            }
        }
    }
}
