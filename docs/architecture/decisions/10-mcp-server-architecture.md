# ADR-10: Simplified MCP Server Architecture

## Status
Accepted

## Context
MAOS needs to provide multi-agent orchestration capabilities to AI tools like Claude Code. After analyzing various integration approaches, the Model Context Protocol (MCP) emerged as the ideal solution because:

- **Standardized Integration**: MCP is becoming the standard for AI tool extensions
- **Tool Discovery**: Clients automatically discover MAOS capabilities
- **Streaming Support**: Real-time updates via Server-Sent Events (SSE)
- **Language Agnostic**: Clients don't need to know MAOS is written in Rust

### PTY Multiplexer Simplification
With our **Orchestrator-as-PTY-Multiplexer** architecture, the MCP server role is dramatically simplified:
- **No complex networking**: All agent communication handled via PTY multiplexer
- **Single process interface**: MCP server only communicates with Orchestrator
- **Focus on orchestration lifecycle**: MCP server manages sessions and streams Orchestrator output
- **Simplified architecture**: Clean separation between MCP (external interface) and PTY (internal process control)

Key architectural insights: 
- **MAOS MCP server spawns single Orchestrator Agent**
- **Only the Orchestrator communicates with Claude Code** - all other agents are PTY-controlled processes
- **Orchestrator serves as the single interface** representing the entire multi-agent system

## Decision
MAOS will be implemented as a **simplified MCP server** that exposes orchestration lifecycle capabilities through tools and resources, while spawning a single **Orchestrator Agent** that manages all other agents via PTY multiplexer. The MCP server focuses on session management and streaming Orchestrator output back to clients.

### Simplified PTY Multiplexer Architecture
```
┌────────────────────────────────────────────────────────────────┐
│   Claude Code (MCP Client)                                     │
│   - User types natural language                                │
│   - LLM interprets into tools                                  │
│   - Displays ONLY Orchestrator output                          │
└────────────┬───────────────────────────────────────────────────┘
             │ MCP Protocol (HTTP/SSE)
             │ ONLY communicates with Orchestrator
             ▼
┌────────────────────────────────────────────────────────────────┐
│   MAOS MCP Server (SIMPLIFIED!)                               │
├────────────────────────────────────────────────────────────────┤
│ Tools:                                                         │
│ • maos/orchestrate      ← Start orchestration                 │
│ • maos/session-status   ← Monitor progress                    │
│ • maos/list-roles       ← List available roles              │
├────────────────────────────────────────────────────────────────┤
│ Resources:                                                     │
│ • Orchestrator output (streaming)                             │
│ • Session status                                               │
│ • Agent role definitions                                       │
└────────────┬───────────────────────────────────────────────────┘
             │ Spawns single Orchestrator Agent
             ▼
┌────────────────────────────────────────────────────────────────┐
│   Orchestrator Agent (PTY Multiplexer)                        │
│                                                                │
│  ┌─────────────────┐ ◄─── SINGLE INTERFACE to Claude Code    │
│  │  ORCHESTRATOR   │      • Only agent with MCP connection   │
│  │  (Claude CLI)   │      • Represents entire system         │
│  └─────────┬───────┘      • Manages all other agents via PTY │
│            │ PTY Control (Hub-and-Spoke)                     │
│            ▼                                                  │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     │
│  │Solution     │     │Backend      │     │Frontend     │     │
│  │Architect    │     │Engineer     │     │Engineer     │     │
│  │(Claude CLI) │     │(Claude CLI) │     │(Claude CLI) │     │
│  │ PTY Control │     │ PTY Control │     │ PTY Control │     │
│  │  NO MCP!    │     │  NO MCP!    │     │  NO MCP!    │     │
│  └─────────────┘     └─────────────┘     └─────────────┘     │
│                                                               │
│  • Orchestrator = Single point of interaction                │
│  • All other agents = PTY-controlled Claude CLI processes    │
│  • Clean separation: MCP ↔ Orchestrator ↔ PTY Processes     │
│  • Real-time coordination via PTY read/write                 │
│  • Cross-platform via portable-pty                           │
└────────────────────────────────────────────────────────────────┘
```

### Simplified MCP Tools and Resources

With PTY multiplexer integration, MAOS exposes a **simplified set** of orchestration capabilities through **three MCP tools** and **three resources**. The complete tool definitions and schemas are documented in the [MCP Tools Reference](../references/mcp-tools.md).

**Tools:**
1. `maos/orchestrate` - Start multi-agent orchestration sessions (spawns Orchestrator with PTY multiplexer)
2. `maos/session-status` - Query orchestration session status and agent activity
3. `maos/list-roles` - List available agent roles

**Resources:**
1. **Orchestrator output (SSE)** - Real-time output from the Orchestrator agent only
2. **Session status** - Current state of orchestration and PTY process management
3. **Agent role definitions** - Available agents and their role specifications

### Orchestrator-Only Streaming

**Critical Architectural Decision**: The MCP server streams **only Orchestrator output** to Claude Code:

