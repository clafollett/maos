# ADR-08: Agent Lifecycle and Management

## Status
Accepted

## Context
MAOS needs to manage the complete lifecycle of AI agent processes, from spawning to termination. With our architecture of MAOS as an MCP server spawning CLI processes, we need:

- Reliable process spawning and monitoring
- Agent role specialization with both predefined and custom roles
- Support for multiple instances of the same role
- Resource management and limits
- Graceful shutdown and cleanup
- Health monitoring and recovery
- Output capture and streaming

Key insights:
- Each agent is a separate CLI process (`claude -p`, etc.)
- Agents can have predefined roles (architect, engineer, etc.) or custom roles
- Multiple agents of the same role can work in parallel
- Agents work in isolated workspaces with shared context
- Output streams back to the MCP client in real-time

## Decision
We will implement comprehensive agent lifecycle management with flexible role-based specialization supporting both predefined and custom roles, multiple instances, and robust process handling.

### Architectural Layering

This ADR provides the low-level process management infrastructure that higher-level orchestration builds upon:

- **ADR-08 provides**: Process spawning, resource management, health monitoring, and lifecycle state management
- **ADR-03 uses**: This infrastructure for session-level orchestration and agent coordination
- **ADR-05 provides**: CLI configurations and integration patterns that this ADR uses for spawning
- **Relationship**: ADR-08 handles the "how" of agent processes, ADR-03 handles the "when/why" of orchestration, ADR-05 handles the "what CLIs are available"

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

### Process Management

```rust
pub struct AgentProcess {
    pub agent_id: String,
    pub role: AgentRole,
    pub instance_number: usize,  // 1, 2, 3, etc. for multiple instances
    pub child: Child,
    pub spawned_at: Instant,
    pub workspace: PathBuf,
    pub stdout_handle: JoinHandle<()>,
    pub stderr_handle: JoinHandle<()>,
    pub health_check_handle: JoinHandle<()>,
}

// Track instance counts per role
pub struct InstanceTracker {
    role_instances: Arc<RwLock<HashMap<String, usize>>>,
}

impl InstanceTracker {
    pub async fn get_next_instance_number(&self, role_name: &str) -> usize {
        let mut instances = self.role_instances.write().await;
        let count = instances.entry(role_name.to_string()).or_insert(0);
        *count += 1;
        *count
    }
    
    pub async fn release_instance_number(&self, role_name: &str) {
        let mut instances = self.role_instances.write().await;
        if let Some(count) = instances.get_mut(role_name) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
}

pub struct ProcessManager {
    agents: Arc<RwLock<HashMap<String, AgentProcess>>>,
    resource_limiter: ResourceLimiter,
    health_monitor: HealthMonitor,
    instance_tracker: InstanceTracker,
    template_generator: TemplateGenerator,
}

### Process Spawning Architecture

The ProcessManager coordinates agent spawning through these key responsibilities:

1. **Template Resolution**: Determine agent configuration from predefined roles or generate custom templates
2. **Resource Validation**: Check system capacity before spawning (memory, instance limits)
3. **Instance Management**: Track multiple instances of the same role with unique identifiers
4. **Environment Setup**: Configure agent workspace and environment variables
5. **Process Lifecycle**: Spawn CLI process with appropriate arguments and resource limits
6. **Monitoring Integration**: Set up health checks, output streaming, and logging

**Agent ID Structure**: `agent_{role}_{instance}_{unique_id}` (e.g., `agent_engineer_1_abc123`)

```

### Health Monitoring

Health monitoring tracks agent process status through:
- **Periodic Health Checks**: Regular verification that agent processes are responsive
- **Process Death Detection**: Automatic cleanup when agents terminate unexpectedly  
- **Resource Monitoring**: Track memory and CPU usage per agent
- **Lifecycle Events**: Log significant agent state changes for debugging and audit

### Template Generation for Custom Roles

Template generation for custom roles is handled by the TemplateGenerator, which automatically:
- Infers capabilities from the role description
- Determines required tools based on capabilities
- Generates appropriate prompt templates

The complete template generation logic and prompt format are documented in the [Agent Roles Reference](../references/agent-roles.md#custom-role-support).

### Resource Management

Resource management enforces system limits through configurable constraints:

**Resource Types**:
- **Total Agent Limit**: Maximum concurrent agents across all roles
- **Per-Role Limits**: Maximum agents per role type (with defaults for custom roles)
- **Memory Limits**: Total memory allocation across all agent processes
- **Process Limits**: OS-level constraints on agent processes

**Resource Validation**: Before spawning agents, validate available capacity and reject requests that would exceed limits.

### Graceful Shutdown

Agent shutdown follows a tiered approach:

1. **Graceful Termination**: Send shutdown signal to agent process
2. **Timeout Waiting**: Allow reasonable time for agent to finish current work
3. **Force Termination**: Kill unresponsive processes after timeout
4. **Resource Cleanup**: Clean up monitoring handles, file descriptors, and workspace

**Shutdown Modes**:
- **Individual Agent**: Shut down specific agent by ID
- **Role-based**: Shut down all agents of a specific role
- **Session-based**: Shut down all agents in a session  
- **System-wide**: Emergency shutdown of all agents

## Consequences

### Positive
- **Flexible Roles**: Support for both predefined and custom agent roles
- **Multiple Instances**: Can run multiple agents of the same role concurrently
- **Role Specialization**: Clear agent responsibilities with optimized prompts
- **Dynamic Templates**: Custom roles get automatically generated templates
- **Resource Control**: Per-role limits with defaults for custom roles
- **Health Monitoring**: Automatic detection of crashed agents
- **Graceful Shutdown**: Clean termination with timeout fallback
- **Isolated Workspaces**: Agents can't interfere with each other
- **Clear Identification**: Agent IDs include role and instance information

### Negative
- **Process Overhead**: Each agent is a full process
- **Platform Differences**: Resource limits vary by OS
- **Recovery Complexity**: Restarting failed agents needs careful state management
- **Template Quality**: Custom role templates may be less optimized than predefined ones

### Mitigation
- Process pooling for frequently used configurations
- Platform-specific resource management code
- Simple retry policies with exponential backoff
- Template caching and refinement over time
- Allow users to provide custom templates for their roles

## References
- **ADR-03: Session Orchestration and State Management** - Uses this process management infrastructure for session-level coordination
- ADR-05: CLI Integration and Process Spawning - CLI-specific spawning patterns
- Process supervision patterns and best practices
- Tokio process management documentation
- Linux resource limits (setrlimit)
- Container orchestration patterns

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*