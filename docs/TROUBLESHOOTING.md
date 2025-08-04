# MAOS Troubleshooting Guide

## Common Issues and Solutions

### Installation Issues

#### "Command not found" (Python Bootstrap)

**Problem**: `uv: command not found` when hooks run

**Solution**:
```bash
# Install uv if missing
curl -LsSf https://astral.sh/uv/install.sh | sh

# Or via pip
pip install uv
```

#### "Command not found" (Future Rust CLI)

**Problem**: `maos: command not found`

**Solution**:
```bash
# Check if maos is installed
which maos

# If not in PATH, add it
export PATH="$PATH:/usr/local/bin"

# Or use full path in settings.json
"command": "/usr/local/bin/maos pre-tool-use"
```

### Hook Issues

#### Hooks Not Running

**Problem**: MAOS commands don't seem to execute

**Diagnosis**:
1. Check file location: `.claude/settings.json` (not `.claude.json`)
2. Validate JSON syntax: `cat .claude/settings.json | jq`
3. Check Claude Code version supports hooks

**Solution**:
```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "echo 'Hook is running' && maos pre-tool-use"
    }]
  }
}
```

#### Hook Timeout

**Problem**: "Hook timed out" errors

**Solution**:
```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use",
      "config": {
        "timeout": 10000  // Increase to 10 seconds
      }
    }]
  }
}
```

### Worktree Issues

#### No Worktrees Created

**Problem**: Agents aren't getting isolated workspaces

**Diagnosis**:
```bash
# Check if worktrees exist
ls worktrees/

# Check session info
cat .maos/sessions/*/agents.json

# Check logs
grep "worktree" .maos/logs/maos.log
```

**Common Causes**:
- Git version too old (need 2.5+)
- Not in a git repository
- Lazy creation - worktree only created on first file operation

#### Worktree Cleanup

**Problem**: Too many old worktrees cluttering repository

**Solution**:
```bash
# List all worktrees
git worktree list

# Prune stale worktrees
git worktree prune

# Remove specific worktree
git worktree remove worktrees/agent-session-123

# Clean all MAOS worktrees (careful!)
rm -rf worktrees/
git worktree prune
```

### Security Blocks

#### "Security violation" Errors

**Problem**: Commands being blocked unexpectedly

**Common Blocks**:
- `rm -rf` with broad paths
- Access to `.env` files
- Path traversal attempts

**Solutions**:
```bash
# Instead of: rm -rf *
# Use: rm -rf specific-directory/

# Instead of: cat .env
# Use: cat .env.example

# Instead of: cd ../../../etc
# Use: absolute paths
```

#### Bypassing Security (Development Only)

**Warning**: Only for development/testing!

```json
{
  "maos": {
    "security": {
      "block_rm_rf": false,
      "block_env_access": false
    }
  }
}
```

### Performance Issues

#### Slow Hook Execution

**Problem**: Hooks taking longer than expected

**Diagnosis**:
```bash
# Enable performance logging
export MAOS_LOG_PERF=1
export MAOS_LOG_LEVEL=debug

# Check timing
grep "PERF" .maos/logs/performance.log
```

**Common Causes**:
- Python interpreter startup (current bootstrap)
- Large repository operations
- Network calls in TTS

**Solutions**:
- Wait for Rust CLI (10-20x faster)
- Disable TTS if not needed
- Use lazy worktree creation

### TTS Issues

#### No Audio Output

**Problem**: Text-to-speech not working

**Diagnosis**:
```bash
# Check API keys
echo $ELEVENLABS_API_KEY
echo $OPENAI_API_KEY

# Test TTS directly
maos notify --test "Hello world"

# Check logs
grep "TTS" .maos/logs/tts.log
```

**Solutions**:
1. Verify API keys are set
2. Check system audio not muted
3. Try different provider
4. Disable TTS if not needed

### Session Issues

#### Stale Sessions

**Problem**: Old sessions not cleaning up

**Solution**:
```bash
# View all sessions
ls .maos/sessions/

# Clean specific session
rm -rf .maos/sessions/sess-12345/

# Clean all old sessions (careful!)
find .maos/sessions -type d -mtime +7 -exec rm -rf {} \;
```

### Debug Mode

#### Enable Maximum Debugging

When nothing else works:

```bash
# Maximum verbosity
export MAOS_DEBUG=1
export MAOS_LOG_LEVEL=trace
export RUST_BACKTRACE=1  # For Rust CLI

# Log to file
export MAOS_LOG_FILE=/tmp/maos-debug.log

# Test specific command
echo '{"tool":"Bash","params":{"command":"ls"}}' | maos pre-tool-use
```

### Getting Help

#### Collect Debug Information

Before reporting issues, collect:

```bash
# System info
uname -a
git --version
python --version  # For bootstrap
rustc --version   # For Rust CLI

# MAOS info
maos --version
cat .claude/settings.json

# Recent logs
tail -n 100 .maos/logs/maos.log
```

#### Report Issues

1. [GitHub Issues](https://github.com/clafollett/maos/issues)
2. Include debug information
3. Describe expected vs actual behavior
4. Include minimal reproduction steps

### Quick Fixes

| Problem | Quick Fix |
|---------|-----------|
| Hooks not running | Check `.claude/settings.json` syntax |
| No worktrees | Verify git 2.5+, check logs |
| Security blocks | Use specific paths, not wildcards |
| Slow performance | Disable TTS, wait for Rust CLI |
| No audio | Check API keys and system audio |

## Prevention Tips

1. **Regular Cleanup**: Run `git worktree prune` weekly
2. **Monitor Logs**: Check `.maos/logs/` for warnings
3. **Update Regularly**: Keep MAOS updated for fixes
4. **Test Changes**: Verify settings.json changes work
5. **Read Errors**: Security blocks include helpful hints

## Related Documentation

- [Quick Start Guide](./QUICK_START.md) - Getting started
- [Security Features](./features/security.md) - Understanding blocks
- [Configuration](./cli/configuration.md) - Settings reference
- [Logging](./features/logging.md) - Debug log details