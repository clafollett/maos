#!/usr/bin/env python3
"""
Utility script to launch MAOS agents with processed templates.
This script handles template processing and workspace setup for agent commands.
"""

import sys
import json
import logging
from pathlib import Path
from template_processor import process_agent_template
from session_manager import SessionManager, SessionType
from security_utils import safe_path_join, validate_agent_name

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def launch_agent(agent_name: str, task: str):
    """Launch an agent with a processed template and prepared workspace."""
    
    # Validate agent name
    if not validate_agent_name(agent_name):
        logger.error(f"Invalid agent name: {agent_name}")
        raise ValueError(f"Invalid agent name: {agent_name}")
    
    # Get current working directory (where the command is run from)
    # This should be the project root when run via slash commands
    cwd = Path.cwd()
    
    # Check if we're part of an orchestration session
    manager = SessionManager()
    
    # Check for orchestration session via environment variable or marker file
    orchestration_session_id = None
    current_session_file = cwd / ".maos" / ".current_orchestration"
    
    if current_session_file.exists():
        try:
            with open(current_session_file, 'r') as f:
                data = json.load(f)
                orchestration_session_id = data.get('session_id')
        except (json.JSONDecodeError, OSError) as e:
            logger.warning(f"Failed to read orchestration marker: {e}")
    
    if orchestration_session_id:
        # We're part of an orchestration, use that session
        session_id = orchestration_session_id
        print(f"Using orchestration session: {session_id[:8]}...")
        # Register this agent with the orchestration session
        manager.add_agent_to_session(session_id, agent_name)
    else:
        # Standalone agent execution
        session = manager.create_session(
            task=task,
            session_type=SessionType.AGENT
        )
        session_id = session.session_id
        print(f"Created standalone session: {session_id[:8]}...")
    
    # Process the template
    template_path = f"assets/agent-roles/{agent_name}.md"
    
    try:
        processed_template = process_agent_template(
            agent_name=agent_name,
            session_id=session_id,
            task=task,
            template_path=template_path,
            cwd=str(cwd)
        )
    except Exception as e:
        logger.error(f"Failed to process template: {e}", exc_info=True)
        print(f"ERROR: Failed to process template: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Create workspace directories relative to CWD using safe path joining
    try:
        workspace_path = safe_path_join(cwd, ".maos", "sessions", session_id, "agents", agent_name)
        workspace_path.mkdir(parents=True, exist_ok=True)
        
        shared_path = safe_path_join(cwd, ".maos", "sessions", session_id, "shared")
        shared_path.mkdir(parents=True, exist_ok=True)
    except (ValueError, OSError) as e:
        logger.error(f"Failed to create workspace directories: {e}")
        print(f"ERROR: Failed to create workspace directories: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Save processed template
    template_file = workspace_path / "agent_template.md"
    try:
        with open(template_file, 'w') as f:
            f.write(processed_template)
    except OSError as e:
        logger.error(f"Failed to save processed template: {e}")
        print(f"ERROR: Failed to save processed template: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Save launch metadata
    metadata = {
        "agent_name": agent_name,
        "session_id": session_id,
        "task": task,
        "cwd": str(cwd),
        "workspace_path": str(workspace_path),
        "shared_path": str(shared_path),
        "template_file": str(template_file)
    }
    
    metadata_file = workspace_path / "launch_metadata.json"
    try:
        with open(metadata_file, 'w') as f:
            json.dump(metadata, f, indent=2)
    except OSError as e:
        logger.error(f"Failed to save launch metadata: {e}")
    
    # Output the launch information
    print(json.dumps(metadata, indent=2))
    
    # Don't return metadata as the return value is never used


def main():
    if len(sys.argv) < 2:
        print("Usage: launch_agent.py <agent_name> [task]", file=sys.stderr)
        print("       If task is not provided, it will be read from stdin", file=sys.stderr)
        sys.exit(1)
    
    agent_name = sys.argv[1]
    
    # Get task from command line or stdin
    if len(sys.argv) >= 3:
        # Task provided as argument (for simple cases)
        task = sys.argv[2]
    else:
        # Read task from stdin (safer for complex input)
        task = sys.stdin.read().strip()
        if not task:
            print("ERROR: No task provided via argument or stdin", file=sys.stderr)
            sys.exit(1)
    
    try:
        launch_agent(agent_name, task)
    except Exception as e:
        logger.error(f"Failed to launch agent: {e}", exc_info=True)
        print(f"ERROR: Failed to launch agent: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()