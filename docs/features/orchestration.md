# MAOS Multi-Agent Orchestration

## Overview

MAOS enables Claude Code to coordinate multiple AI agents working in parallel on complex tasks. Each agent operates in an isolated git worktree, preventing conflicts while enabling sophisticated multi-agent workflows.

## How Orchestration Works

### 1. User Request → Claude Orchestrator

When you ask Claude to build something complex:
```
User: "Build a complete authentication system with frontend and backend"
```

Claude analyzes the request and decides to parallelize the work.

### 2. Automatic Workspace Creation

When Claude spawns sub-agents via the Task tool, MAOS automatically:
- Detects the Task tool usage in pre-tool-use hook
- Creates an isolated git worktree for each agent
- Injects the workspace path into the agent's context
- Tracks the agent in the session

### 3. Parallel Development

Each agent works independently:
```
main/                           # Your main branch (protected)
worktrees/
├── backend-sess-123/          # Backend engineer workspace
├── frontend-sess-123/         # Frontend engineer workspace
├── security-sess-123/         # Security reviewer workspace
└── qa-sess-123/              # QA engineer workspace
```

### 4. Coordination & Integration

Agents coordinate through:
- File locks to prevent conflicts
- Progress tracking for visibility
- Session management for grouping related work
- Orchestrator merges completed work

## Key Features

### Workspace Isolation

Each agent gets a complete copy of the repository:
```bash
# Backend agent's view
/Users/you/project/worktrees/backend-sess-123/
├── src/
├── package.json
├── README.md
└── ... (full repo copy)
```

**Benefits:**
- No merge conflicts during development
- Agents can run tests independently
- Full git history available
- Clean integration back to main

### Lazy Worktree Creation

Worktrees are created only when needed:
1. Agent is spawned via Task tool
2. Agent attempts first file operation
3. MAOS creates worktree on-demand
4. Subsequent operations use existing worktree

This prevents unused worktrees from cluttering your repository.

### Session Management

All agents in a workflow are grouped by session:

```json
{
  "session_id": "sess-1754237743",
  "started_at": "2024-01-20T10:30:00Z",
  "orchestrator": "main",
  "agents": [
    {
      "id": "backend-sess-1754237743",
      "type": "backend-engineer",
      "status": "active",
      "worktree": "worktrees/backend-sess-1754237743"
    },
    {
      "id": "frontend-sess-1754237743",
      "type": "frontend-engineer",
      "status": "active",
      "worktree": "worktrees/frontend-sess-1754237743"
    }
  ]
}
```

### File Lock System

Prevents simultaneous edits to the same file:

```json
{
  "src/auth/login.ts": {
    "locked_by": "frontend-sess-123",
    "locked_at": "2024-01-20T10:35:00Z"
  },
  "src/api/auth.rs": {
    "locked_by": "backend-sess-123",
    "locked_at": "2024-01-20T10:36:00Z"
  }
}
```

### Progress Tracking

Real-time visibility into agent activities:

```json
{
  "backend-sess-123": {
    "tasks_completed": 5,
    "current_task": "Implementing JWT validation",
    "last_update": "2024-01-20T10:40:00Z"
  },
  "frontend-sess-123": {
    "tasks_completed": 3,
    "current_task": "Building login form",
    "last_update": "2024-01-20T10:41:00Z"
  }
}
```

## Orchestration Patterns

### 1. Parallel Development Pattern

Best for independent features:
```
Orchestrator decides:
- Backend: Build REST API
- Frontend: Create UI components
- Database: Design schema
- DevOps: Setup deployment

All work in parallel, merge when complete.
```

### 2. Pipeline Pattern

Best for dependent tasks:
```
Orchestrator coordinates:
1. Backend creates API endpoints
2. Frontend waits, then integrates
3. QA tests the integration
4. Security reviews everything
```

### 3. Review Pattern

Best for quality assurance:
```
Orchestrator workflow:
1. Developer agents implement features
2. Code reviewer examines changes
3. Security auditor checks for vulnerabilities
4. Orchestrator incorporates feedback
```

