# MAOS CLI Commands Reference

## Overview

MAOS provides hook commands that integrate with Claude Code. These commands are called automatically by Claude Code's hook system - users don't invoke them directly.

## Core Commands

### maos pre-tool-use

Executes before Claude Code tools run. Handles security validation and workspace preparation.

**Hook Configuration:**
```json
{
  "PreToolUse": [{
    "command": "maos pre-tool-use"
  }]
}
```

**Features:**
- Blocks dangerous `rm -rf` commands
- Prevents access to `.env` files
- Creates git worktrees for Task tool (agents)
- Enforces workspace isolation
- Updates session tracking

**Exit Codes:**
- `0`: Success, continue with tool
- `2`: Blocked operation, show error to user

### maos post-tool-use

Executes after Claude Code tools complete. Handles cleanup and state management.

**Hook Configuration:**
```json
{
  "PostToolUse": [{
    "command": "maos post-tool-use"
  }]
}
```

**Features:**
- Releases file locks
- Updates progress tracking
- Triggers worktree cleanup
- Logs operation metrics

### maos notify

Handles Claude Code notifications with optional TTS announcements.

**Hook Configuration:**
```json
{
  "Notification": [{
    "command": "maos notify"
  }]
}
```

**Features:**
- Multi-provider TTS support (ElevenLabs, OpenAI, macOS)
- Respects notification preferences
- Engineer name personalization

### maos stop

Executes when Claude Code sessions end.

**Hook Configuration:**
```json
{
  "Stop": [{
    "command": "maos stop"
  }]
}
```

**Features:**
- Session completion announcements
- Response TTS for conversations
- Transcript export with `--chat` flag
- Cleanup of temporary resources

### maos user-prompt-submit

Processes user prompts before Claude receives them.

**Hook Configuration:**
```json
{
  "UserPromptSubmit": [{
    "command": "maos user-prompt-submit"
  }]
}
```

**Features:**
- Prompt logging for audit trails
- Optional validation with `--validate`
- Can block inappropriate prompts
- Context injection capability

## Utility Commands

### maos --version

Shows the installed MAOS version.

```bash
maos --version
# Output: maos 0.1.0
```

### maos --help

Displays help information about available commands.

```bash
maos --help
```

## Hook Integration

All MAOS commands are designed to be called by Claude Code's hook system. They:

1. Read JSON input from stdin
2. Process according to command logic
3. Return appropriate exit codes
4. Write errors to stderr for user visibility

## Environment Variables

MAOS respects these environment variables:

- `ELEVENLABS_API_KEY`: Enable ElevenLabs TTS
- `OPENAI_API_KEY`: Enable OpenAI TTS
- `ANTHROPIC_API_KEY`: Enable Anthropic features
- `ENGINEER_NAME`: Personalize announcements
- `CLAUDE_AGENT_TYPE`: Set by MAOS for agent identification

## Performance

All commands target <10ms execution time to minimize overhead on Claude Code operations.

## Logging

Commands log to:
- `.maos/sessions/{session_id}/` - Session data
- `logs/` - Operation logs

## See Also

- [Installation Guide](./installation.md)
- [Configuration](./configuration.md)
- [Migration Guide](./migration.md)