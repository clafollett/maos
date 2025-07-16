# ADR-08: Agent Lifecycle and Management

## Status
Accepted

## Context
MAOS needs to manage the complete lifecycle of AI agent processes, from spawning to termination. With our streamlined architecture where a single Claude Code Agent manages multiple CLI processes, we need:

- Reliable CLI process spawning and monitoring
- Agent role specialization via `-p` flag
- Session continuity via `--session-id`
- Support for multiple concurrent sessions
- Resource management and process limits
- Graceful shutdown and cleanup
- Health monitoring and recovery
- Output capture and streaming

Key insights:
- Each agent is a Claude CLI process with role via `-p` flag
- Claude Code Agent (single ACP server) manages all CLI processes
- Session continuity preserved via `--session-id` flag
- Phases planned adaptively based on actual outputs
- Agents work independently without direct communication
- Orchestrator coordinates through Claude Code Agent
- Clean separation between MCP (external) and ACP (internal)

## Decision
We will implement streamlined agent lifecycle management where the Claude Code Agent (single ACP server) manages multiple Claude CLI processes with role-based specialization and session continuity.

### Architectural Layering

This ADR provides the process management infrastructure for the Claude Code Agent:

- **ADR-08 provides**: CLI process spawning, resource management, health monitoring, and lifecycle state management
- **ADR-04 uses**: This infrastructure within the Claude Code Agent for managing CLI processes
- **ADR-03 uses**: This infrastructure for session-level orchestration
- **ADR-05 provides**: CLI configurations and integration patterns for spawning
- **Relationship**: ADR-08 handles CLI process management, ADR-04 handles ACP communication, ADR-03 handles orchestration

### Unified State Model Integration

This ADR uses the unified state model for consistent agent lifecycle management across all orchestration layers. The process management layer uses `AgentExecutionState` for tracking individual agent lifecycle:

```
┌─────────┐      ┌─────────┐      ┌───────────┐
│ Pending │ ───► │ Running │ ───► │ Completed │
└─────────┘      └────┬────┘      └───────────┘
                      │
                      ├─────► ┌─────────┐
                      │       │ Failed  │
                      │       └─────────┘
                      │
                      ├─────► ┌───────────┐
                      │       │ Resumable │ (for recovery)
                      │       └───────────┘
                      │
                      └─────► ┌───────────┐
                              │ Cancelled │
                              └───────────┘
```

The process manager also tracks low-level process states separately for technical monitoring.

### Agent Role Definitions

Agent roles and templates are comprehensively documented in the [Agent Roles Reference](../references/agent-roles.md). This includes:

- **AgentRole structure**: Core properties for both predefined and custom roles
- **Predefined roles**: 13 built-in roles with specific capabilities and resource limits
- **AgentTemplate**: Runtime configuration including timeouts, memory limits, and required tools
- **Custom role support**: Dynamic template generation for user-defined roles

### Process Management Architecture

**Claude Code Agent Components:**
- **ACP Server**: Single server managing all CLI processes
- **Process Pool**: Managed Claude CLI processes with different roles
- **Session Manager**: Tracks session-to-process mappings
- **Health Monitor**: Monitors CLI process health
- **Resource Manager**: Controls concurrent process limits

**Key Management Areas:**
- **Process Spawning**: Create CLI processes with appropriate `-p` role flag
- **Session Binding**: Assign `--session-id` for context continuity
- **Resource Limits**: CPU, memory, and concurrent process limits
- **Health Monitoring**: Monitor CLI process status and responsiveness
- **Graceful Shutdown**: Clean termination of CLI processes

**CLI Process Lifecycle:**
1. **Task Receipt**: Orchestrator sends work to Claude Code Agent
2. **Process Selection**: Find or spawn CLI process for the role
3. **Session Assignment**: Use `--session-id` for context continuity
4. **Work Execution**: CLI process executes independently
5. **Result Collection**: Gather outputs for Orchestrator

### Process Spawning Architecture

The Claude Code Agent manages CLI process spawning through:

1. **Role Resolution**: Determine role from Orchestrator request
2. **Resource Validation**: Check system capacity before spawning
3. **Session Management**: Assign or reuse session ID for context
4. **Environment Setup**: Configure workspace and environment
5. **Process Spawning**: Start Claude CLI with `-p` role flag
6. **Session Binding**: Pass `--session-id` for continuity
7. **Monitoring Setup**: Track process health and output

**Process Identification**:
- Session ID serves as primary identifier
- Role and phase tracked for management
- No complex agent ID structure needed

**Key Parameters:**
- **Role Flag**: `-p architect`, `-p engineer`, etc.
- **Session ID**: `--session-id session_abc123`
- **Working Directory**: Isolated workspace per session
- **Environment Variables**: Standard Claude CLI configuration

### Process Lifecycle Patterns

**Phase-Based Execution**:
1. **Phase Start**: Orchestrator determines agents needed for the phase
2. **Multiple Requests**: Orchestrator sends one request per agent to Claude Code Agent
3. **Process Spawning**: Claude Code Agent spawns one CLI process per request
4. **Context Loading**: Each process uses `--session-id` for continuity
5. **Parallel/Sequential Work**: Agents execute based on Orchestrator's strategy
6. **Individual Results**: Each agent returns results independently
7. **Phase Aggregation**: Orchestrator collects all results before next phase
8. **Next Phase Planning**: Orchestrator plans based on aggregated outputs

