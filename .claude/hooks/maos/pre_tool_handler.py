#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# ///

import json
import sys
import os
from pathlib import Path
from typing import Dict, Optional

# Import MAOS backend utilities
sys.path.append(str(Path(__file__).parent))
try:
    from backend import MAOSBackend, extract_file_path_from_tool_input, extract_agent_id_from_environment, PROJECT_ROOT
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
        """Handle Task tool for sub-agent spawning"""
        if not self.backend:
            print("MAOS backend not available, skipping orchestration", file=sys.stderr)
            return
        
        agent_type = tool_input.get('subagent_type')
        if not agent_type:
            return
        
        try:
            session_id = self.get_session_id()
            workspace = self.backend.prepare_workspace(agent_type, session_id)
            
            # Modify the prompt to include workspace path - use absolute path
            original_prompt = tool_input.get('prompt', '')
            workspace_abs = Path(workspace).resolve()
            workspace_instruction = f"\n\nIMPORTANT: Work exclusively in the workspace: {workspace_abs}/\n" \
                                  f"All file operations (Read, Write, Edit, MultiEdit) must use absolute paths starting with: {workspace_abs}/\n" \
                                  f"Never use relative paths or change directories - always use: {workspace_abs}/[filename]\n"
            
            tool_input['prompt'] = original_prompt + workspace_instruction
            
            print(f"🚀 MAOS: Created isolated workspace for {agent_type}: {workspace}", file=sys.stderr)
            print(f"📋 Session: {session_id}", file=sys.stderr)
            
        except Exception as e:
            # Don't block the operation, just log the error
            print(f"⚠️  MAOS workspace creation failed: {e}", file=sys.stderr)
            print("Continuing without workspace isolation", file=sys.stderr)
    
    def handle_file_operations(self, tool_name, tool_input):
        """Handle file operation conflict checking"""
        if not self.backend:
            return
        
        file_path = extract_file_path_from_tool_input(tool_input)
        if not file_path:
            return
        
        try:
            session_id = self.get_session_id()
            agent_id = extract_agent_id_from_environment()
            
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
            print(f"⚠️  MAOS lock check failed: {e}", file=sys.stderr)
    
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