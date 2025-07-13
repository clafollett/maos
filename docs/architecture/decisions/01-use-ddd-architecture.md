# ADR-01: Use Domain-Driven Design Architecture

## Status
Accepted

## Context
MAOS is a multi-agent orchestration system implemented as an MCP (Model Context Protocol) server that:
- Spawns and manages AI CLI processes (`claude -p`, etc.)
- Coordinates agent communication through shared file systems
- Tracks session state and agent lifecycles
- Provides real-time updates via SSE streaming
- Supports multiple concurrent instances

We need an architectural approach that balances the complexity of orchestration with the simplicity required for reliable process management.

## Decision
We will use Domain-Driven Design (DDD) principles adapted for our MCP server architecture:

1. **Domain Layer** (`maos-domain`) - Core orchestration logic
   - Aggregates: Session, Agent, Instance
   - Value Objects: AgentRole, SessionId, AgentId
   - Domain Events: SessionCreated, AgentSpawned, AgentCompleted
   - Domain Services: DependencyResolver, AgentScheduler

2. **Application Layer** (`maos-application`) - MCP tool handlers
   - Tool Handlers: OrchestrateHandler, SpawnAgentHandler
   - Query Handlers: GetSessionStatus, ListAgents
   - Application Services: SessionManager, ProcessManager

3. **Infrastructure Layer** (`maos-infrastructure`) - Technical implementation
   - Storage: SQLite + File System (see ADR-02)
   - Process Management: CLI spawning and monitoring
   - MCP Server: HTTP/SSE protocol implementation
   - Communication: File-based message routing

4. **Presentation Layer** (`maos-server`) - MCP interface
   - MCP Tool Definitions
   - SSE Stream Handlers
   - Resource Providers
   - Instance Management

## Consequences

### Positive
- **Clear Separation of Concerns**: Orchestration logic isolated from MCP protocol details
- **Testability**: Domain logic can be tested without spawning real processes
- **Maintainability**: Each layer has focused responsibilities
- **Extensibility**: New CLI types and roles easily added
- **Pragmatic Approach**: DDD principles without over-engineering
- **Process Isolation**: Clean boundaries between MAOS and agent processes

### Negative
- **Simplified Domain Model**: Less complex than traditional DDD due to process boundaries
- **File System Dependency**: Communication patterns tied to file system
- **Limited Aggregate Boundaries**: Processes can't share memory/transactions

### Risks and Mitigations
- **Risk**: Domain logic leaking into infrastructure
  - **Mitigation**: Keep process spawning details in infrastructure layer
- **Risk**: Over-abstracting simple file operations
  - **Mitigation**: Use straightforward file I/O where appropriate

## Alternatives Considered

### Layered Architecture (Traditional N-Tier)
- **Pros**: Simpler, well-understood
- **Cons**: Tends toward anemic domain models, harder to maintain business logic

### Hexagonal Architecture (Ports and Adapters)
- **Pros**: Good separation of concerns
- **Cons**: Less guidance on internal structure, doesn't address domain complexity

### Microservices Architecture
- **Pros**: High scalability, independent deployment
- **Cons**: Too complex for initial implementation, unnecessary network overhead

## Implementation Guidelines

1. **Domain Layer Rules**:
   - No dependencies on MCP protocol or process management
   - Session and Agent aggregates maintain orchestration state
   - Domain events for significant state changes (logged, not sourced)
   - Focus on orchestration logic, not technical details

2. **Test Strategy**:
   - Domain logic: Pure unit tests without file I/O
   - Application services: Test with mock process manager
   - Infrastructure: Integration tests with real file system
   - End-to-end: Test with actual CLI processes

3. **Dependency Direction**:
   - Domain ← Application ← Infrastructure
   - Domain ← Server (MCP interface)
   - No circular dependencies

4. **Pragmatic Choices**:
   - SQLite for state (not event sourcing) - see ADR-02
   - Simple logging instead of complex events - see ADR-06
   - File-based communication over IPC - see ADR-04
   - Process spawning over embedded agents - see ADR-05

## References
- [Domain-Driven Design by Eric Evans](https://www.amazon.com/Domain-Driven-Design-Tackling-Complexity-Software/dp/0321125215)
- [Clean Architecture by Robert Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [DDD Community](https://github.com/ddd-crew)

---
*Date: 2025-07-09*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*