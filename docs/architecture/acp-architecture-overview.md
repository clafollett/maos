# ACP Architecture Overview

## Introduction

This document provides a comprehensive overview of how MAOS implements the Agent Communication Protocol (ACP) for multi-agent orchestration. Our architecture uses ACP to enable clean separation between external interfaces (MCP) and internal agent coordination.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Claude Code (MCP Client)                     │
│                                                                 │
│  maos/orchestrate ──► Start orchestration session              │
│  maos/session-status ──► Monitor progress                      │
│  maos/list-roles ──► List available agent roles               │
└─────────────────────┬───────────────────────────────────────────┘
                      │ MCP Protocol (External Interface)
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                     MAOS MCP Server                            │
│                                                                 │
│  • Exposes 3 core MCP tools                                    │
│  • Manages orchestration sessions                              │
│  • Streams Orchestrator output to Claude Code                  │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Spawns Orchestrator Process
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│            Orchestrator (Router Agent) - ACP Server            │
│                                                                 │
│  • Plans phases adaptively based on previous outputs           │
│  • Uses Claude for intelligent agent selection                 │
│  • Maintains session registry for context continuity           │
│  • Routes work to Claude Code Agent via ACP                    │
└─────────────────────┬───────────────────────────────────────────┘
                      │ ACP Protocol (Internal Coordination)
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│             Claude Code Agent - ACP Server                     │
│                                                                 │
│  • Single ACP server managing multiple CLI processes           │
│  • Intelligent session assignment and reuse                    │
│  • Process lifecycle management                                │
│  • Resource monitoring and cleanup                             │
│                                                                 │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     │
│  │ Claude CLI  │     │ Claude CLI  │     │ Claude CLI  │     │
│  │ -p architect│     │ -p backend  │     │ -p frontend │     │
│  │ --session-id│     │ --session-id│     │ --session-id│     │
│  └─────────────┘     └─────────────┘     └─────────────┘     │
└─────────────────────────────────────────────────────────────────┘
```

## ACP Protocol Usage

### Protocol Overview
The Agent Communication Protocol (ACP) is a REST-based protocol designed for agent-to-agent communication. In MAOS, we use ACP for internal coordination between the Orchestrator and Claude Code Agent.

### Our ACP Implementation

**Key Design Decision**: We implement ACP as a **point-to-point protocol** rather than a complex network:
- **Orchestrator** ↔ **Claude Code Agent** (single ACP connection)
- No peer-to-peer agent communication
- Clean, simple request/response pattern

### ACP Message Flow

1. **Phase Planning**: Orchestrator uses Claude to plan next phase
2. **Agent Selection**: Orchestrator uses Claude to select optimal agent
3. **Work Request**: Orchestrator sends ACP request to Claude Code Agent
4. **Process Management**: Claude Code Agent spawns/reuses CLI process
5. **Work Execution**: CLI process executes independently
6. **Result Return**: Claude Code Agent returns results via ACP
7. **Next Phase**: Orchestrator plans next phase based on results

## Multi-Agent Single Server Pattern

### Design Rationale
Instead of implementing "every agent = ACP server" (complex), we chose "single server manages all agents" (simple):

**Benefits:**
- **Simplified Architecture**: One ACP server instead of many
- **Efficient Resource Use**: Shared infrastructure for all agents
- **Centralized Control**: Unified process management
- **Easy Debugging**: Single point of coordination
- **Reduced Complexity**: No peer-to-peer networking

### Session Continuity
The key insight is leveraging Claude's `--session-id` flag for context preservation:

```
Phase 1: architect_1 works on API design    (session_abc123)
Phase 2: backend_eng_1 implements APIs      (session_def456)
Phase 3: architect_1 reviews implementation (session_abc123) ← Same session!
```

The architect in Phase 3 has full memory of the original design decisions.

## Intelligent Agent Selection

### Problem
How does the Orchestrator decide which agent to use for a new task?

### Solution
The Orchestrator uses Claude to make intelligent decisions:

```rust
// Orchestrator's decision process
let decision = orchestrator_claude.ask(&format!(
    "Given this session registry:
    architect_1: api_design, system_architecture
    backend_eng_1: auth_service, oauth_implementation
    backend_eng_2: user_service, crud_apis
    
    New task: 'Add password reset to authentication'
    
    Which agent should handle this? Consider expertise and context."
));
```

**Claude's Analysis:**
- Understands "password reset" relates to "authentication"
- Knows backend_eng_1 worked on auth_service
- Recommends reusing backend_eng_1 for context continuity

### Benefits
- **Semantic Understanding**: Claude understands component relationships
- **Context Optimization**: Reuses agents with relevant experience
- **Load Balancing**: Distributes work intelligently
- **Adaptive Learning**: Improves decisions based on work history

## Phase-Based Orchestration

### Adaptive Planning
Unlike traditional project management, MAOS plans phases adaptively:

```
Traditional: Plan entire project → Execute all phases
MAOS: Plan phase → Execute → Learn → Plan next phase
```

### Phase Execution Pattern
1. **Orchestrator Plans**: Uses Claude to determine next phase and agents needed
2. **Agent Selection**: Uses Claude to pick optimal agents for each task
3. **Work Distribution**: Sends individual ACP requests to Claude Code Agent
4. **Parallel/Sequential**: Orchestrator manages execution strategy
5. **Result Aggregation**: Collects all results before planning next phase

### Example Flow
```
Phase 1: Research (1 agent)
├── researcher_1 → "Analyze authentication requirements"
└── Result: "OAuth2 + MFA recommended"

