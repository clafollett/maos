# ADR-12: Adaptive Phase-Based Orchestration

## Status
Accepted (Updated for PTY architecture)

## Context
The POC revealed critical insights about orchestration that shaped our architecture:

### Original Issues:
- **Central Planning Flaw**: Orchestrator attempting to plan entire project upfront
- **Communication Gaps**: File-based messaging created coordination failures
- **Knowledge Silos**: Engineers working without architectural context
- **Misalignment**: Phases disconnected from each other

### PTY-Based Solution:
With our **PTY multiplexer architecture** and **hub-and-spoke pattern**, we now have:
- **Centralized coordination**: PTY multiplexer manages all agent I/O directly
- **Single interface**: Orchestrator agent coordinates through the multiplexer
- **Adaptive planning**: Orchestrator plans phases based on actual agent outputs
- **Clean abstraction**: Terminal complexity hidden from users

Traditional orchestration assumes perfect upfront knowledge. Our approach enables **adaptive discovery** where the Orchestrator coordinates with specialist agents while the PTY multiplexer handles all communication.

## Decision
We will implement an **adaptive orchestration model** where:
1. **PTY Multiplexer** handles all agent process management and I/O
2. **Orchestrator Agent** acts as Project Manager coordinating through the multiplexer
3. **Hub-and-Spoke Pattern** ensures all messages route through central control

The Orchestrator acts as a **Project Manager** coordinating with specialist agents while presenting unified progress to users.

### Core Principles

1. **PTY-Based Coordination**: All agent communication flows through PTY multiplexer
2. **Message Routing**: Hub-and-spoke pattern with 500ms timing for Claude UI
3. **Incremental Planning**: Plan phases based on actual agent outputs and discoveries
4. **Phase Gates**: Aggregate phase results before planning next phase
5. **Unified Progress Reporting**: Present clean, coordinated updates to users
6. **Clean Abstraction**: PTY and process complexity hidden from users

### PTY-Based Orchestration Flow

```
User → MAOS CLI → PTY Multiplexer
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
    Orchestrator  Backend    Frontend
    (coordinator) (worker)    (worker)
```

**1. Session Initialization:**
- User runs `maos orchestrate "build feature X"`
- PTY multiplexer spawns orchestrator agent
- Orchestrator receives task and begins planning

**2. Adaptive Phase Execution:**
- **Agent Planning**: Orchestrator determines which agents needed
- **Task Assignment**: Messages routed through PTY multiplexer
- **Progress Monitoring**: Real-time output capture from PTYs
- **User Updates**: Streaming output to user terminal

**3. Phase Gate Coordination:**
- **Collect Results**: Orchestrator queries agents for deliverables
- **Review and Validate**: Analyze completeness via PTY messages
- **Plan Next Phase**: Determine next steps based on discoveries
- **Report to User**: Present phase completion updates

**4. Iterative Discovery:**
- Continue phase-by-phase until objectives met
- Adapt plan based on real-time agent feedback
- Maintain unified user experience throughout

### PTY Communication Patterns

**Agent Task Assignment (Orchestrator → Agent via PTY):**
```
[From Orchestrator]: Please analyze authentication requirements for our API.
Focus on:
- Security best practices
- Token management approach
- Session handling
- Integration with existing systems

Let me know when you've completed your analysis.
```

**Agent Status Update (Agent → Orchestrator via PTY):**
```
I've completed the authentication analysis. Key findings:

1. OAuth2 with JWT tokens recommended
2. Refresh token rotation for security
3. Redis for session management
4. Need MFA support for enterprise users

The detailed requirements are ready in auth_requirements.md.
```

**Phase Completion Coordination:**
- Orchestrator queries each agent for status
- PTY multiplexer captures all responses
- Orchestrator reviews outputs via messages
- Plans next phase based on discoveries

### Phase-Based Execution Pattern

MAOS uses a unified phase-based execution pattern where:

```
Orchestrator → Plan Phase → Execute Agent(s) → Collect Results → Next Phase/Complete
             ↑                                ↑
        (adaptive planning)            (individual agent results)
```

