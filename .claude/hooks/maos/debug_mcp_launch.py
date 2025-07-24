#!/usr/bin/env python3
"""Debug script to test MCP agent launching."""

import subprocess
import sys
import time
import os
from pathlib import Path

def test_launch():
    """Test launching an agent like MCP does."""
    
    print(f"Current directory: {os.getcwd()}")
    print(f"Python executable: {sys.executable}")
    print(f"Environment PATH: {os.environ.get('PATH', 'NOT SET')}")
    
    cmd = [
        sys.executable,
        ".claude/hooks/maos/launch_agent.py",
        "backend-engineer",
        "Create a test file named mcp_test.txt with content 'Testing MCP launch'"
    ]
    
    print(f"\nCommand to run: {cmd}")
    
    # Launch exactly like MCP server does
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd=Path.cwd()
    )
    
    print(f"Process started with PID: {process.pid}")
    
    # Wait a bit
    time.sleep(3)
    
    # Check if still running
    poll = process.poll()
    if poll is None:
        print("Process is still running")
        
        # Try to get some output
        try:
            stdout, stderr = process.communicate(timeout=5)
            print(f"\nSTDOUT:\n{stdout}")
            print(f"\nSTDERR:\n{stderr}")
        except subprocess.TimeoutExpired:
            print("Process timed out - still running")
            process.kill()
            stdout, stderr = process.communicate()
            print(f"\nSTDOUT after kill:\n{stdout}")
            print(f"\nSTDERR after kill:\n{stderr}")
    else:
        stdout, stderr = process.communicate()
        print(f"Process exited with code: {poll}")
        print(f"\nSTDOUT:\n{stdout}")
        print(f"\nSTDERR:\n{stderr}")

if __name__ == "__main__":
    test_launch()