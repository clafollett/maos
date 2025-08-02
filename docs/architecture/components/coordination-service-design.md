# Coordination Service Design

## Overview

The Coordination Service provides file-based communication and coordination patterns for MAOS agents. It enables agents to share context, synchronize work, and coordinate activities through simple JSON files without complex messaging infrastructure.

## Architecture

```mermaid
graph TB
    subgraph "MAOS Coordination"
        MC[MAOS Coordinator] --> SC[Session Coordinator]
        MC --> AC[Agent Coordinator]
        MC --> LC[Lock Coordinator]
        MC --> PC[Progress Coordinator]
    end
    
    subgraph "JSON Storage"
        SC --> SF[session.json]
        AC --> AF[agents.json]
        LC --> LF[locks.json]
        PC --> PF[progress.json]
    end
    
    subgraph "File System"
        SF --> SD[.maos/sessions/{id}/]
        AF --> SD
        LF --> SD
        PF --> SD
    end
    
    subgraph "Agents"
        A1[Backend Agent] --> MC
        A2[Frontend Agent] --> MC
        A3[QA Agent] --> MC
    end
```

## Core Components

### 1. Session Coordinator

Manages session lifecycle and metadata:

```python
import json
import time
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional

# Status constants
STATUS_ACTIVE = "active"
STATUS_COMPLETED = "completed"
STATUS_IN_PROGRESS = "in_progress"
STATUS_FAILED = "failed"
STATUS_PENDING = "pending"
STATUS_BLOCKED = "blocked"

class SessionCoordinator:
    """Manages MAOS session lifecycle and metadata"""
    
    def __init__(self, maos_dir: Path):
        self.maos_dir = maos_dir
        self.sessions_dir = maos_dir / "sessions"
        self.sessions_dir.mkdir(parents=True, exist_ok=True)
    
    def create_session(self, metadata: Optional[Dict] = None) -> str:
        """Create a new session with unique ID"""
        session_id = f"sess-{int(time.time())}"
        session_dir = self.sessions_dir / session_id
        session_dir.mkdir(parents=True, exist_ok=True)
        
        session_data = {
            "id": session_id,
            "created": datetime.now().isoformat(),
            "metadata": metadata or {},
            "status": STATUS_ACTIVE
        }
        
        # Initialize coordination files
        with open(session_dir / "session.json", 'w') as f:
            json.dump(session_data, f, indent=2)
        
        # Create empty coordination files
        for filename in ["agents.json", "locks.json", "progress.json"]:
            with open(session_dir / filename, 'w') as f:
                json.dump({} if "locks" in filename or "progress" in filename else [], f)
        
        return session_id
    
    def get_session(self, session_id: str) -> Optional[Dict]:
        """Retrieve session data"""
        session_file = self.sessions_dir / session_id / "session.json"
        if session_file.exists():
            with open(session_file) as f:
                return json.load(f)
        return None
    
    def end_session(self, session_id: str):
        """Mark session as completed"""
        session_file = self.sessions_dir / session_id / "session.json"
        if session_file.exists():
            with open(session_file, 'r+') as f:
                data = json.load(f)
                data["status"] = STATUS_COMPLETED
                data["ended"] = datetime.now().isoformat()
                f.seek(0)
                json.dump(data, f, indent=2)
                f.truncate()
```

### 2. Agent Coordinator

Tracks active agents and their workspaces:

```python
class AgentCoordinator:
    """Manages agent registration and tracking"""
    
    def __init__(self, session_dir: Path):
        self.session_dir = session_dir
        self.agents_file = session_dir / "agents.json"
    
    def register_agent(self, agent_type: str, workspace: str) -> Dict:
        """Register a new agent in the session"""
        agents = self._load_agents()
        
        agent_info = {
            "id": f"{agent_type}-{int(time.time())}",
            "type": agent_type,
            "workspace": workspace,
            "registered": datetime.now().isoformat(),
            "status": STATUS_ACTIVE,
            "last_heartbeat": datetime.now().isoformat()
        }
        
        agents.append(agent_info)
        self._save_agents(agents)
        
        return agent_info
    
    def update_agent_status(self, agent_id: str, status: str):
        """Update agent status"""
        agents = self._load_agents()
        
        for agent in agents:
            if agent["id"] == agent_id:
                agent["status"] = status
                agent["last_heartbeat"] = datetime.now().isoformat()
                break
        
        self._save_agents(agents)
    
    def get_active_agents(self) -> List[Dict]:
        """Get all active agents"""
        agents = self._load_agents()
        return [a for a in agents if a["status"] == STATUS_ACTIVE]
    
    def _load_agents(self) -> List[Dict]:
        """Load agents from file"""
        if self.agents_file.exists():
            with open(self.agents_file) as f:
                return json.load(f)
        return []
    
    def _save_agents(self, agents: List[Dict]):
        """Save agents to file"""
        with open(self.agents_file, 'w') as f:
            json.dump(agents, f, indent=2)
```

