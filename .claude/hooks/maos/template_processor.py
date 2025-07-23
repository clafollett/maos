#!/usr/bin/env python3
"""
Template processor for MAOS agent role templates.
Implements two-phase token replacement similar to the Rust implementation.
"""

import json
import re
import logging
import uuid
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, Optional, List, Tuple

# Import security utilities
from security_utils import sanitize_path_component, safe_path_join, validate_agent_name
# Import agent state management (optional feature)
try:
    from agent_state import get_agent_context
    HAS_STATE_SUPPORT = True
except ImportError:
    HAS_STATE_SUPPORT = False

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


# Security utilities are imported from security_utils module


class TemplateProcessor:
    """Processes agent role templates with token replacement and context injection."""
    
    # Special token for agent context JSON injection
    AGENT_CONTEXT_TOKEN = "{AGENT_CONTEXT}"
    
    # Regex to detect tokens (matches {identifier} but not complex JSON)
    # Fixed to properly escape special characters
    TOKEN_PATTERN = re.compile(r'\{([a-zA-Z_][a-zA-Z0-9_]*)\}')
    
    def __init__(self, session_id: str, agent_name: str, task: str, cwd: Optional[str] = None):
        self.session_id = sanitize_path_component(session_id)
        self.agent_name = sanitize_path_component(agent_name)
        self.task = task
        # Use provided CWD or get current working directory
        self.cwd = Path(cwd) if cwd else Path.cwd()
        self.base_path = self.cwd / ".maos"
        # Generate unique agent ID upfront
        self.agent_id = f"{self.agent_name}-{uuid.uuid4().hex[:8]}"
        
    def build_context(self) -> Dict[str, Any]:
        """Build the complete agent context object."""
        # Create workspace paths using safe path joining
        workspace_path = safe_path_join(self.base_path, "sessions", self.session_id, "agents", self.agent_name)
        shared_context = safe_path_join(self.base_path, "sessions", self.session_id, "shared")
        temp_dir = safe_path_join(self.base_path, "sessions", self.session_id, "temp")
        
        # Ensure directories exist
        workspace_path.mkdir(parents=True, exist_ok=True)
        shared_context.mkdir(parents=True, exist_ok=True)
        temp_dir.mkdir(parents=True, exist_ok=True)
        
        # Get agent state context if available (minimal overhead)
        state_context = ""
        if HAS_STATE_SUPPORT:
            try:
                state_context = get_agent_context(
                    self.base_path / "sessions" / self.session_id,
                    self.agent_name,
                    self.agent_id,
                    self.task
                )
            except Exception as e:
                logger.debug(f"State context unavailable: {e}")
        
        context = {
            "identity": {
                "role": self.agent_name.replace("-", "_"),
                "agent_id": self.agent_id,
                "session_id": self.session_id,
                "instance_number": 1
            },
            "assignment": {
                "task": self.task,
                "priority": "Normal",
                "complexity_level": "Medium",
                "deadline": None,
                "project_context": "MAOS Development"
            },
            "environment": {
                "workspace_path": str(workspace_path),
                "shared_context": str(shared_context),
                "project_root": str(self.cwd),
                "output_dir": str(workspace_path),
                "temp_dir": str(temp_dir)
            },
            "metadata": {
                "template_version": "2.0",
                "created_at": datetime.now().isoformat(),
                "state_context": state_context if state_context else None
            }
        }
        
        return context
    
    def get_token_mappings(self, context: Dict[str, Any]) -> Dict[str, str]:
        """Create token to value mappings from context."""
        return {
            # Environment tokens
            "{workspace_path}": context["environment"]["workspace_path"],
            "{shared_context}": context["environment"]["shared_context"],
            "{project_root}": context["environment"]["project_root"],
            "{output_dir}": context["environment"]["output_dir"],
            "{temp_dir}": context["environment"]["temp_dir"],
            
            # Identity tokens
            "{agent_id}": context["identity"]["agent_id"],
            "{session_id}": context["identity"]["session_id"],
            "{instance_number}": str(context["identity"]["instance_number"]),
            
            # Assignment tokens
            "{task}": context["assignment"]["task"],
            "{priority}": context["assignment"]["priority"],
            "{complexity_level}": context["assignment"]["complexity_level"],
        }
    
    def replace_runtime_tokens(self, template: str, context: Dict[str, Any]) -> Tuple[str, List[str]]:
        """
        Phase 1: Replace runtime tokens with actual values.
        Returns tuple of (processed_template, undefined_tokens).
        """
        result = template
        token_mappings = self.get_token_mappings(context)
        
        # Replace all known tokens
        for token, value in token_mappings.items():
            result = result.replace(token, value)
        
        # Find any remaining undefined tokens
        undefined_tokens = []
        for match in self.TOKEN_PATTERN.finditer(result):
            token = match.group(0)
            # Skip AGENT_CONTEXT as it's handled in phase 2
            if token != self.AGENT_CONTEXT_TOKEN and token not in token_mappings:
                undefined_tokens.append(token)
        
        return result, undefined_tokens
    
    def process_template(self, template_content: str) -> str:
        """
        Process template with two-phase token replacement.
        Phase 1: Replace runtime tokens
        Phase 2: Replace {AGENT_CONTEXT} with JSON
        """
        # Build context
        context = self.build_context()
        
        # Phase 1: Replace runtime tokens
        processed, undefined = self.replace_runtime_tokens(template_content, context)
        
        if undefined:
            logger.warning(f"Undefined tokens found: {undefined}")
        
        # Phase 2: Replace {AGENT_CONTEXT} with JSON
        # Properly escape JSON to prevent injection
        context_json = json.dumps(context, indent=2)
        # Ensure no further substitutions can occur
        processed = processed.replace(self.AGENT_CONTEXT_TOKEN, context_json)
        
        return processed
    
    def load_and_process_template(self, template_path: str) -> str:
        """Load a template file and process it."""
        # Validate and resolve template path
        try:
            # Split the template path into parts for safe joining
            # This handles paths like "assets/agent-roles/backend-engineer.md"
            template_parts = Path(template_path).parts
            full_path = safe_path_join(self.cwd, *template_parts)
        except ValueError as e:
            logger.error(f"Invalid template path: {e}")
            raise ValueError(f"Invalid template path: {template_path}")
        
        # Check if file exists
        if not full_path.exists():
            raise FileNotFoundError(f"Template file not found: {full_path}")
        
        try:
            with open(full_path, 'r') as f:
                template_content = f.read()
        except OSError as e:
            logger.error(f"Failed to read template file: {e}")
            raise
        
        return self.process_template(template_content)


def process_agent_template(agent_name: str, session_id: str, task: str, template_path: str, cwd: Optional[str] = None) -> str:
    """
    Convenience function to process an agent template.
    
    Args:
        agent_name: Name of the agent (e.g., "backend-engineer")
        session_id: Current session ID
        task: The task description
        template_path: Path to the role template file
        cwd: Current working directory (optional, defaults to os.getcwd())
        
    Returns:
        Processed template with all tokens replaced
    """
    # Validate agent name
    if not validate_agent_name(agent_name):
        raise ValueError(f"Invalid agent name: {agent_name}")
    
    processor = TemplateProcessor(session_id, agent_name, task, cwd)
    return processor.load_and_process_template(template_path)


if __name__ == "__main__":
    # Test the processor
    import sys
    
    if len(sys.argv) < 5:
        print("Usage: template_processor.py <agent_name> <session_id> <task> <template_path> [cwd]")
        sys.exit(1)
    
    agent_name, session_id, task, template_path = sys.argv[1:5]
    cwd = sys.argv[5] if len(sys.argv) > 5 else None
    
    try:
        processed = process_agent_template(agent_name, session_id, task, template_path, cwd)
        print(processed)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)