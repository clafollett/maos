#!/usr/bin/env python3
"""
Claude CLI Orchestrator - Manages persistent orchestrator session for intelligent routing.
This module handles the creation and management of a long-lived Claude session that
makes all agent routing decisions using AI intelligence.
"""

import json
import subprocess
import logging
import time
from pathlib import Path
from typing import Dict, Optional, Any, Tuple
from datetime import datetime

from security_utils import safe_path_join, sanitize_path_component

logger = logging.getLogger(__name__)


class ClaudeOrchestrator:
    """Manages persistent Claude CLI orchestrator session."""
    
    def __init__(self, session_path: Path):
        """Initialize orchestrator with session path."""
        self.session_path = session_path
        self.manifest_path = safe_path_join(session_path, "manifest.json")
        self.session_id = None
        self._load_session()
    
    def _load_session(self) -> None:
        """Load existing orchestrator session from manifest."""
        if self.manifest_path.exists():
            try:
                with open(self.manifest_path, 'r') as f:
                    manifest = json.load(f)
                    orchestrator_data = manifest.get('orchestrator', {})
                    self.session_id = orchestrator_data.get('session_id')
                    logger.info(f"Loaded orchestrator session: {self.session_id}")
            except Exception as e:
                logger.error(f"Failed to load orchestrator session: {e}")
    
    def ensure_session(self) -> str:
        """Ensure orchestrator session exists, creating if necessary."""
        if self.session_id:
            return self.session_id
        
        logger.info("Creating new orchestrator session...")
        
        # Orchestrator initialization prompt
        prompt = """You are the MAOS Orchestration Agent responsible for intelligent agent routing.

Your role is to:
1. Maintain awareness of all agents and their specializations
2. Make intelligent routing decisions for new tasks
3. Track which agents have worked on which components
4. Ensure efficient reuse of agent expertise

You will be consulted for every agent request to determine:
- Whether to reuse an existing agent with relevant experience
- Whether to create a new agent instance
- How to best utilize agent specializations

Respond with: "Orchestrator ready. I will help route agents efficiently based on their expertise and work history."
"""
        
        try:
            # Spawn Claude CLI to create orchestrator session
            result = subprocess.run([
                'claude',
                '--model', 'opus',
                '--fallback-model', 'sonnet',
                '-p', prompt,
                '--output-format', 'json'
            ], capture_output=True, text=True, check=True)
            
            # Parse JSON output
            output = json.loads(result.stdout)
            self.session_id = output.get('session_id')
            
            if not self.session_id:
                raise ValueError("No session_id in orchestrator creation response")
            
            logger.info(f"Created orchestrator session: {self.session_id}")
            
            # Save to manifest
            self._update_manifest({
                'session_id': self.session_id,
                'created_at': datetime.now().isoformat(),
                'status': 'active',
                'consultations': 0
            })
            
            return self.session_id
            
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to create orchestrator session: {e}")
            logger.error(f"stderr: {e.stderr}")
            raise RuntimeError(f"Failed to create orchestrator session: {e.stderr}")
        except json.JSONDecodeError as e:
            logger.error(f"Failed to parse orchestrator response: {e}")
            raise RuntimeError("Invalid JSON response from Claude CLI")
    
    def consult(self, agent_role: str, task: str, current_manifest: Dict[str, Any]) -> Dict[str, Any]:
        """Consult orchestrator for routing decision."""
        self.ensure_session()
        
        # Build consultation prompt
        agents_summary = self._build_agents_summary(current_manifest.get('agents', {}))
        
        prompt = f"""Current session state:
{agents_summary}

NEW REQUEST:
- Agent Type: {agent_role}
- Task: {task}

Analyze the current agents and their expertise to decide:
1. Should we reuse an existing agent that has relevant experience?
2. Or create a new agent instance?

Consider:
- Which agents have worked on related components
- Which agents have developed relevant expertise  
- Whether the task is a continuation of existing work
- Load balancing across agents

Respond with ONLY valid JSON:
{{
  "decision": "reuse" or "create_new",
  "agent_instance": "backend-engineer-1" or "backend-engineer-3",
  "reasoning": "Brief explanation of the decision",
  "context": "Any relevant context the agent should know about related work"
}}"""
        
        try:
            # Resume orchestrator session for consultation
            result = subprocess.run([
                'claude',
                '--resume', self.session_id,
                '-p', prompt,
                '--output-format', 'json'
            ], capture_output=True, text=True, check=True)
            
            # Parse response
            output = json.loads(result.stdout)
            response_text = output.get('result', '')
            
            # Try to extract JSON from response
            decision = self._parse_routing_decision(response_text)
            
            # Update consultation count
            self._increment_consultations()
            
            return decision
            
        except subprocess.CalledProcessError as e:
            logger.error(f"Orchestrator consultation failed: {e}")
            logger.error(f"stderr: {e.stderr}")
            # Fallback to creating new agent
            return self._create_fallback_decision(agent_role, current_manifest)
        except Exception as e:
            logger.error(f"Failed to consult orchestrator: {e}")
            return self._create_fallback_decision(agent_role, current_manifest)
    
    def _build_agents_summary(self, agents: Dict[str, Any]) -> str:
        """Build a summary of current agents for orchestrator context."""
        if not agents:
            return "No agents currently active."
        
        summary_lines = ["Current agents:"]
        for instance, data in agents.items():
            status = data.get('status', 'unknown')
            expertise = ', '.join(data.get('expertise_areas', []))
            work_count = len(data.get('work_history', []))
            
            summary_lines.append(
                f"- {instance}: {status}, {work_count} tasks completed, "
                f"expertise in [{expertise}]"
            )
            
            # Include recent work
            recent_work = data.get('work_history', [])[-2:]
            if recent_work:
                for work in recent_work:
                    summary_lines.append(f"  • {work}")
        
        return '\n'.join(summary_lines)
    
    def _parse_routing_decision(self, response_text: str) -> Dict[str, Any]:
        """Parse routing decision from orchestrator response."""
        # Try to extract JSON from response
        try:
            # Handle case where response is already parsed JSON
            if isinstance(response_text, dict):
                return response_text
            
            # Try direct JSON parse
            return json.loads(response_text)
        except json.JSONDecodeError:
            # Try to find JSON in the response
            import re
            json_match = re.search(r'\{[^{}]*\}', response_text, re.DOTALL)
            if json_match:
                try:
                    return json.loads(json_match.group())
                except json.JSONDecodeError:
                    pass
        
        # If all parsing fails, raise exception
        raise ValueError(f"Could not parse routing decision from: {response_text}")
    
    def _create_fallback_decision(self, agent_role: str, manifest: Dict[str, Any]) -> Dict[str, Any]:
        """Create fallback routing decision when orchestrator fails."""
        # Find next available instance number
        agents = manifest.get('agents', {})
        existing_instances = [
            name for name in agents.keys() 
            if name.startswith(f"{agent_role}-")
        ]
        
        next_num = 1
        while f"{agent_role}-{next_num}" in existing_instances:
            next_num += 1
        
        return {
            'decision': 'create_new',
            'agent_instance': f"{agent_role}-{next_num}",
            'reasoning': 'Orchestrator unavailable, creating new instance',
            'context': ''
        }
    
    def _update_manifest(self, orchestrator_data: Dict[str, Any]) -> None:
        """Update manifest with orchestrator data."""
        manifest = {}
        if self.manifest_path.exists():
            try:
                with open(self.manifest_path, 'r') as f:
                    manifest = json.load(f)
            except Exception as e:
                logger.error(f"Failed to load manifest: {e}")
        
        manifest['orchestrator'] = orchestrator_data
        
        try:
            self.manifest_path.parent.mkdir(parents=True, exist_ok=True)
            with open(self.manifest_path, 'w') as f:
                json.dump(manifest, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save manifest: {e}")
    
    def _increment_consultations(self) -> None:
        """Increment consultation counter in manifest."""
        if self.manifest_path.exists():
            try:
                with open(self.manifest_path, 'r') as f:
                    manifest = json.load(f)
                
                orchestrator = manifest.get('orchestrator', {})
                orchestrator['consultations'] = orchestrator.get('consultations', 0) + 1
                orchestrator['last_consultation'] = datetime.now().isoformat()
                manifest['orchestrator'] = orchestrator
                
                with open(self.manifest_path, 'w') as f:
                    json.dump(manifest, f, indent=2)
            except Exception as e:
                logger.error(f"Failed to update consultation count: {e}")
    
    def get_status(self) -> Dict[str, Any]:
        """Get orchestrator status."""
        if not self.manifest_path.exists():
            return {'status': 'not_initialized'}
        
        try:
            with open(self.manifest_path, 'r') as f:
                manifest = json.load(f)
                return manifest.get('orchestrator', {'status': 'unknown'})
        except Exception:
            return {'status': 'error'}


def test_orchestrator():
    """Test orchestrator functionality."""
    import tempfile
    
    with tempfile.TemporaryDirectory() as tmpdir:
        session_path = Path(tmpdir)
        orchestrator = ClaudeOrchestrator(session_path)
        
        # Test session creation
        print("Creating orchestrator session...")
        session_id = orchestrator.ensure_session()
        print(f"Session ID: {session_id}")
        
        # Test consultation
        print("\nTesting consultation...")
        test_manifest = {
            'agents': {
                'backend-engineer-1': {
                    'status': 'available',
                    'expertise_areas': ['authentication', 'api'],
                    'work_history': ['Implemented login API', 'Added JWT tokens']
                }
            }
        }
        
        decision = orchestrator.consult(
            'backend-engineer',
            'Add password reset functionality',
            test_manifest
        )
        
        print(f"Decision: {json.dumps(decision, indent=2)}")
        
        # Test status
        print("\nOrchestrator status:")
        print(json.dumps(orchestrator.get_status(), indent=2))


if __name__ == "__main__":
    # Run test if executed directly
    test_orchestrator()