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