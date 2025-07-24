#!/usr/bin/env python3
"""Test script to verify proper process detachment."""

import subprocess
import os
import time
import sys
from pathlib import Path

def test_detached_launch():
    """Test launching a Claude process with proper detachment."""
    
    print(f"Parent PID: {os.getpid()}")
    print(f"Working directory: {os.getcwd()}")
    
    # Create a test task that will create a file to prove it ran
    task = "Create a file named detached_test.txt with the content 'This was created by a detached process at [current timestamp]'"
    
    cmd = [
        "claude",
        "--model", "sonnet", 
        "-p", f"You are a test agent. Task: {task}",
        "--dangerously-skip-permissions"
    ]
    
    print(f"\nLaunching command: {' '.join(cmd)}")
    
    # Launch with detachment
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=True,
        preexec_fn=os.setsid if hasattr(os, 'setsid') else None
    )
    
    pid = process.pid
    print(f"Process started with PID: {pid}")
    
    # Close pipes to detach
    if process.stdout:
        process.stdout.close()
    if process.stderr:
        process.stderr.close()
    if process.stdin:
        process.stdin.close()
    
    print("Detached from process - parent exiting in 2 seconds")
    time.sleep(2)
    
    # Check if process is still running
    try:
        os.kill(pid, 0)  # Signal 0 just checks if process exists
        print(f"Process {pid} is still running (good!)")
    except ProcessLookupError:
        print(f"Process {pid} has already exited (bad!)")
    
    print("Parent script exiting now...")

if __name__ == "__main__":
    test_detached_launch()