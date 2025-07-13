# Model Context Protocol (MCP) Research for MAOS

## Executive Summary

The Model Context Protocol (MCP) is an open protocol that enables large language models to dynamically access external tools and data sources through a standardized client-server architecture. For MAOS (Multi-Agent Orchestration System), MCP provides a robust foundation for exposing orchestration capabilities, managing agent interactions, and facilitating real-time communication between components.

Key findings:
- MCP uses JSON-RPC 2.0 over multiple transport mechanisms (stdio, HTTP with SSE)
- Supports three primary capabilities: Resources (data), Tools (functions), and Prompts (templates)
- Provides SDKs for multiple languages (TypeScript, Python, Java, Kotlin, C#)
- Enables secure, scalable integration patterns suitable for multi-agent systems
- Supports real-time bidirectional communication via Server-Sent Events

## MCP Architecture Overview

### Core Components

1. **Hosts**: LLM applications (like Claude Desktop) that initiate connections
2. **Clients**: Maintain 1:1 connections with servers within host applications
3. **Servers**: Provide context, tools, and prompts to clients

### Communication Architecture

```
┌─────────────────┐         ┌─────────────────┐
│   Host (LLM)    │         │   MCP Server    │
│                 │         │                 │
│  ┌───────────┐  │         │  ┌───────────┐  │
│  │  Client   │◄─┼─────────┼─►│ Resources │  │
│  └───────────┘  │         │  ├───────────┤  │
│                 │         │  │   Tools   │  │
│                 │         │  ├───────────┤  │
│                 │         │  │  Prompts  │  │
│                 │         │  └───────────┘  │
└─────────────────┘         └─────────────────┘
```

### Design Principles

- **Standardization**: "Think of MCP like a USB-C port for AI applications"
- **Flexibility**: Multiple transport options and implementation languages
- **Security**: Built-in authentication, validation, and access controls
- **Modularity**: Servers expose specific capabilities independently

## Protocol Specifications

### Message Format

MCP uses JSON-RPC 2.0 for all communication:

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "execute_task",
    "arguments": {
      "task_id": "12345"
    }
  },
  "id": "request-001"
}
```

### Communication Patterns

1. **Request-Response**: Traditional RPC pattern with response expectations
2. **Notifications**: One-way messages without response requirements
3. **Streaming**: Server-Sent Events for real-time updates

### Protocol Handshake

1. Client connects to server via chosen transport
2. Capabilities negotiation occurs
3. Server exposes available resources, tools, and prompts
4. Client can now interact with server capabilities

## Building Custom MCP Servers

### Development Workflow

1. **Setup Environment**
   ```bash
   # TypeScript
   npm init -y
   npm install @modelcontextprotocol/sdk
   
   # Python
   pip install mcp
   ```

2. **Create Server Instance**
   ```typescript
   // TypeScript Example
   import { Server } from '@modelcontextprotocol/sdk/server/index.js';
   import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
   
   const server = new Server({
     name: 'maos-orchestrator',
     version: '1.0.0',
   });
   ```

3. **Define Capabilities**
   ```python
   # Python Example
   from mcp.server import Server, NotificationOptions
   from mcp.server.models import InitializationOptions
   import mcp.server.stdio
   import mcp.types as types
   
   server = Server("maos-orchestrator")
   
   @server.tool()
   async def orchestrate_agents(
       task_description: str,
       agent_ids: list[str]
   ) -> str:
       """Orchestrate multiple agents to complete a task."""
       # Implementation logic
       return result
   ```

### Server Configuration

```json
{
  "mcpServers": {
    "maos-orchestrator": {
      "type": "sse",
      "url": "http://localhost:8080/mcp",
      "headers": {
        "Authorization": "Bearer ${MAOS_API_KEY}"
      }
    }
  }
}
```

## Resource and Tool Management

### Resources

Resources expose data that can be read by clients:

```typescript
// Define a resource
server.setRequestHandler(ListResourcesRequestSchema, async () => {
  return {
    resources: [
      {
        uri: "maos://agents/list",
        name: "Available Agents",
        description: "List of all registered agents in MAOS",
        mimeType: "application/json"
      }
    ]
  };
});

