# MAOS Implementation Roadmap: PTY Multiplexer Architecture

*This document outlines the major implementation epics for building the **Orchestrator-as-PTY-Multiplexer** Multi-Agent Orchestration System (MAOS) based on our architectural innovations.*

## 🚀 Architecture Overview

MAOS represents a **paradigm shift** in multi-agent systems through:

```
Claude Code ↔ MCP Server ↔ Orchestrator (PTY MULTIPLEXER)
                              ↕ PTY Control  
                          Specialist Agents (Claude CLI)
```

**🎯 Key Breakthroughs:**
- **Perfect Context Continuity**: Agents never lose memory via Claude CLI session binding
- **Intelligent Agent Pool Management**: Sleep/reactivate agents while preserving full context
- **Maximum Focus & Productivity**: Agents work in isolation without interruptions (proven efficient)
- **Minimal Communication**: PTY used only for essential coordination, not chatter
- **Orchestrator-as-PTY-Multiplexer**: Single point of interaction with Claude Code
- **Pure PTY Control**: All agents managed via direct process control  
- **Configurable Transparency**: Users can choose unified progress view or visible agent terminals
- **Dual Lifecycle Support**: Both micro-task (high-efficiency) and phase-based patterns
- **Cross-Platform Compatibility**: Works on Windows, macOS, Linux via portable-pty
- **No Network Dependencies**: Eliminated port management and network complexity

The implementation is organized into **5 major epics**, each building toward the complete PTY multiplexer orchestration system.

## Implementation Epics Overview

<details>
<summary><strong>Epic 1: PTY Multiplexer Foundation & Domain Core 🚀</strong> (4-6 weeks)</summary>

## Epic 1: PTY Multiplexer Foundation & Domain Core 🚀

**Epic: PTY Integration & Domain Foundation**

*Dependencies: None (foundational layer)*

### 🎯 Scope
Implement the **PTY-integrated domain models** and core infrastructure for the Orchestrator-as-PTY-Multiplexer architecture. This epic establishes the foundation for direct process control.

### 🔧 Key Components
- **PTY-Aware Domain Models**: Session, Agent, PtyProcess aggregates with process control capabilities
- **PTY Value Objects**: AgentRole, PtyId, ProcessId, SessionId, ExecutionState
- **PTY Resource Management**: PTY pair allocation/deallocation via portable-pty
- **Storage Schema**: SQLite for session metadata + file system for shared artifacts
- **PTY Message Models**: Structured message formats for agent coordination
- **Single Process State**: Track Claude CLI process lifecycle via PTY

### 🚀 Key Features
- **Single Claude CLI Process**: Each agent is just a Claude CLI process controlled via PTY
- **Cross-Platform PTY**: portable-pty handles Windows, macOS, Linux differences
- **PTY Process Models**: Agent registration and process control data structures
- **Simplified State Tracking**: Monitor single Claude CLI process health

### 📦 Deliverables
- PTY-integrated domain model implementation
- PTY resource management system via portable-pty
- SQLite schema with PTY process tracking
- PTY message format definitions
- Single process state management
- Comprehensive unit tests for PTY integration

---

</details>

<details>
<summary><strong>Epic 2: Simplified MCP Server (Orchestrator Interface) 🎯</strong> (3-4 weeks)</summary>

## Epic 2: Simplified MCP Server (Orchestrator Interface) 🎯

**Epic: Simplified MCP Server Implementation**

*Dependencies: Epic 1 (PTY Foundation)*

### 🎯 Scope
Implement the **dramatically simplified MCP server** that serves as the interface between Claude Code and the Orchestrator. This epic eliminates network complexity and focuses on clean orchestration lifecycle management.

### 🔧 Key Components (SIMPLIFIED!)
- **Streamlined MCP Server**: HTTP/SSE protocol implementation (fewer tools!)
- **Core Tool Definitions**: `maos/orchestrate`, `maos/session-status`, `maos/list-roles` ONLY
- **Orchestrator-Only Streaming**: SSE streaming of ONLY Orchestrator output
- **Session Management**: Track orchestration sessions and PTY multiplexer state
- **Clean Error Handling**: Proper MCP protocol compliance

### 🚀 Key Simplifications
- **ELIMINATED**: Complex networking and port management
- **ELIMINATED**: Multi-agent communication tools
- **ELIMINATED**: Complex multi-agent output multiplexing
- **SIMPLIFIED**: Single Orchestrator output stream to Claude Code
- **FOCUSED**: Pure orchestration lifecycle management

### 📦 Deliverables
- Simplified MCP server with 3 core tools
- Orchestrator-only SSE streaming implementation
- Session lifecycle management
- PTY multiplexer state monitoring
- Clean MCP protocol compliance
- Integration tests with Claude Code

