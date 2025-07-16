# ADR-11: Adaptive Phase-Based Orchestration

## Status
Accepted

## Context
The POC revealed critical insights about orchestration that shaped our PTY multiplexer architecture:

### Original Issues:
- **Central Planning Flaw**: Orchestrator attempting to plan entire project upfront
- **Communication Gaps**: File-based messaging created coordination failures
- **Knowledge Silos**: Engineers working without architectural context
- **Misalignment**: Phases disconnected from each other

### PTY Multiplexer Solution:
With our **Orchestrator-as-PTY-Multiplexer** pattern and **single interface** design, we now have:
- **Real-time coordination**: Direct PTY control and message routing via Orchestrator
- **Single interface**: Orchestrator as sole representative to Claude Code
- **Adaptive planning**: Orchestrator coordinates with agents via PTY multiplexer for incremental discovery
- **Configurable transparency**: PTY processes can be visible for debugging or hidden for clean UX

Traditional orchestration assumes perfect upfront knowledge. Our PTY multiplexer approach enables **adaptive discovery** where the Orchestrator coordinates real-time with specialist agents via PTY control while presenting unified progress to Claude Code.

## Decision
We will implement a **PTY multiplexer-based adaptive orchestration model** where the Orchestrator operates as both:
1. **Single Interface** to Claude Code (via MCP)
2. **PTY Multiplexer** managing specialist agents via direct process control

The Orchestrator acts as a **Project Manager** coordinating real-time with specialist agents via PTY control while presenting unified progress to users.

### Core Principles

1. **Dual Interface Role**: Orchestrator serves as single point to Claude Code while controlling PTY multiplexer
2. **Real-time PTY Coordination**: Direct communication with specialist agents via PTY read/write operations
3. **Incremental Planning**: Plan phases based on real-time agent feedback and discoveries
4. **Phase Gates**: Coordinate phase completion via PTY messages before planning next phase
5. **Unified Progress Reporting**: Present clean, coordinated updates to Claude Code users
6. **Configurable Transparency**: PTY processes can be visible for debugging or hidden for clean UX

### PTY Multiplexer Orchestration Flow

```
Claude Code ↔ MCP Server ↔ Orchestrator (PTY Multiplexer)
                              ↕ PTY Control
                          Specialist Agents
```

**1. Session Initialization:**
- Orchestrator spawned as single Claude CLI process with PTY multiplexer role
- Connects to MCP server for Claude Code interface
- Manages PTY pairs for agent process control

**2. Adaptive Phase Execution:**
- **Agent Spawning**: Create specialist agents as Claude CLI processes via PTY
- **Task Assignment**: Send phase tasks to agents via PTY write operations
- **Real-time Coordination**: Monitor agent progress via PTY read operations
- **User Updates**: Report unified progress to Claude Code REPL Terminal

**3. Phase Gate Coordination:**
- **Collect Results**: Gather phase outputs from agents via PTY communication
- **Review and Validate**: Analyze completeness and quality
- **Plan Next Phase**: Determine next steps based on discoveries
- **Report to User**: Present phase completion to Claude Code

**4. Iterative Discovery:**
- Continue phase-by-phase until objectives met
- Adapt plan based on real-time agent feedback
- Maintain unified user experience throughout

### PTY Coordination Patterns

**Agent Task Assignment (Orchestrator writes to Agent via PTY):**
```
TASK ASSIGNMENT from Orchestrator:
Objective: Analyze authentication requirements for the application
Context: Building secure user authentication system
Deliverables: requirements_doc, constraints_analysis
Success Criteria: Complete analysis with implementation recommendations
```

**Agent Status Update (Orchestrator reads from Agent via PTY):**
```
STATUS UPDATE to Orchestrator:
Progress: Authentication requirements analysis completed
Completed: auth_requirements.md, security_constraints.md created
Next: Ready for implementation phase
Insights: OAuth2 preferred, MFA required for compliance
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
Orchestrator → Spawn Agent → Focused Work (Isolation) → PTY Result → Terminate
             ↑                                        ↑
        (single task)                            (immediate completion)
```
- **Best for**: Independent, atomic tasks (generate test, implement function, review code)
- **Benefits**: Maximum focus, no interruptions, immediate resource cleanup
- **Communication**: Minimal - only task assignment and completion status
- **Lifecycle**: Spawn → Work → Report → Terminate

