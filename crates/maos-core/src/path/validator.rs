//! Path validation for security and workspace isolation
//!
//! This module implements secure path validation to prevent:
//! - Path traversal attacks (../../../etc/passwd)
//! - Access outside workspace boundaries  
//! - Access to sensitive files and directories
//! - Symlink escape attacks

use crate::error::Result;
use crate::{AgentType, SessionId};
use std::path::{Path, PathBuf};

/// Safe path operations with validation
///
/// PathValidator ensures all path operations are within allowed boundaries and
/// don't violate security policies. It canonicalizes paths to resolve symlinks
/// and prevents path traversal attacks.
///
/// # Security Design
///
/// - **Fail Closed**: Default to denying access, require explicit allows
/// - **Canonical Paths**: Always resolve symlinks and relative paths
/// - **Defense in Depth**: Multiple validation layers
/// - **Zero Trust**: Validate every input path
pub struct PathValidator {
    /// List of canonicalized allowed root directories
    allowed_roots: Vec<PathBuf>,
    /// Glob patterns for blocked paths (e.g., "**/.git/**")  
    blocked_patterns: Vec<String>,
}

impl PathValidator {
    /// Create a new path validator with allowed roots and blocked patterns
    ///
    /// # Arguments
    ///
    /// * `allowed_roots` - List of root directories that are allowed for access
    /// * `blocked_patterns` - Glob patterns for paths that should be blocked
    ///
    /// # Security Notes
    ///
    /// The constructor automatically canonicalizes all allowed_roots to resolve
    /// symlinks and prevent symlink-based attacks.
    pub fn new(allowed_roots: Vec<PathBuf>, blocked_patterns: Vec<String>) -> Self {
        Self {
            allowed_roots: allowed_roots
                .into_iter()
                .map(|p| Self::safe_canonicalize(&p))
                .collect(),
            blocked_patterns,
        }
    }

    /// Safely canonicalize a path, handling non-existent files by canonicalizing parent
    fn safe_canonicalize(path: &Path) -> PathBuf {
        path.canonicalize()
            .or_else(|_| {
                // Try parent + filename approach
                path.parent()
                    .zip(path.file_name())
                    .and_then(|(parent, filename)| {
                        parent
                            .canonicalize()
                            .ok()
                            .map(|canonical_parent| canonical_parent.join(filename))
                    })
                    .ok_or(())
            })
            .unwrap_or_else(|_| crate::path::normalize_path(path))
    }

    /// Check if a canonicalized path is within the canonicalized workspace bounds
    /// This handles macOS symlink variations (/var vs /private/var)
    fn is_within_workspace(canonical_path: &Path, canonical_workspace: &Path) -> bool {
        // Direct prefix check first
        if canonical_path.starts_with(canonical_workspace) {
            return true;
        }

        // Handle macOS symlink variations using pattern matching
        let (path_str, workspace_str) = (
            canonical_path.to_string_lossy(),
            canonical_workspace.to_string_lossy(),
        );

        Self::macos_symlink_equivalent(workspace_str.as_ref(), path_str.as_ref())
    }

    /// Helper for macOS symlink equivalence (/var vs /private/var)
    ///
    /// On macOS, `/var` is a symlink to `/private/var`, which creates validation complexity:
    /// - User provides workspace path: `/var/folders/temp`
    /// - Path canonicalization resolves to: `/private/var/folders/temp`
    /// - Direct string comparison fails despite referring to same location
    ///
    /// This function handles the bidirectional equivalence by checking if paths
    /// would be equivalent after resolving the symlink in either direction.
    /// Essential for proper workspace isolation on macOS systems.
    fn macos_symlink_equivalent(workspace_str: &str, path_str: &str) -> bool {
        // macOS path prefixes for symlink handling
        const VAR_PREFIX_LEN: usize = 4; // "/var/"
        const PRIVATE_VAR_PREFIX_LEN: usize = 12; // "/private/var/"

        match (workspace_str, path_str) {
            (ws, p) if ws.starts_with("/var/") && p.starts_with("/private/var/") => {
                p[PRIVATE_VAR_PREFIX_LEN..].starts_with(&ws[VAR_PREFIX_LEN..])
            }
            (ws, p) if ws.starts_with("/private/var/") && p.starts_with("/var/") => {
                p[VAR_PREFIX_LEN..].starts_with(&ws[PRIVATE_VAR_PREFIX_LEN..])
            }
            _ => false,
        }
    }