### 3. Lock Coordinator

Manages file locks to prevent conflicts:

```python
class LockCoordinator:
    """Manages file locks between agents"""
    
    def __init__(self, session_dir: Path):
        self.session_dir = session_dir
        self.locks_file = session_dir / "locks.json"
    
    def acquire_lock(self, file_path: str, agent_id: str) -> bool:
        """Attempt to acquire a lock on a file"""
        locks = self._load_locks()
        
        # Check if file is already locked
        if file_path in locks:
            existing_lock = locks[file_path]
            # Check if lock is stale (>5 minutes old)
            lock_time = datetime.fromisoformat(existing_lock["locked_at"])
            if (datetime.now(lock_time.tzinfo) - lock_time).total_seconds() > 300:
                # Stale lock, can override
                pass
            else:
                return False  # File is locked by another agent
        
        # Acquire lock
        locks[file_path] = {
            "agent_id": agent_id,
            "locked_at": datetime.now().isoformat()
        }
        
        self._save_locks(locks)
        return True
    
    def release_lock(self, file_path: str, agent_id: str) -> bool:
        """Release a lock on a file"""
        locks = self._load_locks()
        
        if file_path in locks and locks[file_path]["agent_id"] == agent_id:
            del locks[file_path]
            self._save_locks(locks)
            return True
        
        return False
    
    def get_locks_for_agent(self, agent_id: str) -> List[str]:
        """Get all locks held by an agent"""
        locks = self._load_locks()
        return [
            path for path, lock in locks.items() 
            if lock["agent_id"] == agent_id
        ]
    
    def _load_locks(self) -> Dict:
        """Load locks from file"""
        if self.locks_file.exists():
            with open(self.locks_file) as f:
                return json.load(f)
        return {}
    
    def _save_locks(self, locks: Dict):
        """Save locks to file"""
        with open(self.locks_file, 'w') as f:
            json.dump(locks, f, indent=2)
```

### 4. Progress Coordinator

Tracks task progress and completion:

```python
class ProgressCoordinator:
    """Tracks agent progress and task completion"""
    
    def __init__(self, session_dir: Path):
        self.session_dir = session_dir
        self.progress_file = session_dir / "progress.json"
    
    def update_progress(self, agent_id: str, task: str, status: str, details: Optional[str] = None):
        """Update progress for an agent's task"""
        progress = self._load_progress()
        
        if agent_id not in progress:
            progress[agent_id] = {}
        
        progress[agent_id][task] = {
            "status": status,
            "updated": datetime.now().isoformat(),
            "details": details
        }
        
        self._save_progress(progress)
    
    def get_agent_progress(self, agent_id: str) -> Dict:
        """Get all progress for an agent"""
        progress = self._load_progress()
        return progress.get(agent_id, {})
    
    def get_overall_progress(self) -> Dict:
        """Get progress summary for all agents"""
        progress = self._load_progress()
        
        summary = {
            "total_agents": len(progress),
            "by_status": {STATUS_COMPLETED: 0, STATUS_IN_PROGRESS: 0, STATUS_FAILED: 0, STATUS_PENDING: 0}
        }
        
        for agent_id, tasks in progress.items():
            for task, info in tasks.items():
                status = info["status"]
                if status in summary["by_status"]:
                    summary["by_status"][status] += 1
        
        return summary
    
    def _load_progress(self) -> Dict:
        """Load progress from file"""
        if self.progress_file.exists():
            with open(self.progress_file) as f:
                return json.load(f)
        return {}
    
    def _save_progress(self, progress: Dict):
        """Save progress to file"""
        with open(self.progress_file, 'w') as f:
            json.dump(progress, f, indent=2)
```

### 5. Main Coordination Service

Ties all coordinators together:

