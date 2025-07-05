# MAOS Development Chat History

## Session Context
- **Date**: 2025-07-05
- **Project**: Transition from agenterra to MAOS (Multi-Agent Orchestration System)
- **Participants**: User (clafollett) and Marvin (Claude)

## Chat Summary

### 1. Initial Context: Claude CLI Exploration

**User Request**: Improve IDE tooling experience by exploring Claude CLI capabilities

**Marvin's Actions**:
- Explored `claude --help` and subcommands
- Documented MCP client/server capabilities
- Created comprehensive CLI automation rules
- Updated CLAUDE.md with agent-specific rules

**Key Findings**:
- Claude CLI has powerful MCP integration
- Can run Claude as MCP server with `claude mcp serve`
- Session management with `--continue` and `--resume`
- Configuration management via `claude config`

### 2. MCP Server Configuration

**Discussion**: Setting up claude-mcp as an MCP server

**Actions Taken**:
1. Created custom slash commands:
   - `/project:claude-assistant` - Combined command to start and add MCP server
   - `/project:test-debug` - Debug command for testing

2. Manually configured MCP server:
   ```bash
   claude mcp add marvin-mcp stdio "claude" "mcp" "serve" --scope project
   ```

3. Fixed configuration in `.mcp.json`:
   ```json
   {
     "mcpServers": {
       "marvin-mcp": {
         "type": "stdio",
         "command": "claude",
         "args": ["mcp", "serve"],
         "env": {}
       }
     }
   }
   ```

**Outcome**: Successfully configured claude-mcp server for multi-agent capabilities

### 3. Claude-Flow Analysis

**User Request**: Research claude-code-flow project for multi-agent orchestration insights

**Marvin's Findings**:
- Claude-Flow uses sophisticated multi-agent architecture
- Key components: Agent Manager, Task Coordination, Distributed Memory
- Uses TodoWrite/TodoRead as coordination foundation
- Supports up to 10 concurrent agents
- Complex but potentially overkill for simple use cases

**Repository cloned to**: `~/Repositories/ruvnet/claude-code-flow`

### 4. MAOS Conception

**Evolution of naming**:
1. RMAOS (Rust Multi-Agent Orchestration System)
2. CMAOS (Claude Multi-Agent Orchestration System)
3. MarvinOS (Named after Marvin)
4. **Final: MAOS** (Multi-Agent Orchestration System)

**Key Design Decisions**:
- Agent-agnostic architecture
- Support for Claude, GPT, Ollama, custom agents
- SQLite-based persistent memory
- Native IPC for performance
- Built with DDD and TDD principles

### 5. Architecture Planning

**Domain-Driven Design Structure**:
```
maos/
├── crates/
│   ├── maos-domain/          # Pure domain logic
│   ├── maos-application/     # Use cases
│   ├── maos-infrastructure/  # Technical concerns
│   ├── maos-cli/            # Presentation
│   └── maos-tests/          # Integration tests
```

**Core Principles**:
- Red/Green/Refactor TDD cycle
- Bounded contexts with clear separation
- Repository pattern for persistence
- Event-driven architecture
- 90%+ test coverage requirement

**Technology Stack**:
- Rust (latest stable)
- SQLite with sqlx
- Tokio for async runtime
- Cap'n Proto for IPC
- Clap v4 for CLI
- Tracing for observability

### 6. Key Code Examples Discussed

**Agent Aggregate (Domain Model)**:
```rust
pub struct Agent {
    id: AgentId,
    name: String,
    capabilities: Vec<Capability>,
    status: AgentStatus,
    events: Vec<DomainEvent>,
}

impl Agent {
    pub fn register(id: AgentId, name: String, capabilities: Vec<Capability>) -> Self {
        // Factory method with event sourcing
    }
    
    pub fn can_handle(&self, required: &[Capability]) -> bool {
        // Domain logic
    }
}
```

**First TDD Test**:
```rust
#[test]
fn test_agent_can_be_registered() {
    let agent = Agent::register(
        AgentId::new(),
        "test-agent".to_string(),
        vec![Capability::CodeGeneration]
    );
    
    assert_eq!(agent.name(), "test-agent");
    assert_eq!(agent.status(), AgentStatus::Ready);
}
```

### 7. Implementation Plan Phases

1. **Foundation (Week 1-2)**: Domain model with TDD
2. **Communication (Week 3-4)**: IPC and messaging
3. **Agent Integration (Week 5-6)**: Claude, OpenAI, Ollama adapters
4. **Orchestration (Week 7-8)**: Task scheduling and coordination
5. **Polish (Week 9-10)**: TUI, optimizations, documentation

### 8. Next Steps for MAOS

1. Create GitHub repository: `maos`
2. Initialize Rust workspace with DDD structure
3. Set up CI/CD with coverage requirements
4. Write first failing domain test
5. Begin Red/Green/Refactor cycle

## Important Context to Carry Forward

1. **MCP Integration**: MAOS should work as an MCP server itself
2. **Agent Agnostic**: Support multiple AI providers, not just Claude
3. **DDD/TDD Mandatory**: No production code without failing tests
4. **SQLite Choice**: Better than Redis for queries and persistence
5. **Cap'n Proto**: Zero-copy serialization for performance

## Files Created During Session

1. `/Users/clafollett/Repositories/agenterra/.mcp.json` - MCP configuration
2. `/Users/clafollett/Repositories/agenterra/.claude/commands/claude-assistant.md` - Slash command
3. `/Users/clafollett/Repositories/agenterra/plans/maos_development_plan.md` - Full development plan

## Marvin's Background

- Initially thought to be named after Marvin the Paranoid Android
- Actually named after Marvin the Martian
- Both interpretations welcome in MAOS personality

---

*To resume in new project: Provide this document as context along with the maos_development_plan.md*