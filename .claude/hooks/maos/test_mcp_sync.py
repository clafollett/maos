#!/usr/bin/env python3
"""Test the synchronous MCP server locally."""

import subprocess
import json

def test_mcp_server():
    """Test basic MCP server functionality."""
    
    # Test listing tools
    cmd = ["python3", ".claude/hooks/maos/maos_mcp_server_sync.py"]
    
    # Send initialization request
    init_request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "0.1.0",
            "capabilities": {
                "tools": {},
                "prompts": {},
                "resources": {}
            }
        }
    }
    
    # Send list tools request
    list_tools_request = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }
    
    process = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    # Send requests
    process.stdin.write(json.dumps(init_request) + "\n")
    process.stdin.write(json.dumps(list_tools_request) + "\n")
    process.stdin.flush()
    
    # Read responses
    init_response = process.stdout.readline()
    tools_response = process.stdout.readline()
    
    print("Init response:", init_response)
    print("Tools response:", tools_response)
    
    # Terminate
    process.terminate()

if __name__ == "__main__":
    test_mcp_server()