// Handle resource reads
server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
  const { uri } = request.params;
  
  if (uri === "maos://agents/list") {
    return {
      contents: [{
        uri: uri,
        mimeType: "application/json",
        text: JSON.stringify(await getAgentList())
      }]
    };
  }
});
```

### Tools

Tools expose executable functions:

```python
@server.tool()
async def create_agent(
    name: str,
    capabilities: list[str],
    model: str = "claude-3-opus-20240229"
) -> dict:
    """Create a new agent in the MAOS system."""
    
    agent = {
        "id": generate_id(),
        "name": name,
        "capabilities": capabilities,
        "model": model,
        "status": "initialized"
    }
    
    # Register agent in system
    await register_agent(agent)
    
    return {
        "success": True,
        "agent": agent
    }
```

### Tool Schema Definition

```typescript
const createAgentTool = {
  name: "create_agent",
  description: "Create a new agent in the MAOS system",
  inputSchema: {
    type: "object",
    properties: {
      name: {
        type: "string",
        description: "Name of the agent"
      },
      capabilities: {
        type: "array",
        items: { type: "string" },
        description: "List of agent capabilities"
      },
      model: {
        type: "string",
        default: "claude-3-opus-20240229",
        description: "LLM model to use"
      }
    },
    required: ["name", "capabilities"]
  }
};
```

## Real-time Communication via SSE

### SSE Transport Implementation

```typescript
// Server-side SSE setup
import express from 'express';
import { SSEServerTransport } from '@modelcontextprotocol/sdk/server/sse.js';

const app = express();

app.post('/mcp', async (req, res) => {
  const transport = new SSEServerTransport('/mcp', res);
  await server.connect(transport);
});

app.get('/mcp', (req, res) => {
  res.writeHead(200, {
    'Content-Type': 'text/event-stream',
    'Cache-Control': 'no-cache',
    'Connection': 'keep-alive'
  });
  
  // Send endpoint information
  res.write(`event: endpoint\ndata: ${JSON.stringify({ url: '/mcp' })}\n\n`);
});
```

### Streaming Updates

```python
async def stream_agent_status(agent_id: str):
    """Stream real-time agent status updates."""
    while True:
        status = await get_agent_status(agent_id)
        
        # Send SSE event
        yield {
            "event": "agent_status",
            "data": {
                "agent_id": agent_id,
                "status": status,
                "timestamp": datetime.now().isoformat()
            }
        }
        
        await asyncio.sleep(1)  # Update every second
```

## Integration with MAOS

### Orchestration Server Architecture

```typescript
class MAOSOrchestrationServer {
  private server: Server;
  private agentManager: AgentManager;
  private taskQueue: TaskQueue;
  
  constructor() {
    this.server = new Server({
      name: 'maos-orchestrator',
      version: '1.0.0',
      capabilities: {
        resources: {},
        tools: {},
        prompts: {}
      }
    });
    
    this.setupTools();
    this.setupResources();
    this.setupPrompts();
  }
  
  private setupTools() {
    // Orchestration tools
    this.server.addTool({
      name: "orchestrate_task",
      description: "Orchestrate a complex task across multiple agents",
      inputSchema: {
        type: "object",
        properties: {
          task: { type: "string" },
          constraints: { type: "object" },
          parallel: { type: "boolean", default: false }
        }
      },
      handler: async (args) => this.orchestrateTask(args)
    });
    
    // Agent management tools
    this.server.addTool({
      name: "spawn_agent",
      description: "Spawn a new agent with specific capabilities",
      handler: async (args) => this.agentManager.spawn(args)
    });
  }
  
  private async orchestrateTask(args: any) {
    const { task, constraints, parallel } = args;
    
    // Task decomposition
    const subtasks = await this.decomposeTask(task, constraints);
    
    // Agent selection
    const agents = await this.selectAgents(subtasks);
    
    // Execute orchestration
    if (parallel) {
      return await this.executeParallel(subtasks, agents);
    } else {
      return await this.executeSequential(subtasks, agents);
    }
  }
}
```

### MAOS-Specific Resources

```python
# Agent Registry Resource
@server.resource("maos://registry/agents")
async def get_agent_registry():
    """Expose the current agent registry."""
    agents = await db.get_all_agents()
    return {
        "mimeType": "application/json",
        "text": json.dumps({
            "agents": agents,
            "total": len(agents),
            "active": sum(1 for a in agents if a["status"] == "active")
        })
    }

# Task History Resource
@server.resource("maos://history/tasks/{task_id}")
async def get_task_history(task_id: str):
    """Get execution history for a specific task."""
    history = await db.get_task_history(task_id)
    return {
        "mimeType": "application/json",
        "text": json.dumps(history)
    }
