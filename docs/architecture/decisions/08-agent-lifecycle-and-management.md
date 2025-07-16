# ADR-08: Agent Lifecycle and PTY Multiplexer Management

## Status
Accepted

## Context
MAOS needs to manage the complete lifecycle of AI agent processes through the **Orchestrator-as-PTY-Multiplexer** pattern. With our simplified architecture where agents are just Claude CLI processes controlled via PTY, we need:

- Reliable PTY-based process spawning and monitoring
- **Centralized agent management through Orchestrator multiplexer**
- Agent role specialization with both predefined and custom roles
- Support for multiple instances of the same role
- Resource management and limits for Claude CLI processes
- Graceful shutdown and cleanup
- Health monitoring and recovery
- **PTY-based communication integration**
- Output capture and streaming through central multiplexer

Key insights:
- Each agent is a single Claude CLI process (`claude --session-id`, etc.)
- **Orchestrator Agent acts as PTY multiplexer managing all agent processes**
- Agents can have predefined roles (architect, engineer, etc.) or custom roles
- Multiple agents of the same role can work in parallel
- Agents work in isolated workspaces with shared context
- **PTY communication enables hub-and-spoke coordination via Orchestrator**
- Output streams back to the MCP client through Orchestrator only
- **No network dependencies or port management needed**

## Decision
We will implement comprehensive agent lifecycle management with **PTY multiplexer integration**, flexible role-based specialization supporting both predefined and custom roles, multiple instances, and robust process handling. All agents will run as single Claude CLI processes managed by the Orchestrator's PTY multiplexer.

### Architectural Layering

This ADR provides the low-level process management infrastructure that higher-level orchestration builds upon:

- **ADR-08 provides**: PTY-based process spawning, **multiplexer management**, resource management, health monitoring, and lifecycle state management
- **ADR-04 uses**: This infrastructure for **PTY multiplexer-based agent communication**
- **ADR-03 uses**: This infrastructure for session-level orchestration and agent coordination
- **ADR-05 provides**: CLI configurations and integration patterns that this ADR uses for spawning
- **Relationship**: ADR-08 handles the "how" of agent processes + PTY multiplexer, ADR-04 handles agent communication, ADR-03 handles the "when/why" of orchestration, ADR-05 handles the "what CLIs are available"

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

### Orchestrator-as-PTY-Multiplexer Architecture

**Agent Process Model:**
Each agent is now a simple, single Claude CLI process managed by the Orchestrator:
- **Claude CLI Process**: The actual AI agent (`claude --session-id`, etc.)
- **PTY Control**: Orchestrator controls agent via PTY read/write operations
- **Session Persistence**: Claude CLI `--session-id` provides memory continuity
- **Workspace**: Isolated workspace with shared context access
- **No Network Layer**: No embedded servers or network dependencies

**Orchestrator Multiplexer Responsibilities:**
- **Process Management**: Spawn, monitor, and control multiple Claude CLI processes
- **Communication Routing**: Central hub for all inter-agent messages
- **Resource Allocation**: Manage system resources across all agents
- **Health Monitoring**: Monitor all agent processes and recover from failures
- **Session Registry**: Track agent IDs to Claude session ID mappings

**Key Management Areas:**
- **Instance Tracking**: Support multiple instances of same role with unique identifiers
- **Resource Limits**: CPU and memory allocation per Claude CLI process
- **Health Monitoring**: Monitor Claude CLI process responsiveness and health
- **Template Generation**: Dynamic role template generation for specialized agents
- **Graceful Shutdown**: Coordinated shutdown of all managed processes

### PTY Multiplexer Process Management

The Orchestrator Agent coordinates all agent lifecycle through these key responsibilities:

1. **Template Resolution**: Determine agent configuration from predefined roles or generate custom templates
2. **Resource Validation**: Check system capacity before spawning (memory, instance limits)
3. **Instance Management**: Track multiple instances of the same role with unique identifiers
4. **Environment Setup**: Configure agent workspace and environment variables
5. **PTY Process Spawning**: Create PTY pair and launch Claude CLI process
6. **Session Registration**: Record agent ID → Claude session ID mapping
7. **Communication Setup**: Establish PTY read/write for message routing
8. **Monitoring Integration**: Set up health checks, output streaming, and logging

**Agent ID Structure**: `agent_{role}_{instance}_{unique_id}` (e.g., `agent_backend_engineer_1_abc123`)

**PTY Integration Points:**
- **PTY Pair Creation**: Each agent gets dedicated PTY master/slave pair
- **Process Control**: Orchestrator controls master side for read/write operations
- **Session Tracking**: Map agent IDs to Claude session IDs for persistence
- **Health Monitoring**: Monitor PTY process health and responsiveness
- **Message Routing**: Route all inter-agent communication through PTY

### Agent Lifecycle Patterns