### 4. Exploration Pattern

Best for research tasks:
```
Multiple agents explore different approaches:
- Agent 1: Try approach A
- Agent 2: Try approach B
- Agent 3: Research best practices
Orchestrator selects best solution
```

## Configuration

### Orchestration Settings

```json
{
  "maos": {
    "orchestration": {
      "max_parallel_agents": 5,
      "worktree_cleanup": true,
      "session_timeout_minutes": 120,
      "lock_timeout_seconds": 300
    }
  }
}
```

### Agent Type Configuration

```json
{
  "maos": {
    "agents": {
      "backend-engineer": {
        "branch_prefix": "backend",
        "default_timeout": 1800
      },
      "frontend-engineer": {
        "branch_prefix": "frontend",
        "default_timeout": 1800
      }
    }
  }
}
```

## Workspace Management

### Viewing Active Sessions

```bash
# List all active sessions
maos session-list

# Show current session details
maos session-info

# List all worktrees
maos worktree-list
```

### Manual Cleanup

```bash
# Clean up completed session
maos session-cleanup sess-123

# Remove all inactive worktrees
maos worktree-prune

# Force cleanup (use with caution)
maos cleanup --all --force
```

## Best Practices

### For Users

1. **Let Claude orchestrate**: Don't manually assign agents
2. **Trust the process**: Agents coordinate automatically
3. **Review before merging**: Check orchestrator's integration
4. **Clean up regularly**: Remove old worktrees periodically

### For Complex Tasks

1. **Be specific**: Clear requirements help orchestration
2. **Think modular**: Break down into independent pieces
3. **Define interfaces**: Clear contracts between components
4. **Set expectations**: Mention if order matters

## Technical Implementation

### Worktree Creation

```rust
pub fn create_agent_worktree(
    agent_type: &str,
    session_id: &str
) -> Result<PathBuf, WorktreeError> {
    let branch_name = format!("agent/{}-{}", agent_type, session_id);
    let worktree_path = format!("worktrees/{}-{}", agent_type, session_id);
    
    // Create git worktree
    Command::new("git")
        .args(&["worktree", "add", "-b", &branch_name, &worktree_path])
        .output()?;
        
    Ok(PathBuf::from(worktree_path))
}
```

### Session Coordination

```rust
pub struct Session {
    id: String,
    agents: HashMap<String, Agent>,
    locks: FileLockManager,
    progress: ProgressTracker,
}

impl Session {
    pub fn coordinate(&mut self) -> Result<(), OrchestrationError> {
        // Update progress
        self.progress.update();
        
        // Check for conflicts
        self.locks.validate()?;
        
        // Coordinate agent activities
        for agent in self.agents.values_mut() {
            agent.check_status()?;
        }
        
        Ok(())
    }
}
```

## Monitoring & Debugging

### Session Logs

```
.maos/sessions/sess-123/
├── session.json       # Session metadata
├── timeline.json      # Event timeline
├── agents/           # Per-agent logs
│   ├── backend.log
│   └── frontend.log
└── metrics.json      # Performance data
```

### Debug Mode

Enable detailed orchestration logging:
```bash
export MAOS_DEBUG=1
export MAOS_LOG_LEVEL=trace
```

## Limitations

1. **Local only**: Orchestration happens on your machine
2. **Git required**: Needs git 2.5+ for worktree support
3. **Disk space**: Each worktree is a full repo copy
4. **Complexity**: Large orchestrations can be hard to follow

## Future Enhancements

- [ ] Distributed orchestration across machines
- [ ] Partial worktrees for large repos
- [ ] Visual orchestration dashboard
- [ ] Agent communication protocols
- [ ] Checkpoint and resume capabilities
- [ ] Intelligent agent selection

## Related Documentation

- [Architecture](../architecture/MAOS-Architecture.md) - System overview
- [Security](./security.md) - Workspace isolation details
- [Commands](../cli/commands.md) - Orchestration commands