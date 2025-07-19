# MAOS - Multi-Agent Orchestration System

## Plan Overview

Build a Rust-based agent-agnostic orchestration system using **Domain-Driven Design (DDD)** principles and **Test-Driven Development (TDD)** with Red/Green/Refactor cycle. MAOS can orchestrate any type of agent - Claude, GPT, local LLMs, or custom tools.

## Core Development Principles

### Domain-Driven Design (DDD)
- **Bounded Contexts** - Clear separation between domains
- **Ubiquitous Language** - Consistent terminology across code and documentation
- **Aggregates** - Properly designed aggregate roots
- **Value Objects** - Immutable domain concepts
- **Domain Events** - Event-driven architecture
- **Repository Pattern** - Abstract persistence details
- **Anti-Corruption Layer** - Clean boundaries with external systems

### Test-Driven Development (TDD)
- **Red/Green/Refactor** - Write failing test, make it pass, improve code
- **Test First** - No production code without failing test
- **Unit Tests** - Test domain logic in isolation
- **Integration Tests** - Test infrastructure adapters
- **Acceptance Tests** - Test complete user scenarios
- **Test Coverage** - Maintain >90% coverage
- **Property-Based Testing** - Use proptest for invariants

## DDD Architecture Layers

### 1. Domain Layer (`maos-domain`)
**Pure business logic - no external dependencies**

```rust
// Example structure
crates/maos-domain/
├── src/
│   ├── aggregates/
│   │   ├── agent.rs         // Agent aggregate root
│   │   ├── task.rs          // Task aggregate root
│   │   └── orchestration.rs // Orchestration aggregate
│   ├── value_objects/
│   │   ├── agent_id.rs      // Strongly typed IDs
│   │   ├── capability.rs    // Agent capabilities
│   │   └── task_status.rs   // Task state
│   ├── events/
│   │   ├── agent_events.rs  // AgentRegistered, AgentRemoved
│   │   └── task_events.rs   // TaskCreated, TaskCompleted
│   ├── services/
│   │   ├── task_scheduler.rs    // Domain service
│   │   └── capability_matcher.rs // Domain service
│   └── repositories/
│       ├── agent_repository.rs   // Trait definition
│       └── task_repository.rs    // Trait definition
```

### 2. Application Layer (`maos-app`)
**Use cases and application services**

```rust
crates/maos-app/
├── src/
│   ├── commands/              // Command handlers
│   │   ├── register_agent.rs
│   │   ├── create_task.rs
│   │   └── start_orchestration.rs
│   ├── queries/               // Query handlers
│   │   ├── get_agent_status.rs
│   │   └── list_active_tasks.rs
│   ├── services/              // Application services
│   │   └── orchestration_service.rs
│   └── dto/                   // Data Transfer Objects
│       ├── agent_dto.rs
│       └── task_dto.rs
```

### 3. Infrastructure Layer (`maos-io`)
**External concerns and adapters**

```rust
crates/maos-io/
├── src/
│   ├── persistence/
│   │   ├── sqlite/
│   │   │   ├── agent_repository_impl.rs
│   │   │   └── task_repository_impl.rs
│   │   └── memory/
│   │       └── in_memory_repository.rs
│   ├── messaging/
│   │   ├── event_bus_impl.rs
│   │   └── ipc_adapter.rs
│   ├── agents/                // Agent adapters
│   │   ├── claude_adapter.rs
│   │   ├── openai_adapter.rs
│   │   └── ollama_adapter.rs
│   └── config/
│       └── configuration.rs
```

### 4. Presentation Layer (`maos-cli`)
**User interfaces and API**

```rust
crates/maos-cli/
├── src/
│   ├── commands/
│   ├── formatters/
│   └── main.rs
```

## TDD Development Workflow

### Phase 1: Domain Model (Week 1-2)
**Red/Green/Refactor for each component**

1. **Agent Aggregate**
   ```rust
   // Start with failing test
   #[test]
   fn test_agent_registration() {
       // Red: Write failing test
       let agent = Agent::register(
           AgentId::new(),
           "claude-1",
           vec![Capability::CodeGeneration]
       );
       assert_eq!(agent.status(), AgentStatus::Ready);
   }
   ```

2. **Task Aggregate**
   ```rust
   #[test]
   fn test_task_assignment() {
       // Red: Test task can be assigned to capable agent
       let task = Task::create("Generate code", vec![Capability::CodeGeneration]);
       let agent = create_test_agent_with_capability(Capability::CodeGeneration);
       
       let result = task.assign_to(agent);
       assert!(result.is_ok());
   }
   ```

3. **Domain Events**
   ```rust
   #[test]
   fn test_task_completion_emits_event() {
       // Red: Test event emission
       let mut task = create_assigned_task();
       let events = task.complete(TaskResult::Success("Done".into()));
       
       assert!(events.contains(&DomainEvent::TaskCompleted { .. }));
   }
   ```

### Phase 2: Application Services (Week 3-4)
**Test use cases end-to-end**

