#!/usr/bin/env python3
"""
MAOS Agent Launcher with Claude CLI Orchestration.
This script handles intelligent agent routing via persistent orchestrator session
and direct Claude CLI execution with streaming output.
"""

import sys
import json
import logging
import time
from pathlib import Path
from datetime import datetime
from typing import Dict, Optional, Any, Tuple

# Import orchestration modules
from claude_orchestrator import ClaudeOrchestrator
from claude_agent import ClaudeAgent
from session_manager import SessionManager, SessionType
from security_utils import safe_path_join, validate_agent_name

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def load_or_create_manifest(session_path: Path) -> Dict[str, Any]:
    """Load existing manifest or create new one."""
    manifest_path = safe_path_join(session_path, "manifest.json")
    
    if manifest_path.exists():
        try:
            with open(manifest_path, 'r') as f:
                return json.load(f)
        except Exception as e:
            logger.error(f"Failed to load manifest: {e}")
    
    # Create new manifest
    return {
        "session_id": session_path.name,
        "created_at": datetime.now().isoformat(),
        "orchestrator": {},
        "agents": {}
    }


def save_manifest(session_path: Path, manifest: Dict[str, Any]) -> None:
    """Save manifest to disk."""
    manifest_path = safe_path_join(session_path, "manifest.json")
    try:
        manifest_path.parent.mkdir(parents=True, exist_ok=True)
        with open(manifest_path, 'w') as f:
            json.dump(manifest, f, indent=2)
    except Exception as e:
        logger.error(f"Failed to save manifest: {e}")


def get_next_instance_number(agent_name: str, manifest: Dict[str, Any]) -> int:
    """Get the next available instance number for an agent role."""
    agents = manifest.get('agents', {})
    existing_numbers = []
    
    for instance_name in agents.keys():
        if instance_name.startswith(f"{agent_name}-"):
            try:
                num = int(instance_name.split('-')[-1])
                existing_numbers.append(num)
            except ValueError:
                pass
    
    return max(existing_numbers) + 1 if existing_numbers else 1


def launch_agent(agent_name: str, task: str):
    """Launch an agent using Claude CLI with intelligent orchestration."""
    
    # Validate agent name
    if not validate_agent_name(agent_name):
        logger.error(f"Invalid agent name: {agent_name}")
        raise ValueError(f"Invalid agent name: {agent_name}")
    
    # Get current working directory
    cwd = Path.cwd()
    
    # Get or create MAOS session
    manager = SessionManager()
    
    # Check for existing session
    orchestration_session_id = None
    current_session_file = cwd / ".maos" / ".current_orchestration"
    
    if current_session_file.exists():
        try:
            with open(current_session_file, 'r') as f:
                data = json.load(f)
                orchestration_session_id = data.get('session_id')
        except Exception as e:
            logger.warning(f"Failed to read orchestration marker: {e}")
    
    if orchestration_session_id:
        session_id = orchestration_session_id
        logger.info(f"Using orchestration session: {session_id}")
    else:
        # Create new session
        session = manager.create_session(
            task=task,
            session_type=SessionType.AGENT
        )
        session_id = session.session_id
        logger.info(f"Created new session: {session_id}")
    
    # Set up paths
    session_path = safe_path_join(cwd, ".maos", "sessions", session_id)
    project_dir = cwd  # Agents work in project root
    shared_context = safe_path_join(session_path, "shared")
    shared_context.mkdir(parents=True, exist_ok=True)
    
    # Create messages directory for inter-agent communication
    messages_dir = safe_path_join(session_path, "messages")
    messages_dir.mkdir(parents=True, exist_ok=True)
    
    # Load manifest
    manifest = load_or_create_manifest(session_path)
    
    # Initialize orchestrator
    orchestrator = ClaudeOrchestrator(session_path)
    orchestrator.ensure_session()
    
    # Get routing decision from orchestrator
    print(f"\n🤔 Consulting orchestrator for {agent_name}...")
    decision = orchestrator.consult(agent_name, task, manifest)
    
    print(f"📋 Decision: {decision['decision']}")
    print(f"💡 Reasoning: {decision['reasoning']}")
    
    # Determine agent instance
    if decision['decision'] == 'reuse':
        agent_instance = decision['agent_instance']
        agent_data = manifest['agents'].get(agent_instance, {})
        agent_session_id = agent_data.get('session_id')
        print(f"♻️  Reusing existing agent: {agent_instance}")
    else:
        # Create new instance
        next_num = get_next_instance_number(agent_name, manifest)
        agent_instance = f"{agent_name}-{next_num}"
        agent_session_id = None
        print(f"🆕 Creating new agent: {agent_instance}")
    
    # Create agent workspace
    agent_workspace = safe_path_join(session_path, "agents", agent_instance)
    agent_workspace.mkdir(parents=True, exist_ok=True)
    
    # Initialize Claude agent executor
    claude_agent = ClaudeAgent(session_path)
    
    # Direct CLI execution with streaming output
    print(f"\n🚀 Executing {agent_instance}...")
    
    if agent_session_id:
        # Resume existing session
        result = claude_agent.resume(
            agent_instance=agent_instance,
            session_id=agent_session_id,
            task=task,
            workspace=agent_workspace,
            shared_context=shared_context,
            project_dir=project_dir
        )
    else:
        # Create new session
        result = claude_agent.spawn_new(
            agent_instance=agent_instance,
            role=agent_name,
            task=task,
            workspace=agent_workspace,
            shared_context=shared_context,
            project_dir=project_dir
        )
    
    if result['success']:
        print(f"✅ Agent completed successfully")
        if result.get('session_id'):
            print(f"📌 Session ID: {result['session_id']}")
        print(f"🛠️  Tools used: {result.get('tool_count', 0)}")
        print(f"⏱️  Duration: {result.get('duration', 0):.1f}s")
    else:
        print(f"❌ Agent failed: {result.get('error', 'Unknown error')}")
    
    # Output launch metadata
    metadata = {
        "agent_instance": agent_instance,
        "agent_role": agent_name,
        "session_id": session_id,
        "agent_session_id": result.get('session_id') if result.get('success') else agent_session_id,
        "task": task,
        "workspace": str(agent_workspace),
        "shared_context": str(shared_context),
        "project_dir": str(project_dir),
        "orchestrator_decision": decision,
        "result": {
            "success": result.get('success', False),
            "tool_count": result.get('tool_count', 0),
            "duration": result.get('duration', 0)
        }
    }
    
    return metadata


def main():
    if len(sys.argv) < 2:
        print("Usage: launch_agent.py <agent_name> [task]", file=sys.stderr)
        print("       If task is not provided, it will be read from stdin", file=sys.stderr)
        sys.exit(1)
    
    agent_name = sys.argv[1]
    
    # Get task from command line or stdin
    task = None
    if len(sys.argv) > 2:
        task = sys.argv[2]
    
    if not task:
        # Read task from stdin
        task = sys.stdin.read().strip()
        if not task:
            print("ERROR: No task provided via argument or stdin", file=sys.stderr)
            sys.exit(1)
    
    try:
        metadata = launch_agent(agent_name, task)
        
        # Output metadata as JSON for potential parsing
        print("\n" + "="*60)
        print("Launch Metadata:")
        print(json.dumps(metadata, indent=2))
        
    except Exception as e:
        logger.error(f"Failed to launch agent: {e}", exc_info=True)
        print(f"ERROR: Failed to launch agent: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()