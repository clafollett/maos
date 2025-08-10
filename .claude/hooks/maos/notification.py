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
import random
import time
from pathlib import Path

try:
    from dotenv import load_dotenv
    load_dotenv()
except ImportError:
    pass  # dotenv is optional


# Add path resolution for proper imports
sys.path.insert(0, str(Path(__file__).parent.parent))
from maos.utils.path_utils import PROJECT_ROOT, LOGS_DIR
from maos.utils.config import is_notification_tts_enabled, get_active_tts_provider


def get_tts_script_path():
    """
    Determine which TTS script to use based on configuration.
    """
    # Get current script directory and construct tts path
    script_dir = Path(__file__).parent
    tts_dir = script_dir / "tts"
    
    # Get active provider from config
    provider = get_active_tts_provider()
    
    # Map providers to script paths (updated for new structure)
    script_map = {
        "macos": tts_dir / "macos.py",
        "elevenlabs": tts_dir / "elevenlabs.py", 
        "openai": tts_dir / "openai.py",
        "pyttsx3": tts_dir / "pyttsx3.py"
    }
    
    tts_script = script_map.get(provider)
    if tts_script and tts_script.exists():
        return str(tts_script)
    
    return None


def simple_jsonl_append(log_path, data):
    """Simple, fast JSONL append without reading existing file."""
    try:
        # Ensure log directory exists
        log_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Simple append to JSONL file
        with open(log_path, 'a', encoding='utf-8') as f:
            f.write(json.dumps(data, separators=(',', ':')) + '\n')
        return True
    except Exception:
        return False


def fire_tts_notification():
    """Fire TTS notification immediately - no blocking."""
    try:
        if not is_notification_tts_enabled():
            return False
            
        tts_script = get_tts_script_path()
        if not tts_script:
            return False
        
        # Get engineer name if available
        engineer_name = os.getenv('ENGINEER_NAME', '').strip()
        
        # Create notification message with 30% chance to include name
        if engineer_name and random.random() < 0.3:
            notification_message = f"{engineer_name}, your agent needs your input"
        else:
            notification_message = "Your agent needs your input"
        
        # Fire TTS in background - don't wait for completion
        subprocess.Popen([
            "uv", "run", tts_script, notification_message
        ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        
        return True
        
    except Exception:
        return False


def main():
    """Optimized notification hook - TTS first, logging fire-and-forget."""
    try:
        # Parse arguments
        parser = argparse.ArgumentParser()
        parser.add_argument('--notify', action='store_true', help='Enable TTS notifications')
        args = parser.parse_args()
        
        # Read JSON input from stdin
        input_data = json.loads(sys.stdin.read())
        
        # 🚀 FIRE TTS IMMEDIATELY - TOP PRIORITY
        start_time = time.time()
        tts_fired = False
        
        # Fire TTS only if --notify flag is set AND not generic waiting message
        if args.notify and input_data.get('message') != 'Claude is waiting for your input':
            tts_fired = fire_tts_notification()
        
        tts_time = time.time() - start_time
        if tts_fired:
            print(f"🚀 Notification TTS fired in {tts_time*1000:.2f}ms", file=sys.stderr)
        
        # 📝 LOGGING IN FIRE-AND-FORGET MODE (don't wait)
        # Log to JSONL format - simple append, no reading existing file
        log_path = LOGS_DIR / "notification.jsonl"
        simple_jsonl_append(log_path, input_data)
        
        # Exit immediately - don't wait for logging to complete
        sys.exit(0)
        
    except json.JSONDecodeError:
        sys.exit(0)  # Graceful exit on bad JSON
    except Exception:
        sys.exit(0)  # Graceful exit on any error


if __name__ == '__main__':
    main()