```

### Event Streaming for Agent Communication

```typescript
// Real-time agent communication
class AgentCommunicationChannel {
  private eventEmitter: EventEmitter;
  
  async broadcastToAgents(event: AgentEvent) {
    // Send via SSE to all connected clients
    this.connections.forEach(conn => {
      conn.write(`event: agent_event\n`);
      conn.write(`data: ${JSON.stringify(event)}\n\n`);
    });
  }
  
  async handleAgentMessage(agentId: string, message: any) {
    // Process inter-agent communication
    const targetAgent = message.targetAgent;
    
    if (targetAgent) {
      // Direct message
      await this.sendDirectMessage(agentId, targetAgent, message);
    } else {
      // Broadcast
      await this.broadcastToAgents({
        type: 'broadcast',
        source: agentId,
        message: message,
        timestamp: Date.now()
      });
    }
  }
}
```

## Code Examples

### Complete Python MCP Server for MAOS

```python
import asyncio
import json
from typing import Any, Dict, List
from datetime import datetime

import mcp.server.stdio
import mcp.types as types
from mcp.server import NotificationOptions, Server
from mcp.server.models import InitializationOptions

# Initialize MAOS orchestration server
server = Server("maos-orchestrator")

# In-memory storage (replace with actual database)
agents: Dict[str, Dict[str, Any]] = {}
tasks: Dict[str, Dict[str, Any]] = {}

@server.list_resources()
async def handle_list_resources() -> List[types.Resource]:
    """List all available MAOS resources."""
    return [
        types.Resource(
            uri="maos://agents",
            name="Agent Registry",
            description="List of all registered agents",
            mimeType="application/json",
        ),
        types.Resource(
            uri="maos://tasks",
            name="Task Queue",
            description="Current task queue and history",
            mimeType="application/json",
        ),
    ]

@server.read_resource()
async def handle_read_resource(uri: str) -> str:
    """Read MAOS resources."""
    if uri == "maos://agents":
        return json.dumps(agents, indent=2)
    elif uri == "maos://tasks":
        return json.dumps(tasks, indent=2)
    else:
        raise ValueError(f"Unknown resource: {uri}")

@server.list_tools()
async def handle_list_tools() -> List[types.Tool]:
    """List all MAOS orchestration tools."""
    return [
        types.Tool(
            name="create_agent",
            description="Create a new agent in MAOS",
            inputSchema={
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "capabilities": {
                        "type": "array",
                        "items": {"type": "string"}
                    },
                    "model": {"type": "string", "default": "claude-3"}
                },
                "required": ["name", "capabilities"]
            },
        ),
        types.Tool(
            name="orchestrate_task",
            description="Orchestrate a task across multiple agents",
            inputSchema={
                "type": "object",
                "properties": {
                    "description": {"type": "string"},
                    "required_capabilities": {
                        "type": "array",
                        "items": {"type": "string"}
                    },
                    "parallel": {"type": "boolean", "default": False}
                },
                "required": ["description"]
            },
        ),
        types.Tool(
            name="get_agent_status",
            description="Get the current status of an agent",
            inputSchema={
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string"}
                },
                "required": ["agent_id"]
            },
        ),
    ]

@server.call_tool()
async def handle_call_tool(
    name: str, arguments: Dict[str, Any]
) -> List[types.TextContent]:
    """Handle MAOS tool calls."""
    
    if name == "create_agent":
        agent_id = f"agent_{len(agents) + 1}"
        agent = {
            "id": agent_id,
            "name": arguments["name"],
            "capabilities": arguments["capabilities"],
            "model": arguments.get("model", "claude-3"),
            "status": "initialized",
            "created_at": datetime.now().isoformat()
        }
        agents[agent_id] = agent
        
        return [
            types.TextContent(
                type="text",
                text=f"Created agent: {json.dumps(agent, indent=2)}"
            )
        ]
    
    elif name == "orchestrate_task":
        task_id = f"task_{len(tasks) + 1}"
        
        # Find suitable agents
        required_caps = arguments.get("required_capabilities", [])
        suitable_agents = []
        
        for agent_id, agent in agents.items():
            if any(cap in agent["capabilities"] for cap in required_caps):
                suitable_agents.append(agent_id)
        
        task = {
            "id": task_id,
            "description": arguments["description"],
            "required_capabilities": required_caps,
            "assigned_agents": suitable_agents,
            "parallel": arguments.get("parallel", False),
            "status": "queued",
            "created_at": datetime.now().isoformat()
        }
        tasks[task_id] = task
        
        # Simulate task execution
        task["status"] = "executing"
        
        result = {
            "task_id": task_id,
            "assigned_agents": suitable_agents,
            "execution_plan": f"Task will be executed by {len(suitable_agents)} agents"
        }
        
        return [
            types.TextContent(
                type="text",
                text=json.dumps(result, indent=2)
            )
        ]
    
    elif name == "get_agent_status":
        agent_id = arguments["agent_id"]
        agent = agents.get(agent_id)
        
        if not agent:
            return [
                types.TextContent(
                    type="text",
                    text=f"Agent {agent_id} not found"
                )
            ]
        
        return [
            types.TextContent(
                type="text",
                text=json.dumps(agent, indent=2)
            )
        ]
    
    else:
        raise ValueError(f"Unknown tool: {name}")

