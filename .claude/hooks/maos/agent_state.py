#!/usr/bin/env python3
"""
Lightweight agent state management for MAOS.
Designed to minimize token overhead while providing basic memory capabilities.
"""

import json
import logging
from pathlib import Path
from datetime import datetime
from typing import Dict, Any
from security_utils import safe_path_join

logger = logging.getLogger(__name__)

class AgentState:
    """Manages lightweight state for agents to maintain context between runs."""
    
    def __init__(self, session_path: Path, agent_name: str):
        self.session_path = session_path
        self.agent_name = agent_name
        self.state_file = safe_path_join(
            session_path, "agents", agent_name, "state.json"
        )
    
    def load(self) -> Dict[str, Any]:
        """Load existing state or return empty state."""
        if self.state_file.exists():
            try:
                with open(self.state_file, 'r') as f:
                    return json.load(f)
            except Exception as e:
                logger.error(f"Failed to load state: {e}")
        
        return {
            "agent_name": self.agent_name,
            "created_at": datetime.now().isoformat(),
            "last_updated": datetime.now().isoformat(),
            "instances": [],
            "summary": {
                "files_created": [],
                "decisions_made": [],
                "key_findings": []
            }
        }
    
    def add_instance(self, agent_id: str, task: str) -> None:
        """Record a new agent instance (lightweight)."""
        state = self.load()
        
        # Keep only last 5 instances to minimize size
        state["instances"].append({
            "agent_id": agent_id,
            "timestamp": datetime.now().isoformat(),
            "task_summary": task[:200]  # First 200 chars only
        })
        if len(state["instances"]) > 5:
            state["instances"] = state["instances"][-5:]
        
        state["last_updated"] = datetime.now().isoformat()
        self._save(state)
    
    def update_summary(self, updates: Dict[str, Any]) -> None:
        """Update summary with key information only."""
        state = self.load()
        
        # Merge updates into summary
        for key, value in updates.items():
            if key in state["summary"]:
                if isinstance(value, list):
                    # Append and deduplicate
                    state["summary"][key].extend(value)
                    state["summary"][key] = list(set(state["summary"][key]))
                    # Keep only last 10 items
                    state["summary"][key] = state["summary"][key][-10:]
                else:
                    state["summary"][key] = value
        
        state["last_updated"] = datetime.now().isoformat()
        self._save(state)
    
    def get_context_prompt(self) -> str:
        """Generate a minimal context prompt for the agent."""
        state = self.load()
        
        if not state["instances"]:
            return ""
        
        # Build minimal context
        context = f"You are {self.agent_name}. "
        
        if state["summary"]["files_created"]:
            context += f"Previous work created: {', '.join(state['summary']['files_created'][-3:])}. "
        
        if state["summary"]["key_findings"]:
            context += f"Key findings: {', '.join(state['summary']['key_findings'][-3:])}. "
        
        return context
    
    def _save(self, state: Dict[str, Any]) -> None:
        """Save state to file."""
        try:
            self.state_file.parent.mkdir(parents=True, exist_ok=True)
            with open(self.state_file, 'w') as f:
                json.dump(state, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save state: {e}")


def get_agent_context(session_path: Path, agent_name: str, agent_id: str, task: str) -> str:
    """Get minimal agent context, updating state."""
    state_manager = AgentState(session_path, agent_name)
    
    # Record this instance
    state_manager.add_instance(agent_id, task)
    
    # Return minimal context
    return state_manager.get_context_prompt()