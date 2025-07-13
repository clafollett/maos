# MAOS Scaffolding Plan Using Agenterra

## Overview
Use Agenterra's existing OpenAPI-to-MCP server generation to scaffold MAOS's MCP server structure, then replace the HTTP REST client calls with our actual orchestration logic.

## Step 1: Generate MCP Server from OpenAPI

```bash
# Navigate to Agenterra directory
cd /Users/clafollett/Repositories/clafollett/agenterra

# Generate MCP server from our dummy OpenAPI spec
cargo run -- scaffold mcp server \
  --schema-path /Users/clafollett/Repositories/maos/docs/architecture/scaffolding/maos-openapi-dummy.yaml \
  --project-name maos-server \
  --base-url http://localhost:8080

# Or if Agenterra is installed:
agenterra scaffold mcp server \
  --schema-path ./docs/architecture/scaffolding/maos-openapi-dummy.yaml \
  --project-name maos-server \
  --base-url http://localhost:8080
```

## Step 2: Expected Generated Structure

Agenterra will generate:
```
maos-server/
├── Cargo.toml              # With agenterra-rmcp dependency
├── src/
│   ├── main.rs            # MCP server entry point
│   ├── tools/
│   │   ├── mod.rs         # Tool registry
│   │   ├── orchestrate.rs # orchestrate tool handler
│   │   ├── register.rs    # register tool handler
│   │   ├── list_agents.rs # list-agents tool handler
│   │   ├── set_default_agent.rs
│   │   ├── get_status.rs
│   │   └── cancel_session.rs
│   ├── resources/
│   │   ├── mod.rs
│   │   └── progress.rs    # SSE progress stream
│   └── transport/
│       └── http_sse.rs    # HTTP/SSE transport
```

## Step 3: Replace Generated HTTP Calls

Each generated tool will have something like:
```rust
// GENERATED CODE
async fn orchestrate_handler(params: Value) -> Result<Value> {
    // Parse input
    let tasks = params["tasks"].as_array().unwrap();
    
    // Make HTTP request to backend
    let response = http_client
        .post("http://localhost:8080/orchestrate")
        .json(&params)
        .send()
        .await?;
    
    Ok(response.json().await?)
}
```

We'll replace with our actual logic:
```rust
// OUR REPLACEMENT
async fn orchestrate_handler(params: Value) -> Result<Value> {
    // Parse input  
    let request: OrchestrationRequest = serde_json::from_value(params)?;
    
    // Create orchestration session using our domain logic
    let session = OrchestrationSession::create(
        request.tasks,
        request.execution_strategy,
    )?;
    
    // Store in PostgreSQL via event sourcing
    let event = DomainEvent::OrchestrationCreated {
        session_id: session.id,
        task_count: session.tasks.len(),
        execution_strategy: session.strategy.to_string(),
    };
    event_store.append(event).await?;
    
    // Start orchestration
    orchestrator.start_session(session).await?;
    
    Ok(json!({
        "session_id": session.id,
        "status": "started",
        "message": "Orchestration session started"
    }))
}
```

## Step 4: Integration Points

### 4.1 Use agenterra-rmcp
- Already included in generated Cargo.toml
- Provides MCP protocol implementation
- Handles tool registration and discovery

### 4.2 Add Our Dependencies
```toml
[dependencies]
# From Agenterra generation
agenterra-rmcp = { git = "https://github.com/clafollett/agenterra-rmcp" }

# Our additions
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
anyhow = "1.0"
```

### 4.3 Keep Generated Structure
- Tool registration boilerplate
- MCP protocol handling
- Transport layer setup
- Resource serving infrastructure

### 4.4 Add Our Components
```rust
// Add to main.rs after generation
mod domain;
mod infrastructure;
mod application;

use crate::infrastructure::PostgresEventStore;
use crate::domain::orchestration::Orchestrator;
use crate::application::ProcessManager;

// Initialize our components
let event_store = PostgresEventStore::new(&database_url).await?;
let process_manager = ProcessManager::new();
let orchestrator = Orchestrator::new(event_store, process_manager);

// Pass to generated MCP server
let server = MaosServer::new(orchestrator);
```

## Step 5: Specific Replacements

### orchestrate.rs
- Remove: HTTP POST to /orchestrate
- Add: Create session, store events, spawn agents

### register.rs  
- Remove: HTTP POST to /register
- Add: CLI detection using Command, update agent registry

### list_agents.rs
- Remove: HTTP GET to /agents
- Add: Query agent registry from memory/PostgreSQL

### get_status.rs
- Remove: HTTP GET to /sessions/{id}/status
- Add: Event replay to build current session state

### cancel_session.rs
- Remove: HTTP POST to /sessions/{id}/cancel  
- Add: Send termination signals to agent processes

### progress.rs (SSE)
- Remove: Proxy to HTTP backend
- Add: Subscribe to PostgreSQL NOTIFY/LISTEN channels

## Benefits of This Approach

1. **Time Savings**: All MCP boilerplate is generated
2. **Consistency**: Uses proven Agenterra patterns
3. **Flexibility**: Easy to customize generated code
4. **Best Practices**: Follows MCP protocol standards
5. **Type Safety**: Generated from OpenAPI schema

## Next Steps

1. Run Agenterra generation command
2. Move generated `maos-server` directory to MAOS repository
3. Systematically replace HTTP calls with domain logic
4. Add our domain models and infrastructure
5. Test with Claude Code MCP client

## Example: Running the Generation

```bash
# From MAOS repository root
cd ../clafollett/agenterra

# Generate the server
cargo run -- scaffold mcp server \
  --schema-path ../maos/docs/architecture/scaffolding/maos-openapi-dummy.yaml \
  --project-name maos-server \
  --base-url http://localhost:8080

# Move generated server to MAOS
mv maos-server ../maos/generated/

# Or directly generate into MAOS directory
cargo run -- scaffold mcp server \
  --schema-path ../maos/docs/architecture/scaffolding/maos-openapi-dummy.yaml \
  --project-name maos-server \
  --base-url http://localhost:8080 \
  --output-dir ../maos/generated/maos-server
```

This approach gives us a working MCP server skeleton in minutes instead of hours, letting us focus on the orchestration logic rather than protocol implementation.