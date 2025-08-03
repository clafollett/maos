# MAOS Configuration Guide

## Overview

MAOS follows a zero-configuration philosophy - it works out of the box with sensible defaults. Configuration is only needed for optional features like TTS providers.

## Configuration Hierarchy

MAOS checks for configuration in this order:
1. Environment variables (highest priority)
2. `.claude/config.json` (project-specific)
3. Built-in defaults (lowest priority)

## Environment Variables

### TTS Provider Keys

Enable text-to-speech providers by setting API keys:

```bash
# ElevenLabs (highest quality)
export ELEVENLABS_API_KEY="your-api-key"

# OpenAI TTS
export OPENAI_API_KEY="your-api-key"

# Anthropic (for completion messages)
export ANTHROPIC_API_KEY="your-api-key"
```

### Personalization

```bash
# Add your name to announcements
export ENGINEER_NAME="Alice"
```

### MAOS Internal

These are set automatically by MAOS:

```bash
# Agent type identification
export CLAUDE_AGENT_TYPE="backend-engineer"
```

## Project Configuration

Create `.claude/config.json` for project-specific settings:

```json
{
  "maos": {
    "tts": {
      "enabled": true,
      "provider": "elevenlabs",
      "voice": "adam"
    },
    "security": {
      "block_env_access": true,
      "allowed_paths": [
        ".env.sample",
        ".env.example"
      ]
    },
    "worktree": {
      "cleanup_on_exit": true,
      "branch_prefix": "agent/session"
    }
  }
}
```

## Hook Configuration

Configure MAOS hooks in `.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use"
    }],
    "PostToolUse": [{
      "command": "maos post-tool-use"
    }],
    "Notification": [{
      "command": "maos notify"
    }],
    "Stop": [{
      "command": "maos stop"
    }],
    "UserPromptSubmit": [{
      "command": "maos user-prompt-submit"
    }]
  }
}
```

## Feature Flags

### TTS Configuration

Control text-to-speech behavior:

```json
{
  "maos": {
    "tts": {
      "enabled": true,
      "response_tts": false,  // Read AI responses aloud
      "completion_tts": true, // Announce task completion
      "notification_tts": true // Announce notifications
    }
  }
}
```

### Security Settings

Customize security behavior:

```json
{
  "maos": {
    "security": {
      "block_rm_rf": true,    // Block dangerous rm commands
      "block_env_access": true, // Block .env file access
      "custom_blocks": [      // Additional patterns to block
        "password",
        "secret"
      ]
    }
  }
}
```

## Default Configuration

MAOS uses these defaults when no configuration is provided:

```json
{
  "maos": {
    "tts": {
      "enabled": true,
      "provider": "auto",  // Auto-detect based on API keys
      "response_tts": false,
      "completion_tts": true,
      "notification_tts": false
    },
    "security": {
      "block_rm_rf": true,
      "block_env_access": true
    },
    "worktree": {
      "cleanup_on_exit": true,
      "branch_prefix": "agent/session",
      "lazy_creation": true
    },
    "performance": {
      "target_latency_ms": 10,
      "log_slow_operations": true
    }
  }
}
```

## Platform-Specific Settings

MAOS automatically detects and applies platform-specific settings:

### macOS
- Uses `say` command as TTS fallback
- Optimized for APFS file system

### Linux
- Uses `pyttsx3` as TTS fallback
- Handles various file system types

### Windows
- Uses Windows Speech API as fallback
- Handles path separators correctly

## Debugging Configuration

Enable debug logging:

```bash
export MAOS_DEBUG=1
export MAOS_LOG_LEVEL=debug
```

View current configuration:

```bash
maos config show
```

## Best Practices

1. **Minimal Configuration**: Only configure what you need
2. **Environment Variables**: Use for sensitive data (API keys)
3. **Project Config**: Use for project-specific settings
4. **Version Control**: Commit `.claude/config.json`, not API keys

## See Also

- [Installation Guide](./installation.md)
- [Commands Reference](./commands.md)
- [Migration Guide](./migration.md)