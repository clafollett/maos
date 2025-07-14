# ADR-04: ACP-Based Agent Communication

## Status
Accepted

## Context
Agents in MAOS need to communicate and coordinate their work effectively. Since each agent runs as a separate CLI process, we need well-defined patterns for:

- Sharing work artifacts (specifications, code, test results)
- Sending messages between agents (including the Orchestrator)
- Coordinating dependencies and handoffs
- Broadcasting status updates
- Requesting help or clarification
- Dynamic agent discovery and coordination

### Previous Approach (File-Based Messaging)
Our initial approach used file-based messaging with shared directories, but this had critical limitations:
- **Latency**: File system polling delays
- **No real-time communication**: Agents had to poll for messages
- **Scalability issues**: File system as message queue doesn't scale
- **No standardized format**: Ad-hoc message structures
- **Discovery problems**: Complex agent discovery mechanisms
- **MCP integration gap**: The `maos/agent-message` tool couldn't reach running CLI processes

### ACP Solution
The [Agent Communication Protocol (ACP)](https://agentcommunicationprotocol.dev/) provides a standardized, REST-based communication framework specifically designed for agent interoperability. ACP enables direct agent-to-agent communication through embedded servers, eliminating the need for message routing through the MCP server.

## Decision
We will implement a **peer-to-peer ACP network** where every agent (including the Orchestrator) runs its own ACP server, enabling direct, real-time, standardized communication between all agents.

### Revolutionary Architecture: Every Agent = ACP Server

```
┌─────────────────────────────────────────────────────────────────┐
│                    Claude Code (MCP Client)                     │
│                                                                 │
│  maos/orchestrate ──► Start orchestration session              │
│  maos/session-status ──► Monitor progress                      │
│  [NO MORE maos/agent-message - eliminated!]                    │
└─────────────────────┬───────────────────────────────────────────┘
                      │ MCP Protocol
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                     MAOS MCP Server                            │
│                                                                 │
│  • Spawns agents into ACP network                              │
│  • Tracks session state                                        │
│  • Streams ACP network activity to Claude Code                 │
│  • Participates in ACP network for coordination                │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Spawns agents with ACP servers
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    ACP Agent Network                           │
│                                                                 │
│  ┌─────────────────┐                                          │
│  │  ORCHESTRATOR   │ ◄─── Meta-agent with ACP server         │
│  │  (ACP Server)   │                                          │
│  └─────────┬───────┘                                          │
│            │ ACP REST API                                     │
│            ▼                                                  │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     │
│  │Solution     │◄────┤Backend      │────►│Frontend     │     │
│  │Architect    │     │Engineer     │     │Engineer     │     │
│  │(ACP Server) │     │(ACP Server) │     │(ACP Server) │     │
│  └─────────────┘     └─────────────┘     └─────────────┘     │
│                                                               │
│  • Every agent runs embedded ACP server                      │
│  • Direct peer-to-peer communication via REST API            │
│  • Standardized message formats across all agents            │
│  • Dynamic agent discovery through ACP mechanisms            │
│  • Orchestrator coordinates via ACP messages                 │
└─────────────────────────────────────────────────────────────────┘
```

### Core ACP Integration Principles

1. **Isolation by Default**: Agents work in focused isolation without interruptions
2. **Communication When Needed**: ACP used only for essential coordination, not chatter
3. **Every Agent is an ACP Server**: Each agent process embeds an ACP server
4. **Peer-to-Peer Network**: Direct agent-to-agent communication, no central routing
5. **Orchestrator as Participant**: Orchestrator is just another agent in the ACP network
6. **Standardized Protocol**: All communication follows ACP REST API specification
7. **Dynamic Discovery**: Agents discover each other through ACP mechanisms
8. **Immediate Delivery, Async Processing**: Real-time message delivery, agents process when convenient

## ACP Communication Architecture

### 1. Agent Process Integration

Each agent spawns with an embedded ACP server that provides:
- **Unique HTTP endpoint** for receiving messages
- **Agent registration** with discoverable metadata
- **Message routing** to/from the agent process
- **Health monitoring** and status reporting

### 2. Message Types and Patterns

Communication follows **minimal, essential-only** ACP message patterns:

**Essential Communication Triggers:**
- **Task Assignment**: When Orchestrator assigns new work
- **Completion Status**: When agents finish assigned work
- **Critical Handoffs**: When work must transfer between agents
- **Direction Changes**: When Orchestrator needs to redirect effort
- **Error/Blocking**: When agents need assistance to proceed
- **Phase Transitions**: When moving between orchestration phases

**Message Categories:**
- **Task Assignment**: Orchestrator assigns work to agents
- **Status Updates**: Agents report progress to Orchestrator (completion, blocking, errors)
- **Essential Handoffs**: Direct agent-to-agent work transfer
- **Critical Announcements**: System-wide notifications (phase changes, direction shifts)
- **Artifact Notifications**: Announce created/updated shared artifacts

**No Chatter Principle**: Messages sent only when **essential for coordination**, not for status broadcasts or social communication.

**Message Structure**: All messages follow ACP standard format with sender/receiver identification, message type, content payload, and timestamps.

### 3. ACP Agent Discovery

Agents discover each other through ACP discovery mechanisms:

**Discovery Methods:**
- **By Role**: Find all agents with specific roles (e.g., "backend_engineer", "architect")
- **By Capability**: Locate agents with specific capabilities (e.g., "database_design", "api_testing")
- **By Status**: Find available/active agents for coordination
- **Session Overview**: Get complete picture of all agents in the orchestration

**Agent Registry Information:**
- Agent ID and role
- Capabilities and status
- ACP endpoint for communication
- Last seen timestamp for health monitoring

### 5. Agent Communication Capabilities

Agents access communication through environment variables and can:
- **Send Messages**: Direct messages to specific agents or broadcast to role groups
- **Receive Messages**: Read from their inbox with automatic message parsing
- **Share Artifacts**: Place specifications, code, and results in shared context directories
- **Broadcast Announcements**: Send system-wide notifications about milestones or status
- **Request Assistance**: Ask for help from agents with specific roles or capabilities

### 6. Status Updates via stdout

Agents report status through structured JSON output:

```json
{"type": "status", "message": "Starting API design", "progress": 0.1}
{"type": "artifact", "path": "shared/context/architecture/api-spec.yaml", "description": "REST API specification"}
{"type": "dependency", "waiting_for": "agent_researcher_001", "reason": "Need database recommendations"}
{"type": "complete", "result": "success", "outputs": ["api-spec.yaml", "system-design.md"]}
```

### 4. Orchestrator as ACP Participant

**Revolutionary Insight**: The Orchestrator operates as a first-class ACP agent, not a central coordinator. This eliminates communication bottlenecks and creates true peer-to-peer coordination.

**Orchestrator Responsibilities:**
- **Task Assignment**: Send work assignments via ACP messages
- **Status Monitoring**: Receive progress updates from agents
- **Adaptive Planning**: Adjust orchestration based on real-time agent feedback
- **Phase Coordination**: Manage transitions between orchestration phases

### 5. Communication Patterns

**Direct Agent-to-Agent Communication:**
- Agents communicate directly for collaboration
- No routing through MCP server or central coordinator
- Real-time coordination for dependencies and handoffs

**Broadcast Communication:**
- System-wide announcements (phase completions, alerts)
- Multi-agent notifications for coordination
- Excludes sender from broadcast recipients automatically

**Shared Context Integration:**
- Agents still share artifacts through file system
- ACP messages announce artifact creation/updates
- Combination of real-time messaging + persistent artifacts

### 6. Integration with MCP Server

**Simplified MCP Server Role:**
- **Remove `maos/agent-message` tool** - No longer needed
- **Spawn agents into ACP network** - Focus on lifecycle management
- **Monitor ACP network activity** - Stream events to Claude Code
- **Session state tracking** - Maintain orchestration overview

**MCP Server Benefits:**
- Simplified tool set focused on orchestration
- No message routing complexity
- Real-time ACP event streaming to Claude Code
- Clean separation of concerns

## Consequences

### Positive
- **Eliminated Communication Disconnect**: Agents communicate directly, no MCP routing needed
- **Real-Time Messaging**: No file system polling delays
- **Standardized Protocol**: Industry-standard ACP communication format
- **Better Scalability**: REST-based communication scales better than file systems
- **Dynamic Discovery**: Built-in mechanisms for agents to find each other
- **Framework Agnostic**: Future-proof for different AI agent types
- **Orchestrator as Equal**: No special communication channels needed
- **Simplified MCP Server**: Remove `maos/agent-message` tool entirely
- **True Peer-to-Peer**: Direct agent-to-agent communication when needed
- **Adaptive Coordination**: Orchestrator adapts based on real-time agent status

### Negative
- **Network Overhead**: HTTP requests have more overhead than file operations
- **Port Management**: Each agent needs unique port allocation
- **ACP Dependency**: Requires ACP protocol implementation
- **Complexity**: More complex than simple file-based messaging
- **Discovery Latency**: Initial agent discovery may have delays

### Mitigation
- **Efficient ACP Implementation**: Use lightweight HTTP servers for ACP
- **Port Pool Management**: Pre-allocate port pools for agent spawning
- **Discovery Caching**: Cache agent information to reduce lookup latency
- **Fallback Mechanisms**: File-based backup for critical communications
- **Monitoring**: Comprehensive ACP network monitoring and debugging tools

## References
- [Agent Communication Protocol (ACP)](https://agentcommunicationprotocol.dev/) - Core communication protocol
- [ACP Specification](https://agentcommunicationprotocol.dev/specification/protocol) - Technical protocol details
- [ACP Examples](https://agentcommunicationprotocol.dev/examples/content-creation) - Multi-agent coordination examples
- **ADR-07: Orchestration Guardrails and Coordination Protocols** - Uses ACP infrastructure for coordination
- **ADR-08: Agent Lifecycle and Management** - ACP server initialization and management
- **ADR-10: MCP Server Architecture** - Simplified MCP server without agent-message tool
- **ADR-11: Adaptive Phase-Based Orchestration** - ACP-based orchestration coordination
- REST API design principles
- Microservices communication patterns
- Actor model for distributed systems

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*