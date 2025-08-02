# MAOS Implementation Guide

## Quick Start: Minimal Working System

This guide shows how to implement MAOS as a backend orchestration system for Claude Code's native sub-agents.

## Critical Implementation Checklist

### Phase 1: Core Requirements (MUST HAVE)

1. **Task Tool Interception** ✅
   - Detect when Claude spawns sub-agents via Task tool
   - Intercept `tool_name == "Task"` with `subagent_type`
   - Create isolated workspace for each agent
   - Modify agent prompt to include workspace path

2. **Git Worktree Management** ✅
   - Create worktree for each agent: `worktrees/{agent}-{session}`
   - Use branch naming: `wrktr/session-{id}/{agent}`
   - Lock worktrees to prevent accidental deletion
   - Clean up completed worktrees automatically

3. **Coordination System** ✅
   - Create `.maos/sessions/{session_id}/` directory structure
   - Track session metadata in `session.json`
   - List active agents in `agents.json`
   - Manage file locks in `locks.json`
   - Update progress in `progress.json`
   - Store observability data (orchestration, timeline, metrics)

### Phase 2: Enhanced Features

4. **File Lock Management**
   - Check locks before Edit/Write operations
   - Warn when files are being edited by another agent
   - Prevent conflicts between parallel agents

5. **Session Management**
   - Generate unique session IDs
   - Track session lifecycle
   - Clean up stale sessions

6. **Progress Tracking**
   - Monitor agent status
   - Track task completion
   - Provide visibility into parallel work

### Phase 3: Production Ready

7. **Error Handling**
   - Graceful git operation failures
   - Recover from corrupted state
   - Log errors for debugging

8. **Performance Optimization**
   - Fast hook execution (< 10ms)
   - Efficient file operations
   - Minimal overhead

9. **Cleanup Automation**
   - Remove completed worktrees
   - Prune stale branches
   - Archive old coordination files

## Phase 1: Basic Hook Integration (Day 1)

### 1.1 Create Basic Workspace Hook

```python
# .claude/hooks/pre_tool_use.py
#!/usr/bin/env python3
import json
import sys
import subprocess
import time
from pathlib import Path

def main():
    input_data = json.load(sys.stdin)
    tool_name = input_data.get('tool_name')
    tool_input = input_data.get('tool_input', {})
    
    # Only intercept Task tool for sub-agents
    if tool_name == "Task" and tool_input.get('subagent_type'):
        agent_type = tool_input['subagent_type']
        
        # Simple workspace creation
        timestamp = int(time.time())
        workspace = f"worktrees/{agent_type}-{timestamp}"
        subprocess.run([
            "git", "worktree", "add", "-b", 
            f"wrktr/session-{timestamp}/{agent_type}", workspace
        ])
        
        # Append workspace to prompt
        tool_input['prompt'] += f"\n\nWork in: {workspace}/"
        
        print(f"Created workspace: {workspace}", file=sys.stderr)
    
    sys.exit(0)

if __name__ == '__main__':
    main()
```

### 1.2 Test It Works

```bash
# Make hook executable
chmod +x .claude/hooks/pre_tool_use.py

# Test with Claude
# User: "Build a simple REST API"
# Claude will spawn agents, hook creates worktrees automatically
```

## Phase 2: Add Coordination (Day 2)

### 2.1 Create Backend Utilities

