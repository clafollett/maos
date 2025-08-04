# MAOS Rust CLI Architecture

## Executive Summary

This document defines the technical architecture for MAOS (Multi-Agent Orchestration System) Rust CLI, which replaces Python hook scripts with a high-performance binary that integrates with Claude Code's hook system. The architecture prioritizes sub-10ms execution, zero runtime dependencies, and seamless Claude Code integration.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        MAOS Workspace                        │
├─────────────────────────────────────────────────────────────┤
│  maos/ (root crate - workspace configuration)               │
│  ├── Cargo.toml (workspace manifest)                        │
│  ├── crates/                                                │
│  │   ├── maos-cli/        # Binary entry point             │
│  │   ├── maos-core/       # Core business logic            │
│  │   ├── maos-security/   # Security validation            │
│  │   ├── maos-worktree/   # Git worktree management        │
│  │   ├── maos-session/    # Session orchestration          │
│  │   ├── maos-tts/        # Text-to-speech providers       │
│  │   └── maos-common/     # Shared types and utilities     │
│  └── target/              # Build artifacts                 │
└─────────────────────────────────────────────────────────────┘
```

## Crate Structure and Dependencies

### Workspace Configuration

```toml
# /Cargo.toml
[workspace]
members = [
    "crates/maos-cli",
    "crates/maos-core",
    "crates/maos-security",
    "crates/maos-worktree",
    "crates/maos-session",
    "crates/maos-tts",
    "crates/maos-common",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["MAOS Contributors"]
license = "MIT"
repository = "https://github.com/clafollett/maos"

[workspace.dependencies]
clap = { version = "4.5", features = ["derive", "env"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.38", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
anyhow = "1.0"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

### Crate Dependency Graph

```
maos-cli
├── maos-core
│   ├── maos-security
│   ├── maos-worktree
│   ├── maos-session
│   └── maos-common
├── maos-tts
│   └── maos-common
└── maos-common
```

## Module Organization

### 1. maos-cli (Binary Crate)

**Purpose**: CLI entry point and command routing

```rust
// crates/maos-cli/src/main.rs
mod commands;
mod config;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "maos")]
#[command(about = "Multi-Agent Orchestration System CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(global = true, long, env = "MAOS_CONFIG")]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    PreToolUse(commands::PreToolUseArgs),
    PostToolUse(commands::PostToolUseArgs),
    Notify(commands::NotifyArgs),
    Stop(commands::StopArgs),
    SubagentStop(commands::SubagentStopArgs),
    PromptSubmit(commands::PromptSubmitArgs),
    SessionInfo(commands::SessionInfoArgs),
    WorktreeList(commands::WorktreeListArgs),
}
```

**Module Structure**:
```
crates/maos-cli/src/
├── main.rs              # Entry point
├── commands/
│   ├── mod.rs          # Command modules
│   ├── pre_tool_use.rs # PreToolUse implementation
│   ├── post_tool_use.rs
│   ├── notify.rs
│   ├── stop.rs
│   ├── session.rs      # session-info, worktree-list
│   └── logging.rs      # prompt-submit
├── config.rs           # Configuration loading
└── error.rs            # CLI-specific errors
```

### 2. maos-core (Core Logic)

**Purpose**: Orchestrates all MAOS operations

```rust
// crates/maos-core/src/lib.rs
pub mod orchestrator;
pub mod hooks;
pub mod coordination;

pub use orchestrator::Orchestrator;

pub struct Orchestrator {
    security: SecurityValidator,
    worktree: WorktreeManager,
    session: SessionManager,
    config: MaosConfig,
}
```

**Module Structure**:
```
crates/maos-core/src/
├── lib.rs
├── orchestrator.rs     # Main orchestration logic
├── hooks/
│   ├── mod.rs
│   ├── pre_tool.rs    # Pre-tool hook logic
│   └── post_tool.rs   # Post-tool hook logic
├── coordination/
│   ├── mod.rs
│   ├── locks.rs       # File locking
│   └── progress.rs    # Progress tracking
└── config.rs          # Core configuration
```

### 3. maos-security (Security Validation)

**Purpose**: Enforces security policies

```rust
// crates/maos-security/src/lib.rs
pub struct SecurityValidator {
    rules: Vec<Box<dyn SecurityRule>>,
}

pub trait SecurityRule: Send + Sync {
    fn validate(&self, context: &SecurityContext) -> Result<(), SecurityViolation>;
}
```

**Module Structure**:
```
crates/maos-security/src/
├── lib.rs
├── validator.rs        # Main validator
├── rules/
│   ├── mod.rs
│   ├── rm_rf.rs       # rm -rf protection
│   ├── env_files.rs   # .env protection
│   ├── paths.rs       # Path traversal protection
│   └── workspace.rs   # Workspace boundary enforcement
└── context.rs         # Security context
```

### 4. maos-worktree (Git Worktree Management)

**Purpose**: Manages git worktree operations

```rust
// crates/maos-worktree/src/lib.rs
pub struct WorktreeManager {
    repo_path: PathBuf,
    worktree_base: PathBuf,
}

impl WorktreeManager {
    pub async fn create_worktree(&self, spec: WorktreeSpec) -> Result<Worktree>;
    pub async fn remove_worktree(&self, id: &str) -> Result<()>;
    pub async fn list_worktrees(&self) -> Result<Vec<Worktree>>;
}
```

**Module Structure**:
```
crates/maos-worktree/src/
├── lib.rs
├── manager.rs         # Worktree lifecycle
├── git.rs            # Git command execution
├── naming.rs         # Branch naming strategy
└── cleanup.rs        # Cleanup operations
```

### 5. maos-session (Session Management)

**Purpose**: Manages multi-agent sessions

```rust
// crates/maos-session/src/lib.rs
pub struct SessionManager {
    session_dir: PathBuf,
}

pub struct Session {
    pub id: String,
    pub agents: Vec<Agent>,
    pub created_at: DateTime<Utc>,
    pub status: SessionStatus,
}
```

**Module Structure**:
```
crates/maos-session/src/
├── lib.rs
├── manager.rs        # Session lifecycle
├── store.rs         # File-based persistence
├── agent.rs         # Agent tracking
├── timeline.rs      # Event timeline
└── metrics.rs       # Performance metrics
```

### 6. maos-tts (Text-to-Speech)

**Purpose**: Multi-provider TTS integration

```rust
// crates/maos-tts/src/lib.rs
pub trait TtsProvider: Send + Sync {
    async fn speak(&self, text: &str) -> Result<()>;
    fn is_available(&self) -> bool;
}

pub struct TtsManager {
    providers: Vec<Box<dyn TtsProvider>>,
}
```

**Module Structure**:
```
crates/maos-tts/src/
├── lib.rs
├── manager.rs       # Provider selection
├── providers/
│   ├── mod.rs
│   ├── elevenlabs.rs
│   ├── openai.rs
│   ├── macos.rs
│   └── pyttsx3.rs
└── config.rs        # TTS configuration
```

### 7. maos-common (Shared Types)

**Purpose**: Common types and utilities

```rust
// crates/maos-common/src/lib.rs
pub mod types;
pub mod errors;
pub mod utils;

// Common types used across crates
pub struct ToolCall {
    pub tool: String,
    pub parameters: serde_json::Value,
    pub working_directory: PathBuf,
}
```

**Module Structure**:
```
crates/maos-common/src/
├── lib.rs
├── types/
│   ├── mod.rs
│   ├── tool_call.rs
│   ├── session.rs
│   └── agent.rs
├── errors.rs        # Common error types
└── utils/
    ├── mod.rs
    ├── fs.rs       # File system utilities
    ├── json.rs     # JSON helpers
    └── time.rs     # Time utilities
```

## Command Implementation Strategy

### Pre-Tool-Use Command

```rust
use maos_security::SecurityValidator;
use maos_core::{Session, WorktreeManager};

// Flow: CLI → Core → Security → Worktree → Session
async fn execute_pre_tool_use(args: PreToolUseArgs) -> Result<()> {
    // 1. Parse tool call from stdin or args
    let tool_call = parse_tool_call(&args)?;
    
    // 2. Security validation
    let validator = SecurityValidator::new();
    validator.validate(&tool_call)?;
    
    // 3. Check if Task tool for sub-agent
    if tool_call.is_task_tool() {
        // 4. Create worktree
        let worktree = worktree_manager.create_worktree(
            WorktreeSpec::from_task(&tool_call)
        ).await?;
        
        // 5. Update session
        session_manager.add_agent(Agent {
            id: tool_call.agent_id(),
            worktree: worktree.path,
            status: AgentStatus::Active,
        }).await?;
        
        // 6. Modify prompt to include workspace
        modify_task_prompt(&mut tool_call, &worktree)?;
    }
    
    // 7. Log and continue
    Ok(())
}
```

### Post-Tool-Use Command

```rust
async fn execute_post_tool_use(args: PostToolUseArgs) -> Result<()> {
    // 1. Update metrics
    metrics::record_tool_execution(&args)?;
    
    // 2. Release any locks
    lock_manager.release_for_tool(&args.tool_id)?;
    
    // 3. Update session progress
    session_manager.update_progress(&args)?;
    
    // 4. Cleanup if needed
    if args.is_session_complete() {
        cleanup_manager.schedule_cleanup(&args.session_id)?;
    }
    
    Ok(())
}
```

## Data Flow Architecture

### Hook Execution Flow

```
Claude Code → settings.json → maos CLI → Hook Processing
                                ↓
                        ┌───────┴────────┐
                        │ Security Check │
                        └───────┬────────┘
                                ↓
                        ┌───────┴────────┐
                        │ Session Update │
                        └───────┬────────┘
                                ↓
                        ┌───────┴────────┐
                        │ Worktree Mgmt  │
                        └───────┬────────┘
                                ↓
                        ┌───────┴────────┐
                        │ State Persist  │
                        └────────────────┘
```

### State Management

```
.maos/                       # Default location (override with $MAOS_SESSION_DIR)
├── config.json              # Global configuration
├── active_session.json      # Current session pointer
└── sessions/
    └── {session_id}/
        ├── session.json     # Session metadata
        ├── agents.json      # Agent registry
        ├── locks.json       # File locks
        ├── progress.json    # Task progress
        ├── timeline.json    # Event log
        └── metrics.json     # Performance data
```

**Session Directory Configuration:**
- Default: `.maos/sessions/` in project root
- Override: `export MAOS_SESSION_DIR=/custom/path/sessions`
- Shared sessions: Point multiple projects to same session directory

## Performance Optimization Strategy

### 1. Fast Startup (<10ms target)

```rust
// Lazy initialization
static CONFIG: OnceCell<MaosConfig> = OnceCell::new();

// Pre-compiled regex patterns
lazy_static! {
    static ref RM_RF_PATTERN: Regex = Regex::new(r"rm\s+-rf").unwrap();
}

// Minimal dependencies
#[cfg(feature = "minimal")]
compile_without_heavy_deps!();
```

### 2. Efficient I/O

```rust
// Buffered file operations
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

// Memory-mapped files for large reads
use memmap2::MmapOptions;

// Atomic file operations
use tempfile::NamedTempFile;
```

### 3. Parallel Processing

```rust
// Concurrent security checks
let validations = tokio::join!(
    check_rm_rf(&tool_call),
    check_env_access(&tool_call),
    check_path_traversal(&tool_call),
);

// Parallel worktree operations
let worktrees = stream::iter(agents)
    .map(|agent| create_worktree(agent))
    .buffer_unordered(4)
    .collect().await?;
```

## Integration Points

### 1. Claude Code Hook Integration

```json
// .claude/settings.json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos",
      "args": ["pre-tool-use"],
      "stdin": true
    }],
    "PostToolUse": [{
      "command": "maos",
      "args": ["post-tool-use"],
      "stdin": true
    }]
  }
}
```

### 2. Environment Variables

```bash
MAOS_CONFIG=/path/to/config.json
MAOS_LOG_LEVEL=debug
MAOS_SESSION_DIR=.maos/sessions
MAOS_WORKTREE_BASE=worktrees
MAOS_TTS_PROVIDER=elevenlabs
ELEVENLABS_API_KEY=xxx
```

### 3. Exit Codes

```rust
pub enum ExitCode {
    Success = 0,
    SecurityViolation = 2,
    ConfigError = 3,
    WorktreeError = 4,
    SessionError = 5,
    TtsError = 6,
    UnknownError = 127,
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rm_rf_detection() {
        let validator = SecurityValidator::new();
        let result = validator.validate_command("rm -rf /");
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
// tests/integration/worktree_test.rs
#[tokio::test]
async fn test_worktree_lifecycle() {
    let temp_repo = TempRepo::new();
    let manager = WorktreeManager::new(&temp_repo);
    
    let worktree = manager.create_worktree(spec).await?;
    assert!(worktree.path.exists());
    
    manager.remove_worktree(&worktree.id).await?;
    assert!(!worktree.path.exists());
}
```

### Performance Benchmarks

```rust
// benches/startup.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_startup(c: &mut Criterion) {
    c.bench_function("maos startup", |b| {
        b.iter(|| {
            Command::new("maos")
                .arg("--version")
                .output()
                .unwrap()
        })
    });
}
```

## Build and Distribution

### Release Profile

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

### Cross-Platform Builds

```yaml
# .github/workflows/release.yml
strategy:
  matrix:
    target:
      - x86_64-unknown-linux-gnu
      - x86_64-apple-darwin
      - aarch64-apple-darwin
      - x86_64-pc-windows-msvc
```

### Binary Size Optimization

```rust
// Conditional features
#[cfg(feature = "full")]
mod full_features;

#[cfg(not(feature = "full"))]
mod minimal_features;

// Dead code elimination
#![cfg_attr(not(debug_assertions), deny(dead_code))]
```

## Migration from Python

### Compatibility Layer

```rust
// Support existing hook format during transition
pub fn parse_legacy_hook_input(input: &str) -> Result<ToolCall> {
    // Parse Python hook JSON format
    let legacy: LegacyFormat = serde_json::from_str(input)?;
    Ok(legacy.into())
}
```

### Feature Parity Checklist

- [x] Security validation (rm -rf, .env)
- [x] Git worktree management
- [x] Session coordination
- [x] File locking
- [x] TTS notifications
- [x] Progress tracking
- [x] Performance logging
- [x] Error handling

## Summary

This architecture provides a solid foundation for the MAOS Rust CLI that:

1. **Achieves <10ms execution** through careful optimization
2. **Maintains zero runtime dependencies** via static linking
3. **Provides seamless Claude Code integration** through hooks
4. **Enables parallel agent development** via git worktrees
5. **Ensures security and reliability** through comprehensive validation

The modular crate structure allows for independent development and testing while maintaining clear boundaries between concerns. The performance-first design ensures minimal impact on Claude Code's operation while providing powerful orchestration capabilities.

## Try It Now

To verify the workspace builds and explore the CLI:

```bash
# Clone and build
git clone https://github.com/clafollett/maos.git
cd maos
cargo build

# Run help to see available commands
cargo run --bin maos -- --help

# Test pre-tool-use command
echo '{"tool":"Bash","params":{"command":"ls"}}' | cargo run --bin maos -- pre-tool-use

# Run tests
cargo test --workspace

# Generate documentation
cargo doc --open
```