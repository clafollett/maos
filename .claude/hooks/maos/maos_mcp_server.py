#!/usr/bin/env python3
"""
MAOS MCP Server (Synchronous) - Runs agents and returns their output.

This version runs agents synchronously and returns their complete output,
avoiding the issues with detached processes.
"""

import asyncio
import json
import logging
import os
from pathlib import Path
from typing import Dict, Any, List, Optional, Union
import subprocess
import sys
import re
import signal

# MCP server imports
try:
    from mcp.server import FastMCP
    from mcp.server.fastmcp import Context
    from mcp.types import TextContent
except ImportError as e:
    print(f"Error: {e}", file=sys.stderr)
    print("Please install mcp: pip install 'mcp[cli]'")
    print("Or try: pip install mcp")
    sys.exit(1)

# Import our existing modules
from session_manager import SessionManager, SessionType, SessionStatus

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(process)d - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Map command names to their roles (from our actual commands)
MAOS_COMMANDS = {
    # Agent roles
    "api-architect": "api-architect",
    "application-architect": "application-architect", 
    "backend-engineer": "backend-engineer",
    "business-analyst": "business-analyst",
    "data-architect": "data-architect",
    "data-scientist": "data-scientist",
    "devops": "devops",
    "frontend-engineer": "frontend-engineer",
    "mobile-engineer": "mobile-engineer",
    "pm": "pm",
    "qa": "qa",
    "researcher": "researcher",
    "reviewer": "reviewer",
    "secops": "secops",
    "security-architect": "security-architect",
    "solution-architect": "solution-architect",
    "techwriter": "techwriter",
    "tester": "tester",
    "ux-designer": "ux-designer",
    
    # Special commands
    "orchestrator": None,  # Special handling
    "session": None,       # Special handling
    "current": None,       # Special handling
    "demo-resume": None,   # Special handling
    "usage": None          # Special handling
}

