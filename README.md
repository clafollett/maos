# MAOS - Multi-Agent Orchestration System

**A high-performance Rust CLI that enhances Claude Code with parallel AI development capabilities.**

## What is MAOS?

MAOS is a compiled Rust binary that replaces fragile Python hook scripts with a fast, reliable CLI. It enhances Claude Code's ability to work with multiple AI agents in parallel through automatic workspace isolation and intelligent coordination.

**Key Features**:
- 🚀 **Blazing Fast**: Target <10ms execution (currently ~50-200ms with Python bootstrap)
- 🔒 **Rock Solid**: Compiled binary that can't be accidentally broken
- 📦 **Easy Install**: `npx @maos/cli` or `brew install maos` (coming soon)
- 👻 **Invisible**: Users interact with Claude Code normally

> **Bootstrap Phase**: MAOS currently uses Python scripts to implement all features while we build the Rust CLI. This means you can use MAOS today! The Python implementation in `.claude/hooks/` provides full functionality and will be replaced by the `maos` binary with the same features but better performance.

## Quick Start

**⚡ See the [Quick Start Guide](docs/QUICK_START.md) to get running in 5 minutes!**

### Current Status: Bootstrap Phase (Python Implementation)

MAOS is currently in bootstrap phase - the Python implementation in `.claude/hooks/` provides full functionality while we build the Rust CLI. 

**Use MAOS Today:**
1. Clone this repository
2. Configure your `.claude/settings.json` with Python hooks
3. Full orchestration features work now!

See [CONTRIBUTING.md](./CONTRIBUTING.md) for setup instructions.

### Future: Rust CLI Installation (Target: Q2 2024)

When the Rust CLI is released, installation will be simple:

```bash
# Via NPX (recommended)
npx @maos/cli setup

# Via Homebrew
brew install maos

# Direct download
curl -sSL https://raw.githubusercontent.com/clafollett/maos/main/scripts/install.sh | sh
```

The Rust CLI will automatically update your `settings.json` hooks during installation!

### Future Configuration

When complete, update your Claude Code `settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use"  // Currently: uv run script.py
    }],
    "PostToolUse": [{
      "command": "maos post-tool-use"
    }]
  }
}
```

That's it! MAOS now handles all orchestration automatically.

## How It Works

1. **User** talks to Claude Code: *"Build me a complete authentication system"*
2. **Claude Code** decides to use multiple agents for parallel work
3. **MAOS hooks** automatically:
   - Create isolated git worktrees for each agent
   - Prevent file conflicts between agents
   - Track progress and coordinate work
   - Clean up when complete
4. **Results** are seamlessly integrated back

No new commands to learn. No complex setup. Just better parallel AI development.

## Architecture

```
User → Claude Code → settings.json → MAOS CLI
                                       ↓
                              ┌────────┴────────┐
                              │   Rust Binary   │
                              │ • Security      │
                              │ • Orchestration │
                              │ • TTS & Notify  │
                              └────────┬────────┘
                                       ↓
                         ┌─────────────┴─────────────┐
                         │     Git Worktrees         │
                         │ • backend-engineer/       │
                         │ • frontend-engineer/      │
                         │ • qa-engineer/            │
                         └───────────────────────────┘
```

## CLI Commands

```bash
# Core hooks
maos pre-tool-use      # Security checks + orchestration
maos post-tool-use     # Cleanup + logging

# Notifications
maos notify            # Smart TTS notifications
maos stop              # Session end with TTS
maos subagent-stop     # Sub-agent cleanup

# Monitoring
maos prompt-submit     # Log user prompts
maos session-info      # Current session status
maos worktree-list     # Active worktrees
```

## Key Features in Detail

### 🔒 **Security First**
- Blocks dangerous `rm -rf` commands before execution
- Prevents access to `.env` files containing secrets
- Validates all paths to prevent directory traversal
- Compiled binary ensures tamper-proof operation

### 🚀 **Performance** (Target)
- Written in Rust for maximum speed
- Zero Python interpreter overhead (when complete)
- Sub-10ms hook execution (vs current ~50-200ms)
- Handles hundreds of tool calls efficiently

### 🎙️ **Smart Notifications**
- Multi-provider TTS support (ElevenLabs, OpenAI, macOS, pyttsx3)
- Automatic provider selection based on API keys
- Configurable voices and text limits
- Session completion announcements

### 🔧 **Professional Distribution** (Coming Soon)
- NPX for Node.js users: `npx @maos/cli`
- Homebrew for macOS: `brew install maos`
- Direct binaries for all platforms
- No Rust toolchain needed for users

## For MAOS Contributors

### Prerequisites

- Rust stable toolchain
- Git 2.5+ (for worktree support)
- Just task runner