async def main():
    """Run the MAOS MCP server."""
    async with mcp.server.stdio.stdio_server() as (read_stream, write_stream):
        await server.run(
            read_stream,
            write_stream,
            InitializationOptions(
                server_name="maos-orchestrator",
                server_version="0.1.0",
                capabilities=server.get_capabilities(
                    notification_options=NotificationOptions(),
                    experimental_capabilities={},
                ),
            ),
        )

if __name__ == "__main__":
    asyncio.run(main())
```

### TypeScript SSE Server Example

```typescript
import express from 'express';
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { SSEServerTransport } from '@modelcontextprotocol/sdk/server/sse.js';
import {
  CallToolRequestSchema,
  ListResourcesRequestSchema,
  ListToolsRequestSchema,
  ReadResourceRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';

const app = express();
app.use(express.json());

// Create MAOS MCP server
const maosServer = new Server({
  name: 'maos-orchestrator',
  version: '1.0.0',
});

// Agent orchestration state
const agents = new Map();
const tasks = new Map();

// Define tools
maosServer.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: 'spawn_agent',
        description: 'Spawn a new agent with specific capabilities',
        inputSchema: {
          type: 'object',
          properties: {
            type: { type: 'string' },
            capabilities: { type: 'array', items: { type: 'string' } },
            config: { type: 'object' }
          },
          required: ['type', 'capabilities']
        }
      },
      {
        name: 'coordinate_agents',
        description: 'Coordinate multiple agents on a task',
        inputSchema: {
          type: 'object',
          properties: {
            task: { type: 'string' },
            agents: { type: 'array', items: { type: 'string' } },
            strategy: { type: 'string', enum: ['parallel', 'sequential', 'hierarchical'] }
          },
          required: ['task', 'agents']
        }
      }
    ]
  };
});

// Handle tool calls
maosServer.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;
  
  switch (name) {
    case 'spawn_agent': {
      const agentId = `agent_${Date.now()}`;
      const agent = {
        id: agentId,
        type: args.type,
        capabilities: args.capabilities,
        config: args.config || {},
        status: 'active',
        createdAt: new Date().toISOString()
      };
      
      agents.set(agentId, agent);
      
      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify({ success: true, agentId, agent }, null, 2)
          }
        ]
      };
    }
    
    case 'coordinate_agents': {
      const taskId = `task_${Date.now()}`;
      const task = {
        id: taskId,
        description: args.task,
        assignedAgents: args.agents,
        strategy: args.strategy || 'parallel',
        status: 'running',
        createdAt: new Date().toISOString()
      };
      
      tasks.set(taskId, task);
      
      // Simulate coordination
      const results = await coordinateAgents(task);
      
      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify({ taskId, results }, null, 2)
          }
        ]
      };
    }
    
    default:
      throw new Error(`Unknown tool: ${name}`);
  }
});

// SSE endpoint
app.post('/mcp', async (req, res) => {
  const transport = new SSEServerTransport('/mcp', res);
  await maosServer.connect(transport);
});

app.get('/mcp', (req, res) => {
  res.writeHead(200, {
    'Content-Type': 'text/event-stream',
    'Cache-Control': 'no-cache',
    'Connection': 'keep-alive',
    'Access-Control-Allow-Origin': '*'
  });
  
  // Send endpoint event
  res.write(`event: endpoint\ndata: {"url": "/mcp"}\n\n`);
  
  // Keep connection alive
  const keepAlive = setInterval(() => {
    res.write(':keepalive\n\n');
  }, 30000);
  
  req.on('close', () => {
    clearInterval(keepAlive);
  });
});

