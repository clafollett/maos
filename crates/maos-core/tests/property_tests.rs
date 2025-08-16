//! Property-based tests for path utilities
//!
//! These tests use proptest to generate thousands of random inputs and verify
//! that our path utilities maintain their invariants under all conditions.
//! This complements our unit tests by exploring the input space exhaustively.

use proptest::{
    collection, option, prop_assert, prop_assert_eq, prop_assert_ne, prop_assume, prop_oneof,
    proptest, strategy::Strategy,
};
use std::path::PathBuf;

use maos_core::path::{PathValidator, normalize_path, paths_equal, relative_path};
use maos_core::{AgentType, SessionId};

/// Generate arbitrary valid path strings for testing
fn arb_path_string() -> impl Strategy<Value = String> {
    collection::vec(
        prop_oneof![
            // Normal path components (start with alphanumeric to avoid problematic patterns)
            "[a-zA-Z0-9][a-zA-Z0-9_-]{0,19}",
            // Special cases that should be handled (but not as path components)
            "\\.",
            "\\.\\.",
            // Unicode components (safe ones)
            "[测试文件]{1,10}",
        ],
        1..10,
    )
    .prop_map(|components| components.join("/"))
}

/// Generate potentially malicious path strings for security testing
fn arb_malicious_path_string() -> impl Strategy<Value = String> {
    prop_oneof![
        // Path traversal patterns
        "(\\.\\./)+(etc|root|home|usr)/[a-z]+",
        // Mixed separators
        "[a-zA-Z0-9]+\\\\[a-zA-Z0-9]+/[a-zA-Z0-9]+",
        // URL encoded attacks
        "(%2e%2e/)+etc/passwd",
        // Control character injection
        "[a-zA-Z]+[\\x00-\\x1F][a-zA-Z]+",
        // Unicode attacks
        "[a-zA-Z]+\\u{FF0F}[a-zA-Z]+",
        // Double encoding
        "(%252e%252e/)+etc/passwd",
    ]
}

/// Generate valid agent type strings
fn arb_agent_type() -> impl Strategy<Value = String> {
    prop_oneof![
        "frontend-engineer",
        "backend-engineer",
        "data-scientist",
        "devops-engineer",
        "[a-z-]{5,20}",
        "[a-zA-Z0-9_-]{3,15}",
    ]
    .prop_map(|s| s.to_string())
}

#[cfg(test)]
mod normalize_path_properties {
    use super::*;

    proptest! {
        /// Property: Normalizing a path twice should yield the same result
        #[test]
        fn normalize_path_idempotent(path_str in arb_path_string()) {
            let path = PathBuf::from(&path_str);
            let normalized1 = normalize_path(&path);
            let normalized2 = normalize_path(&normalized1);

            let norm1_clone = normalized1.clone();
            let norm2_clone = normalized2.clone();
            prop_assert_eq!(normalized1, normalized2,
                "Normalization should be idempotent: {} -> {:?} -> {:?}",
                path_str, norm1_clone, norm2_clone);
        }

        /// Property: Normalized paths should not contain unnecessary dot components
        #[test]
        fn normalize_path_removes_dot_components(path_str in "[a-zA-Z0-9./]{1,50}") {
            let path = PathBuf::from(&path_str);
            let normalized = normalize_path(&path);
            let normalized_str = normalized.to_string_lossy();

            // Should not contain standalone dots (except at the start for relative paths)
            let has_internal_dot = normalized_str.contains("/./") ||
                                 normalized_str.ends_with("/.");

            prop_assert!(!has_internal_dot,
                "Normalized path should not contain internal dot components: {} -> {}",
                path_str, normalized_str);
        }

        /// Property: Absolute paths should remain absolute after normalization
        #[test]
        fn normalize_path_preserves_absolute(path_str in "/[a-zA-Z0-9./]{1,50}") {
            let path = PathBuf::from(&path_str);
            let normalized = normalize_path(&path);

            prop_assert!(normalized.is_absolute(),
                "Absolute paths should remain absolute: {} -> {:?}",
                path_str, normalized);
        }

        /// Property: Relative paths should remain relative (unless they go above root)
        #[test]
        fn normalize_path_preserves_relative(path_str in "[a-zA-Z0-9./]{1,30}") {
            // Filter out paths that start with / to keep them relative
            prop_assume!(!path_str.starts_with('/'));

            let path = PathBuf::from(&path_str);
            let normalized = normalize_path(&path);

            // Relative paths should stay relative unless they've been resolved away
            if !normalized.as_os_str().is_empty() {
                prop_assert!(!normalized.is_absolute(),
                    "Relative paths should remain relative: {} -> {:?}",
                    path_str, normalized);
            }
        }

        /// Property: Cross-platform separator handling
        #[test]
        fn normalize_path_handles_separators(path_str in "[a-zA-Z0-9\\\\./]{1,40}") {
            let path = PathBuf::from(&path_str);
            let normalized = normalize_path(&path);
            let normalized_str = normalized.to_string_lossy();

            const BACKSLASHES: &str = r"\\";

            // On Unix, backslashes should be treated as regular characters, not separators
            // But our normalize_path should handle them as separators for cross-platform support
            if path_str.contains(BACKSLASHES) && !path_str.contains("..") {
                prop_assert!(
                    normalized_str.contains("/") || !normalized_str.contains(BACKSLASHES),
                    "Mixed separators should be normalized: {} -> {}",
                    path_str, normalized_str);
            }
        }
    }
}

