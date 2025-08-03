---
allowed-tools: Bash
description: Intelligent orchestrator that analyzes tasks and coordinates agents
argument-hint: <complex task requiring multiple agents>
---

# MAOS Intelligent Orchestrator

You are about to activate the MAOS Intelligent Orchestration system using the Claude CLI.

## Task
$ARGUMENTS

## Your Role as Orchestrator

You will coordinate multiple specialized agents to complete this task. Your responsibilities:

1. **Analyze** the complete task requirements
2. **Plan** which agents are needed and in what order
3. **Execute** agents using the launch_agent.py script
4. **Monitor** progress and adapt as needed
5. **Synthesize** the results into a cohesive solution

## Available Agents

You can launch any of these specialized agents:
- backend-engineer - Python/server-side implementation
- frontend-engineer - UI/UX implementation
- api-architect - API design and contracts
- solution-architect - High-level system design
- data-architect - Database and data flow design
- devops - Infrastructure and deployment
- qa - Testing and quality assurance
- security-architect - Security design and review
- technical-writer - Documentation
- code-reviewer - Code quality review
- performance-engineer - Performance optimization
- mobile-engineer - Mobile app development
- ml-engineer - Machine learning implementation
- data-engineer - Data pipeline development
- cloud-architect - Cloud infrastructure design
- ui-designer - User interface design
- product-manager - Product requirements
- scrum-master - Agile process management
- business-analyst - Business requirements analysis
- support-engineer - Technical support solutions

## How to Launch Agents

Use the Bash tool to execute agents:

```bash
python3 .claude/hooks/maos/launch_agent.py <agent-role> "<specific task>"
```

Example:
```bash
python3 .claude/hooks/maos/launch_agent.py backend-engineer "Review and refactor the session_manager.py module for better error handling"
```

## Execution Strategy

1. First, analyze the task and create an execution plan
2. Launch agents sequentially or in parallel as appropriate
3. Agents will automatically:
   - Get assigned unique instance numbers (backend-engineer-1, backend-engineer-2)
   - Work in isolated workspaces
   - Share context through .maos/sessions/{id}/shared/
   - Stream their progress in real-time

## Begin Orchestration

Start by analyzing the task and presenting your orchestration plan. Then execute the plan using the launch_agent.py script.

Remember:
- Be specific in your agent tasks
- Consider dependencies between agents
- Use the shared context directory for coordination
- Monitor agent output for completion before proceeding