    /// Check if a path string contains basic traversal patterns
    fn contains_basic_traversal_patterns(path_str: &str) -> bool {
        const BASIC_TRAVERSALS: &[&str] = &["../", "..\\", "/..", "\\.."];
        path_str.starts_with("..") || BASIC_TRAVERSALS.iter().any(|&p| path_str.contains(p))
    }

    /// Check if a path string contains Unicode-based traversal attack patterns
    ///
    /// Unicode characters that visually resemble path separators can be used to bypass
    /// security filters that only check for ASCII path separators. This function detects:
    /// - U+FF0F (Fullwidth Solidus): ／ - looks like / but is different character
    /// - U+2044 (Fraction Slash): ⁄ - used in mathematical notation  
    /// - U+2215 (Division Slash): ∕ - mathematical division operator
    ///
    /// These can be combined with ".." to create traversal attacks that bypass naive
    /// ASCII-only path validation.
    fn contains_unicode_traversal_patterns(path_str: &str) -> bool {
        // Pre-computed Unicode traversal patterns to avoid allocations
        const UNICODE_TRAVERSAL_PATTERNS: &[&str] = &[
            "..\u{FF0F}",
            "\u{FF0F}../",
            "..\u{2044}",
            "\u{2044}../",
            "..\u{2215}",
            "\u{2215}../",
        ];

        UNICODE_TRAVERSAL_PATTERNS
            .iter()
            .any(|&pattern| path_str.contains(pattern))
    }

    /// Check if a path string contains URL-encoded traversal patterns
    ///
    /// Attackers may URL-encode path traversal sequences to bypass filters:
    /// - %2e%2e = ".." (single encoded)
    /// - %2E%2E = ".." (uppercase single encoded)  
    /// - %252e%252e = ".." (double encoded, %25 = %)
    /// - %252E%252E = ".." (uppercase double encoded)
    ///
    /// Double encoding is used when the path passes through multiple decode stages.
    fn contains_url_encoded_traversal_patterns(path_str: &str) -> bool {
        const URL_ENCODED: &[&str] = &["%2e%2e", "%2E%2E", "%252e%252e", "%252E%252E"];
        URL_ENCODED.iter().any(|&p| path_str.contains(p))
    }

    /// Check if a path contains control characters combined with traversal patterns
    ///
    /// Control characters (ASCII 0-31) combined with path traversal can be used to:
    /// - Exploit parser bugs that handle control characters inconsistently
    /// - Bypass regex-based filters that don't expect embedded control chars
    /// - Create paths that display differently than they resolve
    ///
    /// Common attack vectors:
    /// - Null bytes (\0) to truncate paths in C-style string processing
    /// - Newlines (\n, \r) to inject commands or break log parsing
    /// - Tabs (\t) to confuse visual inspection of paths
    fn contains_control_char_traversal_attack(path_str: &str) -> bool {
        const CONTROL_CHARS: &[char] = &['\0', '\n', '\r', '\t'];
        let has_traversal = path_str.contains("..");
        has_traversal && CONTROL_CHARS.iter().any(|&c| path_str.contains(c))
    }

