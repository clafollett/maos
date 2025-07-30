# MAOS Rust CLI Technical Design

## Overview

The MAOS CLI is a high-performance Rust application that manages multi-agent orchestration for Claude Code. It provides worktree management, security hooks, and coordination mechanisms with zero runtime overhead.

## Architecture

### Core Modules

```rust
maos-cli/
├── src/
│   ├── main.rs              // CLI entry point
│   ├── cli/
│   │   ├── mod.rs           // Command definitions
│   │   ├── worktree.rs      // Worktree commands
│   │   ├── agent.rs         // Agent management
│   │   ├── session.rs       // Session management
│   │   └── config.rs        // Configuration
│   ├── core/
│   │   ├── mod.rs           // Core business logic
│   │   ├── worktree.rs      // Git worktree operations
│   │   ├── coordination.rs  // File-based coordination
│   │   ├── locking.rs       // Lock management
│   │   └── security.rs      // Security policies
│   ├── hooks/
│   │   ├── mod.rs           // Hook system
│   │   ├── validator.rs     // Security validation
│   │   ├── policies.rs      // Security policies
│   │   └── audit.rs         // Audit logging
│   └── utils/
│       ├── mod.rs           // Utilities
│       ├── fs.rs            // File system helpers
│       ├── git.rs           // Git operations
│       └── ipc.rs           // Inter-process communication
```

## Command Structure

### Primary Commands

```bash
# Worktree Management
maos worktree create <issue-id> [--branch-prefix=<prefix>]
maos worktree list [--active] [--format=json]
maos worktree switch <issue-id>
maos worktree cleanup [--merged] [--stale=<days>]

# Agent Management
maos agent spawn <agent-type> [--worktree=<id>] [--policy=<policy>]
maos agent list [--active] [--format=json]
maos agent communicate <agent-id> --message=<msg>
maos agent status <agent-id>

# Session Management
maos session start <issue-id> [--agents=<list>]
maos session status [--format=json]
maos session end [--merge]

# Configuration
maos config get <key>
maos config set <key> <value>
maos config validate
```

### Implementation Example

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Global verbosity flag
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage git worktrees for parallel development
    Worktree {
        #[command(subcommand)]
        action: WorktreeCommands,
    },
    /// Manage AI agents
    Agent {
        #[command(subcommand)]
        action: AgentCommands,
    },
    /// Manage development sessions
    Session {
        #[command(subcommand)]
        action: SessionCommands,
    },
    /// Configure MAOS
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum WorktreeCommands {
    /// Create a new worktree for an issue
    Create {
        /// GitHub issue ID
        issue_id: String,
        /// Branch name prefix
        #[arg(long, default_value = "feature")]
        branch_prefix: String,
    },
    /// List all worktrees
    List {
        /// Show only active worktrees
        #[arg(long)]
        active: bool,
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Switch to a worktree
    Switch {
        /// Issue ID or worktree name
        issue_id: String,
    },
    /// Clean up old worktrees
    Cleanup {
        /// Remove only merged worktrees
        #[arg(long)]
        merged: bool,
        /// Remove worktrees older than N days
        #[arg(long)]
        stale: Option<u32>,
    },
}
```

## Performance Optimization Strategies

### 1. Zero-Copy Operations

```rust
use std::borrow::Cow;
use bytes::Bytes;

pub struct FileContent<'a> {
    path: Cow<'a, str>,
    content: Bytes,  // Zero-copy byte buffer
    metadata: FileMetadata,
}

impl<'a> FileContent<'a> {
    pub fn read_zero_copy(path: &'a str) -> Result<Self> {
        let content = std::fs::read(path)?;
        Ok(Self {
            path: Cow::Borrowed(path),
            content: Bytes::from(content),
            metadata: FileMetadata::from_path(path)?,
        })
    }
}
```

### 2. Async I/O with Tokio

```rust
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures::stream::{self, StreamExt};

pub async fn parallel_file_operations(paths: Vec<String>) -> Result<()> {
    const CONCURRENCY_LIMIT: usize = 10;
    
    stream::iter(paths)
        .map(|path| async move {
            process_file(&path).await
        })
        .buffer_unordered(CONCURRENCY_LIMIT)
        .collect::<Vec<_>>()
        .await;
    
    Ok(())
}
```

### 3. Memory-Mapped Files for Large Operations

```rust
use memmap2::{Mmap, MmapOptions};
use std::fs::File;

pub struct MappedFile {
    mmap: Mmap,
}

impl MappedFile {
    pub fn open(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(Self { mmap })
    }
    
    pub fn search_pattern(&self, pattern: &[u8]) -> Option<usize> {
        self.mmap.windows(pattern.len())
            .position(|window| window == pattern)
    }
}
```

### 4. Lock-Free Data Structures

```rust
use crossbeam::channel::{unbounded, Receiver, Sender};
use dashmap::DashMap;
use std::sync::Arc;

pub struct CoordinationState {
    // Lock-free concurrent hashmap
    agent_states: Arc<DashMap<String, AgentState>>,
    // Multi-producer, multi-consumer channel
    messages: (Sender<Message>, Receiver<Message>),
}

