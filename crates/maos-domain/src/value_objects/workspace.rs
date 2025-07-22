use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Errors for Workspace operations
#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("Workspace path cannot be empty")]
    EmptyPath,
    #[error("Workspace path must be absolute")]
    RelativePath,
    #[error("Invalid slug format: {0}")]
    InvalidSlugFormat(String),
}

/// Workspace value object - provides consistent workspace identification
/// using Claude Code compatible path slugging for session isolation
///
/// Transforms paths like `/Users/clafollett/Repositories/maos`
/// into slugs like `Users-clafollett-Repositories-maos`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Workspace {
    slug: String,
    workspace_path: String,
}

impl Workspace {
    /// Create a new Workspace from an absolute path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, WorkspaceError> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        // Validate path is not empty
        if path_str.trim().is_empty() {
            return Err(WorkspaceError::EmptyPath);
        }

        // Validate path is absolute
        if !path.as_ref().is_absolute() {
            return Err(WorkspaceError::RelativePath);
        }

        // Generate slug from normalized path
        let normalized_path = Self::normalize_path(&path_str);
        let slug = Self::generate_slug(&normalized_path);

        Ok(Self {
            slug,
            workspace_path: normalized_path,
        })
    }

    /// Create Workspace from existing slug string (for deserialization)
    pub fn from_slug(slug: String, path: String) -> Result<Self, WorkspaceError> {
        if slug.trim().is_empty() {
            return Err(WorkspaceError::InvalidSlugFormat("Empty slug".to_string()));
        }

        if path.trim().is_empty() {
            return Err(WorkspaceError::EmptyPath);
        }

        // Basic validation - slug should not contain problematic characters
        if slug.contains('/') || slug.contains('\\') {
            return Err(WorkspaceError::InvalidSlugFormat(
                "Slug contains invalid path separators".to_string(),
            ));
        }

        Ok(Self {
            slug,
            workspace_path: path,
        })
    }

    /// Get the slug string
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Get the workspace path
    pub fn path(&self) -> &str {
        &self.workspace_path
    }

    /// Check if this slug represents the same workspace as another path
    pub fn matches_path<P: AsRef<Path>>(&self, path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy();
        let normalized = Self::normalize_path(&path_str);
        self.workspace_path == normalized
    }

    /// Generate a slug from path using Claude Code's approach
    /// `/Users/clafollett/Repositories/maos` -> `Users-clafollett-Repositories-maos`
    fn generate_slug(path: &str) -> String {
        // Remove leading slash and replace remaining slashes with hyphens
        path.strip_prefix('/').unwrap_or(path).replace('/', "-")
    }

    /// Normalize path for consistent slugging across platforms
    fn normalize_path(path: &str) -> String {
        // Convert backslashes to forward slashes and remove trailing slash
        path.replace('\\', "/").trim_end_matches('/').to_string()
    }
}

impl std::fmt::Display for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.slug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_workspace_creation_success() {
        let path = PathBuf::from("/Users/clafollett/Repositories/maos");
        let workspace = Workspace::from_path(&path).unwrap();

