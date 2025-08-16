//! Integration tests for path utilities
//!
//! This test suite ensures all path components work together correctly
//! in real-world scenarios typical of MAOS usage.

use maos_core::path::{PathValidator, normalize_path, paths_equal, relative_path};
use maos_core::{AgentType, SessionId};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_end_to_end_workspace_isolation() {
    // Create temporary directory structure for testing
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create some test directories
    let project_root = base_path.join("projects");
    let logs_dir = base_path.join("logs");
    fs::create_dir_all(&project_root).unwrap();
    fs::create_dir_all(&logs_dir).unwrap();

    // Generate workspaces for different agents
    let session1 = SessionId::generate();
    let session2 = SessionId::generate();
    let frontend_agent: AgentType = "frontend-engineer".to_string();
    let backend_agent: AgentType = "backend-engineer".to_string();

    // Use a temporary validator to generate workspace paths
    let temp_validator = PathValidator::new(vec![], vec![]);
    let workspace1 =
        temp_validator.generate_workspace_path(&project_root, &session1, &frontend_agent);
    let workspace2 =
        temp_validator.generate_workspace_path(&project_root, &session1, &backend_agent);
    let workspace3 =
        temp_validator.generate_workspace_path(&project_root, &session2, &frontend_agent);

    // Create the workspace directories for testing
    fs::create_dir_all(&workspace1).unwrap();
    fs::create_dir_all(&workspace2).unwrap();
    fs::create_dir_all(&workspace3).unwrap();

    // Set up validator with workspaces as allowed roots and realistic patterns
    let allowed_roots = vec![workspace1.clone(), workspace2.clone(), workspace3.clone()];
    let blocked_patterns = vec![
        "*.log".to_string(),
        "*.tmp".to_string(),
        "**/node_modules/**".to_string(),
        "**/.git/**".to_string(),
        "**/.ssh/**".to_string(),
    ];
    let validator = PathValidator::new(allowed_roots, blocked_patterns);

    // Verify workspace isolation
    assert_ne!(
        workspace1, workspace2,
        "Same session, different agents should have different workspaces"
    );
    assert_ne!(
        workspace1, workspace3,
        "Different sessions should have different workspaces"
    );
    assert_ne!(workspace2, workspace3, "All workspaces should be unique");

    // All workspaces should be within project root
    assert!(workspace1.starts_with(&project_root));
    assert!(workspace2.starts_with(&project_root));
    assert!(workspace3.starts_with(&project_root));

    // Test valid file operations within workspace
    let valid_files = vec![
        "src/main.rs",
        "config.toml",
        "data/input.json",
        "docs/README.md",
    ];

    for file_path in valid_files {
        let path = PathBuf::from(file_path);
        let result = validator.validate_workspace_path(&path, &workspace1);
        assert!(
            result.is_ok(),
            "Should allow valid file: {}, error: {:?}",
            file_path,
            result.err()
        );

        // Verify the resolved path is within workspace
        let canonical = result.unwrap();

        // Use our smart path comparison that handles macOS symlinks
        let canonical_workspace = if workspace1.exists() {
            workspace1
                .canonicalize()
                .unwrap_or_else(|_| workspace1.clone())
        } else {
            workspace1.clone()
        };

        assert!(
            canonical.starts_with(&canonical_workspace),
            "File should be within workspace: {file_path}, canonical: {canonical:?}, canonical_workspace: {canonical_workspace:?}"
        );
    }

    // Test blocked file patterns
    let blocked_files = vec![
        "debug.log",
        "temp.tmp",
        "node_modules/react/index.js",
        ".git/config",
        ".ssh/id_rsa",
    ];

    for file_path in blocked_files {
        let path = PathBuf::from(file_path);
        assert!(
            validator.is_blocked_path(&path),
            "Should block file: {file_path}"
        );
    }

    // Test security - path traversal attempts
    let attack_paths = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "%2e%2e/%2e%2e/etc/passwd",
        "safe.txt\0../../../etc/passwd",
        "file.txt\n../../../etc/passwd",
    ];

    for attack_path in attack_paths {
        let path = PathBuf::from(attack_path);
        let result = validator.validate_workspace_path(&path, &workspace1);
        assert!(result.is_err(), "Should block attack: {attack_path}");
    }
}

#[test]
fn test_path_utilities_integration() {
    // Test normalize_path and paths_equal work together
    let paths = vec![
        ("./src/main.rs", "src/main.rs"),
        ("src/../lib/mod.rs", "lib/mod.rs"),
        ("./dir1/./dir2/../file.txt", "dir1/file.txt"),
        ("dir\\subdir\\file.txt", "dir/subdir/file.txt"),
    ];

    for (input, expected) in paths {
        let input_path = PathBuf::from(input);
        let expected_path = PathBuf::from(expected);

        // Test normalization
        let normalized = normalize_path(&input_path);
        assert_eq!(
            normalized, expected_path,
            "Normalization failed for: {input}"
        );

        // Test path equality
        assert!(
            paths_equal(&input_path, &expected_path),
            "Paths should be equal: {input} vs {expected}"
        );

        // Test that normalized paths are equal to themselves
        assert!(
            paths_equal(&normalized, &expected_path),
            "Normalized path should equal expected"
        );
    }
}

