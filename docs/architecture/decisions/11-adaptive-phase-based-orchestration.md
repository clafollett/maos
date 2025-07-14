# ADR-11: Adaptive Phase-Based Orchestration

## Status
Accepted

## Context
The POC revealed critical insights about orchestration that shaped our revolutionary ACP-based architecture:

### Original Issues:
- **Central Planning Flaw**: Orchestrator attempting to plan entire project upfront
- **Communication Gaps**: File-based messaging created coordination failures
- **Knowledge Silos**: Engineers working without architectural context
- **Misalignment**: Phases disconnected from each other

### Revolutionary ACP Solution:
With our **Agent Communication Protocol (ACP) integration** and **Orchestrator-as-Interface** pattern, we now have:
- **Real-time coordination**: Direct agent-to-agent communication via ACP
- **Single interface**: Orchestrator as sole representative to Claude Code
- **Adaptive planning**: Orchestrator coordinates with agents via ACP for incremental discovery
- **Hidden complexity**: ACP network coordination invisible to users

Traditional orchestration assumes perfect upfront knowledge. Our ACP-based approach enables **adaptive discovery** where the Orchestrator coordinates real-time with specialist agents via ACP while presenting unified progress to Claude Code.

## Decision
We will implement an **ACP-based adaptive orchestration model** where the Orchestrator operates as both:
1. **Single Interface** to Claude Code (via MCP)
2. **ACP Network Coordinator** managing specialist agents via direct communication

The Orchestrator acts as a **Project Manager** coordinating real-time with specialist agents via ACP while presenting unified progress to users.

### Core Principles

1. **Dual Interface Role**: Orchestrator serves as single point to Claude Code REPL terminal while coordinating ACP network
2. **Real-time ACP Coordination**: Direct communication with specialist agents via ACP REST API
3. **Incremental Planning**: Plan phases based on real-time agent feedback and discoveries
4. **Phase Gates**: Coordinate phase completion via ACP before planning next phase
5. **Unified Progress Reporting**: Present clean, coordinated updates to Claude Code users
6. **Hidden Complexity**: ACP network coordination invisible to users

### ACP-Based Orchestration Flow

```
Claude Code ↔ MCP Server ↔ Orchestrator (ACP Agent)
                              ↕ ACP Network
                          Specialist Agents
```

**1. Session Initialization:**
- Orchestrator spawned as ACP agent with dual role
- Connects to MCP server for Claude Code interface
- Joins ACP network for agent coordination

**2. Adaptive Phase Execution:**
- **Agent Discovery**: Find available specialist agents via ACP
- **Task Assignment**: Send phase tasks to agents via ACP messages
- **Real-time Coordination**: Monitor agent progress via ACP status updates
- **User Updates**: Report unified progress to Claude Code REPL Terminal

**3. Phase Gate Coordination:**
- **Collect Results**: Gather phase outputs from agents via ACP
- **Review and Validate**: Analyze completeness and quality
- **Plan Next Phase**: Determine next steps based on discoveries
- **Report to User**: Present phase completion to Claude Code REPL Terminal

**4. Iterative Discovery:**
- Continue phase-by-phase until objectives met
- Adapt plan based on real-time agent feedback
- Maintain unified user experience throughout

### ACP Coordination Patterns

**Agent Task Assignment (Orchestrator → Agent):**
```json
{
  "type": "phase_assignment",
  "phase": "research",
  "task": "Analyze authentication requirements",
  "deliverables": ["requirements_doc", "constraints_analysis"],
  "deadline": "2025-07-14T18:00:00Z"
}
```

**Agent Status Update (Agent → Orchestrator):**
```json
{
  "type": "phase_progress",
  "phase": "research", 
  "status": "completed",
  "deliverables": ["auth_requirements.md", "security_constraints.md"],
  "insights": ["OAuth2 preferred", "MFA required"]
}
```

**Phase Completion Coordination:**
- Orchestrator collects all agent deliverables
- Reviews phase outputs for completeness
- Plans next phase based on discoveries
- Reports unified progress to Claude Code

