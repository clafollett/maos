# Python to Rust Migration Guide

## Overview

This guide helps you transition from the current Python hook implementation to the new Rust CLI. The migration is designed to be seamless - just update your hook commands in `.claude/settings.json`.

## Migration Timeline

1. **Current State**: Python scripts in `.claude/hooks/`
2. **Transition Period**: Both Python and Rust available
3. **Future State**: Rust CLI only

## Quick Migration

The MAOS installer automatically updates your `.claude/settings.json` during installation. It intelligently:
- Finds existing Python hook commands
- Replaces them with Rust CLI equivalents
- Preserves any other hooks you have configured
- Creates backups before making changes

Manual migration is also possible if preferred:

### Before (Python)
```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "uv run \"$(git rev-parse --show-toplevel)/.claude/hooks/pre_tool_use.py\""
    }],
    "PostToolUse": [{
      "command": "uv run \"$(git rev-parse --show-toplevel)/.claude/hooks/post_tool_use.py\""
    }],
    "Notification": [{
      "command": "uv run \"$(git rev-parse --show-toplevel)/.claude/hooks/notification.py\""
    }],
    "Stop": [{
      "command": "uv run \"$(git rev-parse --show-toplevel)/.claude/hooks/stop.py\""
    }],
    "UserPromptSubmit": [{
      "command": "uv run \"$(git rev-parse --show-toplevel)/.claude/hooks/user_prompt_submit.py\""
    }]
  }
}
```

### After (Rust)
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

## Feature Parity

The Rust CLI maintains 100% feature parity with Python:

| Python Script | Rust Command | Features Preserved |
|--------------|--------------|-------------------|
| pre_tool_use.py | maos pre-tool-use | ✓ rm -rf blocking<br>✓ .env protection<br>✓ Worktree creation<br>✓ Session management |
| post_tool_use.py | maos post-tool-use | ✓ Lock release<br>✓ Progress tracking<br>✓ Cleanup triggers |
| notification.py | maos notify | ✓ Multi-provider TTS<br>✓ Engineer name |
| stop.py | maos stop | ✓ Completion TTS<br>✓ Response TTS<br>✓ Chat export |
| user_prompt_submit.py | maos user-prompt-submit | ✓ Prompt logging<br>✓ Validation |

## Configuration Migration

### Environment Variables
No changes needed - same environment variables work:
- `ELEVENLABS_API_KEY`
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`
- `ENGINEER_NAME`

### Project Configuration
If you have `.claude/config.json`, it works with both versions.

## Benefits of Migration

### Performance
- **Python**: 50-200ms per hook execution
- **Rust**: <10ms per hook execution
- **Impact**: 10-20x faster operations

### Reliability
- No Python interpreter failures
- No dependency conflicts
- Single binary deployment

### Security
- Compiled validation rules
- No script tampering
- Consistent behavior

## Testing the Migration

1. Install MAOS CLI (see [Installation Guide](./installation.md))
2. Run verification:
   ```bash
   maos --version
   ```
3. Update one hook at a time in `.claude/settings.json`
4. Test each hook works correctly
5. Remove Python scripts when satisfied

## Rollback Plan

If you need to rollback:
1. Simply revert `.claude/settings.json` to Python commands
2. Python scripts remain in place during transition
3. No data or state changes needed

## Troubleshooting

### Command Not Found
Ensure MAOS is in your PATH:
```bash
which maos
```

### Different Behavior
Check debug logs:
```bash
export MAOS_DEBUG=1
```

### Missing Features
Verify version:
```bash
maos --version  # Should be 0.1.0 or higher
```

## Cleanup After Migration

Once fully migrated:
1. Delete `.claude/hooks/` directory
2. Remove Python dependencies
3. Uninstall `uv` if not needed elsewhere

## Support

- [GitHub Issues](https://github.com/clafollett/maos/issues)
- [Documentation](../README.md)
- [Commands Reference](./commands.md)