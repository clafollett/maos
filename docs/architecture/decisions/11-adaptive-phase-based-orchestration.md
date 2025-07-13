# ADR-11: Adaptive Phase-Based Orchestration

## Status
Accepted

## Context
The POC revealed a critical flaw in our orchestration approach: the Orchestrator agent was attempting to plan the entire project upfront, before any research or discovery had been done. This led to:

- Vague, generic task assignments
- Engineers ignoring architectural decisions they never knew existed
- Misalignment between phases
- Wasted effort on assumptions that proved incorrect

Traditional orchestration assumes the orchestrator has perfect knowledge to create a complete plan. In reality, software projects are journeys of discovery where each phase reveals information needed for the next.

## Decision
We will implement an adaptive, phase-based orchestration model where the Orchestrator acts as a Project Manager rather than a central planner.

### Core Principles

1. **Incremental Planning**: The Orchestrator only plans one or two phases ahead based on current knowledge
2. **Phase Gates**: Each phase must complete and be reviewed before the next phase is planned
3. **Feedback Loops**: Outputs from each phase inform the planning of subsequent phases
4. **Dynamic Adaptation**: The project plan evolves based on discoveries and decisions made

### Orchestration Flow

1. **Initial Phase**: Orchestrator always starts with research/discovery
2. **Phase Review**: After each phase completes, Orchestrator reviews outputs
3. **Next Phase Planning**: Based on phase outputs, Orchestrator plans the next phase
4. **Adaptive Execution**: Continue until project objectives are met

### Phase Planning Strategy

- **Research Phase**: Always first, gather requirements and constraints
- **Architecture Phase**: Only planned after research reveals the problem space
- **Implementation Phases**: Planned based on architectural decisions
- **Validation Phases**: Inserted as needed based on complexity

## Consequences

### Positive
- **Better Alignment**: Each phase builds on actual outputs, not assumptions
- **Reduced Waste**: No planning for features that research shows aren't needed
- **Higher Quality**: Engineers work from concrete specifications, not vague ideas
- **Flexibility**: Can adapt to discoveries and changing requirements
- **Clear Dependencies**: Each phase explicitly depends on previous outputs

### Negative
- **Longer Planning Cycles**: Can't see the full plan upfront
- **More Orchestrator Invocations**: Orchestrator runs between each phase
- **Complex State Management**: Must track partial plans and phase outputs
- **Harder Estimation**: Can't predict total phases until project progresses

### Mitigation
- Implement strong phase review mechanisms
- Create patterns for common phase sequences  
- Build tools to visualize evolving project plans
- Set expectations about adaptive nature upfront

## Implementation Notes

The Orchestrator prompt should emphasize:
- "You are a Project Manager, not a fortune teller"
- "Plan only what you can see clearly"
- "Each phase should produce concrete outputs for the next"
- "Review actual outputs before planning next steps"

## References
- POC analysis showing orchestrator planning failures
- Agile methodologies and iterative development
- Project management best practices
- Feedback control systems

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*