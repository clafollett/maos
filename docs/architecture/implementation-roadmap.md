# MAOS Implementation Roadmap

This document outlines the implementation plan for the Multi-Agent Orchestration System (MAOS) based on our streamlined architecture.

## Architecture Overview

MAOS implements a clean, efficient multi-agent orchestration system:

```
Claude Code ↔ MCP Server ↔ Orchestrator (Router Agent)
                              ↕ ACP Protocol  
                          Claude Code Agent (manages CLI processes)
```

## Key Design Principles

- **Simplified Architecture**: Single Claude Code Agent manages all CLI processes
- **Session Continuity**: Claude's `--session-id` preserves context across phases
- **Intelligent Orchestration**: Orchestrator uses Claude for smart agent selection
- **Clean Separation**: MCP for external interface, ACP for internal coordination
- **Phase-Based Execution**: Adaptive planning based on actual outputs

## Implementation Phases

### Phase 1: Core Infrastructure (4-6 weeks)

**Objective**: Establish foundational components and basic orchestration

**Key Components:**
- **Domain Models**: Session, Agent, Orchestrator entities
- **Value Objects**: AgentRole, SessionId, OrchestrationState
- **Storage**: SQLite for session metadata + file system for artifacts
- **Basic ACP**: Simple request/response protocol for Claude Code Agent

**Deliverables:**
- Core domain model implementation
- SQLite schema for session tracking
- Basic ACP protocol implementation
- Unit tests for core components

### Phase 2: MCP Server (3-4 weeks)

**Objective**: Implement MCP server with 3 essential tools

**Key Components:**
- **MCP Protocol**: HTTP/SSE server implementation
- **Core Tools**: `orchestrate`, `session-status`, `list-roles`
- **Orchestrator Spawning**: Launch Orchestrator as separate process
- **SSE Streaming**: Real-time Orchestrator output streaming

**Deliverables:**
- MCP server with 3 tools
- Orchestrator process management
- SSE streaming implementation
- MCP protocol compliance testing

### Phase 3: Claude Code Agent (4-5 weeks)

**Objective**: Implement agent that manages Claude CLI processes

**Key Components:**
- **ACP Server**: Single server managing multiple CLI processes
- **Process Management**: Spawn/monitor/cleanup Claude CLI processes
- **Session Tracking**: Map roles to session IDs
- **Resource Limits**: Concurrent process limits and monitoring

**Deliverables:**
- Claude Code Agent ACP server
- CLI process lifecycle management
- Session registry implementation
- Resource monitoring and limits

### Phase 4: Intelligent Orchestrator (5-7 weeks)

**Objective**: Implement Router Agent with intelligent decision-making

**Key Components:**
- **Orchestrator Process**: Main orchestration logic
- **Phase Planning**: Adaptive phase-by-phase execution
- **Agent Selection**: Claude-powered intelligent agent assignment
- **Session Registry**: Track agent work history and context
- **Progress Reporting**: Unified updates to Claude Code

**Deliverables:**
- Orchestrator Router Agent implementation
- Intelligent agent selection system
- Phase-based execution engine
- Session registry with work context tracking

### Phase 5: Production Features (3-4 weeks)

**Objective**: Production-ready system with monitoring and resilience

**Key Components:**
- **Multi-Session Support**: Handle concurrent orchestration sessions
- **Error Recovery**: Graceful handling of process failures
- **Monitoring**: Health checks and performance metrics
- **Security**: Process sandboxing and resource limits

**Deliverables:**
- Multi-session orchestration support
- Comprehensive error handling and recovery
- Production monitoring and observability
- Security hardening and resource controls

## Implementation Strategy

### Development Approach
1. **Incremental Build**: Each phase builds on previous work
2. **Test-Driven**: Comprehensive testing at each phase
3. **Documentation**: Update docs as architecture evolves
4. **Validation**: Verify simplified architecture assumptions

### Success Criteria
- **Clean Architecture**: Simple, maintainable codebase
- **Reliable Orchestration**: Consistent multi-agent coordination
- **Context Preservation**: Agents maintain memory across phases
- **Performance**: Efficient resource usage and fast response times
- **Extensibility**: Easy to add new agent types and capabilities

### Timeline Estimate
- **Phase 1-2**: Core + MCP (7-10 weeks)
- **Phase 3-4**: Agents + Orchestrator (9-12 weeks)
- **Phase 5**: Production (3-4 weeks)
- **Total**: **19-26 weeks** for complete system

### Key Milestones
- **Week 6**: Basic orchestration working
- **Week 12**: MCP server operational
- **Week 18**: Full agent management
- **Week 24**: Intelligent orchestration complete
- **Week 26**: Production-ready system

## Technical Considerations

### Dependencies
- **Claude CLI**: Core dependency for all agent processes
- **ACP Protocol**: REST-based agent communication
- **MCP Protocol**: External interface to Claude Code
- **SQLite**: Session and metadata storage

### Risk Mitigation
- **Simplified Design**: Reduced complexity lowers implementation risk
- **Proven Components**: Leverage Claude's existing session management
- **Incremental Delivery**: Early feedback on core functionality
- **Comprehensive Testing**: Prevent regressions during development

## References

- **ADR-04**: Agent Communication Patterns - ACP architecture
- **ADR-08**: Agent Lifecycle Management - Process management
- **ADR-10**: MCP Server Architecture - External interface
- **ADR-11**: Adaptive Phase-Based Orchestration - Intelligent coordination

---

*Last Updated: 2025-07-16*
*Author: Development Team*