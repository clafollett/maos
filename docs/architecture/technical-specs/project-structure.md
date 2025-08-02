# MAOS Project Structure and Migration Plan

## Overview

This document defines the new Rust-based MAOS project structure and provides a detailed migration path from the current Python/TypeScript implementation.

## New Project Structure

```
maos/
├── .claude/                    # Agent definitions (preserved)
│   ├── agents/                 # Individual agent configs
│   └── MAOS_SYSTEM.md          # System documentation
│
├── maos-cli/                   # Main CLI application
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── cli/                # Command implementations
│   │   ├── core/               # Business logic
│   │   └── utils/              # Utilities
│   └── tests/
│
├── maos-hooks/                 # Hook system
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs             # Standalone hook binary
│   │   ├── security/           # Security implementation
│   │   └── audit/              # Audit logging
│   └── tests/
│
├── maos-coordination/          # Coordination library
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── communication/      # Message passing
│   │   ├── locking/            # Lock management
│   │   └── tracking/           # Progress tracking
│   └── tests/
│
├── maos-common/                # Shared types and utilities
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── types/              # Common types
│   │   ├── config/             # Configuration
│   │   └── git/                # Git operations
│   └── tests/
│
├── maos-integration/           # Integration with external systems
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── github/             # GitHub integration
│   │   ├── claude/             # Claude Code integration
│   │   └── vcs/                # Version control
│   └── tests/
│
├── docs/                       # Documentation
│   ├── architecture/           # Architecture docs
│   │   ├── decisions/          # ADRs
│   │   ├── requirements/       # PRDs
│   │   └── technical-specs/    # Technical specifications
│   ├── user-guide/             # User documentation
│   └── api/                    # API documentation
│
├── scripts/                    # Build and deployment scripts
│   ├── install.sh              # Installation script
│   ├── migrate.sh              # Migration script
│   └── release.sh              # Release script
│
├── tests/                      # Integration tests
│   ├── integration/
│   ├── e2e/
│   └── fixtures/
│
├── examples/                   # Example configurations
│   ├── hooks/                  # Hook examples
│   ├── workflows/              # Workflow examples
│   └── policies/               # Security policies
│
├── Cargo.toml                  # Workspace configuration
├── Cargo.lock
├── rust-toolchain.toml         # Rust version
├── .gitignore
├── LICENSE
└── README.md
```

## Workspace Configuration

### Root Cargo.toml

```toml
[workspace]
members = [
    "maos-cli",
    "maos-hooks",
    "maos-coordination",
    "maos-common",
    "maos-integration",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
authors = ["MAOS Development Team"]
edition = "2021"
license = "MIT"
repository = "https://github.com/clafollett/maos"

[workspace.dependencies]
# Common dependencies
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.40", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4.5", features = ["derive"] }
uuid = { version = "1.10", features = ["v4", "serde"] }

# Workspace crates
maos-common = { path = "maos-common" }
maos-hooks = { path = "maos-hooks" }
maos-coordination = { path = "maos-coordination" }
maos-integration = { path = "maos-integration" }

[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true
panic = "abort"

[profile.dev]
opt-level = 0
debug = true

[profile.bench]
debug = true
```

## Migration Plan

### Phase 1: Setup New Structure (Week 1)

1. **Initialize Rust workspace**
   ```bash
   # Create workspace structure
   mkdir -p maos-{cli,hooks,coordination,common,integration}/{src,tests}
   
   # Initialize Cargo.toml files
   cargo init --lib maos-common
   cargo init --lib maos-coordination
   cargo init --lib maos-integration
   cargo init --lib maos-hooks
   cargo init --bin maos-cli
   ```

2. **Preserve existing assets**
   - Keep `.claude/` directory unchanged
   - Move Python hooks to `examples/hooks/python/` for reference
   - Archive current implementation in `legacy/` branch

3. **Setup development environment**
   ```bash
   # Create development setup script
   cat > scripts/dev-setup.sh << 'EOF'
   #!/bin/bash
   rustup update stable
   rustup component add rustfmt clippy
   cargo install cargo-watch cargo-audit cargo-outdated
   EOF
   ```

### Phase 2: Core Implementation (Week 2-3)

1. **Implement maos-common**
   - Define shared types (AgentId, TaskId, etc.)
   - Configuration management
   - Git operations wrapper

2. **Implement maos-hooks**
   - Port security policies from Python
   - Implement validation engine
   - Add audit logging

3. **Implement maos-coordination**
   - File-based communication
   - Lock management
   - Progress tracking

### Phase 3: CLI Development (Week 3-4)

1. **Basic CLI structure**
   ```rust
   // Start with core commands
   maos worktree create/list/switch
   maos agent spawn/list
   maos session start/status
   ```

2. **Integration with existing Claude Code**
   - Environment detection
   - Hook registration
   - Agent communication

