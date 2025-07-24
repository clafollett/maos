#!/usr/bin/env python3
"""
Safe testing script for MAOS orchestration components.
Tests detection and info gathering without spawning anything.
"""

import sys
from pathlib import Path

# Add the maos directory to Python path
sys.path.insert(0, str(Path(__file__).parent / "maos"))

from terminal_integration import TerminalIntegration
from claude_orchestrator import ClaudeOrchestrator
import json


def test_terminal_info():
    """Just gather info, don't spawn anything."""
    print("\n🔍 Terminal Environment Detection")
    print("=" * 50)
    
    terminal = TerminalIntegration()
    info = terminal.get_info()
    
    print(f"Environment: {info['environment']}")
    print(f"Platform: {info['platform']}")
    
    if info['environment'] == 'vscode':
        print(f"VS Code variant: {info.get('vscode_variant', 'unknown')}")
        print(f"Available commands: {info.get('available_commands', [])}")
    elif info['environment'] == 'terminal':
        print(f"Terminal app: {info.get('terminal_app', 'unknown')}")
        print(f"Shell: {info.get('shell', 'unknown')}")
    elif info['environment'] == 'tmux':
        print(f"In tmux: Yes")
        print(f"Tmux available: {info.get('tmux_available', False)}")
    
    print("\nEnvironment variables:")
    for key, value in info.get('environment_hints', {}).items():
        print(f"  {key}: {value}")
    
    print("\n✅ Detection complete - no terminals spawned")


def test_orchestrator_info():
    """Check orchestrator without creating session."""
    print("\n🧠 Orchestrator Check")
    print("=" * 50)
    
    # Create temp directory for safe testing
    import tempfile
    with tempfile.TemporaryDirectory() as tmpdir:
        session_path = Path(tmpdir)
        orchestrator = ClaudeOrchestrator(session_path)
        
        status = orchestrator.get_status()
        print(f"Orchestrator status: {json.dumps(status, indent=2)}")
        print("\n✅ Orchestrator check complete - no session created")


def test_agent_command_preview():
    """Preview what commands would be generated."""
    print("\n📝 Agent Command Preview")
    print("=" * 50)
    
    terminal = TerminalIntegration()
    
    # Create a sample command
    from pathlib import Path
    sample_cmd = terminal.create_agent_command(
        agent_instance="backend-engineer-1",
        session_id=None,
        role="backend-engineer",
        task="Create a hello world API",
        workspace=Path("/tmp/test-workspace"),
        project_dir=Path("/tmp/test-project"),
        shared_context=Path("/tmp/test-shared")
    )
    
    print("Sample command that would be executed:")
    print("-" * 50)
    print(sample_cmd)
    print("-" * 50)
    print("\n✅ Command preview complete - nothing executed")


def main():
    print("🛡️  MAOS Safe Testing Mode")
    print("This will only gather information, not spawn anything\n")
    
    try:
        test_terminal_info()
        test_orchestrator_info()
        test_agent_command_preview()
        
        print("\n✨ All safe tests completed!")
        print("\nNext steps if you want to test further:")
        print("1. Run: python3 .claude/hooks/maos/launch_agent.py backend-engineer 'test task' --no-terminal")
        print("2. This would execute without spawning terminals")
        
    except Exception as e:
        print(f"\n❌ Error during testing: {e}")
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())