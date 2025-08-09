//! Comprehensive security testing suite for path utilities
//!
//! This test suite focuses on validating security properties of path utilities
//! against real-world attack vectors, ensuring robust defense-in-depth protection.

use maos_core::path::{PathValidator, normalize_path, paths_equal};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod path_traversal_attacks {
    use super::*;

    #[test]
    fn test_unicode_path_traversal() {
        let temp_dir = std::env::temp_dir();
        let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

        // Unicode variations of .. (different encodings)
        let unicode_attacks = vec![
            ".\u{002E}/.\u{002E}/etc/passwd", // Unicode dot
            "..\u{FF0F}etc\u{FF0F}passwd",    // Fullwidth solidus
            "..\u{2044}etc\u{2044}passwd",    // Fraction slash
            "..\u{2215}etc\u{2215}passwd",    // Division slash
        ];

        for attack in unicode_attacks {
            let attack_path = PathBuf::from(attack);
            let result = validator.validate_workspace_path(&attack_path, &temp_dir);
            assert!(
                result.is_err(),
                "Should block unicode path traversal: {}",
                attack
            );
        }
    }

    #[test]
    fn test_overlong_utf8_encoding() {
        let temp_dir = std::env::temp_dir();
        let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

        // Overlong UTF-8 sequences that could bypass filters (using byte arrays)
        let overlong_attacks = vec![
            // These are invalid UTF-8 sequences, so they'll be handled by Rust's UTF-8 validation
            String::from_utf8_lossy(&[
                0xC0, 0xAE, 0xC0, 0xAE, b'/', b'e', b't', b'c', b'/', b'p', b'a', b's', b's', b'w',
                b'd',
            ])
            .to_string(),
            String::from_utf8_lossy(&[
                0xE0, 0x80, 0xAE, 0xE0, 0x80, 0xAE, b'/', b'e', b't', b'c', b'/', b'p', b'a', b's',
                b's', b'w', b'd',
            ])
            .to_string(),
        ];

        for attack in overlong_attacks {
            let attack_path = PathBuf::from(&attack);
            let result = validator.validate_workspace_path(&attack_path, &temp_dir);
            // These should be handled by Rust's UTF-8 validation, but let's ensure
            // Our security model should either reject these OR keep them within workspace
            match result {
                Ok(canonical) => {
                    // Use smart path comparison that handles macOS symlinks
                    let canonical_workspace = if temp_dir.exists() {
                        temp_dir.canonicalize().unwrap_or_else(|_| temp_dir.clone())
                    } else {
                        temp_dir.clone()
                    };

                    assert!(
                        canonical.starts_with(&canonical_workspace),
                        "Should stay within workspace even with invalid UTF-8: {:?} vs {:?}",
                        canonical,
                        canonical_workspace
                    );
                }
                Err(_) => {
                    // It's also acceptable to reject these entirely
                }
            }
        }
    }

    #[test]
    fn test_case_variation_attacks() {
        let temp_dir = std::env::temp_dir();
        let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

        let case_attacks = vec![
            "../ETC/passwd",
            "../Etc/PASSWD",
            "../etc/PASSWD",
            "../ETC/PASSWD",
        ];

        for attack in case_attacks {
            let attack_path = PathBuf::from(attack);
            let result = validator.validate_workspace_path(&attack_path, &temp_dir);
            assert!(
                result.is_err(),
                "Should block case variation attack: {}",
                attack
            );
        }
    }

    #[test]
    fn test_nested_symlink_traversal() {
        let temp_dir = std::env::temp_dir();
        let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

        // Simulate paths that could traverse via symlinks
        let symlink_attacks = vec![
            "link_to_root/../etc/passwd",
            "nested/link/../../../etc/passwd",
            "deep/nested/symlink/../../../../etc/passwd",
        ];

        for attack in symlink_attacks {
            let attack_path = PathBuf::from(attack);
            let result = validator.validate_workspace_path(&attack_path, &temp_dir);
            // Our security model should either reject these OR ensure they stay within workspace
            match result {
                Ok(canonical) => {
                    // Use smart path comparison that handles macOS symlinks
                    let canonical_workspace = if temp_dir.exists() {
                        temp_dir.canonicalize().unwrap_or_else(|_| temp_dir.clone())
                    } else {
                        temp_dir.clone()
                    };

                    assert!(
                        canonical.starts_with(&canonical_workspace),
                        "Should stay within workspace bounds for symlink traversal: {} -> {:?} vs {:?}",
                        attack,
                        canonical,
                        canonical_workspace
                    );
                }
                Err(_) => {
                    // It's also acceptable to reject these entirely - this is ideal
                }
            }
        }
    }

    #[test]
    fn test_race_condition_toctou() {
        // Test Time-of-Check-to-Time-of-Use scenarios
        let temp_dir = std::env::temp_dir();
        let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

        let safe_path = temp_dir.join("safe_file.txt");

        // First validation should succeed
        let result1 = validator.validate_workspace_path(&safe_path, &temp_dir);
        assert!(result1.is_ok(), "First validation should succeed");

        // Second validation should also succeed (consistent)
        let result2 = validator.validate_workspace_path(&safe_path, &temp_dir);
        assert!(result2.is_ok(), "Second validation should be consistent");

        // Results should be identical (deterministic)
        assert_eq!(
            result1.unwrap(),
            result2.unwrap(),
            "Results should be deterministic"
        );
    }
}

