#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# ///

import json
import subprocess
import time
import os
from pathlib import Path
from datetime import datetime
from typing import Dict, Optional, Any


def get_project_root():
    """Get project root using git or current working directory."""
    try:
        root = subprocess.check_output(
            ['git', 'rev-parse', '--show-toplevel'],
            stderr=subprocess.DEVNULL,
            text=True
        ).strip()
        return Path(root)
    except:
        return Path.cwd()


# Global path constants - always use absolute paths
PROJECT_ROOT = get_project_root()
MAOS_DIR = PROJECT_ROOT / '.maos'
LOGS_DIR = PROJECT_ROOT / 'logs'
HOOKS_DIR = PROJECT_ROOT / '.claude' / 'hooks'
WORKTREES_DIR = PROJECT_ROOT / 'worktrees'


def run_git_command(cmd, cwd=None):
    """Run git command with explicit directory specification."""
    if cwd is None:
        cwd = PROJECT_ROOT
    # Always run git with explicit directory
    full_cmd = ['git', '-C', str(cwd)] + cmd
    return subprocess.run(full_cmd, capture_output=True, text=True)

class MAOSBackend:
    """Backend utilities for MAOS orchestration - not user-facing"""
    
    def __init__(self):
        # Use absolute paths from global constants
        self.maos_dir = MAOS_DIR
        self.sessions_dir = self.maos_dir / "sessions"
        self.worktrees_dir = WORKTREES_DIR
        
        # Ensure directories exist
        self.maos_dir.mkdir(exist_ok=True)
        self.sessions_dir.mkdir(exist_ok=True)
        self.worktrees_dir.mkdir(exist_ok=True)
    
    def get_or_create_session(self, hook_metadata: Optional[Dict] = None) -> str:
        """Get active session ID or create new one"""
        # Try to get from hook metadata first
        if hook_metadata and hook_metadata.get('session_id'):
            session_id = hook_metadata['session_id']
        else:
            # Check for active session
            active_file = self.maos_dir / "active_session.json"
            if active_file.exists():
                try:
                    with open(active_file) as f:
                        session_id = json.load(f).get('session_id')
                        if session_id:
                            return session_id
                except (json.JSONDecodeError, KeyError):
                    pass
            
            # Create new session
            session_id = f"sess-{int(time.time())}"
        
        # Initialize session
        self.init_session(session_id, hook_metadata)
        return session_id
    
    def init_session(self, session_id: str, hook_metadata: Optional[Dict] = None):
        """Initialize a new MAOS session"""
        session_dir = self.sessions_dir / session_id
        session_dir.mkdir(parents=True, exist_ok=True)
        
        session_data = {
            "id": session_id,
            "start": datetime.now().isoformat(),
            "metadata": hook_metadata or {},
            "status": "active"
        }
        
        # Write session data
        with open(session_dir / "session.json", 'w') as f:
            json.dump(session_data, f, indent=2)
        
        # Initialize coordination files
        with open(session_dir / "agents.json", 'w') as f:
            json.dump([], f, indent=2)
        
        with open(session_dir / "pending_agents.json", 'w') as f:
            json.dump({}, f, indent=2)
        
        with open(session_dir / "locks.json", 'w') as f:
            json.dump({}, f, indent=2)
        
        with open(session_dir / "progress.json", 'w') as f:
            json.dump({}, f, indent=2)
        
        # Update active session pointer
        with open(self.maos_dir / "active_session.json", 'w') as f:
            json.dump({"session_id": session_id}, f, indent=2)
    
    def register_pending_agent(self, agent_type: str, session_id: str) -> str:
        """Register an agent for lazy workspace creation"""
        agent_id = f"{agent_type}-{session_id}-{int(time.time())}"
        
        # Store pending agent info in session
        session_dir = self.sessions_dir / session_id
        pending_file = session_dir / "pending_agents.json"
        
        # Load existing pending agents
        pending_agents = {}
        if pending_file.exists():
            try:
                with open(pending_file) as f:
                    pending_agents = json.load(f)
            except (json.JSONDecodeError, FileNotFoundError):
                pending_agents = {}
        
        # Add new pending agent
        pending_agents[agent_id] = {
            "type": agent_type,
            "session": session_id,
            "registered_at": datetime.now().isoformat(),
            "workspace_created": False,
            "workspace_path": None
        }
        
        # Save updated pending agents
        with open(pending_file, 'w') as f:
            json.dump(pending_agents, f, indent=2)
        
        return agent_id
    
    def get_agent_info(self, agent_id: str, session_id: str) -> Optional[Dict]:
        """Get information about a pending or active agent"""
        session_dir = self.sessions_dir / session_id
        
        # Check pending agents first
        pending_file = session_dir / "pending_agents.json"
        if pending_file.exists():
            try:
                with open(pending_file) as f:
                    pending_agents = json.load(f)
                    if agent_id in pending_agents:
                        return pending_agents[agent_id]
            except (json.JSONDecodeError, FileNotFoundError):
                pass
        
        # Check active agents
        agents_file = session_dir / "agents.json"
        if agents_file.exists():
            try:
                with open(agents_file) as f:
                    agents = json.load(f)
                    for agent in agents:
                        if agent.get("id") == agent_id:
                            return agent
            except (json.JSONDecodeError, FileNotFoundError):
                pass
        
        return None
    
    def create_workspace_if_needed(self, agent_id: str, session_id: str) -> Optional[str]:
        """Create workspace for agent if not already created"""
        agent_info = self.get_agent_info(agent_id, session_id)
        if not agent_info:
            return None
        
        # If workspace already created, return its path
        if agent_info.get("workspace_created") and agent_info.get("workspace_path"):
            return agent_info["workspace_path"]
        
        # Create workspace
        agent_type = agent_info["type"]
        workspace_path = self.prepare_workspace(agent_type, session_id)
        
        # Update pending agent to mark workspace as created
        session_dir = self.sessions_dir / session_id
        pending_file = session_dir / "pending_agents.json"
        
        if pending_file.exists():
            try:
                with open(pending_file) as f:
                    pending_agents = json.load(f)
                
                if agent_id in pending_agents:
                    pending_agents[agent_id]["workspace_created"] = True
                    pending_agents[agent_id]["workspace_path"] = workspace_path
                    
                    with open(pending_file, 'w') as f:
                        json.dump(pending_agents, f, indent=2)
            except (json.JSONDecodeError, FileNotFoundError):
                pass
        
        return workspace_path
    
    def prepare_workspace(self, agent_type: str, session_id: str) -> str:
        """Create git worktree for agent"""
        workspace = self.worktrees_dir / f"{agent_type}-{session_id}"
        branch = f"agent/session-{session_id}/{agent_type}"
        
        try:
            # Create worktree using our git wrapper
            result = run_git_command([
                "worktree", "add", "-b", branch, str(workspace)
            ])
            if result.returncode != 0:
                raise subprocess.CalledProcessError(result.returncode, result.args, result.stdout, result.stderr)
            
            # Track in session coordination
            self.register_agent(agent_type, session_id, str(workspace))
            
            return str(workspace)
            
        except subprocess.CalledProcessError as e:
            # If branch exists, try without -b flag
            try:
                result = run_git_command([
                    "worktree", "add", str(workspace), branch
                ])
                if result.returncode != 0:
                    raise subprocess.CalledProcessError(result.returncode, result.args)
                
                self.register_agent(agent_type, session_id, str(workspace))
                return str(workspace)
                
            except subprocess.CalledProcessError:
                # Fall back to unique naming
                timestamp = int(time.time())
                workspace = self.worktrees_dir / f"{agent_type}-{session_id}-{timestamp}"
                branch = f"agent/session-{session_id}/{agent_type}-{timestamp}"
                
                result = run_git_command([
                    "worktree", "add", "-b", branch, str(workspace)
                ])
                if result.returncode != 0:
                    raise subprocess.CalledProcessError(result.returncode, result.args)
                
                self.register_agent(agent_type, session_id, str(workspace))
                return str(workspace)
    
    def register_agent(self, agent_type: str, session_id: str, workspace: str):
        """Register agent in session coordination"""
        session_dir = self.sessions_dir / session_id
        agents_file = session_dir / "agents.json"
        
        # Load existing agents
        agents = []
        if agents_file.exists():
            try:
                with open(agents_file) as f:
                    agents = json.load(f)
            except (json.JSONDecodeError, FileNotFoundError):
                agents = []
        
        # Generate agent ID
        agent_id = f"{agent_type}-{session_id}-{int(time.time())}"
        
        # Add new agent
        agent_data = {
            "id": agent_id,
            "type": agent_type,
            "session": session_id,
            "workspace": workspace,
            "created": datetime.now().isoformat(),
            "status": "active"
        }
        
        agents.append(agent_data)
        
        # Save updated agents list
        with open(agents_file, 'w') as f:
            json.dump(agents, f, indent=2)
    
    def check_file_lock(self, file_path: str, session_id: str) -> Optional[Dict]:
        """Check if file is locked by another agent"""
        session_dir = self.sessions_dir / session_id
        locks_file = session_dir / "locks.json"
        
        if locks_file.exists():
            try:
                with open(locks_file) as f:
                    locks = json.load(f)
                    return locks.get(file_path)
            except (json.JSONDecodeError, FileNotFoundError):
                pass
        
        return None
    
    def update_file_lock(self, file_path: str, agent_id: str, session_id: str, operation: str):
        """Update file lock registry"""
        session_dir = self.sessions_dir / session_id
        locks_file = session_dir / "locks.json"
        
        # Load existing locks
        locks = {}
        if locks_file.exists():
            try:
                with open(locks_file) as f:
                    locks = json.load(f)
            except (json.JSONDecodeError, FileNotFoundError):
                locks = {}
        
        # Update lock
        locks[file_path] = {
            "agent": agent_id,
            "operation": operation,
            "locked_at": datetime.now().isoformat()
        }
        
        # Save locks
        with open(locks_file, 'w') as f:
            json.dump(locks, f, indent=2)
    
    def release_file_lock(self, file_path: str, session_id: str):
        """Release file lock"""
        session_dir = self.sessions_dir / session_id
        locks_file = session_dir / "locks.json"
        
        if locks_file.exists():
            try:
                with open(locks_file) as f:
                    locks = json.load(f)
                
                if file_path in locks:
                    del locks[file_path]
                    
                    with open(locks_file, 'w') as f:
                        json.dump(locks, f, indent=2)
            except (json.JSONDecodeError, FileNotFoundError):
                pass
    
    def update_progress(self, agent_type: str, session_id: str, operation: str, details: Optional[Dict] = None):
        """Update agent progress tracking"""
        session_dir = self.sessions_dir / session_id
        progress_file = session_dir / "progress.json"
        
        # Load existing progress
        progress = {}
        if progress_file.exists():
            try:
                with open(progress_file) as f:
                    progress = json.load(f)
            except (json.JSONDecodeError, FileNotFoundError):
                progress = {}
        
        # Update progress for agent
        if agent_type not in progress:
            progress[agent_type] = []
        
        progress[agent_type].append({
            "operation": operation,
            "timestamp": datetime.now().isoformat(),
            "details": details or {}
        })
        
        # Save progress
        with open(progress_file, 'w') as f:
            json.dump(progress, f, indent=2)
    
    def cleanup_completed_worktrees(self):
        """Remove completed worktrees that have no uncommitted changes"""
        if not self.worktrees_dir.exists():
            return
        
        for worktree in self.worktrees_dir.iterdir():
            if worktree.is_dir():
                try:
                    # Check if worktree has uncommitted changes
                    result = run_git_command(
                        ["diff", "--quiet"], cwd=worktree
                    )
                    
                    # Also check staged changes
                    staged_result = run_git_command(
                        ["diff", "--cached", "--quiet"], cwd=worktree
                    )
                    
                    # If no uncommitted changes, safe to remove
                    if result.returncode == 0 and staged_result.returncode == 0:
                        run_git_command(["worktree", "remove", str(worktree)])
                        
                except subprocess.CalledProcessError:
                    # Skip worktrees we can't check
                    continue
    
    def get_session_status(self, session_id: str) -> Optional[Dict]:
        """Get current session status and agent information"""
        session_dir = self.sessions_dir / session_id
        if not session_dir.exists():
            return None
        
        status = {}
        
        # Load session data
        session_file = session_dir / "session.json"
        if session_file.exists():
            try:
                with open(session_file) as f:
                    status['session'] = json.load(f)
            except (json.JSONDecodeError, FileNotFoundError):
                status['session'] = {}
        
        # Load agents
        agents_file = session_dir / "agents.json"
        if agents_file.exists():
            try:
                with open(agents_file) as f:
                    status['agents'] = json.load(f)
            except (json.JSONDecodeError, FileNotFoundError):
                status['agents'] = []
        
        # Load locks
        locks_file = session_dir / "locks.json"
        if locks_file.exists():
            try:
                with open(locks_file) as f:
                    status['locks'] = json.load(f)
            except (json.JSONDecodeError, FileNotFoundError):
                status['locks'] = {}
        
        # Load progress
        progress_file = session_dir / "progress.json"
        if progress_file.exists():
            try:
                with open(progress_file) as f:
                    status['progress'] = json.load(f)
            except (json.JSONDecodeError, FileNotFoundError):
                status['progress'] = {}
        
        return status


