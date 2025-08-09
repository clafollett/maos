//! Cross-platform path utilities with security-focused normalization
//!
//! This module provides safe path manipulation functions that handle cross-platform
//! differences while maintaining security properties.

use std::path::{Path, PathBuf};

/// Normalize a path by resolving `.` and `..` components safely
///
/// This function normalizes paths by:
/// - Converting all path separators to forward slashes (`/`)
/// - Resolving current directory (`.`) references  
/// - Resolving parent directory (`..`) references safely
/// - Handling URL-encoded and Unicode separator variants for security
/// - Preserving relative vs absolute path semantics
///
/// # Security Features
///
/// - Blocks Unicode separator variants that could bypass security checks
/// - Handles URL-encoded path separators (`%2F`, `%2f`)
/// - Prevents directory traversal beyond the root for absolute paths
/// - Maintains relative path relationships for proper validation
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
/// ## Cross-Platform Compatibility
///
/// ```rust
/// use maos_core::path::normalize_path;
/// use std::path::{Path, PathBuf};
///
/// // Windows-style separators are converted to Unix-style
/// let windows_path = Path::new("src\\components\\ui.tsx");
/// assert_eq!(normalize_path(windows_path), PathBuf::from("src/components/ui.tsx"));
///
/// // Mixed separators are handled consistently
/// let mixed_path = Path::new("src/dir\\subdir/file.txt");
/// assert_eq!(normalize_path(mixed_path), PathBuf::from("src/dir/subdir/file.txt"));
/// ```
///
/// ## Security Handling
///
/// ```rust
/// use maos_core::path::normalize_path;
/// use std::path::{Path, PathBuf};
///
/// // URL-encoded separators are decoded and normalized
/// let encoded_path = Path::new("src%2Fmain.rs");
/// assert_eq!(normalize_path(encoded_path), PathBuf::from("src/main.rs"));
///
/// // Unicode separator variants are normalized
/// let unicode_path = Path::new("src\u{FF0F}main.rs"); // Fullwidth solidus
/// assert_eq!(normalize_path(unicode_path), PathBuf::from("src/main.rs"));
/// ```
///
/// ## Absolute vs Relative Paths
///
/// ```rust
/// use maos_core::path::normalize_path;
/// use std::path::{Path, PathBuf};
///
/// // Absolute paths prevent traversal beyond root
/// let abs_path = Path::new("/tmp/../../../etc/passwd");
/// assert_eq!(normalize_path(abs_path), PathBuf::from("/etc/passwd"));
///
/// // Relative paths preserve .. when they can't be resolved
/// let rel_path = Path::new("../../../etc/passwd");
/// assert_eq!(normalize_path(rel_path), PathBuf::from("../../../etc/passwd"));
/// ```
pub fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;

    // Normalize Windows separators first, then Unicode attack vectors and URL-encoded separators
    const UNICODE_SLASHES: [char; 3] = ['\u{FF0F}', '\u{2044}', '\u{2215}'];
    let path_str = path
        .to_string_lossy()
        .replace('\\', "/") // Windows compatibility
        .replace(UNICODE_SLASHES, "/") // Security: Unicode attack vectors
        .replace("%2F", "/")
        .replace("%2f", "/");

    // Process components using fold for cleaner state management
    let (is_absolute, components) = Path::new(&path_str).components().fold(
        (false, Vec::new()),
        |(mut is_abs, mut comps), comp| {
            match comp {
                Component::Prefix(_) | Component::RootDir => {
                    is_abs = true;
                    comps.clear();
                }
                Component::CurDir => {} // Skip
                Component::ParentDir => {
                    match (comps.is_empty(), is_abs, comps.last()) {
                        (true, false, _) | (false, _, Some(Component::ParentDir)) => {
                            comps.push(comp);
                        }
                        (false, _, Some(_)) => {
                            comps.pop();
                        }
                        _ => {} // Ignore .. at absolute root
                    }
                }
                Component::Normal(_) => comps.push(comp),
            }
            (is_abs, comps)
        },
    );

    // Build result path using iterator chain
    std::iter::once(is_absolute.then_some(Component::RootDir))
        .flatten()
        .chain(components)
        .collect()
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
/// // Paths with different separators are equal after normalization
/// assert!(paths_equal(
///     Path::new("src\\main.rs"),  // Windows-style
///     Path::new("src/main.rs")    // Unix-style
/// ));
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
/// // Mixed separators are handled consistently
/// assert!(paths_equal(
///     Path::new("src/dir\\subdir/file.txt"),
///     Path::new("src\\dir/subdir\\file.txt")
/// ));
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
/// ```
///
/// ## Sibling Directories
///
/// ```rust
/// use maos_core::path::relative_path;
/// use std::path::{Path, PathBuf};
///
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
/// ```
///
/// ## Cross-Platform Usage
///
/// ```rust
/// use maos_core::path::relative_path;
/// use std::path::{Path, PathBuf};
///
/// // Works with mixed separators
/// assert_eq!(
///     relative_path(Path::new("src\\components"), Path::new("src/utils/helpers")),
///     Some(PathBuf::from("../utils/helpers"))
/// );
/// ```
///
/// ## Workspace-Relative File Access
///
/// ```rust
/// use maos_core::path::relative_path;
/// use std::path::{Path, PathBuf};
///
/// let workspace = Path::new("/projects/my-app");
/// let config_file = Path::new("/projects/my-app/config/app.toml");
/// let src_dir = Path::new("/projects/my-app/src");
///
/// // Get relative path from src to config
/// if let Some(rel_path) = relative_path(src_dir, config_file) {
///     println!("Config is at: {}", rel_path.display()); // "../config/app.toml"
/// }
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