async function coordinateAgents(task: any) {
  // Implement actual coordination logic
  return {
    status: 'completed',
    results: task.assignedAgents.map((agentId: string) => ({
      agentId,
      result: 'Task completed successfully'
    }))
  };
}

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`MAOS MCP Server running on port ${PORT}`);
});
```

## Best Practices

### Security

1. **Authentication**
   - Implement OAuth 2.0 for remote servers
   - Use API keys with proper rotation
   - Validate all client connections

2. **Input Validation**
   ```python
   @server.call_tool()
   async def handle_tool_call(name: str, arguments: dict):
       # Validate tool name
       if name not in ALLOWED_TOOLS:
           raise ValueError(f"Unknown tool: {name}")
       
       # Validate arguments
       schema = TOOL_SCHEMAS[name]
       validate_against_schema(arguments, schema)
       
       # Sanitize inputs
       sanitized_args = sanitize_inputs(arguments)
       
       # Execute with proper error handling
       try:
           return await execute_tool(name, sanitized_args)
       except Exception as e:
           logger.error(f"Tool execution failed: {e}")
           raise
   ```

3. **Resource Access Control**
   ```typescript
   server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
     const { uri } = request.params;
     
     // Check permissions
     if (!hasPermission(request.context.user, uri)) {
       throw new Error('Access denied');
     }
     
     // Validate URI format
     if (!isValidResourceURI(uri)) {
       throw new Error('Invalid resource URI');
     }
     
     // Return resource with proper sanitization
     return sanitizeResourceResponse(await readResource(uri));
   });
   ```

### Performance

1. **Connection Management**
   - Implement connection pooling for database access
   - Use efficient serialization (consider MessagePack for binary data)
   - Implement caching for frequently accessed resources

2. **Async Operations**
   ```python
   # Use async/await throughout
   async def process_large_dataset(dataset_id: str):
       # Stream data instead of loading all at once
       async for chunk in stream_dataset(dataset_id):
           processed = await process_chunk(chunk)
           yield processed
   ```

3. **Resource Optimization**
   - Implement pagination for large resource lists
   - Use streaming for real-time data
   - Compress large responses

### Error Handling

```typescript
// Comprehensive error handling
class MCPError extends Error {
  constructor(
    public code: number,
    message: string,
    public data?: any
  ) {
    super(message);
  }
}

server.setErrorHandler(async (error) => {
  if (error instanceof MCPError) {
    return {
      code: error.code,
      message: error.message,
      data: error.data
    };
  }
  
  // Log unexpected errors
  logger.error('Unexpected error:', error);
  
  return {
    code: -32603,
    message: 'Internal error',
    data: { 
      timestamp: new Date().toISOString(),
      requestId: generateRequestId()
    }
  };
});
```

## Related Resources

### Official Documentation
- [MCP Specification](https://modelcontextprotocol.io/specification)
- [TypeScript SDK](https://github.com/modelcontextprotocol/typescript-sdk)
- [Python SDK](https://github.com/modelcontextprotocol/python-sdk)

### Example Implementations
- [Filesystem Server](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem)
- [Memory Server](https://github.com/modelcontextprotocol/servers/tree/main/src/memory)
- [Sequential Thinking Server](https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking)

### Community Resources
- [MCP Discord Community](https://discord.gg/anthropic)
- [GitHub Discussions](https://github.com/modelcontextprotocol/specification/discussions)

### Integration Examples
- [Claude Desktop Configuration](https://docs.anthropic.com/en/docs/claude-code/mcp)
- [Spring Boot MCP Server](https://github.com/modelcontextprotocol/java-sdk/tree/main/spring-boot-starter)

## Conclusion

MCP provides a robust foundation for building MAOS with:
- Standardized protocol for agent communication
- Multiple transport options including real-time SSE
- Strong typing and schema validation
- Built-in security and authentication mechanisms
- Flexible resource and tool management
- Multi-language SDK support

The protocol's design aligns well with MAOS requirements for orchestrating multiple agents, managing resources, and providing real-time updates on task execution. By implementing MCP servers for MAOS, we can leverage existing Claude Code infrastructure while building powerful multi-agent orchestration capabilities.