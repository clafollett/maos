# ADR-02: Adaptive Phase-Based Orchestration

## Status
Accepted (Updated for Claude Code native sub-agents)

## Context
Complex projects require coordination between multiple specialized agents. Traditional orchestration assumes perfect upfront knowledge, but real projects involve discovery, adaptation, and iteration. Claude Code's native sub-agent feature provides the foundation for adaptive orchestration.

### Key Insights:
- **Adaptive Discovery**: Plans evolve based on what agents discover
- **Phase-Based Execution**: Break large projects into manageable phases
- **Specialist Coordination**: Each agent brings domain expertise
- **Emergent Solutions**: Best approaches emerge through collaboration

## Decision
We will implement an **adaptive orchestration model** using Claude Code's native features:

1. **Native Sub-Agents**: Claude Code handles agent spawning and communication
2. **Orchestrator Agent**: Acts as Project Manager coordinating specialists
3. **Phase-Based Workflow**: Iterative discovery and implementation
4. **File-Based Context**: Shared directories for agent collaboration

### Core Principles

1. **Incremental Planning**: Plan phases based on discoveries
2. **Specialist Delegation**: Leverage domain expertise of each agent
3. **Context Preservation**: Use Claude's session continuity
4. **Adaptive Re-planning**: Adjust based on findings
5. **Clear Communication**: Structured updates between agents

### Orchestration Flow

```
User Request → Claude → Orchestrator Agent
                            │
                ┌───────────┼───────────┐
                ▼           ▼           ▼
            Architect   Engineers    Testers
            (planning)  (building)  (validating)
```

### Phase-Based Execution

**Phase 1: Discovery & Planning**
- Architect analyzes requirements
- Researchers gather information
- Product Manager refines scope

**Phase 2: Design & Architecture**
- Solution Architect creates system design
- API Architect defines interfaces
- Security Architect reviews approach

**Phase 3: Implementation**
- Backend/Frontend Engineers build features
- DevOps sets up infrastructure
- Mobile Engineers create apps

**Phase 4: Validation & Integration**
- QA Engineers test functionality
- Security tests vulnerabilities
- Performance optimization

**Phase 5: Documentation & Deployment**
- Tech Writers create documentation
- DevOps handles deployment
- Product Manager validates delivery

### Adaptive Patterns

**1. Discovery-Driven Planning**
```markdown
Based on the architect's analysis, we need:
- GraphQL API (not REST as initially thought)
- Real-time subscriptions for chat features
- Microservices for scalability
```

**2. Cross-Agent Learning**
```markdown
Frontend engineer discovered the API needs pagination.
Backend engineer, please add cursor-based pagination to all list endpoints.
```

**3. Iterative Refinement**
```markdown
QA found performance issues with the current approach.
Let's have the backend engineer optimize the database queries.
```

### Communication Patterns

Agents communicate through:
1. **Direct Messages**: Via Claude's Task tool
2. **Shared Files**: `.maos/shared/` directory
3. **Status Updates**: Structured progress reports
4. **Context Propagation**: Session continuity

## Consequences

### Positive
- Flexible and adaptive to discoveries
- Leverages specialist expertise
- No complex infrastructure needed
- Works with Claude Code's native features
- Clear phase-based progress

### Negative
- Requires orchestrator agent overhead
- May have longer execution time
- Depends on agent communication quality

## Implementation

The orchestrator agent uses patterns like:

```markdown
I'll coordinate this feature development across multiple phases:

Phase 1: Architecture Design
- Spawning architect to design the system...

Phase 2: Implementation
- Based on the architecture, spawning engineers...

Phase 3: Testing
- Now spawning QA to validate the implementation...
```

## References
- Claude Code sub-agent documentation
- Phase-based project management
- Adaptive planning methodologies