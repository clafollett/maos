# MAOS Planning Summary

## What is MAOS?

MAOS (Multi-Agent Orchestration System) is backend orchestration for Claude Code's native sub-agent capabilities. It operates invisibly through hooks to provide workspace isolation and coordination when Claude spawns multiple agents.

**Key Point**: Users NEVER interact with MAOS directly. They use Claude Code normally.

## Core Architecture

```
User → Claude Code → Spawns Sub-agents
                           ↓
                    MAOS Hooks Intercept
                           ↓
                    Backend Utilities:
                    • Create worktrees
                    • Manage coordination
                    • Prevent conflicts
```

## Implementation Plan

### Phase 1: Core Requirements (MVP)

These three components MUST be implemented first:

#### 1. Task Tool Interception
```python
# In pre_tool_use.py
if tool_name == "Task" and tool_input.get('subagent_type'):
    agent_type = tool_input['subagent_type']
    workspace = prepare_workspace(agent_type, session_id)
    tool_input['prompt'] += f"\n\nWork in: {workspace}/"
```

#### 2. Git Worktree Management
- Create isolated workspace for each agent
- Branch naming: `wrktr/session-{id}/{agent}`
- Directory: `worktrees/{agent}-session-{id}`
- Lock worktrees during use

#### 3. Coordination System with Observability
- Directory: `.maos/sessions/{session_id}/`
- Core Files (per session):
  - `session.json` - Session metadata and configuration
  - `agents.json` - Active agents in this session
  - `locks.json` - File locks for this session
  - `progress.json` - Task progress within session
- Observability Files (per session):
  - `orchestration.json` - Agent parent-child relationships
  - `timeline.json` - Event timeline for debugging
  - `metrics.json` - Performance and status metrics

### Phase 2: Enhanced Features

4. **File Lock Management** - Prevent editing conflicts
5. **Session Management** - Track and cleanup sessions
6. **Progress Tracking** - Monitor agent work

### Phase 3: Production Ready

7. **Error Handling** - Graceful failures
8. **Performance** - Sub-10ms hook execution
9. **Automation** - Cleanup and maintenance

## Technical Decisions

1. **Python Only** - No bash scripts (avoids chmod issues)
2. **File-Based** - JSON files, no database needed
3. **Git Native** - Leverage git worktrees for isolation
4. **Hook-Based** - All orchestration through Claude's hook system

## What We're NOT Building

- ❌ User-facing CLI or commands
- ❌ Complex orchestration framework
- ❌ MCP server architecture
- ❌ Rust implementation
- ❌ Database or external services

## Success Metrics

1. **Invisible Operation** - Users don't know MAOS exists
2. **Conflict Prevention** - No file editing collisions
3. **Clean Isolation** - Each agent has separate workspace
4. **Automatic Cleanup** - No manual maintenance needed

## File Structure

```
project/
├── .claude/hooks/
│   ├── pre_tool_use.py      # Main hook - intercepts Task tool
│   ├── post_tool_use.py     # Cleanup hook
│   └── utils/
│       ├── maos_backend.py  # Core utilities
│       ├── worktree_*.py    # Worktree management
│       └── coordination.py  # State management
├── .maos/
│   ├── active_session.json  # Points to current session ID
│   └── sessions/            # All session data
│       └── {session_id}/    # Per-session coordination
│           ├── session.json
│           ├── agents.json
│           ├── locks.json
│           ├── progress.json
│           ├── orchestration.json
│           ├── timeline.json
│           └── metrics.json
└── worktrees/              # Agent workspaces
    ├── backend-sess-123/
    ├── frontend-sess-123/
    └── tester-sess-123/
```

## Development Sequence

1. **Day 1**: Implement basic Task interception and worktree creation
2. **Day 2**: Add coordination files and session management
3. **Day 3**: Add file locking and progress tracking
4. **Day 4**: Testing and error handling
5. **Day 5**: Performance optimization and cleanup automation

## Testing Plan

1. **Basic Test**: User asks Claude to "Build a REST API"
   - Verify worktrees created
   - Check coordination files
   - Confirm agents work in isolation

2. **Conflict Test**: Multiple agents edit same file type
   - Verify lock warnings appear
   - Check no data loss

3. **Cleanup Test**: Complete a task
   - Verify worktrees removed
   - Check branches cleaned up

## Advanced Patterns

### Prompt Chaining Sub-Agents

Claude Code supports a powerful pattern where:
1. Primary orchestrator spawns initial sub-agents
2. Sub-agents return results to the orchestrator
3. Orchestrator analyzes results and spawns additional agents
4. Multiple rounds of parallel work based on discoveries

**Example Flow** (from video):
- Round 1: planner, analyzer(x2), validator, optimizer
- Round 2: executor(x2), monitor, reporter, cleaner
- Each round informed by previous results

**MAOS Implications**:
- Need to preserve worktrees between rounds
- Session management becomes more important
- Coordination files should track multi-phase work
- Parent-child relationships critical for observability

### Multi-Agent Observability

For complex orchestrations, MAOS needs to provide visibility into:

1. **Agent Relationships**
   ```json
   // .maos/sessions/sess-123/orchestration.json
   {
     "session_id": "sess-123",
     "rounds": [
       {
         "round": 1,
         "parent": "primary",
         "agents": ["planner", "analyzer-1", "analyzer-2", "validator", "optimizer"],
         "status": "complete"
       },
       {
         "round": 2,
         "parent": "primary",
         "triggered_by": "round-1-results",
         "agents": ["executor-1", "executor-2", "monitor", "reporter", "cleaner"],
         "status": "in-progress"
       }
     ]
   }
   ```

2. **Timeline Tracking**
   - When each agent started/completed
   - Dependencies between agents
   - Which agents triggered which others
   - Total orchestration time

3. **Debug Visibility**
   - View all agents in a session
   - See parent-child relationships
   - Track data flow between rounds
   - Identify bottlenecks or failures

## Key Insights from Sub-Agent Video

### 1. Sub-Agent Prompting Best Practices
- The `description` field is THE most critical - it determines when agents are called
- Use trigger keywords: "proactively", "must be used", explicit triggers like "if they say X"
- Remember: Sub-agents respond to primary agent, NOT the user
- Sub-agent prompts are SYSTEM prompts, not user prompts

### 2. Context Isolation Implications
- Sub-agents have ZERO context from user conversation
- "It's literally like calling a one-shot prompt to the sub agent"
- This validates our worktree isolation approach
- Must be explicit about what information to pass

### 3. Debugging Requirements
- "These sub agents are hard to debug... we have no idea what's going on"
- Validates our observability features (timeline, orchestration tracking)
- Need to capture inter-agent communication for debugging

### 4. Performance Through Focus
- "Just like a focus engineer, when you're focused on one thing, you perform better"
- Single-purpose agents with clear boundaries
- Avoid agent overload and dependency coupling

## Open Questions

1. How does Claude Code pass session context between agents?
2. What's the best way to detect task completion?
3. Should we implement branch protection for main?
4. How to handle multi-round agent orchestration in worktrees?
5. Can we capture sub-agent prompts/responses for debugging?

## Next Action

When ready to implement:
1. Start with Phase 1, Item 1: Task Tool Interception
2. Test with simple multi-agent request
3. Iterate based on actual Claude Code behavior