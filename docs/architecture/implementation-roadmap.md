# MAOS Implementation Roadmap: Revolutionary ACP Architecture

*This document outlines the major implementation epics for building the world's first **Orchestrator-as-Interface + ACP Network** Multi-Agent Orchestration System (MAOS) based on our revolutionary architectural breakthroughs.*

## 🚀 Revolutionary Architecture Overview

MAOS represents a **paradigm shift** in multi-agent systems through:

```
Claude Code ↔ MCP Server ↔ Orchestrator (SINGLE INTERFACE)
                              ↕ ACP Network  
                          Specialist Agents (PURE ACP)
```

**🎯 Key Breakthroughs:**
- **Perfect Context Continuity**: Agents never lose memory via Claude Code session binding (revolutionary!)
- **Intelligent Agent Pool Management**: Sleep/reactivate agents while preserving full context
- **Maximum Focus & Productivity**: Agents work in isolation without interruptions (proven efficient)
- **Minimal Communication**: ACP used only for essential coordination, not chatter
- **Orchestrator-as-Interface**: Single point of interaction with Claude Code
- **Pure ACP Network**: All agents communicate via standardized Agent Communication Protocol  
- **Hidden Complexity**: Users see unified progress, not agent coordination chaos
- **Dual Lifecycle Support**: Both micro-task (high-efficiency) and phase-based patterns
- **Eliminated Bottlenecks**: No central message routing or broken tools

The implementation is organized into **5 major epics**, each building toward the complete ACP-based orchestration system.

## Implementation Epics Overview

<details>
<summary><strong>Epic 1: ACP Foundation & Domain Core 🚀</strong> (6-8 weeks)</summary>

## Epic 1: ACP Foundation & Domain Core 🚀

**Epic: ACP Integration & Domain Foundation**

*Dependencies: None (foundational layer)*

### 🎯 Scope
Implement the **ACP-integrated domain models** and core infrastructure for the revolutionary Orchestrator-as-Interface architecture. This epic establishes the foundation for peer-to-peer agent communication.

### 🔧 Key Components
- **ACP-Aware Domain Models**: Session, Agent, AcpEndpoint aggregates with communication capabilities
- **ACP Value Objects**: AgentRole, AcpPort, AcpEndpoint, SessionId, ExecutionState
- **Port Pool Management**: Port allocation/deallocation for ACP servers
- **Storage Schema**: SQLite for session metadata + file system for shared artifacts
- **ACP Message Models**: Standardized message formats for agent coordination
- **Dual Process State**: Track both CLI process and ACP server lifecycle

### 🚀 Revolutionary Features
- **Agent Process + ACP Server**: Every agent runs dual processes
- **Port Management**: Unique port allocation for each agent's ACP server
- **ACP Discovery Models**: Agent registration and discovery data structures
- **Unified State Tracking**: Monitor both CLI and ACP server health

### 📦 Deliverables
- ACP-integrated domain model implementation
- Port pool management system
- SQLite schema with ACP endpoint tracking
- ACP message format definitions
- Dual process state management
- Comprehensive unit tests for ACP integration

---

</details>

<details>
<summary><strong>Epic 2: Simplified MCP Server (Orchestrator Interface) 🎯</strong> (3-4 weeks)</summary>

## Epic 2: Simplified MCP Server (Orchestrator Interface) 🎯

**Epic: Revolutionary MCP Server Implementation**

*Dependencies: Epic 1 (ACP Foundation)*

### 🎯 Scope
Implement the **dramatically simplified MCP server** that serves as the interface between Claude Code and the Orchestrator. This epic eliminates broken tools and focuses on clean orchestration lifecycle management.

### 🔧 Key Components (SIMPLIFIED!)
- **Streamlined MCP Server**: HTTP/SSE protocol implementation (fewer tools!)
- **Core Tool Definitions**: `maos/orchestrate`, `maos/session-status`, `maos/list-roles` ONLY
- **Orchestrator-Only Streaming**: SSE streaming of ONLY Orchestrator output
- **Session Management**: Track orchestration sessions and ACP network state
- **Clean Error Handling**: Proper MCP protocol compliance

### 🚀 Revolutionary Simplifications
- **ELIMINATED**: `maos/agent-message` tool (broken and unnecessary!)
- **ELIMINATED**: `maos/spawn-agent` tool (handled by orchestration!)
- **ELIMINATED**: Complex multi-agent output multiplexing
- **SIMPLIFIED**: Single Orchestrator output stream to Claude Code
- **FOCUSED**: Pure orchestration lifecycle management

