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
    except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired):
        return Path.cwd()


# Define common paths as constants
PROJECT_ROOT = get_project_root()
LOGS_DIR = PROJECT_ROOT / 'logs'
MAOS_DIR = PROJECT_ROOT / '.maos'
HOOKS_DIR = PROJECT_ROOT / '.claude' / 'hooks'
MAOS_HOOKS_DIR = HOOKS_DIR / 'maos'  # This is the MAOS package directory
MAOS_HOOKS_SCRIPTS_DIR = MAOS_HOOKS_DIR / 'hooks'  # This is where hook scripts live
TTS_DIR = MAOS_HOOKS_DIR / 'tts'  # TTS scripts directory
WORKTREES_DIR = PROJECT_ROOT / 'worktrees'

def setup_maos_imports():
    """Setup Python import path for MAOS modules.
    
    Call this function at the beginning of any script that needs to import
    from utils, tts, handlers, etc. This provides a single source of truth
    for import path setup across all MAOS scripts.
    """
    import sys
    
    # Add MAOS hooks directory (the package root) to Python path for imports
    maos_path = str(MAOS_HOOKS_DIR)
    if maos_path not in sys.path:
        sys.path.insert(0, maos_path)