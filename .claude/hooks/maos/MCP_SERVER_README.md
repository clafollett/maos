# MAOS MCP Server

This directory contains the Model Context Protocol (MCP) server implementation for MAOS, enabling Claude Code to interact with MAOS agents through native MCP tools.

## Overview

The MCP server exposes all MAOS agent commands as MCP tools, allowing Claude Code to:
- Launch agents without bash tool timeout limitations
- Receive real-time progress updates as agents work
- Get structured output from agent executions
- Manage sessions and view agent workspaces

## Files

- `maos_mcp_server_sync.py` - Synchronous MCP server that runs agents and returns complete output
- `maos_mcp_server.py` - Original async version (deprecated due to process lifecycle issues)
- `maos-mcp-config.json` - MCP configuration file for Claude Code

## Installation

1. Install the MCP Python SDK:
```bash
pip install 'mcp[cli]'
```

2. Copy the MCP config to Claude Code's config directory:
```bash
cp .claude/hooks/maos/maos-mcp-config.json ~/Library/Application\ Support/Code/User/globalStorage/anthropic.claude-code/settings/mcp-servers.json
```

Or merge it with your existing MCP servers configuration.

## Features

### Synchronous Execution
The server runs agents synchronously and waits for completion, ensuring:
- Full output capture
- Proper error handling
- No orphaned processes

### Progress Notifications
The server sends progress updates during agent execution:
- Initial launch notification
- Periodic updates with output preview
- Final completion status

### Structured Output
All tools return formatted TextContent blocks with:
- Status information
- Session details
- Agent output
- Error messages (if any)

### Session Management
The `maos/session` tool provides:
- `list` - View all active sessions
- `show` - Get details of a specific session
- `cleanup` - Remove sessions

## Usage

Once configured, you can use MAOS tools in Claude Code:

```
# Launch a backend engineer
Use the maos/backend-engineer tool to create a REST API

# Check sessions
Use the maos/session tool with action="list"

# View session details
Use the maos/session tool with action="show" and session_id="..."
```

## Transport

The server uses stdio transport, which means:
- Claude Code manages the server lifecycle
- Server starts when needed and stops when done
- No need to run a separate server process

## Architecture

```
Claude Code <-> MCP Protocol (stdio) <-> MAOS MCP Server <-> Claude CLI
```

The server acts as a bridge between Claude Code's MCP client and the Claude CLI, handling:
- Command translation
- Process management
- Output streaming
- Session tracking

## Debugging

Enable debug logging by setting:
```bash
export PYTHONUNBUFFERED=1
export FASTMCP_LOG_LEVEL=DEBUG
```

Server logs will appear in Claude Code's MCP output panel.