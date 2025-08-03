#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# ///

import json
import sys
import re
import os
from pathlib import Path
from typing import Dict, Optional

# Import shared path utilities
sys.path.append(str(Path(__file__).parent))
from utils.path_utils import PROJECT_ROOT, LOGS_DIR

# Import MAOS handler
sys.path.append(str(Path(__file__).parent))
try:
    from maos.pre_tool_handler import handle_maos_pre_tool
except ImportError:
    # Fallback if MAOS not available
    handle_maos_pre_tool = None

def is_dangerous_rm_command(command):
    """
    Comprehensive detection of dangerous rm commands.
    Properly distinguishes between flags and filenames to avoid false positives.
    """
    # Split into tokens to properly identify flags vs filenames
    tokens = command.split()
    
    if not tokens or tokens[0].lower() != 'rm':
        return False
    
    # Remove 'rm' to focus on arguments
    args = tokens[1:]
    
    # Track flags found
    has_recursive = False
    has_force = False
    
    # Check for -- which stops flag processing
    double_dash_index = len(args)
    for i, arg in enumerate(args):
        if arg == '--':
            double_dash_index = i
            break
    
    # Process arguments before --
    for i, arg in enumerate(args[:double_dash_index]):
        arg_lower = arg.lower()
        
        # Long options
        if arg_lower == '--recursive':
            has_recursive = True
        elif arg_lower == '--force':
            has_force = True
        # Short options (must start with single dash and have letters)
        elif arg.startswith('-') and len(arg) > 1 and arg[1] != '-':
            # Check each character in the flag
            for char in arg[1:]:
                if char in 'rR':
                    has_recursive = True
                elif char == 'f':
                    has_force = True
    
    # Check for dangerous combinations
    if has_recursive and has_force:
        return True
    
    # Check for just recursive with dangerous paths
    if has_recursive:
        # Get all non-flag arguments (potential paths)
        paths = []
        for i, arg in enumerate(args):
            # Skip flags and arguments before --
            if i < double_dash_index and arg.startswith('-') and arg != '-':
                continue
            # After --, everything is a path
            if i > double_dash_index:
                paths.append(arg)
            # Before --, non-flag args are paths
            elif not arg.startswith('-') or arg == '-':
                paths.append(arg)
        
        # Check for dangerous paths
        dangerous_patterns = [
            r'^/$',          # Exactly root
            r'^/\*',         # Root with wildcard
            r'^~/?$',        # Home directory
            r'^\$HOME',      # HOME variable
            r'^\.\./?',      # Parent directory
            r'^\*$',         # Just wildcard
            r'^\.$',         # Current directory
        ]
        
        for path in paths:
            for pattern in dangerous_patterns:
                if re.match(pattern, path):
                    return True
    
    return False

def is_env_file_access(tool_name, tool_input):
    """
    Check if any tool is trying to access .env files containing sensitive data.
    """
    if tool_name in ['Read', 'Edit', 'MultiEdit', 'Write', 'Bash']:
        # Check file paths for file-based tools
        if tool_name in ['Read', 'Edit', 'MultiEdit', 'Write']:
            file_path = tool_input.get('file_path', '')
            if '.env' in file_path and not file_path.endswith('.env.sample'):
                return True
        
        # Check bash commands for .env file access
        elif tool_name == 'Bash':
            command = tool_input.get('command', '')
            # Pattern to detect .env file access (but allow .env.sample)
            env_patterns = [
                r'\b\.env\b(?!\.sample)',  # .env but not .env.sample
                r'cat\s+.*\.env\b(?!\.sample)',  # cat .env
                r'echo\s+.*>\s*\.env\b(?!\.sample)',  # echo > .env
                r'touch\s+.*\.env\b(?!\.sample)',  # touch .env
                r'cp\s+.*\.env\b(?!\.sample)',  # cp .env
                r'mv\s+.*\.env\b(?!\.sample)',  # mv .env
            ]
            
            for pattern in env_patterns:
                if re.search(pattern, command):
                    return True
    
    return False

def main():
    try:
        # Read JSON input from stdin
        input_data = json.load(sys.stdin)
        
        tool_name = input_data.get('tool_name', '')
        tool_input = input_data.get('tool_input', {})
        hook_metadata = input_data.get('metadata', {})
        
        # CRITICAL SECURITY CHECKS FIRST (these can block operations)
        
        # Check for .env file access (blocks access to sensitive environment files)
        if is_env_file_access(tool_name, tool_input):
            print("BLOCKED: Access to .env files containing sensitive data is prohibited", file=sys.stderr)
            print("Use .env.sample for template files instead", file=sys.stderr)
            sys.exit(2)  # Exit code 2 blocks tool call and shows error to Claude
        
        # Check for dangerous rm -rf commands
        if tool_name == 'Bash':
            command = tool_input.get('command', '')
            
            # Block rm -rf commands with comprehensive pattern matching
            if is_dangerous_rm_command(command):
                print("BLOCKED: Dangerous rm command detected and prevented", file=sys.stderr)
                sys.exit(2)  # Exit code 2 blocks tool call and shows error to Claude
        
        # MAOS ORCHESTRATION (non-blocking, coordination only)
        if handle_maos_pre_tool:
            try:
                handle_maos_pre_tool(tool_name, tool_input, hook_metadata)
            except Exception as e:
                # Non-blocking MAOS error
                print(f"⚠️  MAOS processing error (non-blocking): {e}", file=sys.stderr)
        
        # LOGGING (original behavior preserved)
        
        # Ensure log directory exists
        LOGS_DIR.mkdir(parents=True, exist_ok=True)
        log_path = LOGS_DIR / 'pre_tool_use.json'
        
        # Read existing log data or initialize empty list
        if log_path.exists():
            with open(log_path, 'r') as f:
                try:
                    log_data = json.load(f)
                except (json.JSONDecodeError, ValueError):
                    log_data = []
        else:
            log_data = []
        
        # Append new data
        log_data.append(input_data)
        
        # Write back to file with formatting
        with open(log_path, 'w') as f:
            json.dump(log_data, f, indent=2)
        
        sys.exit(0)
        
    except json.JSONDecodeError:
        # Gracefully handle JSON decode errors
        sys.exit(0)
    except Exception as e:
        # Handle any other errors gracefully - don't block operations due to MAOS issues
        print(f"⚠️  MAOS hook error (non-blocking): {e}", file=sys.stderr)
        sys.exit(0)

if __name__ == '__main__':
    main()