        assert_eq!(workspace.slug(), "Users-clafollett-Repositories-maos");
        assert_eq!(workspace.path(), "/Users/clafollett/Repositories/maos");
    }

    #[test]
    fn test_workspace_simple_path() {
        let path = PathBuf::from("/tmp/test/workspace");
        let workspace = Workspace::from_path(&path).unwrap();

        assert_eq!(workspace.slug(), "tmp-test-workspace");
        assert_eq!(workspace.path(), "/tmp/test/workspace");
    }

    #[test]
    fn test_workspace_empty_path() {
        let result = Workspace::from_path("");
        assert!(matches!(result, Err(WorkspaceError::EmptyPath)));
    }

    #[test]
    fn test_workspace_relative_path() {
        let result = Workspace::from_path("relative/path");
        assert!(matches!(result, Err(WorkspaceError::RelativePath)));
    }

    #[test]
    fn test_workspace_deterministic() {
        let path1 = PathBuf::from("/tmp/workspace");
        let path2 = PathBuf::from("/tmp/workspace");

        let workspace1 = Workspace::from_path(&path1).unwrap();
        let workspace2 = Workspace::from_path(&path2).unwrap();

        assert_eq!(workspace1.slug(), workspace2.slug());
        assert_eq!(workspace1, workspace2);
    }

    #[test]
    fn test_workspace_different_paths() {
        let path1 = PathBuf::from("/tmp/workspace1");
        let path2 = PathBuf::from("/tmp/workspace2");

        let workspace1 = Workspace::from_path(&path1).unwrap();
        let workspace2 = Workspace::from_path(&path2).unwrap();

        assert_eq!(workspace1.slug(), "tmp-workspace1");
        assert_eq!(workspace2.slug(), "tmp-workspace2");
        assert_ne!(workspace1.slug(), workspace2.slug());
        assert_ne!(workspace1, workspace2);
    }

    #[test]
    fn test_workspace_path_normalization() {
        let path1 = PathBuf::from("/tmp/workspace/");
        let path2 = PathBuf::from("/tmp/workspace");

        let workspace1 = Workspace::from_path(&path1).unwrap();
        let workspace2 = Workspace::from_path(&path2).unwrap();

        // Should be equal after normalization
        assert_eq!(workspace1, workspace2);
        assert_eq!(workspace1.path(), "/tmp/workspace");
        assert_eq!(workspace2.path(), "/tmp/workspace");
        assert_eq!(workspace1.slug(), "tmp-workspace");
    }

    #[test]
    fn test_workspace_matches_path() {
        let path = PathBuf::from("/tmp/workspace");
        let workspace = Workspace::from_path(&path).unwrap();

        assert!(workspace.matches_path("/tmp/workspace"));
        assert!(workspace.matches_path("/tmp/workspace/")); // normalized
        assert!(!workspace.matches_path("/tmp/other"));
    }

    #[test]
    fn test_workspace_from_slug_string() {
        let slug = "Users-clafollett-Repositories-maos".to_string();
        let path = "/Users/clafollett/Repositories/maos".to_string();

        let workspace = Workspace::from_slug(slug.clone(), path.clone()).unwrap();
        assert_eq!(workspace.slug(), &slug);
        assert_eq!(workspace.path(), &path);
    }

    #[test]
    fn test_workspace_from_invalid_slug_string() {
        let result = Workspace::from_slug("invalid/slug".to_string(), "/tmp".to_string());
        assert!(matches!(result, Err(WorkspaceError::InvalidSlugFormat(_))));
    }

    #[test]
    fn test_workspace_display() {
        let path = PathBuf::from("/tmp/workspace");
        let workspace = Workspace::from_path(&path).unwrap();

        let display_str = format!("{}", workspace);
        assert_eq!(display_str, workspace.slug());
        assert_eq!(display_str, "tmp-workspace");
    }

    #[test]
    fn test_workspace_serialization_compatibility() {
        let path = PathBuf::from("/Users/clafollett/Repositories/maos");
        let workspace = Workspace::from_path(&path).unwrap();

        // Test that it can be serialized/deserialized
        let json = serde_json::to_string(&workspace).unwrap();
        let deserialized: Workspace = serde_json::from_str(&json).unwrap();

        assert_eq!(workspace, deserialized);
        assert_eq!(deserialized.slug(), "Users-clafollett-Repositories-maos");
    }

    #[test]
    fn test_claude_code_compatibility() {
        // Test the exact case from the user's example
        let path = PathBuf::from("/Users/clafollett/Repositories/maos");
        let workspace = Workspace::from_path(&path).unwrap();

        // This should match Claude's format: -Users-clafollett-Repositories-maos
        // (though we omit the leading dash for cleaner folder names)
        assert_eq!(workspace.slug(), "Users-clafollett-Repositories-maos");
    }

    #[test]
    fn test_windows_path_handling() {
        // Simulate a Windows path (though we can't create one directly on Unix)
        let windows_style = "/C:/Users/user/Documents/project";
        let workspace = Workspace::from_path(windows_style).unwrap();

        assert_eq!(workspace.slug(), "C:-Users-user-Documents-project");
    }
}
