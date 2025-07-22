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
import random
import subprocess
from pathlib import Path

try:
    from dotenv import load_dotenv
    load_dotenv()
except ImportError:
    pass  # dotenv is optional

# Import config utility
sys.path.append(str(Path(__file__).parent / "utils"))
from config import is_response_tts_enabled, is_completion_tts_enabled, get_engineer_name


def get_completion_messages():
    """Return list of friendly completion messages with engineer name."""
    engineer_name = get_engineer_name()
    name_prefix = f"Hey {engineer_name}! " if engineer_name else ""
    name_suffix = f", {engineer_name}!" if engineer_name else "!"
    
    return [
        f"{name_prefix}All done!",
        f"{name_prefix}We're ready for next task!",
        f"Work complete{name_suffix}",
        f"Task finished{name_suffix}",
        f"Job complete{name_suffix}"
    ]

def get_tts_script_path():
    """
    Determine which TTS script to use based on available API keys.
    Priority order: ElevenLabs > OpenAI > pyttsx3
    """
    # Get current script directory and construct utils/tts path
    script_dir = Path(__file__).parent
    tts_dir = script_dir / "utils" / "tts"
    
    # Check for ElevenLabs API key (highest priority)
    if os.getenv('ELEVENLABS_API_KEY'):
        elevenlabs_script = tts_dir / "elevenlabs_tts.py"
        if elevenlabs_script.exists():
            return str(elevenlabs_script)
    
    # Check for OpenAI API key (second priority)
    if os.getenv('OPENAI_API_KEY'):
        openai_script = tts_dir / "openai_tts.py"
        if openai_script.exists():
            return str(openai_script)
    
    # Fall back to pyttsx3 (no API key required)
    pyttsx3_script = tts_dir / "pyttsx3_tts.py"
    if pyttsx3_script.exists():
        return str(pyttsx3_script)
    
    return None


def get_llm_completion_message():
    """
    Generate completion message using available LLM services.
    Priority order: OpenAI > Anthropic > fallback to random message
    
    Returns:
        str: Generated or fallback completion message
    """
    # Get current script directory and construct utils/llm path
    script_dir = Path(__file__).parent
    llm_dir = script_dir / "utils" / "llm"
    
    # Try OpenAI first (highest priority)
    if os.getenv('OPENAI_API_KEY'):
        oai_script = llm_dir / "oai.py"
        if oai_script.exists():
            try:
                result = subprocess.run([
                    "uv", "run", str(oai_script), "--completion"
                ], 
                capture_output=True,
                text=True,
                timeout=10
                )
                if result.returncode == 0 and result.stdout.strip():
                    return result.stdout.strip()
            except (subprocess.TimeoutExpired, subprocess.SubprocessError):
                pass
    
    # Try Anthropic second
    if os.getenv('ANTHROPIC_API_KEY'):
        anth_script = llm_dir / "anth.py"
        if anth_script.exists():
            try:
                result = subprocess.run([
                    "uv", "run", str(anth_script), "--completion"
                ], 
                capture_output=True,
                text=True,
                timeout=10
                )
                if result.returncode == 0 and result.stdout.strip():
                    return result.stdout.strip()
            except (subprocess.TimeoutExpired, subprocess.SubprocessError):
                pass
    
    # Fallback to random predefined message
    messages = get_completion_messages()
    return random.choice(messages)