**Micro-Task Lifecycle** (Maximum Efficiency):
1. **Spawn**: Create agent for specific, atomic task
2. **Isolation**: Agent works without interruptions or communication
3. **Complete**: Report task completion via PTY message to Orchestrator
4. **Terminate**: Immediate shutdown and resource cleanup
- **Benefits**: Maximum focus, no interruptions, immediate resource reclamation
- **Use Cases**: Generate test, implement function, review code, create documentation

**Phase-Based Lifecycle** (Team Coordination):
1. **Spawn**: Create agent for orchestration phase
2. **Coordinate**: Minimal PTY communication for essential coordination only
3. **Progress**: Work with periodic essential status updates via Orchestrator
4. **Phase Complete**: Report phase completion and deliverables
5. **Continue/Terminate**: Ready for next phase or graceful shutdown
- **Benefits**: Context retention, adaptive planning, coordinated teamwork
- **Use Cases**: Research phases, architecture design, complex implementations

**Lifecycle Selection Criteria:**
- **Micro-Task**: When work is independent, atomic, and requires maximum focus
- **Phase-Based**: When work requires coordination, context, or multi-step execution
- **Resource Considerations**: Micro-tasks conserve resources; phase-based retains context

### Agent Pool Management with Session Binding

**Context Persistence Pattern:**

The Orchestrator maintains an **Agent Resources Registry** that binds agents to Claude Code session IDs for persistent context across activations:

```
Agent Resources Registry (PTY Multiplexer):
┌─────────────────────────────────────────────────────────────┐
│ agent_id           → claude_session_id → status → pty_info │
├─────────────────────────────────────────────────────────────┤
│ researcher_1       → session_abc123    → [SLEEP] → pty_001 │
│ app_architect_1    → session_def456    → [ACTIVE]→ pty_002 │
│ frontend_eng_1     → session_ghi789    → [SLEEP] → pty_003 │
│ frontend_eng_2     → session_jkl012    → [ACTIVE]→ pty_004 │
│ backend_eng_1      → session_mno345    → [SLEEP] → pty_005 │
│ qa_agent_1         → session_pqr678    → [SLEEP] → pty_006 │
└─────────────────────────────────────────────────────────────┘
```

**Agent Registration Flow:**
1. **Initial Spawn**: Orchestrator spawns agent with specific role via PTY
2. **Session Creation**: Agent makes first `claude` CLI call → receives unique session_id
3. **Registration**: Orchestrator captures session_id from agent output
4. **Registry Update**: Orchestrator records agent_id → session_id mapping

**Context-Aware Reactivation:**
1. **Selective Activation**: Orchestrator needs specific agent for new phase
2. **PTY Recreation**: Create new PTY pair for existing session
3. **Session Reuse**: Spawn with existing session_id: `claude --session-id session_abc123`
4. **Context Restoration**: Agent resumes with FULL memory of previous work
5. **Continued Work**: Agent picks up exactly where it left off

**Agent Status Lifecycle:**
- **[SPAWNING]**: Agent PTY being created and session registering
- **[ACTIVE]**: Agent PTY process running and working
- **[SLEEPING]**: Agent PTY process terminated but session preserved
- **[TERMINATED]**: Agent and session permanently closed

**Resource Benefits:**
- **Memory Persistence**: Agents never lose context between activations
- **Resource Efficiency**: PTY processes sleep when not needed, conserving system resources
- **Selective Reactivation**: Reactivate specific agents (e.g., frontend_engineer_1) with full context
- **Scalable Pool Management**: Handle complex orchestrations with many specialist agents

### PTY-Based Health Monitoring

Health monitoring tracks Claude CLI process status through PTY management:
- **PTY Process Health**: Regular verification that Claude CLI processes are responsive
- **Process Death Detection**: Automatic cleanup when agents terminate unexpectedly
- **Session Persistence Check**: Verify Claude sessions remain valid across sleep/wake
- **Resource Monitoring**: Track memory and CPU usage per Claude CLI process
- **Communication Health**: Ensure PTY read/write operations are functioning
- **Lifecycle Events**: Log significant agent state changes for debugging and audit
- **Registry Consistency**: Verify agent registry mapping accuracy

### Template Generation for Custom Roles

Template generation for custom roles is handled by the TemplateGenerator, which automatically:
- Infers capabilities from the role description
- Determines required tools based on capabilities
- Generates appropriate prompt templates for PTY-controlled agents