#[cfg(test)]
mod paths_equal_properties {
    use super::*;

    proptest! {
        /// Property: paths_equal should be reflexive (a path equals itself)
        #[test]
        fn paths_equal_reflexive(path_str in arb_path_string()) {
            let path = PathBuf::from(&path_str);
            prop_assert!(paths_equal(&path, &path),
                "Path should equal itself: {}", path_str);
        }

        /// Property: paths_equal should be symmetric
        #[test]
        fn paths_equal_symmetric(path1_str in arb_path_string(), path2_str in arb_path_string()) {
            let path1 = PathBuf::from(&path1_str);
            let path2 = PathBuf::from(&path2_str);

            let equal_12 = paths_equal(&path1, &path2);
            let equal_21 = paths_equal(&path2, &path1);

            prop_assert_eq!(equal_12, equal_21,
                "paths_equal should be symmetric: {} vs {} -> {} vs {}",
                path1_str, path2_str, equal_12, equal_21);
        }

        /// Property: paths_equal should be transitive
        #[test]
        fn paths_equal_transitive(
            path1_str in arb_path_string(),
            path2_str in arb_path_string(),
            path3_str in arb_path_string()
        ) {
            let path1 = PathBuf::from(&path1_str);
            let path2 = PathBuf::from(&path2_str);
            let path3 = PathBuf::from(&path3_str);

            if paths_equal(&path1, &path2) && paths_equal(&path2, &path3) {
                prop_assert!(paths_equal(&path1, &path3),
                    "paths_equal should be transitive: {} = {} = {} implies {} = {}",
                    path1_str, path2_str, path3_str, path1_str, path3_str);
            }
        }

        /// Property: Normalized versions of the same path should be equal
        #[test]
        fn paths_equal_after_normalization(path_str in arb_path_string()) {
            let original = PathBuf::from(&path_str);
            let normalized = normalize_path(&original);

            prop_assert!(paths_equal(&original, &normalized),
                "Original and normalized paths should be equal: {} vs {:?}",
                path_str, normalized);
        }

        /// Property: Equivalent traversal patterns should be equal
        #[test]
        fn paths_equal_equivalent_traversals(base in "[a-zA-Z]{1,10}", file in "[a-zA-Z]{1,10}") {
            let path1 = PathBuf::from(format!("{base}/{file}"));
            let path2 = PathBuf::from(format!("{base}/./{file}"));
            let path3 = PathBuf::from(format!("{base}/sub/../{file}"));

            prop_assert!(paths_equal(&path1, &path2),
                "Equivalent paths should be equal: {:?} vs {:?}", path1, path2);
            prop_assert!(paths_equal(&path1, &path3),
                "Equivalent paths should be equal: {:?} vs {:?}", path1, path3);
        }
    }
}

#[cfg(test)]
mod relative_path_properties {
    use super::*;

