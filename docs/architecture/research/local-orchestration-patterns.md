# Local IDE Sub-Agent Orchestration Patterns

## Executive Summary

This document presents the recommended orchestration architecture for MAOS sub-agents in a local IDE environment. The design leverages Claude Code's native capabilities with minimal backend support through hooks.

**Key Architecture**: Claude Code's native orchestration + hook-based backend utilities + git worktree isolation = effective parallel development without user-facing complexity.

## Context & Requirements

### Environment Constraints
- **Single machine, single user** (developer's workstation)
- **Local IDE context** (VS Code with Claude Code)
- **Git worktree isolation** for parallel work
- **Claude Code's native Task tool** for sub-agent spawning
- **Hook system** for operation interception
- **No distributed systems complexity** needed

### Key Challenges
1. How to coordinate multiple Claude instances without complex infrastructure
2. Preventing semantic conflicts (beyond git conflicts)
3. Tracking progress across parallel work streams
4. Maintaining simplicity while ensuring reliability

## Recommended Architecture: Lightweight State Machine Orchestrator

### Core Design Principles

1. **File-Based State Management**: Use the filesystem as the source of truth
2. **Event-Driven Coordination**: Agents communicate through file-based events
3. **Lock-Free Design**: Use atomic file operations and timestamps
4. **Progressive Enhancement**: Start simple, add complexity only when needed

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         USER                                │
│              (Normal Claude Code usage)                     │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│               CLAUDE CODE ORCHESTRATOR                      │
│  • Receives user request                                    │
│  • Decides to parallelize                                   │
│  • Uses Task tool to spawn sub-agents                      │
└────────┬────────────────────┬────────────────────┬──────────┘
         │                    │                    │
    Hook intercepts      Task spawns         Sub-agents work
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────┐  ┌──────────────────┐  ┌────────────────┐
│   MAOS HOOKS    │  │   SUB-AGENTS     │  │  WORKTREES     │
│ Backend utility │  │ • Backend Eng    │  │ • backend/     │
│ calls for:      │  │ • Frontend Eng   │  │ • frontend/    │
│ • Worktrees     │  │ • QA Engineer    │  │ • qa/          │
│ • Coordination  │  └──────────────────┘  └────────────────┘
└─────────────────┘
│  ├── tasks/                (task assignments & status)      │
│  ├── events/               (event queue)                    │
│  └── locks/                (resource locks)                 │
└─────────────────┬───────────────────────────────────────────┘
                  │ Read/Write
                  ▼
┌─────────────────┬─────────────────┬─────────────────────────┐
│   Sub-Agent 1   │   Sub-Agent 2   │    Sub-Agent N         │
│  (Worktree 1)   │  (Worktree 2)   │   (Worktree N)         │
│                 │                 │                         │
│  • Reads tasks  │  • Reads tasks  │   • Reads tasks        │
│  • Updates      │  • Updates      │   • Updates            │
│    status       │    status       │     status             │
│  • Emits events │  • Emits events │   • Emits events       │
└─────────────────┴─────────────────┴─────────────────────────┘
```

## Orchestration Patterns

### 1. Workflow State Machine

```json
// .maos/coordination/workflow.json
{
  "workflow_id": "wf_abc123",
  "objective": "Build authentication system",
  "current_phase": "implementation",
  "state": "in_progress",
  "created_at": "2024-01-30T10:00:00Z",
  "phases": {
    "design": {
      "state": "completed",
      "started_at": "2024-01-30T10:00:00Z",
      "completed_at": "2024-01-30T10:30:00Z",
      "agents": ["architect_1"],
      "outputs": ["docs/auth-design.md"]
    },
    "implementation": {
      "state": "in_progress",
      "started_at": "2024-01-30T10:31:00Z",
      "agents": ["backend_eng_1", "frontend_eng_1"],
      "tasks": ["impl_api", "impl_ui"]
    },
    "testing": {
      "state": "pending",
      "dependencies": ["implementation"]
    }
  }
}
```

### 2. Task Assignment & Tracking

```json
// .maos/coordination/tasks/impl_api.json
{
  "task_id": "impl_api",
  "description": "Implement authentication API endpoints",
  "assigned_to": "backend_eng_1",
  "worktree": "worktrees/backend-issue-42-auth-api",
  "state": "in_progress",
  "priority": "high",
  "dependencies": [],
  "started_at": "2024-01-30T10:31:00Z",
  "last_update": "2024-01-30T10:45:00Z",
  "progress": 0.6,
  "files_scope": ["src/api/auth/", "tests/api/auth/"],
  "checkpoints": [
    {
      "time": "2024-01-30T10:35:00Z",
      "message": "Created auth controller skeleton",
      "progress": 0.2
    },
    {
      "time": "2024-01-30T10:45:00Z",
      "message": "Implemented login endpoint",
      "progress": 0.6
    }
  ]
}
```

### 3. Agent Registry

```json
// .maos/coordination/agents/backend_eng_1.json
{
  "agent_id": "backend_eng_1",
  "role": "backend_engineer",
  "session_id": "sess_def456",
  "worktree": "worktrees/backend-issue-42-auth-api",
  "status": "active",
  "started_at": "2024-01-30T10:31:00Z",
  "last_heartbeat": "2024-01-30T10:46:00Z",
  "current_task": "impl_api",
  "capabilities": ["api_development", "database_design", "testing"],
  "resource_usage": {
    "files_modified": 5,
    "lines_changed": 250
  }
}
```

### 4. Event-Driven Coordination

```json
// .maos/coordination/events/evt_1706610360_task_progress.json
{
  "event_id": "evt_1706610360",
  "type": "task_progress",
  "source": "backend_eng_1",
  "timestamp": "2024-01-30T10:46:00Z",
  "data": {
    "task_id": "impl_api",
    "progress": 0.6,
    "message": "Login endpoint complete, starting on registration"
  }
}

// .maos/coordination/events/evt_1706610400_dependency_ready.json
{
  "event_id": "evt_1706610400",
  "type": "dependency_ready",
  "source": "backend_eng_1",
  "timestamp": "2024-01-30T10:46:40Z",
  "data": {
    "dependency": "auth_api_spec",
    "consumers": ["frontend_eng_1"],
    "location": ".maos/shared/api-spec.json"
  }
}
```

### 5. Resource Locking (Lightweight)

```json
// .maos/coordination/locks/src_api_auth.lock
{
  "resource": "src/api/auth/**",
  "locked_by": "backend_eng_1",
  "locked_at": "2024-01-30T10:31:00Z",
  "reason": "Implementing authentication endpoints",
  "exclusive": false,  // false = warn on conflict, true = block
  "expires_at": "2024-01-30T12:31:00Z"  // 2-hour default
}
```

## Coordination Mechanisms

### 1. Task Claiming Pattern

```python
# Pseudo-code for task claiming
def claim_next_task(agent_id, capabilities):
    tasks_dir = ".maos/coordination/tasks/"
    
    for task_file in sorted(glob(f"{tasks_dir}/*.json")):
        with FileLock(f"{task_file}.lock"):
            task = read_json(task_file)
            
            if task["state"] == "pending" and \
               task_matches_capabilities(task, capabilities):
                # Atomic claim
                task["state"] = "claimed"
                task["assigned_to"] = agent_id
                task["claimed_at"] = utcnow()
                write_json(task_file, task)
                return task
    
    return None
```

### 2. Progress Monitoring Pattern

```python
# Orchestrator monitors all agents
def monitor_agent_progress():
    agents_dir = ".maos/coordination/agents/"
    timeout = 300  # 5 minutes
    
    for agent_file in glob(f"{agents_dir}/*.json"):
        agent = read_json(agent_file)
        
        if (utcnow() - agent["last_heartbeat"]) > timeout:
            # Agent is stale
            handle_stale_agent(agent)
        
        # Check task progress
        if agent["current_task"]:
            task = read_task(agent["current_task"])
            update_workflow_progress(task)
```

### 3. Dependency Resolution Pattern

```python
# Check if dependencies are satisfied
def check_dependencies(task_id):
    task = read_task(task_id)
    
    for dep in task.get("dependencies", []):
        if isinstance(dep, str) and dep.startswith("task:"):
            dep_task = read_task(dep[5:])
            if dep_task["state"] != "completed":
                return False
        elif isinstance(dep, str) and dep.startswith("file:"):
            if not os.path.exists(dep[5:]):
                return False
    
    return True
```

### 4. Conflict Prevention Pattern

```python
# Hook integration for conflict prevention
def pre_file_edit_hook(agent_id, file_path):
    # Check if file is locked
    lock = find_lock_for_path(file_path)
    
    if lock and lock["locked_by"] != agent_id:
        if lock["exclusive"]:
            return {"decision": "block", 
                    "reason": f"File locked by {lock['locked_by']}"}
        else:
            return {"decision": "warn",
                    "reason": f"File being edited by {lock['locked_by']}"}
    
    # Check semantic boundaries
    agent = read_agent(agent_id)
    if not file_in_agent_scope(file_path, agent):
        return {"decision": "warn",
                "reason": "File outside assigned scope"}
    
    return {"decision": "allow"}
```

## Implementation Patterns

### 1. Orchestrator Main Loop

```python
async def orchestrator_main_loop():
    workflow = load_workflow()
    
    while workflow["state"] != "completed":
        # Check current phase
        current_phase = workflow["phases"][workflow["current_phase"]]
        
        # Spawn agents for pending tasks
        for task_id in current_phase.get("tasks", []):
            task = read_task(task_id)
            if task["state"] == "pending" and check_dependencies(task_id):
                agent_id = spawn_agent_for_task(task)
                task["state"] = "assigned"
                task["assigned_to"] = agent_id
                write_task(task_id, task)
        
        # Process events
        for event in read_new_events():
            handle_event(event, workflow)
        
        # Check phase completion
        if all_tasks_completed(current_phase):
            advance_to_next_phase(workflow)
        
        # Health checks
        check_agent_health()
        cleanup_stale_resources()
        
        await asyncio.sleep(5)  # Poll every 5 seconds
```

### 2. Sub-Agent Behavior

```python
# Sub-agent main loop
async def agent_main_loop(agent_id, role):
    register_agent(agent_id, role)
    
    while True:
        # Heartbeat
        update_heartbeat(agent_id)
        
        # Check for assigned tasks
        task = get_assigned_task(agent_id)
        if not task:
            task = claim_next_task(agent_id, get_capabilities(role))
        
        if task:
            # Update status
            update_task_state(task["task_id"], "in_progress")
            
            # Execute task
            result = await execute_task(task)
            
            # Report completion
            update_task_state(task["task_id"], "completed", result)
            emit_event("task_completed", {
                "task_id": task["task_id"],
                "outputs": result["outputs"]
            })
        else:
            # No tasks available
            await asyncio.sleep(10)
```

### 3. Sub-Command Integration

```bash
# maos orchestrate - Start orchestration
maos orchestrate "Build authentication system" \
  --phases design,implementation,testing \
  --parallel-limit 4

# maos status - Check orchestration status
maos status
# Output:
# Workflow: Build authentication system
# Phase: implementation (2/3)
# Active Agents:
#   - backend_eng_1: Implementing auth API (60% complete)
#   - frontend_eng_1: Building login UI (40% complete)
# Completed: design
# Pending: testing

# maos agent list - List active agents
maos agent list
# Output:
# ID              Role              Task          Status    Worktree
# backend_eng_1   backend_engineer  impl_api      active    worktrees/backend-auth
# frontend_eng_1  frontend_engineer impl_ui       active    worktrees/frontend-auth

# maos task assign - Manual task assignment
maos task assign impl_tests --to qa_engineer --priority high
```

## Trade-offs & Design Decisions

### 1. File-Based vs Memory-Based Coordination

**Choice: File-Based**

Pros:
- Persistent state survives crashes
- Easy debugging (just read files)
- Natural fit with git worktrees
- Simple implementation

Cons:
- Slightly slower than memory
- Need to handle file locking
- Polling overhead

**Mitigation**: Use file watching (inotify/FSEvents) for real-time updates

### 2. Lock-Free vs Locking Strategy

**Choice: Optimistic Lock-Free with Warnings**

Pros:
- No deadlocks
- Better developer experience
- Conflicts are rare in practice

Cons:
- Potential for conflicts
- Requires conflict resolution

**Mitigation**: Use exclusive locks only for critical resources

### 3. Push vs Pull Task Assignment

**Choice: Pull-Based (Agents Claim Tasks)**

Pros:
- Self-balancing load
- Agents work at their own pace
- Simpler orchestrator

Cons:
- Potential for task starvation
- Less control over assignment

**Mitigation**: Orchestrator can assign high-priority tasks directly

### 4. Orchestration Complexity

**Choice: Simple State Machine**

Pros:
- Easy to understand and debug
- Reliable and predictable
- Good enough for most workflows

Cons:
- Less flexible than full workflow engines
- Manual phase definition

**Mitigation**: Support custom workflow definitions for advanced users

## Prototype Code Snippets

### 1. Workflow Definition DSL

```yaml
# .maos/workflows/auth-system.yaml
name: "Authentication System"
phases:
  - name: design
    parallel: false
    tasks:
      - id: design_auth
        description: "Design authentication architecture"
        role: solution_architect
        outputs:
          - docs/auth-design.md
          - docs/api-spec.yaml
  
  - name: implementation
    parallel: true
    max_parallel: 3
    tasks:
      - id: impl_api
        description: "Implement auth API"
        role: backend_engineer
        dependencies: [design_auth]
        scope: ["src/api/auth/", "tests/api/auth/"]
      
      - id: impl_ui
        description: "Build auth UI"
        role: frontend_engineer
        dependencies: [design_auth]
        scope: ["src/ui/auth/", "tests/ui/auth/"]
      
      - id: impl_db
        description: "Create auth database schema"
        role: data_architect
        dependencies: [design_auth]
        scope: ["db/migrations/", "db/seeds/"]
  
  - name: testing
    parallel: true
    tasks:
      - id: test_integration
        description: "Integration testing"
        role: qa_engineer
        dependencies: [impl_api, impl_ui, impl_db]
```

### 2. Event Handler Example

```python
class EventHandler:
    def __init__(self, workflow):
        self.workflow = workflow
        self.handlers = {
            "task_completed": self.handle_task_completed,
            "dependency_ready": self.handle_dependency_ready,
            "agent_failed": self.handle_agent_failed,
            "conflict_detected": self.handle_conflict
        }
    
    def handle_task_completed(self, event):
        task_id = event["data"]["task_id"]
        
        # Update workflow
        self.workflow.mark_task_completed(task_id)
        
        # Check for dependent tasks
        for task in self.workflow.get_pending_tasks():
            if task_id in task.get("dependencies", []):
                if self.workflow.check_dependencies(task["id"]):
                    self.emit_event("task_ready", {"task_id": task["id"]})
    
    def handle_conflict(self, event):
        # Log conflict
        logger.warning(f"Conflict detected: {event['data']}")
        
        # Notify orchestrator
        self.workflow.add_issue({
            "type": "conflict",
            "agents": event["data"]["agents"],
            "resource": event["data"]["resource"]
        })
```

### 3. Hook Integration

```rust
// Rust hook for coordination
use maos_orchestration::{CoordinationClient, FileScope};

pub struct OrchestrationHook {
    client: CoordinationClient,
}

impl PreToolHook for OrchestrationHook {
    async fn validate(
        &self,
        tool_name: &str,
        tool_input: &Value,
    ) -> Result<HookResponse> {
        match tool_name {
            "Write" | "Edit" => {
                let file_path = tool_input["file_path"].as_str()?;
                let agent_id = get_agent_id_from_context()?;
                
                // Check coordination rules
                match self.client.check_file_access(agent_id, file_path).await? {
                    AccessDecision::Allow => Ok(HookResponse::allow()),
                    AccessDecision::Warn(msg) => {
                        eprintln!("Warning: {}", msg);
                        Ok(HookResponse::allow())
                    }
                    AccessDecision::Block(reason) => {
                        Ok(HookResponse::block(reason))
                    }
                }
            }
            _ => Ok(HookResponse::allow()),
        }
    }
}
```

## Recommendations

### 1. Start Simple
- Begin with basic file-based coordination
- Use simple JSON files for state
- Add complexity only when needed

### 2. Leverage Existing Tools
- Use git worktrees for isolation
- Integrate with Claude Code's Task tool
- Build on the existing hook system

### 3. Focus on Developer Experience
- Clear status visibility
- Helpful error messages
- Easy debugging

### 4. Plan for Growth
- Design can scale to more complex workflows
- Could add distributed coordination later
- Keep interfaces clean for future extensions

## Conclusion

This lightweight state machine orchestrator provides the right balance of simplicity and functionality for MAOS's local IDE context. By using file-based coordination and event-driven patterns, we can achieve reliable multi-agent orchestration without the complexity of distributed systems.

The design leverages proven patterns from systems like Airflow and Temporal but adapts them for single-machine use. The focus on developer experience and progressive enhancement ensures the system remains approachable while still being powerful enough for complex workflows.

Key advantages:
- **Simple**: No external dependencies or complex infrastructure
- **Reliable**: Persistent state and crash recovery
- **Flexible**: Supports various workflow patterns
- **Transparent**: Easy to understand and debug
- **Integrated**: Works naturally with Claude Code and git