```python
# .claude/hooks/utils/maos_backend.py
# This is NOT for users - only for hooks to call

import json
import subprocess
import time
from pathlib import Path
from datetime import datetime

class MAOSBackend:
    def __init__(self):
        self.maos_dir = Path(".maos")
        self.maos_dir.mkdir(exist_ok=True)
        (self.maos_dir / "sessions").mkdir(exist_ok=True)
    
    def init_session(self, hook_metadata=None):
        """Initialize a new MAOS session"""
        # Try to get session ID from hook metadata first
        if hook_metadata and hook_metadata.get('session_id'):
            session_id = hook_metadata['session_id']
        else:
            session_id = f"sess-{int(time.time())}"
        
        # Create session directory
        session_dir = self.maos_dir / "sessions" / session_id
        session_dir.mkdir(parents=True, exist_ok=True)
        
        session_data = {
            "id": session_id,
            "start": datetime.now().isoformat(),
            "metadata": hook_metadata or {}
        }
        
        # Write session data to session-specific directory
        with open(session_dir / "session.json", 'w') as f:
            json.dump(session_data, f, indent=2)
        
        # Update active session pointer
        with open(self.maos_dir / "active_session.json", 'w') as f:
            json.dump({"session_id": session_id}, f, indent=2)
        
        return session_id
    
    def prepare_workspace(self, agent_type, session_id):
        """Create a git worktree for an agent"""
        workspace = f"worktrees/{agent_type}-{session_id}"
        branch = f"wrktr/session-{session_id}/{agent_type}"
        
        # Create worktree
        subprocess.run([
            "git", "worktree", "add", "-b", branch, workspace
        ], check=True)
        
        # Track in session-specific coordination
        session_dir = self.maos_dir / "sessions" / session_id
        agents_file = session_dir / "agents.json"
        
        if agents_file.exists():
            with open(agents_file) as f:
                agents = json.load(f)
        else:
            agents = []
        
        agents.append({
            "type": agent_type,
            "session": session_id,
            "workspace": workspace,
            "created": datetime.now().isoformat()
        })
        
        with open(agents_file, 'w') as f:
            json.dump(agents, f, indent=2)
        
        return workspace
    
    def cleanup(self):
        """Remove completed worktrees"""
        worktrees_dir = Path("worktrees")
        if not worktrees_dir.exists():
            return
        
        for worktree in worktrees_dir.iterdir():
            if worktree.is_dir():
                # Check if work is complete (no uncommitted changes)
                result = subprocess.run(
                    ["git", "-C", str(worktree), "diff", "--quiet"],
                    capture_output=True
                )
                if result.returncode == 0:
                    # Clean, can remove
                    subprocess.run(
                        ["git", "worktree", "remove", str(worktree)],
                        capture_output=True
                    )
```

### 2.2 Enhanced Hook with Coordination

```python
# .claude/hooks/pre_tool_use.py (enhanced)
import json
import sys
import os
from pathlib import Path

# Import our backend utilities
sys.path.append(str(Path(__file__).parent))
from utils.maos_backend import MAOSBackend

class MAOSCoordinator:
    def __init__(self, hook_metadata=None):
        self.backend = MAOSBackend()
        self.maos_dir = Path(".maos")
        self.hook_metadata = hook_metadata or {}
        self._session_id = None
    
    def get_session_id(self):
        if self._session_id:
            return self._session_id
        
        # Try to get from hook metadata
        if self.hook_metadata.get('session_id'):
            self._session_id = self.hook_metadata['session_id']
            return self._session_id
        
        # Try to get active session
        active_file = self.maos_dir / "active_session.json"
        if active_file.exists():
            with open(active_file) as f:
                self._session_id = json.load(f).get('session_id')
                if self._session_id:
                    return self._session_id
        
        # Create new session
        self._session_id = self.backend.init_session(self.hook_metadata)
        return self._session_id
    
    def prepare_workspace(self, agent_type):
        session_id = self.get_session_id()
        return self.backend.prepare_workspace(agent_type, session_id)
    
    def check_file_lock(self, file_path):
        session_id = self.get_session_id()
        locks_file = self.maos_dir / "sessions" / session_id / "locks.json"
        if locks_file.exists():
            with open(locks_file) as f:
                locks = json.load(f)
                return locks.get(file_path)
        return None

def main():
    input_data = json.load(sys.stdin)
    hook_metadata = input_data.get('metadata', {})
    coordinator = MAOSCoordinator(hook_metadata)
    tool_name = input_data.get('tool_name')
    tool_input = input_data.get('tool_input', {})
    
    # Handle sub-agent spawning
    if tool_name == "Task" and tool_input.get('subagent_type'):
        agent_type = tool_input['subagent_type']
        workspace = coordinator.prepare_workspace(agent_type)
        tool_input['prompt'] += f"\n\nIMPORTANT: Work exclusively in {workspace}/"
    
    # Handle file operations
    elif tool_name in ["Edit", "Write", "MultiEdit"]:
        file_path = tool_input.get('file_path', '')
        lock = coordinator.check_file_lock(file_path)
        if lock and lock['agent'] != os.environ.get('CLAUDE_AGENT_ID'):
            print(f"⚠️  {file_path} is being edited by {lock['agent']}", 
                  file=sys.stderr)
    
    sys.exit(0)

if __name__ == '__main__':
    main()
```

