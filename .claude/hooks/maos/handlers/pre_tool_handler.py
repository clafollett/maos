#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# ///

import json
import sys
import os
from pathlib import Path
from typing import Dict, Optional

# Add path resolution for proper imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
# Import MAOS backend utilities
try:
    from maos.backend import MAOSBackend, extract_file_path_from_tool_input, extract_agent_id_from_environment, PROJECT_ROOT
except ImportError:
    # Fallback if backend not available
    MAOSBackend = None
    PROJECT_ROOT = Path.cwd()


class MAOSCoordinator:
    """MAOS coordination layer for Claude Code sub-agents"""
    
    def __init__(self, hook_metadata=None):
        self.hook_metadata = hook_metadata or {}
        self.backend = MAOSBackend() if MAOSBackend else None
        self._session_id = None
    
    def get_session_id(self):
        """Get or create session ID"""
        if not self.backend:
            return "fallback-session"
        
        if not self._session_id:
            self._session_id = self.backend.get_or_create_session(self.hook_metadata)
        return self._session_id
    
    def handle_subagent_spawning(self, tool_input):
        """Handle Task tool for sub-agent spawning with lazy workspace creation"""
        if not self.backend:
            print("MAOS backend not available, skipping orchestration", file=sys.stderr)
            return
        
        agent_type = tool_input.get('subagent_type')
        if not agent_type:
            return
        
        try:
            session_id = self.get_session_id()
            
            # Register agent for lazy workspace creation (don't create worktree yet)
            agent_id = self.backend.register_pending_agent(agent_type, session_id)
            
            # Modify the prompt to include conditional workspace instruction
            original_prompt = tool_input.get('prompt', '')
            workspace_path = f"{PROJECT_ROOT}/worktrees/{agent_type}-{session_id}"
            
            workspace_instruction = f"""

WORKSPACE MANAGEMENT:
Your workspace will be created automatically when you first perform file operations.
When using Read, Write, Edit, or MultiEdit tools, the system will:
1. Create your isolated workspace at: {workspace_path}/
2. All subsequent file operations must use paths within this workspace
3. Use absolute paths starting with your workspace directory

Note: The workspace is created on-demand to save resources. Don't manually create directories - let the system handle workspace setup when you first need it."""
            
            tool_input['prompt'] = original_prompt + workspace_instruction
            
            # Set agent type in environment for later identification
            os.environ['CLAUDE_AGENT_TYPE'] = agent_type
            
            print(f"🚀 MAOS: Registered {agent_type} for lazy workspace creation", file=sys.stderr)
            print(f"📋 Session: {session_id}, Agent ID: {agent_id}", file=sys.stderr)
            
        except Exception as e:
            # Don't block the operation, just log the error
            print(f"⚠️  MAOS agent registration failed: {e}", file=sys.stderr)
            print("Continuing without workspace isolation", file=sys.stderr)
    
    def handle_file_operations(self, tool_name, tool_input):
        """Handle file operation with lazy workspace creation"""
        if not self.backend:
            return
        
        file_path = extract_file_path_from_tool_input(tool_input)
        if not file_path:
            return
        
        try:
            session_id = self.get_session_id()
            agent_id = extract_agent_id_from_environment()
            
            # Check if this agent needs a workspace created
            # Try to find agent info by checking pending agents
            session_dir = Path(PROJECT_ROOT) / ".maos" / "sessions" / session_id
            pending_file = session_dir / "pending_agents.json"
            
            if pending_file.exists():
                # Look for matching pending agent
                pending_agents = json.load(open(pending_file))
                for pid, agent_info in pending_agents.items():
                    # Check if this is our agent (match by type in environment)
                    agent_type_env = os.environ.get('CLAUDE_AGENT_TYPE', '')
                    if agent_type_env and agent_info.get('type') == agent_type_env:
                        # Create workspace if needed
                        if not agent_info.get('workspace_created'):
                            workspace_path = self.backend.create_workspace_if_needed(pid, session_id)
                            if workspace_path:
                                print(f"🏗️  MAOS: Created workspace for {agent_type_env} at {workspace_path}", file=sys.stderr)
                                # Enforce workspace usage
                                self.enforce_workspace_path(tool_name, file_path, workspace_path)
                        elif agent_info.get('workspace_path'):
                            # Workspace exists, enforce its usage
                            self.enforce_workspace_path(tool_name, file_path, agent_info['workspace_path'])
                        break
            
            # Check for existing lock
            lock_info = self.backend.check_file_lock(file_path, session_id)
            if lock_info and lock_info.get('agent') != agent_id:
                print(f"⚠️  File {file_path} is being edited by {lock_info.get('agent', 'another agent')}", 
                      file=sys.stderr)
                print(f"Operation: {lock_info.get('operation', 'unknown')}", file=sys.stderr)
            
            # Update lock for this operation
            if tool_name in ["Edit", "Write", "MultiEdit"]:
                self.backend.update_file_lock(file_path, agent_id, session_id, tool_name)
            
        except Exception as e:
            # Non-blocking error
            print(f"⚠️  MAOS file operation handling failed: {e}", file=sys.stderr)
    
    def update_progress(self, tool_name, tool_input):
        """Update progress tracking"""
        if not self.backend:
            return
        
        try:
            session_id = self.get_session_id()
            agent_id = extract_agent_id_from_environment()
            
            # Extract relevant details for progress tracking
            details = {}
            if tool_name in ["Edit", "Write", "MultiEdit"]:
                file_path = extract_file_path_from_tool_input(tool_input)
                if file_path:
                    details['file_path'] = file_path
            elif tool_name == "Bash":
                command = tool_input.get('command', '')[:100]  # Truncate long commands
                details['command'] = command
            
            self.backend.update_progress(agent_id, session_id, tool_name, details)
            
        except Exception as e:
            # Non-blocking error
            pass
    
    def enforce_workspace_path(self, tool_name, file_path, workspace_path):
        """Enforce that file operations use the assigned workspace"""
        # Convert to Path objects for comparison
        file_path_obj = Path(file_path)
        workspace_path_obj = Path(workspace_path)
        
        # Check if file path is absolute and outside workspace
        if file_path_obj.is_absolute():
            try:
                # Resolve paths for accurate comparison
                file_resolved = file_path_obj.resolve()
                workspace_resolved = workspace_path_obj.resolve()
                
                if not str(file_resolved).startswith(str(workspace_resolved)):
                    # Block the operation
                    print(f"\n❌ BLOCKED: File operations must use assigned workspace", file=sys.stderr)
                    print(f"   Attempted: {file_path}", file=sys.stderr)
                    print(f"   ✅ Use instead: {workspace_path}/{file_path_obj.name}", file=sys.stderr)
                    print(f"\n   Your workspace: {workspace_path}/", file=sys.stderr)
                    print(f"   All file operations MUST use paths within this directory\n", file=sys.stderr)
                    sys.exit(2)  # Exit code 2 blocks the operation
            except Exception:
                # If we can't resolve paths, be conservative and block
                if not file_path.startswith(workspace_path):
                    print(f"\n❌ BLOCKED: File operations must use assigned workspace", file=sys.stderr)
                    print(f"   Your workspace: {workspace_path}/\n", file=sys.stderr)
                    sys.exit(2)


def handle_maos_pre_tool(tool_name: str, tool_input: Dict, hook_metadata: Optional[Dict] = None):
    """Main MAOS pre-tool processing function"""
    # Initialize MAOS coordinator
    coordinator = MAOSCoordinator(hook_metadata)
    
    # Handle sub-agent spawning through Task tool
    if tool_name == "Task" and tool_input.get('subagent_type'):
        coordinator.handle_subagent_spawning(tool_input)
    
    # Handle file operations for conflict detection
    elif tool_name in ["Edit", "Write", "MultiEdit", "Read"]:
        coordinator.handle_file_operations(tool_name, tool_input)
    
    # Update progress tracking
    coordinator.update_progress(tool_name, tool_input)