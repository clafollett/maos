# MAOS True Architecture - Hook-Based Backend Orchestration

## What MAOS Actually Is

MAOS (Multi-Agent Orchestration System) is a **backend orchestration system** that enhances Claude Code's native sub-agent capabilities through hooks and git worktree isolation. It is **NOT** a user-facing tool.

## Key Principle: MAOS is Invisible

Users never interact with MAOS directly. They simply:
1. Talk to Claude Code normally
2. Claude Code decides when to parallelize work
3. Everything else happens automatically behind the scenes

## The Real Architecture

```
┌──────────────────────────────────────────────────────────┐
│                         USER                             │
│                   (Uses Claude Code normally)            │
└───────────────────────────┬──────────────────────────────┘
                            │ Natural language
                            ▼
┌──────────────────────────────────────────────────────────┐
│                     CLAUDE CODE                          │
│              (Main instance / Orchestrator)              │
│  • Receives user request                                 │
│  • Decides to parallelize work                           │
│  • Uses Task tool to spawn sub-agents                    │
└──────────┬──────────────────┬────────────────────────────┘
           │                  │
           │ Hooks            │ Task tool
           ▼                  ▼
┌─────────────────┐  ┌─────────────────────────────────────┐
│   MAOS HOOKS    │  │          SUB-AGENTS                 │
│ • pre_tool_use  │  │  • Backend Engineer (in worktree)   │
│ • post_tool_use │  │  • Frontend Engineer (in worktree)  │
│ • Intercept ops │  │  • QA Engineer (in worktree)        │
└────────┬────────┘  └─────────────────────────────────────┘
         │ Calls
         ▼
┌──────────────────────────────────────────────────────────┐
│                    MAOS CLI (Backend Only)               │
│  • Git worktree management                               │
│  • File-based coordination                               │
│  • Lock management                                       │
│  • Cleanup utilities                                     │
└──────────────────────────────────────────────────────────┘
```

## How It Actually Works

### 1. User Request
```
User: "Build me a complete authentication system"
```

### 2. Claude Code Orchestration
Claude (orchestrator role) analyzes the request and decides to parallelize:
```markdown
I'll build this authentication system using multiple specialized agents working in parallel:
- Backend engineer for API
- Frontend engineer for UI
- Security specialist for review
- QA engineer for testing
```

### 3. Hook Interception (Automatic)
When Claude prepares to spawn agents, hooks intercept:

```python
# .claude/hooks/pre_tool_use.py
if tool_name == "Task" and params.get("subagent_type"):
    # MAOS CLI creates worktree
    subprocess.run(["maos-internal", "prepare-workspace", 
                    params["subagent_type"], session_id])
    
    # Modify params to include workspace path
    params["prompt"] += f"\nWork in: ./worktrees/{agent_id}/"
```

### 4. Sub-Agent Spawning
Claude uses native Task tool:
```python
Task(
    subagent_type="backend-engineer",
    prompt="Implement authentication API endpoints. Work in: ./worktrees/backend-auth-123/"
)
```

### 5. Parallel Work in Isolation
Each agent works in its own git worktree:
```
main/                          # Original repository
worktrees/
├── backend-auth-123/         # Backend engineer workspace
├── frontend-auth-123/        # Frontend engineer workspace
├── security-auth-123/        # Security specialist workspace
└── qa-auth-123/             # QA engineer workspace
```

### 6. Coordination (File-Based)
Agents coordinate through session-scoped files:
```
.maos/
├── active_session.json       # Points to current session
├── sessions/
│   └── {session_id}/        # Per-session coordination
│       ├── session.json     # Session metadata
│       ├── agents.json      # Active agents in session
│       ├── locks.json       # File locks for session
│       ├── progress.json    # Task completion tracking
│       ├── orchestration.json # Agent relationships
│       ├── timeline.json    # Event timeline
│       └── metrics.json     # Performance metrics
└── logs/                    # Audit trail
```

## What MAOS is NOT

1. **NOT a user-facing CLI** - Users never type `maos` commands
2. **NOT an agent spawner** - Claude Code's Task tool does that
3. **NOT a complex orchestrator** - Just backend utilities
4. **NOT enterprise software** - Single developer tool

## MAOS Components

### 1. Hook System (Python)
Located in `.claude/hooks/`:
- **pre_tool_use.py**: Intercepts operations for coordination
- **post_tool_use.py**: Updates state after operations
- **utils/**: Helper functions for hooks

### 2. Backend Utilities (Python)
Python modules in `.claude/hooks/utils/` for hooks to import:
```python
# NOT for users - only imported by hooks
from utils.maos_backend import MAOSBackend

backend = MAOSBackend()
backend.prepare_workspace(agent_type, session_id)
backend.check_lock(file_path, agent_id)
backend.update_progress(task_id, status)
backend.cleanup_session(session_id)
```

### 3. Coordination Files
Simple JSON files in `.maos/`:
- No database needed
- File system is the source of truth
- Atomic operations for consistency

## Example: Complete Flow

1. **User**: "Add password reset feature"

2. **Claude Orchestrator**: 
   - Decides to spawn backend and frontend agents
   - Hook intercepts Task tool usage

3. **Hook Actions**:
   ```bash
   # Automatically called by pre_tool_use hook
   maos-internal prepare-workspace backend-engineer sess-789
   maos-internal prepare-workspace frontend-engineer sess-789
   ```

4. **Agents Work**:
   - Backend engineer implements API in `worktrees/backend-sess-789/`
   - Frontend engineer builds UI in `worktrees/frontend-sess-789/`
   - Hooks prevent file conflicts

5. **Completion**:
   - Orchestrator merges work back to main
   - Hook calls cleanup: `maos-internal cleanup-session sess-789`

## Implementation Priorities

### Phase 1: Core MAOS Functionality (Critical)

**Must implement these three things:**

1. **Task Tool Interception**
   - Hook detects `tool_name == "Task"` with `subagent_type`
   - Creates workspace and modifies agent prompt
   - Example: `tool_input['prompt'] += f"\n\nWork in: {workspace}/"`

2. **Git Worktree Creation**
   - Branch pattern: `wrktr/session-{id}/{agent}`
   - Workspace: `worktrees/{agent}-session-{id}`
   - Lock worktrees to prevent deletion

3. **Coordination Directory**
   - Create `.maos/coordination/`
   - Initialize: `session.json`, `agents.json`, `locks.json`
   - Track all active work

### Phase 2: Enhanced Coordination
1. File locking mechanism
2. Progress tracking
3. Post-tool hooks for state updates

### Phase 3: Production Polish
1. Automatic cleanup
2. Better error handling
3. Performance optimization

## Key Design Decisions

1. **Hooks Over Commands**: Users never need to learn MAOS
2. **Files Over Database**: Simple, debuggable, portable
3. **Git Native**: Leverage git's power for isolation
4. **Minimal Dependencies**: Just git and basic scripting

## Success Criteria

1. **Invisible to Users**: They just talk to Claude normally
2. **Prevents Conflicts**: Agents don't step on each other
3. **Easy to Debug**: Just look at files and git branches
4. **Fast**: Minimal overhead on operations

## What We're NOT Building

- User-facing commands
- Complex orchestration engine  
- Enterprise features
- Distributed systems
- Agent spawning (Claude does that)

## Summary

MAOS is a simple backend system that makes Claude Code's native sub-agents work better through:
1. Git worktree isolation
2. Hook-based coordination
3. File-based state tracking

Users get parallel AI development without learning anything new.