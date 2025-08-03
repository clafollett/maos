---
allowed-tools: Bash, Read, Task
description: Session management - list, show, resume, fork, archive, cleanup, status
argument-hint: list [status]|show <id>|resume <id>|fork <id>|archive <id>|cleanup [--all|<id>]|status <id> <new_status>
---

# MAOS Session Manager

Advanced session management with full lifecycle control.

## Design Philosophy

Every execution creates a **unique, immutable session**. Sessions are never reused automatically - you explicitly choose to resume, fork, or start fresh.

## Commands

### List Sessions
```bash
# List all sessions
python3 .claude/hooks/maos/session_manager.py list

# List only active sessions  
python3 .claude/hooks/maos/session_manager.py list active

# List completed sessions
python3 .claude/hooks/maos/session_manager.py list completed
```

### Show Session Details
```bash
# Show full details (supports partial IDs)
python3 .claude/hooks/maos/session_manager.py show "$ARGUMENTS"
```

### Resume a Session
```bash
# Get resume information
python3 .claude/hooks/maos/session_manager.py resume "$ARGUMENTS"
```

Then use the Task tool to resume the orchestration with the session context.

### Fork a Session
```bash
# Create a new session based on an existing one
python3 .claude/hooks/maos/session_manager.py fork "$ARGUMENTS"
```

### Archive a Session
```bash
# Archive completed sessions to save space
python3 .claude/hooks/maos/session_manager.py archive "$ARGUMENTS"
```

### Clean Up Sessions
```bash
# Remove sessions older than 7 days
python3 .claude/hooks/maos/session_manager.py cleanup

# Delete a specific session
python3 .claude/hooks/maos/session_manager.py cleanup <session_id>

# Purge ALL MAOS data (removes entire .maos directory)
python3 .claude/hooks/maos/session_manager.py cleanup --all
```

### Update Session Status
```bash
# Change session status (active|completed|failed|timeout|cancelled|paused)
python3 .claude/hooks/maos/session_manager.py status <session_id> <new_status>
```

## Workflow Examples

### 1. Normal Execution
```
User: /maos:orchestrator Build a REST API
→ Creates session: abc123...
→ Agents work in .maos/sessions/abc123/
→ Status: completed
```

### 2. Interrupted & Resume
```
User: /maos:orchestrator Build a REST API  
→ Creates session: abc123...
→ Timeout after backend agent
→ Status: timeout

User: /maos:session show abc123
→ Shows what was completed

User: /maos:session resume abc123
→ Provides resume context
→ Can continue where left off
```

### 3. Fork & Experiment
```
User: /maos:session fork abc123
→ Creates session: def456...
→ Copies shared context
→ Try different approach

User: Compare both implementations!
```

### 4. Review Past Work
```
User: /maos:session list completed
→ See all finished projects

User: /maos:session show xyz789
→ Review what was built
→ See workspace location
```

## Session States

- **active**: Currently running
- **completed**: Successfully finished
- **failed**: Error occurred
- **timeout**: Execution timed out
- **cancelled**: User cancelled
- **paused**: Manually paused for later

## Benefits

1. **Every run is unique**: No accidental mixing of different attempts
2. **Full history**: Review any past execution
3. **Explicit control**: You decide when to resume vs start fresh
4. **Experimentation**: Fork sessions to try variations
5. **Clean workspaces**: Each attempt has its own isolated workspace

## Execute Command

```bash
cd "$(pwd)" && python3 .claude/hooks/maos/session_manager.py "$ARGUMENTS"
```