# ADR-08: Agent Lifecycle and Management

## Status
Accepted

## Context
MAOS needs to manage the complete lifecycle of AI agent processes, from spawning to termination. With our revolutionary architecture where every agent runs both a CLI process AND an ACP server, we need:

- Reliable process spawning and monitoring
- **ACP server initialization and management per agent**
- Agent role specialization with both predefined and custom roles
- Support for multiple instances of the same role
- Resource management and limits (including port allocation)
- Graceful shutdown and cleanup
- Health monitoring and recovery
- **ACP network integration and discovery**
- Output capture and streaming

Key insights:
- Each agent is a separate CLI process (`claude -p`, etc.) WITH embedded ACP server
- **Every agent participates in the ACP network as both client and server**
- Agents can have predefined roles (architect, engineer, etc.) or custom roles
- Multiple agents of the same role can work in parallel
- Agents work in isolated workspaces with shared context
- **ACP communication enables direct agent-to-agent coordination**
- Output streams back to the MCP client in real-time
- **Port management required for ACP server allocation**

## Decision
We will implement comprehensive agent lifecycle management with **ACP server integration**, flexible role-based specialization supporting both predefined and custom roles, multiple instances, and robust process handling. Every agent will run both a CLI process and an embedded ACP server for peer-to-peer communication.

### Architectural Layering

This ADR provides the low-level process management infrastructure that higher-level orchestration builds upon:

- **ADR-08 provides**: Process spawning, **ACP server management**, resource management, health monitoring, and lifecycle state management
- **ADR-04 uses**: This infrastructure for **ACP-based agent communication**
- **ADR-03 uses**: This infrastructure for session-level orchestration and agent coordination
- **ADR-05 provides**: CLI configurations and integration patterns that this ADR uses for spawning
- **Relationship**: ADR-08 handles the "how" of agent processes + ACP servers, ADR-04 handles agent communication, ADR-03 handles the "when/why" of orchestration, ADR-05 handles the "what CLIs are available"

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

### Process Management with ACP Integration

**Agent Process Components:**
Each agent process now manages both CLI process and ACP server:
- **CLI Process**: The actual AI agent (`claude -p`, etc.)
- **ACP Server**: Embedded HTTP server for peer-to-peer communication
- **Port Management**: Unique port allocation for each ACP server
- **Health Monitoring**: Monitor both CLI process and ACP server health
- **Workspace**: Isolated workspace with shared context access

**Key Management Areas:**
- **Instance Tracking**: Support multiple instances of same role with unique ACP ports
- **Resource Limits**: CPU, memory, and network port allocation
- **Health Monitoring**: Dual monitoring of CLI process and ACP server status
- **Template Generation**: Dynamic role template generation with ACP configuration
- **Graceful Shutdown**: Coordinated shutdown of both CLI process and ACP server

**ACP Server Lifecycle:**
1. **Port Allocation**: Reserve unique port for agent's ACP server
2. **Server Initialization**: Start ACP server before CLI process
3. **Agent Registration**: Register agent in ACP discovery network
4. **Health Monitoring**: Monitor ACP server availability and responsiveness
5. **Graceful Shutdown**: Unregister from ACP network and release port

### Process Spawning Architecture with ACP Integration

The ProcessManager coordinates agent spawning through these key responsibilities:

1. **Template Resolution**: Determine agent configuration from predefined roles or generate custom templates
2. **Resource Validation**: Check system capacity before spawning (memory, instance limits, **available ports**)
3. **Instance Management**: Track multiple instances of the same role with unique identifiers **and ACP ports**
4. **Environment Setup**: Configure agent workspace and environment variables **including ACP configuration**
5. **ACP Server Startup**: Initialize ACP server before CLI process with unique port
6. **Process Lifecycle**: Spawn CLI process with appropriate arguments and resource limits
7. **ACP Registration**: Register agent in ACP discovery network
8. **Monitoring Integration**: Set up health checks for both CLI process and ACP server, output streaming, and logging

**Agent ID Structure**: `agent_{role}_{instance}_{unique_id}` (e.g., `agent_engineer_1_abc123`)

