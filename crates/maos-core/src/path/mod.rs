//! Path utilities and validation for MAOS
//!
//! This module provides secure path operations with comprehensive validation
//! to prevent path traversal attacks and ensure workspace isolation in multi-agent
//! environments.
//!
//! # Overview
//!
//! The path utilities are designed around three core components:
//! - **[`PathValidator`]**: Secure path validation with workspace isolation
//! - **Path normalization**: Cross-platform path cleaning and standardization  
//! - **Path utilities**: Safe comparison, relative path calculation, and more
//!
//! # Quick Start
//!
//! ```rust
//! use maos_core::path::{PathValidator, normalize_path, paths_equal};
//! use std::path::PathBuf;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a validator with allowed workspace roots
//! let workspace = PathBuf::from("/tmp/my_workspace");
//! let validator = PathValidator::new(
//!     vec![workspace.clone()],
//!     vec!["*.tmp".to_string(), "**/.git/**".to_string()]
//! );
//!
//! // Validate file access within workspace
//! let file_path = PathBuf::from("src/main.rs");
//! let safe_path = validator.validate_workspace_path(&file_path, &workspace)?;
//! println!("Safe path: {:?}", safe_path);
//!
//! // Normalize paths for consistent handling
//! let messy_path = PathBuf::from("./dir/../file.txt");
//! let clean_path = normalize_path(&messy_path);
//! assert_eq!(clean_path, PathBuf::from("file.txt"));
//! # Ok(())
//! # }
//! ```
//!
//! # Security Model
//!
//! All path operations are designed with **security-first principles**:
//!
//! ## Defense in Depth
//! - **Path canonicalization**: Resolves symlinks and relative components
//! - **Traversal detection**: Blocks `../`, URL encoding, Unicode variants
//! - **Workspace isolation**: Ensures paths stay within allowed boundaries
//! - **Pattern blocking**: Glob-based filtering of sensitive files/directories
//!
//! ## Fail Closed
//! - Default to denying access
//! - Require explicit allow lists for workspace roots
//! - Reject suspicious patterns even if they might be legitimate
//!
//! ## Zero Trust
//! - Validate every input path
//! - Never trust user-provided paths
//! - Always canonicalize before making security decisions
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │  Raw User Path  │───▶│  Path Validator  │───▶│  Safe Canonical │
//! │  "src/../etc"   │    │                  │    │  Path           │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//!                               │
//!                               ▼
//!                        ┌──────────────────┐
//!                        │  Security Checks │
//!                        │  • Traversal     │
//!                        │  • Workspace     │
//!                        │  • Patterns      │
//!                        └──────────────────┘
//! ```
//!
//! # Common Patterns
//!
//! ## Multi-Agent Workspace Isolation
//!
//! ```rust
//! use maos_core::path::PathValidator;
//! use maos_core::{SessionId, AgentType};
//! use std::path::PathBuf;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let project_root = PathBuf::from("/projects");
//! let session_id = SessionId::generate();
//! let agent_type: AgentType = "backend-engineer".to_string();
//!
//! // Create isolated workspace for this agent
//! let validator = PathValidator::new(vec![project_root.clone()], vec![
//!     "*.log".to_string(),
//!     "**/.git/**".to_string(),
//!     "**/node_modules/**".to_string(),
//! ]);
//!
//! let workspace = validator.generate_workspace_path(
//!     &project_root,
//!     &session_id,
//!     &agent_type
//! );
//! println!("Agent workspace: {:?}", workspace);
//! # Ok(())
//! # }
//! ```
//!
//! ## Cross-Platform Path Handling
//!
//! ```rust
//! use maos_core::path::{normalize_path, paths_equal};
//! use std::path::PathBuf;
//!
//! // Platform-specific separator handling
//! #[cfg(windows)]
//! {
//!     let windows_path = PathBuf::from("src\\components\\ui.tsx");
//!     let forward_path = PathBuf::from("src/components/ui.tsx");
//!     // Windows treats both as equivalent
//!     assert!(paths_equal(&windows_path, &forward_path));
//! }
//!
//! #[cfg(not(windows))]
//! {
//!     let unix_path = PathBuf::from("src/components/ui.tsx");
//!     let same_path = PathBuf::from("src/components/ui.tsx");
//!     // Unix paths with forward slashes
//!     assert!(paths_equal(&unix_path, &same_path));
//! }
//!
//! // Complex traversals are resolved consistently
//! let complex = PathBuf::from("./src/../lib/./utils/../mod.rs");
//! let simple = PathBuf::from("lib/mod.rs");
//! assert_eq!(normalize_path(&complex), simple);
//! ```
//!
//! # Error Handling
//!
//! Path validation errors provide detailed context while avoiding information
//! leakage:
//!
//! ```rust
//! use maos_core::path::PathValidator;
//! use maos_core::error::{PathValidationError, MaosError};
//! use std::path::PathBuf;
//!
//! # fn example() {
//! let validator = PathValidator::new(vec![PathBuf::from("/workspace")], vec![]);
//! let workspace = PathBuf::from("/workspace");
//!
//! match validator.validate_workspace_path(&PathBuf::from("../etc/passwd"), &workspace) {
//!     Ok(safe_path) => println!("Safe: {:?}", safe_path),
//!     Err(err) => {
//!         match &err {
//!             MaosError::PathValidation(path_err) => {
//!                 match path_err {
//!                     PathValidationError::PathTraversal { path } => {
//!                         eprintln!("Blocked traversal attempt: {:?}", path);
//!                     },
//!                     PathValidationError::OutsideWorkspace { path, workspace } => {
//!                         eprintln!("Path {:?} outside workspace {:?}", path, workspace);
//!                     },
//!                     _ => eprintln!("Path validation failed: {:?}", path_err),
//!                 }
//!             },
//!             _ => eprintln!("Path validation failed: {}", err),
//!         }
//!     }
//! }
//! # }

pub mod utils;
pub mod validator;

// Re-export main types for convenience
pub use utils::{normalize_path, paths_equal, relative_path};
pub use validator::PathValidator;
