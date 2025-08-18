use maos_core::constants;
use std::time::Duration;

#[test]
fn test_directory_constants() {
    // Test configuration directory constants
    assert_eq!(constants::MAOS_ROOT_DIR, ".maos");
    assert_eq!(constants::CONFIG_FILE_NAME, "config.json");
    assert_eq!(constants::SESSIONS_DIR_NAME, "sessions");
    assert_eq!(constants::WORKSPACES_DIR_NAME, "workspaces");
    assert_eq!(constants::LOGS_DIR_NAME, "logs");
}

#[test]
fn test_performance_target_constants() {
    // Test performance targets are reasonable
    assert_eq!(constants::MAX_EXECUTION_TIME_MS, 10);
    assert_eq!(constants::MAX_MEMORY_USAGE_MB, 5);
    assert_eq!(constants::MAX_BINARY_SIZE_MB, 10);

    // These are compile-time constants - no need for runtime assertions
}

#[test]
fn test_timeout_constants() {
    // Test timeout values are Duration types
    assert_eq!(
        constants::DEFAULT_OPERATION_TIMEOUT,
        Duration::from_millis(5000)
    );
    assert_eq!(constants::FILE_LOCK_TIMEOUT, Duration::from_millis(1000));
    assert_eq!(constants::TTS_TIMEOUT, Duration::from_millis(10000));

    // Verify relationships between timeouts
    assert!(constants::FILE_LOCK_TIMEOUT < constants::DEFAULT_OPERATION_TIMEOUT);
    assert!(constants::DEFAULT_OPERATION_TIMEOUT < constants::TTS_TIMEOUT);
}

#[test]
fn test_file_naming_patterns() {
    // Test file naming constants
    assert_eq!(constants::SESSION_FILE_NAME, "session.json");
    assert_eq!(constants::AGENTS_FILE_NAME, "agents.json");
    assert_eq!(constants::LOCKS_FILE_NAME, "locks.json");
    assert_eq!(constants::PROGRESS_FILE_NAME, "progress.json");
    assert_eq!(constants::TIMELINE_FILE_NAME, "timeline.json");
    assert_eq!(constants::METRICS_FILE_NAME, "metrics.json");

    // All should end with .json
    assert!(constants::SESSION_FILE_NAME.ends_with(".json"));
    assert!(constants::AGENTS_FILE_NAME.ends_with(".json"));
    assert!(constants::LOCKS_FILE_NAME.ends_with(".json"));
}

#[test]
fn test_log_file_patterns() {
    // Test log file configuration constants
    assert_eq!(constants::LOG_FILE_PATTERN, "session-{session_id}.log");
    assert_eq!(constants::MAX_LOG_FILE_SIZE, 10 * 1024 * 1024); // 10MB
    assert_eq!(constants::MAX_LOG_FILES_PER_SESSION, 10);

    // Verify pattern contains placeholder
    assert!(constants::LOG_FILE_PATTERN.contains("{session_id}"));

    // Sizes are compile-time constants - verify they're set correctly
}

#[test]
fn test_constants_are_immutable() {
    // This test verifies constants are actually const
    // These should all be compile-time constants
    const _CONFIG_DIR: &str = constants::MAOS_ROOT_DIR;
    const _SESSION_FILE: &str = constants::SESSION_FILE_NAME;
    const _MAX_TIME: u64 = constants::MAX_EXECUTION_TIME_MS;

    // Duration constants should also be const
    const _TIMEOUT: Duration = constants::DEFAULT_OPERATION_TIMEOUT;
}

#[test]
fn test_constants_usage_in_format() {
    // Test that patterns can be used in formatting
    let session_id = "test-123";
    let formatted = constants::LOG_FILE_PATTERN.replace("{session_id}", session_id);
    assert_eq!(formatted, "session-test-123.log");
    assert!(formatted.ends_with(".log"));
}