### 📦 Deliverables
- Simplified MCP server with 3 core tools
- Orchestrator-only SSE streaming implementation
- Session lifecycle management
- ACP network state monitoring
- Clean MCP protocol compliance
- Integration tests with Claude Code

---

</details>

<details>
<summary><strong>Epic 3: ACP Agent Network (Dual Process Management) ⚡</strong> (4-5 weeks)</summary>

## Epic 3: ACP Agent Network (Dual Process Management) ⚡

**Epic: ACP-Integrated Agent Lifecycle & Network**

*Dependencies: Epic 1 (ACP Foundation), Epic 2 (MCP Server)*

### 🎯 Scope
Implement the revolutionary **dual process agent management** system where every agent runs both a CLI process AND an ACP server. This epic creates the peer-to-peer agent communication network.

### 🔧 Key Components
- **Dual Process Spawning**: CLI process + ACP server for each agent
- **Port Pool Management**: Allocate unique ports for each agent's ACP server
- **ACP Network Integration**: Agent registration and discovery via ACP
- **Dual Health Monitoring**: Monitor both CLI process and ACP server health
- **Role Template System**: 20 specialized agent role templates with ACP integration
- **Coordinated Shutdown**: Graceful termination of both processes

### 🚀 Revolutionary Features
- **Every Agent = ACP Server**: Peer-to-peer communication capability
- **No File-Based Messaging**: ELIMINATED - pure ACP communication
- **Dynamic Discovery**: Agents find each other through ACP network
- **Real-time Status**: Instant agent health and progress updates
- **Network Resilience**: ACP network handles agent failures gracefully

### 📦 Deliverables
- Dual process agent spawning system
- ACP server integration for all agents
- Port pool management and allocation
- ACP network registration and discovery
- Dual health monitoring (CLI + ACP)
- Complete role template library with ACP integration
- Coordinated agent lifecycle tests
- ACP network resilience testing

---

</details>

<details>
<summary><strong>Epic 4: Orchestrator-as-Interface (Revolutionary Coordination) 👑</strong> (6-8 weeks)</summary>

## Epic 4: Orchestrator-as-Interface (Revolutionary Coordination) 👑

**Epic: Dual-Role Orchestrator Implementation**

*Dependencies: Epic 3 (ACP Agent Network)*

### 🎯 Scope
Implement the **game-changing Orchestrator-as-Interface** pattern where the Orchestrator serves as both the single point of interaction with Claude Code AND the ACP network coordinator. This is the crown jewel of MAOS.

### 🔧 Key Components
- **Dual-Role Orchestrator**: Single interface to Claude Code + ACP network coordinator
- **ACP-Based Phase Management**: Coordinate phases via direct agent communication
- **Adaptive Planning**: Plan phases based on real-time agent feedback via ACP
- **Unified Progress Reporting**: Present clean, coordinated updates to Claude Code
- **Hidden Complexity Management**: ACP coordination invisible to users
- **Real-time Agent Coordination**: Direct specialist agent coordination via ACP

### 🚀 Revolutionary Features
- **MAXIMUM AGENT FOCUS**: Agents work in isolation without interruptions
- **MINIMAL ESSENTIAL COMMUNICATION**: ACP used only when coordination required
- **SINGLE INTERFACE**: Only Orchestrator talks to Claude Code users
- **PURE ACP COORDINATION**: All agent coordination via essential-only ACP messages
- **HIDDEN COMPLEXITY**: Users see unified progress, not agent chaos
- **DUAL LIFECYCLE SUPPORT**: Both micro-task and phase-based agent patterns
- **REAL-TIME ADAPTATION**: Plan adjusts based on live agent feedback
- **NO SUMMARIZERS NEEDED**: Direct agent communication eliminates need
- **CLEAN USER EXPERIENCE**: Professional, unified project management interface

### 📦 Deliverables
- Dual-role Orchestrator agent implementation
- ACP-based adaptive phase management
- Real-time agent task assignment via ACP
- Unified progress reporting to Claude Code
- Phase-gate coordination via ACP messages
- Agent discovery and specialist allocation
- Clean user interface with hidden complexity
- End-to-end orchestration testing with ACP network

---

</details>

<details>
<summary><strong>Epic 5: Production-Ready ACP System 🏆</strong> (4-6 weeks)</summary>

## Epic 5: Production-Ready ACP System 🏆

**Epic: ACP Network Production Features**

*Dependencies: Epic 4 (Orchestrator-as-Interface)*

### 🎯 Scope
Implement production-ready features for the **ACP network orchestration system** including multi-instance support, ACP network monitoring, security, and operational capabilities. This epic delivers a **production-ready revolutionary multi-agent system**.