**Session Continuity Benefits**:
- **Perfect Memory**: CLI processes retain full context via session ID
- **Adaptive Planning**: Each phase planned based on real outputs
- **Resource Efficiency**: Reuse processes across phases
- **Clean Handoffs**: Work naturally flows between phases

**Resource Management:**
- **Process Pool**: Limit concurrent CLI processes
- **Session Mapping**: One session ID per logical agent
- **Graceful Scaling**: Add processes as needed within limits

### Session Management with Claude Code Agent

**Session Registry Pattern:**

The Claude Code Agent maintains a session registry for context persistence:

```
Session Registry:
┌─────────────────────────────────────────────────────────────┐
│ session_id       → role        → status     → phase        │
├─────────────────────────────────────────────────────────────┤
│ session_abc123   → researcher  → [IDLE]     → Phase 1      │
│ session_def456   → architect   → [ACTIVE]   → Phase 2      │
│ session_ghi789   → engineer    → [IDLE]     → Phase 2      │
│ session_jkl012   → engineer    → [ACTIVE]   → Phase 3      │
│ session_mno345   → engineer    → [IDLE]     → Phase 2      │
│ session_pqr678   → qa          → [IDLE]     → Phase 3      │
└─────────────────────────────────────────────────────────────┘
```

**Session Lifecycle:**
1. **Work Request**: Orchestrator requests work for specific role
2. **Session Selection**: Find existing session or create new one
3. **Process Spawning**: Start CLI with `--session-id`
4. **Work Execution**: Process works independently
5. **Result Return**: Outputs sent back to Orchestrator

**Context Preservation:**
- **Session Continuity**: `--session-id` preserves all context
- **Phase Awareness**: Each phase builds on previous work
- **Efficient Reuse**: Same session across multiple phases
- **Clean Isolation**: Sessions don't interfere with each other

**Process States:**
- **[ACTIVE]**: CLI process currently running
- **[IDLE]**: No active process, session preserved
- **[TERMINATED]**: Session permanently closed

### Health Monitoring

The Claude Code Agent monitors CLI process health through:
- **Process Status**: Regular checks that CLI processes are running
- **Output Monitoring**: Track process output for errors or stalls
- **Resource Usage**: Monitor memory and CPU per process
- **Session Health**: Ensure session IDs remain valid
- **Timeout Detection**: Identify stuck or unresponsive processes
- **Lifecycle Events**: Log process starts, stops, and errors

### Template Generation for Custom Roles

Template generation for custom roles is handled by the TemplateGenerator, which automatically:
- Infers capabilities from the role description
- Determines required tools based on capabilities
- Generates appropriate prompt templates

The complete template generation logic and prompt format are documented in the [Agent Roles Reference](../references/agent-roles.md#custom-role-support).

### Resource Management

The Claude Code Agent enforces resource limits:

**Resource Types**:
- **Process Limit**: Maximum concurrent CLI processes
- **Memory Limits**: Total memory allocation across processes
- **Session Limits**: Maximum active sessions
- **CPU Limits**: Process CPU usage constraints

**Resource Validation**:
- Check available capacity before spawning
- Queue requests when at capacity
- Gracefully handle resource exhaustion
- Clean up idle processes when needed

### Graceful Shutdown

The Claude Code Agent handles shutdown cleanly:

1. **Process Termination**: Send shutdown signal to CLI process
2. **Timeout Waiting**: Allow time to finish current work
3. **Force Termination**: Kill unresponsive processes
4. **Session Preservation**: Keep session IDs for resumption
5. **Resource Cleanup**: Clean up handles and workspace

**Shutdown Modes**:
- **Process Shutdown**: Terminate specific CLI process
- **Session Shutdown**: Close session permanently
- **System Shutdown**: Stop all processes and Claude Code Agent

## Consequences

### Positive
- **Simplified Architecture**: Single ACP server manages all processes
- **Perfect Context**: Session IDs preserve complete context
- **Resource Efficiency**: Reuse processes across phases
- **Clean Abstraction**: Claude Code Agent hides complexity
- **Easy Extension**: Simple to add other CLI tools
- **Phase-Based Clarity**: Clean work boundaries
- **Reduced Overhead**: One ACP server instead of many
- **Better Debugging**: Centralized process management

### Negative
- **Process Management**: Must handle multiple CLI processes
- **Session Tracking**: Need to manage session-to-process mapping
- **Platform Differences**: Resource limits vary by OS
- **Recovery Complexity**: Restarting failed processes carefully

### Mitigation
- **Process Pool**: Efficient CLI process lifecycle management
- **Session Registry**: Clean mapping of sessions to processes
- **Platform Adaptation**: OS-specific resource handling
- **Robust Recovery**: Leverage --session-id for resumption

## References
- **ADR-04: ACP-Based Agent Communication** - Defines the ACP integration that this ADR implements
- **ADR-03: Session Orchestration and State Management** - Uses this process management infrastructure for session-level coordination
- **ADR-05: CLI Integration and Process Spawning** - CLI-specific spawning patterns
- [Agent Communication Protocol (ACP)](https://agentcommunicationprotocol.dev/) - Core communication protocol for agent servers
- Process supervision patterns and best practices
- Tokio process management documentation
- Linux resource limits (setrlimit)
- Container orchestration patterns
- HTTP server lifecycle management

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*