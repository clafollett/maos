#!/usr/bin/env python3
"""
Session Manager for MAOS - Each execution is a unique, immutable session.
"""

import uuid
import json
import shutil
import logging
from pathlib import Path
from datetime import datetime
from enum import Enum
from typing import Optional, List

# Import security utilities
from security_utils import sanitize_path_component, safe_path_join, validate_agent_name

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


# Security utilities are imported from security_utils module


class SessionStatus(Enum):
    ACTIVE = "active"
    COMPLETED = "completed"
    FAILED = "failed"
    TIMEOUT = "timeout"
    CANCELLED = "cancelled"
    PAUSED = "paused"


class SessionType(Enum):
    ORCHESTRATION = "orchestration"
    AGENT = "agent"


class MAOSSession:
    """Represents a single MAOS execution session."""
    
    def __init__(self, session_id: str, session_type: SessionType, 
                 task: str, parent_session: Optional[str] = None):
        self.session_id = sanitize_path_component(session_id)
        self.session_type = session_type
        self.task = task
        self.parent_session = sanitize_path_component(parent_session) if parent_session else None
        self.created = datetime.now()
        self.status = SessionStatus.ACTIVE
        self.metadata = {
            "agents": [],
            "workspace": f".maos/sessions/{self.session_id}",
            "logs": [],
            "checkpoints": [],
            "outputs": []
        }
    
    def to_dict(self) -> dict:
        return {
            "session_id": self.session_id,
            "type": self.session_type.value,
            "task": self.task,
            "parent_session": self.parent_session,
            "created": self.created.isoformat(),
            "status": self.status.value,
            "metadata": self.metadata
        }
    
    @classmethod
    def from_dict(cls, data: dict) -> 'MAOSSession':
        session = cls(
            session_id=data["session_id"],
            session_type=SessionType(data["type"]),
            task=data["task"],
            parent_session=data.get("parent_session")
        )
        session.created = datetime.fromisoformat(data["created"])
        session.status = SessionStatus(data["status"])
        session.metadata = data["metadata"]
        return session


