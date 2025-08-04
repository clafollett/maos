# MAOS Quick Start Guide

Get up and running with MAOS in 5 minutes!

## What is MAOS?

MAOS (Multi-Agent Orchestration System) enhances Claude Code to work with multiple AI agents in parallel, each in their own isolated workspace. It's invisible to users - just talk to Claude normally!

## Current Setup (Python Bootstrap)

### 1. Clone MAOS

```bash
git clone https://github.com/clafollett/maos.git
cd maos
```

### 2. Configure Claude Code

Create or update `.claude/settings.json` in your project:

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

### 3. Test It Works

Ask Claude: "Build me a login system with frontend and backend components"

Claude will automatically:
- Spawn specialized agents (backend-engineer, frontend-engineer)
- Create isolated workspaces for each
- Coordinate their work
- Merge results back

## What You Get

✅ **Security**: Blocks dangerous commands like `rm -rf /`  
✅ **Isolation**: Each agent works in a separate git worktree  
✅ **Coordination**: Automatic file locking prevents conflicts  
✅ **Visibility**: Track progress in `.maos/sessions/`  

## Future: Rust CLI (Q2 2024)

When the Rust CLI releases, migration is simple:

**Before (Python):**
```json
"command": "uv run \"$(git rev-parse --show-toplevel)/.claude/hooks/pre_tool_use.py\""
```

**After (Rust):**
```json
"command": "maos pre-tool-use"
```

That's it! Same features, 10-20x faster.

## Common Commands

```bash
# See active sessions
ls .maos/sessions/

# View current agents
ls worktrees/

# Check logs
tail -f .maos/logs/maos.log

# Clean up old worktrees
git worktree prune
```

## Troubleshooting

### "Command not found"
- Python bootstrap: Ensure `uv` is installed
- Future Rust: Check `maos` is in your PATH

### No worktrees created
- Verify `.claude/settings.json` is properly configured
- Check `.maos/logs/` for errors

### Blocked commands
- MAOS blocks dangerous operations for safety
- Use more specific paths instead of wildcards

## Next Steps

- Read [Architecture Overview](../ARCHITECTURE.md) to understand how it works
- Check [Security Features](./features/security.md) for protection details
- See [CLI Commands](./cli/commands.md) for all available commands

## Get Help

- 🐛 [Report Issues](https://github.com/clafollett/maos/issues)
- 💬 [Discussions](https://github.com/clafollett/maos/discussions)
- 📚 [Full Documentation](../README.md)

---

**Remember**: You don't need to learn MAOS commands. Just talk to Claude Code normally and enjoy parallel AI development!