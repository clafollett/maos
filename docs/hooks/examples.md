# MAOS Hook Configuration Examples

## Basic Configurations

### Minimal Setup

For users who just want basic orchestration:

```json
{
  "hooks": {
    "PreToolUse": [{"command": "maos pre-tool-use"}],
    "PostToolUse": [{"command": "maos post-tool-use"}]
  }
}
```

### Full Feature Set

All MAOS features enabled:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use",
      "config": {"exitOnFailure": true}
    }],
    "PostToolUse": [{
      "command": "maos post-tool-use",
      "config": {"exitOnFailure": false}
    }],
    "Notification": [{
      "command": "maos notify",
      "config": {"exitOnFailure": false}
    }],
    "Stop": [{
      "command": "maos stop",
      "config": {"exitOnFailure": false}
    }],
    "UserPromptSubmit": [{
      "command": "maos user-prompt-submit",
      "config": {"exitOnFailure": false}
    }]
  }
}
```

## Development Environments

### Local Development

With debug logging and fast timeouts:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "MAOS_DEBUG=1 MAOS_LOG_LEVEL=debug maos pre-tool-use",
      "config": {
        "timeout": 2000,
        "exitOnFailure": true
      }
    }],
    "PostToolUse": [{
      "command": "MAOS_DEBUG=1 maos post-tool-use",
      "config": {
        "timeout": 1000,
        "exitOnFailure": false
      }
    }]
  }
}
```

### CI/CD Environment

Stricter settings for automated environments:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use",
      "config": {
        "exitOnFailure": true,
        "timeout": 10000
      }
    }],
    "PostToolUse": [{
      "command": "maos post-tool-use",
      "config": {
        "exitOnFailure": true,
        "timeout": 5000
      }
    }]
  }
}
```

## Platform-Specific Examples

### macOS with TTS

Using macOS say command fallback:

```json
{
  "hooks": {
    "PreToolUse": [{"command": "maos pre-tool-use"}],
    "PostToolUse": [{"command": "maos post-tool-use"}],
    "Notification": [{
      "command": "maos notify",
      "config": {"exitOnFailure": false}
    }],
    "Stop": [{
      "command": "maos stop --voice=Samantha",
      "config": {"exitOnFailure": false}
    }]
  }
}
```

### Windows Configuration

With Windows-specific paths:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "C:\\Program Files\\MAOS\\maos.exe pre-tool-use"
    }],
    "PostToolUse": [{
      "command": "C:\\Program Files\\MAOS\\maos.exe post-tool-use"
    }]
  }
}
```

### Linux with Custom Installation

For non-standard installations:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "/opt/maos/bin/maos pre-tool-use"
    }],
    "PostToolUse": [{
      "command": "/opt/maos/bin/maos post-tool-use"
    }]
  }
}
```

## Project-Specific Configurations

### Web Development Project

Enhanced security for web projects:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --block-patterns='*.key,*.pem,*.env.production'",
      "config": {"exitOnFailure": true}
    }],
    "PostToolUse": [{"command": "maos post-tool-use"}]
  }
}
```

### Data Science Project

With notebook support:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --allow-notebooks",
      "config": {"exitOnFailure": true}
    }],
    "PostToolUse": [{
      "command": "maos post-tool-use --cleanup-outputs"
    }]
  }
}
```

### Enterprise Project

With audit logging and compliance:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --audit-log=/var/log/maos/audit.log",
      "config": {
        "exitOnFailure": true,
        "timeout": 5000
      }
    }],
    "PostToolUse": [{
      "command": "maos post-tool-use --audit-log=/var/log/maos/audit.log"
    }],
    "UserPromptSubmit": [{
      "command": "maos user-prompt-submit --compliance-mode",
      "config": {"exitOnFailure": true}
    }]
  }
}
```

## Advanced Configurations

### Multi-Tool Pipeline

