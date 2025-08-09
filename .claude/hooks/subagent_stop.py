#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "python-dotenv",
# ]
# ///

import argparse
import json
import os
import sys
import subprocess
from pathlib import Path
from datetime import datetime

try:
    from dotenv import load_dotenv
    load_dotenv()
except ImportError:
    pass  # dotenv is optional


# Import shared utilities
sys.path.append(str(Path(__file__).parent))
from utils.path_utils import PROJECT_ROOT, LOGS_DIR
from utils.config import is_completion_tts_enabled, get_active_tts_provider


def get_tts_script_path():
    """
    Determine which TTS script to use based on config.json provider setting.
    Environment variables are only used for authentication, NOT provider selection.
    """
    # Get current script directory and construct utils/tts path
    script_dir = Path(__file__).parent
    tts_dir = script_dir / "utils" / "tts"
    
    # Use config.json to determine provider (canonical authority)
    provider = get_active_tts_provider()
    
    # Map provider to script file
    provider_scripts = {
        'elevenlabs': tts_dir / "elevenlabs_tts.py",
        'openai': tts_dir / "openai_tts.py", 
        'macos': tts_dir / "macos_tts.py",
        'pyttsx3': tts_dir / "pyttsx3_tts.py"
    }
    
    script_path = provider_scripts.get(provider)
    if script_path and script_path.exists():
        return str(script_path)
    
    # Fallback to macos if configured provider not available
    fallback_script = tts_dir / "macos_tts.py" 
    if fallback_script.exists():
        return str(fallback_script)
        
    return None


def announce_subagent_completion():
    """Announce subagent completion using the best available TTS service."""
    try:
        # First check if completion TTS is enabled in config
        if not is_completion_tts_enabled():
            return  # TTS disabled via config
            
        tts_script = get_tts_script_path()
        if not tts_script:
            return  # No TTS scripts available
        
        # Use fixed message for subagent completion
        completion_message = "Subagent Complete"
        
        # Call the TTS script with the completion message
        subprocess.run([
            "uv", "run", tts_script, completion_message
        ], 
        capture_output=True,  # Suppress output
        timeout=10  # 10-second timeout
        )
        
    except (subprocess.TimeoutExpired, subprocess.SubprocessError, FileNotFoundError):
        # Fail silently if TTS encounters issues
        pass
    except Exception:
        # Fail silently for any other errors
        pass


def main():
    try:
        # Parse command line arguments
        parser = argparse.ArgumentParser()
        parser.add_argument('--chat', action='store_true', help='Copy transcript to chat.json')
        args = parser.parse_args()
        
        # Read JSON input from stdin
        input_data = json.load(sys.stdin)

        # Extract required fields
        session_id = input_data.get("session_id", "")
        stop_hook_active = input_data.get("stop_hook_active", False)

        # Ensure log directory exists
        LOGS_DIR.mkdir(parents=True, exist_ok=True)
        log_path = LOGS_DIR / "subagent_stop.json"

        # Read existing log data or initialize empty list
        if os.path.exists(log_path):
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
        
        # Handle --chat switch (same as stop.py)
        if args.chat and 'transcript_path' in input_data:
            transcript_path = input_data['transcript_path']
            if os.path.exists(transcript_path):
                # Read .jsonl file and convert to JSON array
                chat_data = []
                try:
                    with open(transcript_path, 'r') as f:
                        for line in f:
                            line = line.strip()
                            if line:
                                try:
                                    chat_data.append(json.loads(line))
                                except json.JSONDecodeError:
                                    pass  # Skip invalid lines
                    
                    # Write to logs/chat.json
                    chat_file = LOGS_DIR / 'chat.json'
                    with open(chat_file, 'w') as f:
                        json.dump(chat_data, f, indent=2)
                except Exception:
                    pass  # Fail silently

        # Announce subagent completion via TTS
        announce_subagent_completion()

        sys.exit(0)

    except json.JSONDecodeError:
        # Handle JSON decode errors gracefully
        sys.exit(0)
    except Exception:
        # Handle any other errors gracefully
        sys.exit(0)


if __name__ == "__main__":
    main()