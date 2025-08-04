# MAOS Development Commands
# Install just: https://github.com/casey/just#installation
# Usage: just <recipe>

# Default recipe (runs when you just type 'just')
default:
    @just --list

# ========================
# Git & Worktree Commands
# ========================

# List all active worktrees
worktree-list:
    @echo "📋 Active worktrees:"
    @git worktree list

# Clean up stale worktrees
worktree-cleanup:
    @echo "🧹 Pruning stale worktrees..."
    @git worktree prune
    @echo "✅ Cleanup complete"

# Show git status across all worktrees
status-all:
    @echo "📊 Status of all worktrees:"
    @for worktree in $(git worktree list --porcelain | grep "worktree" | cut -d' ' -f2); do \
        echo "\n📁 $$worktree:"; \
        git -C "$$worktree" status -s || echo "  (no changes)"; \
    done

# ========================
# Development Setup
# ========================

# Complete development environment setup
dev-setup:
    @echo "🚀 Setting up MAOS development environment..."
    @just setup-git-hooks
    @echo "✅ Development environment ready!"

# Set up git hooks for pre-commit linting
setup-git-hooks:
    #!/usr/bin/env bash
    echo "🪝 Setting up git hooks..."
    mkdir -p .git/hooks
    
    # Create pre-commit hook
    cat > .git/hooks/pre-commit << 'HOOK_EOF'
    #!/bin/sh
    # MAOS Pre-commit Hook - Runs linting and formatting checks
    
    set -e  # Exit on any error
    
    echo "🪝 MAOS Pre-commit checks starting..."
    
    # Check if we're in a Rust project
    if [ ! -f "Cargo.toml" ]; then
        echo "⚠️  No Cargo.toml found, skipping Rust checks"
        exit 0
    fi
    
    # Run formatting check
    echo "🎨 Checking code formatting..."
    cargo fmt -- --check || {
        echo "❌ Code formatting issues found!"
        echo "💡 Run 'cargo fmt' to fix formatting"
        exit 1
    }
    
    # Run clippy lints
    echo "📎 Running clippy lints..."
    cargo clippy --all-targets --all-features -- -D warnings || {
        echo "❌ Clippy warnings found!"
        echo "💡 Run 'cargo clippy --fix' to fix some issues automatically"
        exit 1
    }
    
    # Run tests
    echo "🧪 Running tests..."
    cargo test --quiet || {
        echo "❌ Tests failed!"
        echo "💡 Fix failing tests before committing"
        exit 1
    }
    
    echo "✅ All pre-commit checks passed!"
    HOOK_EOF
    
    chmod +x .git/hooks/pre-commit
    echo "✅ Git hooks installed! Commits will now run formatting and linting checks."

# Run all pre-commit checks manually
pre-commit:
    @echo "🔍 Running pre-commit checks..."
    @cargo fmt -- --check
    @cargo clippy --all-targets --all-features -- -D warnings
    @cargo test --quiet
    @echo "✅ All checks passed!"

# ========================
# Code Quality Commands
# ========================

# Format all Rust code
format fmt:
    @echo "🎨 Formatting code..."
    @cargo fmt
    @echo "✅ Formatting complete!"

# Run clippy lints
lint:
    @echo "📎 Running clippy..."
    @cargo clippy --all-targets --all-features -- -D warnings
    @echo "✅ No linting issues!"

# Fix clippy warnings automatically
fix:
    @echo "🔧 Auto-fixing clippy warnings..."
    @cargo clippy --fix --allow-dirty --allow-staged
    @echo "✅ Auto-fix complete!"

# Run tests
test:
    @echo "🧪 Running tests..."
    @cargo test
    @echo "✅ All tests passed!"

# Check everything (format, lint, test)
check: format lint test
    @echo "✅ All checks passed!"

# ========================
# MAOS Coordination
# ========================

# Show current MAOS session info
session-info:
    @echo "🤖 MAOS Session Info:"
    @if [ -f .maos/session.json ]; then \
        cat .maos/session.json | python -m json.tool; \
    else \
        echo "No active session"; \
    fi

# Show active agents
agents:
    @echo "👥 Active Agents:"
    @if [ -f .maos/coordination/agents.json ]; then \
        cat .maos/coordination/agents.json | python -m json.tool; \
    else \
        echo "No active agents"; \
    fi

# Clean MAOS coordination files (for debugging)
clean-maos:
    @echo "🧹 Cleaning MAOS coordination files..."
    @rm -rf .maos/
    @echo "✅ MAOS files cleaned"

# ========================
# Development Helpers
# ========================

# Check Python hooks are executable
check-hooks:
    @echo "🪝 Checking Claude hooks..."
    @ls -la .claude/hooks/*.py 2>/dev/null || echo "No hooks found"

# Watch MAOS coordination files for changes
watch-maos:
    @echo "👀 Watching MAOS coordination files..."
    @echo "Press Ctrl+C to stop"
    @watch -n 1 'echo "=== Session ===" && cat .maos/session.json 2>/dev/null | python -m json.tool || echo "No session"; echo "\n=== Agents ===" && cat .maos/coordination/agents.json 2>/dev/null | python -m json.tool || echo "No agents"'