**Phase-Based Pattern** (Team Coordination):
```
Orchestrator → Spawn Agent → Phase Work → PTY Progress → Next Phase/Terminate
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

**Context-Aware Orchestration:**

The Orchestrator manages a **persistent Agent Resources Registry** that enables context continuity across the entire orchestration lifecycle:

**Agent Pool Management PTY Messages:**

**Agent Registration (Orchestrator reads from Agent via PTY):**
```
AGENT REGISTRATION to Orchestrator:
Agent ID: frontend_engineer_1
Role: frontend_engineer
Claude Session ID: session_ghi789
Capabilities: react, typescript, testing
Status: ready for tasks
Initial spawn completed
```

**Agent Sleep Request (Orchestrator reads from Agent via PTY):**
```
SLEEP REQUEST to Orchestrator:
Agent ID: backend_engineer_1
Phase Completed: api_implementation
Deliverables: user_api.rs, auth_service.rs
Ready for sleep: true
Session preserved for future reactivation
```

**Selective Agent Reactivation (Orchestrator writes to Agent via PTY):**
```
REACTIVATION from Orchestrator:
Agent ID: frontend_engineer_1
Claude Session ID: session_ghi789
New Phase: ui_integration
Context: You previously worked on frontend components
New Tasks: integrate_auth_components, add_error_handling
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
- **Perfect Context Continuity**: Agents never lose memory between activations
- **Intelligent Resource Management**: Agents sleep when idle, conserving system resources
- **Selective Expert Reactivation**: Choose specific agents with relevant context for new phases
- **Maximum Focus & Productivity**: Agents work in isolation without interruptions (proven efficient)
- **Minimal Communication Overhead**: PTY used only when essential, not for chatter
- **Dual Lifecycle Support**: Both micro-task (high-efficiency) and phase-based patterns
- **Unified User Experience**: Single, clean interface via Orchestrator to Claude Code
- **Agent Pool Coordination**: Orchestrator manages persistent specialist agent resources
- **Essential-Only Coordination**: Communication triggers limited to critical needs
- **Better Alignment**: Each phase builds on actual agent outputs via PTY feedback
- **Reduced Waste**: No planning for features that research agents show aren't needed
- **Higher Quality**: Specialist agents work from concrete specifications with full context
- **Configurable Transparency**: PTY processes can be hidden for clean UX or visible for debugging/learning
- **Adaptive Planning**: Real-time agent feedback enables dynamic plan adjustment
- **Seamless Collaboration**: Agents reference and build upon each other's previous work
- **Performance**: Direct PTY communication vs. complex file-based coordination
- **Memory Persistence**: Claude Code session binding ensures no context loss

### Negative
- **Orchestrator Complexity**: Dual role as both user interface and PTY multiplexer
- **PTY Dependency**: Requires portable-pty for cross-platform process control
- **Process Management Overhead**: Managing multiple Claude CLI processes via PTY
- **Debugging Complexity**: PTY process coordination hidden from users
- **More Orchestrator Invocations**: Orchestrator runs between each phase
- **Complex State Management**: Must track partial plans and phase outputs
- **Harder Estimation**: Can't predict total phases until project progresses

### Mitigation
- **PTY Health Monitoring**: Robust monitoring of PTY processes for reliability
- **Orchestrator Training**: Clear prompts about dual interface/coordinator role
- **Phase Review Mechanisms**: Strong PTY-based phase completion validation
- **Common Patterns**: Templates for typical phase sequences and PTY coordination
- **Debug Tools**: PTY process activity monitoring for troubleshooting
- **Graceful Degradation**: Fallback mechanisms when PTY coordination fails

## Implementation Notes

### Orchestrator Dual Role Training

The Orchestrator prompt should emphasize the **dual interface/coordinator role**:

**As Claude Code Interface:**
- "You are the ONLY voice users hear - represent the entire team professionally"
- "Present unified, clear progress updates to Claude Code users"
- "Hide PTY process complexity - users don't need to see agent coordination"
- "Report phase completions and discoveries in user-friendly language"

**As PTY Multiplexer Coordinator:**
- "Coordinate with specialist agents via PTY messages for real-time collaboration"
- "Assign specific, concrete tasks to agents based on their expertise"
- "Monitor agent progress via PTY status updates"
- "Plan only 1-2 phases ahead based on actual agent outputs, not assumptions"
- "Each phase should produce concrete deliverables from specialist agents"

**Project Management Principles:**
- "You are a Project Manager coordinating specialists, not a fortune teller"
- "Review actual agent deliverables before planning next steps"
- "Adapt the plan based on discoveries from specialist agents"

## References
- **ADR-04: Orchestrator-as-PTY-Multiplexer Communication** - Defines PTY coordination patterns used by Orchestrator
- **ADR-08: Agent Lifecycle and PTY Multiplexer Management** - PTY process spawning for specialist agents
- **ADR-10: MCP Server Architecture** - Orchestrator-only interface to Claude Code
- [Tmux-Orchestrator](https://github.com/Jedward23/Tmux-Orchestrator) - Inspiration for PTY multiplexer patterns
- [portable-pty](https://docs.rs/portable-pty/) - Cross-platform PTY implementation
- POC analysis showing Orchestrator planning failures
- Agile methodologies and iterative development
- Project management best practices
- Feedback control systems
- Adaptive project management patterns

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*