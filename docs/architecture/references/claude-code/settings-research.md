# Claude Code Settings Research for MAOS

## Executive Summary

Claude Code provides a comprehensive, hierarchical settings system designed for enterprise-grade multi-agent orchestration. The configuration framework supports fine-grained permission controls, multiple authentication providers, extensive hooks for automation, and robust security boundaries. Key features include:

- **Hierarchical Settings**: Enterprise > CLI > Local Project > Shared Project > User
- **Granular Permissions**: Tool-specific rules with pattern matching
- **Multiple Auth Providers**: Anthropic API, AWS Bedrock, Google Vertex AI
- **Extensibility**: Hooks system for custom automation and control
- **Security-First Design**: Folder restrictions, command blocklists, isolated contexts
- **Multi-Profile Support**: Different configuration contexts via settings files
- **Performance Controls**: MCP timeouts, streaming options, batch processing

## Settings Hierarchy and Precedence

### Configuration Precedence (Highest to Lowest)
1. **Enterprise Policies** - `/Library/Application Support/ClaudeCode/managed-settings.json` (macOS)
2. **Command Line Arguments** - Override any configured settings
3. **Local Project Settings** - `.claude/settings.local.json` (git-ignored)
4. **Shared Project Settings** - `.claude/settings.json` (version controlled)
5. **User Settings** - `~/.claude/settings.json`

### Configuration File Locations

```bash
# User settings
~/.claude/settings.json

# Project settings (shared with team)
.claude/settings.json

# Project settings (personal, git-ignored)
.claude/settings.local.json

# Enterprise managed settings
# macOS
/Library/Application Support/ClaudeCode/managed-settings.json
# Linux/Windows
/etc/claude-code/managed-settings.json
```

## Permission Management for Agents

### Permission Modes

1. **"default"** - Prompts for permission on first use of each tool
2. **"acceptEdits"** - Automatically accepts file edit permissions for the session
3. **"plan"** - Claude can analyze but not modify files or execute commands
4. **"bypassPermissions"** - Skips all permission prompts (requires safe environment)

### Permission Rule Syntax

```json
{
  "permissions": {
    "allow": [
      "Bash(npm run build)",          // Exact match
      "Bash(npm run test:*)",         // Prefix match with wildcard
      "Edit(docs/**)",                // Directory-specific edits
      "WebFetch(domain:example.com)"  // Domain-specific web fetches
    ],
    "deny": [
      "Bash(curl:*)",                 // Block all curl commands
      "Bash(rm -rf*)",                // Block dangerous deletions
      "Edit(/etc/**)"                 // Block system file edits
    ],
    "defaultMode": "acceptEdits",
    "disableBypassPermissionsMode": true,
    "additionalDirectories": [
      "/path/to/shared/resources",
      "/path/to/library/code"
    ]
  }
}
```

### Tool-Specific Permission Examples

```json
{
  "permissions": {
    "allow": [
      // Bash tool permissions
      "Bash(git diff:*)",             // Allow all git diff commands
      "Bash(npm run lint)",           // Allow specific npm script
      "Bash(pytest tests/unit/*)",    // Allow unit tests only
      
      // Edit tool permissions
      "Edit(src/**/*.js)",            // Allow JS edits in src
      "Edit(*.md)",                   // Allow markdown edits
      
      // WebFetch permissions
      "WebFetch(domain:api.example.com)", // Allow specific API domain
      "WebFetch(domain:*.trusted.com)",   // Allow subdomain pattern
      
      // Agent tool permissions
      "Agent",                        // Allow all agent operations
      
      // Read/Write permissions
      "Read(/data/public/**)",        // Allow reading public data
      "Write(logs/**)"                // Allow writing to logs
    ]
  }
}
```

## Environment Configuration

### Core Environment Variables

```bash
# Authentication
export ANTHROPIC_API_KEY="sk-ant-..."

# Provider Selection
export CLAUDE_CODE_USE_BEDROCK=1      # Use AWS Bedrock
export CLAUDE_CODE_USE_VERTEX=1       # Use Google Vertex AI

# AWS Bedrock Configuration
export AWS_REGION=us-east-1
export AWS_PROFILE=production

# Google Vertex AI Configuration
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/credentials.json"
export GOOGLE_CLOUD_PROJECT="my-project-id"

# Gateway Configuration
export ANTHROPIC_BASE_URL="https://gateway.company.com/anthropic"
export ANTHROPIC_BEDROCK_BASE_URL="https://gateway.company.com/bedrock"
export ANTHROPIC_VERTEX_BASE_URL="https://gateway.company.com/vertex"

# Performance and Debugging
export MCP_TIMEOUT=10000              # 10-second MCP server timeout
export ANTHROPIC_LOG=debug            # Enable debug logging
export DISABLE_TELEMETRY=1            # Opt-out of telemetry

# Security
export CLAUDE_CODE_SKIP_BEDROCK_AUTH=1  # When gateway handles AWS auth
```

