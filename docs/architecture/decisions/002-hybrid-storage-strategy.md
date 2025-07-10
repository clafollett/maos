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
```
~/.maos/
├── instances/
│   └── {instance-id}.lock       # Unique lock per MCP server instance
├── maos.db                      # SQLite database for metadata
├── projects/
│   ├── {workspace-hash}/        # Hash of project path
│   │   ├── sessions/
│   │   │   ├── {session-id}/
│   │   │   │   ├── metadata.json
│   │   │   │   ├── agents/
│   │   │   │   │   ├── {agent-id}.json
│   │   │   │   │   └── {agent-id}/
│   │   │   │   │       ├── stdout.log
│   │   │   │   │       ├── stderr.log
│   │   │   │   │       ├── messages.jsonl
│   │   │   │   │       └── workspace/
│   │   │   │   ├── shared/
│   │   │   │   │   ├── context/
│   │   │   │   │   └── messages/
│   │   │   │   └── results/
│   │   │   └── active-session    # Symlink to current session
│   │   └── config.json           # Project-specific settings
│   └── global/                   # Non-project sessions
├── agents/
│   ├── templates/
│   └── registry.json
└── logs/
```

### SQLite Schema
```sql
-- Track running MAOS instances
CREATE TABLE instances (
    id TEXT PRIMARY KEY,
    pid INTEGER NOT NULL,
    port INTEGER,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_heartbeat TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(pid)
);

-- Project workspace tracking
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    workspace_hash TEXT UNIQUE NOT NULL,
    workspace_path TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_accessed TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Orchestration sessions
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    project_id INTEGER REFERENCES projects(id),
    instance_id TEXT REFERENCES instances(id),
    status TEXT NOT NULL CHECK(status IN ('active', 'completed', 'failed', 'cancelled')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    metadata JSON
);

-- Agent processes
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id),
    role TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'running', 'completed', 'failed')),
    pid INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    exit_code INTEGER,
    config JSON
);

-- Inter-agent dependencies
CREATE TABLE agent_dependencies (
    agent_id TEXT REFERENCES agents(id),
    depends_on_agent_id TEXT REFERENCES agents(id),
    PRIMARY KEY (agent_id, depends_on_agent_id)
);

-- Simple task tracking
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id),
    agent_id TEXT REFERENCES agents(id),
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSON
);

-- Indexes for performance
CREATE INDEX idx_sessions_project ON sessions(project_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_agents_session ON agents(session_id);
CREATE INDEX idx_agents_status ON agents(status);
CREATE INDEX idx_instances_heartbeat ON instances(last_heartbeat);
```

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
- Claude Code's session storage pattern
- SQLite's excellent concurrent read performance
- Unix philosophy: files as universal interface

---
*Date: 2025-07-09*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollettLaFollett Labs LLC)*