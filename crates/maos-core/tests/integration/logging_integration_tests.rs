use maos_core::SessionId;
use maos_core::logging::{
    LogFormat, LogLevel, LogOutput, LoggingConfig, RollingLogConfig, SessionLogger, init_logging,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_session_logger_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();
    let config = RollingLogConfig::default();

    let logger = SessionLogger::new(session_id.clone(), log_dir.clone(), config)
        .expect("Failed to create session logger");

    // Verify log directory was created
    assert!(log_dir.exists());
    assert!(log_dir.is_dir());

    // Verify logger is initialized with correct session
    assert_eq!(logger.session_id(), &session_id);
}

#[test]
fn test_session_logger_write() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();
    let config = RollingLogConfig::default();

    let mut logger = SessionLogger::new(session_id.clone(), log_dir.clone(), config)
        .expect("Failed to create session logger");

    // Write some log entries
    logger.write("First log entry").expect("Failed to write");
    logger.write("Second log entry").expect("Failed to write");

    // Verify log file exists and contains entries
    let log_file = log_dir.join(format!("session-{}.log", session_id.as_str()));
    assert!(log_file.exists());

    let contents = fs::read_to_string(&log_file).expect("Failed to read log file");
    assert!(contents.contains("First log entry"));
    assert!(contents.contains("Second log entry"));
}

#[test]
fn test_log_rotation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();

    // Create config with small max size to trigger rotation
    let config = RollingLogConfig {
        max_file_size_bytes: 100, // Very small to trigger rotation
        max_files_per_session: 3,
        compress_on_roll: false, // Disable compression for simplicity
        file_pattern: "session-{session_id}.log".to_string(),
    };

    // Store pattern before moving config
    let pattern = config
        .file_pattern
        .replace("{session_id}", session_id.as_str());

    let mut logger = SessionLogger::new(session_id.clone(), log_dir.clone(), config)
        .expect("Failed to create session logger");

    // Write enough data to trigger rotation
    let long_entry = "x".repeat(60);
    logger.write(&long_entry).expect("Failed to write");
    logger.write(&long_entry).expect("Failed to write"); // Should trigger rotation

    // Verify rotation happened using the same logic as production code
    let base_path = log_dir.join(pattern);
    let rotated_file = base_path.with_extension("log.1");
    assert!(rotated_file.exists(), "Rotated file should exist");
}

#[test]
fn test_concurrent_logging() {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();
    let config = RollingLogConfig::default();

    let logger = Arc::new(
        SessionLogger::new(session_id.clone(), log_dir.clone(), config)
            .expect("Failed to create session logger")
            .into_thread_safe(),
    );

    // Spawn multiple threads writing concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let logger = Arc::clone(&logger);
            thread::spawn(move || {
                for j in 0..10 {
                    logger
                        .write(&format!("Thread {i} entry {j}"))
                        .expect("Failed to write");
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify all entries were written
    let log_file = log_dir.join(format!("session-{}.log", session_id.as_str()));
    let contents = fs::read_to_string(&log_file).expect("Failed to read log file");

    // Should have 100 total entries (10 threads * 10 entries each)
    let line_count = contents.lines().count();
    assert_eq!(line_count, 100, "Should have exactly 100 log entries");
}

#[test]
fn test_init_logging() {
    let config = LoggingConfig {
        level: LogLevel::Info,
        format: LogFormat::Plain,
        output: LogOutput::Stdout,
        enable_performance_logs: false,
        enable_security_logs: false,
        rolling: RollingLogConfig::default(),
    };

    // Initialize logging (should not panic)
    init_logging(&config).expect("Failed to initialize logging");

    // Test that we can log after initialization
    tracing::info!("Test log message");
    tracing::debug!("This should be filtered out"); // Below Info level
}

#[test]
fn test_log_compression() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();

    let config = RollingLogConfig {
        max_file_size_bytes: 100,
        max_files_per_session: 3,
        compress_on_roll: true, // Enable compression
        file_pattern: "session-{session_id}.log".to_string(),
    };

    let mut logger = SessionLogger::new(session_id.clone(), log_dir.clone(), config)
        .expect("Failed to create session logger");

    // Write data to trigger rotation with compression
    let long_entry = "x".repeat(60);
    logger.write(&long_entry).expect("Failed to write");
    logger.write(&long_entry).expect("Failed to write");

    // Verify compressed file exists
    let compressed_file = log_dir.join(format!("session-{}.log.1.gz", session_id.as_str()));
    assert!(compressed_file.exists(), "Compressed file should exist");
}

#[test]
fn test_max_files_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();

    let config = RollingLogConfig {
        max_file_size_bytes: 50,
        max_files_per_session: 2, // Only keep 2 rotated files
        compress_on_roll: false,
        file_pattern: "session-{session_id}.log".to_string(),
    };

    let mut logger = SessionLogger::new(session_id.clone(), log_dir.clone(), config)
        .expect("Failed to create session logger");

    // Write enough to create multiple rotations
    for i in 0..5 {
        let entry = format!("Entry {}: {}", i, "x".repeat(40));
        logger.write(&entry).expect("Failed to write");
    }

    // Should have current log + 2 rotated files maximum
    let log_files: Vec<_> = fs::read_dir(&log_dir)
        .expect("Failed to read dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|s| s.starts_with(&format!("session-{}", session_id.as_str())))
                .unwrap_or(false)
        })
        .collect();

    assert!(log_files.len() <= 3, "Should not exceed max files limit");
}