# Utility functions for hooks
def get_backend() -> MAOSBackend:
    """Get MAOS backend instance"""
    return MAOSBackend()


def extract_file_path_from_tool_input(tool_input: Dict) -> Optional[str]:
    """Extract file path from various tool inputs"""
    # Direct file_path parameter
    if 'file_path' in tool_input:
        return tool_input['file_path']
    
    # MultiEdit edits (take first one)
    if 'edits' in tool_input and tool_input['edits']:
        return tool_input.get('file_path')
    
    return None


def extract_agent_id_from_environment() -> str:
    """Extract agent ID from environment or generate default"""
    # Try various environment variables that might contain agent ID
    agent_id = os.environ.get('CLAUDE_AGENT_ID')
    if agent_id:
        return agent_id
    
    # Fall back to process ID
    return f"agent-{os.getpid()}"


if __name__ == '__main__':
    # CLI for testing backend utilities
    import sys
    
    if len(sys.argv) < 2:
        print("Usage: python maos_backend.py <command> [args...]")
        print("Commands:")
        print("  status [session_id]  - Show session status")
        print("  cleanup              - Clean up completed worktrees")
        print("  test-workspace <agent_type> - Test workspace creation")
        sys.exit(1)
    
    backend = MAOSBackend()
    command = sys.argv[1]
    
    if command == "status":
        session_id = sys.argv[2] if len(sys.argv) > 2 else None
        if not session_id:
            # Get active session
            active_file = MAOS_DIR / "active_session.json"
            if active_file.exists():
                with open(active_file) as f:
                    session_id = json.load(f).get('session_id')
        
        if session_id:
            status = backend.get_session_status(session_id)
            if status:
                print(json.dumps(status, indent=2))
            else:
                print(f"Session {session_id} not found")
        else:
            print("No active session found")
    
    elif command == "cleanup":
        backend.cleanup_completed_worktrees()
        print("Cleanup completed")
    
    elif command == "test-workspace":
        if len(sys.argv) < 3:
            print("Usage: python maos_backend.py test-workspace <agent_type>")
            sys.exit(1)
        
        agent_type = sys.argv[2]
        session_id = backend.get_or_create_session()
        workspace = backend.prepare_workspace(agent_type, session_id)
        print(f"Created workspace: {workspace}")
        print(f"Session: {session_id}")
    
    else:
        print(f"Unknown command: {command}")
        sys.exit(1)
