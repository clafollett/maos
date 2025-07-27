# PRD-03: Specialized Agent Implementation
## Product Requirements Document (PRD)

**Version:** 2.0 (Updated for Claude Code native sub-agents)  
**Date:** 2025-01-26  
**Author:** MAOS Development Team  

---

## Executive Summary

This PRD documents the specialized AI agents implemented for MAOS using Claude Code's native sub-agent feature. The system provides 22 specialized agents covering all aspects of software development, from architecture to deployment.

**Key Value Propositions:**
- **Native Integration**: Leverages Claude Code's built-in sub-agent capabilities
- **Specialized Expertise**: Each agent has deep domain knowledge
- **Zero Infrastructure**: No complex orchestration code needed
- **Immediate Value**: Agents work out-of-the-box with Claude Code
- **Project Adaptive**: Agents adapt to existing project patterns

## Agent Categories

### 1. Architecture & Design (5 agents)
- **application-architect**: Application-level architecture and patterns
- **solution-architect**: Enterprise solution design
- **api-architect**: REST/GraphQL API design
- **data-architect**: Database and data flow design
- **security-architect**: Security architecture and threat modeling

### 2. Engineering (5 agents)
- **backend-engineer**: Server-side development
- **frontend-engineer**: UI/UX implementation
- **mobile-engineer**: iOS/Android development
- **devops-engineer**: Infrastructure and deployment
- **secops-engineer**: Security operations

### 3. Quality & Testing (3 agents)
- **qa-engineer**: Comprehensive quality assurance
- **tester**: Focused testing execution
- **code-reviewer**: Code quality and standards

### 4. Product & Business (3 agents)
- **product-manager**: Product strategy and requirements
- **business-analyst**: Business process analysis
- **ux-designer**: User experience design

### 5. Research & Documentation (4 agents)
- **researcher**: Technical research and analysis
- **tech-writer**: Technical documentation
- **adr-specialist**: Architecture Decision Records
- **prd-specialist**: Product Requirements Documents

### 6. Data & Coordination (2 agents)
- **data-scientist**: Data analysis and ML
- **orchestrator**: Multi-agent coordination

## Implementation Details

### Agent Structure
Each agent is defined as a Markdown file in `.claude/agents/` with:
- **YAML frontmatter**: Name, description, and allowed tools
- **Role definition**: Clear responsibilities and expertise
- **Project integration**: Adaptive patterns for different codebases
- **Best practices**: Domain-specific guidelines

### Example Agent Definition
```yaml
---
name: backend-engineer
description: Implements server-side applications, APIs, and business logic
tools: Task, Read, Write, Edit, MultiEdit, Bash, Grep, Glob, LS, WebSearch, TodoWrite
---
```

### Key Features

1. **Project Adaptation**
   - Agents discover existing patterns
   - Honor established conventions
   - Support multiple languages/frameworks

2. **Tool Access Control**
   - Each agent has specific tools
   - Security through capability limiting
   - Task tool enables delegation

3. **Communication Patterns**
   - File-based context sharing
   - Direct agent-to-agent via Task tool
   - Session continuity for context

## Usage Patterns

### Direct Invocation
```bash
claude --sub-agent backend-engineer
```

### Task Tool Delegation
```markdown
I'll use the backend-engineer sub-agent to implement the API
```

### Multi-Agent Workflows
```markdown
1. Architect designs the system
2. Engineers implement components
3. QA validates functionality
4. Tech writer documents
```

## Benefits

### For Developers
- Instant access to specialized expertise
- No setup or configuration needed
- Consistent quality across domains
- Reduced cognitive load

### For Teams
- Parallel work on different aspects
- Consistent coding standards
- Comprehensive documentation
- Built-in quality checks

### For Organizations
- Standardized development practices
- Reduced onboarding time
- Improved code quality
- Better knowledge retention

## Future Enhancements

1. **Custom Agent Templates**: Organization-specific agents
2. **Agent Specialization**: Industry or framework-specific variants
3. **Learning Patterns**: Agents that adapt to team preferences
4. **Metrics & Analytics**: Track agent usage and effectiveness

## Conclusion

MAOS's specialized agents provide immediate value through Claude Code's native features, eliminating the need for complex infrastructure while delivering professional-grade AI assistance across all aspects of software development.