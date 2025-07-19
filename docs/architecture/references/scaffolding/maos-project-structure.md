# MAOS Project Structure

## Overview

This document outlines the recommended project structure for MAOS based on Domain-Driven Design principles and our architectural decisions. The structure follows the rust-analyzer pattern with a virtual workspace and crates organized under the `crates/` directory.

## Directory Structure

```
maos/
├── Cargo.toml                    # Virtual workspace manifest
├── Cargo.lock                    # Shared dependency lockfile
├── README.md                     # Project overview
├── LICENSE                       # MIT License
├── .github/                      # GitHub configuration
│   ├── workflows/               # CI/CD workflows
│   └── ISSUE_TEMPLATE/          # Issue templates
├── docs/                        # Documentation (existing)
└── crates/                      # All crates organized here
    ├── maos/                    # Main binary (CLI/MCP server)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs          # Entry point
    │       ├── cli/             # CLI command handling
    │       │   ├── mod.rs
    │       │   ├── orchestrate.rs
    │       │   └── status.rs
    │       ├── server/          # MCP server implementation
    │       │   ├── mod.rs
    │       │   ├── tools.rs
    │       │   └── streaming.rs
    │       └── shared/          # Shared utilities
    │           ├── mod.rs
    │           └── config.rs
    ├── maos-domain/             # Domain layer
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── aggregates/
    │       │   ├── mod.rs
    │       │   ├── session.rs   # Session aggregate
    │       │   ├── agent.rs     # Agent aggregate
    │       │   └── instance.rs  # Instance aggregate
    │       ├── value_objects/
    │       │   ├── mod.rs
    │       │   ├── agent_role.rs    # AgentRole value object
    │       │   ├── session_id.rs    # SessionId value object
    │       │   └── agent_id.rs      # AgentId value object
    │       ├── events/
    │       │   ├── mod.rs
    │       │   └── session_events.rs
    │       └── services/
    │           ├── mod.rs
    │           └── dependency_resolver.rs
    ├── maos-app/                # Application layer
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── handlers/        # MCP tool handlers
    │       │   ├── mod.rs
    │       │   ├── orchestrate_handler.rs
    │       │   ├── spawn_agent_handler.rs
    │       │   ├── session_status_handler.rs
    │       │   └── list_roles_handler.rs
    │       ├── services/
    │       │   ├── mod.rs
    │       │   ├── session_manager.rs
    │       │   └── process_manager.rs
    │       └── dto/             # Data transfer objects
    │           ├── mod.rs
    │           └── mcp_types.rs
    └── maos-io/                 # I/O operations layer
        ├── Cargo.toml
        └── src/
            ├── lib.rs
            ├── persistence/
            │   ├── mod.rs
            │   ├── sqlite/
            │   │   ├── mod.rs
            │   │   ├── schema.sql
            │   │   ├── session_repository.rs
            │   │   └── instance_repository.rs
            │   └── filesystem/
            │       ├── mod.rs
            │       ├── message_queue.rs
            │       └── shared_context.rs
            ├── process/
            │   ├── mod.rs
            │   ├── agent_spawner.rs
            │   ├── health_monitor.rs
            │   └── output_streamer.rs
            ├── acp/
            │   ├── mod.rs
            │   ├── protocol.rs
            │   └── server.rs
            └── logging/
                ├── mod.rs
                └── session_logger.rs
```

## Layer Dependencies

```
┌─────────────────┐
│      maos       │ (Main Binary)
├─────────────────┤
│    maos-app     │ (Application)
├─────────────────┤
│  maos-domain    │ (Domain)
└─────────────────┘
        ↑
        │
┌─────────────────┐
│    maos-io      │ (I/O Operations)
└─────────────────┘
```

## Cargo Workspace Configuration

### Root Cargo.toml (Virtual Manifest)
```toml
[workspace]
members = [
    "crates/maos",
    "crates/maos-domain",
    "crates/maos-app",
    "crates/maos-io"
]
resolver = "3"

[workspace.package]
version = "0.1.0"
authors = ["MAOS Contributors"]
edition = "2024"
license = "MIT"
repository = "https://github.com/clafollett/maos"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
```

### Main Binary Cargo.toml
```toml
[package]
name = "maos"
version.workspace = true
edition.workspace = true

[[bin]]
name = "maos"
path = "src/main.rs"

[dependencies]
maos-domain = { path = "../maos-domain" }
maos-app = { path = "../maos-app" }
maos-io = { path = "../maos-io" }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
clap = { version = "4.0", features = ["derive"] }
```

### Domain Layer Cargo.toml
```toml
[package]
name = "maos-domain"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
uuid.workspace = true
thiserror.workspace = true
chrono.workspace = true
```

### Application Layer Cargo.toml
```toml
[package]
name = "maos-app"
version.workspace = true
edition.workspace = true

[dependencies]
maos-domain = { path = "../maos-domain" }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
anyhow.workspace = true
tracing.workspace = true
```

### I/O Layer Cargo.toml
```toml
[package]
name = "maos-io"
version.workspace = true
edition.workspace = true

[dependencies]
maos-domain = { path = "../maos-domain" }
maos-app = { path = "../maos-app" }
tokio.workspace = true
sqlx.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
anyhow.workspace = true
tracing.workspace = true
```

## Key Implementation Files

### Domain: AgentRole
```rust
// crates/maos-domain/src/value_objects/agent_role.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentRole {
    pub name: String,
    pub description: String,
    pub responsibilities: String,
    pub is_predefined: bool,
    pub instance_suffix: Option<String>,
}
```

### Application: SessionManager
```rust
// crates/maos-app/src/services/session_manager.rs
pub struct SessionManager {
    session_repository: Arc<dyn SessionRepository>,
    process_manager: Arc<ProcessManager>,
    logger: Arc<SessionLogger>,
}
```

### I/O: AgentSpawner
```rust
// crates/maos-io/src/process/agent_spawner.rs
pub struct AgentSpawner {
    instance_tracker: InstanceTracker,
    template_generator: TemplateGenerator,
}
```

### Main Binary: Entry Point
```rust
// crates/maos/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "maos")]
#[command(about = "Multi-Agent Orchestration System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP server
    Serve,
    /// Start orchestration (CLI mode)
    Orchestrate { task: String },
    /// Show status
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Serve => {
            // Start MCP server
            server::start_mcp_server().await?;
        }
        Commands::Orchestrate { task } => {
            // Run CLI orchestration
            cli::orchestrate_task(task).await?;
        }
        Commands::Status => {
            // Show status
            cli::show_status().await?;
        }
    }
    
    Ok(())
}
```

## Development Guidelines

1. **Domain Layer**: Keep it pure - no I/O, no frameworks
2. **Application Layer**: Orchestrate domain objects, define use cases
3. **I/O Layer**: All technical concerns (DB, files, processes, ACP)
4. **Main Binary**: Thin layer for CLI/MCP protocol handling

## Testing Strategy

1. **Unit Tests**: In each crate's `src/` directory alongside the code
2. **Integration Tests**: In each crate's `tests/` directory
3. **End-to-End Tests**: In workspace root `tests/` directory
4. **Benchmarks**: In `benches/` directories when needed

## Next Steps

1. Create the workspace structure
2. Implement domain models first (TDD approach)
3. Build I/O repositories
4. Create application services
5. Wire up main binary last

---

*This structure follows the rust-analyzer pattern and provides a solid foundation for MAOS development.*