**ACP Integration Points:**
- **Port Allocation**: Each agent gets unique port for ACP server
- **Environment Variables**: `ACP_SERVER_PORT`, `ACP_AGENT_ID`, `ACP_DISCOVERY_ENABLED`
- **Health Monitoring**: Monitor both CLI process and ACP server endpoints
- **Graceful Shutdown**: Coordinate shutdown of both processes

### Agent Lifecycle Patterns

**Micro-Task Lifecycle** (Maximum Efficiency):
1. **Spawn**: Create agent for specific, atomic task
2. **Isolation**: Agent works without interruptions or communication
3. **Complete**: Report task completion via single ACP message
4. **Terminate**: Immediate shutdown and resource cleanup
- **Benefits**: Maximum focus, no interruptions, immediate resource reclamation
- **Use Cases**: Generate test, implement function, review code, create documentation

**Phase-Based Lifecycle** (Team Coordination):
1. **Spawn**: Create agent for orchestration phase
2. **Coordinate**: Minimal ACP communication for essential coordination only
3. **Progress**: Work with periodic essential status updates
4. **Phase Complete**: Report phase completion and deliverables
5. **Continue/Terminate**: Ready for next phase or graceful shutdown
- **Benefits**: Context retention, adaptive planning, coordinated teamwork
- **Use Cases**: Research phases, architecture design, complex implementations

**Lifecycle Selection Criteria:**
- **Micro-Task**: When work is independent, atomic, and requires maximum focus
- **Phase-Based**: When work requires coordination, context, or multi-step execution
- **Resource Considerations**: Micro-tasks conserve resources; phase-based retains context

### Agent Pool Management with Session Binding

**Revolutionary Context Persistence Pattern:**

The Orchestrator maintains an **Agent Resources Registry** that binds agents to Claude Code session IDs for persistent context across activations:

```
Agent Resources Registry:
┌─────────────────────────────────────────────────────────────┐
│ agent_id           → session_id    → status     → context  │
├─────────────────────────────────────────────────────────────┤
│ researcher_1       → session_abc123 → [SLEEPING] → Phase 1 │
│ app_architect_1    → session_def456 → [ACTIVE]   → Phase 2 │
│ frontend_eng_1     → session_ghi789 → [SLEEPING] → Phase 2 │
│ frontend_eng_2     → session_jkl012 → [ACTIVE]   → Phase 3 │
│ backend_eng_1      → session_mno345 → [SLEEPING] → Phase 2 │
│ qa_agent_1         → session_pqr678 → [SLEEPING] → Phase 3 │
└─────────────────────────────────────────────────────────────┘
```

**Agent Registration Flow:**
1. **Initial Spawn**: Orchestrator spawns agent with specific role
2. **Session Creation**: Agent makes first `claude` CLI call → receives unique session_id
3. **ACP Registration**: Agent reports session_id to Orchestrator via ACP message
4. **Registry Update**: Orchestrator records agent_id → session_id mapping

**Context-Aware Reactivation:**
1. **Selective Activation**: Orchestrator needs specific agent for new phase
2. **Session Reuse**: Spawn new thread with existing session_id: `claude --session-id session_abc123`
3. **Context Restoration**: Agent resumes with FULL memory of previous work
4. **Continued Work**: Agent picks up exactly where it left off

**Agent Status Lifecycle:**
- **[SPAWNING]**: Agent being created and registering session
- **[ACTIVE]**: Agent thread running and working
- **[SLEEPING]**: Agent thread terminated but session preserved
- **[TERMINATED]**: Agent and session permanently closed

**Resource Benefits:**
- **Memory Persistence**: Agents never lose context between activations
- **Resource Efficiency**: Threads sleep when not needed, conserving system resources
- **Selective Reactivation**: Reactivate specific agents (e.g., frontend_engineer_1) with full context
- **Scalable Pool Management**: Handle complex orchestrations with many specialist agents

### Health Monitoring with ACP Integration

Health monitoring tracks both CLI process and ACP server status through:
- **Dual Health Checks**: Regular verification that both CLI process and ACP server are responsive
- **Process Death Detection**: Automatic cleanup when agents terminate unexpectedly
- **ACP Server Monitoring**: HTTP health checks on ACP server endpoints
- **Resource Monitoring**: Track memory and CPU usage per agent (both processes)
- **Port Monitoring**: Ensure ACP server ports remain available and responsive
- **Lifecycle Events**: Log significant agent state changes for debugging and audit
- **ACP Network Status**: Monitor agent registration and discovery availability