### Agent Lifecycle Patterns

MAOS supports **two complementary agent lifecycle patterns** for maximum efficiency:

**Micro-Task Pattern** (High-Efficiency Focused Work):
```
Orchestrator → Spawn Agent → Focused Work (Isolation) → ACP Result → Terminate
             ↑                                        ↑
        (single task)                            (immediate completion)
```
- **Best for**: Independent, atomic tasks (generate test, implement function, review code)
- **Benefits**: Maximum focus, no interruptions, immediate resource cleanup
- **Communication**: Minimal - only task assignment and completion status
- **Lifecycle**: Spawn → Work → Report → Terminate

**Phase-Based Pattern** (Team Coordination):
```
Orchestrator → Spawn Agent → Phase Work → ACP Progress → Next Phase/Terminate
             ↑                          ↑
        (phase assignment)        (minimal coordination)
```
- **Best for**: Complex, multi-step work requiring coordination
- **Benefits**: Context retention, adaptive planning, team collaboration
- **Communication**: Essential only - phase assignments, completions, critical handoffs
- **Lifecycle**: Spawn → Multiple tasks → Phase complete → Next phase or terminate

**Pattern Selection Criteria:**
- **Micro-Task**: When work is independent and can be completed in isolation
- **Phase-Based**: When work requires coordination, context, or multiple steps
- **Hybrid**: Mix both patterns within same orchestration as needed

### Agent Pool Coordination with Session Binding

**Revolutionary Context-Aware Orchestration:**

The Orchestrator manages a **persistent Agent Resources Registry** that enables context continuity across the entire orchestration lifecycle:

**Agent Pool Management ACP Messages:**

**Agent Registration (Agent → Orchestrator):**
```json
{
  "type": "agent_registration",
  "from": "frontend_engineer_1",
  "to": "orchestrator",
  "timestamp": "2025-07-14T10:30:00Z",
  "content": {
    "agent_id": "frontend_engineer_1",
    "role": "frontend_engineer", 
    "claude_session_id": "session_ghi789",
    "capabilities": ["react", "typescript", "testing"],
    "status": "ready",
    "initial_spawn": true
  }
}
```

**Agent Sleep Request (Agent → Orchestrator):**
```json
{
  "type": "agent_sleep_request",
  "from": "backend_engineer_1",
  "to": "orchestrator",
  "timestamp": "2025-07-14T15:45:00Z",
  "content": {
    "agent_id": "backend_engineer_1",
    "phase_completed": "api_implementation",
    "deliverables": ["user_api.rs", "auth_service.rs"],
    "ready_for_sleep": true,
    "session_preserved": true
  }
}
```

**Selective Agent Reactivation (Orchestrator → Agent Pool):**
```json
{
  "type": "agent_reactivation",
  "from": "orchestrator",
  "to": "frontend_engineer_1",
  "timestamp": "2025-07-14T16:00:00Z",
  "content": {
    "agent_id": "frontend_engineer_1",
    "claude_session_id": "session_ghi789",
    "new_phase": "ui_integration",
    "context_continuity": true,
    "previous_work_reference": "phase_2_frontend_components",
    "new_tasks": ["integrate_auth_components", "add_error_handling"]
  }
}
```

**Orchestrator Pool Management Patterns:**

1. **Initial Pool Creation**: Spawn agents for orchestration and build registry
2. **Context Tracking**: Monitor which agents worked on which phases/components
3. **Strategic Sleep**: Put agents to sleep when phase work complete
4. **Selective Reactivation**: Choose specific agents with relevant context for new phases
5. **Memory Continuity**: Agents resume with full knowledge of previous work
6. **Resource Optimization**: Only active agents consume system resources

**Pool Coordination Benefits:**
- **Perfect Context Continuity**: frontend_engineer_1 remembers all UI work from previous phases
- **Intelligent Agent Selection**: Reactivate the architect who designed specific components
- **Resource Efficiency**: Sleep unused agents while preserving their context
- **Seamless Collaboration**: Agents can reference and build upon each other's previous work

