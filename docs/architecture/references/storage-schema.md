# MAOS Storage Schema Reference

## Overview

This document consolidates all storage-related schemas and file system structures used by MAOS. It serves as the single source of truth for storage architecture, eliminating duplications across ADRs.

## SQLite Database Schemas

MAOS uses two SQLite databases:

1. **Global Instance Database** (`~/.maos/instances.db`) - Tracks all running MAOS instances
2. **Project Database** (`~/.maos/projects/{workspace-slug}/project.db`) - Project-specific data

### Global Instance Database Schema

```sql
-- Track all running MAOS instances across all projects
CREATE TABLE instances (
    id TEXT PRIMARY KEY,
    pid INTEGER NOT NULL,
    port INTEGER NOT NULL,
    workspace_hash TEXT NOT NULL,
    workspace_path TEXT NOT NULL,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_heartbeat TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'running', -- running, stopped, crashed
    UNIQUE(pid),
    UNIQUE(port)
);

CREATE INDEX idx_workspace_hash ON instances(workspace_hash);
CREATE INDEX idx_port ON instances(port);
CREATE INDEX idx_status ON instances(status);
CREATE INDEX idx_heartbeat ON instances(last_heartbeat);
```

### Project Database Schema

```sql
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
    instance_id TEXT,  -- References instances.id from global DB
    objective TEXT NOT NULL,
    strategy TEXT NOT NULL CHECK(strategy IN ('parallel', 'sequential', 'adaptive', 'pipeline')),
    state TEXT NOT NULL CHECK(state IN ('pending', 'planning', 'executing', 'completed', 'failed', 'cancelled')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    agent_count INTEGER DEFAULT 0,
    completed_agents INTEGER DEFAULT 0,
    failed_agents INTEGER DEFAULT 0,
    metadata JSON,
    error_message TEXT
);

-- Agent assignments
CREATE TABLE session_agents (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    role_instance INTEGER DEFAULT 1,  -- For multiple instances of same role
    task TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('pending', 'running', 'completed', 'failed')),
    dependencies JSON,  -- Array of agent IDs this agent depends on
    pid INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    exit_code INTEGER,
    error_message TEXT,
    config JSON,  -- Agent-specific configuration
    
    FOREIGN KEY (session_id) REFERENCES sessions(id),
    UNIQUE(session_id, role, role_instance)
);

-- Inter-agent dependencies (normalized)
CREATE TABLE agent_dependencies (
    agent_id TEXT REFERENCES session_agents(id),
    depends_on_agent_id TEXT REFERENCES session_agents(id),
    PRIMARY KEY (agent_id, depends_on_agent_id)
);

-- Session events for audit trail
CREATE TABLE session_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    event_type TEXT NOT NULL,
    agent_id TEXT,
    data JSON,
    
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Simple task tracking
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id),
    agent_id TEXT REFERENCES session_agents(id),
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSON
);

-- Indexes for performance
CREATE INDEX idx_sessions_project ON sessions(project_id);
CREATE INDEX idx_sessions_state ON sessions(state);
CREATE INDEX idx_sessions_created ON sessions(created_at);
CREATE INDEX idx_agents_session ON session_agents(session_id);
CREATE INDEX idx_agents_state ON session_agents(state);
CREATE INDEX idx_events_session_time ON session_events(session_id, timestamp);
```

## File System Structure

```
~/.maos/
├── instances.db                 # Global instance tracking database
├── instances/
│   └── {workspace-slug}-{instance-id}.lock  # Unique lock per MCP server instance
├── projects/
│   ├── {workspace-slug}/        # Hash of project path for uniqueness
│   │   ├── project.db          # Project-specific SQLite database
│   │   ├── config.json         # Project-specific settings
│   │   ├── sessions/
│   │   │   ├── {session-id}/
│   │   │   │   ├── metadata.json       # Session configuration
│   │   │   │   ├── agents/
│   │   │   │   │   └── {agent-id}/
│   │   │   │   │       ├── info.json   # Agent metadata
│   │   │   │   │       ├── stdout.log  # Standard output
│   │   │   │   │       ├── stderr.log  # Standard error
│   │   │   │   │       ├── messages.jsonl  # Agent's message history
│   │   │   │   │       └── workspace/  # Agent's isolated workspace
│   │   │   │   ├── shared/
│   │   │   │   │   ├── context/       # Shared specifications, requirements
│   │   │   │   │   │   ├── requirements.md
│   │   │   │   │   │   ├── architecture.md
│   │   │   │   │   │   └── ...
│   │   │   │   │   └── messages/      # Inter-agent communication
│   │   │   │   │       ├── broadcast/  # Broadcast messages
│   │   │   │   │       └── {from-agent}-to-{to-agent}/  # Direct messages
│   │   │   │   │           └── {timestamp}-{msg-id}.json
│   │   │   │   └── results/           # Final outputs
│   │   │   │       └── {artifact-name}
│   │   │   └── active -> {session-id}  # Symlink to current session
│   │   └── logs/
│   │       └── orchestration-{date}.log
│   └── global/                 # Non-project-specific sessions
│       └── sessions/           # Same structure as project sessions
├── agents/
│   ├── templates/              # Agent prompt templates
│   │   ├── architect.txt
│   │   ├── engineer.txt
│   │   └── ...
│   └── registry.json           # Agent role definitions
└── logs/
    └── maos-{date}.log        # Global MAOS logs
```

## Workspace Hash Calculation

The workspace hash is calculated to ensure unique project identification:

```rust
use sha2::{Sha256, Digest};

fn calculate_workspace_hash(workspace_path: &Path) -> String {
    let canonical_path = workspace_path.canonicalize()
        .unwrap_or_else(|_| workspace_path.to_path_buf());
    
    let mut hasher = Sha256::new();
    hasher.update(canonical_path.to_string_lossy().as_bytes());
    format!("{:x}", hasher.finalize())
}
```

## Lock File Format

Lock files contain instance metadata:

```json
{
  "instance_id": "inst_abc123",
  "pid": 12345,
  "port": 3000,
  "workspace_path": "/Users/me/project",
  "started_at": "2024-01-10T10:30:00Z",
  "last_heartbeat": "2024-01-10T10:35:00Z"
}
```

## Message File Format

Inter-agent messages follow this structure:

```json
{
  "id": "msg_xyz789",
  "from": "agent_engineer_1_abc",
  "to": "agent_architect_1_def",
  "session_id": "sess_123",
  "timestamp": "2024-01-10T10:30:00Z",
  "type": "request|response|notification",
  "subject": "API Design Review",
  "body": {
    "content": "Please review the proposed API changes...",
    "attachments": ["shared/context/api-spec.yaml"]
  },
  "metadata": {
    "priority": "normal|high|urgent",
    "requires_response": true,
    "correlation_id": "msg_previous_123"
  }
}
```

## Usage Notes

1. **Database Locations**:
   - Global instance DB: `~/.maos/instances.db`
   - Project DB: `~/.maos/projects/{workspace-slug}/project.db`

2. **Workspace Isolation**:
   - Each agent gets its own workspace directory
   - Agents communicate through the shared message system
   - Direct file access between agents is prohibited

3. **Cleanup Policy**:
   - Lock files are removed when instances shutdown cleanly
   - Session data is retained for audit purposes
   - Old logs are rotated based on age/size

## References

This schema consolidates and supersedes storage definitions from:
- ADR-002: Hybrid Storage Strategy (base architecture)
- ADR-007: Multi-Instance Architecture (instance management)
- ADR-009: Session Management (session/agent tracking)