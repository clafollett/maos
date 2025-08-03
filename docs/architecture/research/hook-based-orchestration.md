# Hook-Based Orchestration for Claude Code Sub-Agents

## Overview

This document describes how MAOS provides backend orchestration support for Claude Code's native sub-agent capabilities through hooks and minimal utilities.

## Core Principle: Invisible Orchestration

MAOS operates entirely through hooks - users never interact with it directly. When Claude Code decides to parallelize work, MAOS hooks automatically:
1. Prepare isolated workspaces (git worktrees)
2. Prevent file conflicts
3. Track progress
4. Clean up when done

## Architecture

```
User Request → Claude Code → Decides to parallelize
                                     ↓
                            ┌────────────────────┐
                            │ pre_tool_use hook  │
                            │ (intercepts Task)  │
                            └────────┬───────────┘
                                     ↓
                            ┌────────────────────┐
                            │ MAOS Backend CLI   │
                            │ • Create worktree  │
                            │ • Setup workspace  │
                            └────────┬───────────┘
                                     ↓
                            Claude spawns agent with
                            modified prompt including
                            workspace path
```

## Implementation Components

### 1. Hook System

#### pre_tool_use.py
```python
def main():
    input_data = json.load(sys.stdin)
    tool_name = input_data.get('tool_name')
    tool_input = input_data.get('tool_input', {})
    
    # Intercept Task tool for sub-agent spawning
    if tool_name == "Task" and tool_input.get('subagent_type'):
        agent_type = tool_input['subagent_type']
        session_id = get_or_create_session()
        
        # Call backend to prepare workspace
        result = subprocess.run([
            "maos-backend", "prepare-workspace",
            agent_type, session_id
        ], capture_output=True, text=True)
        
        if result.returncode == 0:
            workspace = result.stdout.strip()
            # Modify prompt to include workspace
            tool_input['prompt'] += f"\n\nIMPORTANT: Work in {workspace}"
            
            # Update coordination state
            update_agent_registry(agent_type, session_id, workspace)
    
    # For file operations, check locks
    elif tool_name in ["Edit", "Write", "MultiEdit"]:
        file_path = tool_input.get('file_path')
        if file_path and check_file_lock(file_path):
            print(f"WARNING: {file_path} is being edited by another agent", 
                  file=sys.stderr)
    
    sys.exit(0)
```

#### post_tool_use.py
```python
def main():
    input_data = json.load(sys.stdin)
    tool_name = input_data.get('tool_name')
    
    # Track progress after operations
    if tool_name in ["Edit", "Write", "MultiEdit"]:
        update_progress(tool_name, input_data)
    
    # Clean up after task completion
    elif tool_name == "Task":
        check_and_cleanup_completed_agents()
```

### 2. Backend Utilities (Python Only)

Python modules imported by hooks - no shell scripts needed:

```python
# .claude/hooks/utils/maos_backend.py
import json
import subprocess
from pathlib import Path
from datetime import datetime

class MAOSBackend:
    """Backend utilities for MAOS - not user-facing"""
    
    def prepare_workspace(self, agent_type, session_id):
        """Create git worktree for agent"""
        workspace = f"worktrees/{agent_type}-{session_id}"
        branch = f"maos/{session_id}/{agent_type}"
        
        # Create worktree
        subprocess.run([
            "git", "worktree", "add", "-b", branch, workspace
        ], check=True)
        
        # Setup coordination files
        workspace_path = Path(workspace)
        maos_dir = workspace_path / ".maos"
        maos_dir.mkdir(exist_ok=True)
        
        agent_data = {
            "agent": agent_type,
            "session": session_id,
            "created": datetime.now().isoformat()
        }
        
        with open(maos_dir / "agent.json", 'w') as f:
            json.dump(agent_data, f, indent=2)
        
        return workspace
    
    def check_lock(self, file_path, agent_id):
        """Check if file is locked by another agent"""
        locks_file = Path(".maos/coordination/locks.json")
        if locks_file.exists():
            with open(locks_file) as f:
                locks = json.load(f)
                return locks.get(file_path)
        return None
    
    def cleanup_session(self, session_id):
        """Remove worktrees and cleanup state"""
        # Find and remove worktrees for this session
        worktrees_dir = Path("worktrees")
        if worktrees_dir.exists():
            for worktree in worktrees_dir.glob(f"*-{session_id}"):
                subprocess.run([
                    "git", "worktree", "remove", str(worktree)
                ], capture_output=True)
```

### 3. Coordination State

Simple file-based state in `.maos/`:

```
.maos/
├── session.json          # Current session info
├── coordination/
│   ├── agents.json      # Active agents and workspaces
│   ├── locks.json       # File locks
│   └── progress.json    # Task progress
└── logs/                # Audit trail
```

## Key Patterns

### 1. Workspace Preparation
When Claude spawns a sub-agent:
1. Hook intercepts Task tool
2. Backend creates git worktree
3. Hook modifies agent prompt to include workspace
4. Agent works in isolation

### 2. Conflict Prevention
Before file operations:
1. Hook checks if file is locked
2. Warns if another agent is editing
3. Updates lock registry after edit

### 3. Progress Tracking
After operations:
1. Post-hook updates progress
2. Tracks which agent did what
3. Provides visibility for orchestrator

### 4. Automatic Cleanup
When agents complete:
1. Post-hook detects completion
2. Backend removes worktree
3. Cleans up coordination state

## Benefits

1. **Zero User Learning**: Just use Claude normally
2. **Automatic Isolation**: No manual worktree management
3. **Conflict Prevention**: Hooks prevent issues before they happen
4. **Clean Integration**: Works with Claude's native features

## What This Is NOT

- NOT a user-facing CLI
- NOT a complex orchestrator
- NOT an agent framework
- NOT enterprise software

It's just backend glue that makes parallel agents work better.