impl CoordinationState {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            agent_states: Arc::new(DashMap::new()),
            messages: (tx, rx),
        }
    }
    
    pub fn update_agent_state(&self, agent_id: String, state: AgentState) {
        self.agent_states.insert(agent_id, state);
    }
}
```

## Integration with Claude Code

### 1. Environment Detection

```rust
pub struct ClaudeEnvironment {
    pub session_id: Option<String>,
    pub agent_type: Option<String>,
    pub workspace_root: PathBuf,
}

impl ClaudeEnvironment {
    pub fn detect() -> Self {
        Self {
            session_id: env::var("CLAUDE_SESSION_ID").ok(),
            agent_type: env::var("CLAUDE_AGENT_TYPE").ok(),
            workspace_root: env::current_dir().unwrap_or_default(),
        }
    }
    
    pub fn is_claude_environment(&self) -> bool {
        self.session_id.is_some()
    }
}
```

### 2. Hook Integration

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HookEvent {
    pub hook_event_name: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub timestamp: u64,
}

pub async fn process_hook_event(event: HookEvent) -> Result<HookResponse> {
    let policy = SecurityPolicy::from_env()?;
    let validator = SecurityValidator::new(policy);
    
    match validator.validate(&event).await {
        Ok(()) => Ok(HookResponse::allow()),
        Err(violation) => Ok(HookResponse::block(violation.reason())),
    }
}
```

### 3. Worktree Automation

```rust
pub struct WorktreeManager {
    repo_path: PathBuf,
    config: WorktreeConfig,
}

impl WorktreeManager {
    pub async fn create_for_issue(&self, issue_id: &str) -> Result<Worktree> {
        // Fetch issue details from GitHub
        let issue = self.fetch_github_issue(issue_id).await?;
        
        // Generate branch name
        let branch_name = format!(
            "{}/issue-{}/{}",
            self.config.branch_prefix,
            issue_id,
            slugify(&issue.title)
        );
        
        // Create worktree
        let worktree_path = self.repo_path
            .parent()
            .unwrap()
            .join(format!("maos-{}", issue_id));
        
        self.git_create_worktree(&branch_name, &worktree_path)?;
        
        // Initialize MAOS structure
        self.initialize_maos_structure(&worktree_path).await?;
        
        Ok(Worktree {
            path: worktree_path,
            branch: branch_name,
            issue_id: issue_id.to_string(),
        })
    }
}
```

## Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MaosError {
    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),
    
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Agent communication failed: {0}")]
    CommunicationError(String),
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// Result type alias
pub type MaosResult<T> = Result<T, MaosError>;
```

## Configuration

```toml
# ~/.config/maos/config.toml

[core]
default_branch_prefix = "feature"
auto_cleanup_days = 30
parallel_operations = true

[security]
default_policy = "standard"
audit_log_path = "~/.maos/audit.log"
enable_hooks = true

[worktree]
base_path = "../"
naming_pattern = "maos-{issue_id}"
auto_fetch = true

[agent]
communication_timeout = 30
max_concurrent_agents = 10
shared_directory = ".maos/shared"

[github]
api_token_env = "GITHUB_TOKEN"
default_owner = "clafollett"
default_repo = "maos"
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_worktree_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WorktreeManager::new(temp_dir.path());
        
        let worktree = manager.create_for_issue("123").await.unwrap();
        
        assert!(worktree.path.exists());
        assert_eq!(worktree.issue_id, "123");
    }
    
    #[test]
    fn test_security_validation() {
        let policy = SecurityPolicy::strict();
        let validator = SecurityValidator::new(policy);
        
        let event = HookEvent {
            hook_event_name: "PreToolUse".to_string(),
            tool_name: "Write".to_string(),
            tool_input: json!({
                "file_path": "/etc/passwd",
                "content": "malicious"
            }),
            timestamp: 0,
        };
        
        assert!(validator.validate(&event).is_err());
    }
}
```

## Benchmarking

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_file_operations(c: &mut Criterion) {
    c.bench_function("zero_copy_read", |b| {
        b.iter(|| {
            FileContent::read_zero_copy(black_box("test.txt"))
        });
    });
    
    c.bench_function("parallel_processing", |b| {
        b.iter(|| {
            let paths: Vec<_> = (0..100)
                .map(|i| format!("file_{}.txt", i))
                .collect();
            parallel_file_operations(black_box(paths))
        });
    });
}

criterion_group!(benches, benchmark_file_operations);
criterion_main!(benches);
```

## Build Configuration

```toml
# Cargo.toml
[package]
name = "maos-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.5", features = ["derive", "env"] }
tokio = { version = "1.40", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
git2 = "0.19"
dashmap = "6.0"
crossbeam = "0.8"
bytes = "1.7"
memmap2 = "0.9"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }
tempfile = "3.10"

[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true

[profile.bench]
debug = true
```

This design provides a solid foundation for the MAOS Rust CLI with high performance, security, and seamless integration with Claude Code.