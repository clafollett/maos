# MAOS Project Structure

## Overview

This document outlines the recommended project structure for MAOS based on Domain-Driven Design principles and our architectural decisions.

## Directory Structure

```
maos/
├── Cargo.toml                    # Workspace definition
├── README.md                     # Project overview
├── LICENSE                       # MIT License
├── .github/                      # GitHub configuration
│   ├── workflows/               # CI/CD workflows
│   └── ISSUE_TEMPLATE/          # Issue templates
├── docs/                        # Documentation (existing)
├── maos-domain/                 # Domain layer
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── aggregates/
│       │   ├── mod.rs
│       │   ├── session.rs      # Session aggregate
│       │   ├── agent.rs        # Agent aggregate
│       │   └── instance.rs     # Instance aggregate
│       ├── value_objects/
│       │   ├── mod.rs
│       │   ├── agent_role.rs   # AgentRole value object
│       │   ├── session_id.rs   # SessionId value object
│       │   └── agent_id.rs     # AgentId value object
│       ├── events/
│       │   ├── mod.rs
│       │   └── session_events.rs
│       └── services/
│           ├── mod.rs
│           └── dependency_resolver.rs
├── maos-application/            # Application layer
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── handlers/           # MCP tool handlers
│       │   ├── mod.rs
│       │   ├── orchestrate_handler.rs
│       │   ├── spawn_agent_handler.rs
│       │   ├── session_status_handler.rs
│       │   └── list_roles_handler.rs
│       ├── services/
│       │   ├── mod.rs
│       │   ├── session_manager.rs
│       │   └── process_manager.rs
│       └── dto/               # Data transfer objects
│           ├── mod.rs
│           └── mcp_types.rs
├── maos-infrastructure/         # Infrastructure layer
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── persistence/
│       │   ├── mod.rs
│       │   ├── sqlite/
│       │   │   ├── mod.rs
│       │   │   ├── schema.sql
│       │   │   ├── session_repository.rs
│       │   │   └── instance_repository.rs
│       │   └── filesystem/
│       │       ├── mod.rs
│       │       ├── message_queue.rs
│       │       └── shared_context.rs
│       ├── process/
│       │   ├── mod.rs
│       │   ├── agent_spawner.rs
│       │   ├── health_monitor.rs
│       │   └── output_streamer.rs
│       ├── mcp/
│       │   ├── mod.rs
│       │   ├── protocol.rs
│       │   └── streaming.rs
│       └── logging/
│           ├── mod.rs
│           └── session_logger.rs
├── maos-server/                 # Presentation layer (MCP Server)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # Entry point
│       ├── server.rs           # MCP server setup
│       ├── tools/              # Tool definitions
│       │   ├── mod.rs
│       │   └── definitions.rs
│       ├── resources/          # Resource providers
│       │   ├── mod.rs
│       │   └── providers.rs
│       └── config/
│           ├── mod.rs
│           └── settings.rs
├── maos-cli/                    # CLI management tool
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── commands/
│           ├── mod.rs
│           ├── serve.rs        # Start MCP server
│           ├── instances.rs    # Manage instances
│           └── sessions.rs     # Query sessions
├── tests/                       # Integration tests
│   ├── common/
│   │   └── mod.rs
│   ├── mcp_integration.rs
│   ├── process_spawning.rs
│   └── orchestration.rs
└── examples/                    # Example workflows
    ├── basic_orchestration.md
    ├── custom_roles.md
    └── multi_agent_workflow.md
```

## Layer Dependencies

```
┌─────────────────┐
│  maos-server    │ (Presentation)
├─────────────────┤
│maos-application │ (Application)
├─────────────────┤
│  maos-domain    │ (Domain)
└─────────────────┘
        ↑
        │
┌─────────────────┐
│maos-infrastructure│ (Infrastructure)
└─────────────────┘
```

## Cargo Workspace Configuration

### Root Cargo.toml
```toml
[workspace]
members = [
    "maos-domain",
    "maos-application", 
    "maos-infrastructure",
    "maos-server",
    "maos-cli",
]

[workspace.package]
version = "0.1.0"
authors = ["MAOS Contributors"]
edition = "2021"
license = "MIT"

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
name = "maos-application"
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

### Infrastructure Layer Cargo.toml
```toml
[package]
name = "maos-infrastructure"
version.workspace = true
edition.workspace = true

[dependencies]
maos-domain = { path = "../maos-domain" }
maos-application = { path = "../maos-application" }
tokio.workspace = true
sqlx.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
anyhow.workspace = true
tracing.workspace = true
```

### Server Layer Cargo.toml
```toml
[package]
name = "maos-server"
version.workspace = true
edition.workspace = true

[[bin]]
name = "maos-server"
path = "src/main.rs"

[dependencies]
maos-domain = { path = "../maos-domain" }
maos-application = { path = "../maos-application" }
maos-infrastructure = { path = "../maos-infrastructure" }
tokio.workspace = true
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
```

## Key Implementation Files

### Domain: AgentRole
```rust
// maos-domain/src/value_objects/agent_role.rs
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
// maos-application/src/services/session_manager.rs
pub struct SessionManager {
    session_repository: Arc<dyn SessionRepository>,
    process_manager: Arc<ProcessManager>,
    logger: Arc<SessionLogger>,
}
```

### Infrastructure: AgentSpawner
```rust
// maos-infrastructure/src/process/agent_spawner.rs
pub struct AgentSpawner {
    instance_tracker: InstanceTracker,
    template_generator: TemplateGenerator,
}
```

### Server: MCP Handler
```rust
// maos-server/src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    let app = create_mcp_server().await?;
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
```

## Development Guidelines

1. **Domain Layer**: Keep it pure - no I/O, no frameworks
2. **Application Layer**: Orchestrate domain objects, define use cases
3. **Infrastructure Layer**: All technical concerns (DB, files, processes)
4. **Server Layer**: Thin layer for MCP protocol handling

## Testing Strategy

1. **Unit Tests**: In each crate's `src/` directory
2. **Integration Tests**: In root `tests/` directory
3. **Examples**: In `examples/` for documentation
4. **Benchmarks**: In `benches/` when needed

## Next Steps

1. Create the workspace structure
2. Implement domain models first
3. Build infrastructure repositories
4. Create application services
5. Wire up MCP server last