class SessionManager:
    """Enhanced session manager with full lifecycle control."""
    
    def __init__(self):
        self.base_dir = Path(".maos")
        self.sessions_dir = self.base_dir / "sessions"
        self.registry_dir = self.base_dir / "registry"
        self.registry_dir.mkdir(parents=True, exist_ok=True)
        
    def create_session(self, task: str, session_type: SessionType = SessionType.ORCHESTRATION,
                      parent_session: Optional[str] = None, claude_session_id: Optional[str] = None) -> MAOSSession:
        """Create a new session - uses Claude session ID if provided, otherwise generates one."""
        session_id = sanitize_path_component(claude_session_id) if claude_session_id else str(uuid.uuid4())
        session = MAOSSession(session_id, session_type, task, parent_session)
        
        # Create workspace using safe path joining
        workspace = safe_path_join(self.sessions_dir, session_id)
        workspace.mkdir(parents=True, exist_ok=True)
        
        # Create subdirectories
        (workspace / "agents").mkdir(exist_ok=True)
        (workspace / "shared").mkdir(exist_ok=True)
        (workspace / "checkpoints").mkdir(exist_ok=True)
        (workspace / "logs").mkdir(exist_ok=True)
        
        # Save session metadata
        self._save_session(session)
        
        # Log session creation
        self._log_event(session_id, "session_created", {
            "task": task,
            "type": session_type.value
        })
        
        return session
    
    def get_session(self, session_id: str) -> Optional[MAOSSession]:
        """Retrieve a session by ID."""
        # Sanitize session_id
        safe_session_id = sanitize_path_component(session_id)
        registry_file = safe_path_join(self.registry_dir, f"{safe_session_id}.json")
        
        if not registry_file.exists():
            return None
            
        try:
            with open(registry_file, 'r') as f:
                data = json.load(f)
        except (json.JSONDecodeError, OSError) as e:
            logger.error(f"Failed to read session {safe_session_id}: {e}")
            return None
        
        return MAOSSession.from_dict(data)
    
    def get_sessions_by_claude_id(self, claude_session_id: str) -> List[MAOSSession]:
        """Get MAOS session by Claude session ID (they're the same now!)."""
        session = self.get_session(claude_session_id)
        return [session] if session else []
    
    def update_status(self, session_id: str, status: SessionStatus):
        """Update session status."""
        session = self.get_session(session_id)
        if session:
            session.status = status
            self._save_session(session)
            self._log_event(session_id, "status_changed", {"new_status": status.value})
            
            # If completing an orchestration session, clean up the marker
            if status == SessionStatus.COMPLETED:
                orchestration_marker = Path('.maos/.current_orchestration')
                if orchestration_marker.exists():
                    try:
                        with open(orchestration_marker, 'r') as f:
                            data = json.load(f)
                            if data.get('session_id') == session_id:
                                orchestration_marker.unlink()
                    except (json.JSONDecodeError, OSError) as e:
                        logger.warning(f"Failed to clean up orchestration marker: {e}")
    
    def add_checkpoint(self, session_id: str, checkpoint_name: str, data: dict):
        """Add a checkpoint to enable resuming."""
        session = self.get_session(session_id)
        if not session:
            return
            
        checkpoint = {
            "name": checkpoint_name,
            "timestamp": datetime.now().isoformat(),
            "data": data
        }
        
        # Save to checkpoint file using safe path
        checkpoint_file = safe_path_join(
            self.sessions_dir, session_id, "checkpoints", f"{sanitize_path_component(checkpoint_name)}.json"
        )
        checkpoint_file.parent.mkdir(exist_ok=True)
        
        try:
            with open(checkpoint_file, 'w') as f:
                json.dump(checkpoint, f, indent=2)
        except OSError as e:
            logger.error(f"Failed to save checkpoint {checkpoint_name}: {e}")
            return
        
        # Update session metadata
        session.metadata["checkpoints"].append(checkpoint_name)
        self._save_session(session)
        
        self._log_event(session_id, "checkpoint_created", {"checkpoint": checkpoint_name})
    
    def list_sessions(self, status: Optional[SessionStatus] = None,
                     limit: int = 20) -> List[MAOSSession]:
        """List sessions with optional filtering."""
        sessions = []
        
        try:
            registry_files = sorted(
                self.registry_dir.glob("*.json"), 
                key=lambda x: x.stat().st_mtime, 
                reverse=True
            )[:limit]
        except OSError as e:
            logger.error(f"Failed to list registry files: {e}")
            return sessions
        
        for registry_file in registry_files:
            try:
                with open(registry_file, 'r') as f:
                    data = json.load(f)
                
                session = MAOSSession.from_dict(data)
                if status is None or session.status == status:
                    sessions.append(session)
            except (json.JSONDecodeError, OSError) as e:
                logger.warning(f"Failed to load session from {registry_file}: {e}")
                continue
        
        return sessions
    
    def resume_session(self, session_id: str) -> Optional[dict]:
        """Get resume information for a session."""
        session = self.get_session(session_id)
        if not session:
            return None
        
        # Gather resume context
        resume_info = {
            "session": session.to_dict(),
            "workspace": str(safe_path_join(self.sessions_dir, session_id)),
            "checkpoints": []
        }
        
        # Load checkpoints
        checkpoint_dir = safe_path_join(self.sessions_dir, session_id, "checkpoints")
        if checkpoint_dir.exists():
            try:
                for checkpoint_file in checkpoint_dir.glob("*.json"):
                    with open(checkpoint_file, 'r') as f:
                        resume_info["checkpoints"].append(json.load(f))
            except (json.JSONDecodeError, OSError) as e:
                logger.warning(f"Failed to load checkpoints: {e}")
        
        return resume_info
    
    def fork_session(self, source_session_id: str, new_task: Optional[str] = None) -> Optional[MAOSSession]:
        """Create a new session based on an existing one."""
        source = self.get_session(source_session_id)
        if not source:
            return None
        
        # Create new session
        new_session = self.create_session(
            task=new_task or f"Fork of: {source.task}",
            session_type=source.session_type,
            parent_session=source_session_id
        )
        
        # Copy workspace structure (but not content)
        source_workspace = safe_path_join(self.sessions_dir, source_session_id)
        new_workspace = safe_path_join(self.sessions_dir, new_session.session_id)
        
        # Copy shared context only
        source_shared = source_workspace / "shared"
        if source_shared.exists():
            try:
                shutil.copytree(source_shared, new_workspace / "shared", dirs_exist_ok=True)
            except OSError as e:
                logger.error(f"Failed to copy shared context: {e}")
        
        self._log_event(new_session.session_id, "session_forked", {
            "source_session": source_session_id
        })
        
        return new_session
    
    def archive_session(self, session_id: str):
        """Archive a session (compress workspace, keep metadata)."""
        session = self.get_session(session_id)
        if not session:
            return
        
        workspace = safe_path_join(self.sessions_dir, session_id)
        if workspace.exists():
            # Create archive
            archive_dir = self.base_dir / "archives"
            archive_dir.mkdir(exist_ok=True)
            
            archive_path = archive_dir / f"{session_id}.tar.gz"
            
            try:
                shutil.make_archive(
                    str(archive_path.with_suffix('')), 
                    'gztar', 
                    workspace
                )
            except OSError as e:
                logger.error(f"Failed to archive session {session_id}: {e}")
                return
            
            # Remove original workspace
            try:
                shutil.rmtree(workspace)
            except OSError as e:
                logger.error(f"Failed to remove workspace after archiving: {e}")
            
            # Update session metadata
            session.metadata["archived"] = True
            session.metadata["archive_path"] = str(archive_path)
            self._save_session(session)
            
            self._log_event(session_id, "session_archived", {})
    
    def get_session_summary(self, session_id: str) -> Optional[str]:
        """Generate a human-readable summary of a session."""
        session = self.get_session(session_id)
        if not session:
            return None
        
        summary = f"""# Session Summary: {session_id[:8]}...

**Task**: {session.task}
**Status**: {session.status.value}
**Created**: {session.created.strftime('%Y-%m-%d %H:%M:%S')}
**Type**: {session.session_type.value}
**Workspace**: {session.metadata['workspace']}

## Agents Involved
{chr(10).join(f"- {agent}" for agent in session.metadata.get('agents', []))}

## Checkpoints
{chr(10).join(f"- {cp}" for cp in session.metadata.get('checkpoints', []))}

## Outputs
{chr(10).join(f"- {out}" for out in session.metadata.get('outputs', []))}
"""
        
        if session.parent_session:
            summary += f"\n**Parent Session**: {session.parent_session}"
        
        return summary
    
    def _save_session(self, session: MAOSSession):
        """Save session to registry."""
        registry_file = safe_path_join(self.registry_dir, f"{session.session_id}.json")
        try:
            with open(registry_file, 'w') as f:
                json.dump(session.to_dict(), f, indent=2)
        except OSError as e:
            logger.error(f"Failed to save session {session.session_id}: {e}")
    
    def add_agent_to_session(self, session_id: str, agent_name: str):
        """Add an agent to an orchestration session."""
        # Validate agent name
        if not validate_agent_name(agent_name):
            logger.error(f"Invalid agent name: {agent_name}")
            return
            
        session = self.get_session(session_id)
        if session and 'agents' in session.metadata:
            if agent_name not in session.metadata['agents']:
                session.metadata['agents'].append(agent_name)
                self._save_session(session)
                self._log_event(session_id, "agent_added", {"agent": agent_name})
    
    def _log_event(self, session_id: str, event_type: str, data: dict):
        """Log an event for a session."""
        log_entry = {
            "timestamp": datetime.now().isoformat(),
            "event": event_type,
            "data": data
        }
        
        log_file = safe_path_join(self.sessions_dir, session_id, "logs", "events.jsonl")
        log_file.parent.mkdir(parents=True, exist_ok=True)
        
        try:
            with open(log_file, 'a') as f:
                f.write(json.dumps(log_entry) + '\n')
        except OSError as e:
            logger.error(f"Failed to log event {event_type}: {e}")