```rust
#[tokio::test]
async fn test_create_task_use_case() {
    // Arrange
    let repo = MockTaskRepository::new();
    let event_bus = MockEventBus::new();
    let use_case = CreateTaskUseCase::new(repo, event_bus);
    
    // Act
    let result = use_case.execute(CreateTaskCommand {
        description: "Analyze code",
        required_capabilities: vec![Capability::CodeAnalysis],
    }).await;
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(repo.saved_tasks.len(), 1);
    assert_eq!(event_bus.published_events.len(), 1);
}
```

### Phase 3: Infrastructure Adapters (Week 5-6)
**Test adapters against contracts**

```rust
#[tokio::test]
async fn test_sqlite_agent_repository() {
    // Test repository implementation
    let pool = create_test_db_pool().await;
    let repo = SqliteAgentRepository::new(pool);
    
    let agent = create_test_agent();
    repo.save(&agent).await.unwrap();
    
    let loaded = repo.find_by_id(agent.id()).await.unwrap();
    assert_eq!(loaded, Some(agent));
}
```

## Development Process

### 1. Start with Domain Tests
```bash
# Create domain model test first
touch crates/maos-domain/tests/agent_aggregate_test.rs

# Red: Write failing test
# Green: Implement minimal code
# Refactor: Improve design
```

### 2. Build Outside-In
- Start with acceptance test
- Work down through layers
- Mock external dependencies
- Replace mocks with implementations

### 3. Continuous Integration
```yaml
# .github/workflows/ci.yml
- Run all tests on every commit
- Fail if coverage drops below 90%
- Run property-based tests
- Check for DDD violations
```

## Example Domain Model

```rust
// Value Object
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

// Aggregate Root
pub struct Agent {
    id: AgentId,
    name: String,
    capabilities: Vec<Capability>,
    status: AgentStatus,
    events: Vec<DomainEvent>,
}

impl Agent {
    // Factory method
    pub fn register(id: AgentId, name: String, capabilities: Vec<Capability>) -> Self {
        let mut agent = Self {
            id,
            name,
            capabilities,
            status: AgentStatus::Ready,
            events: vec![],
        };
        
        agent.record_event(DomainEvent::AgentRegistered {
            agent_id: agent.id.clone(),
            capabilities: agent.capabilities.clone(),
        });
        
        agent
    }
    
    // Domain logic
    pub fn can_handle(&self, required: &[Capability]) -> bool {
        required.iter().all(|cap| self.capabilities.contains(cap))
    }
    
    // Event sourcing
    fn record_event(&mut self, event: DomainEvent) {
        self.events.push(event);
    }
    
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }
}
```

## Project Structure (DDD-Aligned)

```
maos/
├── Cargo.toml
├── crates/
│   ├── maos-domain/          # Pure domain logic
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── tests/
│   ├── maos-app/            # Use cases
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── tests/
│   ├── maos-io/             # Technical concerns
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── tests/
│   ├── maos/               # Main binary
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── tests/
│   └── maos-tests/          # Integration tests
│       ├── Cargo.toml
│       └── tests/
├── docs/
│   ├── architecture/
│   ├── domain-model/
│   └── ubiquitous-language.md
└── .github/
    └── workflows/
        └── ci.yml
```

## Success Metrics

- **Code Coverage** > 90%
- **Test Execution** < 30 seconds
- **Domain Purity** - No I/O in domain layer
- **Clear Boundaries** - No layer violations
- **Event Sourcing** - Full audit trail
- **Performance** - Same as before

## Initial Commands

```bash
# Create the repository
gh repo create maos --public --description "Multi-Agent Orchestration System in Rust"
git clone https://github.com/[username]/maos
cd maos

# Initialize Rust project with workspace
cargo init --name maos

# Create domain layer first (TDD)
cargo new --lib crates/maos-domain
cd crates/maos-domain
cargo add uuid
cargo add thiserror
cargo add --dev tokio --features full
cargo add --dev proptest

# Start with first failing test
echo '#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_can_be_registered() {
        // This test should fail - no Agent type exists yet
        let agent_id = AgentId::new();
        let agent = Agent::register(
            agent_id,
            "test-agent".to_string(),
            vec![Capability::CodeGeneration]
        );
        
        assert_eq!(agent.name(), "test-agent");
        assert_eq!(agent.status(), AgentStatus::Ready);
    }
}' > tests/agent_aggregate_test.rs

# Run test (should fail)
cargo test
```

## Integration with Current Project

MAOS can integrate with your current agenterra project:
- Use agenterra's protocol definitions
- Import OpenAPI specs for agent communication
- Leverage existing MCP configurations
- The `.mcp.json` configuration can be used to register MAOS as an MCP server

## Next Steps

1. Create GitHub repository: `maos`
2. Initialize Rust workspace structure
3. Set up CI/CD with GitHub Actions and coverage requirements
4. Write first failing domain test
5. Begin Red/Green/Refactor cycle
6. Weekly progress reviews

---

*"Where's the earth-shattering kaboom?"* - It's in the perfectly orchestrated agent swarm!

*Created from Claude Code session*
*Session ID: [Session ID not available in environment]*
*Date: 2025-07-05*