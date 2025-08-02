#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# ///

"""
Integration test for MAOS hook system.
Tests the pre_tool_use.py hook with simulated Claude Code input.
"""

import json
import subprocess
import sys
from pathlib import Path

def test_task_tool_interception():
    """Test that Task tool with subagent_type gets intercepted correctly"""
    print("Testing Task tool interception...")
    
    # Simulate Claude Code hook input for Task tool with sub-agent
    hook_input = {
        "session_id": "test-session-123",
        "transcript_path": "/test/path",
        "cwd": str(Path.cwd()),
        "hook_event_name": "PreToolUse",
        "tool_name": "Task",
        "tool_input": {
            "subagent_type": "backend-engineer",
            "prompt": "Build a REST API for user management",
            "instructions": "Focus on security and performance"
        },
        "metadata": {
            "session_id": "test-session-123"
        }
    }
    
    # Run the pre_tool_use hook
    hook_path = Path(__file__).parent.parent / "pre_tool_use.py"
    result = subprocess.run(
        ["python3", str(hook_path)],
        input=json.dumps(hook_input),
        text=True,
        capture_output=True
    )
    
    print(f"Hook exit code: {result.returncode}")
    if result.stderr:
        print("Hook stderr:")
        print(result.stderr)
    
    # Check that workspace was created
    if "Created isolated workspace" in result.stderr:
        print("✅ Workspace creation successful")
    else:
        print("❌ Workspace creation failed")
    
    return result.returncode == 0

def test_file_operation_tracking():
    """Test file operation tracking and locking"""
    print("\nTesting file operation tracking...")
    
    # Simulate Edit tool call
    hook_input = {
        "session_id": "test-session-123",
        "transcript_path": "/test/path",
        "cwd": str(Path.cwd()),
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "tool_input": {
            "file_path": "/test/example.py",
            "old_string": "old code",
            "new_string": "new code"
        },
        "metadata": {
            "session_id": "test-session-123"
        }
    }
    
    hook_path = Path(__file__).parent.parent / "pre_tool_use.py"
    result = subprocess.run(
        ["python3", str(hook_path)],
        input=json.dumps(hook_input),
        text=True,
        capture_output=True
    )
    
    print(f"Edit hook exit code: {result.returncode}")
    if result.stderr:
        print("Edit hook stderr:")
        print(result.stderr)
    
    return result.returncode == 0

def test_security_blocking():
    """Test security blocking functionality"""
    print("\nTesting security blocking...")
    
    # Test dangerous rm command blocking
    hook_input = {
        "session_id": "test-session-123",
        "transcript_path": "/test/path",
        "cwd": str(Path.cwd()),
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {
            "command": "rm -rf /",
            "description": "Dangerous command test"
        }
    }
    
    hook_path = Path(__file__).parent.parent / "pre_tool_use.py"
    result = subprocess.run(
        ["python3", str(hook_path)],
        input=json.dumps(hook_input),
        text=True,
        capture_output=True
    )
    
    print(f"Security test exit code: {result.returncode}")
    if result.stderr:
        print("Security test stderr:")
        print(result.stderr)
    
    # Should return exit code 2 (blocked)
    if result.returncode == 2:
        print("✅ Security blocking successful")
        return True
    else:
        print("❌ Security blocking failed")
        return False

def main():
    """Run integration tests"""
    print("🧪 MAOS Hook Integration Tests")
    print("=" * 40)
    
    tests = [
        test_task_tool_interception,
        test_file_operation_tracking, 
        test_security_blocking
    ]
    
    passed = 0
    total = len(tests)
    
    for test in tests:
        try:
            if test():
                passed += 1
        except Exception as e:
            print(f"❌ Test failed with exception: {e}")
    
    print("\n" + "=" * 40)
    print(f"Tests passed: {passed}/{total}")
    
    if passed == total:
        print("🎉 All tests passed!")
        return 0
    else:
        print("💥 Some tests failed")
        return 1

if __name__ == "__main__":
    sys.exit(main())