**Why Orchestrator-Only:**
- **Single Point of Interaction**: Orchestrator represents the entire multi-agent system
- **Clean Interface**: Claude Code users see unified output, not chaos from multiple agents
- **Clear Delegation**: Orchestrator coordinates via PTY multiplexer, reports results via MCP
- **Simplified Architecture**: No complex multi-agent output multiplexing needed

**SSE Event Types:**
- **`orchestrator-output`** - Real-time output from Orchestrator agent
- **`session-status`** - High-level orchestration progress updates
- **`session-complete`** - Final orchestration results

**Streaming Architecture:**
- **Monitor Only Orchestrator**: Subscribe to Orchestrator's stdout/stderr via single Claude CLI process
- **PTY Coordination Hidden**: All inter-agent communication happens via PTY (invisible to MCP)
- **Unified Experience**: Claude Code sees clean, coordinated output
- **Performance**: Single agent stream vs. complex multi-agent multiplexing
- **Cross-Platform**: Works on Windows, macOS, Linux via portable-pty

### MCP Server Configuration

The server configuration and transport details are documented in the [MCP Tools Reference](../references/mcp-tools.md#mcp-server-configuration).

**Key Configuration Elements:**
- **Orchestrator Process**: Single Claude CLI process with session persistence
- **PTY Management**: Cross-platform PTY pair allocation via portable-pty
- **Resource Limits**: Configurable limits on concurrent agents and memory usage
- **Session Persistence**: Claude CLI `--session-id` for memory continuity
- **Workspace Management**: Isolated workspaces with shared context access

## Implementation Strategy with PTY Multiplexer

### Phase 1: Simplified MCP Server + PTY Multiplexer
1. Implement simplified MCP protocol handler (3 core tools)
2. Create **PTY-integrated** tool definitions
3. **Orchestrator process spawning** with PTY multiplexer capabilities
4. **Orchestrator-only output streaming** (clean, unified interface)

### Phase 2: PTY Multiplexer Features
1. **PTY process management and monitoring**
2. Session persistence with Claude CLI session IDs
3. **Coordinated error recovery** (PTY process restart with session restoration)

### Phase 3: Production Features
1. Multi-instance support with **PTY resource management**
2. Resource limits including **process and memory allocation**
3. Security sandboxing for PTY-controlled Claude CLI processes
4. **PTY multiplexer performance monitoring**

## Consequences

### Positive
- **Dramatically Simplified MCP Server**: Clean, focused architecture with minimal tools
- **Orchestrator-Only Interface**: Clean, unified user experience via single agent interface
- **Natural Integration**: Users interact with MAOS through their preferred AI tool
- **Language Processing**: The LLM handles natural language parsing
- **Configurable Transparency**: PTY coordination can be visible for debugging or hidden for clean UX
- **Tool Agnostic**: Works with any MCP-compatible client
- **Clean Separation**: MCP ↔ Orchestrator ↔ PTY Processes
- **Streamlined Tool Set**: Just 3 core tools for orchestration
- **Performance**: Single agent output stream vs. complex multi-agent multiplexing
- **Clear Delegation Model**: Orchestrator represents entire multi-agent system
- **Cross-Platform Compatibility**: Works on Windows, macOS, Linux without dependencies
- **No Network Complexity**: No port management, discovery, or network configuration
- **Simplified Deployment**: Single MCP server process with PTY multiplexer

### Negative
- **MCP Dependency**: Requires clients to support MCP
- **PTY Dependency**: Requires portable-pty for cross-platform compatibility
- **Limited by MCP**: Must work within protocol constraints
- **Process Management Complexity**: Managing multiple Claude CLI processes via PTY

### Mitigation
- **Simplified Architecture**: PTY multiplexer reduces overall complexity vs. networking
- **PTY Abstraction**: portable-pty handles cross-platform PTY differences
- **Optimize SSE Streaming**: Single Orchestrator stream is efficient
- **Contribute to Standards**: Engage with MCP community for improvements
- **Provide Standalone CLI**: For non-MCP users who want direct access
- **Robust Process Monitoring**: Health checks and automatic restart capabilities

## References
- **ADR-04: Orchestrator-as-PTY-Multiplexer Communication** - Defines the PTY communication that simplifies this MCP server
- **ADR-08: Agent Lifecycle and PTY Multiplexer Management** - Provides PTY process spawning infrastructure
- [MCP Tools Reference](../references/mcp-tools.md) - Complete tool and resource definitions
- [Agent Roles Reference](../references/agent-roles.md) - Agent role specifications
- [Tmux-Orchestrator](https://github.com/Jedward23/Tmux-Orchestrator) - Inspiration for PTY multiplexer patterns
- [portable-pty](https://docs.rs/portable-pty/) - Cross-platform PTY implementation
- [Model Context Protocol Specification](https://modelcontextprotocol.io) - MCP external interface protocol
- [Claude Code MCP Documentation](https://docs.anthropic.com/en/docs/claude-code/mcp) - Client integration
- SSE (Server-Sent Events) for real-time streaming

---
*Date: 2025-07-16*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*