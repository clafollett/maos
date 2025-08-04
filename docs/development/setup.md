# MAOS Development Setup

## Prerequisites

Before starting MAOS development, ensure you have:

- **Git**: Version 2.5+ (for worktree support)
- **Rust**: Stable toolchain (1.88.0+)
- **Just**: Task runner for development workflows
- **OS**: Linux, macOS, or Windows

## Quick Start

```bash
# Clone the repository
git clone https://github.com/clafollett/maos.git
cd maos

# Source the development environment
source stack.env

# Run automated setup
just dev-setup
```

This will:
1. Validate your Rust toolchain
2. Install required cargo tools (cargo-audit, etc.)
3. Set up git hooks for code quality
4. Verify all dependencies

## Detailed Setup

### 1. Install Rust

If you don't have Rust installed:

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add cargo to PATH
source ~/.cargo/env

# Verify installation
rustc --version  # Should show 1.88.0 or later
cargo --version
```

### 2. Install Just

Just is our task runner for consistent development workflows:

```bash
# macOS
brew install just

# Linux
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/bin
export PATH="$PATH:$HOME/bin"

# Windows (PowerShell)
winget install --id Casey.Just --exact

# Alternative: Install via Cargo
cargo install just
```

### 3. Clone and Configure

```bash
# Clone with SSH (recommended)
git clone git@github.com:clafollett/maos.git

# Or with HTTPS
git clone https://github.com/clafollett/maos.git

cd maos

# Load development environment
source stack.env

# This sets:
# - Rust toolchain version
# - Cargo configuration
# - Development tool paths
```

### 4. Run Setup

```bash
# Complete development setup
just dev-setup

# This installs:
# - cargo-audit (security scanning)
# - cargo-tarpaulin (code coverage)
# - Additional development tools
```

## Workspace Structure

MAOS uses a Cargo workspace with multiple crates:

```
maos/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── maos-cli/          # Main CLI application
│   ├── maos-core/         # Core orchestration logic
│   ├── maos-security/     # Security features
│   ├── maos-tts/          # TTS providers
│   ├── maos-config/       # Configuration management
│   ├── maos-testing/      # Shared testing utilities
│   └── maos-common/       # Common types and traits
├── .claude/               # Claude Code configuration
├── stack.env              # Development environment
└── justfile              # Development tasks
```

## IDE Setup

### VS Code (Recommended)

MAOS includes VS Code configuration:

```bash
# Install recommended extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension skellock.just
code --install-extension tamasfe.even-better-toml

# Open project
code .
```

Included configurations:
- `.vscode/settings.json` - Rust analyzer settings
- `.vscode/launch.json` - Debug configurations
- `.vscode/extensions.json` - Recommended extensions

### IntelliJ IDEA / CLion

1. Install Rust plugin
2. Open project as Cargo workspace
3. Configure rustfmt on save
4. Set up Just task runner

### Vim/Neovim

```vim
" Example config for Neovim with rust-analyzer
Plug 'neovim/nvim-lspconfig'
Plug 'simrat39/rust-tools.nvim'

lua << EOF
require('rust-tools').setup({
  server = {
    settings = {
      ["rust-analyzer"] = {
        checkOnSave = {
          command = "clippy"
        }
      }
    }
  }
})
EOF
```

## Development Commands

Essential commands for daily development:

```bash
# See all available commands
just

# Format code
just format

# Run lints
just lint

# Run tests
just test

# Build project
just build

# Full pre-commit checks
just pre-commit

# Run specific crate tests
just test-crate maos-security

# Check for security vulnerabilities
just audit

# Generate code coverage
just coverage
```

## Environment Variables

For development, you may want to set:

```bash
# Enable debug logging
export MAOS_DEBUG=1
export MAOS_LOG_LEVEL=trace

# TTS testing (optional)
export ELEVENLABS_API_KEY="your-test-key"
export OPENAI_API_KEY="your-test-key"

# Performance monitoring
export MAOS_LOG_PERF=1
```

## Common Development Tasks

### Adding a New Crate

```bash
# Create new crate
cargo new --lib crates/maos-newfeature

# Add to workspace in root Cargo.toml
[workspace]
members = [
    "crates/maos-cli",
    "crates/maos-newfeature",  # Add this
]

# Set up crate dependencies
cd crates/maos-newfeature
# Edit Cargo.toml
```

### Running Integration Tests

```bash
# Run all integration tests
just test-integration

# Run with real git operations
MAOS_TEST_REAL_GIT=1 just test-integration

# Run specific integration test
cargo test --test git_worktree_test
```

### Debugging

```bash
# Debug build with symbols
just build-debug

# Run with verbose output
RUST_LOG=trace cargo run -- pre-tool-use

# Use VS Code debugger
# Press F5 with launch configuration
```

## Troubleshooting

### Rust Version Issues

```bash
# Check current version
rustc --version

# Update to latest stable
rustup update stable
rustup default stable

# If using rustup
rustup override set stable
```

### Cargo Build Failures

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for conflicts
cargo tree --duplicates
```

### Git Hook Issues

```bash
# Skip hooks temporarily
git commit --no-verify

# Reinstall hooks
just setup-git-hooks

# Check hook permissions
ls -la .git/hooks/
```

### Performance Issues

```bash
# Use release builds for testing
just build-release

# Profile compilation time
cargo build --timings

# Use sccache for faster rebuilds
cargo install sccache
export RUSTC_WRAPPER=sccache
```

## Development Workflow

1. **Pick an issue** from GitHub
2. **Create feature branch**: `git checkout -b feature/issue-X/description`
3. **Write tests first** (TDD)
4. **Implement feature**
5. **Run checks**: `just pre-commit`
6. **Commit with conventional format**: `feat: add feature X (#issue)`
7. **Push and create PR**

## Best Practices

1. **Always run `just pre-commit`** before pushing
2. **Write tests for new features**
3. **Keep commits focused and atomic**
4. **Update documentation with code changes**
5. **Use conventional commit messages**
6. **Check performance impact of changes**

## Getting Help

- **Discord**: Join our development chat
- **GitHub Issues**: Report bugs or request features
- **Documentation**: Check `/docs` for details
- **Code Review**: Tag maintainers in PRs

## Next Steps

- [Testing Guide](./testing.md) - Writing and running tests
- [Architecture](../architecture/rust-cli-architecture.md) - System design
- [Contributing](../../CONTRIBUTING.md) - Contribution guidelines