---

</details>

<details>
<summary><strong>Epic 3: PTY Agent Network (Single Process Management) ⚡</strong> (3-4 weeks)</summary>

## Epic 3: PTY Agent Network (Single Process Management) ⚡

**Epic: PTY-Integrated Agent Lifecycle & Multiplexer**

*Dependencies: Epic 1 (PTY Foundation), Epic 2 (MCP Server)*

### 🎯 Scope
Implement the **single process agent management** system where every agent is just a Claude CLI process controlled via PTY by the Orchestrator multiplexer. This epic creates the hub-and-spoke agent communication system.

### 🔧 Key Components
- **Single Process Spawning**: Claude CLI process per agent controlled via PTY
- **PTY Resource Management**: Allocate PTY pairs via portable-pty for each agent
- **Orchestrator Registry**: Agent registration and tracking via central multiplexer
- **PTY Health Monitoring**: Monitor Claude CLI process health via PTY
- **Role Template System**: 20 specialized agent role templates with PTY integration
- **Coordinated Shutdown**: Graceful termination of PTY processes

### 🚀 Key Features
- **Every Agent = Claude CLI Process**: Simple, reliable single process model
- **No Network Dependencies**: ELIMINATED - pure PTY communication
- **Central Registration**: Agents register with Orchestrator multiplexer
- **Real-time Status**: Instant agent health and progress updates via PTY
- **Cross-Platform**: PTY works on Windows, macOS, Linux via portable-pty

### 📦 Deliverables
- Single process agent spawning system via PTY
- PTY multiplexer integration for all agents
- PTY resource management and allocation
- Orchestrator agent registry and tracking
- PTY health monitoring for Claude CLI processes
- Complete role template library with PTY integration
- Coordinated agent lifecycle tests
- PTY multiplexer resilience testing

---

</details>

<details>
<summary><strong>Epic 4: Orchestrator-as-PTY-Multiplexer (Advanced Coordination) 👑</strong> (4-6 weeks)</summary>

## Epic 4: Orchestrator-as-PTY-Multiplexer (Advanced Coordination) 👑

**Epic: PTY Multiplexer Orchestrator Implementation**

*Dependencies: Epic 3 (PTY Agent Network)*

### 🎯 Scope
Implement the **game-changing Orchestrator-as-PTY-Multiplexer** pattern where the Orchestrator serves as both the single point of interaction with Claude Code AND the PTY multiplexer coordinator. This is the crown jewel of MAOS.

### 🔧 Key Components
- **PTY Multiplexer Orchestrator**: Single interface to Claude Code + PTY process coordinator
- **PTY-Based Phase Management**: Coordinate phases via direct agent communication
- **Adaptive Planning**: Plan phases based on real-time agent feedback via PTY
- **Unified Progress Reporting**: Present clean, coordinated updates to Claude Code
- **Configurable Transparency Management**: PTY processes can be visible or hidden based on user preference
- **Real-time Agent Coordination**: Direct specialist agent coordination via PTY

### 🚀 Key Features
- **MAXIMUM AGENT FOCUS**: Agents work in isolation without interruptions
- **MINIMAL ESSENTIAL COMMUNICATION**: PTY used only when coordination required
- **SINGLE INTERFACE**: Only Orchestrator talks to Claude Code users
- **PURE PTY COORDINATION**: All agent coordination via essential-only PTY messages
- **CONFIGURABLE TRANSPARENCY**: Users can watch agents work or see unified progress
- **DUAL LIFECYCLE SUPPORT**: Both micro-task and phase-based agent patterns
- **REAL-TIME ADAPTATION**: Plan adjusts based on live agent feedback
- **NO NETWORK COMPLEXITY**: Direct process control eliminates networking
- **CLEAN USER EXPERIENCE**: Professional, unified project management interface
- **CROSS-PLATFORM**: Works anywhere Claude CLI and portable-pty work

### 📦 Deliverables
- PTY multiplexer Orchestrator agent implementation
- PTY-based adaptive phase management
- Real-time agent task assignment via PTY
- Unified progress reporting to Claude Code
- Phase-gate coordination via PTY messages
- Agent discovery and specialist allocation
- Clean user interface with optional agent terminal visibility
- End-to-end orchestration testing with PTY multiplexer

---

</details>

<details>
<summary><strong>Epic 5: Production-Ready PTY System 🏆</strong> (3-4 weeks)</summary>

## Epic 5: Production-Ready PTY System 🏆

**Epic: PTY Multiplexer Production Features**

*Dependencies: Epic 4 (Orchestrator-as-PTY-Multiplexer)*

### 🎯 Scope
Implement production-ready features for the **PTY multiplexer orchestration system** including multi-instance support, PTY monitoring, security, and operational capabilities. This epic delivers a **production-ready multi-agent system**.

