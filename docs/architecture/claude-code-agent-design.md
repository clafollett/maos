# Claude Code Agent Design

## Overview

The Claude Code Agent is a single ACP server that manages multiple Claude CLI processes with different roles and sessions. It serves as the bridge between the Orchestrator and the actual Claude CLI processes, providing a clean abstraction for process management.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│               Claude Code Agent (ACP Server)                   │
│                                                                 │
│  Session Registry:                                              │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ agent_id      │ session_id      │ role        │ context     │ │
│  │ architect_1   │ session_abc123  │ architect   │ api_design  │ │
│  │ backend_eng_1 │ session_def456  │ backend_eng │ auth_svc    │ │
│  │ backend_eng_2 │ session_ghi789  │ backend_eng │ user_svc    │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  Active Processes:                                              │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     │
│  │ Claude CLI  │     │ Claude CLI  │     │ Claude CLI  │     │
│  │ -p architect│     │ -p backend  │     │ -p frontend │     │
│  │ Process     │     │ Process     │     │ Process     │     │
│  └─────────────┘     └─────────────┘     └─────────────┘     │
│                                                               │
└─────────────────────────────────────────────────────────────────┘
```

## Core Responsibilities

### 1. Process Management
- **Spawn CLI Processes**: Create new `claude -p <role>` processes on demand
- **Session Assignment**: Assign new or existing session IDs via `--session-id`
- **Process Monitoring**: Track health and status of all CLI processes
- **Resource Control**: Enforce limits on concurrent processes
- **Cleanup**: Gracefully terminate processes when work is complete

### 2. Session Registry
- **Track Sessions**: Maintain mapping of agent_id → session_id → role → context
- **Context Preservation**: Ensure agents can resume work with full context
- **Ordinal Assignment**: Assign incremental IDs (backend_eng_1, backend_eng_2, etc.)
- **Work History**: Track what each agent has worked on for intelligent reuse

### 3. ACP Protocol Implementation
- **Request Handling**: Accept work requests from Orchestrator
- **Response Management**: Return results and status updates
- **Error Handling**: Graceful handling of process failures
- **Health Monitoring**: Provide status information about managed processes

## Key Design Patterns

### Single Server Pattern
- **One ACP Server**: Manages all CLI processes from a single endpoint
- **Efficient Resource Use**: Shared infrastructure for all agents
- **Centralized Control**: Unified process management and monitoring
- **Simplified Architecture**: No complex peer-to-peer networking

### Session Continuity
- **Automatic Session Management**: Handles session ID assignment transparently
- **Context Preservation**: Leverages Claude's `--session-id` for memory
- **Intelligent Reuse**: Connects new work to agents with relevant context
- **Flexible Creation**: Creates new sessions when needed for different components

### Process Lifecycle
```
1. Work Request → 2. Session Lookup → 3. Process Spawn → 4. Work Execution → 5. Result Return
                                    ↓
                                6. Process Cleanup (optional)
```

## Implementation Details

### ACP Server Interface
The Claude Code Agent exposes these ACP endpoints:

- **POST /agents/run**: Execute work with a specific role and optional session
- **GET /agents/status**: Get status of all managed processes
- **GET /agents/sessions**: List current session registry
- **DELETE /agents/sessions/{id}**: Terminate a specific session

### Work Request Format
```json
{
  "agent_id": "backend_eng_1",
  "role": "backend_engineer", 
  "session_id": "session_def456",  // Optional - reuse existing
  "task": "Add two-factor authentication to auth service",
  "context": {
    "component": "auth_service",
    "priority": "high"
  }
}
```

### Session Registry Structure
```rust
struct SessionEntry {
    agent_id: String,        // e.g., "backend_eng_1"
    session_id: String,      // e.g., "session_def456"
    role: String,            // e.g., "backend_engineer"
    work_context: Vec<String>, // e.g., ["auth_service", "oauth_impl"]
    created_at: DateTime<Utc>,
    last_used: DateTime<Utc>,
    process_pid: Option<u32>,
}
```

## Resource Management

### Process Limits
- **Concurrent Processes**: Configurable limit on active CLI processes
- **Memory Management**: Monitor and limit per-process memory usage
- **Timeout Handling**: Automatic cleanup of long-running processes
- **Resource Cleanup**: Proper cleanup of terminated processes

### Session Management
- **Session Reuse**: Intelligent reuse of existing sessions when beneficial
- **Session Cleanup**: Automatic cleanup of unused sessions
- **Context Tracking**: Maintain work history for intelligent agent selection
- **Load Balancing**: Distribute work across multiple agents of same role

## Error Handling

### Process Failures
- **Health Monitoring**: Regular health checks of CLI processes
- **Automatic Restart**: Restart failed processes when possible
- **Graceful Degradation**: Continue with reduced capacity during failures
- **Error Reporting**: Clear error messages to Orchestrator

### Session Issues
- **Session Recovery**: Attempt to recover corrupted sessions
- **Fallback Creation**: Create new sessions when recovery fails
- **Context Preservation**: Maintain work history even during failures
- **Cleanup**: Remove invalid sessions from registry

## Security Considerations

### Process Sandboxing
- **Isolated Workspaces**: Each process runs in isolated workspace
- **Resource Limits**: Enforce CPU and memory limits per process
- **Network Restrictions**: Limit network access for CLI processes
- **File System Access**: Restrict file system access to workspace

### Session Security
- **Session Isolation**: Sessions cannot access each other's data
- **Access Control**: Validate requests before spawning processes
- **Data Protection**: Secure storage of session metadata
- **Audit Logging**: Log all process creation and termination

## Performance Considerations

### Optimization Strategies
- **Process Pooling**: Reuse processes for similar work when possible
- **Session Caching**: Cache session metadata for fast lookups
- **Lazy Loading**: Only load session data when needed
- **Resource Monitoring**: Track and optimize resource usage

### Scalability
- **Horizontal Scaling**: Design allows for multiple Claude Code Agents
- **Load Distribution**: Balance work across available processes
- **Resource Scaling**: Adjust limits based on system capacity
- **Monitoring**: Track performance metrics for optimization

## Integration Points

### With Orchestrator
- **ACP Communication**: Clean request/response pattern
- **Status Updates**: Regular health and progress reporting
- **Work Distribution**: Receive and distribute work requests
- **Result Aggregation**: Collect and return work results

### With Claude CLI
- **Process Management**: Spawn and manage CLI processes
- **Session Integration**: Leverage `--session-id` for context
- **Output Capture**: Capture and stream process output
- **Error Handling**: Handle CLI process failures gracefully

## References

- **ADR-04**: Agent Communication Patterns - ACP protocol details
- **ADR-08**: Agent Lifecycle Management - Process management requirements
- **ADR-11**: Adaptive Phase-Based Orchestration - Integration with Orchestrator
- [claude_code_agent.rs](./references/examples/claude_code_agent.rs) - Implementation example
- [session_registry_example.rs](./references/examples/session_registry_example.rs) - Session management example

---

*Last Updated: 2025-07-16*
*Author: Development Team*