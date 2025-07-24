#!/usr/bin/env python3
"""
Claude CLI Agent Executor - Manages agent execution via Claude CLI.
This module handles spawning new agent sessions, resuming existing sessions,
and managing the streaming output from Claude CLI processes.
"""

import json
import subprocess
import logging
import asyncio
import time
from pathlib import Path
from typing import Dict, Optional, Any, List, Callable
from datetime import datetime
import threading
from queue import Queue

from security_utils import safe_path_join

logger = logging.getLogger(__name__)


class StreamingOutput:
    """Handles streaming JSON output from Claude CLI."""
    
    def __init__(self, process: subprocess.Popen, agent_instance: str):
        self.process = process
        self.agent_instance = agent_instance
        self.session_id = None
        self.full_response = []
        self.tool_count = 0
        self.is_complete = False
        self.error = None
        
    def parse_line(self, line: str) -> Optional[Dict[str, Any]]:
        """Parse a single line of streaming JSON output."""
        if not line.strip():
            return None
            
        try:
            return json.loads(line)
        except json.JSONDecodeError:
            logger.debug(f"Non-JSON line: {line}")
            return None
    
    def process_event(self, event: Dict[str, Any]) -> None:
        """Process a streaming event from Claude."""
        event_type = event.get('type', '')
        
        if event_type == 'system':
            # Check for session initialization
            if event.get('subtype') == 'init':
                self.session_id = event.get('session_id')
                logger.info(f"[{self.agent_instance}] Session ID: {self.session_id}")
                
        elif event_type == 'assistant':
            # Extract content from assistant messages
            message = event.get('message', {})
            content = message.get('content', [])
            
            for item in content:
                if 'text' in item:
                    text = item['text']
                    self.full_response.append(text)
                    
                    # Log interesting content
                    if any(keyword in text for keyword in ['Creating', 'Writing', 'Generated', 'File:']):
                        preview = text[:80] + '...' if len(text) > 80 else text
                        logger.info(f"[{self.agent_instance}] > {preview.strip()}")
                        
                elif 'name' in item:
                    # Tool use
                    tool_name = item.get('name', 'unknown')
                    self.tool_count += 1
                    logger.info(f"[{self.agent_instance}] Using tool: {tool_name} (#{self.tool_count})")
                    # Print to stdout for real-time visibility
                    print(f"  🔧 Using tool: {tool_name}")
                    
        elif event_type == 'result':
            # Check for completion
            if event.get('subtype') == 'success':
                self.is_complete = True
                logger.info(f"[{self.agent_instance}] Completed successfully")
            elif event.get('subtype') == 'error':
                self.error = event.get('error', 'Unknown error')
                logger.error(f"[{self.agent_instance}] Error: {self.error}")


