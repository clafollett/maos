# ADR-003: MCP Server Architecture

## Status
Accepted

## Context
MAOS needs to provide multi-agent orchestration capabilities to AI tools like Claude Code. After analyzing various integration approaches, the Model Context Protocol (MCP) emerged as the ideal solution because:

- **Standardized Integration**: MCP is becoming the standard for AI tool extensions
- **Tool Discovery**: Clients automatically discover MAOS capabilities
- **Streaming Support**: Real-time updates via Server-Sent Events (SSE)
- **Language Agnostic**: Clients don't need to know MAOS is written in Rust

Key architectural insight: **MAOS becomes an MCP server that Claude Code (or any MCP client) connects to**, reversing the traditional flow where the orchestrator calls the AI.

## Decision
MAOS will be implemented as an MCP server that exposes orchestration capabilities through tools and resources, while spawning actual agent work via CLI processes.

### Architecture Overview
```
┌──────────────────────────────────┐
│   Claude Code (MCP Client)       │
│   - User types natural language  │
│   - LLM interprets into tools    │
│   - Displays agent outputs       │
└────────────┬─────────────────────┘
             │ MCP Protocol (HTTP/SSE)
             ▼
┌──────────────────────────────────┐
│   MAOS MCP Server                │
├──────────────────────────────────┤
│ Tools:                           │
│ • maos/orchestrate               │
│ • maos/spawn-agent               │
│ • maos/agent-message             │
│ • maos/session-status            │
├──────────────────────────────────┤
│ Resources:                       │
│ • Agent outputs (streaming)      │
│ • Session status                 │
│ • Agent templates                │
└────────────┬─────────────────────┘
             │ Process Spawning
             ▼
┌──────────────────────────────────┐
│   Agent Processes                │
│   ┌─────────┐ ┌─────────┐       │
│   │Claude -p│ │Claude -p│ ...   │
│   └─────────┘ └─────────┘       │
└──────────────────────────────────┘
```

### MCP Tools

#### 1. Orchestrate Tool
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
                  "enum": ["architect", "engineer", "researcher", "qa", "pm", "devops", "security", "data_scientist", "designer", "documenter", "reviewer", "analyst", "tester"],
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

#### 2. Spawn Agent Tool
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
            "enum": ["architect", "engineer", "researcher", "qa", "pm", "devops", "security", "data_scientist", "designer", "documenter", "reviewer", "analyst", "tester"],
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

#### 3. Inter-Agent Messaging Tool
```json
{
  "name": "maos/agent-message", 
  "description": "Send a message between agents",
  "inputSchema": {
    "type": "object",
    "properties": {
      "from_agent": { "type": "string" },
      "to_agent": { 
        "type": "string",
        "description": "Target agent ID or role-based selector (e.g., 'engineer_*', 'all_engineers')"
      },
      "message": { "type": "string" },
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

#### 4. Session Status Tool
```json
{
  "name": "maos/session-status",
  "description": "Get the current status of an orchestration session",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_id": { "type": "string" },
      "include_agents": { "type": "boolean", "default": true },
      "include_messages": { "type": "boolean", "default": false }
    },
    "required": ["session_id"]
  }
}
```

#### 5. List Roles Tool
```json
{
  "name": "maos/list-roles",
  "description": "List available predefined roles and active custom roles",
  "inputSchema": {
    "type": "object",
    "properties": {
      "include_predefined": { "type": "boolean", "default": true },
      "include_custom": { "type": "boolean", "default": true },
      "session_id": { 
        "type": "string", 
        "description": "Optional: List only roles active in a specific session"
      }
    }
  }
}
```

### MCP Resources

#### 1. Agent Output Streams
```json
{
  "uri": "maos://sessions/{session_id}/agents/{agent_id}/output",
  "name": "Agent Output Stream",
  "description": "Real-time output from a specific agent",
  "mimeType": "text/event-stream"
}
```

#### 2. Session Status
```json
{
  "uri": "maos://sessions/{session_id}/status",
  "name": "Session Status",
  "description": "Current status of all agents in a session",
  "mimeType": "application/json"
}
```

#### 3. Agent Registry
```json
{
  "uri": "maos://agents/available",
  "name": "Available Agents",
  "description": "List of registered CLI tools and their capabilities",
  "mimeType": "application/json"
}
```

### SSE Streaming Implementation

```rust
// Stream agent output back to MCP client
impl McpServer {
    async fn stream_agent_output(&self, session_id: &str, agent_id: &str) {
        let output_path = format!("~/.maos/projects/{}/sessions/{}/agents/{}/stdout.log",
            workspace_hash, session_id, agent_id);
        
        let mut file = File::open(&output_path).await?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        
        while reader.read_line(&mut line).await? > 0 {
            // Send SSE event
            self.send_event(Event {
                id: None,
                event: Some("agent-output".to_string()),
                data: json!({
                    "session_id": session_id,
                    "agent_id": agent_id,
                    "line": line.trim(),
                    "timestamp": Utc::now()
                }).to_string(),
            }).await?;
            
            line.clear();
        }
    }
}
```

### MCP Server Configuration

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
  "tools": ["maos/orchestrate", "maos/spawn-agent", "maos/agent-message", "maos/session-status", "maos/list-roles"],
  "resources": ["sessions", "agents", "templates"]
}
```

## Implementation Strategy

### Phase 1: Basic MCP Server
1. Implement MCP protocol handler
2. Create tool definitions
3. Basic process spawning
4. Simple stdout streaming

### Phase 2: Advanced Features
1. Inter-agent messaging
2. Dependency management
3. Session persistence
4. Error recovery

### Phase 3: Production Features
1. Multi-instance support
2. Resource limits
3. Security sandboxing
4. Performance monitoring

## Consequences

### Positive
- **Natural Integration**: Users interact with MAOS through their preferred AI tool
- **Language Processing**: The LLM handles natural language parsing
- **Streaming Updates**: Real-time visibility into agent activities
- **Tool Agnostic**: Works with any MCP-compatible client
- **Clean Separation**: MAOS focuses on orchestration, not NLP

### Negative
- **MCP Dependency**: Requires clients to support MCP
- **Network Overhead**: HTTP/SSE adds latency vs direct calls
- **Limited by MCP**: Must work within protocol constraints

### Mitigation
- Provide standalone CLI for non-MCP users
- Optimize SSE streaming for performance
- Contribute to MCP standard for missing features

## References
- [Model Context Protocol Specification](https://modelcontextprotocol.io)
- [Claude Code MCP Documentation](https://docs.anthropic.com/en/docs/claude-code/mcp)
- SSE (Server-Sent Events) for real-time streaming

---
*Date: 2025-07-09*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollettLaFollett Labs LLC)*