class MAOSMCPServer:
    """MCP Server for MAOS - runs agents synchronously."""
    
    def __init__(self):
        logger.info(f"MAOSMCPServer.__init__ called - PID: {os.getpid()}")
        
        # Store project root but don't change directory in init
        script_dir = Path(__file__).parent
        self.project_root = script_dir.parent.parent.parent
        logger.info(f"Project root: {self.project_root}")
        
        self.mcp = FastMCP("maos")
        self.running_agents: Dict[str, asyncio.Task] = {}
        self.orchestrator_session: Optional[str] = None
        self._setup_tools()
        
        logger.info("MAOSMCPServer initialization complete")
        
    def _setup_tools(self):
        """Register all MAOS commands as MCP tools."""
        self._register_all_tools()
        
    def _register_all_tools(self):
        """Register all MAOS tools using the modern MCP API."""
        
        # Register agent commands
        for cmd_name, role in MAOS_COMMANDS.items():
            if role:  # Regular agent commands
                self._register_agent_tool(cmd_name, role)
            else:  # Special commands
                self._register_special_tools(cmd_name)
        
    def _register_agent_tool(self, cmd_name: str, role: str):
        """Register an agent tool for the given command and role."""
        
        # Create tool function with dynamic docstring
        def create_agent_tool(role_name):
            async def agent_tool(task: str, ctx: Context) -> List[TextContent]:
                """Launch an agent with the specified task."""
                try:
                    # Start progress reporting
                    await ctx.report_progress(0.1, 1.0, f"Launching {role_name} agent...")
                    
                    # Run the agent with progress context
                    logger.info(f"Tool {role_name} starting execution...")
                    result = await self._run_agent_with_progress(role_name, task, ctx)
                    logger.info(f"Tool {role_name} execution complete with status: {result['status']}")
                    
                    # Return as TextContent blocks
                    if result['status'] == 'running':
                        content = f"""## Agent Launched 🚀

**Status**: Running in background
**Process ID**: {result.get('pid', 'Unknown')}
**Session ID**: `{result['session_id']}`
**Workspace**: {result['workspace']}

The {role_name} agent is processing your request. Since this may take time, it's running in the background.

### Check Progress
Use the session tool:
```
action: "show"
session_id: "{result['session_id']}"
```

### Early Output
```
{result['output']}
```"""
                    else:
                        content = f"""## Agent Results

**Status**: {result['status']}
**Session ID**: {result['session_id']}
**Workspace**: {result['workspace']}

### Summary
{result['summary']}

### Output
```
{result['output']}
```"""
                        
                        if result.get('error'):
                            content += f"\n\n### Errors\n```\n{result['error']}\n```"
                    
                    # Send final progress
                    if ctx:
                        await ctx.report_progress(1.0, 1.0, "Done!")
                    
                    logger.info(f"Returning {'early' if result['status'] == 'running' else 'complete'} results")
                    return [TextContent(type="text", text=content)]
                        
                except Exception as e:
                    logger.error(f"Failed to launch {role_name}: {e}", exc_info=True)
                    error_text = f"Error: {str(e)}\n\nCheck server logs for more information."
                    return [TextContent(type="text", text=error_text)]
            
            # Set dynamic docstring
            agent_tool.__doc__ = f"Launch a {role_name} agent with the specified task."
            return agent_tool
        
        # Create and register the tool
        agent_tool = create_agent_tool(role)
        self.mcp.tool(name=f"maos/{cmd_name}")(agent_tool)

    def _register_special_tools(self, cmd_name: str):
        """Register special tools for orchestrator, session, current, and usage."""
        
        if cmd_name == "orchestrator":
            @self.mcp.tool(name="maos/orchestrator")
            async def orchestrator_tool(task: str) -> List[TextContent]:
                """Create an orchestration session to coordinate multiple agents."""
                try:
                    manager = SessionManager()
                    session = manager.create_session(
                        task=task,
                        session_type=SessionType.ORCHESTRATION
                    )
                    self.orchestrator_session = session.session_id
                    
                    # Create orchestrator marker
                    marker_path = Path(".maos/.current_orchestration")
                    marker_path.parent.mkdir(exist_ok=True)
                    with open(marker_path, 'w') as f:
                        json.dump({"session_id": session.session_id}, f)
                    
                    text = f"""## Orchestration Session Created

**Session ID**: {session.session_id}
**Task**: {task}

### Next Steps
1. Analyze the task and identify which agents are needed
2. Launch agents using their respective tools (e.g., maos/backend-engineer)
3. Agents will coordinate through shared context
4. Use maos/session with action='show' to monitor progress

The orchestration session is now active. Launch the appropriate agents to complete your task."""
                    
                    return [TextContent(type="text", text=text)]
                    
                except Exception as e:
                    logger.error(f"Failed to create orchestration session: {e}")
                    return [TextContent(type="text", text=f"Error: {str(e)}")]
        
        elif cmd_name == "current":
            @self.mcp.tool(name="maos/current")
            async def current_tool() -> List[TextContent]:
                """Show current orchestration session."""
                marker_path = Path(".maos/.current_orchestration")
                if not marker_path.exists():
                    return [TextContent(type="text", text="No active orchestration session.")]
                    
                try:
                    with open(marker_path) as f:
                        data = json.load(f)
                except (json.JSONDecodeError, OSError) as e:
                    logger.warning(f"Failed to read orchestration marker: {e}")
                    return [TextContent(type="text", text="Failed to read orchestration session data.")]
                    
                manager = SessionManager()
                session = manager.get_session(data["session_id"])
                
                if not session:
                    return [TextContent(type="text", text="Orchestration session not found.")]
                    
                text = f"""## Current Orchestration Session

**Session ID**: {session.session_id}
**Status**: {session.status.value}
**Created**: {session.created.isoformat()}

### Task
{session.task}"""
                
                return [TextContent(type="text", text=text)]
        
        elif cmd_name == "usage":
            @self.mcp.tool(name="maos/usage")
            async def usage_tool() -> List[TextContent]:
                """Show MAOS usage statistics."""
                manager = SessionManager()
                sessions = manager.list_sessions()
                
                # Calculate statistics
                total = len(sessions)
                by_status = {}
                by_type = {}
                
                for s in sessions:
                    by_status[s.status.value] = by_status.get(s.status.value, 0) + 1
                    by_type[s.session_type.value] = by_type.get(s.session_type.value, 0) + 1
                    
                text = f"""## MAOS Usage Statistics

**Total Sessions**: {total}
**Workspace**: {manager.base_dir}

### By Status
"""
                for status, count in by_status.items():
                    text += f"- {status}: {count}\n"
                
                text += "\n### By Type\n"
                for type_name, count in by_type.items():
                    text += f"- {type_name}: {count}\n"
                    
                return [TextContent(type="text", text=text)]
                
        elif cmd_name == "session":
            @self.mcp.tool(name="maos/session")
            async def session_tool(action: str, session_id: str = "none") -> List[TextContent]:
                """Manage MAOS sessions (list, show, cleanup)."""
                logger.info(f"Session tool called with action={action}, session_id={session_id}")
                manager = SessionManager()
                
                if action == "list":
                    sessions = manager.list_sessions()
                    
                    if not sessions:
                        return [TextContent(type="text", text="No active sessions.")]
                    
                    # Format sessions as a table
                    text = "## Active Sessions\n\n"
                    text += "| Session ID | Type | Status | Created | Task |\n"
                    text += "|------------|------|--------|---------|------|\n"
                    
                    for s in sessions:
                        task_preview = s.task[:50] + "..." if len(s.task) > 50 else s.task
                        text += f"| {s.session_id[:8]}... | {s.session_type.value} | {s.status.value} | {s.created.strftime('%Y-%m-%d %H:%M')} | {task_preview} |\n"
                    
                    return [TextContent(type="text", text=text)]
                    
                elif action == "show" and session_id and session_id != "none":
                    session = manager.get_session(session_id)
                    if not session:
                        return [TextContent(type="text", text=f"Error: Session {session_id} not found")]
                        
                    # Read manifest for agent details
                    session_path = Path(".maos/sessions") / session_id
                    manifest_path = session_path / "manifest.json"
                    manifest = {}
                    if manifest_path.exists():
                        try:
                            with open(manifest_path) as f:
                                manifest = json.load(f)
                        except (json.JSONDecodeError, OSError) as e:
                            logger.warning(f"Failed to read manifest: {e}")
                            manifest = {}
                    
                    # Format session details
                    text = f"""## Session Details

**Session ID**: {session.session_id}
**Type**: {session.session_type.value}
**Status**: {session.status.value}
**Created**: {session.created.isoformat()}
**Workspace**: {session_path}

### Task
{session.task}
"""
                    
                    if manifest.get("agents"):
                        text += "\n### Agents\n"
                        for agent_id, agent_info in manifest["agents"].items():
                            text += f"\n**{agent_id}**\n"
                            text += f"- Role: {agent_info.get('role', 'unknown')}\n"
                            text += f"- Status: {agent_info.get('status', 'unknown')}\n"
                            text += f"- Created: {agent_info.get('created_at', 'unknown')}\n"
                            text += f"- Workspace: {agent_info.get('workspace', 'unknown')}\n"
                    
                    return [TextContent(type="text", text=text)]
                    
                elif action == "cleanup":
                    logger.info(f"Cleanup action with session_id='{session_id}' (type: {type(session_id)})")
                    if session_id == "all":
                        # Purge all MAOS data
                        import shutil
                        try:
                            if manager.base_dir.exists():
                                shutil.rmtree(manager.base_dir)
                            return [TextContent(type="text", text="✅ All MAOS data purged successfully.")]
                        except Exception as e:
                            return [TextContent(type="text", text=f"❌ Failed to purge data: {str(e)}")]
                    elif session_id and session_id != "none":
                        # Delete specific session
                        try:
                            session = manager.get_session(session_id)
                            if session:
                                workspace = Path(".maos/sessions") / session_id
                                if workspace.exists():
                                    import shutil
                                    shutil.rmtree(workspace)
                                
                                registry_file = Path(".maos/registry") / f"{session_id}.json"
                                if registry_file.exists():
                                    registry_file.unlink()
                                
                                return [TextContent(type="text", text=f"✅ Session {session_id} deleted successfully.")]
                            else:
                                return [TextContent(type="text", text=f"❌ Session {session_id} not found.")]
                        except Exception as e:
                            return [TextContent(type="text", text=f"❌ Failed to delete session: {str(e)}")]
                    else:
                        return [TextContent(type="text", text="❌ Session ID required for cleanup. Use 'all' to delete all sessions, or specify a session ID.")]
                            
                return [TextContent(type="text", text="❌ Invalid action or missing parameters.")]
    
    async def _run_agent_with_progress(self, role: str, task: str, ctx: Optional[Context] = None) -> Dict[str, Any]:
        """Run an agent with progress notifications."""
        logger.info(f"Starting agent {role} with task: {task[:100]}...")
        
        # Create session structure
        import uuid
        from datetime import datetime
        
        session_id = str(uuid.uuid4())
        session_path = Path(".maos/sessions") / session_id
        session_path.mkdir(parents=True, exist_ok=True)
        
        # Create directories
        (session_path / "agents").mkdir(exist_ok=True)
        (session_path / "shared").mkdir(exist_ok=True)
        (session_path / "messages").mkdir(exist_ok=True)
        (session_path / "logs").mkdir(exist_ok=True)
        
        # Create agent instance
        agent_instance = f"{role}-1"  # Simple numbering for MCP
        agent_workspace = session_path / "agents" / agent_instance
        agent_workspace.mkdir(exist_ok=True)
        
        # Log session creation
        with open(session_path / "logs" / "events.jsonl", "w") as f:
            event = {
                "timestamp": datetime.now().isoformat(),
                "event": "session_created",
                "data": {"task": task, "type": "agent"}
            }
            f.write(json.dumps(event) + "\n")
        
        # Create manifest
        manifest = {
            "session_id": session_id,
            "created_at": datetime.now().isoformat(),
            "agents": {
                agent_instance: {
                    "role": role,
                    "status": "active",
                    "created_at": datetime.now().isoformat(),
                    "workspace": str(agent_workspace)
                }
            }
        }
        
        with open(session_path / "manifest.json", "w") as f:
            json.dump(manifest, f, indent=2)
        
        # Build agent prompt
        prompt = f"""You are a {role} agent in the MAOS system.

Your task: {task}

IMPORTANT: Work in the project directory: {self.project_root}
You have full access to create, edit, and manage files.

When you complete the task, provide a brief summary of what you did."""
        
        # Build Claude command
        cmd = [
            "claude",
            "--model", "sonnet",
            "-p", prompt,
            "--dangerously-skip-permissions"
        ]
        
        try:
            # Log the command we're about to run
            logger.info(f"Launching Claude CLI with command: {' '.join(cmd)}")
            logger.info(f"Working directory: {self.project_root}")
            
            # Send initial progress notification
            if ctx:
                await ctx.report_progress(0.2, 1.0, f"Setting up {role} workspace...")
            
            # Run the process with streaming output
            process = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=str(self.project_root)
            )
            
            logger.info(f"Process started with PID: {process.pid}")
            
            # Send progress update
            if ctx:
                await ctx.report_progress(0.3, 1.0, f"Agent running (PID: {process.pid})...")
            
            # Collect output with streaming updates
            stdout_lines = []
            stderr_lines = []
            line_count = 0
            
            async def read_stream(stream, lines_list, stream_name):
                nonlocal line_count
                while True:
                    line = await stream.readline()
                    if not line:
                        break
                    
                    decoded_line = line.decode('utf-8')
                    lines_list.append(decoded_line)
                    line_count += 1
                    
                    # Log every line for debugging
                    if line_count <= 5 or line_count % 10 == 0:
                        logger.debug(f"{stream_name} line {line_count}: {decoded_line.strip()[:100]}")
                    
                    # Send progress updates periodically
                    if ctx and line_count % 5 == 0:
                        # Extract meaningful info from recent lines
                        recent_lines = lines_list[-3:]
                        preview = ' '.join(line.strip() for line in recent_lines[-3:])
                        if len(preview) > 100:
                            preview = preview[:100] + "..."
                        
                        progress = 0.3 + (0.6 * min(line_count / 100, 1))  # Progress from 0.3 to 0.9
                        await ctx.report_progress(
                            progress=progress,
                            total=1.0,
                            message=f"Processing... ({line_count} lines) - {preview}"
                        )
            
            # For the Inspector, we need to return quickly to avoid timeout
            # Start reading output but don't wait for completion
            asyncio.create_task(read_stream(process.stdout, stdout_lines, "stdout"))
            asyncio.create_task(read_stream(process.stderr, stderr_lines, "stderr"))
            
            # Wait a short time to see if it's a quick task
            try:
                return_code = await asyncio.wait_for(process.wait(), timeout=3.0)
                # Quick completion
                stdout = ''.join(stdout_lines)
                stderr = ''.join(stderr_lines)
                logger.info(f"Process completed quickly with return code: {return_code}")
            except asyncio.TimeoutError:
                # Still running - return immediately
                logger.info(f"Process {process.pid} still running after 3s - returning early")
                return_code = None
                stdout = ''.join(stdout_lines) if stdout_lines else "Agent is processing..."
                stderr = ''.join(stderr_lines)
            
            stdout = ''.join(stdout_lines)
            stderr = ''.join(stderr_lines)
            
            logger.info(f"Process completed with return code: {return_code}")
            
            # Send final progress
            if ctx:
                await ctx.report_progress(0.95, 1.0, "Finalizing results...")
            
            # Register session with SessionManager
            manager = SessionManager()
            session = manager.create_session(
                task=task,
                session_type=SessionType.AGENT,
                claude_session_id=session_id
            )
            
            logger.info(f"Session registered: {session_id}")
            
            # Save the output to the session logs
            with open(session_path / "logs" / "stdout.log", "w") as f:
                f.write(stdout)
            with open(session_path / "logs" / "stderr.log", "w") as f:
                f.write(stderr)
            
            if return_code is not None:
                # Process completed quickly
                output_lines = stdout.strip().split('\n') if stdout else []
                summary = "Task completed" if return_code == 0 else "Task failed"
                
                # Try to extract a better summary from the output
                if output_lines:
                    for line in reversed(output_lines[-20:]):
                        if any(phrase in line.lower() for phrase in ['completed', 'finished', 'created', 'updated', 'summary']):
                            summary = line.strip()
                            break
                
                return {
                    "status": "completed" if return_code == 0 else "failed",
                    "session_id": session_id,
                    "agent_instance": agent_instance,
                    "workspace": str(agent_workspace),
                    "summary": summary,
                    "output": stdout[:1000] + "..." if len(stdout) > 1000 else stdout,
                    "error": stderr if stderr else None,
                    "return_code": return_code
                }
            else:
                # Process still running - return immediately
                return {
                    "status": "running",
                    "session_id": session_id,
                    "agent_instance": agent_instance,
                    "workspace": str(agent_workspace),
                    "summary": "Agent launched and is processing in the background.",
                    "output": stdout[:500] if stdout else "Agent starting...",
                    "error": None,
                    "return_code": None,
                    "pid": process.pid
                }
                
        except Exception as e:
            logger.error(f"Failed to run agent {role}: {e}", exc_info=True)
            return {"status": "error", "error": str(e)}
        
    def run(self):
        """Run the MCP server using stdio transport."""
        # Run with stdio transport for Claude Code integration
        self.mcp.run(transport="stdio")


def main():
    """Main entry point."""
    logger.info(f"=== MCP SERVER STARTING - PID: {os.getpid()} ===")
    
    # Change to project root before creating server
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent.parent
    os.chdir(project_root)
    logger.info(f"Changed working directory to project root: {project_root}")
    
    server = MAOSMCPServer()
    logger.info("Server instance created, starting run()...")
    server.run()


if __name__ == "__main__":
    logger.info(f"Script started directly - PID: {os.getpid()}")
    main()