## Consequences

### Positive
- **Perfect Context Continuity**: Agents never lose memory between activations (revolutionary!)
- **Intelligent Resource Management**: Agents sleep when idle, conserving system resources
- **Selective Expert Reactivation**: Choose specific agents with relevant context for new phases
- **Maximum Focus & Productivity**: Agents work in isolation without interruptions (proven efficient)
- **Minimal Communication Overhead**: ACP used only when essential, not for chatter
- **Dual Lifecycle Support**: Both micro-task (high-efficiency) and phase-based patterns
- **Unified User Experience**: Single, clean interface via Orchestrator to Claude Code
- **Agent Pool Coordination**: Orchestrator manages persistent specialist agent resources
- **Essential-Only Coordination**: Communication triggers limited to critical needs
- **Better Alignment**: Each phase builds on actual agent outputs via ACP feedback
- **Reduced Waste**: No planning for features that research agents show aren't needed
- **Higher Quality**: Specialist agents work from concrete specifications with full context
- **Hidden Complexity**: ACP network coordination invisible to users
- **Adaptive Planning**: Real-time agent feedback enables dynamic plan adjustment
- **Seamless Collaboration**: Agents reference and build upon each other's previous work
- **Performance**: Direct ACP communication vs. complex file-based coordination
- **Memory Persistence**: Claude Code session binding ensures no context loss

### Negative
- **Orchestrator Complexity**: Dual role as both user interface and ACP coordinator
- **ACP Dependency**: Requires reliable ACP network for coordination
- **Network Overhead**: HTTP-based ACP communication adds latency
- **Debugging Complexity**: ACP network coordination hidden from users
- **More Orchestrator Invocations**: Orchestrator runs between each phase
- **Complex State Management**: Must track partial plans and phase outputs
- **Harder Estimation**: Can't predict total phases until project progresses

### Mitigation
- **ACP Health Monitoring**: Robust monitoring of ACP network for reliability
- **Orchestrator Training**: Clear prompts about dual interface/coordinator role
- **Phase Review Mechanisms**: Strong ACP-based phase completion validation
- **Common Patterns**: Templates for typical phase sequences and ACP coordination
- **Debug Tools**: ACP network activity monitoring for troubleshooting
- **Graceful Degradation**: Fallback mechanisms when ACP coordination fails

## Implementation Notes

### Orchestrator Dual Role Training

The Orchestrator prompt should emphasize the **dual interface/coordinator role**:

**As Claude Code Interface:**
- "You are the ONLY voice users hear - represent the entire team professionally"
- "Present unified, clear progress updates to Claude Code users"
- "Hide ACP network complexity - users don't need to see agent coordination"
- "Report phase completions and discoveries in user-friendly language"

**As ACP Network Coordinator:**
- "Coordinate with specialist agents via ACP messages for real-time collaboration"
- "Assign specific, concrete tasks to agents based on their expertise"
- "Monitor agent progress via ACP status updates"
- "Plan only 1-2 phases ahead based on actual agent outputs, not assumptions"
- "Each phase should produce concrete deliverables from specialist agents"

**Project Management Principles:**
- "You are a Project Manager coordinating specialists, not a fortune teller"
- "Review actual agent deliverables before planning next steps"
- "Adapt the plan based on discoveries from specialist agents"

## References
- **ADR-04: ACP-Based Agent Communication** - Defines ACP coordination patterns used by Orchestrator
- **ADR-08: Agent Lifecycle and Management** - ACP server spawning for specialist agents
- **ADR-10: MCP Server Architecture** - Orchestrator-only interface to Claude Code
- [Agent Communication Protocol (ACP)](https://agentcommunicationprotocol.dev/) - Core agent coordination protocol
- POC analysis showing Orchestrator planning failures
- Agile methodologies and iterative development
- Project management best practices
- Feedback control systems
- Adaptive project management patterns

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*