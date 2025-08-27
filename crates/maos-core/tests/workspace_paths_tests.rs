//! Integration tests for path utilities
//!
//! This test suite ensures all path components work together correctly
//! in real-world scenarios typical of MAOS usage.

use maos_core::path::{PathValidator, normalize_path, paths_equal, relative_path};
use maos_core::{AgentType, SessionId};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_workspace_isolation_logic() {
    // ✅ PROPER TEST: Tests workspace path generation logic, not OS file system
    let session1 = SessionId::generate();
    let session2 = SessionId::generate();
    let frontend_agent: AgentType = "frontend-engineer".to_string();
    let backend_agent: AgentType = "backend-engineer".to_string();

    let validator = PathValidator::new(vec![], vec![]);
    let project_root = PathBuf::from("/mock/projects");

    // Test workspace path generation produces unique paths
    let workspace1 = validator.generate_workspace_path(&project_root, &session1, &frontend_agent);
    let workspace2 = validator.generate_workspace_path(&project_root, &session1, &backend_agent);
    let workspace3 = validator.generate_workspace_path(&project_root, &session2, &frontend_agent);

    // Verify workspace isolation logic
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

    // Test path validation logic with mock workspace
    let allowed_roots = vec![workspace1.clone()];
    let blocked_patterns = vec![
        "*.log".to_string(),
        "*.tmp".to_string(),
        "**/node_modules/**".to_string(),
        "**/.git/**".to_string(),
        "**/.ssh/**".to_string(),
    ];
    let validator = PathValidator::new(allowed_roots, blocked_patterns);

    // Test valid files (should pass validation logic)
    let valid_files = vec![
        "src/main.rs",
        "config.toml",
        "data/input.json",
        "docs/README.md",
    ];
    for file_path in valid_files {
        let path = PathBuf::from(file_path);
        // This tests our validation logic, not file system operations
        assert!(
            !validator.is_blocked_path(&path),
            "Should not block valid file: {file_path}"
        );
    }

    // Test blocked file patterns (should fail validation logic)
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

    // Test security - path traversal detection logic
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
    let mut paths = vec![
        ("./src/main.rs", "src/main.rs"),
        ("src/../lib/mod.rs", "lib/mod.rs"),
        ("./dir1/./dir2/../file.txt", "dir1/file.txt"),
    ];

    // Platform-specific: backslashes are path separators on Windows, literal chars on Unix
    #[cfg(windows)]
    paths.push(("dir\\subdir\\file.txt", "dir\\subdir\\file.txt"));
    #[cfg(not(windows))]
    paths.push(("dir/subdir/file.txt", "dir/subdir/file.txt")); // Use forward slashes on Unix

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
    // Platform behavior differs: Windows treats backslashes as separators, Unix doesn't
    #[cfg(windows)]
    let cross_platform_tests = vec![
        // On Windows, backslashes are path separators
        ("src\\main.rs", "src\\main.rs"),
        ("dir\\subdir\\file.txt", "dir\\subdir\\file.txt"),
        (".\\current\\file.txt", "current\\file.txt"), // . is removed
        ("..\\parent\\file.txt", "..\\parent\\file.txt"), // .. preserved
        // Mixed separators work on Windows
        ("src/subdir\\file.txt", "src/subdir\\file.txt"),
        ("dir\\sub/another\\file.txt", "dir\\sub/another\\file.txt"),
    ];

    #[cfg(not(windows))]
    let cross_platform_tests = vec![
        // On Unix, only forward slashes are separators
        ("src/main.rs", "src/main.rs"),
        ("dir/subdir/file.txt", "dir/subdir/file.txt"),
        ("./current/file.txt", "current/file.txt"), // . is removed
        ("../parent/file.txt", "../parent/file.txt"), // .. preserved
        // Backslashes are just filename characters on Unix
        ("file\\with\\backslashes.txt", "file\\with\\backslashes.txt"),
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
    // ✅ PROPER TEST: Tests Unicode path normalization logic, not OS file system
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

    // Test that Unicode attacks are blocked by validation logic
    let mock_workspace = PathBuf::from("/mock/workspace");
    let validator = PathValidator::new(vec![mock_workspace.clone()], vec![]);

    let unicode_attacks = vec![
        "..\\u{FF0F}etc\\u{2044}passwd",
        "测试\\u{2215}..\\u{FF0F}system",
        "файл\\u{2044}..\\u{2215}root",
    ];

    for attack in unicode_attacks {
        let path = PathBuf::from(attack);
        let result = validator.validate_workspace_path(&path, &mock_workspace);
        assert!(result.is_err(), "Should block Unicode attack: {attack}");
    }
}

#[test]
fn test_deep_path_logic_correctness() {
    // ✅ PROPER TEST: Tests deep path handling logic, not OS performance
    let deep_path_components = [
        "level1", "level2", "level3", "level4", "level5", "level6", "level7", "level8", "level9",
        "level10",
    ];
    let deep_path = deep_path_components.join("/");
    let deep_traversal = format!("{}/{}", "../".repeat(10), "etc/passwd");

    let mock_workspace = PathBuf::from("/mock/workspace");
    let validator = PathValidator::new(vec![mock_workspace.clone()], vec![]);

    // Test normalization logic with deep paths
    let deep_normalized = normalize_path(&PathBuf::from(&deep_path));
    assert_eq!(
        deep_normalized,
        PathBuf::from(&deep_path),
        "Deep path normalization should work"
    );

    // Test validation logic correctly handles deep paths
    let valid_deep = PathBuf::from(format!("{deep_path}/file.txt"));
    // Test our validation logic can handle deep valid paths
    assert!(
        !validator.is_blocked_path(&valid_deep),
        "Should allow deep valid paths in logic"
    );

    // Test validation logic correctly rejects deep traversal attacks
    let traversal_path = PathBuf::from(deep_traversal);
    let result = validator.validate_workspace_path(&traversal_path, &mock_workspace);
    assert!(result.is_err(), "Should block deep traversal attacks");
}

#[test]
#[cfg(unix)] // This test is Unix-specific due to symlink creation
fn test_symlink_escape_prevention() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().to_path_buf();
    let symlink_path = workspace_path.join("escape_link");

    use std::os::unix::fs::symlink;
    // Try to create symlink to parent directory
    let _ = symlink("../../etc", &symlink_path);

    // PathValidator should detect this when resolving paths
    let allowed_roots = vec![workspace_path.clone()];
    let validator = PathValidator::new(allowed_roots, vec![]);

    // Symlink that escapes workspace should be rejected
    let result = validator.validate_workspace_path(&symlink_path, &workspace_path);
    // If symlink was created and points outside workspace, it should be rejected
    if symlink_path.exists() {
        assert!(
            result.is_err(),
            "Symlink escaping workspace should be rejected"
        );
    }
}
