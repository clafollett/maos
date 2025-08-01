# MAOS Worktree System - Developer Guide

## Overview

This guide explains how MAOS automatically manages git worktrees for Claude Code sub-agents. This is **internal documentation** for developers working on MAOS itself.

**Important**: End users never interact with worktrees directly. This all happens automatically through hooks.

## How It Works

When a user asks Claude Code to build something complex, Claude may decide to use multiple agents working in parallel. MAOS hooks automatically:

1. **Intercept** the Task tool when Claude spawns sub-agents
2. **Create** isolated git worktrees for each agent
3. **Track** progress through coordination files
4. **Clean up** when work is complete

## System Architecture

```
User: "Build authentication system"
  ↓
Claude Code: "I'll use multiple agents..."
  ↓
pre_tool_use.py hook intercepts Task tool
  ↓
MAOSBackend.prepare_workspace() creates worktree
  ↓
Agent works in isolated worktree
  ↓
post_tool_use.py hook tracks progress/cleanup
```

## Technical Implementation

### Hook Integration

```python
# .claude/hooks/pre_tool_use.py
if tool_name == "Task" and tool_input.get('subagent_type'):
    backend = MAOSBackend()
    workspace = backend.prepare_workspace(
        agent_type=tool_input['subagent_type'],
        session_id=backend.get_session_id()
    )
    # Modify agent prompt to include workspace
    tool_input['prompt'] += f"\n\nWork in: {workspace}/"
```

### Worktree Creation (Backend)

```python
# .claude/hooks/utils/maos_backend.py
def prepare_workspace(self, agent_type, issue_num, issue_summary):
    safe_summary = issue_summary.lower().replace(' ', '-')[:20]
    workspace = f"worktrees/{agent_type}-issue-{issue_num}-{safe_summary}"
    branch = f"wrktr/issue-{issue_num}/{agent_type}-{safe_summary}"
    
    # Create worktree
    subprocess.run([
        "git", "worktree", "add", "-b", branch, workspace
    ], check=True)
    
    # Track in coordination files
    self.register_agent(agent_type, session_id, workspace)
    
    return workspace
```

### Automatic Cleanup

```python
# .claude/hooks/post_tool_use.py
def cleanup_completed_worktrees():
    backend = MAOSBackend()
    for agent in backend.get_completed_agents():
        if agent['status'] == 'completed':
            backend.remove_worktree(agent['workspace'])
```

## Directory Structure

When MAOS creates worktrees, it follows this structure:

```
maos/
├── worktrees/                         # Auto-created agent worktrees
│   ├── backend-issue-42-auth/         # Backend agent workspace
│   │   └── (full repo copy)
│   ├── frontend-issue-42-auth/        # Frontend agent workspace
│   │   └── (full repo copy)
│   └── qa-issue-42-auth/              # QA agent workspace
│       └── (full repo copy)
└── .maos/                            # Coordination files
    ├── session.json                  # Current session metadata
    ├── coordination/
    │   ├── agents.json              # Active agent registry
    │   ├── locks.json               # File lock tracking
    │   └── progress.json            # Task completion status
    └── logs/                        # Event history
```

## Debugging MAOS

For developers working on MAOS itself, use these commands to debug:

```bash
# View current worktrees (for debugging only)
git worktree list

# Check MAOS session info
cat .maos/session.json | python -m json.tool

# Watch coordination files
just watch-maos

# Clean up after testing
just clean-maos
git worktree prune
```

## Implementation Details

### Coordination Files

MAOS uses simple JSON files for coordination:

```json
// .maos/coordination/agents.json
[
  {
    "type": "backend-engineer",
    "issue_num": "42",
    "issue_summary": "auth-system",
    "workspace": "worktrees/backend-issue-42-auth-system",
    "branch": "wrktr/issue-42/backend-auth-system",
    "created": "2024-01-30T10:00:00Z",
    "status": "active"
  }
]
```

### File Locking

Prevents agents from conflicting:

```json
// .maos/coordination/locks.json
{
  "src/api/auth.py": {
    "agent": "backend-engineer",
    "locked_at": "2024-01-30T10:00:00Z"
  }
}
```

### Progress Tracking

Monitors agent work:

```json
// .maos/coordination/progress.json
{
  "backend-engineer": {
    "status": "in_progress",
    "updated": 1706610000,
    "details": "Implementing login endpoint"
  }
}
```

## Contributing to MAOS

If you're improving MAOS itself:

1. **Test hooks locally** - Modify `.claude/hooks/*.py` and test with Claude Code
2. **Use Python only** - No shell scripts (avoids chmod issues)
3. **Keep it simple** - File-based coordination, no databases
4. **Maintain invisibility** - Users should never know MAOS exists

## Key Principles

- **Automatic**: Everything happens through hooks
- **Invisible**: Users never see or interact with MAOS
- **Simple**: Just Python and git commands
- **Reliable**: File-based state survives crashes

Remember: MAOS is backend infrastructure, not a user tool!