### 🔧 Key Components
- **ACP Network State Management**: Complete ACP network state persistence and recovery
- **Multi-Session Support**: Multiple concurrent orchestration sessions with ACP isolation
- **ACP Network Monitoring**: Comprehensive monitoring of agent communication and health
- **Security & Sandboxing**: Resource limits for both CLI processes and ACP servers
- **Port Pool Management**: Production-grade port allocation and cleanup
- **ACP Network Recovery**: Coordinated recovery of CLI processes and ACP servers
- **Performance Optimization**: ACP communication optimization and resource management

### 🚀 Revolutionary Production Features
- **ACP Network Resilience**: Handle agent failures gracefully in ACP network
- **Session Isolation**: Multiple orchestration sessions with separate ACP networks
- **Comprehensive Monitoring**: Full visibility into ACP network activity and performance
- **Resource Management**: Manage both process resources and network ports
- **Production Security**: Sandboxed agents with controlled ACP communication
- **Performance Tuning**: Optimized ACP message routing and agent coordination

### 📦 Deliverables
- Multi-session ACP network architecture
- Comprehensive ACP network monitoring and observability
- Production-grade security and sandboxing for dual processes
- ACP network state persistence and recovery
- Performance optimization and resource management
- Automated cleanup of sessions, processes, and ports
- Production deployment and operational documentation
- **COMPLETE PRODUCTION-READY MAOS SYSTEM** 🚀

---

</details>

## 🚀 Revolutionary Implementation Strategy

### 🎯 ACP-First Development Approach
1. **Sequential Epic Development**: Each epic builds toward complete ACP network
2. **ACP Integration MVP**: Test ACP communication at each epic
3. **Dual Process Testing**: Comprehensive testing of CLI + ACP server combinations
4. **Revolutionary Architecture Validation**: Prove Orchestrator-as-Interface pattern works

### 🏆 Success Criteria
- **ACP Network Functional**: Agents communicate seamlessly via ACP at each epic
- **Orchestrator Interface Works**: Single, clean interface to Claude Code maintained
- **Hidden Complexity Achieved**: Users never see agent coordination complexity
- **Performance Benchmarks**: ACP communication meets performance requirements
- **Production Deployment Ready**: Complete, scalable, secure multi-agent system

### ⏱️ Revolutionary Timeline
- **Epic 1-2**: ACP Foundation (6-8 weeks) - Longer due to revolutionary architecture
- **Epic 3**: ACP Agent Network (4-5 weeks) - Build the peer-to-peer network
- **Epic 4**: Orchestrator-as-Interface (6-8 weeks) - The crown jewel implementation
- **Epic 5**: Production ACP System (4-6 weeks) - Polish and production features
- **Total Estimated**: **20-27 weeks** for complete revolutionary system

### 🎯 Key Milestones
- **Week 8**: First ACP agent communication working
- **Week 13**: Complete ACP agent network operational
- **Week 21**: Orchestrator-as-Interface fully functional
- **Week 27**: Production-ready revolutionary MAOS system delivered

---

## References

### 🔥 Revolutionary Architecture Documentation
- **[ADR-04: ACP-Based Agent Communication](./decisions/04-agent-communication-patterns.md)** - Revolutionary peer-to-peer agent network
- **[ADR-08: Agent Lifecycle and Management](./decisions/08-agent-lifecycle-and-management.md)** - Dual process management (CLI + ACP)
- **[ADR-10: MCP Server Architecture](./decisions/10-mcp-server-architecture.md)** - Simplified MCP server with Orchestrator-only interface
- **[ADR-11: Adaptive Phase-Based Orchestration](./decisions/11-adaptive-phase-based-orchestration.md)** - Orchestrator-as-Interface pattern

### 🌐 External Standards and Protocols
- **[Agent Communication Protocol (ACP)](https://agentcommunicationprotocol.dev/)** - Core agent communication standard
- **[Model Context Protocol (MCP)](https://modelcontextprotocol.io)** - External interface to Claude Code

### 📚 Supporting Documentation
- [Agent Roles](./references/agent-roles.md) - 20 specialized agent role definitions
- [MCP Tools](./references/mcp-tools.md) - Simplified MCP tool definitions
- [Storage Schema](./references/storage-schema.md) - ACP-integrated database design
- [POC Learnings](./references/poc-learnings.md) - Lessons that led to ACP revolution

---

*Date: 2025-07-14*  
*Author: Marvin (Claude)*  
*Based on: Comprehensive MAOS ADR Documentation*