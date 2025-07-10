# ADR-005: Simple Logging and Audit Trail

## Status
Accepted

## Context
MAOS needs to maintain visibility into orchestration activities for debugging, monitoring, and audit purposes. Originally, we considered a full event sourcing approach with PostgreSQL, but after analyzing our actual needs:

- We don't need event replay for state reconstruction (SQLite tracks current state)
- We don't need complex projections or CQRS patterns
- We do need to debug what happened during orchestration
- We do need basic audit trails for compliance
- We want to keep deployment simple (no PostgreSQL)

## Decision
We will implement simple, file-based logging instead of complex event sourcing.

### Logging Architecture

```
~/.maos/
├── projects/
│   └── {workspace-hash}/
│       └── sessions/
│           └── {session-id}/
│               ├── session.log         # Main session event log
│               └── agents/
│                   └── {agent-id}/
│                       ├── stdout.log   # Agent output
│                       ├── stderr.log   # Agent errors
│                       └── events.log   # Agent lifecycle events
└── logs/
    └── {instance-id}.log              # MAOS server logs
```

### Session Event Log Format

Simple newline-delimited JSON (JSONL):

```json
{"timestamp":"2024-01-15T10:00:00Z","event":"session_created","session_id":"sess_123","data":{"objective":"Build auth system","strategy":"parallel"}}
{"timestamp":"2024-01-15T10:00:05Z","event":"agent_spawned","session_id":"sess_123","data":{"agent_id":"agent_abc","role":"architect","task":"Design the authentication system"}}
{"timestamp":"2024-01-15T10:00:10Z","event":"agent_started","session_id":"sess_123","data":{"agent_id":"agent_abc","pid":12345}}
{"timestamp":"2024-01-15T10:15:30Z","event":"artifact_created","session_id":"sess_123","data":{"agent_id":"agent_abc","path":"shared/context/architecture/auth-design.md"}}
{"timestamp":"2024-01-15T10:20:00Z","event":"agent_completed","session_id":"sess_123","data":{"agent_id":"agent_abc","exit_code":0,"duration_seconds":900}}
{"timestamp":"2024-01-15T10:30:00Z","event":"session_completed","session_id":"sess_123","data":{"total_agents":3,"duration_seconds":1800}}
```

### Event Types

Core events we log:

| Event | Description | Data |
|-------|-------------|------|
| `session_created` | New orchestration session | objective, strategy, agent_count |
| `session_completed` | Session finished | total_agents, duration, status |
| `session_failed` | Session error | error_message, failed_agent |
| `agent_spawned` | Agent created | agent_id, role, task, dependencies |
| `agent_started` | Process started | agent_id, pid, workspace |
| `agent_completed` | Agent finished | agent_id, exit_code, duration |
| `agent_failed` | Agent error | agent_id, error, exit_code |
| `artifact_created` | File created in shared context | agent_id, path, type |
| `message_sent` | Inter-agent message | from_agent, to_agent, message_type |
| `checkpoint` | Progress milestone | agent_id, milestone, progress |

### Implementation

```rust
pub struct SessionLogger {
    log_file: File,
    session_id: String,
}

impl SessionLogger {
    pub fn new(session_dir: &Path, session_id: &str) -> Result<Self> {
        let log_path = session_dir.join("session.log");
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
            
        Ok(Self {
            log_file,
            session_id: session_id.to_string(),
        })
    }
    
    pub fn log_event(&mut self, event: &str, data: Value) -> Result<()> {
        let entry = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "event": event,
            "session_id": self.session_id,
            "data": data,
        });
        
        writeln!(self.log_file, "{}", entry)?;
        self.log_file.flush()?;
        Ok(())
    }
}

// Usage
logger.log_event("agent_spawned", json!({
    "agent_id": agent_id,
    "role": "architect",
    "task": "Design authentication system"
}))?;
```

### Log Rotation and Retention

```rust
pub struct LogManager {
    retention_days: u32,
}

impl LogManager {
    pub async fn cleanup_old_logs(&self) -> Result<()> {
        let cutoff = Utc::now() - Duration::days(self.retention_days as i64);
        
        // Walk through session directories
        for entry in WalkDir::new("~/.maos/projects") {
            if entry.path().ends_with("session.log") {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.modified()? < cutoff {
                        // Archive or delete old logs
                        self.archive_log(entry.path()).await?;
                    }
                }
            }
        }
        Ok(())
    }
}
```

### Querying Logs

Simple grep-based queries for common cases:

```bash
# Find all failed agents
grep '"event":"agent_failed"' ~/.maos/projects/*/sessions/*/session.log

# Get timeline for a session
grep '"session_id":"sess_123"' ~/.maos/projects/*/sessions/sess_123/session.log | jq -r '[.timestamp, .event] | @tsv'

# Find all architecture artifacts created
grep '"event":"artifact_created"' ~/.maos/projects/*/sessions/*/session.log | jq 'select(.data.path | contains("architecture"))'
```

For complex queries, we can build simple tools:

```rust
pub fn query_session_events(session_id: &str, event_type: Option<&str>) -> Result<Vec<LogEntry>> {
    let log_path = format!("~/.maos/projects/{}/sessions/{}/session.log", 
        get_workspace_hash(), session_id);
    
    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    
    let mut events = Vec::new();
    for line in reader.lines() {
        let entry: LogEntry = serde_json::from_str(&line?)?;
        if event_type.map_or(true, |t| entry.event == t) {
            events.push(entry);
        }
    }
    
    Ok(events)
}
```

## Consequences

### Positive
- **Simple Implementation**: Just append to files
- **Easy Debugging**: Human-readable logs, grep-friendly
- **Low Overhead**: No database transactions
- **Crash Safe**: Append-only files with flush
- **Standard Tools**: Use jq, grep, tail for analysis

### Negative
- **No Complex Queries**: Can't easily do SQL-like joins
- **No Automatic Replay**: Can't reconstruct state from events
- **Manual Cleanup**: Need log rotation strategy

### Mitigation
- Build simple query tools as needed
- SQLite maintains current state (don't need replay)
- Implement configurable retention policy

## Migration from Event Sourcing

This replaces the original complex event sourcing approach with pragmatic logging:

| Original Event Sourcing | Simple Logging |
|------------------------|----------------|
| PostgreSQL JSONB events | File-based JSONL |
| Event projections | Direct SQLite updates |
| Event replay | Read logs for debugging |
| CQRS patterns | Simple CRUD |
| Domain events | Log entries |

## References
- The Log: What every software engineer should know about real-time data's unifying abstraction
- JSONL (JSON Lines) format specification
- Unix philosophy: text streams as universal interface

---
*Date: 2025-07-09*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollettLaFollett Labs LLC)*