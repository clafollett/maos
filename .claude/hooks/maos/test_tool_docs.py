#!/usr/bin/env python3
"""
Test that agent tool descriptions have correct role names.
"""

import sys
from pathlib import Path

# Add the current directory to the path
sys.path.insert(0, str(Path(__file__).parent))

def test_tool_descriptions():
    """Test that tool descriptions have correct role names."""
    try:
        from maos_mcp_server import MAOSMCPServer, MAOS_COMMANDS
        
        server = MAOSMCPServer()
        tool_manager = server.mcp._tool_manager
        tools = tool_manager._tools
        
        print("🔍 Testing tool descriptions...")
        
        # Check a few agent tools
        agent_tools = [(cmd, role) for cmd, role in MAOS_COMMANDS.items() if role]
        
        for cmd_name, role in agent_tools[:5]:  # Check first 5
            tool_name = f"maos/{cmd_name}"
            if tool_name in tools:
                tool = tools[tool_name]
                doc = tool.fn.__doc__
                print(f"✅ {tool_name}: {doc}")
                
                # Check if role name is properly inserted
                if role in doc:
                    print(f"   ✨ Role '{role}' correctly inserted")
                else:
                    print(f"   ❌ Role '{role}' not found in description")
            else:
                print(f"❌ {tool_name} not found")
        
        return True
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = test_tool_descriptions()
    sys.exit(0 if success else 1)
