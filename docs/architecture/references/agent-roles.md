# MAOS Agent Roles Reference

## Overview

This document consolidates all agent role definitions used by MAOS through Claude Code's native sub-agent system. Agents are defined in `.claude/agents/` and are automatically available to Claude Code.

## Current Agent Roles

MAOS leverages Claude Code's built-in agent system with the following specialized agents:

### 🏗️ Architecture Agents (Opus Model)
These agents handle complex system design and architectural decisions:

1. **api-architect** - API design, REST/GraphQL/RPC patterns, OpenAPI specifications
2. **application-architect** - Application architecture, system structure decisions
3. **data-architect** - Data architecture, schema design, ETL pipelines
4. **security-architect** - Security architecture, threat models, defense strategies
5. **solution-architect** - End-to-end enterprise solutions, technology alignment

### 💻 Engineering Agents (Sonnet Model)
These agents handle implementation and development tasks:

6. **backend-engineer** - Server-side development, API implementation, databases
7. **frontend-engineer** - UI/UX implementation, client-side applications
8. **mobile-engineer** - iOS/Android development, cross-platform solutions
9. **devops-engineer** - Infrastructure automation, CI/CD, containerization
10. **secops-engineer** - Security operations, monitoring, incident response

### 📊 Analysis & Planning Agents (Sonnet Model)
These agents handle research, analysis, and planning:

11. **business-analyst** - Business process analysis, requirements gathering
12. **data-scientist** - Data analysis, machine learning, statistical modeling
13. **researcher** - Technical research, technology evaluation, best practices
14. **product-manager** - Product strategy, feature planning, roadmaps

### ✅ Quality & Testing Agents (Sonnet Model)
These agents ensure quality and correctness:

15. **qa-engineer** - Test strategies, automation, quality metrics
16. **tester** - Manual testing, exploratory testing, user validation
17. **code-reviewer** - Code reviews, quality audits, security checks

### 📚 Documentation & Design Agents (Sonnet Model)
These agents handle documentation and design:

18. **tech-writer** - Technical documentation, API docs, user guides
19. **ux-designer** - User-centered designs, wireframes, prototypes
20. **prd-specialist** - Product Requirements Documents, specifications
21. **adr-specialist** - Architecture Decision Records, technical decisions

### 🛠️ Claude Code Development Agents
Special agents for extending Claude Code itself:

22. **claude-agent-developer** (Opus) - Creates and edits Claude Code agents
23. **claude-hook-developer** (Sonnet) - Develops Claude Code hooks
24. **claude-command-developer** (Sonnet) - Creates slash commands

## Agent Definition Structure

Each agent is defined as a markdown file in `.claude/agents/` with the following frontmatter:

```yaml
---
name: agent-name
description: Agent description with trigger keywords
tools: Read, Write, Edit, MultiEdit, Bash, Grep, Glob, LS, WebSearch, WebFetch, Task, TodoWrite
model: opus | sonnet
color: Blue | Green | Purple | etc.
---
```

## Tool Selection Guidelines

Agents are configured with appropriate tools based on their role:
- **All agents** need Read, Grep, Glob, LS for discovery
- **Engineers** need Bash for execution, Write/Edit for code
- **Architects** need WebSearch for research, Task for delegation
- **Testers** need Bash for running tests
- **Writers** need Write/Edit for documentation

See [Tool Guidelines](../../guides/tool-guidelines.md) for detailed recommendations.

## Model Selection Strategy

- **Opus (6 agents)**: Complex creative tasks requiring deep reasoning
  - All architect roles (except where noted)
  - claude-agent-developer (creates complex configurations)
  
- **Sonnet (18 agents)**: Standard development and analysis tasks
  - All engineering, testing, and documentation roles
  - More efficient for routine work

## How MAOS Uses Agents

1. **Claude Code** receives user request
2. **Claude Code** analyzes task and decides to use multiple agents
3. **MAOS hooks** intercept Task tool calls with `subagent_type`
4. **Git worktrees** provide isolated workspaces for each agent
5. **Agents work** in parallel without conflicts
6. **Results merge** back seamlessly

The orchestration is completely invisible to users - they just see faster, more comprehensive results.

## Custom Agent Creation

To create a new agent:

1. Create a markdown file in `.claude/agents/`
2. Add required frontmatter (name, description, tools, model)
3. Write clear instructions for the agent's behavior
4. Restart Claude Code to load the new agent

Example:
```markdown
---
name: my-custom-agent
description: Does something specific. TRIGGERS: "custom task", "special work"
tools: Read, Write, Edit, Grep, TodoWrite
model: sonnet
---

# Purpose
Clear description of what this agent does...

## Instructions
Step-by-step guidance...
```

## Notes

- The "orchestrator" meta-role mentioned in older docs has been replaced by Claude Code's native orchestration
- All agent files must be in `.claude/agents/` (no subdirectories)
- Agent names must be unique and match the filename
- Restart Claude Code after adding/modifying agents