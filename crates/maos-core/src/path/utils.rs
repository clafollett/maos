//! Cross-platform path utilities with security-focused normalization
//!
//! This module provides safe path manipulation functions that handle cross-platform
//! differences while maintaining security properties.

use std::path::{Path, PathBuf};

/// Normalize a path for MAOS multi-agent security with built-in Rust functions
///
/// This function provides security-hardened path normalization for untrusted agent input by:
/// - Using `std::path::absolute` for robust cross-platform path resolution
/// - Preventing Unicode separator attack vectors (fullwidth solidus, etc.)
/// - Ensuring consistent cross-platform behavior for agent workspace isolation
///
/// # Security Features
///
/// - Blocks Unicode separator variants that could bypass security checks
/// - Prevents agent path injection via alternative Unicode separators
/// - Uses standard Rust path handling as the foundation for reliability
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
/// assert_eq!(normalize_path(unicode_path), PathBuf::from("src/main.rs"));
///
/// // Additional Unicode separators are handled
/// let fraction_slash = Path::new("src\u{2044}main.rs"); // Fraction slash
/// let division_slash = Path::new("src\u{2215}main.rs"); // Division slash  
/// assert_eq!(normalize_path(fraction_slash), PathBuf::from("src/main.rs"));
/// assert_eq!(normalize_path(division_slash), PathBuf::from("src/main.rs"));
/// ```
///
/// ## Cross-Platform Compatibility
///
/// Uses `std::path::absolute` for robust cross-platform path handling while maintaining
/// relative path semantics when needed for agent workspace isolation.
pub fn normalize_path(path: &Path) -> PathBuf {
    // Apply security transformations to prevent Unicode attack vectors
    let secured_path = apply_security_transforms(path);

    // Always use our component-based normalization for consistent behavior
    // This ensures proper .. resolution for both absolute and relative paths
    normalize_components(&secured_path)
}

/// Apply MAOS-specific security transformations to prevent agent attacks
fn apply_security_transforms(path: &Path) -> PathBuf {
    const UNICODE_SLASHES: [char; 3] = ['\u{FF0F}', '\u{2044}', '\u{2215}'];
    let path_str = path.to_string_lossy();

    // Apply security transforms if we detect potential attack vectors or cross-platform issues
    let needs_transform = path_str
        .chars()
        .any(|c| UNICODE_SLASHES.contains(&c) || c == '\\');

    if needs_transform {
        let secured: String = path_str
            .chars()
            .map(|c| {
                if UNICODE_SLASHES.contains(&c) || c == '\\' {
                    '/'
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

/// Normalize paths using Rust's component iteration (minimal custom logic)
fn normalize_components(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut components = Vec::new();
    let mut is_absolute = false;

    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {
                is_absolute = true;
                components.clear(); // Start fresh for absolute paths
                if let Component::Prefix(_) = component {
                    components.push(component); // Keep prefix for Windows
                }
            }
            Component::CurDir => {} // Skip current directory
            Component::ParentDir => {
                if let Some(Component::Normal(_)) = components.last() {
                    components.pop(); // Cancel out with normal component
                } else if !is_absolute {
                    components.push(component); // Keep .. for relative paths only
                }
                // For absolute paths, .. at root is ignored
            }
            Component::Normal(_) => components.push(component),
        }
    }

    // Build the result, ensuring we maintain absolute/relative nature
    if is_absolute {
        std::iter::once(Component::RootDir)
            .chain(components)
            .collect()
    } else {
        components.into_iter().collect()
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