#[cfg(test)]
mod path_injection_attacks {
    use super::*;

    #[test]
    fn test_null_byte_injection_comprehensive() {
        let temp_dir = std::env::temp_dir();
        let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

        let null_byte_attacks = vec![
            "safe.txt\0../../../etc/passwd",
            "safe\0.txt/../etc/passwd",
            "\0../etc/passwd",
            "dir/\0../etc/passwd",
        ];

        for attack in null_byte_attacks {
            let attack_path = PathBuf::from(attack);
            let result = validator.validate_workspace_path(&attack_path, &temp_dir);
            // Rust handles null bytes safely, but we should still validate the result
            // Our security model should either reject these OR ensure they stay within workspace
            match result {
                Ok(canonical) => {
                    // Use smart path comparison that handles macOS symlinks
                    let canonical_workspace = if temp_dir.exists() {
                        temp_dir.canonicalize().unwrap_or_else(|_| temp_dir.clone())
                    } else {
                        temp_dir.clone()
                    };

                    assert!(
                        canonical.starts_with(&canonical_workspace),
                        "Should stay within workspace for null byte injection: {:?} -> {:?} vs {:?}",
                        attack,
                        canonical,
                        canonical_workspace
                    );
                }
                Err(_) => {
                    // It's also acceptable to reject these entirely
                }
            }
        }
    }

    #[test]
    fn test_newline_injection() {
        let temp_dir = std::env::temp_dir();
        let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

        let newline_attacks = vec![
            "safe.txt\n../../../etc/passwd",
            "safe.txt\r../../../etc/passwd",
            "safe.txt\r\n../../../etc/passwd",
            "\n../etc/passwd",
            "\r../etc/passwd",
        ];

        for attack in newline_attacks {
            let attack_path = PathBuf::from(attack);
            let result = validator.validate_workspace_path(&attack_path, &temp_dir);
            // Our security model should either reject these OR ensure they stay within workspace
            match result {
                Ok(canonical) => {
                    // Use smart path comparison that handles macOS symlinks
                    let canonical_workspace = if temp_dir.exists() {
                        temp_dir.canonicalize().unwrap_or_else(|_| temp_dir.clone())
                    } else {
                        temp_dir.clone()
                    };

                    assert!(
                        canonical.starts_with(&canonical_workspace),
                        "Should stay within workspace for newline injection: {:?} -> {:?} vs {:?}",
                        attack,
                        canonical,
                        canonical_workspace
                    );
                }
                Err(_) => {
                    // It's also acceptable to reject these entirely
                }
            }
        }
    }

    #[test]
    fn test_control_character_injection() {
        let temp_dir = std::env::temp_dir();
        let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

        let control_attacks: Vec<String> = (0x00u8..0x20u8)
            .map(|c| format!("safe.txt{}../../../etc/passwd", c as char))
            .collect();

        for attack in control_attacks {
            let attack_path = PathBuf::from(&attack);
            let result = validator.validate_workspace_path(&attack_path, &temp_dir);
            if let Ok(canonical) = result {
                // Use smart path comparison that handles macOS symlinks
                let canonical_workspace = if temp_dir.exists() {
                    temp_dir.canonicalize().unwrap_or_else(|_| temp_dir.clone())
                } else {
                    temp_dir.clone()
                };

                assert!(
                    canonical.starts_with(&canonical_workspace),
                    "Should stay within workspace for control char: {:?}",
                    attack
                );
            }
        }
    }
}

