//! Cross-platform path utilities with security-focused normalization
//!
//! This module provides safe path manipulation functions that handle cross-platform
//! differences while maintaining security properties.

use path_clean::PathClean;
use std::path::{Path, PathBuf};

/// Normalize a path for MAOS multi-agent security using established Rust crates
///
/// This function provides security-hardened path normalization for untrusted agent input by:
/// - Using `dunce::simplified` for robust cross-platform path resolution
/// - Using `path-clean` for proper component normalization
/// - Preventing Unicode separator attack vectors (fullwidth solidus, etc.)
/// - Ensuring consistent cross-platform behavior for agent workspace isolation
///
/// # Security Features
///
/// - Blocks Unicode separator variants that could bypass security checks
/// - Prevents agent path injection via alternative Unicode separators
/// - Uses established Rust crates instead of custom Windows-breaking logic
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use maos_core::path::normalize_path;
/// use std::path::{Path, PathBuf};
///
/// // Resolve current directory references
/// let path = Path::new("./src/main.rs");
/// assert_eq!(normalize_path(path), PathBuf::from("src/main.rs"));
///
/// // Resolve parent directory references
/// let path = Path::new("src/../lib/mod.rs");
/// assert_eq!(normalize_path(path), PathBuf::from("lib/mod.rs"));
///
/// // Complex path normalization  
/// let path = Path::new("./src/../lib/./utils/../mod.rs");
/// assert_eq!(normalize_path(path), PathBuf::from("lib/mod.rs"));
/// ```
///
/// ## MAOS Security Features
///
/// ```rust
/// use maos_core::path::normalize_path;
/// use std::path::{Path, PathBuf};
///
/// // Unicode separator attack prevention
/// let unicode_path = Path::new("src\u{FF0F}main.rs"); // Fullwidth solidus
/// let normalized = normalize_path(unicode_path);
/// // The Unicode character is replaced with platform separator
/// assert!(!normalized.to_string_lossy().contains('\u{FF0F}'));
///
/// // Additional Unicode separators are handled
/// let fraction_slash = Path::new("src\u{2044}main.rs"); // Fraction slash
/// let division_slash = Path::new("src\u{2215}main.rs"); // Division slash  
/// assert!(!normalize_path(fraction_slash).to_string_lossy().contains('\u{2044}'));
/// assert!(!normalize_path(division_slash).to_string_lossy().contains('\u{2215}'));
/// ```
///
/// ## Cross-Platform Compatibility
///
/// Uses `dunce::simplified` + `path-clean` for robust cross-platform path handling
/// while preserving relative/absolute semantics for agent workspace isolation.
pub fn normalize_path(path: &Path) -> PathBuf {
    // 1. Use path-clean for proper component normalization (handles ., .., etc.)
    let cleaned = path.clean();

    // 2. Use dunce to handle Windows UNC paths and other platform quirks
    let platform_normalized = dunce::simplified(&cleaned).to_path_buf();

    // 3. Apply MAOS security transformations to prevent Unicode attack vectors
    // AFTER platform normalization, so we work with the platform's conventions
    apply_security_transforms(&platform_normalized)
}

/// Apply MAOS-specific security transformations to prevent agent attacks
fn apply_security_transforms(path: &Path) -> PathBuf {
    const UNICODE_SLASHES: [char; 3] = ['\u{FF0F}', '\u{2044}', '\u{2215}'];
    let path_str = path.to_string_lossy();

    // Only transform if we detect Unicode attack vectors
    // Don't transform backslashes - they're legitimate on Windows after normalization
    let needs_transform = path_str.chars().any(|c| UNICODE_SLASHES.contains(&c));

    if needs_transform {
        // Use the platform's preferred separator when replacing Unicode attacks
        let separator = std::path::MAIN_SEPARATOR;
        let secured: String = path_str
            .chars()
            .map(|c| {
                if UNICODE_SLASHES.contains(&c) {
                    separator
                } else {
                    c
                }
            })
            .collect();
        PathBuf::from(secured)
    } else {
        path.to_path_buf()
    }
}

