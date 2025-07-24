#!/usr/bin/env python3
"""Test script to debug MCP agent execution."""

import asyncio
import sys
import logging
from pathlib import Path

# Add the hooks directory to Python path
sys.path.insert(0, str(Path(__file__).parent))

from launch_agent import launch_agent

# Set up logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

def test_direct_launch():
    """Test launching an agent directly."""
    print("Testing direct agent launch...")
    
    try:
        result = launch_agent(
            "backend-engineer",
            "Create a simple test file test_mcp.txt with 'Hello from MCP test'"
        )
        print(f"Result: {result}")
        
        # Check if file was created
        if Path("test_mcp.txt").exists():
            print("✅ Success! File was created.")
            with open("test_mcp.txt") as f:
                print(f"Content: {f.read()}")
        else:
            print("❌ File was not created")
            
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()

async def test_async_launch():
    """Test async launch like MCP server does."""
    print("\nTesting async agent launch...")
    
    loop = asyncio.get_event_loop()
    
    try:
        # This is how the MCP server runs it
        result = await loop.run_in_executor(
            None,
            launch_agent,
            "backend-engineer",
            "Create a simple test file test_mcp_async.txt with 'Hello from async MCP test'"
        )
        print(f"Async Result: {result}")
        
    except Exception as e:
        print(f"Async Error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    # Test direct launch
    test_direct_launch()
    
    # Test async launch
    asyncio.run(test_async_launch())