#[test]
fn test_relative_path_with_workspace_scenarios() {
    // Test relative_path with typical workspace scenarios
    let scenarios = vec![
        // Agent wants to access file relative to project root
        (
            "/projects/session1_frontend",
            "/projects/session1_frontend/src/main.rs",
            Some("src/main.rs"),
        ),
        // Agent wants to reference shared config
        (
            "/projects/session1_frontend/src",
            "/projects/session1_frontend/config.toml",
            Some("../config.toml"),
        ),
        // Agent wants to access sibling directory
        (
            "/projects/session1_frontend/src",
            "/projects/session1_frontend/tests/unit.rs",
            Some("../tests/unit.rs"),
        ),
        // Same directory reference
        (
            "/projects/session1_frontend",
            "/projects/session1_frontend",
            Some("."),
        ),
    ];

    for (base_str, target_str, expected) in scenarios {
        let base = PathBuf::from(base_str);
        let target = PathBuf::from(target_str);

        let result = relative_path(&base, &target);

        match expected {
            Some(expected_str) => {
                assert!(
                    result.is_some(),
                    "Should find relative path from {base_str} to {target_str}"
                );
                let relative = result.unwrap();
                assert_eq!(
                    relative,
                    PathBuf::from(expected_str),
                    "Wrong relative path from {base_str} to {target_str}: got {relative:?}, expected {expected_str}"
                );
            }
            None => {
                assert!(
                    result.is_none(),
                    "Should not find relative path from {base_str} to {target_str}"
                );
            }
        }
    }
}

#[test]
fn test_cross_platform_compatibility() {
    // Test that our utilities handle cross-platform paths correctly
    let cross_platform_tests = vec![
        // Windows-style paths should be normalized to Unix-style
        ("src\\main.rs", "src/main.rs"),
        ("dir\\subdir\\file.txt", "dir/subdir/file.txt"),
        (".\\current\\file.txt", "current/file.txt"),
        ("..\\parent\\file.txt", "../parent/file.txt"),
        // Mixed separators should be normalized
        ("src/subdir\\file.txt", "src/subdir/file.txt"),
        ("dir\\sub/another\\file.txt", "dir/sub/another/file.txt"),
    ];

    for (input, expected) in cross_platform_tests {
        let input_path = PathBuf::from(input);
        let expected_path = PathBuf::from(expected);

        // Test normalization handles mixed separators
        let normalized = normalize_path(&input_path);
        assert_eq!(
            normalized, expected_path,
            "Cross-platform normalization failed: {input} -> expected {expected}, got {normalized:?}"
        );

        // Test path equality works across separator styles
        assert!(
            paths_equal(&input_path, &expected_path),
            "Cross-platform paths should be equal: {input} vs {expected}"
        );
    }
}

#[test]
fn test_unicode_path_handling() {
    // Test that our utilities handle Unicode paths correctly
    let unicode_tests = vec![
        ("测试/文件.txt", "测试/文件.txt"),
        ("./тест/файл.txt", "тест/файл.txt"),
        ("📁/📄.txt", "📁/📄.txt"),
        ("español/niño.txt", "español/niño.txt"),
    ];

    for (input, expected) in unicode_tests {
        let input_path = PathBuf::from(input);
        let expected_path = PathBuf::from(expected);

        let normalized = normalize_path(&input_path);
        assert_eq!(
            normalized, expected_path,
            "Unicode normalization failed: {input}"
        );

        assert!(
            paths_equal(&input_path, &expected_path),
            "Unicode paths should be equal: {input}"
        );
    }

    // Test that Unicode attacks are still blocked
    let temp_dir = TempDir::new().unwrap();
    let validator = PathValidator::new(vec![temp_dir.path().to_path_buf()], vec![]);

    let unicode_attacks = vec![
        "..\\u{FF0F}etc\\u{2044}passwd",
        "测试\\u{2215}..\\u{FF0F}system",
        "файл\\u{2044}..\\u{2215}root",
    ];

    for attack in unicode_attacks {
        let path = PathBuf::from(attack);
        let result = validator.validate_workspace_path(&path, temp_dir.path());
        assert!(result.is_err(), "Should block Unicode attack: {attack}");
    }
}

#[test]
fn test_performance_with_deep_nesting() {
    // Test that our utilities perform well with deeply nested paths
    let deep_path_components = [
        "level1", "level2", "level3", "level4", "level5", "level6", "level7", "level8", "level9",
        "level10",
    ];
    let deep_path = deep_path_components.join("/");
    let deep_traversal = format!("{}/{}", "../".repeat(10), "etc/passwd");

    let temp_dir = TempDir::new().unwrap();
    let validator = PathValidator::new(vec![temp_dir.path().to_path_buf()], vec![]);

    // Test normalization of deep paths
    let deep_normalized = normalize_path(&PathBuf::from(&deep_path));
    assert_eq!(
        deep_normalized,
        PathBuf::from(&deep_path),
        "Deep path normalization should work"
    );

    // Test validation of deep paths (should pass)
    let valid_deep = PathBuf::from(format!("{deep_path}/file.txt"));
    let result = validator.validate_workspace_path(&valid_deep, temp_dir.path());
    assert!(result.is_ok(), "Should allow deep valid paths");

    // Test validation rejects deep traversal attacks
    let traversal_path = PathBuf::from(deep_traversal);
    let result = validator.validate_workspace_path(&traversal_path, temp_dir.path());
    assert!(result.is_err(), "Should block deep traversal attacks");
}
