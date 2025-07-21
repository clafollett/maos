# ADR-10: MCP Server Architecture

## Status
Accepted

## Context
MAOS needs to provide multi-agent orchestration capabilities to AI tools like Claude Code. After analyzing various integration approaches, the Model Context Protocol (MCP) emerged as the ideal solution because:

- **Standardized Integration**: MCP is becoming the standard for AI tool extensions
- **Tool Discovery**: Clients automatically discover MAOS capabilities
- **Streaming Support**: Real-time updates via Server-Sent Events (SSE)
- **Language Agnostic**: Clients don't need to know MAOS is written in Rust

### Revolutionary ACP Integration Insight
With our **Agent Communication Protocol (ACP) integration**, the MCP server role is dramatically simplified:
- **No message routing needed**: Agents communicate directly via ACP
- **Remove `maos/agent-message` tool**: This tool was fundamentally broken and is no longer needed
- **Focus on orchestration lifecycle**: MCP server manages sessions and streams ACP network activity
- **Simplified architecture**: Clean separation between MCP (external interface) and ACP (internal agent network)

Key architectural insights: 
- **MAOS MCP server spawns agents into an ACP network**
- **Only the Orchestrator communicates with Claude Code** - all other agents are pure ACP participants
- **Orchestrator serves as the single interface** representing the entire multi-agent system

## Decision
MAOS will be implemented as a **simplified MCP server** that exposes orchestration lifecycle capabilities through tools and resources, while spawning agents into an **ACP network** for peer-to-peer communication. The MCP server focuses on session management and streaming ACP network activity back to clients.

### Streamlined Multi-Agent Single Server Architecture
```
┌───────────────────────────────────────────────────────────┐
│                    Claude Code (MCP Client)               │
│                                                           │
│  maos/orchestrate ──► Start orchestration session         │
│  maos/session-status ──► Monitor progress                 │
│  maos/list-roles ──► List available agent roles           │
└─────────────────────┬─────────────────────────────────────┘
                      │ MCP Protocol
                      ▼
┌───────────────────────────────────────────────────────────┐
│                     MAOS MCP Server                       │
│                                                           │
│  • Provides MCP tools for orchestration                   │
│  • Tracks session state                                   │
│  • Streams orchestrator output to Claude Code             │
└─────────────────────┬─────────────────────────────────────┘
                      │ Spawns Orchestrator
                      ▼
┌───────────────────────────────────────────────────────────┐
│          Orchestrator (Router Agent) - ACP Server         │
│                                                           │
│  • Analyzes tasks and plans phases                        │
│  • Routes work to Claude Code Agent                       │
│  • Tracks progress and adapts planning                    │
└─────────────────────┬─────────────────────────────────────┘
                      │ ACP Requests
                      ▼
┌───────────────────────────────────────────────────────────┐
│             Claude Code Agent - ACP Server                │
│                                                           │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐  │
│  │ Claude CLI  │     │ Claude CLI  │     │ Claude CLI  │  │
│  │ -p architect│     │ -p backend  │     │ -p frontend │  │
│  │ Process     │     │ Process     │     │ Process     │  │
│  └─────────────┘     └─────────────┘     └─────────────┘  │
│                                                           │
│  • Single ACP server manages multiple CLI processes       │
│  • Each process has different role via -p flag            │
│  • Session continuity via --session-id                    │
│  • Efficient resource utilization                         │
└───────────────────────────────────────────────────────────┘
```

### Simplified MCP Tools and Resources

With ACP integration, MAOS exposes a **simplified set** of orchestration capabilities through **three MCP tools** and **three resources**. The complete tool definitions and schemas are documented in the [MCP Tools Reference](../references/mcp-tools.md).

**Tools:**
1. `maos/orchestrate` - Start multi-agent orchestration sessions (spawns agents into ACP network)
2. `maos/session-status` - Query orchestration session status and ACP network activity
3. `maos/list-roles` - List available agent roles

**Resources:**
1. **Orchestrator output (SSE)** - Real-time output from the Orchestrator agent only
2. **Session status** - Current state of orchestration and ACP network
3. **Agent discovery info** - Available agents and their ACP endpoints for debugging

### Orchestrator-Only Streaming

