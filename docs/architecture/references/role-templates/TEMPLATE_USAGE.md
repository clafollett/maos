# MAOS Role Template Usage Guide

This guide explains how to use the detailed role-specific prompt templates in MAOS.

## Overview

MAOS provides 13 predefined role templates, each optimized for specific types of tasks:

1. **[Architect](architect.md)** - System design and technical specifications
2. **[Engineer](engineer.md)** - Code implementation and development
3. **[Researcher](researcher.md)** - Technology evaluation and recommendations
4. **[QA](qa.md)** - Quality assurance and code review
5. **[PM](pm.md)** - Project coordination and progress tracking
6. **[DevOps](devops.md)** - Infrastructure and deployment management
7. **[Security](security.md)** - Security analysis and compliance
8. **[Data Scientist](data_scientist.md)** - Data analysis and ML models
9. **[UX Designer](ux_designer.md)** - User interface/experience design (distinct from system architecture)
10. **[Documenter](documenter.md)** - Technical and user documentation
11. **[Reviewer](reviewer.md)** - Code and design reviews
12. **[Analyst](analyst.md)** - Requirements and business analysis
13. **[Tester](tester.md)** - Testing strategies and execution

## How Templates Work

### Variable Substitution

Templates use two types of variables:

1. **Curly-brace variables `{var}`** - Replaced by MAOS before spawning:
   - `{role_name}` - The agent's role
   - `{agent_id}` - Unique agent identifier
   - `{session_id}` - Current session ID
   - `{instance_number}` - Instance number for this role
   - `{task}` - The specific task description
   - `{custom_role_desc}` - Additional role description (if custom)

2. **Dollar-sign variables `$VAR`** - Environment variables for agent use:
   - `$MAOS_WORKSPACE` - Agent's private workspace
   - `$MAOS_SHARED_CONTEXT` - Shared context directory
   - `$MAOS_MESSAGE_DIR` - Message queue location
   - `$MAOS_PROJECT_ROOT` - Project root directory

### Template Structure

Each template follows this structure:

```markdown
# [Role] Agent Prompt Template

## Identity
- Agent identification and context

## Environment
- Available directories and resources

## Current Task
- The specific task to accomplish

## Your Responsibilities as a [Role]
- Primary focus
- Key deliverables
- Workflow guidelines
- Examples and patterns
- Communication templates
- Status reporting format

## Remember
- Key principles and best practices
```

## Using Templates in Practice

### 1. Spawning an Agent with Default Template

```json
{
  "tool": "spawn_agent",
  "task": "Design the authentication system",
  "role": "architect"
}
```

This automatically uses the architect template with:
- Predefined responsibilities
- Architect-specific workflows
- Design documentation patterns
- Communication protocols

### 2. Customizing Role Behavior

```json
{
  "tool": "spawn_agent",
  "task": "Design the authentication system with OAuth2 focus",
  "role": {
    "name": "architect",
    "description": "Security-focused architect specializing in OAuth2",
    "responsibilities": "Design secure authentication with OAuth2, OIDC, and MFA support"
  }
}
```

### 3. Creating Custom Roles

```json
{
  "tool": "spawn_agent",
  "task": "Analyze API performance bottlenecks",
  "role": {
    "name": "performance_analyst",
    "description": "Specializes in API performance optimization",
    "responsibilities": "Profile APIs, identify bottlenecks, recommend optimizations"
  }
}
```

## Template Selection Guide

### By Task Type

| Task Type | Recommended Role | Why |
|-----------|-----------------|-----|
| System architecture | Architect | Has technical design patterns and architecture templates |
| Code implementation | Engineer | Includes coding standards and patterns |
| Bug investigation | QA/Tester | Testing mindset and debugging workflows |
| Performance analysis | Data Scientist | Statistical analysis capabilities |
| Security audit | Security | Security checklists and threat models |
| API design | Architect/Analyst | Technical requirements and interface design |
| Documentation | Documenter | Documentation standards and templates |
| Code review | Reviewer | Review checklists and feedback formats |
| Project planning | PM | Coordination and tracking templates |
| Infrastructure | DevOps | IaC templates and deployment patterns |