### Template Generation for Custom Roles

Template generation for custom roles is handled by the TemplateGenerator, which automatically:
- Infers capabilities from the role description
- Determines required tools based on capabilities
- Generates appropriate prompt templates

The complete template generation logic and prompt format are documented in the [Agent Roles Reference](../references/agent-roles.md#custom-role-support).

### Resource Management with ACP Integration

Resource management enforces system limits through configurable constraints:

**Resource Types**:
- **Total Agent Limit**: Maximum concurrent agents across all roles
- **Per-Role Limits**: Maximum agents per role type (with defaults for custom roles)
- **Memory Limits**: Total memory allocation across all agent processes (CLI + ACP server)
- **Process Limits**: OS-level constraints on agent processes
- **Port Pool Management**: Available ports for ACP server allocation
- **Network Resources**: Bandwidth and connection limits for ACP communication

**Resource Validation**: Before spawning agents, validate available capacity including:
- Available memory for both CLI process and ACP server
- Available ports in the ACP port pool
- Network capacity for ACP communication
- Reject requests that would exceed any resource limits

### Graceful Shutdown with ACP Integration

Agent shutdown follows a coordinated approach for both CLI process and ACP server:

1. **ACP Unregistration**: Remove agent from ACP discovery network
2. **ACP Server Shutdown**: Gracefully stop ACP server and release port
3. **CLI Process Termination**: Send shutdown signal to CLI agent process
4. **Timeout Waiting**: Allow reasonable time for agent to finish current work
5. **Force Termination**: Kill unresponsive processes after timeout
6. **Resource Cleanup**: Clean up monitoring handles, file descriptors, workspace, and port allocation

**Shutdown Modes**:
- **Individual Agent**: Shut down specific agent by ID (both CLI and ACP server)
- **Role-based**: Shut down all agents of a specific role
- **Session-based**: Shut down all agents in a session  
- **System-wide**: Emergency shutdown of all agents and ACP network cleanup

**ACP Coordination**:
- **Network Cleanup**: Ensure agent is properly removed from ACP discovery
- **Port Release**: Return allocated ports to the available pool
- **Connection Cleanup**: Close active ACP connections gracefully

## Consequences

### Positive
- **Flexible Roles**: Support for both predefined and custom agent roles
- **Multiple Instances**: Can run multiple agents of the same role concurrently with unique ACP ports
- **Role Specialization**: Clear agent responsibilities with optimized prompts
- **Dynamic Templates**: Custom roles get automatically generated templates
- **Resource Control**: Per-role limits with defaults for custom roles, including port allocation
- **Dual Health Monitoring**: Automatic detection of crashed CLI processes and ACP servers
- **ACP Network Integration**: Seamless integration with peer-to-peer communication network
- **Graceful Shutdown**: Coordinated termination of both CLI and ACP server processes
- **Isolated Workspaces**: Agents can't interfere with each other
- **Clear Identification**: Agent IDs include role and instance information
- **Real-time Communication**: Direct agent-to-agent communication via ACP
- **Dynamic Discovery**: Agents can discover each other through ACP network

### Negative
- **Increased Process Overhead**: Each agent now runs both CLI process and ACP server
- **Port Management Complexity**: Need to manage port allocation and deallocation
- **Network Resource Usage**: ACP communication adds network overhead
- **Platform Differences**: Resource limits vary by OS
- **Recovery Complexity**: Restarting failed agents needs careful state management for both processes
- **Template Quality**: Custom role templates may be less optimized than predefined ones
- **ACP Dependency**: Agents depend on ACP server functionality for communication

### Mitigation
- **Efficient ACP Implementation**: Use lightweight HTTP servers for ACP
- **Port Pool Management**: Pre-allocate port pools for efficient allocation
- **Process Pooling**: Reuse configurations for frequently used agent types
- **Platform-specific Resource Management**: Adapt resource limits by OS
- **Coordinated Recovery**: Retry policies with exponential backoff for both CLI and ACP processes
- **Template Caching**: Cache and refine templates over time
- **Custom Template Support**: Allow users to provide custom templates for their roles
- **ACP Health Monitoring**: Robust health checks for ACP server availability

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