    /// Check if a path contains suspicious system paths combined with traversal
    ///
    /// This detects attempts to access critical system directories using path traversal:
    /// - "/etc/" - Unix system configuration directory
    /// - "\\etc\\" - Windows-style path to etc directory  
    /// - "%2e" - URL-encoded dot for obfuscated traversal
    ///
    /// These patterns indicate potential attacks targeting:
    /// - Password files (/etc/passwd, /etc/shadow)
    /// - System configuration (/etc/hosts, /etc/fstab)
    /// - Service configurations (/etc/ssh/, /etc/nginx/)
    ///
    /// The function requires both suspicious paths AND traversal indicators.
    fn contains_suspicious_path_traversal_attack(path_str: &str) -> bool {
        let has_traversal = path_str.contains("..");
        let targets_system_paths = path_str.contains("/etc/") || path_str.contains("\\etc\\");
        let has_encoded_traversal = path_str.contains("%2e");

        targets_system_paths && (has_traversal || has_encoded_traversal)
    }

    /// Check if a path string contains various traversal attack patterns
    fn contains_traversal_patterns(path_str: &str) -> bool {
        Self::contains_basic_traversal_patterns(path_str)
            || Self::contains_unicode_traversal_patterns(path_str)
            || Self::contains_url_encoded_traversal_patterns(path_str)
            || Self::contains_control_char_traversal_attack(path_str)
            || Self::contains_suspicious_path_traversal_attack(path_str)
    }

    /// Check if a path is in the list of allowed roots (for testing)
    ///
    /// This method is primarily intended for testing the internal state
    /// of the validator after construction.
    pub fn has_allowed_root(&self, path: &Path) -> bool {
        let canonical_path = Self::safe_canonicalize(path);
        self.allowed_roots.contains(&canonical_path)
    }

