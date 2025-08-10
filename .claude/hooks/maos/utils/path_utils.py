"""
Shared path utilities for Claude Code hooks.
Provides centralized path resolution and common directory constants.
"""

from pathlib import Path
import subprocess


def get_project_root():
    """Get project root using git or current working directory."""
    try:
        root = subprocess.check_output(
            ['git', 'rev-parse', '--show-toplevel'],
            stderr=subprocess.DEVNULL,
            text=True
        ).strip()
        return Path(root)
    except:
        return Path.cwd()


# Define common paths as constants
PROJECT_ROOT = get_project_root()
LOGS_DIR = PROJECT_ROOT / 'logs'
MAOS_DIR = PROJECT_ROOT / '.maos'
HOOKS_DIR = PROJECT_ROOT / '.claude' / 'hooks'
WORKTREES_DIR = PROJECT_ROOT / 'worktrees'