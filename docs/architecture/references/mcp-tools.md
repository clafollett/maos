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
    "maos/spawn-agent",
    "maos/agent-message",
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
      "role": "architect",
      "task": "Design the API architecture and database schema"
    },
    {
      "role": "engineer",
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
      "role": "architect",
      "status": "pending"
    },
    {
      "agent_id": "agent_engineer_backend_1_ghi789",
      "role": "engineer",
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

### 2. maos/spawn-agent

Spawn a single specialized agent for a specific task, optionally within an existing session.

```json
{
  "name": "maos/spawn-agent",
  "description": "Spawn a specialized agent for a specific task",
  "inputSchema": {
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
        "description": "Task description for the agent"
      },
      "session_id": {
        "type": "string",
        "description": "Session to add agent to"
      },
      "instance_suffix": {
        "type": "string",
        "description": "Optional suffix for agent identification"
      },
      "dependencies": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Agent IDs this agent depends on"
      },
      "context": {
        "type": "object",
        "description": "Additional context for the agent"
      },
      "template_override": {
        "type": "object",
        "properties": {
          "prompt_template": {
            "type": "string",
            "description": "Custom prompt template for this agent"
          },
          "timeout_seconds": {
            "type": "integer",
            "description": "Override default timeout"
          },
          "max_memory_mb": {
            "type": "integer",
            "description": "Override memory limit"
          }
        },
        "description": "Override default template settings"
      }
    },
    "required": ["role", "task", "session_id"]
  }
}
```

**Example Usage:**
```json
{
  "role": {
    "name": "performance_analyst",
    "description": "Analyzes API performance metrics",
    "responsibilities": "Profile endpoints, identify bottlenecks, suggest optimizations"
  },
  "task": "Analyze the performance of the user management API",
  "session_id": "sess_abc123",
  "dependencies": ["agent_engineer_backend_1_ghi789"],
  "template_override": {
    "timeout_seconds": 3600
  }
}
```

### 3. maos/agent-message

Send messages between agents for coordination and information sharing.

```json
{
  "name": "maos/agent-message", 
  "description": "Send a message between agents",
  "inputSchema": {
    "type": "object",
    "properties": {
      "from_agent": { 
        "type": "string",
        "description": "Sender agent ID"
      },
      "to_agent": { 
        "type": "string",
        "description": "Target agent ID or role-based selector (e.g., 'engineer_*', 'all_engineers')"
      },
      "message": { 
        "type": "string",
        "description": "Message content"
      },
      "type": {
        "type": "string",
        "enum": ["request", "response", "notification", "broadcast"],
        "default": "notification"
      }
    },
    "required": ["from_agent", "to_agent", "message"]
  }
}
```

**Role-Based Selectors:**
- `engineer_*` - All engineers in the session
- `all_engineers` - Same as above
- `*` - All agents (broadcast)
- `architect_1` - Specific architect instance

### 4. maos/session-status

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
      "role": "architect",
      "state": "completed",
      "started_at": "2024-01-10T10:00:00Z",
      "completed_at": "2024-01-10T10:15:00Z"
    },
    {
      "agent_id": "agent_engineer_backend_1_ghi789",
      "role": "engineer",
      "state": "running",
      "started_at": "2024-01-10T10:15:00Z"
    }
  ]
}
```

### 5. maos/list-roles

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

### 3. Agent Registry

List of available agent roles and their capabilities.

```json
{
  "uri": "maos://agents/available",
  "name": "Available Agents",
  "description": "List of registered CLI tools and their capabilities",
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
      "role": "engineer",
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

### Adding an Agent to Existing Session

```javascript
await client.callTool("maos/spawn-agent", {
  role: "devops",
  task: "Set up deployment pipeline for the payment service",
  session_id: result.session_id,
  dependencies: [result.agents[1].agent_id] // Depends on engineer
});
```

## References

This document consolidates MCP tool definitions from:
- ADR-003: MCP Server Architecture (complete tool and resource definitions)