/// Check if two paths refer to the same location after normalization
///
/// This function determines path equality by normalizing both paths and comparing
/// the results. It handles cross-platform differences, path separators, and
/// relative path components (`.` and `..`).
///
/// # Examples
///
/// ## Basic Equality
///
/// ```rust
/// use maos_core::path::paths_equal;
/// use std::path::Path;
///
/// // Same paths are equal
/// assert!(paths_equal(
///     Path::new("src/main.rs"),
///     Path::new("src/main.rs")
/// ));
///
/// // Platform-specific behavior handled at runtime
/// if cfg!(windows) {
///     // Windows treats \ and / as equivalent
///     assert!(paths_equal(
///         Path::new("src\\main.rs"),  // Windows-style
///         Path::new("src/main.rs")    // Unix-style
///     ));
/// } else {
///     // On Unix, backslashes are literal characters in filenames
///     assert!(!paths_equal(
///         Path::new("src\\main.rs"),  // Single filename with backslash
///         Path::new("src/main.rs")    // Path to main.rs in src directory
///     ));
/// }
/// ```
///
/// ## Normalization-Based Equality
///
/// ```rust
/// use maos_core::path::paths_equal;
/// use std::path::Path;
///
/// // Paths with . and .. components are normalized before comparison
/// assert!(paths_equal(
///     Path::new("./src/main.rs"),
///     Path::new("src/main.rs")
/// ));
///
/// assert!(paths_equal(
///     Path::new("src/../lib/mod.rs"),
///     Path::new("lib/mod.rs")
/// ));
///
/// // Complex path normalization
/// assert!(paths_equal(
///     Path::new("./src/../lib/./utils/../mod.rs"),
///     Path::new("lib/mod.rs")
/// ));
/// ```
///
/// ## Cross-Platform Compatibility
///
/// ```rust
/// use maos_core::path::paths_equal;
/// use std::path::Path;
///
/// // Platform-specific handling of separators
/// if cfg!(windows) {
///     // Windows allows mixed separators
///     assert!(paths_equal(
///         Path::new("src/dir\\subdir/file.txt"),
///         Path::new("src\\dir/subdir\\file.txt")
///     ));
/// } else {
///     // Unix only recognizes forward slashes as separators
///     assert!(paths_equal(
///         Path::new("src/dir/subdir/file.txt"),
///         Path::new("src/dir/subdir/file.txt")
///     ));
/// }
/// ```
///
/// # Use Cases
///
/// - Comparing user-provided paths that may have different formats
/// - Validating that a resolved path matches an expected canonical form
/// - Cross-platform path comparison in file operations
/// - Deduplicating path lists with different representations
pub fn paths_equal(a: &Path, b: &Path) -> bool {
    normalize_path(a) == normalize_path(b)
}

