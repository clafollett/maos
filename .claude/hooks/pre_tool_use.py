#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "python-dotenv",
# ]
# ///

import json
import sys
import re
import os
import asyncio
import time
import threading
from pathlib import Path
from typing import Dict, Optional
from concurrent.futures import ThreadPoolExecutor

try:
    from dotenv import load_dotenv
    load_dotenv()
except ImportError:
    pass  # dotenv is optional

# Import shared path utilities
sys.path.append(str(Path(__file__).parent))
from utils.path_utils import PROJECT_ROOT, LOGS_DIR
from utils.async_logging import log_hook_data, log_hook_data_sync, get_task_manager

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
            # Block .env files but allow .env.sample and stack.env
            if '.env' in file_path and not file_path.endswith('.env.sample') and not file_path.endswith('stack.env'):
                return True
        
        # Check bash commands for .env file access
        elif tool_name == 'Bash':
            command = tool_input.get('command', '')
            # Pattern to detect .env file access (but allow .env.sample and stack.env)
            env_patterns = [
                r'(?<!stack)\.env\b(?!\.sample)',  # .env but not .env.sample or stack.env
                r'cat\s+.*(?<!stack)\.env\b(?!\.sample)',  # cat .env
                r'echo\s+.*>\s*(?<!stack)\.env\b(?!\.sample)',  # echo > .env
                r'touch\s+.*(?<!stack)\.env\b(?!\.sample)',  # touch .env
                r'cp\s+.*(?<!stack)\.env\b(?!\.sample)',  # cp .env
                r'mv\s+.*(?<!stack)\.env\b(?!\.sample)',  # mv .env
            ]
            
            for pattern in env_patterns:
                if re.search(pattern, command):
                    return True
    
    return False

async def run_maos_background(tool_name: str, tool_input: Dict, hook_metadata: Dict) -> None:
    """Run MAOS orchestration in background without blocking."""
    if not handle_maos_pre_tool:
        return
    
    def maos_task():
        try:
            handle_maos_pre_tool(tool_name, tool_input, hook_metadata)
        except Exception as e:
            print(f"⚠️  MAOS processing error (background): {e}", file=sys.stderr)
    
    # Run MAOS in background task manager
    task_manager = get_task_manager()
    await task_manager.run_background_task(maos_task, timeout=10.0)


async def main_async():
    """ASYNC main function - security checks first, everything else in background."""
    try:
        start_time = time.time()
        
        # Read JSON input from stdin
        input_data = json.load(sys.stdin)
        
        tool_name = input_data.get('tool_name', '')
        tool_input = input_data.get('tool_input', {})
        hook_metadata = input_data.get('metadata', {})
        
        # 🚨 CRITICAL SECURITY CHECKS FIRST (these can block operations)
        # These must run synchronously to block dangerous operations
        
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
        
        security_time = time.time() - start_time
        print(f"🔒 Security checks completed in {security_time*1000:.2f}ms", file=sys.stderr)
        
        # 🚀 EVERYTHING ELSE RUNS IN BACKGROUND (non-blocking)
        
        # Start background tasks in parallel
        background_tasks = []
        
        # MAOS orchestration in background
        if handle_maos_pre_tool:
            maos_task = asyncio.create_task(run_maos_background(tool_name, tool_input, hook_metadata))
            background_tasks.append(maos_task)
        
        # Async logging (JSONL format for true append-only)
        log_path = LOGS_DIR / 'pre_tool_use.jsonl'  # Changed to .jsonl
        logging_task = asyncio.create_task(log_hook_data(log_path, input_data))
        background_tasks.append(logging_task)
        
        # Give background tasks a moment to start, but don't wait for completion
        if background_tasks:
            # Wait just long enough to ensure tasks are started (non-blocking)
            try:
                await asyncio.wait_for(
                    asyncio.gather(*background_tasks, return_exceptions=True),
                    timeout=0.1  # Very short timeout - just to start tasks
                )
            except asyncio.TimeoutError:
                # This is expected - we don't want to block
                pass
        
        total_time = time.time() - start_time
        print(f"⚡ Pre-tool hook completed in {total_time*1000:.2f}ms (background tasks running)", file=sys.stderr)
        
        sys.exit(0)
        
    except json.JSONDecodeError:
        # Gracefully handle JSON decode errors
        sys.exit(0)
    except Exception as e:
        # Handle any other errors gracefully - don't block operations
        print(f"⚠️  Pre-tool hook error (non-blocking): {e}", file=sys.stderr)
        sys.exit(0)


def main():
    """Wrapper to run async main with fallback."""
    try:
        # Try async version first
        try:
            asyncio.get_running_loop()
            # Loop already running, create new thread
            result = [None]
            exception = [None]
            
            def run_async():
                try:
                    new_loop = asyncio.new_event_loop()
                    asyncio.set_event_loop(new_loop)
                    result[0] = new_loop.run_until_complete(main_async())
                except Exception as e:
                    exception[0] = e
                finally:
                    new_loop.close()
            
            thread = threading.Thread(target=run_async)
            thread.start()
            thread.join()
            
            if exception[0]:
                raise exception[0]
                
        except RuntimeError:
            # No running event loop - normal case
            asyncio.run(main_async())
            
    except Exception:
        # Fallback to synchronous version
        main_sync()


def main_sync():
    """Fallback synchronous version with optimized I/O."""
    try:
        start_time = time.time()
        
        # Read JSON input from stdin
        input_data = json.load(sys.stdin)
        
        tool_name = input_data.get('tool_name', '')
        tool_input = input_data.get('tool_input', {})
        hook_metadata = input_data.get('metadata', {})
        
        # 🚨 SECURITY CHECKS FIRST (blocking)
        
        if is_env_file_access(tool_name, tool_input):
            print("BLOCKED: Access to .env files containing sensitive data is prohibited", file=sys.stderr)
            print("Use .env.sample for template files instead", file=sys.stderr)
            sys.exit(2)
        
        if tool_name == 'Bash':
            command = tool_input.get('command', '')
            if is_dangerous_rm_command(command):
                print("BLOCKED: Dangerous rm command detected and prevented", file=sys.stderr)
                sys.exit(2)
        
        security_time = time.time() - start_time
        print(f"🔒 Security checks completed in {security_time*1000:.2f}ms", file=sys.stderr)
        
        # 🚀 BACKGROUND PROCESSING
        
        def background_work():
            # MAOS orchestration
            if handle_maos_pre_tool:
                try:
                    handle_maos_pre_tool(tool_name, tool_input, hook_metadata)
                except Exception as e:
                    print(f"⚠️  MAOS processing error (background): {e}", file=sys.stderr)
            
            # Fast JSONL logging
            log_path = LOGS_DIR / 'pre_tool_use.jsonl'
            log_hook_data_sync(log_path, input_data)
        
        # Start background thread
        bg_thread = threading.Thread(target=background_work, daemon=True)
        bg_thread.start()
        
        # Give background thread a tiny moment to start
        bg_thread.join(timeout=0.05)
        
        total_time = time.time() - start_time
        print(f"⚡ Pre-tool hook completed in {total_time*1000:.2f}ms (sync fallback)", file=sys.stderr)
        
        sys.exit(0)
        
    except json.JSONDecodeError:
        sys.exit(0)
    except Exception as e:
        print(f"⚠️  Pre-tool hook error (sync fallback): {e}", file=sys.stderr)
        sys.exit(0)

if __name__ == '__main__':
    main()