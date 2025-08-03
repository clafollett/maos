#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# ///

import json
from pathlib import Path

def get_config_path():
    """Get the path to the Claude hooks configuration file."""
    # Look for config in .claude directory relative to current working directory
    config_path = Path.cwd() / ".claude" / "config.json"
    if config_path.exists():
        return config_path
    
    # Fallback to script directory
    script_dir = Path(__file__).parent.parent
    config_path = script_dir / "config.json"
    if config_path.exists():
        return config_path
    
    return None

def load_config():
    """Load configuration from config.json with fallback to environment variables."""
    import platform
    
    # Platform-specific defaults
    is_macos = platform.system() == "Darwin"
    default_provider = "macos" if is_macos else "pyttsx3"
    default_macos_voice = "Alex"  # Alex is the default macOS system voice
    
    default_config = {
        "tts": {
            "enabled": True,
            "provider": default_provider,
            "text_length_limit": 2000,
            "timeout": 120,
            "voices": {
                "macos": {
                    "voice": default_macos_voice,
                    "rate": 190,
                    "quality": 127
                },
                "pyttsx3": {
                    "voice": "default",
                    "rate": 190,
                    "volume": 0.9
                },
                "elevenlabs": {
                    # Charlie
                    "voice_id": "IKne3meq5aSn9XLyUdCD",
                    "model": "eleven_turbo_v2_5",
                    "output_format": "mp3_44100_128"
                }
            },
            "responses": {
                "enabled": True
            },
            "completion": {
                "enabled": False
            },
            "notifications": {
                "enabled": True
            }
        },
        "engineer": {
            "name": ""
        }
    }
    
    config_path = get_config_path()
    if config_path and config_path.exists():
        try:
            with open(config_path, 'r') as f:
                config = json.load(f)
                # Merge with defaults to handle missing keys
                return merge_configs(default_config, config)
        except (json.JSONDecodeError, FileNotFoundError):
            pass
    
    # Return default config if no file found
    return default_config

def merge_configs(default, user):
    """Recursively merge user config with default config."""
    result = default.copy()
    for key, value in user.items():
        if key in result and isinstance(result[key], dict) and isinstance(value, dict):
            result[key] = merge_configs(result[key], value)
        else:
            result[key] = value
    return result

def save_config(config):
    """Save configuration to config.json."""
    config_path = get_config_path()
    if not config_path:
        # Create config in .claude directory
        claude_dir = Path.cwd() / ".claude"
        claude_dir.mkdir(exist_ok=True)
        config_path = claude_dir / "config.json"
    
    try:
        with open(config_path, 'w') as f:
            json.dump(config, f, indent=2)
        return True
    except (OSError, json.JSONDecodeError):
        return False

def get_tts_config():
    """Get TTS-specific configuration."""
    config = load_config()
    return config.get('tts', {})

def is_tts_enabled():
    """Check if TTS master switch is enabled."""
    return get_tts_config().get('enabled', True)

def is_response_tts_enabled():
    """Check if response TTS is enabled (master switch AND responses.enabled)."""
    tts_config = get_tts_config()
    master_enabled = tts_config.get('enabled', True)
    responses_enabled = tts_config.get('responses', {}).get('enabled', True)
    return master_enabled and responses_enabled

def is_completion_tts_enabled():
    """Check if completion TTS is enabled (master switch AND completion.enabled)."""
    tts_config = get_tts_config()
    master_enabled = tts_config.get('enabled', True)
    completion_enabled = tts_config.get('completion', {}).get('enabled', False)
    return master_enabled and completion_enabled

def is_notification_tts_enabled():
    """Check if notification TTS is enabled (master switch AND notifications.enabled)."""
    tts_config = get_tts_config()
    master_enabled = tts_config.get('enabled', True)
    notifications_enabled = tts_config.get('notifications', {}).get('enabled', True)
    return master_enabled and notifications_enabled

def get_tts_provider():
    """Get the preferred TTS provider (macos, elevenlabs) - applies globally."""
    return get_tts_config().get('provider', 'macos')

def get_active_tts_provider():
    """Get the active TTS provider based on availability and config."""
    import os  # Local import to avoid unused import warning
    provider = get_tts_provider()
    
    # Check if ElevenLabs is configured and available
    if provider == 'elevenlabs':
        if os.getenv('ELEVENLABS_API_KEY'):
            return 'elevenlabs'
        # Fallback to macOS if ElevenLabs not available
        return 'macos'
    
    return provider

def get_text_length_limit():
    """Get the maximum text length for TTS processing."""
    return get_tts_config().get('text_length_limit', 2000)

def get_tts_timeout():
    """Get the timeout in seconds for TTS operations."""
    return get_tts_config().get('timeout', 120)

def get_voice_for_provider(provider):
    """Get the voice setting for a specific provider."""
    voices = get_tts_config().get('voices', {})
    return voices.get(provider, '')

def get_macos_config():
    """Get macOS TTS configuration."""
    voices = get_tts_config().get('voices', {})
    macos_config = voices.get('macos', {})
    
    return {
        'voice': macos_config.get('voice', 'Lee (Premium)'),
        'rate': macos_config.get('rate', 200),
        'quality': macos_config.get('quality', 127)
    }

def get_elevenlabs_config():
    """Get ElevenLabs configuration."""
    voices = get_tts_config().get('voices', {})
    elevenlabs_config = voices.get('elevenlabs', {})
    
    # Return defaults if not configured
    return {
        'voice_id': elevenlabs_config.get('voice_id', 'FNMROvc7ZdHldafWFMqC'),
        'model': elevenlabs_config.get('model', 'eleven_turbo_v2_5'),
        'output_format': elevenlabs_config.get('output_format', 'mp3_44100_128')
    }

def get_engineer_name():
    """Get the engineer name from config."""
    config = load_config()
    return config.get('engineer', {}).get('name', '')

if __name__ == "__main__":
    # CLI for testing config
    import sys
    if len(sys.argv) > 1:
        if sys.argv[1] == "show":
            config = load_config()
            print(json.dumps(config, indent=2))
        elif sys.argv[1] == "tts":
            print(f"TTS enabled: {is_tts_enabled()}")
            print(f"TTS provider: {get_tts_provider()}")
            print(f"macOS voice: {get_voice_for_provider('macos')}")
            print(f"ElevenLabs voice: {get_voice_for_provider('elevenlabs')}")
    else:
        print("Usage: python config.py [show|tts]")