**Key Characteristics:**
- **Flexible Phase Size**: A phase can have one agent (simple task) or many (complex coordination)
- **Session Continuity**: Agents maintain context via session IDs when needed
- **Parallel/Sequential**: Orchestrator determines execution strategy per phase
- **Adaptive Planning**: Each phase planned based on previous results

**Examples:**
- **Single-Agent Phase**: "Generate unit tests" - one QA agent, isolated work
- **Multi-Agent Phase**: "Implement feature" - architect, backend, frontend agents working in parallel
- **Sequential Phase**: "Review and refactor" - reviewer agent followed by engineer agent

The beauty of this approach is its simplicity - everything is just a phase with N agents, where N can be 1 for simple tasks or many for complex coordination.

### Session Registry for Context Continuity

The PTY multiplexer maintains a **session registry** that tracks agents by role and Claude session IDs:

**Session Registry (maintained by PTY multiplexer):**

| pty_id  | claude_session_id | role          | status   | work_context    |
|---------|-------------------|---------------|----------|-----------------|
| pts/2   | session_abc123    | orchestrator  | active   | main_coordinator|
| pts/3   | session_def456    | backend       | active   | auth_service    |
| pts/4   | session_ghi789    | backend       | idle     | user_service    |
| pts/5   | session_jkl012    | frontend      | active   | auth_ui         |
| pts/6   | session_mno345    | frontend      | idle     | dashboard       |
| pts/7   | session_pqr678    | qa            | active   | api_tests       |

**Intelligent Agent Selection:**
The Orchestrator (running as a Claude agent itself) makes intelligent routing decisions:

1. **Task Analysis**: Orchestrator understands the task requirements
2. **Registry Query**: PTY multiplexer provides current agent status
3. **Smart Routing**: Orchestrator picks best agent or requests new spawn
4. **Considers Factors**:
   - Agent specialization and past work
   - Current agent availability (active/idle)
   - Context continuity via session IDs
   - Related component expertise
   - Workload distribution

**PTY Benefits:**
- **Direct Status**: Know immediately if agent is responsive
- **Session Persistence**: Claude --session-id preserves all context
- **Process Control**: Can restart crashed agents with same session
- **Real-time Monitoring**: See exactly what each agent is doing

**Benefits:**
- **Intelligent Reuse**: Orchestrator picks the right expert for each task
- **Component Specialization**: Different engineers can focus on different parts
- **Context Preservation**: Each agent maintains deep knowledge of their area
- **Flexible Scaling**: Can have multiple agents of same role working in parallel

## Consequences

### Positive
- **Context Continuity**: Agents maintain memory between activations via session IDs
- **Resource Management**: Agents consume resources only when active
- **Selective Reactivation**: Choose specific agents with relevant context for new phases
- **Focused Execution**: Agents work without interruptions
- **Minimal Communication**: Only essential coordination messages
- **Flexible Phases**: Support for single-agent to multi-agent phases
- **Unified Interface**: Single, clean interface via Orchestrator to Claude Code
- **Session Registry**: Orchestrator maintains simple role-to-session mappings
- **Efficient Coordination**: Communication limited to critical needs
- **Better Alignment**: Each phase builds on actual agent outputs
- **Reduced Waste**: No planning for features that aren't needed
- **Higher Quality**: Specialist agents work from concrete specifications
- **Clean Abstraction**: Implementation complexity hidden from users
- **Adaptive Planning**: Feedback enables dynamic plan adjustment
- **Collaborative Building**: Agents reference previous work
- **Performance**: Streamlined communication architecture
- **Session Persistence**: Claude's session binding preserves context

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

### Orchestrator Training

The Orchestrator prompt should emphasize the **interface and coordination role**:

**As Claude Code Interface:**
- "You are the ONLY voice users hear - represent the entire team professionally"
- "Present unified, clear progress updates to Claude Code users"
- "Hide implementation complexity - users don't need to see agent coordination"
- "Report phase completions and discoveries in user-friendly language"

**As Agent Coordinator:**
- "Coordinate with specialist agents through Claude Code Agent"
- "Assign specific, concrete tasks to agents based on their expertise"
- "Monitor agent progress and collect results"
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