### Environment Variables in Settings

```json
{
  "env": {
    "CLAUDE_CODE_ENABLE_TELEMETRY": "1",
    "OTEL_METRICS_EXPORTER": "otlp",
    "NODE_ENV": "production",
    "PYTHONPATH": "/usr/local/lib/python3.9/site-packages",
    "CUSTOM_AGENT_VAR": "value"
  }
}
```

## Hooks and Automation

### Hook Types

1. **PreToolUse** - Runs before tool execution, can block
2. **PostToolUse** - Runs after successful tool completion
3. **Notification** - Triggers on system notifications
4. **Stop** - Runs when main agent finishes
5. **SubagentStop** - Runs when subagent completes

### Hook Configuration Examples

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash\\(.*\\)",
        "hooks": [
          {
            "type": "command",
            "command": "/usr/local/bin/audit-command"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Edit\\(.*\\.py\\)",
        "hooks": [
          {
            "type": "command",
            "command": "black --check ${CLAUDE_LAST_EDITED_FILE}"
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "echo 'Session complete' | slack-notify"
          }
        ]
      }
    ]
  }
}
```

### Advanced Hook Use Cases

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash\\(docker.*\\)",
        "hooks": [
          {
            "type": "command",
            "command": "check-docker-permissions.sh",
            "description": "Verify Docker command safety"
          }
        ]
      },
      {
        "matcher": "WebFetch\\(.*\\)",
        "hooks": [
          {
            "type": "command",
            "command": "validate-url-allowlist.py ${CLAUDE_TOOL_ARGS}",
            "description": "Check URL against allowlist"
          }
        ]
      }
    ]
  }
}
```

## Security Best Practices

### Folder Access Restrictions
- Claude Code can only access the folder where it was started and subfolders
- Cannot traverse to parent directories
- Additional directories must be explicitly allowed

### Command Security
- Context-aware analysis detects harmful instructions
- Input sanitization prevents command injection
- Command blocklist blocks risky operations
- Network requests require approval
- Isolated context windows for web fetches

### Enterprise Security Configuration

```json
{
  "security": {
    "permissions": {
      "defaultMode": "plan",
      "disableBypassPermissionsMode": true,
      "deny": [
        "Bash(sudo*)",
        "Bash(su *)",
        "Bash(chmod*)",
        "Bash(chown*)",
        "Edit(/etc/**)",
        "Edit(/usr/**)",
        "Edit(/var/**)"
      ]
    },
    "additionalDirectories": [],
    "hooks": {
      "PreToolUse": [
        {
          "matcher": ".*",
          "hooks": [
            {
              "type": "command",
              "command": "/opt/security/audit-tool-use.sh"
            }
          ]
        }
      ]
    }
  }
}
```

## Performance Optimization Settings

### CLI Performance Flags

```bash
# Limit agent turns for faster completion
claude --max-turns 5 "Quick task"

# Use streaming for real-time output
claude --output-format stream-json "Generate report"

# Skip interactive mode for automation
claude --print "Check syntax" --input-format json

# Verbose logging for debugging
claude --verbose "Debug this issue"
```

### MCP Server Configuration

```json
{
  "mcpServers": {
    "custom-tools": {
      "command": "node",
      "args": ["/path/to/mcp-server.js"],
      "env": {
        "MCP_TIMEOUT": "15000",
        "TOOL_CACHE_SIZE": "100MB"
      }
    }
  }
}
```

### Resource Management

```json
{
  "performance": {
    "maxConcurrentTools": 5,
    "toolTimeout": 30000,
    "memoryLimit": "2GB",
    "cpuThrottle": 0.8
  }
}
```

## Multi-Profile Management

### Profile Structure

```bash
# Development profile
~/.claude/profiles/development/settings.json

# Production profile
~/.claude/profiles/production/settings.json

# Testing profile
~/.claude/profiles/testing/settings.json
```

### Profile Switching

```bash
# Using environment variable
export CLAUDE_PROFILE=production
claude "Deploy application"

# Using CLI flag
claude --profile development "Run tests"

# Profile-specific settings
{
  "profiles": {
    "development": {
      "permissions": {
        "defaultMode": "bypassPermissions"
      },
      "env": {
        "NODE_ENV": "development"
      }
    },
    "production": {
      "permissions": {
        "defaultMode": "plan",
        "deny": ["Bash(rm*)", "Bash(delete*)"]
      },
      "env": {
        "NODE_ENV": "production"
      }
    }
  }
}
```

## Code Examples

### Complete Multi-Agent Configuration