Combining MAOS with other tools:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "command": "security-scanner --pre-check"
      },
      {
        "command": "maos pre-tool-use"
      }
    ],
    "PostToolUse": [
      {
        "command": "maos post-tool-use"
      },
      {
        "command": "git-auto-commit --if-clean"
      }
    ]
  }
}
```

### Custom Environment Variables

With project-specific settings:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "MAOS_PROJECT=myapp MAOS_TEAM=backend maos pre-tool-use"
    }],
    "Stop": [{
      "command": "SLACK_WEBHOOK=https://... maos stop --notify-slack"
    }]
  }
}
```

### Performance Monitoring

With detailed metrics:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --metrics-export=/tmp/maos-metrics.json"
    }],
    "PostToolUse": [{
      "command": "maos post-tool-use --metrics-export=/tmp/maos-metrics.json"
    }],
    "Stop": [{
      "command": "maos stop && maos export-metrics --format=prometheus"
    }]
  }
}
```

## Migration Examples

### From Python Hooks (Current)

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "uv run \"$(git rev-parse --show-toplevel)/.claude/hooks/pre_tool_use.py\""
    }],
    "PostToolUse": [{
      "command": "uv run \"$(git rev-parse --show-toplevel)/.claude/hooks/post_tool_use.py\""
    }]
  }
}
```

### To MAOS CLI (Simple)

```json
{
  "hooks": {
    "PreToolUse": [{"command": "maos pre-tool-use"}],
    "PostToolUse": [{"command": "maos post-tool-use"}]
  }
}
```

### Gradual Migration

Running both during transition:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "command": "maos pre-tool-use || uv run \"$(git rev-parse --show-toplevel)/.claude/hooks/pre_tool_use.py\""
      }
    ]
  }
}
```

## Team Configurations

### Frontend Team

Focus on UI/UX safety:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --protect-patterns='*.css,*.scss,design-system/*'"
    }],
    "Notification": [{
      "command": "maos notify --voice=nova"
    }]
  }
}
```

### Backend Team

API and database protection:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --protect-patterns='migrations/*,*.sql,api/v1/*'"
    }],
    "PostToolUse": [{
      "command": "maos post-tool-use --validate-migrations"
    }]
  }
}
```

### DevOps Team

Infrastructure safety:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --terraform-mode --dry-run-first"
    }],
    "Stop": [{
      "command": "maos stop --export-tfplan"
    }]
  }
}
```

## Debugging Configurations

### Maximum Verbosity

For troubleshooting issues:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "MAOS_DEBUG=1 MAOS_LOG_LEVEL=trace RUST_BACKTRACE=1 maos pre-tool-use 2>>/tmp/maos-debug.log"
    }]
  }
}
```

### Profiling Mode

For performance analysis:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --profile --timing-output=/tmp/maos-timing.json"
    }]
  }
}
```

## Special Use Cases

### Monorepo Configuration

For large monorepos:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --sparse-worktrees --max-size=100MB"
    }]
  }
}
```

### Educational Environment

With safety guards for students:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --training-mode --extra-confirmations"
    }],
    "UserPromptSubmit": [{
      "command": "maos user-prompt-submit --learning-mode"
    }]
  }
}
```

### Research Projects

With experiment tracking:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --experiment-tracking"
    }],
    "Stop": [{
      "command": "maos stop --export-experiment-results"
    }]
  }
}
```

## Best Practices Examples

### Production-Ready

Balanced for reliability and features:

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
    "Stop": [{
      "command": "maos stop",
      "config": {
        "exitOnFailure": false,
        "timeout": 10000
      }
    }]
  }
}
```

### Minimal Overhead

For performance-critical environments:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use --fast-mode",
      "config": {"timeout": 1000}
    }]
  }
}
```

## Troubleshooting Examples

### When MAOS Isn't in PATH

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "$(which maos || echo /usr/local/bin/maos) pre-tool-use"
    }]
  }
}
```

### With Fallback Commands

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use || echo 'MAOS not available, continuing without protection'"
    }]
  }
}
```

## Related Documentation

- [Settings.json Reference](./settings-json.md) - Detailed configuration options
- [Commands Reference](../cli/commands.md) - All available commands
- [Migration Guide](../cli/migration.md) - Moving from Python hooks