```python
class CoordinationService:
    """Main coordination service that orchestrates all coordinators
    
    The CoordinationService provides a unified interface for managing multi-agent
    coordination in MAOS. It handles session lifecycle, agent registration,
    file locking, and progress tracking through a file-based coordination system.
    
    Purpose:
    - Coordinate multiple Claude Code agents working in parallel
    - Prevent file conflicts through lock management
    - Track agent progress and session status
    - Provide visibility into multi-agent workflows
    
    Responsibilities:
    - Session management (creation, tracking, cleanup)
    - Agent coordination (registration, status, workspaces)
    - File lock management (acquisition, release, conflict prevention)
    - Progress tracking (task status, completion metrics)
    
    Usage:
        # Initialize the service
        coord_service = CoordinationService(project_root)
        
        # Start a new session
        session_id = coord_service.start_session({"task": "Build feature"})
        
        # Get coordinators for the session
        coords = coord_service.get_coordinators(session_id)
        
        # Coordinate file operations
        if coord_service.coordinate_file_edit(agent_id, file_path):
            # Perform file edit
            coord_service.release_file(agent_id, file_path)
    
    Architecture:
    - Uses JSON files for all coordination data
    - Stores data in .maos/sessions/{session_id}/ directory
    - Each session has agents.json, locks.json, progress.json files
    - Designed for simplicity and transparency (no database required)
    """
    
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.maos_dir = project_root / ".maos"
        self.maos_dir.mkdir(exist_ok=True)
        
        self.session_coordinator = SessionCoordinator(self.maos_dir)
        self._current_session = None
    
    def start_session(self, metadata: Optional[Dict] = None) -> str:
        """Start a new coordination session"""
        session_id = self.session_coordinator.create_session(metadata)
        self._current_session = session_id
        return session_id
    
    def get_coordinators(self, session_id: Optional[str] = None):
        """Get all coordinators for a session"""
        sid = session_id or self._current_session
        if not sid:
            raise ValueError("No active session")
        
        session_dir = self.maos_dir / "sessions" / sid
        
        return {
            "agent": AgentCoordinator(session_dir),
            "lock": LockCoordinator(session_dir),
            "progress": ProgressCoordinator(session_dir)
        }
    
    def coordinate_file_edit(self, agent_id: str, file_path: str, session_id: Optional[str] = None):
        """Coordinate a file edit operation"""
        coords = self.get_coordinators(session_id)
        
        # Try to acquire lock
        if coords["lock"].acquire_lock(file_path, agent_id):
            coords["progress"].update_progress(
                agent_id, f"edit_{file_path}", STATUS_IN_PROGRESS, 
                f"Editing {file_path}"
            )
            return True
        else:
            coords["progress"].update_progress(
                agent_id, f"edit_{file_path}", STATUS_BLOCKED, 
                f"Waiting for lock on {file_path}"
            )
            return False
    
    def release_file(self, agent_id: str, file_path: str, session_id: Optional[str] = None):
        """Release a file after editing"""
        coords = self.get_coordinators(session_id)
        
        if coords["lock"].release_lock(file_path, agent_id):
            coords["progress"].update_progress(
                agent_id, f"edit_{file_path}", STATUS_COMPLETED, 
                f"Finished editing {file_path}"
            )
            return True
        return False
```

## File Formats

### Session File
`.maos/sessions/{session_id}/session.json`:
```json
{
  "id": "sess-1706610000",
  "created": "2024-01-30T10:00:00Z",
  "metadata": {
    "user_request": "Build authentication system",
    "issue_number": "42"
  },
  "status": "active"
}
```

### Agents File
`.maos/sessions/{session_id}/agents.json`:
```json
[
  {
    "id": "backend-engineer-1706610000",
    "type": "backend-engineer",
    "workspace": "/path/to/worktrees/backend-engineer-sess-123",
    "registered": "2024-01-30T10:00:00Z",
    "status": "active",
    "last_heartbeat": "2024-01-30T10:05:00Z"
  }
]
```

### Locks File
`.maos/sessions/{session_id}/locks.json`:
```json
{
  "src/api/auth.py": {
    "agent_id": "backend-engineer-1706610000",
    "locked_at": "2024-01-30T10:00:00Z"
  }
}
```

### Progress File
`.maos/sessions/{session_id}/progress.json`:
```json
{
  "backend-engineer-1706610000": {
    "setup_auth_models": {
      "status": "completed",
      "updated": "2024-01-30T10:10:00Z",
      "details": "Created User and Session models"
    },
    "implement_login": {
      "status": "in_progress",
      "updated": "2024-01-30T10:15:00Z",
      "details": "Working on login endpoint"
    }
  }
}
```

## Integration Example

How the coordination service is used in hooks:

```python
# In pre_tool_use.py
from pathlib import Path
from maos_coordination import CoordinationService

def pre_tool_use(tool_name, tool_args):
    if tool_name == "Edit":
        coord_service = CoordinationService(Path.cwd())
        coords = coord_service.get_coordinators()
        
        file_path = tool_args.get("file_path")
        agent_id = get_current_agent_id()  # From context
        
        if not coord_service.coordinate_file_edit(agent_id, file_path):
            return {
                "error": "File is locked by another agent",
                "retry_after": 5
            }
    
    return tool_args
```

## Performance Characteristics

- **File Operations**: 1-5ms for JSON read/write
- **Lock Acquisition**: <1ms (simple file check)
- **Progress Updates**: 2-3ms
- **Session Creation**: 10-20ms (creates multiple files)

## Benefits of File-Based Coordination

1. **Simple**: No database or message queue needed
2. **Transparent**: Easy to debug by inspecting JSON files
3. **Resilient**: Survives process crashes
4. **Portable**: Works on any filesystem
5. **No Dependencies**: Pure Python standard library

This design provides robust coordination capabilities while maintaining MAOS's philosophy of simplicity and transparency.