The complete template generation logic and prompt format are documented in the [Agent Roles Reference](../references/agent-roles.md#custom-role-support).

### Resource Management for PTY Multiplexer

Resource management enforces system limits through configurable constraints:

**Resource Types**:
- **Total Agent Limit**: Maximum concurrent agents across all roles
- **Per-Role Limits**: Maximum agents per role type (with defaults for custom roles)
- **Memory Limits**: Total memory allocation across all Claude CLI processes
- **Process Limits**: OS-level constraints on agent processes
- **PTY Pair Limits**: Maximum number of concurrent PTY pairs
- **File Descriptor Limits**: OS limits on open file descriptors

**Resource Validation**: Before spawning agents, validate available capacity including:
- Available memory for Claude CLI process
- Available PTY pairs for new agent
- File descriptor availability
- Reject requests that would exceed any resource limits

### Graceful Shutdown with PTY Management

Agent shutdown follows a coordinated approach through PTY control:

1. **Communication Shutdown**: Stop routing messages to/from agent
2. **Session Preservation**: Ensure Claude session state is saved
3. **PTY Process Termination**: Send shutdown signal to Claude CLI process
4. **Timeout Waiting**: Allow reasonable time for agent to finish current work
5. **Force Termination**: Kill unresponsive processes after timeout
6. **Resource Cleanup**: Clean up PTY pairs, file descriptors, and workspace
7. **Registry Update**: Update agent status to [SLEEPING] or [TERMINATED]

**Shutdown Modes**:
- **Individual Agent**: Shut down specific agent by ID
- **Role-based**: Shut down all agents of a specific role
- **Session-based**: Shut down all agents in a session  
- **System-wide**: Emergency shutdown of all agents and PTY cleanup

**PTY Coordination**:
- **Process Cleanup**: Ensure PTY processes are properly terminated
- **Resource Release**: Return PTY pairs and file descriptors to available pool
- **Session Preservation**: Maintain Claude session mappings for future reactivation

### Process Management Overview (Orchestrator & Agents)

**Orchestrator Agent Process:**
- **Role**: Acts as PTY multiplexer managing all other agents
- **Lifecycle**: Spawned by MCP server and persists for entire orchestration
- **Responsibilities**: Process management, communication routing, resource allocation
- **Communication**: Receives commands from MCP server, sends status updates back

**Managed Agent Processes:**
- **Role**: Specialized Claude CLI processes for specific tasks
- **Lifecycle**: Spawned/managed by Orchestrator, can sleep/wake as needed
- **Responsibilities**: Execute assigned tasks in their domain expertise
- **Communication**: All communication routed through Orchestrator via PTY

**Process Isolation:**
- Each Claude CLI process runs in isolated environment
- Orchestrator mediates all inter-agent communication
- Shared file system for artifacts, PTY for messages
- No direct process-to-process communication

## Consequences

### Positive
- **Simplified Process Model**: Single Claude CLI process per agent (no embedded servers)
- **Cross-Platform Compatibility**: PTY works on Windows, macOS, Linux
- **Flexible Roles**: Support for both predefined and custom agent roles
- **Multiple Instances**: Can run multiple agents of the same role concurrently
- **Role Specialization**: Clear agent responsibilities with optimized prompts
- **Dynamic Templates**: Custom roles get automatically generated templates
- **Resource Control**: Per-role limits with defaults for custom roles
- **PTY Health Monitoring**: Automatic detection of crashed Claude CLI processes
- **Centralized Management**: All agent lifecycle managed through Orchestrator
- **Graceful Shutdown**: Clean termination of PTY processes
- **Isolated Workspaces**: Agents can't interfere with each other
- **Clear Identification**: Agent IDs include role and instance information
- **Session Persistence**: Perfect memory continuity via Claude CLI sessions
- **No Network Dependencies**: No ports, discovery, or network configuration

### Negative
- **PTY Dependency**: Requires portable-pty for cross-platform compatibility
- **Central Coordination**: Orchestrator manages all agent communication
- **Process Management Complexity**: Managing multiple Claude CLI processes via PTY
- **Platform Differences**: Resource limits vary by OS
- **Recovery Complexity**: Restarting failed agents needs careful state management
- **Template Quality**: Custom role templates may be less optimized than predefined ones

### Mitigation
- **PTY Abstraction**: portable-pty handles cross-platform PTY differences
- **Orchestrator Resilience**: Robust error handling and recovery in Orchestrator
- **Process Monitoring**: Health checks and automatic restart capabilities
- **Platform-specific Resource Management**: Adapt resource limits by OS
- **Coordinated Recovery**: Retry policies with exponential backoff for Claude CLI processes
- **Template Caching**: Cache and refine templates over time
- **Custom Template Support**: Allow users to provide custom templates for their roles

## References
- **ADR-04: Orchestrator-as-PTY-Multiplexer Communication** - Defines the PTY communication that this ADR implements
- **ADR-03: Session Orchestration and State Management** - Uses this process management infrastructure for session-level coordination
- **ADR-05: CLI Integration and Process Spawning** - CLI-specific spawning patterns
- [portable-pty](https://docs.rs/portable-pty/) - Cross-platform PTY implementation
- [Tmux-Orchestrator](https://github.com/Jedward23/Tmux-Orchestrator) - Inspiration for multiplexer patterns
- Process supervision patterns and best practices
- Tokio process management documentation
- Linux resource limits (setrlimit)
- PTY and terminal multiplexer design principles

---
*Date: 2025-07-16*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*