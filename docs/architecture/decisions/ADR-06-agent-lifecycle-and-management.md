# ADR-06: Agent Lifecycle and Management

## Status
Accepted (Updated for PTY-based architecture)

## Context
MAOS needs to manage the complete lifecycle of AI agent processes, from spawning to termination. With our PTY multiplexer architecture, we need:

- Reliable Claude CLI process spawning via PTY
- Agent role specialization through initial briefing
- Session continuity via `--session-id`
- Multiple concurrent agent management
- Resource limits and process monitoring
- Graceful shutdown and cleanup
- Health monitoring and crash recovery
- Real-time output capture and streaming

Key insights from PTY approach:
- Each agent is a Claude CLI process in its own PTY
- PTY multiplexer manages all agent processes directly
- Session continuity preserved via `--session-id` flag
- Direct I/O control enables immediate response
- No network timeouts or port management
- Process crashes detected immediately
- Output captured in real-time for routing

## Decision
We will implement PTY-based agent lifecycle management where the MAOS CLI (PTY multiplexer) directly manages multiple Claude CLI processes through pseudo-terminal interfaces.

### Architectural Layering

This ADR provides the process management infrastructure for the PTY multiplexer:

- **ADR-02 provides**: PTY multiplexer architecture this builds upon
- **ADR-03 ensures**: Works in any terminal environment
- **ADR-04 uses**: This infrastructure for session management
- **ADR-05 uses**: This infrastructure for message routing between agents
- **Relationship**: ADR-06 handles PTY process lifecycle, ADR-02 provides PTY abstraction

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

**PTY Multiplexer Components:**
- **PTY Manager**: Creates and manages pseudo-terminals
- **Process Registry**: Tracks all active Claude CLI processes
- **Session Manager**: Maps sessions to PTY handles
- **Health Monitor**: Detects process crashes/hangs
- **Resource Manager**: Controls concurrent agent limits

**Key Management Areas:**
- **PTY Creation**: Allocate pseudo-terminal for each agent
- **Process Spawning**: Launch Claude CLI in PTY
- **Session Binding**: Use `--session-id` for context
- **I/O Control**: Direct read/write to agent PTY
- **Output Capture**: Real-time streaming and logging
- **Graceful Shutdown**: Clean PTY and process termination

**Agent Process Lifecycle:**
1. **Spawn Request**: User requests agent with role and task
2. **PTY Allocation**: Create new pseudo-terminal
3. **Process Launch**: Start Claude CLI with session ID
4. **Role Briefing**: Send initial role template and task
5. **Execution**: Agent works independently
6. **Output Monitoring**: Capture and route responses
7. **Termination**: Clean shutdown or timeout handling

### Process Spawning Architecture

The PTY multiplexer manages agent spawning through:

1. **Role Selection**: User specifies agent role
2. **PTY Creation**: Allocate new pseudo-terminal
3. **Process Launch**: Start Claude CLI in PTY
4. **Session Setup**: Pass `--session-id` for persistence
5. **Role Briefing**: Send role template via PTY
6. **Task Assignment**: Send specific task to agent
7. **Monitoring Start**: Begin output capture

**Process Identification**:
- PTY handle as technical identifier
- Session ID for Claude context continuity
- Agent role for user reference
- Simple naming like "backend-7f3a"

**Key Implementation:**
```rust
pub async fn spawn_agent(role: &str, task: &str) -> Result<AgentHandle> {
    // Create PTY
    let pty = self.pty_backend.create_pty()?;
    
    // Launch Claude CLI
    let cmd = ["claude", "--session-id", &session_id];
    pty.spawn_process(&cmd)?;
    
    // Wait for Claude to start
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Send role briefing
    let briefing = load_role_template(role, task)?;
    send_message(&mut pty, &briefing).await?;
    
    Ok(AgentHandle { pty, role, session_id })
}
```

### Process Lifecycle Patterns

**Lifecycle Patterns**:
1. **User Request**: `maos spawn backend "implement auth"`
2. **PTY Creation**: Allocate new pseudo-terminal
3. **Process Start**: Launch Claude CLI with session ID
4. **Active Work**: Agent processes task independently
5. **Output Streaming**: Real-time capture to user/logs
6. **Message Routing**: Inter-agent messages via orchestrator
7. **Task Completion**: Agent signals done or timeout
8. **Session Persist**: Keep session ID for future use

**Session Continuity Benefits**:
- **Perfect Memory**: Claude CLI retains full context
- **Fast Resume**: `maos resume <session-id>`
- **Cross-Phase Work**: Same agent across project phases
- **Crash Recovery**: Restart with same session ID

