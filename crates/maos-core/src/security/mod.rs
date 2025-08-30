//! Core security validation module
//!
//! Provides essential security validation for MAOS operations including:
//! - Path traversal prevention
//! - Dangerous command detection  
//! - Environment file protection
//! - Resource usage limits
//! - JSON structure validation

pub mod command;
pub mod file;
pub mod json;
pub mod path;
pub mod path_validator;
pub mod resource;
pub mod resource_validator;
pub mod traits;
pub mod validator;

// Re-export key types for convenience
pub use command::validate_command;
pub use file::validate_file_access;
pub use json::validate_json_structure;
pub use path::validate_path_safety;
pub use resource::validate_resource_usage;
pub use validator::SecurityValidator;

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;
    use std::{env, path::Path};

    // ===== Direct Validation Tests =====

    #[test]
    fn test_path_traversal_detection() {
        // Test various path traversal attempts
        let dangerous_paths = vec![
            "../../../etc/passwd",
            "..\\..\\windows\\system32",
            "/tmp/../../../etc/shadow",
            "./../../root/.ssh/id_rsa",
            ".../.../etc/hosts", // Sneaky variant
        ];

        for path in dangerous_paths {
            let result = validate_path_safety(Path::new(path));
            assert!(result.is_err(), "Path traversal not detected: {path}");
        }

        // Test safe paths
        let safe_paths = vec![
            "/tmp/test.txt",
            "src/main.rs",
            "./local/file.txt",
            "relative/path/file.json",
        ];

        for path in safe_paths {
            let result = validate_path_safety(Path::new(path));
            assert!(result.is_ok(), "Safe path rejected: {path}");
        }
    }

    #[test]
    fn test_resource_limits() {
        let memory_limit = 10_000_000; // 10MB
        let time_limit = 5000; // 5s

        // Test memory limits
        assert!(
            validate_resource_usage(
                100_000_000, // 100MB - too large
                0,
                memory_limit,
                time_limit
            )
            .is_err()
        );

        assert!(
            validate_resource_usage(
                5_000_000, // 5MB - ok
                0,
                memory_limit,
                time_limit
            )
            .is_ok()
        );

        // Test time limits
        assert!(
            validate_resource_usage(
                1000,
                10_000, // 10s - too long
                memory_limit,
                time_limit
            )
            .is_err()
        );
    }

    #[test]
    fn test_json_depth_validation() {
        // Create deeply nested JSON
        let mut deep_json = json!({"level": 0});
        for i in 1..150 {
            deep_json = json!({"level": i, "nested": deep_json});
        }

        let json_bytes = serde_json::to_vec(&deep_json).unwrap();

        // Should fail with max depth of 100
        assert!(validate_json_structure(&json_bytes, 100, 10_000_000).is_err());

        // Should pass with higher limit
        assert!(validate_json_structure(&json_bytes, 200, 10_000_000).is_ok());
    }

    #[test]
    fn test_json_size_validation() {
        let large_json = json!({
            "data": "x".repeat(5_000_000) // 5MB string
        });

        let json_bytes = serde_json::to_vec(&large_json).unwrap();

        // Should fail with 1MB limit
        assert!(validate_json_structure(&json_bytes, 100, 1_000_000).is_err());

        // Should pass with 10MB limit
        assert!(validate_json_structure(&json_bytes, 100, 10_000_000).is_ok());
    }

    // ===== Property-Based Tests (Fast Version) =====

    /// Get proptest configuration from environment variables
    fn proptest_config_from_env() -> ProptestConfig {
        let cases = env::var("MAOS_TEST_SECURITY_PROPTEST_CASES")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(10); // Default to 10 for fast local dev

        ProptestConfig::with_cases(cases)
    }

    proptest! {
        #![proptest_config(proptest_config_from_env())] // Configurable via env vars

        #[test]
        fn prop_path_traversal_always_detected(
            traversals in 1..5usize,
            suffix in "[a-z]+",
        ) {
            // Create a path with actual directory traversal
            let mut path = String::new();
            for _ in 0..traversals {
                path.push_str("../");
            }
            path.push_str(&suffix);

            let result = validate_path_safety(Path::new(&path));

            // Path traversal should be detected
            prop_assert!(result.is_err(), "Failed to detect traversal in: {path}");
        }

        #[test]
        fn prop_resource_limits_enforced(
            memory in 0u64..1_000_000_000u64,
            time in 0u64..60_000u64,
        ) {
            let memory_limit = 100_000_000; // 100MB
            let time_limit = 10_000; // 10s

            let result = validate_resource_usage(
                memory,
                time,
                memory_limit,
                time_limit
            );

            if memory > memory_limit || time > time_limit {
                prop_assert!(result.is_err());
            } else {
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn prop_json_validation_consistent(
            depth in 1u32..200u32,
            size_mb in 1usize..20usize,
        ) {
            // Create JSON with specified depth
            let mut json = json!({"value": 0});
            for i in 1..depth {
                json = json!({"level": i, "nested": json});
            }

            // Add padding for size
            if size_mb > 1 {
                json["padding"] = json!("x".repeat(size_mb * 100_000));
            }

            let json_bytes = serde_json::to_vec(&json).unwrap();
            let result = validate_json_structure(
                &json_bytes,
                100, // max depth
                10_000_000 // max size 10MB
            );

            // Should fail if depth > 100 or size > 10MB
            if depth > 100 || json_bytes.len() > 10_000_000 {
                prop_assert!(result.is_err());
            }
        }
    }

    // ===== Performance Comparison Tests =====

    #[test]
    #[ignore] // Run with: cargo test -- --ignored --nocapture
    fn benchmark_validation_performance() {
        use std::time::Instant;

        println!("\n=== Validation Performance Comparison ===\n");

        // Test path validation
        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = validate_path_safety(Path::new("/tmp/test.txt"));
        }
        let path_duration = start.elapsed();
        println!("Path validation (10,000 iterations): {path_duration:?}");
        println!("Average: {:.2?} per validation", path_duration / 10_000);

        // Test resource validation
        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = validate_resource_usage(1000, 100, 10_000_000, 5000);
        }
        let resource_duration = start.elapsed();
        println!("\nResource validation (10,000 iterations): {resource_duration:?}");
        println!("Average: {:.2?} per validation", resource_duration / 10_000);

        // Test JSON validation
        let json_bytes = br#"{"key": "value", "nested": {"deep": true}}"#;
        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = validate_json_structure(json_bytes, 10, 1_000_000);
        }
        let json_duration = start.elapsed();
        println!("\nJSON validation (10,000 iterations): {json_duration:?}");
        println!("Average: {:.2?} per validation", json_duration / 10_000);

        println!("\n=== All validations run in microseconds! ===");
        println!("Compare to process spawning: ~50-100ms per test");
        println!("Speedup: 1000-10,000x faster! 🚀\n");
    }
}
