# ADR-07: Orchestration Guardrails and Coordination Protocols

## Status
Accepted

## Context
The POC revealed critical coordination failures in our multi-agent orchestration system. The Orchestrator created vague task assignments that led to misalignment between architects and engineers, resulting in architectural decisions being ignored during implementation.

### Key Issues Identified

1. **Communication Breakdown**: Engineers never referenced architectural documentation
2. **Vague Task Assignments**: Generic tasks like "Build components" without specific requirements
3. **Missing Coordination**: No validation between design and implementation phases
4. **Parallel Execution Risks**: Multiple agents building incompatible systems simultaneously
5. **Context Loss**: Important decisions made in one phase were lost in subsequent phases

These failures led to architects designing comprehensive systems while engineers built basic prototypes that ignored architectural decisions entirely.

## Decision
We will implement a comprehensive orchestration guardrail system with mandatory coordination phases, enhanced task specifications, and validation gates to ensure proper alignment between architectural decisions and implementation.

### Core Principles

1. **Mandatory Document Reading**: All agents must read and acknowledge relevant architectural documents before starting work
2. **Specific Task Requirements**: Every task must include specific architectural constraints, API contracts, and UI specifications
3. **Phase-Gate Validation**: Implementation phases cannot proceed without validation against architectural decisions
4. **Coordination Phases**: Explicit coordination phases between design and implementation to ensure alignment
5. **Compliance Verification**: Specialized agents verify implementation compliance with architectural decisions

### Architectural Layering

This ADR builds on ADR-04 (Agent Communication Patterns) by adding higher-level coordination protocols:

- **ADR-04 provides**: Technical communication infrastructure (message routing, agent discovery, file-based messaging)
- **ADR-07 adds**: Coordination policies and quality gates that USE ADR-04's infrastructure
- **Relationship**: ADR-07 implements orchestration protocols on top of ADR-04's communication foundation

### Orchestration Enhancements

#### 1. Enhanced Task Specifications
All engineering tasks must include:
- Specific architectural documents to follow
- UI/UX specifications with exact requirements
- API contracts and integration points
- Technology stack constraints
- Validation criteria for completion

#### 2. Mandatory Coordination Phases
- **Project Setup Phase**: Between architecture and implementation
- **Compliance Review Phase**: Verification that implementation matches design
- **Integration Validation Phase**: Ensuring all components work together

#### 3. Agent Requirements Framework
Building on ADR-04's agent discovery and messaging infrastructure, every agent assignment includes enhanced metadata:
- `must_read`: Required documents before starting (validated via message acknowledgment)
- `validation_criteria`: How work will be evaluated (communicated via structured messages)
- `coordination_spec`: What other agents depend on this work (tracked through ADR-04's dependency system)

#### 4. Inter-Phase Knowledge Transfer Pattern
To prevent information overload and context loss between phases, we implement systematic knowledge transfer using specialized Summarizer agents:

**Phase Output Summarization**:
Each phase concludes with a Summarizer agent that creates targeted summaries:
- **Decisions Summary**: Key architectural/technical decisions made
- **Requirements Summary**: Constraints and requirements for next phases  
- **Context Summary**: Background information needed to understand decisions
- **Validation Summary**: Criteria for verifying compliance with phase outputs

**Structured Summary Format**:
```markdown
# Phase Summary: [Phase Name]

## Key Decisions
- [Architectural/technical decisions with rationale]

## Requirements for Next Phase
- [Specific constraints and requirements]

## Context & Background
- [Essential background for understanding decisions]

## Validation Criteria
- [How compliance can be verified]

## Reference Documents
- [Links to detailed documentation]
```

**Targeted Information Filtering**:
Summarizer agents filter information based on the receiving phase's role to prevent context overload while preserving critical decisions.

#### 5. Validation Gates
Before proceeding between phases:
- Architecture compliance verification
- Integration testing requirements
- Requirements coverage validation

## Consequences

### Positive
- **Reduced Misalignment**: Explicit coordination prevents architectural decisions from being ignored
- **Better Quality**: Validation gates catch issues before they compound
- **Clear Accountability**: Each agent has specific requirements and validation criteria
- **Improved Communication**: Standardized handoff formats ensure information transfer
- **Risk Mitigation**: Multiple checkpoints prevent major course corrections later

### Negative
- **Increased Complexity**: More coordination overhead in orchestration planning
- **Longer Execution Time**: Additional phases and validation steps
- **Higher Token Usage**: More detailed task specifications and documentation requirements
- **Potential Bottlenecks**: Validation gates may slow parallel execution

### Mitigation
- Implement efficient validation patterns to minimize overhead
- Use summarizer agents to maintain context without overwhelming detail
- Create templates for common coordination patterns
- Balance thoroughness with execution speed based on project complexity

## Implementation Notes

### Orchestrator Enhancements
The Orchestrator must be enhanced to:
- Generate specific task requirements referencing concrete documents
- Insert coordination phases automatically based on project structure  
- Create validation checkpoints between major phases
- Track compliance across agent interactions using ADR-04's message routing

### Agent Prompt Enhancements
Agent prompts must emphasize:
- Reading required documents before starting work (acknowledge via ADR-04 messaging)
- Validating work against specific criteria (report status via ADR-04 status updates)
- Communicating decisions clearly for other agents (use ADR-04's structured message formats)

### Integration with Communication Infrastructure
This ADR leverages ADR-04's infrastructure in specific ways:
- **Validation Gates**: Use ADR-04's message acknowledgment patterns to verify agents have read required documents
- **Coordination Phases**: Use ADR-04's broadcast patterns to announce phase transitions
- **Summarizer Agents**: Use ADR-04's shared context directory to publish phase summaries
- **Compliance Tracking**: Use ADR-04's status update patterns to monitor validation progress

## References
- **ADR-04: Agent Communication Patterns** - Provides the technical communication infrastructure that this ADR builds upon
- ADR-03: Session Orchestration and State Management - Session-level coordination integration
- ADR-08: Agent Lifecycle and Management - Process-level coordination hooks
- ADR-11: Adaptive Phase-Based Orchestration - Phase-based planning approach
- POC failure analysis documenting coordination breakdowns
- Software quality assurance methodologies
- Enterprise coordination patterns

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*