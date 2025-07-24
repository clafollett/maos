#!/usr/bin/env python3
"""Direct agent launcher that bypasses orchestrator for MCP."""

import subprocess
import sys
import json
from pathlib import Path
from datetime import datetime
import uuid

def launch_agent_direct(role: str, task: str) -> dict:
    """Launch an agent directly without orchestrator consultation."""
    
    # Create a simple session
    session_id = str(uuid.uuid4())
    session_path = Path(".maos/sessions") / session_id
    session_path.mkdir(parents=True, exist_ok=True)
    
    # Create basic structure
    (session_path / "agents").mkdir(exist_ok=True)
    (session_path / "shared").mkdir(exist_ok=True)
    (session_path / "messages").mkdir(exist_ok=True)
    (session_path / "logs").mkdir(exist_ok=True)
    
    # Create agent instance
    agent_instance = f"{role}-1"
    agent_workspace = session_path / "agents" / agent_instance
    agent_workspace.mkdir(exist_ok=True)
    
    # Log session creation
    with open(session_path / "logs" / "events.jsonl", "w") as f:
        event = {
            "timestamp": datetime.now().isoformat(),
            "event": "session_created",
            "data": {"task": task, "type": "agent"}
        }
        f.write(json.dumps(event) + "\n")
    
    # Build Claude command directly
    cmd = [
        "claude",
        "--model", "sonnet",
        "-p", f"You are a {role} agent. Task: {task}",
        "--dangerously-skip-permissions"
    ]
    
    print(f"Launching: {' '.join(cmd)}")
    
    # Start the process
    try:
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        return {
            "success": True,
            "session_id": session_id,
            "agent_instance": agent_instance,
            "pid": process.pid,
            "workspace": str(agent_workspace)
        }
        
    except Exception as e:
        return {
            "success": False,
            "error": str(e)
        }

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: direct_agent_launch.py <role> <task>")
        sys.exit(1)
        
    result = launch_agent_direct(sys.argv[1], sys.argv[2])
    print(json.dumps(result, indent=2))