**Critical Architectural Decision**: The MCP server streams **only Orchestrator output** to Claude Code:

**Why Orchestrator-Only:**
- **Single Point of Interaction**: Orchestrator represents the entire multi-agent system
- **Clean Interface**: Claude Code users see unified output, not chaos from multiple agents
- **Clear Delegation**: Orchestrator coordinates via ACP, reports results via MCP
- **Simplified Architecture**: No complex multi-agent output multiplexing needed

**SSE Event Types:**
- **`orchestrator-output`** - Real-time output from Orchestrator agent
- **`session-status`** - High-level orchestration progress updates
- **`session-complete`** - Final orchestration results

**Streaming Architecture:**
- **Monitor Only Orchestrator**: Subscribe to Orchestrator's stdout/stderr
- **ACP Coordination Hidden**: All inter-agent communication happens via ACP (invisible to MCP)
- **Unified Experience**: Claude Code sees clean, coordinated output
- **Performance**: Single agent stream vs. complex multi-agent multiplexing

### MCP Server Configuration

The server configuration and transport details are documented in the [MCP Tools Reference](../references/mcp-tools.md#mcp-server-configuration).

## Implementation Strategy with ACP Integration

### Phase 1: Simplified MCP Server + ACP Network
1. Implement simplified MCP protocol handler (fewer tools!)
2. Create **ACP-integrated** tool definitions
3. **ACP server spawning** with agent processes
4. **Orchestrator-only output streaming** (clean, unified interface)

### Phase 2: ACP Network Features
1. **ACP discovery and monitoring**
2. Session persistence with ACP network state
3. **Coordinated error recovery** (CLI + ACP server)

### Phase 3: Production Features
1. Multi-instance support with **port pool management**
2. Resource limits including **network and port allocation**
3. Security sandboxing for both CLI and ACP processes
4. **ACP network performance monitoring**

## Consequences

### Positive
- **Dramatically Simplified MCP Server**: Clean, focused architecture with minimal tools
- **Orchestrator-Only Interface**: Clean, unified user experience via single agent interface
- **Natural Integration**: Users interact with MAOS through their preferred AI tool
- **Language Processing**: The LLM handles natural language parsing
- **Hidden Complexity**: ACP coordination happens behind the scenes, invisible to users
- **Tool Agnostic**: Works with any MCP-compatible client
- **Clean Separation**: MCP ↔ Orchestrator ↔ ACP Network
- **Streamlined Tool Set**: Just 3 core tools for orchestration
- **Performance**: Single agent output stream vs. complex multi-agent multiplexing
- **Clear Delegation Model**: Orchestrator represents entire multi-agent system

### Negative
- **MCP Dependency**: Requires clients to support MCP
- **Network Overhead**: HTTP/SSE + ACP adds network usage
- **Limited by MCP**: Must work within protocol constraints
- **ACP Integration Complexity**: Requires ACP protocol implementation

### Mitigation
- **Simplified Architecture**: ACP integration actually reduces overall complexity
- **Efficient ACP Implementation**: Use lightweight HTTP servers for ACP
- **Optimize SSE Streaming**: Single ACP network stream is more efficient
- **Contribute to Standards**: Engage with both MCP and ACP communities
- **Provide Standalone CLI**: For non-MCP users who want direct access

## References
- **ADR-04: ACP-Based Agent Communication** - Defines the Multi-Agent Single Server architecture
- **ADR-08: Agent Lifecycle and Management** - Provides CLI process management for Claude Code Agent
- **ADR-11: Adaptive Phase-Based Orchestration** - Defines Router Agent pattern for Orchestrator
- [MCP Tools Reference](../references/mcp-tools.md) - Complete tool and resource definitions
- [Agent Roles Reference](../references/agent-roles.md) - Agent role specifications
- [Agent Communication Protocol (ACP)](https://agentcommunicationprotocol.dev/) - Internal coordination protocol
- [Model Context Protocol Specification](https://modelcontextprotocol.io) - External interface protocol
- [Claude Code MCP Documentation](https://docs.anthropic.com/en/docs/claude-code/mcp) - Client integration
- SSE (Server-Sent Events) for real-time streaming

---
*Date: 2025-07-09*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*