### Development Setup

```bash
# Clone the repository
git clone https://github.com/clafollett/maos.git
cd maos

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build --release

# Run tests
cargo test

# Install locally for testing
cargo install --path .
```

## Architecture Overview

### Multi-Agent Orchestration
- **Automatic Isolation**: Each agent works in its own git worktree
- **Conflict Prevention**: File locking prevents simultaneous edits
- **Session Management**: Tracks multi-agent workflows seamlessly
- **Progress Tracking**: Real-time visibility into agent activities

### Implementation Details
- **Rust CLI**: Fast, reliable, single binary distribution
- **File-Based Coordination**: JSON files for state management
- **Hook Integration**: Works through Claude Code's settings.json
- **Git Worktrees**: Complete workspace isolation per agent

## Documentation

### For Users
- **[Quick Start Guide](docs/QUICK_START.md)**: Get running in 5 minutes! ⚡
- **[Installation Guide](docs/cli/installation.md)**: NPX, Homebrew, and binary installation
- **[CLI Reference](docs/cli/commands.md)**: Complete command documentation
- **[Configuration](docs/cli/configuration.md)**: settings.json and config options
- **[Migration Guide](docs/cli/migration.md)**: Moving from Python hooks to MAOS CLI
- **[Troubleshooting](docs/TROUBLESHOOTING.md)**: Common issues and solutions 🛠️

### For Contributors
- **[Architecture Overview](ARCHITECTURE.md)**: Rust crate structure and design
- **[Contributing Guide](CONTRIBUTING.md)**: Development setup and guidelines
- **[Development Workflow](docs/DEVELOPMENT_WORKFLOW.md)**: Standards and processes

## Roadmap

### Phase 1: Rust CLI Development (Current)
- [ ] Core CLI structure with clap
- [ ] Security features (rm -rf blocking, .env protection)
- [ ] MAOS orchestration (worktrees, sessions, locks)
- [ ] TTS integration (multi-provider support)
- [ ] Configuration management
- [ ] Comprehensive test suite

### Phase 2: Distribution
- [ ] NPM package for npx distribution
- [ ] Homebrew formula
- [ ] GitHub releases with binaries
- [ ] Installation scripts
- [ ] Auto-update mechanism

### Phase 3: Enhanced Features
- [ ] Performance profiling and optimization
- [ ] Advanced debugging commands
- [ ] Plugin system for custom hooks
- [ ] Web dashboard for monitoring
- [ ] Integration with more Claude Code features

## Why Choose MAOS?

### **Performance Matters**
- Eliminate Python startup overhead on every tool call
- Handle hundreds of operations without slowdown
- Near-instant execution for all commands

### **Professional Software**
- Compiled binary that can't be accidentally modified
- Consistent behavior across all environments
- Enterprise-ready security and reliability

### **Developer Experience**
- Simple installation via familiar tools (NPX, Homebrew)
- Clean configuration in settings.json
- Comprehensive documentation and support

### **Future-Proof**
- Written in Rust for long-term maintainability
- Extensible architecture for new features
- Active development and community

## Project Structure

```
maos/
├── Cargo.toml                    # Workspace configuration
├── crates/
│   ├── maos-cli/                 # Main CLI application
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # CLI entry point
│   │       └── commands/         # Subcommand implementations
│   ├── maos-core/                # Core orchestration logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── session.rs        # Session management
│   │       └── worktree.rs       # Git worktree operations
│   ├── maos-security/            # Security features
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── validators.rs     # Path and command validation
│   └── maos-tts/                 # TTS provider integration
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── providers/        # ElevenLabs, OpenAI, etc.
├── .claude/
│   ├── agents/                   # Agent configurations
│   └── config.example.json       # Example configuration
├── docs/
│   ├── cli/                      # User documentation
│   └── development/              # Contributor guides
└── scripts/
    ├── install.sh                # Installation script
    └── release.sh                # Release automation
```

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Key Development Principles

1. **Performance First** - Every millisecond counts in hook execution
2. **Type Safety** - Leverage Rust's type system for reliability
3. **User Experience** - Installation and usage must be effortless
4. **Backward Compatible** - Smooth migration from Python hooks
5. **Well Tested** - Comprehensive test coverage required

## Support

- **Documentation**: [GitHub Wiki](https://github.com/clafollett/maos/wiki)
- **Issues**: [GitHub Issues](https://github.com/clafollett/maos/issues)
- **Discussions**: [GitHub Discussions](https://github.com/clafollett/maos/discussions)

## License

MIT License - see [LICENSE](LICENSE) for details.

---

*MAOS: Professional multi-agent orchestration for Claude Code. Fast. Reliable. Invisible.*