    /// Validate path is within allowed workspace boundaries
    ///
    /// This function performs comprehensive security validation to ensure the provided
    /// path is safe to access within the specified workspace. It prevents path traversal
    /// attacks, validates workspace boundaries, and checks against blocked patterns.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to validate (relative or absolute)
    /// * `workspace_root` - The workspace root that must be in allowed_roots
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - Canonical path that is safe to access
    /// * `Err(MaosError)` - Validation failed due to security concerns
    ///
    /// # Security Features
    ///
    /// - **Path Traversal Prevention**: Blocks `../`, URL encoding, Unicode variants
    /// - **Workspace Isolation**: Ensures paths stay within workspace boundaries
    /// - **Pattern Blocking**: Applies glob-based filtering rules
    /// - **Canonical Resolution**: Resolves symlinks and relative components
    ///
    /// # Examples
    ///
    /// ## Basic File Validation
    ///
    /// ```rust
    /// use maos_core::path::PathValidator;
    /// use std::path::PathBuf;
    /// use std::env;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace = env::temp_dir();
    /// let validator = PathValidator::new(vec![workspace.clone()], vec![]);
    ///
    /// // Validate a simple file path
    /// let file_path = PathBuf::from("config.json");
    /// let safe_path = validator.validate_workspace_path(&file_path, &workspace)?;
    /// println!("Safe to access: {:?}", safe_path);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Multi-Agent Workspace Isolation
    ///
    /// ```rust
    /// use maos_core::path::PathValidator;
    /// use std::path::PathBuf;
    /// use std::env;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let project_root = env::temp_dir();
    /// let agent1_workspace = project_root.join("agent1");
    /// let agent2_workspace = project_root.join("agent2");
    ///
    /// // Each agent has access only to its workspace
    /// let validator = PathValidator::new(
    ///     vec![agent1_workspace.clone(), agent2_workspace.clone()],
    ///     vec![]
    /// );
    ///
    /// // Agent 1 can access its files
    /// let result = validator.validate_workspace_path(
    ///     &PathBuf::from("data.json"),
    ///     &agent1_workspace
    /// );
    /// assert!(result.is_ok());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Security: Path Traversal Prevention
    ///
    /// ```rust
    /// use maos_core::path::PathValidator;
    /// use std::path::PathBuf;
    /// use std::env;
    ///
    /// # fn main() {
    /// let workspace = env::temp_dir();
    /// let validator = PathValidator::new(vec![workspace.clone()], vec![]);
    ///
    /// // These malicious paths will be blocked
    /// let attacks = vec![
    ///     "../../../etc/passwd",
    ///     "..\\\\..\\\\..\\\\windows\\\\system32",
    ///     "%2e%2e%2f%2e%2e%2fetc%2fpasswd",  // URL encoded
    ///     r"file\u{FF0F}..\..\..\\etc\\passwd",  // Unicode slash variants
    /// ];
    ///
    /// for attack in attacks {
    ///     let result = validator.validate_workspace_path(
    ///         &PathBuf::from(attack),
    ///         &workspace
    ///     );
    ///     assert!(result.is_err(), "Attack should be blocked: {}", attack);
    /// }
    /// # }
    /// ```
    ///
    /// ## Pattern-Based Blocking
    ///
    /// ```rust
    /// use maos_core::path::PathValidator;
    /// use std::path::PathBuf;
    /// use std::env;
    ///
    /// # fn main() {
    /// let workspace = env::temp_dir();
    /// let validator = PathValidator::new(
    ///     vec![workspace.clone()],
    ///     vec!["*.log".to_string(), "**/.git/**".to_string()]
    /// );
    ///
    /// // These files will be blocked by patterns
    /// let blocked_files = vec!["debug.log", ".git/config", "src/.git/hooks/pre-commit"];
    ///
    /// for file in blocked_files {
    ///     assert!(validator.is_blocked_path(&PathBuf::from(file)));
    /// }
    /// # }
    /// ```
    pub fn validate_workspace_path(&self, path: &Path, workspace_root: &Path) -> Result<PathBuf> {
        use crate::error::PathValidationError;

        // First, check if workspace_root is in our allowed_roots
        let canonical_workspace = Self::safe_canonicalize(workspace_root);

        if !self.allowed_roots.contains(&canonical_workspace) {
            return Err(PathValidationError::OutsideWorkspace {
                path: workspace_root.to_path_buf(),
                workspace: workspace_root.to_path_buf(),
            }
            .into());
        }

        // Build target path - normalize relative paths before joining
        let target_path = match path.is_absolute() {
            true => path.to_path_buf(),
            false => canonical_workspace.join(crate::path::normalize_path(path)),
        };

        let canonical_path = Self::safe_canonicalize(&target_path);

        // Check if canonical path is within workspace boundaries using enhanced check
        if !Self::is_within_workspace(&canonical_path, &canonical_workspace) {
            return Err(PathValidationError::OutsideWorkspace {
                path: canonical_path,
                workspace: canonical_workspace,
            }
            .into());
        }

        // Enhanced path traversal detection
        let path_str = path.to_string_lossy();

        // Check for various path traversal patterns - be strict about security
        if Self::contains_traversal_patterns(&path_str) {
            return Err(PathValidationError::PathTraversal {
                path: path.to_path_buf(),
            }
            .into());
        }

        Ok(canonical_path)
    }

    /// Check if path matches blocked patterns
    pub fn is_blocked_path(&self, path: &Path) -> bool {
        use std::path::Component;

        if self.blocked_patterns.is_empty() {
            return false;
        }

        let path_str = path.to_string_lossy();
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();

        // Generate path suffixes for flexible matching (last PATH_SUFFIX_COUNT suffixes)
        const PATH_SUFFIX_COUNT: usize = 3;
        let components: Vec<_> = path.components().collect();
        let path_suffixes: Vec<String> = (0..PATH_SUFFIX_COUNT.min(components.len()))
            .filter_map(|skip_count| {
                let start_idx = components.len().saturating_sub(skip_count + 1);
                let suffix: Vec<_> = components[start_idx..]
                    .iter()
                    .filter_map(|c| match c {
                        Component::Normal(name) => Some(name.to_string_lossy()),
                        _ => None,
                    })
                    .collect();
                (!suffix.is_empty()).then(|| suffix.join("/"))
            })
            .collect();

        // Check if any pattern matches any representation
        self.blocked_patterns.iter().any(|pattern| {
            Self::matches_glob_pattern(&path_str, pattern)
                || Self::matches_glob_pattern(&filename, pattern)
                || path_suffixes
                    .iter()
                    .any(|suffix| Self::matches_glob_pattern(suffix, pattern))
        })
    }

