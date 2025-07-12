# MAOS Orchestration Flow

This document describes the complete flow of orchestration in MAOS, from user request to task completion.

## Overview

MAOS orchestration follows a simple but powerful pattern:
1. User makes a natural language request
2. Orchestrator agent analyzes and plans
3. MAOS executes the plan by spawning agents
4. Agents work and coordinate through files
5. Results stream back to the user

## Detailed Flow

### Step 1: User Request

The user invokes orchestration through their CLI tool:

```
/orchestrate Build a REST API for managing blog posts with authentication
```

### Step 2: MCP Tool Invocation

The CLI tool (Claude Code, Aider, etc.) recognizes the `/orchestrate` command and calls the MAOS MCP tool with the request.

### Step 3: Orchestrator Agent Creation

MAOS immediately spawns an Orchestrator agent with a crafted prompt:

```rust
let orchestrator_prompt = format!(
    r#"You are the Orchestrator agent for MAOS (Multi-Agent Orchestration System).
    
    User Request: "{}"
    
    Analyze this request and create an execution plan. Consider:
    - What needs to be researched or designed first?
    - What can be done in parallel?
    - What are the dependencies between tasks?
    - Which agent roles are best suited for each task?
    
    Output a JSON execution plan with phases and agents.
    
    Available roles: researcher, architect, engineer, qa_engineer, 
    security_reviewer, documenter, ux_designer, devops, data_scientist
    
    Output ONLY valid JSON in this format:
    {{
      "phases": [
        {{
          "name": "Phase Name",
          "parallel": boolean,
          "agents": [
            {{"role": "role_name", "task": "specific task"}}
          ]
        }}
      ]
    }}"#,
    user_request
);
```

### Step 4: Execution Plan Generation

The Orchestrator agent (powered by Claude, GPT-4, etc.) analyzes the request and outputs a structured plan:

```json
{
  "phases": [
    {
      "name": "Research and Design",
      "parallel": false,
      "agents": [
        {
          "role": "researcher",
          "task": "Research best practices for blog APIs, authentication methods, and recommended tech stacks"
        },
        {
          "role": "architect",
          "task": "Design REST API architecture with authentication, including endpoints, data models, and security approach"
        }
      ]
    },
    {
      "name": "Implementation",
      "parallel": true,
      "agents": [
        {
          "role": "engineer",
          "task": "Implement authentication system with JWT tokens and user management"
        },
        {
          "role": "engineer",
          "task": "Implement blog post CRUD operations with proper authorization"
        },
        {
          "role": "engineer",
          "task": "Create database schema and migration scripts"
        }
      ]
    },
    {
      "name": "Quality Assurance",
      "parallel": true,
      "agents": [
        {
          "role": "qa_engineer",
          "task": "Write comprehensive tests for all API endpoints"
        },
        {
          "role": "security_reviewer",
          "task": "Review authentication implementation and API security"
        },
        {
          "role": "documenter",
          "task": "Create API documentation with examples"
        }
      ]
    }
  ]
}
```

### Step 5: Plan Execution

MAOS parses the JSON plan and executes it phase by phase:

```rust
for phase in execution_plan.phases {
    info!("Starting phase: {}", phase.name);
    
    if phase.parallel {
        // Spawn all agents in this phase simultaneously
        let handles: Vec<_> = phase.agents
            .into_iter()
            .map(|agent_spec| {
                tokio::spawn(async move {
                    spawn_agent(agent_spec.role, agent_spec.task).await
                })
            })
            .collect();
            
        // Wait for all parallel agents to complete
        for handle in handles {
            handle.await??;
        }
    } else {
        // Execute agents sequentially
        for agent_spec in phase.agents {
            spawn_agent(agent_spec.role, agent_spec.task).await?;
            wait_for_completion().await?;
        }
    }
}
```

### Step 6: Agent Execution

Each spawned agent:
1. Receives its role and specific task
2. Works in its isolated workspace
3. Reads from shared context for coordination
4. Writes outputs to shared locations
5. Sends status updates

### Step 7: Progress Streaming

MAOS monitors all agents and streams progress back to the user:

