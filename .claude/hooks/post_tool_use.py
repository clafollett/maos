#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# ///

import json
import sys
from pathlib import Path
from typing import Dict, Optional

# Import shared path utilities
sys.path.append(str(Path(__file__).parent))
from utils.path_utils import PROJECT_ROOT, LOGS_DIR

# Import MAOS handler
sys.path.append(str(Path(__file__).parent))
try:
    from maos.post_tool_handler import handle_maos_post_tool
except ImportError:
    # Fallback if MAOS not available
    handle_maos_post_tool = None

def main():
    try:
        # Read JSON input from stdin
        input_data = json.load(sys.stdin)
        
        tool_name = input_data.get('tool_name', '')
        tool_input = input_data.get('tool_input', {})
        tool_response = input_data.get('tool_response', {})
        hook_metadata = input_data.get('metadata', {})
        
        # MAOS POST-PROCESSING (non-blocking)
        if handle_maos_post_tool:
            try:
                handle_maos_post_tool(tool_name, tool_input, tool_response, hook_metadata)
            except Exception as e:
                # Non-blocking MAOS error
                print(f"⚠️  MAOS post-processing error (non-blocking): {e}", file=sys.stderr)
        
        # RUST FORMATTING AND LINTING (non-blocking)
        if tool_name in ['Edit', 'MultiEdit']:
            try:
                file_path = tool_input.get('file_path', '')
                if file_path.endswith('.rs'):
                    print("🦀 Formatting and linting Rust code...", file=sys.stderr)
                    import subprocess
                    
                    # Run cargo fmt
                    subprocess.run(['cargo', 'fmt'], check=False, cwd=PROJECT_ROOT)
                    
                    # Run cargo clippy with fixes
                    subprocess.run([
                        'cargo', 'clippy', 
                        '--fix', '--allow-dirty', '--allow-staged', 
                        '--', '-D', 'warnings'
                    ], check=False, cwd=PROJECT_ROOT)
                    
                    print("✅ Rust formatting and linting complete", file=sys.stderr)
            except Exception as e:
                # Non-blocking Rust tooling error
                print(f"⚠️  Rust tooling error (non-blocking): {e}", file=sys.stderr)
        
        # LOGGING (original behavior preserved)
        
        # Ensure log directory exists
        LOGS_DIR.mkdir(parents=True, exist_ok=True)
        log_path = LOGS_DIR / 'post_tool_use.json'
        
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
        # Handle JSON decode errors gracefully
        sys.exit(0)
    except Exception as e:
        # Handle any other errors gracefully - don't block due to MAOS issues
        sys.exit(0)

if __name__ == '__main__':
    main()