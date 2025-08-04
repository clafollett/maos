# MAOS Security Features

## Overview

MAOS implements multiple layers of security to protect your development environment from accidental or malicious damage. All security features are enforced at the hook level, preventing dangerous operations before they execute.

## Core Security Features

### 1. Dangerous Command Blocking

MAOS blocks destructive commands that could harm your system:

```bash
# These commands are BLOCKED:
rm -rf /
rm -rf /*
sudo rm -rf
rm -rf ~
rm -rf $HOME
```

**How it works:**
- Pre-tool-use hook intercepts all Bash commands
- Pattern matching identifies dangerous operations
- Blocked commands return exit code 2 with clear error message
- Agent learns to avoid these patterns through feedback

### 2. Environment File Protection

Sensitive files containing secrets are protected from access:

```bash
# Protected files:
.env
.env.local
.env.production
.env.staging
.env.development

# Allowed files:
.env.example
.env.sample
.env.template
```

**Protection includes:**
- Read operations blocked
- Write operations blocked
- Edit operations blocked
- Move/delete operations blocked

### 3. Path Traversal Prevention

MAOS validates all file paths to prevent directory traversal attacks:

```bash
# Blocked patterns:
../../../etc/passwd
/etc/shadow
~/../../sensitive-file
```

**Validation includes:**
- Canonical path resolution
- Symlink following prevention
- Absolute path validation
- Workspace boundary enforcement

### 4. Git Operation Safety

Git operations are validated to prevent repository corruption:

```bash
# Protected operations:
git push --force to main/master
git reset --hard on shared branches
git clean -fdx without confirmation
```

## Workspace Isolation

### Agent Workspace Enforcement

Each sub-agent operates in an isolated git worktree:

```
main/                    # Protected main branch
worktrees/
├── backend-123/        # Backend agent isolated workspace
├── frontend-123/       # Frontend agent isolated workspace
└── security-123/       # Security agent isolated workspace
```

**Enforcement mechanism:**
1. Agent attempts file operation outside workspace
2. MAOS blocks with exit code 2
3. Error message shows correct workspace path
4. Agent retries with correct path

### File Lock Management

Prevents simultaneous edits to the same file:

```json
{
  "locks": {
    "src/auth.rs": {
      "agent_id": "backend-123",
      "locked_at": "2024-01-20T10:30:00Z",
      "session_id": "sess-789"
    }
  }
}
```

## Security Configuration

### Default Security Settings

```json
{
  "maos": {
    "security": {
      "block_rm_rf": true,
      "block_env_access": true,
      "enforce_workspace_isolation": true,
      "validate_paths": true
    }
  }
}
```

### Custom Security Rules

Add project-specific security patterns:

```json
{
  "maos": {
    "security": {
      "custom_blocks": [
        "production.db",
        "*.key",
        "secrets/*"
      ],
      "allowed_env_files": [
        ".env.example",
        ".env.test"
      ]
    }
  }
}
```

## Implementation Details

### Rust Security Module

The security validation is implemented in the [`maos-security`](../architecture/rust-cli-architecture.md#maos-security) crate:

```rust
pub fn validate_command(cmd: &str) -> Result<(), SecurityError> {
    // Check against dangerous patterns
    if is_dangerous_rm(cmd) {
        return Err(SecurityError::DangerousCommand(
            "Blocked dangerous rm command".into()
        ));
    }
    
    // Additional validations...
    Ok(())
}
```

### Performance Impact

Security checks are designed for minimal overhead:
- Pattern matching: <0.1ms
- Path validation: <0.5ms
- Total overhead: <1ms per operation

## Security Audit Trail

All blocked operations are logged for audit:

```
.maos/logs/security/
├── 2024-01-20-blocked-commands.log
├── 2024-01-20-env-access-attempts.log
└── 2024-01-20-workspace-violations.log
```

## Best Practices

### For Users

1. **Trust the blocks**: If MAOS blocks an operation, it's protecting you
2. **Use allowed alternatives**: `.env.example` instead of `.env`
3. **Work within workspaces**: Let MAOS handle isolation

### For Contributors

1. **Security-first design**: Always validate inputs
2. **Fail securely**: Block by default, allow by exception
3. **Clear error messages**: Help users understand why operations were blocked
4. **Performance matters**: Security checks must be fast

## Threat Model

MAOS protects against:

1. **Accidental damage**: Typos in dangerous commands
2. **AI hallucinations**: Claude suggesting dangerous operations
3. **Secret exposure**: Preventing .env file access
4. **Repository corruption**: Protecting git integrity
5. **Cross-agent interference**: Workspace isolation

MAOS does NOT protect against:
- Malicious users with system access
- Deliberate circumvention of protections
- Zero-day vulnerabilities in dependencies

## Future Enhancements

- [ ] Configurable security levels (strict/moderate/permissive)
- [ ] Security policy templates for common frameworks
- [ ] Integration with secret scanning tools
- [ ] Real-time security alerts via TTS
- [ ] Machine learning for anomaly detection

## Related Documentation

- [Configuration Guide](../cli/configuration.md) - Security settings
- [Architecture](../architecture/rust-cli-architecture.md) - Security module design
- [Commands Reference](../cli/commands.md) - Security in each command