```
[Orchestrator] Creating execution plan...
[Orchestrator] Plan created with 3 phases

[Phase 1: Research and Design]
[Researcher] Investigating blog API patterns...
[Researcher] Found 5 authentication strategies...
[Architect] Designing API structure...
[Architect] Created OpenAPI specification...

[Phase 2: Implementation]
[Engineer-1] Implementing JWT authentication...
[Engineer-2] Creating blog post endpoints...
[Engineer-3] Setting up PostgreSQL schema...
```

### Step 8: Completion

When all phases complete, MAOS returns the final result to the user with:
- Summary of what was accomplished
- Location of all deliverables
- Any important notes or warnings

## Coordination Patterns

### File-Based Communication

Agents coordinate through the filesystem:

```
~/.maos/sessions/{session_id}/
├── shared_context/           # Shared deliverables
│   ├── architecture/         # Architectural decisions
│   │   └── api-design.md
│   ├── implementation/       # Code implementations
│   │   ├── auth/
│   │   └── blog/
│   └── documentation/        # Docs and specs
├── workspaces/              # Individual agent workspaces
│   ├── agent_researcher_1/
│   ├── agent_architect_1/
│   └── agent_engineer_1/
└── messages/                # Inter-agent messages
    └── message_queue.json
```

### Message Passing

Agents can send messages to each other:

```json
{
  "from": "agent_engineer_1",
  "to": "agent_architect_1",
  "timestamp": "2024-01-15T10:30:00Z",
  "subject": "Clarification needed on auth flow",
  "body": "The JWT refresh token flow isn't clear in the design. Should we use..."
}
```

### Dynamic Sub-Orchestration

When agents need help, they can trigger sub-orchestration:

```
Engineer: "I need help choosing between REST and GraphQL for this API"
    ↓
MAOS: Spawns mini-orchestrator
    ↓
Mini-Orchestrator: "Spawn a researcher to compare REST vs GraphQL for blog APIs"
    ↓
Researcher: Investigates and reports findings
    ↓
Engineer: Continues with informed decision
```

## Error Handling

### Orchestrator Failures
If the Orchestrator agent fails to produce valid JSON:
1. Retry with clarified prompt
2. Fall back to simple sequential execution
3. Report error to user

### Agent Failures
If an individual agent fails:
1. Log the error
2. Attempt to continue with other agents
3. Mark phase as partially complete
4. Report failures in final summary

### Deadlock Prevention
- Agents have timeouts
- File locks are released on crash
- Circular dependencies are detected

## Examples

### Example 1: Simple Feature
```
Request: "Add user profile management to the API"

Orchestrator creates:
- Phase 1: Design (sequential)
  - Architect: Design profile endpoints and data model
- Phase 2: Implementation (parallel)
  - Engineer: Implement profile CRUD operations
  - QA: Write tests for profile endpoints
```

### Example 2: Complex System
```
Request: "Build a real-time collaborative document editor"

Orchestrator creates:
- Phase 1: Research (sequential)
  - Researcher: Research CRDT algorithms and real-time sync
- Phase 2: Architecture (sequential)
  - Architect: Design system with WebSockets and conflict resolution
- Phase 3: Implementation (parallel)
  - Engineer: Implement CRDT engine
  - Engineer: Build WebSocket server
  - Engineer: Create editor UI
  - Engineer: Set up document storage
- Phase 4: Integration (sequential)
  - Lead Engineer: Integrate all components
- Phase 5: Quality (parallel)
  - QA: Test conflict resolution
  - QA: Test real-time sync
  - Security: Review access control
```

## Best Practices

1. **Clear Task Descriptions**: The Orchestrator should create specific, actionable tasks
2. **Appropriate Parallelization**: Only parallelize truly independent tasks
3. **Logical Phases**: Group related work into phases
4. **Resource Awareness**: Don't spawn too many agents at once
5. **Dependency Management**: Sequential phases for dependent work

## Summary

The orchestration flow is designed to be:
- **Simple**: Natural language in, working system out
- **Flexible**: Handles any request through intelligent planning
- **Transparent**: All work visible in the filesystem
- **Efficient**: Parallel execution where appropriate
- **Reliable**: Clear error handling and recovery