# MAOS MCP Tools Reference

## Overview

This document consolidates all Model Context Protocol (MCP) tool and resource definitions exposed by MAOS. It serves as the single source of truth for the MCP API surface.

## MCP Server Configuration

```json
{
  "name": "maos",
  "version": "1.0.0",
  "description": "Multi-Agent Orchestration System",
  "transport": {
    "type": "http",
    "port": 3000,
    "features": ["sse"]
  },
  "tools": [
    "maos/orchestrate",
    "maos/session-status",
    "maos/list-roles"
  ],
  "resources": ["sessions", "agents", "templates"]
}
```

## Tools

### 1. maos/orchestrate

Start a multi-agent orchestration session with multiple agents working on a common objective.

```json
{
  "name": "maos/orchestrate",
  "description": "Start a multi-agent orchestration session",
  "inputSchema": {
    "type": "object",
    "properties": {
      "objective": {
        "type": "string",
        "description": "High-level goal to accomplish"
      },
      "agents": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "role": {
              "oneOf": [
                {
                  "type": "string",
                  "enum": ["architect", "engineer", "researcher", "qa", "pm", 
                           "devops", "security", "data_scientist", "ux_designer", 
                           "documenter", "reviewer", "analyst", "tester"],
                  "description": "Predefined agent role"
                },
                {
                  "type": "object",
                  "properties": {
                    "name": {
                      "type": "string",
                      "description": "Custom role name"
                    },
                    "description": {
                      "type": "string",
                      "description": "Brief role overview"
                    },
                    "responsibilities": {
                      "type": "string",
                      "description": "Detailed list of responsibilities"
                    }
                  },
                  "required": ["name", "description"],
                  "description": "Custom agent role definition"
                }
              ]
            },
            "task": {
              "type": "string",
              "description": "Specific task for this agent"
            },
            "instance_suffix": {
              "type": "string",
              "description": "Optional suffix for agent identification (e.g., 'frontend', 'backend')"
            }
          },
          "required": ["role", "task"]
        }
      },
      "strategy": {
        "type": "string",
        "enum": ["parallel", "sequential", "adaptive", "pipeline"],
        "default": "parallel"
      },
      "max_agents_per_role": {
        "type": "object",
        "description": "Maximum number of agents per role name",
        "additionalProperties": {
          "type": "integer",
          "minimum": 1
        }
      }
    },
    "required": ["objective"]
  }
}
```

**Example Usage:**
```json
{
  "objective": "Build a REST API for user management",
  "agents": [
    {
      "role": "api_architect",
      "task": "Design the API architecture and database schema"
    },
    {
      "role": "backend_engineer",
      "task": "Implement the API endpoints",
      "instance_suffix": "backend"
    },
    {
      "role": "qa",
      "task": "Write comprehensive tests"
    }
  ],
  "strategy": "sequential"
}
```

**Response:**
```json
{
  "session_id": "sess_abc123",
  "status": "started",
  "agents": [
    {
      "agent_id": "agent_architect_1_def456",
      "role": "api_architect",
      "status": "pending"
    },
    {
      "agent_id": "agent_engineer_backend_1_ghi789",
      "role": "backend_engineer",
      "status": "pending"
    },
    {
      "agent_id": "agent_qa_1_jkl012",
      "role": "qa",
      "status": "pending"
    }
  ]
}
```

### 2. maos/session-status

Get the current status of an orchestration session and its agents.

```json
{
  "name": "maos/session-status",
  "description": "Get the current status of an orchestration session",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_id": { 
        "type": "string",
        "description": "Session identifier"
      },
      "include_agents": { 
        "type": "boolean", 
        "default": true,
        "description": "Include detailed agent information"
      },
      "include_messages": { 
        "type": "boolean", 
        "default": false,
        "description": "Include recent inter-agent messages"
      }
    },
    "required": ["session_id"]
  }
}
```

**Response Example:**
```json
{
  "session_id": "sess_abc123",
  "objective": "Build a REST API for user management",
  "state": "executing",
  "strategy": "sequential",
  "progress": {
    "total_agents": 3,
    "completed": 1,
    "running": 1,
    "pending": 1
  },
  "agents": [
    {
      "agent_id": "agent_architect_1_def456",
      "role": "api_architect",
      "state": "completed",
      "started_at": "2024-01-10T10:00:00Z",
      "completed_at": "2024-01-10T10:15:00Z"
    },
    {
      "agent_id": "agent_engineer_backend_1_ghi789",
      "role": "backend_engineer",
      "state": "running",
      "started_at": "2024-01-10T10:15:00Z"
    }
  ]
}
```

