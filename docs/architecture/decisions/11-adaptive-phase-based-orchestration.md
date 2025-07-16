# ADR-11: Adaptive Phase-Based Orchestration

## Status
Accepted

## Context
The POC revealed critical insights about orchestration that shaped our architecture:

### Original Issues:
- **Central Planning Flaw**: Orchestrator attempting to plan entire project upfront
- **Communication Gaps**: File-based messaging created coordination failures
- **Knowledge Silos**: Engineers working without architectural context
- **Misalignment**: Phases disconnected from each other

### ACP-Based Solution:
With our **Agent Communication Protocol (ACP) integration** and **Orchestrator-as-Interface** pattern, we now have:
- **Centralized coordination**: Orchestrator manages all agent interactions via Claude Code Agent
- **Single interface**: Orchestrator as sole representative to Claude Code
- **Adaptive planning**: Orchestrator plans phases based on actual agent outputs
- **Clean abstraction**: Implementation complexity hidden from users

Traditional orchestration assumes perfect upfront knowledge. Our approach enables **adaptive discovery** where the Orchestrator coordinates with specialist agents while presenting unified progress to Claude Code.

## Decision
We will implement an **adaptive orchestration model** where the Orchestrator operates as both:
1. **Single Interface** to Claude Code (via MCP)
2. **Router Agent** coordinating specialist agents through Claude Code Agent

The Orchestrator acts as a **Project Manager** coordinating with specialist agents while presenting unified progress to users.

### Core Principles

1. **Dual Interface Role**: Orchestrator serves as single point to Claude Code while coordinating agents
2. **Centralized Coordination**: All agent communication flows through Claude Code Agent
3. **Incremental Planning**: Plan phases based on actual agent outputs and discoveries
4. **Phase Gates**: Aggregate phase results before planning next phase
5. **Unified Progress Reporting**: Present clean, coordinated updates to Claude Code users
6. **Clean Abstraction**: Implementation complexity hidden from users

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
- **Agent Planning**: Determine which agents needed for phase
- **Task Assignment**: Send individual requests to Claude Code Agent per agent
- **Progress Monitoring**: Track agent execution and collect results
- **User Updates**: Report unified progress to Claude Code

**3. Phase Gate Coordination:**
- **Collect Results**: Gather outputs from all agents in phase
- **Review and Validate**: Analyze completeness and quality
- **Plan Next Phase**: Determine next steps based on discoveries
- **Report to User**: Present phase completion to Claude Code

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

The Orchestrator maintains a **session registry** that tracks agents by role and assigns ordinal IDs when multiple agents of the same role exist:

```
Session Registry:
┌──────────────────────────────────────────────────────────────────────┐
│ agent_id        │ session_id      │ role          │ work_context    │
├──────────────────────────────────────────────────────────────────────┤
│ architect_1     │ session_abc123  │ architect     │ api_design, ... │
│ backend_eng_1   │ session_def456  │ backend_eng   │ auth_service    │
│ backend_eng_2   │ session_ghi789  │ backend_eng   │ user_service    │
│ frontend_eng_1  │ session_jkl012  │ frontend_eng  │ auth_ui         │
│ frontend_eng_2  │ session_mno345  │ frontend_eng  │ dashboard       │
│ qa_1            │ session_pqr678  │ qa            │ api_tests       │
└──────────────────────────────────────────────────────────────────────┘
```

**Intelligent Agent Selection:**
The Orchestrator uses its own Claude session to make intelligent agent assignment decisions. When a new task needs to be assigned, the Orchestrator:

1. **Analyzes the Task**: Uses Claude to understand the task requirements and context
2. **Reviews Registry**: Provides Claude with the current session registry and work history
3. **Makes Smart Decision**: Claude recommends which existing agent to use or whether to create new
4. **Considers Factors**:
   - Semantic understanding of work relationships
   - Component dependencies and interactions
   - Agent expertise based on past work
   - Workload distribution across agents
   - Context continuity benefits

**Implementation Examples:**
- [session_registry_example.rs](../references/examples/session_registry_example.rs) - Basic session registry implementation
- [intelligent_agent_selection.rs](../references/examples/intelligent_agent_selection.rs) - How Orchestrator uses Claude for smart agent assignment decisions

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