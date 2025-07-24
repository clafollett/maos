#!/usr/bin/env python3
"""
Test script for MAOS MCP Server
"""

import asyncio
import json
import sys
from pathlib import Path

# Add the current directory to the path
sys.path.insert(0, str(Path(__file__).parent))

def test_imports():
    """Test that all required modules can be imported."""
    try:
        from maos_mcp_server import MAOSMCPServer, MAOS_COMMANDS
        return True
    except ImportError as e:
        print(f"❌ Import error: {e}")
        return False

def test_command_mapping():
    """Test that MAOS_COMMANDS mapping is correct."""
    try:
        from maos_mcp_server import MAOS_COMMANDS
        
        total_commands = len(MAOS_COMMANDS)
        agent_commands = len([v for v in MAOS_COMMANDS.values() if v is not None])
        
        print(f"✅ Command mapping: {total_commands} total, {agent_commands} agents")
        return True
    except ImportError as e:
        print(f"❌ Command mapping test failed: {e}")
        return False

def test_server_initialization():
    """Test that the MCP server initializes successfully."""
    try:
        from maos_mcp_server import MAOSMCPServer
        server = MAOSMCPServer()
        return True
    except Exception as e:
        print(f"❌ Server initialization failed: {e}")
        return False

def test_tool_registration():
    """Test that all tools are properly registered."""
    try:
        from maos_mcp_server import MAOSMCPServer, MAOS_COMMANDS
        server = MAOSMCPServer()
        
        # Count expected tools (all commands except demo-resume)
        expected_tools = len([cmd for cmd in MAOS_COMMANDS.keys() if cmd != "demo-resume"])
        
        # Check that FastMCP has tools registered via the internal _tool_manager
        if hasattr(server.mcp, '_tool_manager'):
            tool_manager = server.mcp._tool_manager
            if hasattr(tool_manager, '_tools'):
                actual_tools = len(tool_manager._tools)
                tool_names = list(tool_manager._tools.keys())
            else:
                actual_tools = 0
                tool_names = []
        else:
            # Fallback - check if we can access tools through the mcp server
            actual_tools = 0
            tool_names = []
            
        print(f"✅ Tools: expected {expected_tools}, registered {actual_tools}")
        
        if tool_names:
            print(f"📋 Tools: {', '.join(sorted(tool_names))}")
        
        return actual_tools == expected_tools
        
    except Exception as e:
        print(f"❌ Tool registration test failed: {e}")
        return False

def main():
    """Run all tests."""
    print("🧪 Testing MAOS MCP Server...")
    
    tests = [
        ("Import test", test_imports),
        ("Command mapping test", test_command_mapping),
        ("Server initialization test", test_server_initialization),
        ("Tool registration test", test_tool_registration),
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        try:
            if test_func():
                print(f"✅ {test_name} passed")
                passed += 1
            else:
                print(f"❌ {test_name} failed")
        except Exception as e:
            print(f"❌ {test_name} failed with error: {e}")
    
    print(f"\n📊 Test Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 All tests passed!")
        return 0
    else:
        print("⚠️  Some tests failed")
        return 1

if __name__ == "__main__":
    sys.exit(main())
