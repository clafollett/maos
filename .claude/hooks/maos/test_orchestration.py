#!/usr/bin/env python3
"""
Test the MAOS orchestration system with direct CLI execution.
"""

import subprocess
import sys
from pathlib import Path

def run_test(agent: str, task: str):
    """Run a test with the given agent and task."""
    print(f"\n{'='*60}")
    print(f"Testing: {agent}")
    print(f"Task: {task}")
    print('='*60)
    
    cmd = [
        sys.executable,
        ".claude/hooks/maos/launch_agent.py",
        agent,
        task
    ]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True)
        print(result.stdout)
        if result.stderr:
            print("STDERR:", result.stderr)
        return result.returncode == 0
    except Exception as e:
        print(f"ERROR: {e}")
        return False

def main():
    """Run orchestration tests."""
    print("🧪 MAOS Orchestration Test Suite")
    print("Testing direct CLI execution with streaming output\n")
    
    tests = [
        ("backend-engineer", "Create a simple README.md file explaining MAOS"),
        ("frontend-engineer", "Create an index.html file with a basic MAOS landing page"),
        ("qa", "Review the created files and suggest improvements")
    ]
    
    passed = 0
    for agent, task in tests:
        if run_test(agent, task):
            passed += 1
            print("✅ PASSED")
        else:
            print("❌ FAILED")
    
    print(f"\n📊 Results: {passed}/{len(tests)} tests passed")
    
    # Clean up test files
    print("\n🧹 Cleaning up test files...")
    for file in ["README.md", "index.html", "test.py"]:
        p = Path(file)
        if p.exists():
            p.unlink()
            print(f"   Removed {file}")

if __name__ == "__main__":
    main()