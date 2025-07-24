#!/usr/bin/env python3
"""
Simple wrapper to launch agents synchronously for MCP server.
This avoids async/sync issues by running agents in a separate process.
"""

import sys
import json
import subprocess
from pathlib import Path

def launch_agent_subprocess(role: str, task: str) -> dict:
    """Launch an agent in a subprocess and return immediately with session info."""
    
    # Build the command
    cmd = [
        sys.executable,
        ".claude/hooks/maos/launch_agent.py",
        role,
        task
    ]
    
    try:
        # Start the process but don't wait for it to complete
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            cwd=Path.cwd()  # Ensure we're in project root
        )
        
        # Get the PID for tracking
        pid = process.pid
        
        # Wait briefly to get initial output (session creation)
        import time
        time.sleep(2)  # Give it 2 seconds to create session
        
        # Check if session was created
        from session_manager import SessionManager
        manager = SessionManager()
        sessions = manager.list_sessions()
        
        if sessions:
            # Get the most recent session
            latest = max(sessions, key=lambda s: s.created)
            return {
                "success": True,
                "pid": pid,
                "session_id": latest.session_id,
                "status": "started",
                "message": f"Agent {role} launched (PID: {pid})"
            }
        else:
            return {
                "success": False,
                "pid": pid,
                "error": "No session created",
                "message": "Agent may still be starting"
            }
            
    except Exception as e:
        return {
            "success": False,
            "error": str(e),
            "message": f"Failed to launch agent: {e}"
        }


if __name__ == "__main__":
    # Test mode
    if len(sys.argv) < 3:
        print("Usage: mcp_agent_wrapper.py <role> <task>")
        sys.exit(1)
        
    role = sys.argv[1]
    task = sys.argv[2]
    
    result = launch_agent_subprocess(role, task)
    print(json.dumps(result, indent=2))