```json
{
  "agents": {
    "researcher": {
      "permissions": {
        "allow": ["Read", "WebFetch", "WebSearch"],
        "deny": ["Edit", "Write", "Bash"]
      },
      "env": {
        "AGENT_ROLE": "researcher",
        "MAX_SEARCH_RESULTS": "20"
      }
    },
    "developer": {
      "permissions": {
        "allow": ["Read", "Edit", "Write", "Bash(npm*)", "Bash(git*)"],
        "deny": ["Bash(rm -rf*)", "Bash(sudo*)"]
      },
      "env": {
        "AGENT_ROLE": "developer",
        "AUTO_FORMAT": "true"
      }
    },
    "reviewer": {
      "permissions": {
        "allow": ["Read", "Bash(git diff*)", "Agent"],
        "deny": ["Edit", "Write"]
      },
      "env": {
        "AGENT_ROLE": "reviewer"
      }
    }
  }
}
```

### SDK Integration Example

```typescript
import { ClaudeCode } from '@anthropic-ai/claude-code-sdk';

const claudeCode = new ClaudeCode({
  apiKey: process.env.ANTHROPIC_API_KEY,
  settings: {
    permissions: {
      allow: ['Read', 'Edit(src/**)', 'Bash(npm test)'],
      defaultMode: 'acceptEdits'
    },
    hooks: {
      PostToolUse: [{
        matcher: 'Edit\\(.*\\.ts\\)',
        command: 'npm run lint:fix'
      }]
    }
  },
  maxTurns: 10,
  abortController: new AbortController()
});

// Execute with custom configuration
const result = await claudeCode.sendMessage({
  text: "Implement the user authentication module",
  settings: {
    env: {
      DATABASE_URL: process.env.DATABASE_URL,
      JWT_SECRET: process.env.JWT_SECRET
    }
  }
});
```

### Enterprise Deployment Script

```bash
#!/bin/bash
# deploy-claude-code-enterprise.sh

# Create managed settings directory
sudo mkdir -p "/Library/Application Support/ClaudeCode"

# Deploy enterprise configuration
sudo cat > "/Library/Application Support/ClaudeCode/managed-settings.json" << 'EOF'
{
  "permissions": {
    "defaultMode": "plan",
    "disableBypassPermissionsMode": true,
    "deny": [
      "Bash(sudo*)",
      "Bash(su *)",
      "Edit(/etc/**)",
      "Edit(/System/**)"
    ],
    "allow": [
      "Bash(git*)",
      "Bash(npm*)",
      "Bash(python -m pytest*)"
    ]
  },
  "env": {
    "COMPANY_PROXY": "https://proxy.company.com:8080",
    "ANTHROPIC_BASE_URL": "https://api-gateway.company.com/anthropic"
  },
  "hooks": {
    "PreToolUse": [{
      "matcher": ".*",
      "hooks": [{
        "type": "command",
        "command": "/opt/company/security/audit-claude-code.sh"
      }]
    }]
  }
}
EOF

# Set appropriate permissions
sudo chmod 644 "/Library/Application Support/ClaudeCode/managed-settings.json"
```

## Related Resources

### Official Documentation
- [Claude Code Overview](https://docs.anthropic.com/en/docs/claude-code/overview)
- [Identity and Access Management](https://docs.anthropic.com/en/docs/claude-code/iam)
- [Security](https://docs.anthropic.com/en/docs/claude-code/security)
- [Hooks](https://docs.anthropic.com/en/docs/claude-code/hooks)
- [CLI Reference](https://docs.anthropic.com/en/docs/claude-code/cli-reference)
- [SDK Documentation](https://docs.anthropic.com/en/docs/claude-code/sdk)
- [Model Context Protocol](https://docs.anthropic.com/en/docs/claude-code/mcp)

### Configuration Topics
- [Settings Management](https://docs.anthropic.com/en/docs/claude-code/settings)
- [Environment Configuration](https://docs.anthropic.com/en/docs/claude-code/configuration)
- [Enterprise Deployment](https://docs.anthropic.com/en/docs/claude-code/third-party-integrations)
- [GitHub Actions Integration](https://docs.anthropic.com/en/docs/claude-code/github-actions)

### MAOS-Specific Considerations

For building a Multi-Agent Orchestration System, key settings capabilities include:

1. **Agent Isolation**: Use different permission profiles for each agent type
2. **Hierarchical Control**: Enterprise policies override individual agent settings
3. **Audit Trail**: Hooks provide comprehensive logging and control
4. **Resource Limits**: MCP timeouts and performance settings prevent runaway agents
5. **Security Boundaries**: Folder restrictions and permission modes enforce isolation
6. **Dynamic Configuration**: Environment variables and CLI flags enable runtime adjustment
7. **Extensibility**: MCP servers and hooks allow custom tool integration

The settings system provides the foundation for secure, scalable multi-agent orchestration with fine-grained control over each agent's capabilities and resources.