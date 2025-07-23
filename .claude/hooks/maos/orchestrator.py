#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "python-dotenv",
# ]
# ///

import json
import sys
import os
import logging
from pathlib import Path
from datetime import datetime
# Type hints available if needed in future

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('.maos/orchestration_errors.log'),
        logging.StreamHandler(sys.stderr)
    ]
)
logger = logging.getLogger(__name__)

# Add current directory to path for local imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from session_manager import SessionManager, SessionType
from security_utils import sanitize_path_component

try:
    from dotenv import load_dotenv
    load_dotenv()
except ImportError:
    pass


# Security utilities are imported from security_utils module


class MAOSOrchestrator:
    """MAOS Orchestration handler - spawns an intelligent orchestrator agent."""
    
    # Only trigger on explicit orchestrator command
    ORCHESTRATION_COMMAND = '/maos:orchestrator'
    
    def __init__(self, session_id: str, prompt: str, task: str = None):
        # Sanitize session_id to prevent path traversal
        self.session_id = sanitize_path_component(session_id)
        self.prompt = prompt
        self.task = task or self.extract_task()
        
    def is_orchestration_request(self) -> bool:
        """Check if this is an explicit orchestration request."""
        return self.prompt.startswith(self.ORCHESTRATION_COMMAND)
        
    def extract_task(self) -> str:
        """Extract the actual task from the prompt."""
        if self.prompt.startswith(self.ORCHESTRATION_COMMAND):
            return self.prompt[len(self.ORCHESTRATION_COMMAND):].strip()
        return self.prompt
        
    def generate_orchestration_prompt(self) -> str:
        """Generate the prompt for the orchestrator agent to analyze and plan."""
        task = self.extract_task()
        
        return f"""## MAOS Orchestration Planning

You are the MAOS Orchestrator Agent. Your job is to analyze this task and determine which specialized agents are needed and how to coordinate them.

**Task to orchestrate**: {task}

**Available Agents**:
- api-architect: API design and architecture
- application-architect: Application-level architecture  
- backend-engineer: Backend engineering and server-side development
- business-analyst: Data analysis and business insights
- data-architect: Data architecture and design
- data-scientist: Machine learning and data science
- devops: Infrastructure and deployment
- frontend-engineer: Frontend engineering and UI development
- mobile-engineer: Mobile application development
- pm: Product management
- qa: Quality assurance and testing
- researcher: Technology research and analysis
- reviewer: Code review and quality assessment
- secops: Security operations and compliance
- security-architect: Security architecture design
- solution-architect: Solution architecture and high-level design
- techwriter: Technical documentation
- tester: Test implementation and execution
- ux-designer: User experience design

**Your Analysis Should Include**:
1. Break down the task into specific subtasks
2. Identify which agents are needed for each subtask
3. Determine the optimal order of execution (sequential vs parallel)
4. Define what each agent should deliver
5. Identify dependencies between agents

**Use the Task tool to spawn each required agent with**:
- Their specific subtask
- Clear deliverables expected
- Any dependencies on other agents' outputs

Session workspace: .maos/sessions/{self.session_id}/

Begin by analyzing the task and creating an execution plan."""
        
    def create_session_workspace(self):
        """Create session workspace directory."""
        workspace = Path(f".maos/sessions/{self.session_id}")
        workspace.mkdir(parents=True, exist_ok=True)
        return workspace


def append_log_entry(log_file: Path, entry: dict):
    """Efficiently append to JSONL log file."""
    log_file.parent.mkdir(parents=True, exist_ok=True)
    with open(log_file, 'a', encoding='utf-8') as f:
        # Use JSONEncoder for consistent formatting
        json.dump(entry, f, ensure_ascii=False, separators=(',', ':'))
        f.write('\n')


def main():
    try:
        # Read input from stdin
        input_data = json.loads(sys.stdin.read())
        session_id = input_data.get('session_id', 'unknown')
        prompt = input_data.get('prompt', '')
        
        # Validate input
        if not session_id or not prompt:
            logger.error("Missing required input: session_id or prompt")
            sys.exit(1)
        
        # Always save the current Claude session ID for slash commands to use
        session_file = Path('.maos/.current_claude_session')
        session_file.parent.mkdir(parents=True, exist_ok=True)
        with open(session_file, 'w') as f:
            json.dump({
                'session_id': session_id,
                'prompt': prompt,
                'timestamp': datetime.now().isoformat()
            }, f)
        
        # Initialize orchestrator
        orchestrator = MAOSOrchestrator(session_id, prompt)
        
        # Check if this is an orchestration request
        if orchestrator.is_orchestration_request():
            # Extract the task
            task = orchestrator.extract_task()
            
            # Use Claude's session ID directly for MAOS
            manager = SessionManager()
            
            # Check if we already have a session for this Claude session
            existing_sessions = manager.get_sessions_by_claude_id(session_id)
            
            if not existing_sessions:
                # Create new session using Claude's session ID
                session = manager.create_session(
                    task=task,
                    session_type=SessionType.ORCHESTRATION,
                    claude_session_id=session_id
                )
            else:
                # Use existing session
                session = existing_sessions[0]
                print(f"📎 Continuing with existing session for this Claude conversation")
            
            # Update orchestrator with the session ID
            orchestrator.session_id = session.session_id
            orchestrator.task = task
            
            # Create session workspace
            orchestrator.create_session_workspace()
            
            # Create orchestration marker for agents
            marker_file = Path('.maos/.current_orchestration')
            marker_file.parent.mkdir(parents=True, exist_ok=True)
            with open(marker_file, 'w') as f:
                json.dump({'session_id': session.session_id}, f)
            
            # Instead of trying to parse and guess agents,
            # inject a prompt that tells Claude to use the Task tool
            # to spawn an orchestrator agent that will figure it out
            
            orchestration_context = orchestrator.generate_orchestration_prompt()
            
            print(orchestration_context)
            print("\n---\n")
            print("Use the Task tool to begin orchestration planning and execution.")
            
            # Log orchestration activation using JSONL format
            log_data = {
                'timestamp': datetime.now().isoformat(),
                'session_id': session_id,  # Using Claude session ID directly
                'orchestration_activated': True,
                'task': task,
                'method': 'intelligent_orchestration'
            }
            
            log_file = Path('.maos/orchestration.jsonl')
            append_log_entry(log_file, log_data)
        
        # Always exit successfully
        sys.exit(0)
        
    except json.JSONDecodeError as e:
        logger.error(f"Failed to parse input JSON: {e}", exc_info=True)
        sys.exit(1)
    except Exception as e:
        logger.error(f"Failed to process orchestration: {e}", exc_info=True)
        sys.exit(1)


if __name__ == '__main__':
    main()