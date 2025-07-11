# ADR-002: Hybrid Storage Strategy (SQLite + File System)

## Status
Accepted

## Context
MAOS needs a storage solution that balances simplicity, reliability, and performance for multi-agent orchestration. After analyzing our requirements and the realization that MAOS will operate as an MCP server spawning CLI processes, we need:

- **Metadata Storage**: Session IDs, agent registry, task tracking, file paths
- **Agent Communication**: Inter-agent messages, shared context, outputs
- **Session Persistence**: Survive restarts, project-based isolation
- **Simple Deployment**: No Docker, no external databases
- **Multi-Instance Support**: Multiple MAOS servers can run simultaneously

Key realizations:
- Heavy persistence (PostgreSQL) is overkill for orchestration metadata
- File-based communication is transparent and debuggable
- SQLite provides sufficient ACID guarantees for metadata
- Project isolation mirrors Claude Code's approach

## Decision
We will use a hybrid storage approach:
1. **SQLite** for metadata and coordination
2. **File System** for agent communication and outputs
3. **Project-based isolation** in `~/.maos/` directory

### Storage Architecture

MAOS uses a hybrid approach with SQLite for metadata and the file system for agent communication. The complete storage schema and file system structure are documented in the [Storage Schema Reference](../references/storage-schema.md).

Key components:
- **SQLite Databases**: Global instance tracking and project-specific metadata
- **File System**: Agent outputs, inter-agent messages, and shared context
- **Project Isolation**: Each project has its own storage area under `~/.maos/projects/{workspace-hash}/`

### File System Patterns

#### Agent Output Streaming
```rust
// Each agent writes to its own log files
let stdout_path = format!("{}/agents/{}/stdout.log", session_dir, agent_id);
let stderr_path = format!("{}/agents/{}/stderr.log", session_dir, agent_id);

// Stream output back to MCP client
tail_file(stdout_path, |line| {
    mcp_server.send_resource_update(
        format!("maos://sessions/{}/agents/{}/output", session_id, agent_id),
        json!({ "line": line, "stream": "stdout" })
    );
});
```

#### Inter-Agent Messaging
```rust
// Message queue directory per agent pair
let message_dir = format!("{}/shared/messages/{}-to-{}", 
    session_dir, from_agent, to_agent);

// Append-only message files
let message_file = format!("{}/{}.jsonl", message_dir, timestamp);
```

#### Shared Context
```rust
// Agents can read/write shared specifications
let context_dir = format!("{}/shared/context", session_dir);

// Example: Architect writes, Engineer reads
let spec_file = format!("{}/system-design.md", context_dir);
```

## Consequences

### Positive
- **Simple Deployment**: No Docker or external databases required
- **Transparent**: All communication visible in file system
- **Debuggable**: Can inspect agent interactions directly
- **Reliable**: SQLite for ACID metadata, files for data
- **Scalable**: Each session isolated, no contention
- **Familiar**: Follows Claude Code's project isolation pattern

### Negative
- **Not Distributed**: Single-machine only (acceptable for v1)
- **Manual Cleanup**: Need periodic cleanup of old sessions
- **File System Limits**: Very large outputs need management

### Mitigation
- Implement configurable retention policies
- Add file size limits and rotation
- Future: Could add distributed storage adapter

## References
- [Storage Schema Reference](../references/storage-schema.md) - Complete storage schemas and structure
- Claude Code's session storage pattern
- SQLite's excellent concurrent read performance
- Unix philosophy: files as universal interface

---
*Date: 2025-07-09*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*