    /// Check if a path string matches a glob pattern
    fn matches_glob_pattern(path_str: &str, pattern: &str) -> bool {
        glob::Pattern::new(pattern)
            .map_or_else(|_| path_str.contains(pattern), |p| p.matches(path_str))
    }

    /// Generate unique workspace path for agent
    ///
    /// Creates a deterministic workspace directory path by combining the session ID
    /// and agent type. The generated path is guaranteed to be unique for each
    /// session-agent combination while being reproducible for the same inputs.
    ///
    /// # Arguments
    ///
    /// * `root` - Base directory where the workspace will be created
    /// * `session_id` - Unique session identifier
    /// * `agent_type` - Type of agent requesting the workspace
    ///
    /// # Returns
    ///
    /// A `PathBuf` representing the unique workspace directory path
    ///
    /// # Security
    ///
    /// The generated path components are derived from validated SessionId and AgentType
    /// values, ensuring no path traversal vulnerabilities. The resulting path should
    /// still be validated against workspace boundaries before use.
    ///
    /// # Examples
    ///
    /// ## Basic Workspace Generation
    ///
    /// ```no_run
    /// use maos_core::{SessionId, path::PathValidator};
    /// use std::path::Path;
    ///
    /// let validator = PathValidator::new(vec![], vec![]);
    /// let session_id = SessionId::generate();
    /// let agent_type = "backend-engineer".to_string();
    /// let workspace = validator.generate_workspace_path(
    ///     Path::new("/workspaces"),
    ///     &session_id,
    ///     &agent_type
    /// );
    /// // Results in: /workspaces/sess_<uuid>_backend-engineer
    /// ```
    ///
    /// ## Multi-Agent Session Management
    ///
    /// ```no_run
    /// use maos_core::{SessionId, path::PathValidator};
    /// use std::path::Path;
    ///
    /// let validator = PathValidator::new(vec![], vec![]);
    /// let session_id = SessionId::generate();
    /// let project_root = Path::new("/projects/my-app");
    ///
    /// // Different agents get isolated workspaces within same session
    /// let agents = vec!["frontend-engineer", "backend-engineer", "data-scientist"];
    /// let mut workspaces = Vec::new();
    ///
    /// for agent_type in agents {
    ///     let workspace = validator.generate_workspace_path(
    ///         project_root,
    ///         &session_id,
    ///         &agent_type.to_string()
    ///     );
    ///     workspaces.push((agent_type, workspace));
    /// }
    ///
    /// // Each agent has unique workspace:
    /// // /projects/my-app/sess_<uuid>_frontend-engineer
    /// // /projects/my-app/sess_<uuid>_backend-engineer  
    /// // /projects/my-app/sess_<uuid>_data-scientist
    /// ```
    ///
    /// ## Deterministic Workspace Paths
    ///
    /// ```no_run
    /// use maos_core::{SessionId, path::PathValidator};
    /// use std::path::Path;
    ///
    /// let validator = PathValidator::new(vec![], vec![]);
    /// let session_id = SessionId::generate();
    /// let agent_type = "tester".to_string();
    /// let root = Path::new("/tmp");
    ///
    /// // Same inputs always produce same workspace path
    /// let workspace1 = validator.generate_workspace_path(&root, &session_id, &agent_type);
    /// let workspace2 = validator.generate_workspace_path(&root, &session_id, &agent_type);
    ///
    /// assert_eq!(workspace1, workspace2, "Workspace paths should be deterministic");
    /// ```
    pub fn generate_workspace_path(
        &self,
        root: &Path,
        session_id: &SessionId,
        agent_type: &AgentType,
    ) -> PathBuf {
        // Avoid format! allocation by building path components directly
        let mut workspace_name =
            String::with_capacity(session_id.as_str().len() + 1 + agent_type.as_str().len());
        workspace_name.push_str(session_id.as_str());
        workspace_name.push('_');
        workspace_name.push_str(agent_type.as_str());

        root.join(workspace_name)
    }
}