## Phase 3: Polish and Automation (Day 3)

### 3.1 Add Cleanup Hook

```python
# .claude/hooks/post_tool_use.py
#!/usr/bin/env python3
import json
import sys
from pathlib import Path

# Import our backend utilities
sys.path.append(str(Path(__file__).parent))
from utils.maos_backend import MAOSBackend

def main():
    # Periodically cleanup completed worktrees
    backend = MAOSBackend()
    backend.cleanup()
    sys.exit(0)

if __name__ == '__main__':
    main()
```

### 3.2 Add Status Tracking

```python
# .claude/hooks/utils/status.py
import json
import time
from pathlib import Path

def update_agent_progress(agent_type, status, session_id, details=None):
    """Update agent progress in session-specific coordination files"""
    session_dir = Path(f".maos/sessions/{session_id}")
    progress_file = session_dir / "progress.json"
    
    if progress_file.exists():
        with open(progress_file) as f:
            progress = json.load(f)
    else:
        progress = {}
    
    progress[agent_type] = {
        "status": status,
        "updated": time.time(),
        "details": details
    }
    
    with open(progress_file, 'w') as f:
        json.dump(progress, f, indent=2)
```

## Testing Your Implementation

### Basic Test Flow

1. **User Request**:
   ```
   "Build a user authentication system with login and registration"
   ```

2. **Claude Orchestrator Response**:
   ```
   I'll build this using multiple specialized agents:
   - Backend engineer for API
   - Frontend engineer for UI
   - Security specialist for review
   ```

3. **Behind the Scenes**:
   - Hook intercepts Task tool calls
   - Creates worktrees automatically
   - Agents work in isolation
   - No user interaction needed

### Verify It's Working

```bash
# Check worktrees were created
git worktree list

# Check coordination files
ls -la .maos/coordination/

# Monitor agent progress
watch -n 1 'cat .maos/coordination/progress.json'
```

## Common Issues and Solutions

### Issue: Hooks Not Running
```bash
# Ensure hooks are executable
chmod +x .claude/hooks/*.py

# Check Claude Code sees them
ls -la .claude/hooks/
```

### Issue: Worktree Conflicts
```bash
# Manual cleanup if needed
git worktree prune
git worktree remove worktrees/old-agent-dir
```

### Issue: Coordination State Corruption
```bash
# Reset coordination
rm -rf .maos/coordination
mkdir -p .maos/coordination
echo "[]" > .maos/coordination/agents.json
```

## What NOT to Build

1. **User-facing CLI** - Users talk to Claude, not MAOS
2. **Complex orchestration** - Claude decides what agents to spawn
3. **Agent framework** - Use Claude's native Task tool
4. **Database** - Simple JSON files are enough

## Summary

MAOS is just backend glue that makes Claude Code's parallel agents work better:
- Hooks intercept operations
- Backend utilities manage worktrees
- Coordination through simple files
- Users never know it exists

Total implementation: ~200 lines of code across 3-4 files.