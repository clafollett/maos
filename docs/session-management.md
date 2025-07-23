# MAOS Session Management

## Overview

MAOS implements a session management system to properly scope agent work and enable resume capability.

## Session Types

### 1. Orchestration Sessions
Created when `/maos:orchestrator` is invoked:
- Tracks the entire orchestration run
- All spawned agents inherit this session
- Enables resume if the same task is requested again
- Task hashing ensures identical tasks reuse sessions

### 2. Standalone Agent Sessions  
Created when individual agents are invoked directly:
- Each agent gets its own session
- No cross-agent coordination
- Isolated workspace

## Session Flow

```
User: /maos:orchestrator Create a REST API

1. Orchestrator Hook:
   - Extracts task: "Create a REST API"
   - Creates task hash: "a1b2c3d4"
   - Checks for existing session with same hash
   - Creates new session: "uuid-1234"
   - Sets as current session

2. Agent Spawning:
   - Backend Engineer checks current session
   - Uses orchestration session "uuid-1234"
   - Creates workspace: .maos/sessions/uuid-1234/agents/backend-engineer/

3. Resume Capability:
   - If interrupted, running same command again:
     - Finds existing session by task hash
     - Resumes with same session ID
     - Agents continue in same workspace
```

## Session Lifecycle

1. **Creation**: 
   - New orchestration → New session
   - Standalone agent → New session (unless orchestration active)

2. **Active Period**:
   - Sessions remain active for 24 hours
   - Can be resumed within this window

3. **Completion**:
   - Mark complete with `/maos:session complete`
   - Automatic cleanup after 7 days

## Benefits

1. **Clean Separation**: Different tasks don't mix workspaces
2. **Resume Support**: Interrupted work can continue
3. **Traceability**: All agent work tied to specific sessions
4. **Resource Management**: Old sessions cleaned up automatically

## Commands

- `/maos:session list` - View active sessions
- `/maos:session complete` - Mark current session done
- `/maos:session cleanup` - Remove old sessions