# Convenience functions for command-line usage

def resolve_session_id(manager: SessionManager, partial_id: str) -> Optional[str]:
    """Resolve a partial session ID to a full ID."""
    sessions = manager.list_sessions()
    matches = [s for s in sessions if s.session_id.startswith(partial_id)]
    
    if not matches:
        print(f"No session found matching: {partial_id}")
        return None
    elif len(matches) > 1:
        print(f"Multiple sessions match: {partial_id}")
        for s in matches:
            print(f"  - {s.session_id}")
        return None
    else:
        return matches[0].session_id


def cli_main():
    """CLI interface for session management."""
    import sys
    
    if len(sys.argv) < 2:
        print("Usage: session_manager.py <command> [args]")
        print("Commands:")
        print("  list [status]        - List sessions")
        print("  show <session_id>    - Show session details")
        print("  resume <session_id>  - Get resume information")
        print("  fork <session_id>    - Fork a session")
        print("  archive <session_id> - Archive a session")
        print("  cleanup [session_id] - Clean up old sessions or specific session")
        print("  status <session_id> <new_status> - Update status")
        sys.exit(1)
    
    command = sys.argv[1]
    manager = SessionManager()
    
    if command == "list":
        status_filter = None
        if len(sys.argv) > 2:
            try:
                status_filter = SessionStatus(sys.argv[2])
            except ValueError:
                print(f"Invalid status: {sys.argv[2]}")
                print(f"Valid statuses: {', '.join(s.value for s in SessionStatus)}")
                sys.exit(1)
        
        sessions = manager.list_sessions(status=status_filter)
        print(f"\n{'ID':<12} {'Status':<12} {'Type':<15} {'Created':<20} {'Task':<50}")
        print("-" * 120)
        
        for session in sessions:
            print(f"{session.session_id[:8]}... {session.status.value:<12} "
                  f"{session.session_type.value:<15} "
                  f"{session.created.strftime('%Y-%m-%d %H:%M'):<20} "
                  f"{session.task[:50]}")
    
    elif command == "show" and len(sys.argv) > 2:
        partial_id = sys.argv[2]
        session_id = resolve_session_id(manager, partial_id)
        if session_id:
            summary = manager.get_session_summary(session_id)
            print(summary)
    
    elif command == "resume" and len(sys.argv) > 2:
        session_id = sys.argv[2]
        resume_info = manager.resume_session(session_id)
        if resume_info:
            print(json.dumps(resume_info, indent=2))
        else:
            print(f"Session not found: {session_id}")
    
    elif command == "fork" and len(sys.argv) > 2:
        source_id = sys.argv[2]
        new_task = sys.argv[3] if len(sys.argv) > 3 else None
        
        new_session = manager.fork_session(source_id, new_task)
        if new_session:
            print(f"Created fork: {new_session.session_id}")
            print(f"Task: {new_session.task}")
        else:
            print(f"Failed to fork session: {source_id}")
    
    elif command == "archive" and len(sys.argv) > 2:
        partial_id = sys.argv[2]
        session_id = resolve_session_id(manager, partial_id)
        if session_id:
            manager.archive_session(session_id)
            print(f"Archived session: {session_id}")
    
    elif command == "status" and len(sys.argv) > 3:
        session_id = sys.argv[2]
        new_status = sys.argv[3]
        
        try:
            status = SessionStatus(new_status)
            manager.update_status(session_id, status)
            print(f"Updated session {session_id} status to: {new_status}")
        except ValueError:
            print(f"Invalid status: {new_status}")
            print(f"Valid statuses: {', '.join(s.value for s in SessionStatus)}")
    
    elif command == "cleanup":
        if len(sys.argv) > 2:
            # Delete specific session
            partial_id = sys.argv[2]
            session_id = resolve_session_id(manager, partial_id)
            if session_id:
                # Create delete_session method
                session = manager.get_session(session_id)
                if session:
                    # Delete workspace
                    workspace = safe_path_join(manager.sessions_dir, session_id)
                    if workspace.exists():
                        try:
                            shutil.rmtree(workspace)
                        except OSError as e:
                            logger.error(f"Failed to delete workspace: {e}")
                    
                    # Delete registry
                    registry_file = safe_path_join(manager.registry_dir, f"{session_id}.json")
                    if registry_file.exists():
                        try:
                            registry_file.unlink()
                        except OSError as e:
                            logger.error(f"Failed to delete registry file: {e}")
                    
                    print(f"Deleted session: {session_id}")
                else:
                    print(f"Session not found: {session_id}")
        else:
            # Cleanup old sessions
            print("Cleaning up sessions older than 7 days...")
            # Simple cleanup - remove old registry files
            cutoff = datetime.now().timestamp() - (7 * 86400)
            count = 0
            for registry_file in manager.registry_dir.glob("*.json"):
                try:
                    if registry_file.stat().st_mtime < cutoff:
                        with open(registry_file, 'r') as f:
                            data = json.load(f)
                        session_id = data.get('session_id')
                        if session_id:
                            workspace = safe_path_join(manager.sessions_dir, session_id)
                            if workspace.exists():
                                shutil.rmtree(workspace)
                        registry_file.unlink()
                        count += 1
                except (json.JSONDecodeError, OSError) as e:
                    logger.warning(f"Failed to cleanup session {registry_file}: {e}")
                    continue
            print(f"Cleaned up {count} old sessions")


if __name__ == "__main__":
    cli_main()