def trigger_conversation_tts(input_data):
    """
    Trigger TTS for conversational responses if enabled.
    Reads the transcript to find the latest assistant response.
    """
    # Check if response TTS is enabled
    if not is_response_tts_enabled():
        return
    
    # Get transcript path
    transcript_path = input_data.get('transcript_path')
    if not transcript_path or not os.path.exists(transcript_path):
        return
    
    try:
        # Read the transcript file (JSONL format)
        assistant_messages = []
        with open(transcript_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        entry = json.loads(line)
                        # Check for assistant messages with text content
                        if (entry.get('type') == 'assistant' and 
                            entry.get('message', {}).get('content')):
                            content = entry['message']['content']
                            for item in content:
                                if item.get('type') == 'text' and item.get('text'):
                                    assistant_messages.append(item['text'])
                    except json.JSONDecodeError:
                        continue
        
        # Get the most recent assistant message
        if not assistant_messages:
            return
        
        latest_response = assistant_messages[-1]
        
        # Skip if it's too short or looks like a completion message
        if len(latest_response.strip()) < 20:
            return
        
        # Skip if it contains mainly tool calls or excessive code blocks
        MAX_CODE_BLOCKS = 2
        has_function_calls = '<function_calls>' in latest_response
        has_many_code_blocks = latest_response.count('```') > MAX_CODE_BLOCKS
        
        if has_function_calls or has_many_code_blocks:
            return
        
        # Get script directory and construct path to response TTS
        script_dir = Path(__file__).parent
        tts_script = script_dir / "utils" / "tts" / "response_tts.py"
        
        if not tts_script.exists():
            return
        
        # Trigger TTS with proper process management
        subprocess.run([
            "uv", "run", str(tts_script), latest_response
            ], 
            capture_output=True,  # Capture output to prevent blocking
            timeout=120,  # 120-second timeout to prevent hanging
            check=False  # Don't raise exception on non-zero exit
        )
        
    except subprocess.SubprocessError as e:
        # Log subprocess errors for debugging
        print(f"TTS subprocess error: {e}", file=sys.stderr)
    except (OSError, json.JSONDecodeError):
        # Silent failure for expected file/parsing errors
        pass
    except Exception as e:
        # Log any unexpected errors
        print(f"TTS unexpected error: {e}", file=sys.stderr)

def announce_completion():
    """Announce completion using the best available TTS service."""
    try:
        # Skip completion announcement if response TTS is enabled
        # (since assistant will already be speaking the response)
        if is_response_tts_enabled():
            return
        
        # Check if completion TTS is enabled
        if not is_completion_tts_enabled():
            return
        
        tts_script = get_tts_script_path()
        if not tts_script:
            return  # No TTS scripts available
        
        # Get completion message (LLM-generated or fallback)
        completion_message = get_llm_completion_message()
        
        # Call the TTS script with the completion message
        subprocess.run([
            "uv", "run", tts_script, completion_message
        ], 
        capture_output=True,  # Suppress output
        timeout=10  # 10-second timeout
        )
        
    except subprocess.TimeoutExpired:
        print("Completion TTS timeout after 10 seconds", file=sys.stderr)
    except FileNotFoundError as e:
        print(f"Completion TTS file not found: {e}", file=sys.stderr)
    except subprocess.SubprocessError as e:
        print(f"Completion TTS subprocess error: {e}", file=sys.stderr)
    except Exception as e:
        print(f"Completion TTS unexpected error: {e}", file=sys.stderr)


def main():
    try:
        # Parse command line arguments
        parser = argparse.ArgumentParser()
        parser.add_argument('--chat', action='store_true', help='Copy transcript to chat.json')
        args = parser.parse_args()
        
        # Read JSON input from stdin
        input_data = json.load(sys.stdin)

        # Extract required fields (for future use)
        # session_id = input_data.get("session_id", "")
        # stop_hook_active = input_data.get("stop_hook_active", False)

        # Ensure log directory exists
        log_dir = os.path.join(os.getcwd(), "logs")
        os.makedirs(log_dir, exist_ok=True)
        log_path = os.path.join(log_dir, "stop.json")

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
        
        # Handle --chat switch
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
                    chat_file = os.path.join(log_dir, 'chat.json')
                    with open(chat_file, 'w') as f:
                        json.dump(chat_data, f, indent=2)
                except Exception:
                    pass  # Fail silently

        # Handle response TTS for conversational content
        trigger_conversation_tts(input_data)

        # Announce completion via TTS
        announce_completion()

        sys.exit(0)

    except json.JSONDecodeError:
        # Handle JSON decode errors gracefully
        sys.exit(0)
    except Exception:
        # Handle any other errors gracefully
        sys.exit(0)


if __name__ == "__main__":
    main()