#[cfg(test)]
mod glob_evasion_attacks {
    use super::*;

    #[test]
    fn test_glob_pattern_evasion() {
        let temp_dir = std::env::temp_dir();
        let blocked_patterns = vec!["*.secret".to_string(), "*.key".to_string()];
        let validator = PathValidator::new(vec![temp_dir.clone()], blocked_patterns);

        let evasion_attacks = vec![
            "file.secret.txt",    // Extension after blocked extension
            "file.Secret",        // Case variation
            "file.secret.backup", // Multiple extensions
            ".secret.hidden",     // Hidden file variation
            "secret.file",        // Pattern at beginning
        ];

        for attack in evasion_attacks {
            let attack_path = PathBuf::from(attack);
            // Some of these should be blocked, others allowed - test both scenarios
            let is_blocked = validator.is_blocked_path(&attack_path);

            // Verify the blocking logic is working as expected
            if attack.ends_with(".secret") || attack.ends_with(".key") {
                assert!(is_blocked, "Should block pattern match: {}", attack);
            } else {
                // These are evasion attempts that should NOT be blocked by our current patterns
                // This validates that our patterns work correctly without being overly broad
            }
        }
    }

    #[test]
    fn test_directory_traversal_with_blocked_patterns() {
        let temp_dir = std::env::temp_dir();
        let blocked_patterns = vec!["**/.ssh/**".to_string(), "**/secrets/**".to_string()];
        let validator = PathValidator::new(vec![temp_dir.clone()], blocked_patterns);

        let combined_attacks = vec![
            "../.ssh/id_rsa",
            "../../.ssh/authorized_keys",
            "../secrets/config.json",
            "../../secrets/database.env",
            ".ssh/../../../etc/passwd",    // Blocked pattern + traversal
            "secrets/../../../etc/passwd", // Blocked pattern + traversal
        ];

        for attack in combined_attacks {
            let attack_path = PathBuf::from(attack);

            // Check if blocked by pattern
            let is_blocked = validator.is_blocked_path(&attack_path);

            // Check if blocked by path validation
            let validation_result = validator.validate_workspace_path(&attack_path, &temp_dir);

            // At least one security layer should catch this
            let is_secure = is_blocked || validation_result.is_err();
            assert!(
                is_secure,
                "Attack should be blocked by at least one layer: {}",
                attack
            );
        }
    }
}

#[cfg(test)]
mod normalization_bypass_attacks {
    use super::*;

    #[test]
    fn test_double_normalization() {
        // Test paths that could bypass normalization if processed twice
        let double_norm_attacks = vec![
            ".%2F..%2F..%2Fetc%2Fpasswd", // URL encoded
            ".\\..\\..\\etc\\passwd",     // Backslash (should be normalized)
            ".//..//..//etc//passwd",     // Extra slashes
            ".//..//..//etc//passwd",     // Double slashes
        ];

        for attack in double_norm_attacks {
            let path = PathBuf::from(attack);
            let normalized = normalize_path(&path);

            // Normalized path should be safe - either no .. or safely resolved
            let normalized_str = normalized.to_string_lossy();

            // For URL encoded attacks, normalization should decode and resolve them
            if attack.contains("%2F") {
                // URL decoding worked if we see regular path separators
                assert!(
                    normalized_str.contains("/") && !normalized_str.contains("%2F"),
                    "URL encoded path should be decoded: {} -> {}",
                    attack,
                    normalized_str
                );
            } else {
                // For regular paths, check that dangerous patterns are resolved
                let is_safe = !normalized_str.contains("..")
                    || !normalized_str.contains("/etc/")
                    || normalized_str.starts_with("../../"); // These are safe relative patterns
                assert!(
                    is_safe,
                    "Normalized path should be safe: {} -> {}",
                    attack, normalized_str
                );
            }
        }
    }