    proptest! {
        /// Property: relative_path from a path to itself should return "."
        #[test]
        fn relative_path_to_self(path_str in arb_path_string()) {
            let path = PathBuf::from(&path_str);
            let normalized = normalize_path(&path);

            match relative_path(&normalized, &normalized) {
                Some(rel) => {
                    let rel_clone = rel.clone();
                    prop_assert_eq!(rel, PathBuf::from("."),
                        "Relative path to self should be '.': {} -> {:?}", path_str, rel_clone);
                }
                None => prop_assert!(false, "Should always find relative path to self: {}", path_str)
            }
        }

        /// Property: If relative_path returns a result, joining it should get back to target
        #[test]
        fn relative_path_correctness(
            base_str in "[a-zA-Z0-9/]{3,15}",  // Use simpler paths to avoid edge cases
            target_str in "[a-zA-Z0-9/]{3,15}"
        ) {
            let base = PathBuf::from(&base_str);
            let target = PathBuf::from(&target_str);

            // Skip cases where either path contains problematic patterns
            prop_assume!(!base_str.contains("..") && !target_str.contains(".."));
            prop_assume!(!base_str.starts_with("/") && !target_str.starts_with("/"));

            if let Some(relative) = relative_path(&base, &target) {
                let reconstructed = base.join(&relative);
                let reconstructed_normalized = normalize_path(&reconstructed);
                let target_normalized = normalize_path(&target);

                prop_assert!(paths_equal(&reconstructed_normalized, &target_normalized),
                    "base.join(relative_path) should equal target: {} + {:?} = {:?} vs {} -> {:?}",
                    base_str, relative, reconstructed_normalized, target_str, target_normalized);
            }
        }

        /// Property: Relative paths should not contain unnecessary components
        #[test]
        fn relative_path_minimal(
            base_str in "[a-zA-Z0-9/]{5,20}",
            target_str in "[a-zA-Z0-9/]{5,20}"
        ) {
            let base = PathBuf::from(&base_str);
            let target = PathBuf::from(&target_str);

            if let Some(relative) = relative_path(&base, &target) {
                let rel_str = relative.to_string_lossy();

                // Should not contain unnecessary current directory references
                prop_assert!(!rel_str.contains("/./"),
                    "Relative path should not contain unnecessary current dir refs: {:?}", relative);

                // Should not end with /. unless it's just "."
                if rel_str != "." {
                    prop_assert!(!rel_str.ends_with("/."),
                        "Relative path should not end with /.: {:?}", relative);
                }
            }
        }
    }
}

#[cfg(test)]
mod path_validator_properties {
    use super::*;

    proptest! {
        /// Property: Workspace path generation should be deterministic
        #[test]
        fn workspace_generation_deterministic(
            agent_type_str in arb_agent_type(),
            _seed in 0u64..1000u64
        ) {
            // Create deterministic session ID from seed
            let session_id = SessionId::generate(); // Use generated session for deterministic test
            let agent_type: AgentType = agent_type_str.clone();

            let temp_dir = std::env::temp_dir();
            let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

            let workspace1 = validator.generate_workspace_path(&temp_dir, &session_id, &agent_type);
            let workspace2 = validator.generate_workspace_path(&temp_dir, &session_id, &agent_type);

            prop_assert_eq!(workspace1, workspace2,
                "Same inputs should generate same workspace: session={}, agent={}",
                session_id.as_str(), agent_type_str);
        }

        /// Property: Different session IDs should generate different workspaces
        #[test]
        fn workspace_generation_unique_sessions(
            agent_type_str in arb_agent_type(),
            seed1 in 0u64..1000u64,
            seed2 in 0u64..1000u64
        ) {
            prop_assume!(seed1 != seed2);

            let session1 = SessionId::generate();
            let session2 = SessionId::generate();
            let agent_type: AgentType = agent_type_str;

            let temp_dir = std::env::temp_dir();
            let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

            let workspace1 = validator.generate_workspace_path(&temp_dir, &session1, &agent_type);
            let workspace2 = validator.generate_workspace_path(&temp_dir, &session2, &agent_type);

            prop_assert_ne!(workspace1, workspace2,
                "Different sessions should generate different workspaces");
        }

        /// Property: Generated workspaces should always be within the root directory
        #[test]
        fn workspace_generation_within_root(
            agent_type_str in arb_agent_type(),
            _seed in 0u64..1000u64
        ) {
            let session_id = SessionId::generate();
            let agent_type: AgentType = agent_type_str;

            let temp_dir = std::env::temp_dir();
            let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

            let workspace = validator.generate_workspace_path(&temp_dir, &session_id, &agent_type);

            prop_assert!(workspace.starts_with(&temp_dir),
                "Generated workspace should be within root: {:?} vs {:?}",
                workspace, temp_dir);
        }

        /// Property: Valid paths within workspace should be accepted
        #[test]
        fn path_validation_accepts_valid_paths(
            file_name in "[a-zA-Z0-9][a-zA-Z0-9_-]{0,19}\\.(txt|rs|json|toml)",  // Start with alphanumeric, then allow hyphens
            subdir in option::of("[a-zA-Z0-9][a-zA-Z0-9_-]{0,14}")
        ) {
            let temp_dir = std::env::temp_dir();
            let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

            let path = match subdir {
                Some(dir) => PathBuf::from(format!("{dir}/{file_name}")),
                None => PathBuf::from(file_name),
            };

            let result = validator.validate_workspace_path(&path, &temp_dir);
            prop_assert!(result.is_ok(),
                "Valid path should be accepted: {:?} in {:?}", path, temp_dir);
        }

        /// Property: Malicious paths should be rejected or contained
        #[test]
        fn path_validation_handles_malicious_paths(malicious_path in arb_malicious_path_string()) {
            let temp_dir = std::env::temp_dir();
            let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

            let path = PathBuf::from(&malicious_path);
            let result = validator.validate_workspace_path(&path, &temp_dir);

            match result {
                Ok(canonical) => {
                    // Use smart path comparison that handles macOS symlinks
                    let canonical_workspace = if temp_dir.exists() {
                        temp_dir
                            .canonicalize()
                            .unwrap_or_else(|_| temp_dir.clone())
                    } else {
                        temp_dir.clone()
                    };

                    // If accepted, must be contained within workspace
                    prop_assert!(canonical.starts_with(&canonical_workspace),
                        "Accepted path must be within workspace: {} -> {:?} vs {:?}",
                        malicious_path, canonical, canonical_workspace);
                }
                Err(_) => {
                    // Rejection is also acceptable for security
                }
            }
        }

        /// Property: Blocked patterns should consistently block matching paths
        #[test]
        fn blocked_patterns_consistent(
            pattern in "\\*\\.(tmp|log|secret|key)",
            filename_base in "[a-zA-Z0-9_]{1,10}"
        ) {
            let temp_dir = std::env::temp_dir();
            let validator = PathValidator::new(vec![temp_dir.clone()], vec![pattern.clone()]);

            // Create filenames that should match the pattern
            let matching_files = vec![
                format!("{}.tmp", filename_base),
                format!("{}.log", filename_base),
                format!("{}.secret", filename_base),
                format!("{}.key", filename_base),
            ];

            for file in matching_files {
                if pattern.contains(&file[file.rfind('.').unwrap_or(0)..]) {
                    let path = PathBuf::from(&file);
                    prop_assert!(validator.is_blocked_path(&path),
                        "File matching pattern should be blocked: {} matches {}",
                        file, pattern);
                }
            }
        }
    }
}