### 🔧 Key Components
- **PTY Multiplexer State Management**: Complete PTY process state persistence and recovery
- **Multi-Session Support**: Multiple concurrent orchestration sessions with PTY isolation
- **PTY Process Monitoring**: Comprehensive monitoring of agent communication and health
- **Security & Sandboxing**: Resource limits for Claude CLI processes
- **PTY Resource Management**: Production-grade PTY allocation and cleanup
- **PTY Process Recovery**: Coordinated recovery of Claude CLI processes with session restoration
- **Performance Optimization**: PTY communication optimization and resource management

### 🚀 Production Features
- **PTY Multiplexer Resilience**: Handle agent failures gracefully in PTY system
- **Session Isolation**: Multiple orchestration sessions with separate PTY multiplexers
- **Comprehensive Monitoring**: Full visibility into PTY multiplexer activity and performance
- **Resource Management**: Manage process resources and PTY pairs efficiently
- **Production Security**: Sandboxed agents with controlled PTY communication
- **Cross-Platform Performance**: Optimized PTY message routing and agent coordination

### 📦 Deliverables
- Multi-session PTY multiplexer architecture
- Comprehensive PTY process monitoring and observability
- Production-grade security and sandboxing for Claude CLI processes
- PTY multiplexer state persistence and recovery
- Performance optimization and resource management
- Automated cleanup of sessions, processes, and PTY resources
- Production deployment and operational documentation
- **COMPLETE PRODUCTION-READY MAOS SYSTEM** 🚀

---

</details>

## 🚀 Implementation Strategy

### 🎯 PTY-First Development Approach
1. **Sequential Epic Development**: Each epic builds toward complete PTY multiplexer system
2. **PTY Integration MVP**: Test PTY communication at each epic
3. **Single Process Testing**: Comprehensive testing of Claude CLI process control
4. **Architecture Validation**: Prove Orchestrator-as-PTY-Multiplexer pattern works

### 🏆 Success Criteria
- **PTY Multiplexer Functional**: Agents communicate seamlessly via PTY at each epic
- **Orchestrator Interface Works**: Single, clean interface to Claude Code maintained
- **Transparency Options**: Users can choose between clean unified view or visible agent terminals
- **Performance Benchmarks**: PTY communication meets performance requirements
- **Production Deployment Ready**: Complete, scalable, secure multi-agent system

### ⏱️ Timeline
- **Epic 1-2**: PTY Foundation (4-6 weeks) - Simplified due to eliminated network complexity
- **Epic 3**: PTY Agent Network (3-4 weeks) - Build the multiplexer system
- **Epic 4**: Orchestrator-as-Interface (6-8 weeks) - The crown jewel implementation
- **Epic 5**: Production PTY System (3-4 weeks) - Polish and production features
- **Total Estimated**: **20-27 weeks** for complete system

### 🎯 Key Milestones
- **Week 6**: First PTY agent communication working
- **Week 10**: Complete PTY multiplexer system operational
- **Week 21**: Orchestrator-as-Interface fully functional
- **Week 27**: Production-ready MAOS system delivered

---

## References

### 🔥 Architecture Documentation
- **[ADR-04: Orchestrator-as-PTY-Multiplexer Communication](./decisions/04-agent-communication-patterns.md)** - Hub-and-spoke agent control
- **[ADR-08: Agent Lifecycle and PTY Multiplexer Management](./decisions/08-agent-lifecycle-and-management.md)** - Single process management (Claude CLI via PTY)
- **[ADR-10: MCP Server Architecture](./decisions/10-mcp-server-architecture.md)** - Simplified MCP server with Orchestrator-only interface
- **[ADR-11: Adaptive Phase-Based Orchestration](./decisions/11-adaptive-phase-based-orchestration.md)** - Orchestrator-as-Interface pattern

### 🌐 External Standards and Protocols
- **[Tmux-Orchestrator](https://github.com/Jedward23/Tmux-Orchestrator)** - Inspiration for PTY multiplexer patterns
- **[portable-pty](https://docs.rs/portable-pty/)** - Cross-platform PTY implementation
- **[Model Context Protocol (MCP)](https://modelcontextprotocol.io)** - External interface to Claude Code

### 📚 Supporting Documentation
- [Agent Roles](./references/agent-roles.md) - 20 specialized agent role definitions
- [MCP Tools](./references/mcp-tools.md) - Simplified MCP tool definitions
- [Storage Schema](./references/storage-schema.md) - PTY-integrated database design
- [POC Learnings](./references/poc-learnings.md) - Lessons learned leading to the PTY multiplexer concept

---

*Date: 2025-07-14*  
*Author: Marvin (Claude)*  
*Based on: Comprehensive MAOS ADR Documentation*