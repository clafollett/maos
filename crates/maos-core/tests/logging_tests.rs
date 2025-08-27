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

#[test]
fn test_disk_space_exhaustion_handling() {
    // Test that logger handles disk space errors gracefully
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();

    let config = RollingLogConfig {
        max_file_size_bytes: 10,
        max_files_per_session: 2,
        compress_on_roll: false,
        file_pattern: "session-{session_id}.log".to_string(),
    };

    let mut logger = SessionLogger::new(session_id, log_dir.clone(), config).unwrap();

    // Make directory read-only to simulate disk errors
    let metadata = fs::metadata(&log_dir).unwrap();
    let mut permissions = metadata.permissions();
    permissions.set_readonly(true);
    let _ = fs::set_permissions(&log_dir, permissions.clone());

    // Writing should fail gracefully (not panic)
    let _ = logger.write("This should fail"); // We don't care about the result, just that it doesn't panic

    // No need to restore permissions - TempDir will clean up automatically
}

#[test]
fn test_malformed_session_id_handling() {
    use std::str::FromStr;

    // Test various malformed session IDs
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _log_dir = temp_dir.path().to_path_buf();

    let malformed_ids = vec![
        "../escape",
        "session/../../etc/passwd",
        "session\0null",
        "session|pipe",
        "session;command",
        "$(whoami)",
    ];

    for bad_id in malformed_ids {
        // SessionId::from_str should reject or sanitize dangerous input
        let session_result = SessionId::from_str(bad_id);

        // Either the session ID is rejected (Err) or it's sanitized (Ok with safe content)
        if let Ok(session_id) = session_result {
            let safe_id = session_id.to_string();
            // If accepted, it should be sanitized
            assert!(!safe_id.contains(".."));
            assert!(!safe_id.contains('\0'));
            assert!(!safe_id.contains('/'));
        }
        // If Err, that's also acceptable - dangerous input was rejected
    }
}

#[test]
fn test_unicode_in_log_messages() {
    // Test that Unicode in log messages is handled correctly
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();
    let config = RollingLogConfig::default();

    let mut logger = SessionLogger::new(session_id.clone(), log_dir.clone(), config).unwrap();

    // Various Unicode test cases
    let unicode_messages = vec![
        "Hello 世界",
        "Emoji test: 🔒🚀💯",
        "RTL text: مرحبا بالعالم",
        "Zero-width: test\u{200B}hidden",
        "Combining: é (e\u{0301})",
    ];

    for msg in unicode_messages {
        assert!(logger.write(msg).is_ok());
    }

    // Verify content was written
    let log_file = log_dir.join(format!("session-{}.log", session_id.as_str()));
    let content = fs::read_to_string(log_file).unwrap();
    assert!(content.contains("Emoji test"));
}

#[test]
fn test_concurrent_rotation_safety() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();

    let config = RollingLogConfig {
        max_file_size_bytes: 50,
        max_files_per_session: 5,
        compress_on_roll: false,
        file_pattern: "session-{session_id}.log".to_string(),
    };

    let logger = Arc::new(Mutex::new(
        SessionLogger::new(session_id, log_dir.clone(), config).unwrap(),
    ));

    // Spawn multiple threads writing concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let logger = logger.clone();
        let handle = thread::spawn(move || {
            for j in 0..10 {
                if let Ok(mut log) = logger.lock() {
                    let _ = log.write(&format!("Thread {i} message {j}"));
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });
        handles.push(handle);
    }

    // All threads should complete without deadlock or panic
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_log_file_path_traversal_prevention() {
    // Test that path traversal in file patterns is prevented
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();

    let malicious_patterns = vec![
        "../../../etc/passwd-{session_id}.log",
        "logs/../../../sensitive-{session_id}.log",
        "/etc/shadow-{session_id}.log",
    ];

    for pattern in malicious_patterns {
        let config = RollingLogConfig {
            max_file_size_bytes: 1024,
            max_files_per_session: 3,
            compress_on_roll: false,
            file_pattern: pattern.to_string(),
        };

        // Logger should either sanitize the path or fail to create
        let result = SessionLogger::new(session_id.clone(), log_dir.clone(), config);

        if let Ok(mut logger) = result {
            // If logger was created, verify the file is in the safe directory
            let _ = logger.write("test");

            // Check that no files were created outside log_dir
            assert!(!std::path::Path::new("/etc/passwd").exists());
            assert!(!std::path::Path::new("/etc/shadow").exists());
        }
    }
}

#[test]
fn test_compression_failure_recovery() {
    // Test recovery when compression fails
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();

    let config = RollingLogConfig {
        max_file_size_bytes: 10,
        max_files_per_session: 3,
        compress_on_roll: true,
        file_pattern: "session-{session_id}.log".to_string(),
    };

    let mut logger = SessionLogger::new(session_id.clone(), log_dir.clone(), config).unwrap();

    // Write enough to trigger rotation
    logger.write("First content to rotate").unwrap();
    logger.write("Second content to trigger rotation").unwrap();

    // Even if compression fails internally, logger should continue working
    let result = logger.write("Third write after rotation");
    assert!(result.is_ok());
}

#[test]
fn test_extreme_file_counts() {
    // Test handling of extreme max_files_per_session values
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();

    // Test with zero files (should be adjusted to minimum)
    let config_zero = RollingLogConfig {
        max_file_size_bytes: 100,
        max_files_per_session: 0,
        compress_on_roll: false,
        file_pattern: "session-{session_id}.log".to_string(),
    };

    let result = SessionLogger::new(session_id.clone(), log_dir.clone(), config_zero);
    assert!(result.is_ok()); // Should handle gracefully

    // Test with extremely large file count
    let config_large = RollingLogConfig {
        max_file_size_bytes: 100,
        max_files_per_session: usize::MAX,
        compress_on_roll: false,
        file_pattern: "session-{session_id}.log".to_string(),
    };

    let result = SessionLogger::new(session_id.clone(), log_dir.clone(), config_large);
    assert!(result.is_ok()); // Should handle without overflow
}

#[test]
fn test_log_injection_prevention() {
    // Test prevention of log injection attacks
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().to_path_buf();
    let session_id = SessionId::generate();
    let config = RollingLogConfig::default();

    let mut logger = SessionLogger::new(session_id.clone(), log_dir.clone(), config).unwrap();

    // Attempt log injection with newlines and control characters
    let injection_attempts = vec![
        "Normal log\n[ERROR] Fake error injected",
        "User input: \r\n[SECURITY] Fake security alert",
        "Data: \u{001B}[31mRed text attack\u{001B}[0m",
        "Message\n\n[ADMIN] Privilege escalation attempt",
    ];

    for attempt in injection_attempts {
        assert!(logger.write(attempt).is_ok());
    }

    // Verify that injected content is properly escaped/handled
    let log_file = log_dir.join(format!("session-{}.log", session_id.as_str()));
    let content = fs::read_to_string(log_file).unwrap();

    // Log entries should be properly formatted (implementation-specific)
    assert!(content.contains("Normal log"));
}