#[cfg(test)]
mod security_properties {
    use super::*;

    proptest! {
        /// Property: No path traversal attack should escape workspace bounds
        #[test]
        fn no_path_traversal_escapes_workspace(
            traversal_depth in 1u8..20u8,
            target_path in "[a-zA-Z0-9/]{1,30}"
        ) {
            let temp_dir = std::env::temp_dir();
            let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

            // Create path traversal attack
            let attack = format!("{}{}", "../".repeat(traversal_depth as usize), target_path);
            let attack_path = PathBuf::from(attack.clone());

            let result = validator.validate_workspace_path(&attack_path, &temp_dir);

            match result {
                Ok(canonical) => {
                    // If accepted, must still be within workspace bounds
                    prop_assert!(canonical.starts_with(&temp_dir),
                        "Path traversal should not escape workspace: {} -> {:?} vs {:?}",
                        attack, canonical, temp_dir);
                }
                Err(_) => {
                    // Rejection is the preferred security behavior
                }
            }
        }

        /// Property: Unicode attack variations should not bypass security
        #[test]
        fn unicode_attacks_contained(
            base_attack in "\\.\\./etc/[a-z]+",
            unicode_sep in "[\u{FF0F}\u{2044}\u{2215}]"
        ) {
            let temp_dir = std::env::temp_dir();
            let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

            // Replace normal slashes with Unicode equivalents
            let unicode_attack = base_attack.replace("/", &unicode_sep.to_string());
            let attack_path = PathBuf::from(&unicode_attack);

            let result = validator.validate_workspace_path(&attack_path, &temp_dir);

            match result {
                Ok(canonical) => {
                    // Unicode attacks should not escape workspace if accepted
                    prop_assert!(canonical.starts_with(&temp_dir),
                        "Unicode attack should not escape: {} -> {:?}",
                        unicode_attack, canonical);
                }
                Err(_) => {
                    // Rejection is preferred for Unicode attacks
                }
            }
        }

        /// Property: Control character injection should not compromise security
        #[test]
        fn control_character_injection_safe(
            base_path in "[a-zA-Z0-9]{1,10}",
            control_char in 0u8..32u8, // ASCII control characters range
            attack_suffix in "\\.\\./etc/passwd"
        ) {
            let temp_dir = std::env::temp_dir();
            let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

            // Inject control character between safe path and attack
            let attack = format!("{}{}{}", base_path, control_char as char, attack_suffix);
            let attack_path = PathBuf::from(&attack);

            let result = validator.validate_workspace_path(&attack_path, &temp_dir);

            match result {
                Ok(canonical) => {
                    // Control character injection should not escape workspace
                    prop_assert!(canonical.starts_with(&temp_dir),
                        "Control char injection should not escape: {:?} -> {:?}",
                        attack, canonical);
                }
                Err(_) => {
                    // Rejection is acceptable
                }
            }
        }
    }
}
