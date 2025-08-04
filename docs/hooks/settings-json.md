# Claude Code Hook Configuration

## Overview

MAOS integrates with Claude Code through the hook system configured in `.claude/settings.json`. This file tells Claude Code to run MAOS commands at specific points during tool execution.

## Basic Configuration

### Minimal settings.json

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use"
    }],
    "PostToolUse": [{
      "command": "maos post-tool-use"
    }]
  }
}
```

### Full Configuration

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use",
      "config": {
        "exitOnFailure": true,
        "timeout": 5000
      }
    }],
    "PostToolUse": [{
      "command": "maos post-tool-use",
      "config": {
        "exitOnFailure": false,
        "timeout": 3000
      }
    }],
    "Notification": [{
      "command": "maos notify",
      "config": {
        "exitOnFailure": false
      }
    }],
    "Stop": [{
      "command": "maos stop",
      "config": {
        "exitOnFailure": false
      }
    }],
    "UserPromptSubmit": [{
      "command": "maos user-prompt-submit",
      "config": {
        "exitOnFailure": false
      }
    }]
  }
}
```

## Hook Types

### PreToolUse

Runs before any Claude Code tool executes:

```json
{
  "PreToolUse": [{
    "command": "maos pre-tool-use"
  }]
}
```

**Purpose:**
- Security validation (blocks dangerous commands)
- Workspace preparation (creates worktrees)
- Session management
- Resource allocation

**Exit Codes:**
- `0`: Continue with tool execution
- `2`: Block tool execution and show error

### PostToolUse

Runs after tool execution completes:

```json
{
  "PostToolUse": [{
    "command": "maos post-tool-use"
  }]
}
```

**Purpose:**
- Release locks
- Update progress
- Log metrics
- Trigger cleanup

### Notification

Handles Claude Code notifications:

```json
{
  "Notification": [{
    "command": "maos notify"
  }]
}
```

**Purpose:**
- TTS announcements
- Status updates
- Error alerts
- Progress notifications

### Stop

Executes when session ends:

```json
{
  "Stop": [{
    "command": "maos stop"
  }]
}
```

**Purpose:**
- Session cleanup
- Final announcements
- Export transcripts
- Cleanup worktrees

### UserPromptSubmit

Processes user prompts before Claude sees them:

```json
{
  "UserPromptSubmit": [{
    "command": "maos user-prompt-submit"
  }]
}
```

**Purpose:**
- Prompt logging
- Content validation
- Context injection
- Audit trail

## Configuration Options

### Hook-Level Options

```json
{
  "command": "maos pre-tool-use",
  "config": {
    "exitOnFailure": true,      // Stop on hook failure
    "timeout": 5000,            // Timeout in milliseconds
    "retries": 0,               // Retry attempts
    "continueOnError": false    // Continue despite errors
  }
}
```

### Multiple Hooks

You can chain multiple commands:

```json
{
  "PreToolUse": [
    {
      "command": "maos pre-tool-use"
    },
    {
      "command": "custom-validator"
    }
  ]
}
```

## Data Flow

### Input (via stdin)

Hooks receive JSON data from Claude Code:

```json
{
  "tool": "Bash",
  "params": {
    "command": "git status"
  },
  "context": {
    "session_id": "sess-123",
    "agent_type": "backend-engineer"
  }
}
```

### Output (via exit code)

- `0`: Success, continue
- `2`: Block operation (PreToolUse only)
- Other: Error condition

### Error Messages (via stderr)

```
Security violation: Blocked dangerous command
Use a more specific path instead of 'rm -rf /'
```

## Migration from Python Hooks

### Current Python Configuration

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "uv run \"$(git rev-parse --show-toplevel)/.claude/hooks/pre_tool_use.py\""
    }]
  }
}
```

### Migrated to MAOS CLI

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use"
    }]
  }
}
```

## Platform-Specific Configurations

### Windows

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos.exe pre-tool-use"
    }]
  }
}
```

### Using Full Paths

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "/usr/local/bin/maos pre-tool-use"
    }]
  }
}
```

### Using NPX

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "npx @maos/cli pre-tool-use"
    }]
  }
}
```

## Environment Variables

Hooks inherit environment from Claude Code plus:

```bash
# Set by Claude Code
CLAUDE_AGENT_TYPE="backend-engineer"
CLAUDE_SESSION_ID="sess-123"

# MAOS-specific
MAOS_DEBUG=1
MAOS_LOG_LEVEL=debug
```

## Debugging Hooks

### Enable Debug Output

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "MAOS_DEBUG=1 maos pre-tool-use"
    }]
  }
}
```

### Log to File

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use 2>>/tmp/maos-debug.log"
    }]
  }
}
```

### Test Hook Manually

```bash
# Simulate hook input
echo '{"tool":"Bash","params":{"command":"ls"}}' | maos pre-tool-use
```

## Best Practices

### 1. Keep It Simple

Start with minimal configuration:
```json
{
  "hooks": {
    "PreToolUse": [{"command": "maos pre-tool-use"}],
    "PostToolUse": [{"command": "maos post-tool-use"}]
  }
}
```

### 2. Set Appropriate Timeouts

```json
{
  "config": {
    "timeout": 5000  // 5 seconds is plenty for MAOS
  }
}
```

### 3. Handle Failures Gracefully

```json
{
  "PostToolUse": [{
    "command": "maos post-tool-use",
    "config": {
      "exitOnFailure": false  // Don't block on cleanup
    }
  }]
}
```

### 4. Version Control

Always commit `.claude/settings.json` to your repository:
```bash
git add .claude/settings.json
git commit -m "Configure MAOS hooks"
```

## Troubleshooting

### Hooks Not Running

1. Check file location: `.claude/settings.json` (not `.claude.json`)
2. Validate JSON syntax
3. Ensure MAOS is in PATH
4. Check Claude Code version supports hooks

### Command Not Found

```json
// Use full path if needed
{
  "command": "/usr/local/bin/maos pre-tool-use"
}
```

### Permission Denied

```bash
# Make MAOS executable
chmod +x /usr/local/bin/maos
```

### Slow Hook Execution

Check for:
- Network calls in hooks
- Large log files
- Disk I/O issues

## Security Considerations

1. **Validate Hook Commands**: Only use trusted binaries
2. **Avoid Shell Expansion**: Use direct commands
3. **Limit Permissions**: Hooks run with your user permissions
4. **Audit Changes**: Review settings.json changes in PRs

## Related Documentation

- [Commands Reference](../cli/commands.md) - All MAOS commands
- [Migration Guide](../cli/migration.md) - Moving from Python
- [Examples](./examples.md) - Real-world configurations