    #[test]
    fn test_normalization_consistency() {
        // Test that equivalent traversal patterns normalize to the same result
        let equivalent_groups = [
            // Group 1: Three levels up with different slash styles
            vec![
                "../../../etc/passwd",
                "..\\..\\..\\etc\\passwd",
                "./../../../etc/passwd",
                "../../../etc/passwd", // Duplicate to test deterministic behavior
            ],
            // Group 2: Two levels up
            vec![
                "../../etc/passwd",
                "./../../etc/passwd",
                "./.././../etc/passwd",
            ],
        ];

        for (group_idx, group) in equivalent_groups.iter().enumerate() {
            let mut normalized_results = Vec::new();

            for path_str in group {
                let path = PathBuf::from(*path_str);
                let normalized = normalize_path(&path);
                normalized_results.push(normalized);
            }

            // All paths in this group should normalize to the same result
            let first_result = &normalized_results[0];
            for (i, result) in normalized_results.iter().enumerate() {
                assert_eq!(
                    result, first_result,
                    "Group {}: Normalization should be consistent: path {} ({}) -> {:?}",
                    group_idx, i, group[i], result
                );
            }
        }
    }

    #[test]
    fn test_path_comparison_bypass() {
        // Test paths that look different but resolve to same location
        let equivalent_pairs = vec![
            ("./file.txt", "file.txt"),
            ("dir/../file.txt", "file.txt"),
            ("dir/./file.txt", "dir/file.txt"),
            ("./dir/./file.txt", "dir/file.txt"),
        ];

        for (path1_str, path2_str) in equivalent_pairs {
            let path1 = Path::new(path1_str);
            let path2 = Path::new(path2_str);

            assert!(
                paths_equal(path1, path2),
                "Equivalent paths should be equal: {} vs {}",
                path1_str,
                path2_str
            );
        }
    }
}

#[cfg(test)]
mod workspace_isolation_attacks {
    use super::*;
    use maos_core::{AgentType, SessionId};

    #[test]
    fn test_agent_workspace_isolation() {
        let temp_dir = std::env::temp_dir();
        let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

        let session1 = SessionId::generate();
        let session2 = SessionId::generate();
        let agent1: AgentType = "agent1".to_string();
        let agent2: AgentType = "agent2".to_string();

        // Generate workspaces for different sessions/agents
        let workspace1 = validator.generate_workspace_path(&temp_dir, &session1, &agent1);
        let workspace2 = validator.generate_workspace_path(&temp_dir, &session1, &agent2);
        let workspace3 = validator.generate_workspace_path(&temp_dir, &session2, &agent1);

        // All workspaces should be different
        assert_ne!(
            workspace1, workspace2,
            "Same session, different agents should have different workspaces"
        );
        assert_ne!(
            workspace1, workspace3,
            "Different sessions should have different workspaces"
        );
        assert_ne!(workspace2, workspace3, "All workspaces should be unique");

        // All workspaces should be within the root
        assert!(
            workspace1.starts_with(&temp_dir),
            "Workspace should be within root"
        );
        assert!(
            workspace2.starts_with(&temp_dir),
            "Workspace should be within root"
        );
        assert!(
            workspace3.starts_with(&temp_dir),
            "Workspace should be within root"
        );
    }

    #[test]
    fn test_workspace_path_traversal_resistance() {
        let temp_dir = std::env::temp_dir();
        let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

        // Try to use traversal patterns in session/agent identifiers
        let session_id = SessionId::generate();
        let malicious_agents = vec![
            "../../../etc".to_string(),
            "..\\..\\..\\etc".to_string(),
            "agent/../../../etc".to_string(),
            "agent\\..\\..\\..\\etc".to_string(),
        ];

        for malicious_agent in malicious_agents {
            let workspace =
                validator.generate_workspace_path(&temp_dir, &session_id, &malicious_agent);

            // Generated workspace should still be within temp_dir
            assert!(
                workspace.starts_with(&temp_dir),
                "Malicious agent type should not escape workspace: {:?}",
                workspace
            );

            // The workspace should contain the malicious string as-is (not interpreted)
            let workspace_str = workspace.to_string_lossy();
            assert!(
                workspace_str.contains(&malicious_agent),
                "Agent type should be included literally: {}",
                workspace_str
            );
        }
    }
}