Phase 2: Design (2 agents in parallel)
├── architect_1 → "Design auth service architecture"
├── security_1 → "Define security requirements"
└── Results: API spec + security guidelines

Phase 3: Implementation (3 agents in parallel)
├── backend_eng_1 → "Implement auth service" (uses architect_1's design)
├── frontend_eng_1 → "Build auth UI" (uses security_1's requirements)
├── qa_1 → "Write integration tests"
└── Results: Working authentication system
```

## Session Registry Design

### Registry Structure

| agent_id        | session_id      | role          | work_context    |
|-----------------|-----------------|---------------|-----------------|
| architect_1     | session_abc123  | architect     | api_design, ... |
| backend_eng_1   | session_def456  | backend_eng   | auth_service    |
| backend_eng_2   | session_ghi789  | backend_eng   | user_service    |
| frontend_eng_1  | session_jkl012  | frontend_eng  | auth_ui         |
| qa_1            | session_pqr678  | qa            | api_tests       |

### Intelligent Reuse
The Claude Code Agent can intelligently reuse sessions:

**Exact Match**: Need auth work → Use backend_eng_1 (auth expert)
**Related Work**: Need user profiles → Use backend_eng_2 (user expert)
**New Component**: Need payment service → Create backend_eng_3

## Error Handling and Recovery

### Process Failures
- **Health Monitoring**: Regular checks of CLI processes
- **Automatic Restart**: Restart failed processes when possible
- **Session Recovery**: Attempt to recover using existing session ID
- **Graceful Degradation**: Continue with reduced capacity

### Network Issues
- **ACP Retry Logic**: Automatic retry with exponential backoff
- **Timeout Handling**: Reasonable timeouts for all requests
- **Circuit Breaker**: Prevent cascading failures
- **Status Reporting**: Clear error messages to Orchestrator

## Performance Considerations

### Optimization Strategies
- **Process Reuse**: Reuse CLI processes for similar work
- **Session Caching**: Cache session metadata for fast lookups
- **Lazy Loading**: Only load data when needed
- **Resource Monitoring**: Track and optimize resource usage

### Scalability
- **Horizontal Scaling**: Can run multiple Claude Code Agents
- **Load Distribution**: Balance work across available processes
- **Resource Limits**: Configurable limits prevent resource exhaustion
- **Monitoring**: Comprehensive metrics for optimization

## Security Model

### Process Isolation
- **Sandboxed Processes**: Each CLI process runs in isolated environment
- **Resource Limits**: CPU, memory, and network limits per process
- **Workspace Isolation**: Separate workspaces for different sessions
- **Access Control**: Validate all requests before process creation

### Session Security
- **Session Isolation**: Sessions cannot access each other's data
- **Secure Storage**: Encrypted storage of session metadata
- **Audit Logging**: Log all process creation and termination
- **Data Protection**: Secure handling of sensitive information

## Monitoring and Observability

### Key Metrics
- **Active Sessions**: Number of active agent sessions
- **Process Health**: Status of all CLI processes
- **Resource Usage**: CPU, memory, and network utilization
- **Request Latency**: Time to process ACP requests
- **Error Rates**: Frequency of process failures and errors

### Logging Strategy
- **Structured Logs**: JSON-formatted logs for easy parsing
- **Correlation IDs**: Track requests across system components
- **Performance Metrics**: Detailed timing information
- **Error Details**: Comprehensive error information for debugging

## Future Enhancements

### Potential Improvements
- **Multiple CLI Tools**: Support for other CLI tools beyond Claude
- **Advanced Scheduling**: Sophisticated work scheduling algorithms
- **Predictive Scaling**: Anticipate resource needs based on workload
- **Machine Learning**: Learn from past orchestrations to improve decisions

### Extension Points
- **Plugin System**: Allow custom agent types and capabilities
- **Custom Protocols**: Support for other communication protocols
- **Integration APIs**: APIs for external system integration
- **Workflow Templates**: Reusable orchestration patterns

## References

- **ADR-04**: Agent Communication Patterns - Core ACP architecture decisions
- **ADR-08**: Agent Lifecycle Management - Process management details
- **ADR-10**: MCP Server Architecture - External interface design
- **ADR-11**: Adaptive Phase-Based Orchestration - Intelligent coordination
- [ACP Specification](https://agentcommunicationprotocol.dev/) - Official ACP protocol documentation
- [claude_code_agent.rs](./references/examples/claude_code_agent.rs) - Implementation example
- [intelligent_agent_selection.rs](./references/examples/intelligent_agent_selection.rs) - Agent selection example

---

*Last Updated: 2025-07-16*
*Author: Development Team*