class ClaudeAgent:
    """Executes agents via Claude CLI."""
    
    def __init__(self, session_path: Path):
        """Initialize agent executor with session path."""
        self.session_path = session_path
        self.manifest_path = safe_path_join(session_path, "manifest.json")
        
    def spawn_new(self, agent_instance: str, role: str, task: str, 
                  workspace: Path, shared_context: Path,
                  project_dir: Path, model: str = "sonnet") -> Dict[str, Any]:
        """Create new agent session and execute task."""
        
        logger.info(f"Spawning new agent: {agent_instance}")
        
        # Build agent prompt
        prompt = self._build_agent_prompt(role, task, workspace, shared_context, project_dir)
        
        # Prepare Claude CLI command
        cmd = [
            'claude',
            '--model', model,
            '-p', prompt,
            '--output-format', 'stream-json',
            '--verbose',  # Claude CLI requires this with stream-json
            '--add-dir', str(project_dir),      # Main project directory
            '--add-dir', str(shared_context),   # Shared context
            '--add-dir', str(workspace),        # Agent's private workspace
            '--dangerously-skip-permissions'
        ]
        
        # Add messages directory if it exists
        messages_dir = workspace.parent.parent / "messages"
        if messages_dir.exists():
            cmd.extend(['--add-dir', str(messages_dir)])
        
        # Add fallback for certain roles
        if role in ['architect', 'researcher']:
            cmd.extend(['--fallback-model', 'sonnet'])
            
        # Log the command for debugging
        logger.info(f"Executing command: {' '.join(cmd)}")
        
        # Execute and stream output
        result = self._execute_streaming(cmd, agent_instance)
        
        if result['success']:
            # Update manifest with new agent
            self._update_agent_manifest(agent_instance, {
                'session_id': result['session_id'],
                'role': role,
                'status': 'available',
                'created_at': datetime.now().isoformat(),
                'workspace': str(workspace),
                'work_history': [task],
                'expertise_areas': self._infer_expertise(task),
                'last_active': datetime.now().isoformat()
            })
        
        return result
    
    def resume(self, agent_instance: str, session_id: str, task: str,
               workspace: Path, shared_context: Path, 
               project_dir: Path) -> Dict[str, Any]:
        """Resume existing agent session with new task."""
        
        logger.info(f"Resuming agent: {agent_instance} (session: {session_id})")
        
        # Build resume prompt
        prompt = f"""Continue your work with this new task:

{task}

Remember to:
1. Check your previous work in the project directory
2. Build upon what you've already created
3. Update shared context with your progress
4. Maintain consistency with your previous decisions
"""
        
        # Prepare resume command
        cmd = [
            'claude',
            '--resume', session_id,
            '-p', prompt,
            '--output-format', 'stream-json',
            '--add-dir', str(project_dir),      # Main project directory
            '--add-dir', str(shared_context),   # Shared context
            '--add-dir', str(workspace),        # Agent's private workspace
            '--dangerously-skip-permissions'
        ]
        
        # Add messages directory if it exists
        messages_dir = workspace.parent.parent / "messages"
        if messages_dir.exists():
            cmd.extend(['--add-dir', str(messages_dir)])
        
        # Execute and stream output
        result = self._execute_streaming(cmd, agent_instance)
        
        if result['success']:
            # Update agent work history
            self._update_agent_work(agent_instance, task)
        
        return result
    
    def _execute_streaming(self, cmd: List[str], agent_instance: str) -> Dict[str, Any]:
        """Execute Claude CLI command with streaming output."""
        start_time = time.time()
        
        try:
            # Start process
            process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1
            )
            
            # Create streaming handler
            stream = StreamingOutput(process, agent_instance)
            
            # Process stdout in thread
            def read_stdout():
                for line in process.stdout:
                    event = stream.parse_line(line)
                    if event:
                        stream.process_event(event)
            
            stdout_thread = threading.Thread(target=read_stdout)
            stdout_thread.start()
            
            # Collect stderr
            stderr_lines = []
            def read_stderr():
                for line in process.stderr:
                    stderr_lines.append(line)
                    
            stderr_thread = threading.Thread(target=read_stderr)
            stderr_thread.start()
            
            # Wait for completion with timeout
            timeout = 3600  # 1 hour max
            try:
                return_code = process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                logger.error(f"[{agent_instance}] Timeout after {timeout}s")
                process.kill()
                return {
                    'success': False,
                    'error': 'Process timeout',
                    'session_id': stream.session_id,
                    'partial_response': '\n'.join(stream.full_response)
                }
            
            # Wait for threads
            stdout_thread.join()
            stderr_thread.join()
            
            # Build result
            elapsed = time.time() - start_time
            
            if return_code == 0:
                logger.info(f"[{agent_instance}] Completed in {elapsed:.1f}s")
                return {
                    'success': True,
                    'session_id': stream.session_id,
                    'response': '\n'.join(stream.full_response),
                    'tool_count': stream.tool_count,
                    'duration': elapsed
                }
            else:
                error_msg = '\n'.join(stderr_lines)
                logger.error(f"[{agent_instance}] Failed: {error_msg}")
                return {
                    'success': False,
                    'error': error_msg,
                    'session_id': stream.session_id,
                    'partial_response': '\n'.join(stream.full_response)
                }
                
        except Exception as e:
            logger.error(f"[{agent_instance}] Exception: {e}")
            return {
                'success': False,
                'error': str(e),
                'session_id': None
            }
    
    def _build_agent_prompt(self, role: str, task: str, workspace: Path,
                           shared_context: Path, project_dir: Path) -> str:
        """Build initial prompt for agent."""
        
        role_guidance = self._get_role_guidance(role)
        
        return f"""You are a {role} agent in the MAOS system.

Your task: {task}

DIRECTORY STRUCTURE:
• Project directory (MAIN WORK HERE): {project_dir}
  - This is where ALL your code and deliverables go
  - Other agents also work here - check for existing files
  
• Shared context: {shared_context}
  - Read summaries from other agents here
  - Write your own summary when done
  - Use for coordination and knowledge sharing
  
• Your private workspace: {workspace}
  - Temporary files only
  - Planning documents, drafts, experiments
  - Nothing here is visible to other agents
  
• Messages directory: {workspace.parent.parent}/messages
  - Inter-agent communication
  - Status updates and notifications

WORKFLOW:
1. Check shared context for previous work/decisions
2. Review project directory for existing code/docs
3. Do your work in the project directory
4. Use your workspace for temporary/draft files
5. Write a summary to shared context when done

{role_guidance}

Remember: Other agents may be working in parallel. Coordinate through shared context and messages."""
    
    def _get_role_guidance(self, role: str) -> str:
        """Get role-specific guidance."""
        guidance = {
            'researcher': """RESEARCHER SPECIFIC:
- Place research documents in project/docs/research/
- Create clear, well-structured documentation
- Include sources and references""",
            
            'architect': """ARCHITECT SPECIFIC:
- Design documents in project/docs/design/
- API specifications in project/docs/api/
- Architecture decision records in project/docs/adr/""",
            
            'backend-engineer': """BACKEND ENGINEER SPECIFIC:
- Source code in appropriate language directories
- Follow project structure and conventions
- Include configuration files""",
            
            'frontend-engineer': """FRONTEND ENGINEER SPECIFIC:
- UI components in appropriate framework structure
- Follow design specifications exactly
- Ensure responsive design""",
            
            'qa': """QA ENGINEER SPECIFIC:
- Tests in language-appropriate test directories
- Test documentation in project/docs/testing/
- Include test data and fixtures"""
        }
        
        return guidance.get(role, "")
    
    def _infer_expertise(self, task: str) -> List[str]:
        """Infer expertise areas from task description."""
        expertise = []
        
        # Simple keyword matching for now
        keywords = {
            'auth': ['authentication', 'login', 'password', 'jwt', 'oauth'],
            'api': ['api', 'endpoint', 'rest', 'graphql'],
            'database': ['database', 'sql', 'postgres', 'mongodb'],
            'ui': ['ui', 'interface', 'frontend', 'design'],
            'security': ['security', 'encryption', 'protection'],
            'testing': ['test', 'testing', 'qa', 'quality']
        }
        
        task_lower = task.lower()
        for area, words in keywords.items():
            if any(word in task_lower for word in words):
                expertise.append(area)
        
        return expertise
    
    def _update_agent_manifest(self, agent_instance: str, agent_data: Dict[str, Any]) -> None:
        """Update manifest with agent information."""
        manifest = {}
        if self.manifest_path.exists():
            try:
                with open(self.manifest_path, 'r') as f:
                    manifest = json.load(f)
            except Exception as e:
                logger.error(f"Failed to load manifest: {e}")
        
        if 'agents' not in manifest:
            manifest['agents'] = {}
            
        manifest['agents'][agent_instance] = agent_data
        
        try:
            self.manifest_path.parent.mkdir(parents=True, exist_ok=True)
            with open(self.manifest_path, 'w') as f:
                json.dump(manifest, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save manifest: {e}")
    
    def _update_agent_work(self, agent_instance: str, task: str) -> None:
        """Update agent's work history and expertise."""
        if not self.manifest_path.exists():
            return
            
        try:
            with open(self.manifest_path, 'r') as f:
                manifest = json.load(f)
            
            if 'agents' in manifest and agent_instance in manifest['agents']:
                agent = manifest['agents'][agent_instance]
                
                # Update work history
                work_history = agent.get('work_history', [])
                work_history.append(task)
                # Keep last 10 items
                agent['work_history'] = work_history[-10:]
                
                # Update expertise
                new_expertise = self._infer_expertise(task)
                existing = set(agent.get('expertise_areas', []))
                existing.update(new_expertise)
                agent['expertise_areas'] = list(existing)
                
                # Update last active
                agent['last_active'] = datetime.now().isoformat()
                
                manifest['agents'][agent_instance] = agent
                
                with open(self.manifest_path, 'w') as f:
                    json.dump(manifest, f, indent=2)
                    
        except Exception as e:
            logger.error(f"Failed to update agent work: {e}")


def test_agent():
    """Test agent functionality."""
    import tempfile
    
    with tempfile.TemporaryDirectory() as tmpdir:
        session_path = Path(tmpdir)
        project_dir = session_path / "project"
        shared_context = session_path / "shared"
        workspace = session_path / "agents" / "test-agent-1"
        
        # Create directories
        project_dir.mkdir(parents=True)
        shared_context.mkdir(parents=True)
        workspace.mkdir(parents=True)
        
        agent = ClaudeAgent(session_path)
        
        print("Testing agent spawn...")
        result = agent.spawn_new(
            agent_instance="test-agent-1",
            role="backend-engineer",
            task="Create a simple hello world API endpoint",
            workspace=workspace,
            shared_context=shared_context,
            project_dir=project_dir
        )
        
        print(f"Result: {json.dumps(result, indent=2)}")


if __name__ == "__main__":
    # Run test if executed directly
    test_agent()