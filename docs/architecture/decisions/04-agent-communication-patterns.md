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
We will implement a **Multi-Agent Single Server** architecture using ACP, where a single Claude Code Agent manages multiple CLI processes, and the Orchestrator acts as a Router Agent to coordinate work.

### Streamlined Architecture: Multi-Agent Single Server

```
┌─────────────────────────────────────────────────────────────────┐
│                    Claude Code (MCP Client)                     │
│                                                                 │
│  maos/orchestrate ──► Start orchestration session              │
│  maos/session-status ──► Monitor progress                      │
│  maos/list-roles ──► List available agent roles               │
└─────────────────────┬───────────────────────────────────────────┘
                      │ MCP Protocol
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                     MAOS MCP Server                            │
│                                                                 │
│  • Provides MCP tools for orchestration                        │
│  • Tracks session state                                        │
│  • Streams orchestrator output to Claude Code                  │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Spawns Orchestrator
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│            Orchestrator (Router Agent) - ACP Server            │
│                                                                 │
│  • Analyzes tasks and plans phases                             │
│  • Routes work to appropriate agents                           │
│  • Tracks agent sessions                                       │
└─────────────────────┬───────────────────────────────────────────┘
                      │ ACP Requests
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│             Claude Code Agent - ACP Server                     │
│                                                                 │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     │
│  │ Claude CLI  │     │ Claude CLI  │     │ Claude CLI  │     │
│  │ -p architect│     │ -p backend  │     │ -p frontend │     │
│  │ Process     │     │ Process     │     │ Process     │     │
│  └─────────────┘     └─────────────┘     └─────────────┘     │
│                                                               │
│  • Single ACP server manages multiple CLI processes           │
│  • Each process has different role via -p flag               │
│  • Session continuity via --session-id                       │
│  • Efficient resource utilization                            │
└─────────────────────────────────────────────────────────────────┘
```

### Core ACP Integration Principles

1. **Multi-Agent Single Server**: One ACP server manages multiple CLI agent processes
2. **Router Agent Pattern**: Orchestrator analyzes tasks and routes to appropriate agents
3. **Session Continuity**: Claude CLI's --session-id preserves context across phases
4. **Efficient Resource Use**: Single server process instead of many
5. **Standardized Protocol**: All communication follows ACP REST API specification
6. **Phase-by-Phase Planning**: Orchestrator plans based on actual outputs, not assumptions
7. **Extensible Design**: Easy to add Gemini Agent, Codex Agent, etc.
8. **Clean Separation**: MCP for external interface, ACP for internal coordination

## ACP Communication Architecture

### 1. Claude Code Agent Integration

The Claude Code Agent runs a single ACP server that:
- **Manages multiple CLI processes** with different roles via -p flag
- **Provides unified endpoint** for all agent communication
- **Routes messages** to appropriate CLI processes based on session
- **Monitors health** of all managed CLI processes
- **Maintains session continuity** via --session-id flag

### 2. Message Types and Patterns

Communication follows **phase-based coordination** patterns:

**Communication Flow:**
- **Phase Initiation**: Orchestrator determines which agents needed for phase
- **Per-Agent Requests**: Orchestrator sends individual request to Claude Code Agent for each agent
- **Process Spawning**: Claude Code Agent spawns one CLI process per request
- **Parallel/Sequential Execution**: Orchestrator manages execution strategy
- **Individual Results**: Each agent's results returned separately to Orchestrator
- **Phase Completion**: Orchestrator aggregates all results before planning next phase

**Message Categories:**
- **Work Requests**: Orchestrator sends phase-specific tasks to Claude Code Agent
- **Execution Status**: Claude Code Agent reports CLI process status
- **Phase Results**: Completed work artifacts and summaries
- **Error Handling**: Process failures or blocking issues

**Phase-by-Phase Principle**: Each phase planned based on actual outputs from previous phases, not assumptions.

**Message Structure**: All messages follow ACP standard format with phase context, session tracking, and result artifacts.

### 3. Session and Agent Management

The Claude Code Agent manages sessions and CLI processes:

**Session Management:**
- **Session Tracking**: Maintains mapping of sessions to CLI processes
- **Process Pool**: Manages available Claude CLI processes
- **Role Assignment**: Assigns roles via -p flag for each phase
- **Context Preservation**: Uses --session-id for continuity across phases

**Process Lifecycle:**
- **On-Demand Spawning**: CLI processes created when needed
- **Session Binding**: Each process bound to specific session via --session-id
- **Resource Management**: Monitors and limits concurrent processes
- **Clean Termination**: Graceful shutdown when phases complete

### 4. Artifact Sharing

Agents share work through file system artifacts:
- **Shared Context**: Common directories for specifications and code
- **Phase Outputs**: Each phase produces artifacts for next phase
- **Result Aggregation**: Claude Code Agent collects outputs for Orchestrator
- **Version Control**: Git integration for tracking changes

### 5. Orchestrator as Router Agent

The Orchestrator acts as a Router Agent, coordinating work through the Claude Code Agent:

**Router Responsibilities:**
- **Task Analysis**: Breaks down high-level objectives into phases
- **Work Distribution**: Routes phase work to Claude Code Agent
- **Progress Tracking**: Monitors phase completion and results
- **Adaptive Planning**: Plans next phase based on actual outputs

### 6. Communication Patterns

**Phase-Based Communication:**
- Orchestrator makes multiple requests to Claude Code Agent per phase (one per agent)
- Each request spawns one Claude CLI process for a specific role/task
- Orchestrator coordinates parallel/sequential execution of agents within phase
- No direct agent-to-agent communication needed
- Each agent's results flow back individually to Orchestrator

**Efficient Resource Use:**
- Single ACP server instead of many
- Reduced network overhead and port management
- Simplified message routing through one endpoint
- Clean phase boundaries for work organization

### 7. Integration with MCP Server

**Streamlined MCP Tools:**
- **`maos/orchestrate`**: Start orchestration with Claude Code Agent
- **`maos/session-status`**: Monitor orchestration progress
- **`maos/list-roles`**: List available agent roles
- **REMOVED**: `maos/agent-message` and `maos/spawn-agent` (no longer needed)

**Clean Architecture Benefits:**
- MCP handles external interface to Claude Code
- ACP handles internal coordination with Claude Code Agent
- Clear separation between external and internal protocols
- Simplified tool surface area

## Consequences

### Positive
- **Simplified Architecture**: Single ACP server manages all agents
- **Efficient Resource Use**: One server process instead of many
- **Session Continuity**: Claude's --session-id preserves context perfectly
- **Easy Extension**: Simple to add Gemini Agent, Codex Agent, etc.
- **Phase-Based Clarity**: Clean boundaries between work phases
- **Reduced Complexity**: No peer-to-peer discovery or routing needed
- **Better Debugging**: All communication through one central point
- **Cost Optimization**: Reuse CLI processes across phases
- **Clean Tool Interface**: Only 3 MCP tools needed

### Negative
- **Single Point of Failure**: Claude Code Agent becomes critical component
- **Process Management**: Must handle multiple CLI processes efficiently
- **ACP Dependency**: Still requires ACP protocol implementation
- **Sequential Phases**: Less parallelism than peer-to-peer approach

### Mitigation
- **Robust Error Handling**: Comprehensive failure recovery in Claude Code Agent
- **Process Pool Management**: Efficient CLI process lifecycle management
- **Health Monitoring**: Active monitoring of all CLI processes
- **Graceful Degradation**: Continue with reduced capacity if processes fail
- **Session Recovery**: Leverage --session-id for resumption after failures

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