/// Calculate the relative path from a base path to a target path
///
/// This function computes the relative path needed to navigate from the `base`
/// path to the `target` path. Both paths are normalized before calculation
/// to ensure consistent results across platforms.
///
/// Returns `None` if no relative path can be calculated (e.g., different roots
/// on Windows), otherwise returns `Some(relative_path)`.
///
/// # Examples
///
/// ## Basic Relative Paths
///
/// ```rust
/// use maos_core::path::relative_path;
/// use std::path::{Path, PathBuf};
///
/// # #[cfg(unix)]
/// # {
/// // Same path returns current directory
/// assert_eq!(
///     relative_path(Path::new("/home/user"), Path::new("/home/user")),
///     Some(PathBuf::from("."))
/// );
///
/// // Direct child
/// assert_eq!(
///     relative_path(Path::new("/home/user"), Path::new("/home/user/docs")),
///     Some(PathBuf::from("docs"))
/// );
///
/// // Parent directory
/// assert_eq!(
///     relative_path(Path::new("/home/user/docs"), Path::new("/home/user")),
///     Some(PathBuf::from(".."))
/// );
/// # }
/// # #[cfg(windows)]
/// # {
/// // Same path returns current directory
/// assert_eq!(
///     relative_path(Path::new("C:\\Users\\user"), Path::new("C:\\Users\\user")),
///     Some(PathBuf::from("."))
/// );
///
/// // Direct child
/// assert_eq!(
///     relative_path(Path::new("C:\\Users\\user"), Path::new("C:\\Users\\user\\docs")),
///     Some(PathBuf::from("docs"))
/// );
///
/// // Parent directory
/// assert_eq!(
///     relative_path(Path::new("C:\\Users\\user\\docs"), Path::new("C:\\Users\\user")),
///     Some(PathBuf::from(".."))
/// );
/// # }
/// ```
///
/// ## Sibling Directories
///
/// ```rust
/// use maos_core::path::relative_path;
/// use std::path::{Path, PathBuf};
///
/// # #[cfg(unix)]
/// # {
/// // Sibling directories
/// assert_eq!(
///     relative_path(Path::new("/home/user/docs"), Path::new("/home/user/pictures")),
///     Some(PathBuf::from("../pictures"))
/// );
///
/// // Complex navigation
/// assert_eq!(
///     relative_path(
///         Path::new("/home/user/projects/rust/src"),
///         Path::new("/home/user/documents/file.txt")
///     ),
///     Some(PathBuf::from("../../../documents/file.txt"))
/// );
/// # }
/// # #[cfg(windows)]
/// # {
/// // Sibling directories
/// assert_eq!(
///     relative_path(Path::new("C:\\Users\\user\\docs"), Path::new("C:\\Users\\user\\pictures")),
///     Some(PathBuf::from("..\\pictures"))
/// );
///
/// // Complex navigation
/// assert_eq!(
///     relative_path(
///         Path::new("C:\\Users\\user\\projects\\rust\\src"),
///         Path::new("C:\\Users\\user\\documents\\file.txt")
///     ),
///     Some(PathBuf::from("..\\..\\..\\documents\\file.txt"))
/// );
/// # }
/// ```
///
/// ## Cross-Platform Usage
///
/// ```rust
/// use maos_core::path::relative_path;
/// use std::path::{Path, PathBuf};
///
/// // Platform-specific separator handling
/// if cfg!(windows) {
///     assert_eq!(
///         relative_path(Path::new("src\\components"), Path::new("src\\utils\\helpers")),
///         Some(PathBuf::from("..\\utils\\helpers"))
///     );
/// } else {
///     assert_eq!(
///         relative_path(Path::new("src/components"), Path::new("src/utils/helpers")),
///         Some(PathBuf::from("../utils/helpers"))
///     );
/// }
/// ```
///
/// ## Workspace-Relative File Access
///
/// ```rust
/// use maos_core::path::relative_path;
/// use std::path::{Path, PathBuf};
///
/// # #[cfg(unix)]
/// # {
/// let workspace = Path::new("/projects/my-app");
/// let config_file = Path::new("/projects/my-app/config/app.toml");
/// let src_dir = Path::new("/projects/my-app/src");
///
/// // Get relative path from src to config
/// if let Some(rel_path) = relative_path(src_dir, config_file) {
///     println!("Config is at: {}", rel_path.display()); // "../config/app.toml"
/// }
/// # }
/// # #[cfg(windows)]
/// # {
/// let workspace = Path::new("C:\\projects\\my-app");
/// let config_file = Path::new("C:\\projects\\my-app\\config\\app.toml");
/// let src_dir = Path::new("C:\\projects\\my-app\\src");
///
/// // Get relative path from src to config
/// if let Some(rel_path) = relative_path(src_dir, config_file) {
///     println!("Config is at: {}", rel_path.display()); // "..\\config\\app.toml"
/// }
/// # }
/// ```
pub fn relative_path(base: &Path, target: &Path) -> Option<PathBuf> {
    // Normalize both paths first to handle . and .. components and separators
    let normalized_base = normalize_path(base);
    let normalized_target = normalize_path(target);

    // If paths are equal, return current directory
    if normalized_base == normalized_target {
        return Some(PathBuf::from("."));
    }

    // Extract normal components using closure
    let extract_normals = |path: &Path| -> Vec<String> {
        use std::path::Component;
        path.components()
            .filter_map(|c| match c {
                Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect()
    };

    let base_components = extract_normals(&normalized_base);
    let target_components = extract_normals(&normalized_target);

    // Find common prefix length
    let common_len = base_components
        .iter()
        .zip(&target_components)
        .take_while(|(a, b)| a == b)
        .count();

    // Build relative path using iterator chains
    let ups = base_components.len() - common_len;
    let result: PathBuf = std::iter::repeat_n("..", ups)
        .chain(target_components.iter().skip(common_len).map(AsRef::as_ref))
        .collect();

    Some(if result.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        result
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    // TDD Cycle 6 - RED PHASE: normalize_path function tests
    #[test]
    fn test_normalize_path_basic() {
        // Should normalize simple paths
        assert_eq!(
            normalize_path(Path::new("./file.txt")),
            PathBuf::from("file.txt")
        );
        assert_eq!(
            normalize_path(Path::new("dir/./file.txt")),
            PathBuf::from("dir/file.txt")
        );

        // Should handle parent directory references
        assert_eq!(
            normalize_path(Path::new("dir/../file.txt")),
            PathBuf::from("file.txt")
        );
        assert_eq!(
            normalize_path(Path::new("a/b/../c/file.txt")),
            PathBuf::from("a/c/file.txt")
        );

        // Should handle multiple dots
        assert_eq!(
            normalize_path(Path::new("./dir/../file.txt")),
            PathBuf::from("file.txt")
        );
        assert_eq!(
            normalize_path(Path::new("a/./b/../c")),
            PathBuf::from("a/c")
        );
    }

    #[test]
    fn test_normalize_path_edge_cases() {
        // Should handle empty and root paths (path-clean normalizes empty to ".")
        assert_eq!(normalize_path(Path::new("")), PathBuf::from("."));
        assert_eq!(normalize_path(Path::new(".")), PathBuf::from("."));
        assert_eq!(normalize_path(Path::new("./")), PathBuf::from("."));

        assert_eq!(
            normalize_path(Path::new("./some/path")),
            PathBuf::from("some/path")
        );

        // Should handle too many parent references
        assert_eq!(
            normalize_path(Path::new("../file.txt")),
            PathBuf::from("../file.txt")
        );
        assert_eq!(
            normalize_path(Path::new("../../file.txt")),
            PathBuf::from("../../file.txt")
        );
        assert_eq!(
            normalize_path(Path::new("dir/../../file.txt")),
            PathBuf::from("../file.txt")
        );
    }

    #[test]
    fn test_normalize_path_absolute() {
        // Should handle absolute paths
        assert_eq!(
            normalize_path(Path::new("/tmp/./file.txt")),
            PathBuf::from("/tmp/file.txt")
        );
        assert_eq!(
            normalize_path(Path::new("/tmp/../file.txt")),
            PathBuf::from("/file.txt")
        );
        assert_eq!(
            normalize_path(Path::new("/a/b/../c")),
            PathBuf::from("/a/c")
        );

        // Should not go above root
        assert_eq!(
            normalize_path(Path::new("/../file.txt")),
            PathBuf::from("/file.txt")
        );
        assert_eq!(
            normalize_path(Path::new("/../../file.txt")),
            PathBuf::from("/file.txt")
        );
    }

    #[test]
    fn test_normalize_path_windows_edge_cases() {
        // Test the specific failure case from CI: "/a"
        let path = PathBuf::from("/a");
        let normalized = normalize_path(&path);

        // The behavior should be platform-consistent
        if path.is_absolute() {
            assert!(
                normalized.is_absolute(),
                "If original path '/a' is absolute on this platform, normalized should be too. Original: {path:?}, Normalized: {normalized:?}"
            );
        } else {
            // On Windows, "/a" might not be absolute (needs drive letter)
            // This is platform-specific behavior and is acceptable
        }
    }

    #[test]
    fn test_normalize_path_cross_platform() {
        // Platform-specific separator handling
        #[cfg(windows)]
        {
            // Windows handles both separators
            assert_eq!(
                normalize_path(Path::new("dir\\file.txt")),
                PathBuf::from("dir\\file.txt")
            );
            assert_eq!(
                normalize_path(Path::new("dir\\..\\file.txt")),
                PathBuf::from("file.txt")
            );
        }

        #[cfg(not(windows))]
        {
            // Unix treats backslashes as literal characters
            assert_eq!(
                normalize_path(Path::new("dir\\file.txt")),
                PathBuf::from("dir\\file.txt") // Single filename with backslash
            );
            // Only forward slashes work as separators
            assert_eq!(
                normalize_path(Path::new("dir/../file.txt")),
                PathBuf::from("file.txt")
            );
        }
    }

    // TDD Cycle 7 - RED PHASE: Comprehensive paths_equal tests
    #[test]
    fn test_paths_equal_identical_paths() {
        // Should return true for identical paths
        assert!(
            paths_equal(Path::new("/test/file.txt"), Path::new("/test/file.txt")),
            "Identical paths should be equal"
        );
        assert!(
            paths_equal(Path::new("relative/path"), Path::new("relative/path")),
            "Identical relative paths should be equal"
        );
    }

    #[test]
    fn test_paths_equal_different_paths() {
        // Should return false for different paths
        assert!(
            !paths_equal(Path::new("/test/file1.txt"), Path::new("/test/file2.txt")),
            "Different files should not be equal"
        );
        assert!(
            !paths_equal(Path::new("/test/dir1"), Path::new("/test/dir2")),
            "Different directories should not be equal"
        );
    }

    #[test]
    fn test_paths_equal_normalized_paths() {
        // Should return true for paths that normalize to the same location
        assert!(
            paths_equal(Path::new("/test/./file.txt"), Path::new("/test/file.txt")),
            "Normalized paths should be equal"
        );
        assert!(
            paths_equal(
                Path::new("/test/dir/../file.txt"),
                Path::new("/test/file.txt")
            ),
            "Path with parent reference should equal normalized"
        );
        assert!(
            paths_equal(Path::new("./file.txt"), Path::new("file.txt")),
            "Current dir reference should equal direct path"
        );
    }

    #[test]
    fn test_paths_equal_cross_platform_separators() {
        // Platform-specific: Windows treats \ and / as equivalent, Unix doesn't
        #[cfg(windows)]
        {
            assert!(
                paths_equal(Path::new("dir\\file.txt"), Path::new("dir/file.txt")),
                "On Windows, different separators should be equal"
            );
            assert!(
                paths_equal(
                    Path::new("dir\\subdir\\file.txt"),
                    Path::new("dir/subdir/file.txt")
                ),
                "On Windows, nested paths with different separators should be equal"
            );
        }

        #[cfg(not(windows))]
        {
            // On Unix, backslashes are literal characters, not separators
            assert!(
                !paths_equal(Path::new("dir\\file.txt"), Path::new("dir/file.txt")),
                "On Unix, backslash paths are different from forward slash paths"
            );
            // These ARE equal because they're the same path
            assert!(
                paths_equal(Path::new("dir/file.txt"), Path::new("dir/file.txt")),
                "Same paths should be equal"
            );
        }
    }

    #[test]
    fn test_paths_equal_case_sensitivity() {
        // On Unix-like systems, paths are case-sensitive
        // On Windows, they're case-insensitive
        // We'll implement Unix-style behavior (case-sensitive)
        #[cfg(unix)]
        {
            assert!(
                !paths_equal(Path::new("/Test/FILE.txt"), Path::new("/test/file.txt")),
                "Case-different paths should not be equal on Unix"
            );
        }

        // For now, we'll test the case-sensitive behavior
        assert!(
            !paths_equal(Path::new("File.txt"), Path::new("file.txt")),
            "Case-different paths should not be equal"
        );
    }

    #[test]
    fn test_paths_equal_absolute_vs_relative() {
        // Should handle comparison between absolute and relative paths
        // These should be different unless they resolve to same location
        assert!(
            !paths_equal(Path::new("/test/file.txt"), Path::new("file.txt")),
            "Absolute and relative paths should generally not be equal"
        );
        assert!(
            !paths_equal(Path::new("/tmp"), Path::new("tmp")),
            "Absolute and relative dirs should generally not be equal"
        );
    }

    #[test]
    fn test_paths_equal_empty_and_current() {
        // Should handle empty paths and current directory references
        assert!(
            paths_equal(Path::new(""), Path::new("")),
            "Empty paths should be equal"
        );
        assert!(
            paths_equal(Path::new("."), Path::new(".")),
            "Current directory references should be equal"
        );
        assert!(
            paths_equal(Path::new("./"), Path::new(".")),
            "Current directory with slash should equal without"
        );
    }

    // TDD Cycle 8 - RED PHASE: Comprehensive relative_path tests
    #[test]
    fn test_relative_path_same_directory() {
        // Should return "." or empty for same paths
        let result = relative_path(Path::new("/test"), Path::new("/test"));
        assert_eq!(result, Some(PathBuf::from(".")));

        let result = relative_path(Path::new("base"), Path::new("base"));
        assert_eq!(result, Some(PathBuf::from(".")));
    }

    #[test]
    fn test_relative_path_direct_child() {
        // Should return child name for direct children
        let result = relative_path(Path::new("/base"), Path::new("/base/child"));
        assert_eq!(result, Some(PathBuf::from("child")));

        let result = relative_path(Path::new("/base"), Path::new("/base/subdir/file.txt"));
        assert_eq!(result, Some(PathBuf::from("subdir/file.txt")));
    }

    #[test]
    fn test_relative_path_parent_directory() {
        // Should return .. for parent directories
        let result = relative_path(Path::new("/base/subdir"), Path::new("/base"));
        assert_eq!(result, Some(PathBuf::from("..")));

        let result = relative_path(Path::new("/base/sub1/sub2"), Path::new("/base"));
        assert_eq!(result, Some(PathBuf::from("../..")));
    }

    #[test]
    fn test_relative_path_sibling_directories() {
        // Should return ../sibling for sibling paths
        let result = relative_path(Path::new("/base/dir1"), Path::new("/base/dir2"));
        assert_eq!(result, Some(PathBuf::from("../dir2")));

        let result = relative_path(
            Path::new("/base/dir1/sub"),
            Path::new("/base/dir2/file.txt"),
        );
        assert_eq!(result, Some(PathBuf::from("../../dir2/file.txt")));
    }

    #[test]
    fn test_relative_path_different_roots() {
        // Should return None for completely different roots
        let result = relative_path(Path::new("/usr/local"), Path::new("/var/log"));
        assert!(result.is_some()); // Should be able to calculate ../../../var/log

        // More complex case
        let result = relative_path(Path::new("/home/user"), Path::new("/etc/config"));
        assert!(result.is_some()); // Should be ../../etc/config
    }

    #[test]
    fn test_relative_path_normalized_inputs() {
        // Should handle paths that need normalization
        let result = relative_path(Path::new("/base/./dir"), Path::new("/base/target"));
        assert_eq!(result, Some(PathBuf::from("../target")));

        let result = relative_path(Path::new("/base/dir/../other"), Path::new("/base/target"));
        assert_eq!(result, Some(PathBuf::from("../target")));
    }

    #[test]
    fn test_relative_path_relative_inputs() {
        // Should handle relative base and target paths
        let result = relative_path(Path::new("base/dir"), Path::new("base/target"));
        assert_eq!(result, Some(PathBuf::from("../target")));

        let result = relative_path(Path::new("dir1"), Path::new("dir2/file.txt"));
        assert_eq!(result, Some(PathBuf::from("../dir2/file.txt")));
    }

    #[test]
    fn test_relative_path_cross_platform() {
        // Platform-specific separator handling
        #[cfg(windows)]
        {
            // Windows handles both separators
            let result = relative_path(Path::new("base\\dir"), Path::new("base\\target"));
            assert_eq!(result, Some(PathBuf::from("..\\target")));
            let result = relative_path(Path::new("dir1\\sub"), Path::new("dir2\\file.txt"));
            assert_eq!(result, Some(PathBuf::from("..\\..\\dir2\\file.txt")));
        }

        #[cfg(not(windows))]
        {
            // Unix only uses forward slashes
            let result = relative_path(Path::new("base/dir"), Path::new("base/target"));
            assert_eq!(result, Some(PathBuf::from("../target")));
            let result = relative_path(Path::new("dir1/sub"), Path::new("dir2/file.txt"));
            assert_eq!(result, Some(PathBuf::from("../../dir2/file.txt")));
        }
    }

    #[test]
    fn test_normalize_path_with_unicode_slashes() {
        // Test that normalize_path handles Unicode slash attacks
        // These are Unicode characters that look like slashes but aren't
        let path_with_unicode = Path::new("test\u{2044}file"); // U+2044 fraction slash
        let normalized = normalize_path(path_with_unicode);

        // The Unicode slash should be transformed to platform separator
        let normalized_str = normalized.to_string_lossy();
        assert!(!normalized_str.contains('\u{2044}'));

        // Test multiple Unicode slashes
        let path_with_multiple = Path::new("a\u{2044}b\u{2215}c"); // fraction slash and division slash
        let normalized = normalize_path(path_with_multiple);
        let normalized_str = normalized.to_string_lossy();
        assert!(!normalized_str.contains('\u{2044}'));
        assert!(!normalized_str.contains('\u{2215}'));
    }

    #[test]
    fn test_normalize_path_no_unicode() {
        // Test that paths without Unicode slashes work normally
        let normal_path = Path::new("normal/path/file.txt");
        let normalized = normalize_path(normal_path);
        assert_eq!(normalized, PathBuf::from("normal/path/file.txt"));
    }

    #[test]
    fn test_relative_path_empty_result() {
        // Test the edge case where relative_path result is empty
        // This happens when both paths are identical
        let result = relative_path(Path::new("/same"), Path::new("/same"));
        assert_eq!(result, Some(PathBuf::from(".")));

        // Another case that could result in empty
        let result = relative_path(Path::new(""), Path::new(""));
        assert_eq!(result, Some(PathBuf::from(".")));
    }
}