**Resource Management:**
- **PTY Pool**: Limit concurrent terminals
- **Process Limits**: Max agents per system
- **Memory Tracking**: Monitor total usage
- **CPU Throttling**: Prevent system overload

### Session Management

**Session Registry Pattern:**

The PTY multiplexer maintains a registry of all active sessions:

**Session Registry:**

| session_id     | role       | status   | phase   |
|----------------|------------|----------|---------|
| session_abc123 | researcher | [IDLE]   | Phase 1 |
| session_def456 | architect  | [ACTIVE] | Phase 2 |
| session_ghi789 | engineer   | [IDLE]   | Phase 2 |
| session_jkl012 | engineer   | [ACTIVE] | Phase 3 |
| session_mno345 | engineer   | [IDLE]   | Phase 2 |
| session_pqr678 | qa         | [IDLE]   | Phase 3 |

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

The PTY multiplexer monitors agent health through:
- **Process Status**: Direct detection of process exit
- **PTY I/O**: Monitor for output/input activity
- **Heartbeat**: Periodic status checks via PTY
- **Resource Usage**: Track memory and CPU usage
- **Timeout Detection**: Configurable inactivity timeouts
- **Crash Detection**: Immediate notification on process death
- **Output Analysis**: Parse for error patterns

### Template Generation for Custom Roles

Template generation for custom roles is handled by the TemplateGenerator, which automatically:
- Infers capabilities from the role description
- Determines required tools based on capabilities
- Generates appropriate prompt templates

The complete template generation logic and prompt format are documented in the [Agent Roles Reference](../references/agent-roles.md#custom-role-support).

### Resource Management

The PTY multiplexer enforces resource limits:

**Resource Types**:
- **PTY Limit**: Maximum concurrent pseudo-terminals
- **Process Limit**: Maximum agent processes
- **Memory Budget**: Total memory across all agents
- **Output Buffers**: Scrollback buffer limits

**Resource Strategies**:
```rust
pub struct ResourceLimits {
    max_agents: usize,        // e.g., 10
    max_memory_mb: usize,     // e.g., 4096
    max_output_kb: usize,     // e.g., 1024 per agent
    idle_timeout: Duration,   // e.g., 30 minutes
}
```

### Graceful Shutdown

The PTY multiplexer handles shutdown cleanly:

1. **Notification**: Send shutdown message to agents
2. **Grace Period**: Allow 30s to finish work
3. **PTY Close**: Close pseudo-terminal handles
4. **Process Termination**: SIGTERM then SIGKILL
5. **Session Save**: Preserve session IDs for resume
6. **Cleanup**: Remove temporary files and buffers

**Shutdown Modes**:
- **Agent Shutdown**: Terminate specific agent
- **Session Shutdown**: Close session but preserve ID
- **System Shutdown**: Stop all agents and multiplexer

## Consequences

### Positive
- **Direct Control**: Immediate process management via PTY
- **No Timeouts**: No network protocol limitations
- **Perfect Context**: Session IDs preserve complete context
- **Real-time I/O**: Direct access to agent input/output
- **Cross-Platform**: Works with portable-pty everywhere
- **Crash Detection**: Immediate notification of failures
- **Simple Architecture**: No network complexity
- **Better Debugging**: Can inspect PTY output directly

### Negative
- **PTY Complexity**: Pseudo-terminal handling is non-trivial
- **Platform Differences**: PTY behavior varies by OS
- **Resource Limits**: System PTY limits may apply
- **No Remote Agents**: All agents must be local

### Mitigation
- **PTY Abstraction**: Use portable-pty for cross-platform support
- **Robust Error Handling**: Handle PTY edge cases gracefully
- **Resource Monitoring**: Track and respect system limits
- **Future Enhancement**: Could add remote agent support later

## References
- **ADR-02: PTY Multiplexer Architecture** - Core PTY design this implements
- **ADR-03: Terminal-Agnostic Design** - Ensures works in any terminal
- **ADR-04: Session Management** - Uses this for session persistence
- **ADR-05: PTY-Based Communication** - Message routing patterns
- [portable-pty](https://docs.rs/portable-pty/) - Cross-platform PTY library
- [tmux-orchestrator](https://github.com/Jedward23/Tmux-Orchestrator) - Inspiration for process management
- PTY programming best practices
- Process supervision patterns

---
*Date: 2024-01-15*  
*Updated: 2024-01-25* - Pivoted from ACP to PTY architecture  
*Author: MAOS Team*  
*Reviewers: @clafollett*