### 3. maos/list-roles

List available predefined roles and active custom roles.

```json
{
  "name": "maos/list-roles",
  "description": "List available predefined roles and active custom roles",
  "inputSchema": {
    "type": "object",
    "properties": {
      "include_predefined": { 
        "type": "boolean", 
        "default": true,
        "description": "Include built-in roles"
      },
      "include_custom": { 
        "type": "boolean", 
        "default": true,
        "description": "Include custom roles" 
      },
      "session_id": { 
        "type": "string", 
        "description": "Optional: List only roles active in a specific session"
      }
    }
  }
}
```

**Response Example:**
```json
{
  "predefined_roles": [
    {
      "name": "architect",
      "description": "Designs system architecture and creates technical specifications",
      "capabilities": ["system-design", "technical-specifications", "architecture-diagrams"]
    },
    {
      "name": "engineer",
      "description": "Implements code based on specifications",
      "capabilities": ["code-implementation", "testing", "debugging"]
    }
  ],
  "custom_roles": [
    {
      "name": "performance_analyst",
      "description": "Analyzes API performance metrics",
      "session_id": "sess_abc123",
      "created_at": "2024-01-10T10:20:00Z"
    }
  ]
}
```

## Resources

### 1. Agent Output Streams

Real-time streaming of agent output via Server-Sent Events (SSE).

```json
{
  "uri": "maos://sessions/{session_id}/agents/{agent_id}/output",
  "name": "Agent Output Stream",
  "description": "Real-time output from a specific agent",
  "mimeType": "text/event-stream"
}
```

**SSE Event Format:**
```
event: agent-output
data: {
  "session_id": "sess_abc123",
  "agent_id": "agent_engineer_1_def456",
  "line": "Implementing user authentication endpoint...",
  "timestamp": "2024-01-10T10:30:00Z"
}

event: agent-status
data: {
  "session_id": "sess_abc123",
  "agent_id": "agent_engineer_1_def456",
  "status": "completed",
  "summary": "Successfully implemented 5 API endpoints"
}
```

### 2. Session Status Resource

Current status of all agents in a session.

```json
{
  "uri": "maos://sessions/{session_id}/status",
  "name": "Session Status",
  "description": "Current status of all agents in a session",
  "mimeType": "application/json"
}
```

### 3. Agent Roles

List of available agent roles that can be used in orchestration.

```json
{
  "uri": "maos://roles",
  "name": "Agent Roles",
  "description": "List of available agent roles and their capabilities",
  "mimeType": "application/json"
}
```

## Error Responses

All tools follow a consistent error response format:

```json
{
  "error": {
    "code": "RESOURCE_LIMIT_EXCEEDED",
    "message": "Maximum number of engineer agents (3) already reached",
    "details": {
      "role": "backend_engineer",
      "current_count": 3,
      "max_allowed": 3
    }
  }
}
```

**Common Error Codes:**
- `SESSION_NOT_FOUND` - Invalid session ID
- `AGENT_NOT_FOUND` - Invalid agent ID
- `RESOURCE_LIMIT_EXCEEDED` - Too many agents or resources
- `INVALID_ROLE` - Unknown or invalid role specification
- `DEPENDENCY_NOT_MET` - Required agent dependencies not satisfied
- `SPAWN_FAILED` - Failed to spawn agent process

## Usage Examples

### Starting an Orchestration Session

```javascript
// MCP client pseudo-code
const result = await client.callTool("maos/orchestrate", {
  objective: "Create a microservice for payment processing",
  agents: [
    { role: "architect", task: "Design the service architecture" },
    { role: "engineer", task: "Implement the payment service" },
    { role: "security", task: "Review security implications" },
    { role: "qa", task: "Create integration tests" }
  ],
  strategy: "adaptive"
});

// Subscribe to agent outputs
const stream = client.subscribeResource(
  `maos://sessions/${result.session_id}/agents/${result.agents[0].agent_id}/output`
);
```


## References

This document consolidates MCP tool definitions from:
- ADR-003: MCP Server Architecture (complete tool and resource definitions)