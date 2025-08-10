//! Logging infrastructure for MAOS
//!
//! This module provides a thin wrapper around the `tracing` ecosystem (the de facto
//! standard for Rust logging) with MAOS-specific enhancements for session management,
//! log rotation, and compression.
//!
//! # Architecture
//!
//! The logging system is built on top of:
//! - **`tracing`** - For structured logging and instrumentation
//! - **`tracing-subscriber`** - For log formatting and filtering
//! - **Custom `SessionLogger`** - For per-session file management and rotation
//!
//! # Why Not Just Use tracing Directly?
//!
//! While `tracing` provides excellent logging primitives, MAOS needs:
//! - **Session isolation** - Each session gets its own log files
//! - **Automatic rotation** - Prevent unbounded log growth
//! - **Compression** - Reduce storage for archived logs
//! - **Thread-safe file writes** - Multiple agents logging concurrently
//!
//! # Features
//!
//! - **Structured logging** with JSON, plain text, or pretty formats
//! - **Per-session log files** with automatic directory creation
//! - **Size-based rotation** with configurable limits
//! - **Optional gzip compression** for rotated logs
//! - **Thread-safe operations** using `parking_lot::RwLock`
//! - **Zero-cost when disabled** via compile-time filtering
//!
//! # Examples
//!
//! ## Basic Setup
//!
//! ```rust,no_run
//! use maos_core::logging::{LoggingConfig, LogLevel, LogFormat, LogOutput, init_logging};
//! use maos_core::SessionId;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = LoggingConfig {
//!     level: LogLevel::Info,
//!     format: LogFormat::Json,
//!     output: LogOutput::Both,
//!     enable_performance_logs: true,
//!     enable_security_logs: true,
//!     rolling: Default::default(),
//! };
//!
//! init_logging(&config)?;
//!
//! // Now use standard tracing macros
//! let session_id = SessionId::generate();
//! tracing::info!("MAOS initialized");
//! tracing::debug!(session_id = ?session_id, "Session started");
//! # Ok(())
//! # }
//! ```
//!
//! ## Session-Based Logging
//!
//! ```rust,no_run
//! use maos_core::logging::{SessionLogger, RollingLogConfig};
//! use maos_core::SessionId;
//! use std::path::PathBuf;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let session_id = SessionId::generate();
//! let log_dir = PathBuf::from("/var/log/maos/sessions");
//! let config = RollingLogConfig::default();
//!
//! let mut logger = SessionLogger::new(session_id, log_dir, config)?;
//!
//! // Write structured log entries
//! logger.write(r#"{"level":"info","msg":"Agent started"}"#)?;
//! logger.write(r#"{"level":"debug","msg":"Processing request"}"#)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Thread-Safe Concurrent Logging
//!
//! ```rust,no_run
//! use maos_core::logging::{SessionLogger, RollingLogConfig};
//! use maos_core::SessionId;
//! use std::sync::Arc;
//! use std::thread;
//! use std::path::PathBuf;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let session_id = SessionId::generate();
//! let log_dir = PathBuf::from("/tmp/logs");
//! let config = RollingLogConfig::default();
//! let logger = Arc::new(SessionLogger::new(session_id, log_dir, config)?.into_thread_safe());
//!
//! let handles: Vec<_> = (0..10)
//!     .map(|i| {
//!         let logger = Arc::clone(&logger);
//!         thread::spawn(move || {
//!             logger.write(&format!("Thread {} log entry", i)).unwrap();
//!         })
//!     })
//!     .collect();
//!
//! for handle in handles {
//!     handle.join().unwrap();
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Performance
//!
//! - **Minimal overhead** - Tracing macros compile to no-ops when disabled
//! - **Buffered writes** - Reduces syscall overhead
//! - **Lock-free reads** - Using `parking_lot::RwLock`
//! - **Lazy formatting** - Messages only formatted if level is enabled
//!
//! # Integration with Tracing Ecosystem
//!
//! Since we use standard `tracing`, you get compatibility with:
//! - OpenTelemetry exporters
//! - Jaeger/Zipkin tracing
//! - Application performance monitoring (APM) tools
//! - Custom subscribers and layers

mod config;
mod init;
mod session;

pub use config::{LogFormat, LogLevel, LogOutput, LoggingConfig, RollingLogConfig};
pub use init::init_logging;
pub use session::SessionLogger;