### Phase 4: Testing and Migration (Week 4-5)

1. **Comprehensive testing**
   - Unit tests for each crate
   - Integration tests
   - Performance benchmarks

2. **Migration tooling**
   ```bash
   # Migration script
   scripts/migrate.sh --preserve-agents --archive-legacy
   ```

3. **Documentation updates**
   - Update READMEs
   - API documentation
   - Migration guide

## Compatibility Bridge

### Python Hook Adapter

During migration, support existing Python hooks:

```rust
// maos-hooks/src/adapters/python.rs
pub struct PythonHookAdapter {
    script_path: PathBuf,
    python_exe: PathBuf,
}

impl HookAdapter for PythonHookAdapter {
    async fn execute(&self, event: HookEvent) -> Result<HookResponse> {
        let input = serde_json::to_string(&event)?;
        
        let output = Command::new(&self.python_exe)
            .arg(&self.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;
        
        let response: HookResponse = serde_json::from_slice(&output.stdout)?;
        Ok(response)
    }
}
```

### Configuration Migration

```rust
// maos-common/src/config/migration.rs
pub fn migrate_claude_config() -> Result<MaosConfig> {
    let claude_config_path = dirs::home_dir()
        .unwrap()
        .join(".claude/config.json");
    
    if claude_config_path.exists() {
        let claude_config: ClaudeConfig = serde_json::from_reader(
            File::open(claude_config_path)?
        )?;
        
        // Convert to MAOS config
        Ok(MaosConfig {
            default_policy: claude_config.security_policy,
            workspace_base: claude_config.workspace_path,
            github_token: claude_config.github_token,
            ..Default::default()
        })
    } else {
        Ok(MaosConfig::default())
    }
}
```

## File Organization Standards

### Source Code Organization

```rust
// Each module should have clear responsibilities
// Example: maos-cli/src/cli/worktree.rs

mod create;
mod list;
mod switch;
mod cleanup;

pub use create::CreateCommand;
pub use list::ListCommand;
pub use switch::SwitchCommand;
pub use cleanup::CleanupCommand;

pub fn configure() -> Command {
    Command::new("worktree")
        .about("Manage git worktrees for parallel development")
        .subcommand(create::command())
        .subcommand(list::command())
        .subcommand(switch::command())
        .subcommand(cleanup::command())
}
```

### Test Organization

```rust
// Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_worktree_creation() {
        // Test implementation
    }
}

// Integration tests in tests/ directory
// tests/integration/worktree_test.rs
```

### Documentation Standards

```rust
//! # Module Documentation
//! 
//! This module provides worktree management functionality.
//! 
//! ## Examples
//! 
//! ```rust
//! use maos_cli::worktree;
//! 
//! let wt = worktree::create("issue-123")?;
//! ```

/// Creates a new worktree for the specified issue.
/// 
/// # Arguments
/// 
/// * `issue_id` - The GitHub issue identifier
/// 
/// # Returns
/// 
/// Returns the created `Worktree` instance or an error.
pub fn create(issue_id: &str) -> Result<Worktree> {
    // Implementation
}
```

## Build and Release Process

### Local Development

```bash
# Watch mode for development
cargo watch -x check -x test -x "run -- --help"

# Run with debug logging
RUST_LOG=debug cargo run -- worktree create issue-123
```

### CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check

  release:
    if: startsWith(github.ref, 'refs/tags/')
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - uses: softprops/action-gh-release@v1
        with:
          files: target/release/maos
```

### Installation Methods

1. **From source**
   ```bash
   git clone https://github.com/clafollett/maos
   cd maos
   cargo install --path maos-cli
   ```

2. **Pre-built binaries**
   ```bash
   curl -fsSL https://github.com/clafollett/maos/releases/latest/download/install.sh | sh
   ```

3. **Package managers** (future)
   ```bash
   # Homebrew
   brew install maos
   
   # Cargo
   cargo install maos-cli
   ```

## Backwards Compatibility

### Agent Definition Compatibility

The `.claude/agents/` directory structure remains unchanged, ensuring all existing agent definitions continue to work.

### Hook Compatibility Layer

```toml
# ~/.config/maos/config.toml
[hooks]
# Support for legacy Python hooks during transition
legacy_hooks_enabled = true
legacy_hooks_path = "~/.claude/hooks"

# New Rust hooks
rust_hooks_enabled = true
default_policy = "standard"
```

### Environment Variables

```bash
# Existing Claude Code variables (preserved)
CLAUDE_SESSION_ID
CLAUDE_AGENT_TYPE

# New MAOS variables
MAOS_WORKSPACE
MAOS_POLICY
MAOS_LOG_LEVEL
```

This structure provides a clean, maintainable foundation for MAOS while ensuring smooth migration from the current implementation.