### By Output Type

| Desired Output | Best Role | Template Features |
|----------------|-----------|-------------------|
| Technical specs | Architect | Specification templates |
| Working code | Engineer | Implementation patterns |
| Test suites | Tester | Test case templates |
| Security report | Security | Threat assessment formats |
| Data insights | Data Scientist | Analysis frameworks |
| UI mockups | UX Designer | Design system guidelines |
| User guides | Documenter | Documentation templates |
| Requirements | Analyst | Requirements templates |

## Communication Patterns

All templates include structured communication:

### Status Updates
```json
{"type": "status", "message": "Current activity", "progress": 0.5}
```

### Inter-Agent Messages
```json
{
  "type": "request|notification|query",
  "to": "agent_id or 'all'",
  "subject": "Brief subject",
  "body": "Detailed message",
  "priority": "low|medium|high|critical",
  "context": {}
}
```

### Task Completion
```json
{
  "type": "complete",
  "result": "success|failure",
  "outputs": ["file1", "file2"],
  "metrics": {}
}
```

## Best Practices

### 1. Match Role to Task
- Use Architect for design tasks
- Use Engineer for implementation
- Use Tester for quality assurance
- Don't use Engineer for pure analysis

### 2. Leverage Role Strengths
- Architects excel at system-level thinking
- Engineers focus on clean implementation
- Testers think about edge cases
- Security experts consider vulnerabilities

### 3. Coordinate Roles
```python
# Example: Building a feature
tasks = [
    {"role": "analyst", "task": "Gather requirements"},
    {"role": "architect", "task": "Design the system"},
    {"role": "engineer", "task": "Implement the feature"},
    {"role": "tester", "task": "Create test suite"},
    {"role": "reviewer", "task": "Review implementation"},
    {"role": "documenter", "task": "Write user guide"}
]
```

### 4. Use Templates as Starting Points
- Templates provide structure
- Agents adapt based on specific tasks
- Custom roles extend capabilities
- Context shapes behavior

## Advanced Usage

### Combining Roles
For complex tasks, spawn multiple specialized agents:

```python
# Frontend + Backend + Database
spawn_agent(role="engineer", task="Implement React frontend", 
           suffix="frontend")
spawn_agent(role="engineer", task="Build REST API", 
           suffix="backend")
spawn_agent(role="engineer", task="Design database schema", 
           suffix="database")
```

### Role Progression
Natural task flow through roles:

1. **Analyst** → Gather requirements
2. **Architect** → Design solution
3. **Engineer** → Implement code
4. **Tester** → Verify quality
5. **Reviewer** → Final approval
6. **DevOps** → Deploy to production
7. **Documenter** → Update documentation

### Parallel Execution
Roles that work well in parallel:

- **Engineer + Tester** - TDD approach
- **Architect + Security** - Secure technical design
- **Multiple Engineers** - Different components
- **QA + Documenter** - Quality and docs

## Troubleshooting

### Common Issues

1. **Wrong Role Selection**
   - Symptom: Agent struggles with task
   - Solution: Match role expertise to task type

2. **Missing Context**
   - Symptom: Agent lacks information
   - Solution: Provide comprehensive task description

3. **Role Overlap**
   - Symptom: Multiple agents doing same work
   - Solution: Clear task boundaries

4. **Communication Breakdown**
   - Symptom: Agents not coordinating
   - Solution: Use structured message formats

### Performance Tips

1. **Specialized > General**
   - Specific roles perform better
   - Clear responsibilities improve focus

2. **Appropriate Resources**
   - Data Scientists need more memory
   - Engineers benefit from longer timeouts

3. **Incremental Tasks**
   - Break large tasks into steps
   - Each role handles its expertise

## Summary

MAOS role templates provide:
- Structured approach to tasks
- Consistent communication patterns
- Role-specific best practices
- Flexible customization options

Choose roles based on:
- Task requirements
- Desired outputs
- Required expertise
- Workflow patterns

The templates ensure agents:
- Understand their responsibilities
- Follow proven workflows
- Communicate effectively
- Deliver consistent results