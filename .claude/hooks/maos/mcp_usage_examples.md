# MAOS MCP Server Usage Examples

## Overview

The MAOS MCP Server mirrors all `/maos:` Claude Code commands as MCP tools, providing the same interface without timeout limitations.

## Available MCP Tools

### Core Orchestration
- `maos/orchestrator` - Create orchestration session to coordinate multiple agents
- `maos/session` - Manage sessions (list, show, cleanup)
- `maos/current` - Show current orchestration session
- `maos/usage` - Show usage statistics

### Agent Tools (mirrors /maos: commands)
- `maos/api-architect` - API design and contracts
- `maos/application-architect` - Application architecture design
- `maos/backend-engineer` - Backend/server implementation
- `maos/business-analyst` - Business requirements analysis
- `maos/data-architect` - Data architecture and modeling
- `maos/data-scientist` - Data analysis and ML
- `maos/devops` - Infrastructure and deployment
- `maos/frontend-engineer` - Frontend/UI implementation
- `maos/mobile-engineer` - Mobile app development
- `maos/pm` - Product management
- `maos/qa` - Quality assurance and testing
- `maos/researcher` - Research and analysis
- `maos/reviewer` - Code and design reviews
- `maos/secops` - Security operations
- `maos/security-architect` - Security architecture
- `maos/solution-architect` - Solution design
- `maos/techwriter` - Technical documentation
- `maos/tester` - Testing and test automation
- `maos/ux-designer` - User experience design

## Usage Patterns

### 1. Direct Agent Usage (No Orchestrator)
```
Use the maos/backend-engineer tool with task: "Create a REST API for user management"
```

### 2. Orchestrated Multi-Agent Flow
```
Step 1: Create orchestration session
Use the maos/orchestrator tool with task: "Build a complete user authentication system"

Step 2: Launch coordinated agents
Use the maos/api-architect tool with task: "Design API contracts for authentication"
Use the maos/backend-engineer tool with task: "Implement authentication API"
Use the maos/frontend-engineer tool with task: "Create login/signup UI"
Use the maos/qa tool with task: "Create test plan for authentication"

Step 3: Monitor progress
Use the maos/session tool with action: "list"
Use the maos/session tool with action: "show" and session_id: "[session-id]"
```

### 3. Code Review Example
```
For reviewing the MAOS Python code:

Use the maos/reviewer tool with task: "Review Python code quality in .claude/hooks/maos/"
Use the maos/security-architect tool with task: "Analyze security vulnerabilities" 
Use the maos/backend-engineer tool with task: "Implement fixes from code review"
```

### 4. Session Management
```
List all sessions:
Use the maos/session tool with action: "list"

Show specific session:
Use the maos/session tool with action: "show" and session_id: "abc123"

Cleanup all sessions:
Use the maos/session tool with action: "cleanup" and session_id: "--all"
```

## Key Benefits

1. **Maintains Same Interface** - Tools mirror exact /maos: commands
2. **No Timeout Issues** - MCP server handles long-running tasks
3. **Orchestrator Preserved** - Still coordinate multiple agents
4. **Background Execution** - Agents run async, check status later
5. **Session Management** - Full lifecycle control

## MCP vs Claude Code Commands

| Claude Code Command | MCP Tool | Notes |
|-------------------|----------|-------|
| `/maos:orchestrator` | `maos/orchestrator` | Creates orchestration session |
| `/maos:backend-engineer` | `maos/backend-engineer` | Direct agent access |
| `/maos:session list` | `maos/session` with `action: "list"` | Parameter-based |
| `/maos:current` | `maos/current` | No parameters needed |

## Setting Up

1. Install MCP package:
   ```bash
   pip install mcp
   ```

2. Configure Claude Code with the provided `maos-mcp-config.json`

3. Tools will appear in Claude Code as `maos/*` instead of `/maos:`

## Important Notes

- All agents still use the same orchestration infrastructure
- Shared context and session management work identically
- Agents coordinate through `.maos/sessions/{id}/shared/`
- The orchestrator creates a session that agents can join