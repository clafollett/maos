# The Orchestrator Agent

## Overview

The Orchestrator is a special agent that is automatically spawned whenever a user invokes the `/orchestrate` command. Unlike other agents that perform specific tasks, the Orchestrator's sole purpose is to understand the user's request and create an intelligent execution plan for other agents to follow.

## Key Concepts

### Implied Agent
The Orchestrator is "implied" - users never explicitly request it, but it's always the first agent spawned in any orchestration workflow. It acts as the intelligence layer that translates natural language requests into structured execution plans.

### Leveraging CLI Intelligence
Rather than building complex natural language processing into MAOS itself, the Orchestrator leverages the underlying CLI tool (Claude Code, Aider, etc.) to understand requests and plan execution. MAOS stays simple - it just orchestrates.

## How It Works

### 1. User Request
```
/orchestrate Build a real-time chat application with user authentication
```

### 2. Orchestrator Spawning
MAOS automatically spawns an Orchestrator agent with a carefully crafted prompt:

```
You are the Orchestrator agent for MAOS. Analyze this request and create
an execution plan with phases, specifying which agents to spawn and whether
they should run in parallel or sequentially.

User Request: "Build a real-time chat application with user authentication"

Output a JSON execution plan only.
```

### 3. Execution Plan Generation
The Orchestrator (powered by Claude or another CLI) analyzes the request and outputs:

```json
{
  "phases": [
    {
      "name": "Research and Architecture",
      "parallel": false,
      "agents": [
        {
          "role": "researcher",
          "task": "Research real-time messaging technologies and authentication best practices"
        },
        {
          "role": "solution_architect",
          "task": "Design system architecture based on research findings"
        }
      ]
    },
    {
      "name": "Implementation",
      "parallel": true,
      "agents": [
        {
          "role": "backend_engineer",
          "task": "Implement WebSocket server with JWT authentication"
        },
        {
          "role": "frontend_engineer",
          "task": "Build chat UI with real-time updates"
        },
        {
          "role": "data_architect",
          "task": "Set up message persistence and user storage"
        }
      ]
    }
  ]
}
```

### 4. Plan Execution
MAOS parses the JSON plan and executes it exactly as specified, spawning agents in the correct order and parallelism.

## Benefits

### Simplicity
MAOS doesn't need complex pattern matching or natural language understanding. It just needs to:
- Spawn an Orchestrator
- Parse JSON output
- Execute the plan

### Flexibility
The Orchestrator can handle any request because it uses the full intelligence of the underlying CLI tool. No predefined patterns or limitations.

### Adaptability
Different CLI tools might have different strengths. The Orchestrator leverages whatever tool the user prefers (Claude, GPT-4, etc.).

## Dynamic Orchestration

The Orchestrator pattern also enables dynamic sub-orchestration:

```rust
// When an agent needs help
Backend Engineer: "I need help choosing between PostgreSQL and MongoDB for message storage"

// MAOS spawns a mini-orchestrator
Mini-Orchestrator: "A backend engineer needs database selection help. 
                   Spawn a researcher to compare options."

// Result: New researcher agent spawned to help
```

## Implementation Details

### Orchestrator Prompt Template
```
You are the Orchestrator agent for MAOS (Multi-Agent Orchestration System).

Your job is to:
1. Understand the user's request
2. Break it down into phases
3. Determine which agents are needed
4. Decide what can run in parallel

User Request: "{}"

Output a JSON execution plan with this structure:
{
  "phases": [
    {
      "name": "Phase Name",
      "parallel": boolean,
      "agents": [
        {
          "role": "agent_role",
          "task": "Specific task description"
        }
      ]
    }
  ]
}

Consider:
- Dependencies between tasks
- Optimal parallelization
- Appropriate agent roles
- Clear task descriptions

Output ONLY valid JSON.
```

### Supported Agent Roles
- **Architecture**: solution_architect, application_architect, data_architect, api_architect, security_architect
- **Engineering**: backend_engineer, frontend_engineer, mobile_engineer
- **Analysis**: researcher, data_scientist, analyst
- **Quality**: qa, reviewer, tester
- **Coordination**: pm, documenter
- **Specialized**: devops, security, ux_designer

### Error Handling
If the Orchestrator fails to produce valid JSON or crashes, MAOS should:
1. Retry with a clarified prompt
2. Fall back to a simple sequential plan
3. Report the error to the user

## Examples

### Example 1: API Development
```
Request: "Create a REST API for user management"

Orchestrator Output:
{
  "phases": [
    {
      "name": "Design",
      "parallel": false,
      "agents": [
        {"role": "api_architect", "task": "Design REST API structure and endpoints"}
      ]
    },
    {
      "name": "Implementation",
      "parallel": true,
      "agents": [
        {"role": "backend_engineer", "task": "Implement user CRUD operations"},
        {"role": "backend_engineer", "task": "Implement authentication endpoints"},
        {"role": "data_architect", "task": "Create user database schema"}
      ]
    }
  ]
}
```

### Example 2: Codebase Analysis
```
Request: "Analyze this codebase for security vulnerabilities"

Orchestrator Output:
{
  "phases": [
    {
      "name": "Analysis",
      "parallel": true,
      "agents": [
        {"role": "security", "task": "Scan for SQL injection vulnerabilities"},
        {"role": "security", "task": "Check authentication implementation"},
        {"role": "security", "task": "Review API security headers"},
        {"role": "security", "task": "Analyze dependency vulnerabilities"}
      ]
    },
    {
      "name": "Report",
      "parallel": false,
      "agents": [
        {"role": "documenter", "task": "Compile security findings into report"}
      ]
    }
  ]
}
```

## Future Enhancements

1. **Learning**: Orchestrator could learn from successful plans
2. **Templates**: Common patterns could be suggested
3. **Feedback Loop**: Agents could report back to refine plans
4. **Cost Optimization**: Consider resource usage in planning

## Summary

The Orchestrator Agent is the brain of MAOS - it translates human intent into executable plans. By leveraging existing CLI tool intelligence rather than building our own NLP, MAOS stays simple, flexible, and powerful.