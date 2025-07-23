#!/usr/bin/env python3
"""
Common security utilities for MAOS components.
Provides centralized functions for path sanitization and validation.
"""

import re
from pathlib import Path
from typing import Union


def sanitize_path_component(component: str) -> str:
    """
    Remove dangerous characters from path components.
    
    Args:
        component: The path component to sanitize
        
    Returns:
        Sanitized path component with dangerous characters removed
    """
    # Remove any path separators and parent directory references
    return re.sub(r'[/\\]|\.\.', '', component)


def safe_path_join(base: Union[Path, str], *parts: str) -> Path:
    """
    Safely join path components, preventing directory traversal attacks.
    
    Args:
        base: The base directory path
        *parts: Additional path components to join
        
    Returns:
        Joined path that is guaranteed to be within the base directory
        
    Raises:
        ValueError: If the resulting path would escape the base directory
    """
    base = Path(base) if isinstance(base, str) else base
    
    safe_parts = []
    for part in parts:
        # Remove any path separators and parent directory references
        safe_part = sanitize_path_component(part)
        safe_parts.append(safe_part)
    
    result = base.joinpath(*safe_parts)
    
    # Ensure the result is within the base directory
    try:
        result.resolve().relative_to(base.resolve())
        return result
    except ValueError:
        raise ValueError(f"Path escape attempt detected: {result}")


def validate_agent_name(agent_name: str) -> bool:
    """
    Validate that an agent name contains only safe characters.
    
    Args:
        agent_name: The agent name to validate
        
    Returns:
        True if valid, False otherwise
    """
    return bool(re.match(r'^[a-zA-Z0-9_-]+$', agent_name))


def validate_session_id(session_id: str) -> bool:
    """
    Validate that a session ID contains only safe characters.
    Session IDs can be UUIDs or other alphanumeric identifiers.
    
    Args:
        session_id: The session ID to validate
        
    Returns:
        True if valid, False otherwise
    """
    # Allow alphanumeric, hyphens, and underscores (covers UUIDs and custom IDs)
    return bool(re.match(r'^[a-zA-Z0-9_-]+$', session_id))