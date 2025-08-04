# MAOS CLI Installation Guide

## Quick Install (Coming Soon)

MAOS will be distributed as a compiled binary with no runtime dependencies. Choose your preferred installation method:

### NPX (Recommended)
```bash
npx @maos/cli setup
```

This will:
- Install the MAOS binary
- Automatically update your `.claude/settings.json`
- Create backups of existing configuration
- Migrate Python hooks to Rust commands

### Homebrew
```bash
brew install maos
```

### Direct Download
```bash
curl -sSL https://raw.githubusercontent.com/clafollett/maos/main/scripts/install.sh | sh
```

## Current Development Setup

During the bootstrap phase, MAOS uses Python hooks:

1. Clone the repository:
   ```bash
   git clone https://github.com/clafollett/maos.git
   cd maos
   ```

2. Configure Claude Code hooks in `.claude/settings.json`:
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

## System Requirements

- **Git**: Version 2.5+ (for worktree support)
- **Claude Code**: Latest version with hook support
- **Operating Systems**: Linux, macOS, Windows

## Verification

Once installed, verify MAOS is working:

```bash
maos --version
# Expected: maos 0.1.0

maos --help
# Shows available commands
```

## Distribution Channels

### NPM Package (@maos/cli)
- Auto-detects platform and architecture
- Downloads appropriate binary
- Handles PATH setup automatically
- Supports `npx` for one-time use

### Homebrew (macOS/Linux)
- Standard `brew` installation
- Automatic updates via `brew upgrade`
- Handles dependencies if any

### Binary Releases
- GitHub Releases page
- Pre-compiled for all major platforms
- Checksums provided for verification
- No installation wizard needed

## Uninstallation

### NPX
```bash
npm uninstall -g @maos/cli
```

### Homebrew
```bash
brew uninstall maos
```

### Direct Download
```bash
rm /usr/local/bin/maos
```

## Troubleshooting

### Permission Denied
If you get permission errors, ensure the binary is executable:
```bash
chmod +x /path/to/maos
```

### Command Not Found
Add MAOS to your PATH:
```bash
export PATH="$PATH:/path/to/maos/bin"
```

### Hook Configuration Issues
Ensure your `.claude/settings.json` uses absolute paths or the `git rev-parse` pattern shown above.

## Next Steps

- [CLI Commands Reference](./commands.md)
- [Configuration Guide](./configuration.md)
- [Migration from Python](./migration.md)