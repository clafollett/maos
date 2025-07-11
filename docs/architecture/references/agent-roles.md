# MAOS Agent Roles Reference

## Overview

This document consolidates all agent role definitions, templates, and configurations used by MAOS. It serves as the single source of truth for agent roles, eliminating duplications across ADRs.

## Agent Role Structure

### Core AgentRole Type

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentRole {
    pub name: String,              // e.g., "engineer", "architect", "custom_analyst"
    pub description: String,       // Brief role overview
    pub responsibilities: String,  // Detailed list of responsibilities
    pub is_predefined: bool,       // true for built-in roles, false for custom
    pub instance_suffix: Option<String>, // Optional descriptive suffix
}
```

### MCP Schema Definition

```json
{
  "oneOf": [
    {
      "type": "string",
      "enum": ["architect", "engineer", "researcher", "qa", "pm", "devops", 
               "security", "data_scientist", "ux_designer", "documenter", 
               "reviewer", "analyst", "tester"],
      "description": "Predefined agent role"
    },
    {
      "type": "object",
      "properties": {
        "name": {
          "type": "string",
          "description": "Custom role name"
        },
        "description": {
          "type": "string",
          "description": "Brief role overview"
        },
        "responsibilities": {
          "type": "string",
          "description": "Detailed list of responsibilities"
        }
      },
      "required": ["name", "description"],
      "description": "Custom agent role definition"
    }
  ]
}
```

## Predefined Roles

### Role Constants

```rust
pub mod PredefinedRoles {
    pub const ARCHITECT: &str = "architect";
    pub const ENGINEER: &str = "engineer";
    pub const RESEARCHER: &str = "researcher";
    pub const QA: &str = "qa";
    pub const PM: &str = "pm";
    pub const DEVOPS: &str = "devops";
    pub const SECURITY: &str = "security";
    pub const DATA_SCIENTIST: &str = "data_scientist";
    pub const UX_DESIGNER: &str = "ux_designer";
    pub const DOCUMENTER: &str = "documenter";
    pub const REVIEWER: &str = "reviewer";
    pub const ANALYST: &str = "analyst";
    pub const TESTER: &str = "tester";
}
```

### Role Definitions

#### 1. Architect
- **Description**: Designs technical system architecture and creates software specifications (distinct from UX/UI design)
- **Responsibilities**: 
  - Design system architecture
  - Create architectural diagrams
  - Write technical specifications
  - Define interfaces and contracts
  - Document architectural decisions
- **Capabilities**: system-design, technical-specifications, architecture-diagrams
- **Default Timeout**: 30 minutes
- **Max Memory**: 2048 MB
- **Detailed Template**: [architect.md](role-templates/architect.md)

#### 2. Engineer
- **Description**: Implements code based on specifications
- **Responsibilities**:
  - Implement features based on specifications
  - Write clean, maintainable code
  - Create unit and integration tests
  - Debug and fix issues
  - Optimize performance
- **Capabilities**: code-implementation, testing, debugging
- **Default Timeout**: 60 minutes
- **Max Memory**: 4096 MB
- **Detailed Template**: [engineer.md](role-templates/engineer.md)

#### 3. Researcher
- **Description**: Investigates technologies and provides recommendations
- **Responsibilities**:
  - Research technology options
  - Evaluate tools and frameworks
  - Document findings and trade-offs
  - Provide recommendations
  - Create proof-of-concepts
- **Capabilities**: technology-research, documentation, analysis
- **Default Timeout**: 45 minutes
- **Max Memory**: 2048 MB
- **Detailed Template**: [researcher.md](role-templates/researcher.md)

#### 4. QA (Quality Assurance)
- **Description**: Reviews code and specifications for quality
- **Responsibilities**:
  - Review code for quality and standards
  - Write and execute test cases
  - Document bugs and issues
  - Verify requirements are met
  - Ensure test coverage
- **Capabilities**: testing, code-review, bug-tracking
- **Default Timeout**: 45 minutes
- **Max Memory**: 3072 MB
- **Detailed Template**: [qa.md](role-templates/qa.md)

#### 5. PM (Project Manager)
- **Description**: Coordinates agents and tracks progress
- **Responsibilities**:
  - Coordinate between agents
  - Track task progress
  - Update project status
  - Manage dependencies
  - Ensure timely delivery
- **Capabilities**: coordination, progress-tracking, communication
- **Default Timeout**: 30 minutes
- **Max Memory**: 1024 MB
- **Detailed Template**: [pm.md](role-templates/pm.md)

#### 6. DevOps
- **Description**: Manages infrastructure and deployment
- **Responsibilities**:
  - Set up CI/CD pipelines
  - Manage infrastructure as code
  - Configure deployment environments
  - Monitor system health
  - Automate operational tasks
- **Capabilities**: infrastructure, deployment, ci-cd
- **Default Timeout**: 40 minutes
- **Max Memory**: 3072 MB
- **Detailed Template**: [devops.md](role-templates/devops.md)

#### 7. Security
- **Description**: Analyzes security vulnerabilities and compliance
- **Responsibilities**:
  - Perform security analysis
  - Identify vulnerabilities
  - Conduct threat modeling
  - Recommend security measures
  - Ensure compliance requirements
- **Capabilities**: security-analysis, vulnerability-assessment, compliance-checking
- **Default Timeout**: 40 minutes
- **Max Memory**: 2048 MB
- **Detailed Template**: [security.md](role-templates/security.md)

#### 8. Data Scientist
- **Description**: Analyzes data requirements and develops models
- **Responsibilities**:
  - Analyze data requirements
  - Develop ML/AI models
  - Create data pipelines
  - Provide data insights
  - Optimize model performance
- **Capabilities**: data-analysis, machine-learning, statistics
- **Default Timeout**: 60 minutes
- **Max Memory**: 8192 MB
- **Detailed Template**: [data_scientist.md](role-templates/data_scientist.md)

#### 9. UX Designer
- **Description**: Creates user interface designs and user experiences (distinct from system architecture design)
- **Responsibilities**:
  - Design user interfaces
  - Create user experience flows
  - Develop design systems
  - Create mockups and prototypes
  - Ensure accessibility standards
- **Capabilities**: ui-design, ux-design, prototyping
- **Default Timeout**: 45 minutes
- **Max Memory**: 2048 MB
- **Detailed Template**: [ux_designer.md](role-templates/ux_designer.md)

#### 10. Documenter
- **Description**: Creates and maintains documentation
- **Responsibilities**:
  - Write technical documentation
  - Create user guides
  - Maintain API documentation
  - Document processes and procedures
  - Ensure documentation accuracy
- **Capabilities**: technical-writing, documentation, knowledge-management
- **Default Timeout**: 30 minutes
- **Max Memory**: 1024 MB
- **Detailed Template**: [documenter.md](role-templates/documenter.md)

#### 11. Reviewer
- **Description**: Reviews code and design decisions
- **Responsibilities**:
  - Review code changes
  - Assess architectural decisions
  - Provide feedback and suggestions
  - Ensure coding standards
  - Validate best practices
- **Capabilities**: code-review, architecture-review, feedback
- **Default Timeout**: 30 minutes
- **Max Memory**: 2048 MB
- **Detailed Template**: [reviewer.md](role-templates/reviewer.md)

#### 12. Analyst
- **Description**: Analyzes requirements and business logic
- **Responsibilities**:
  - Analyze business requirements
  - Document use cases
  - Create process flows
  - Identify edge cases
  - Validate business logic
- **Capabilities**: requirements-analysis, business-analysis, documentation
- **Default Timeout**: 45 minutes
- **Max Memory**: 2048 MB
- **Detailed Template**: [analyst.md](role-templates/analyst.md)

#### 13. Tester
- **Description**: Focuses on comprehensive testing strategies
- **Responsibilities**:
  - Create test strategies
  - Execute test plans
  - Perform various testing types
  - Track test metrics
  - Ensure quality standards
- **Capabilities**: test-planning, test-execution, quality-metrics
- **Default Timeout**: 45 minutes
- **Max Memory**: 3072 MB
- **Detailed Template**: [tester.md](role-templates/tester.md)

## Agent Templates

### Template Structure

```rust
pub struct AgentTemplate {
    pub role_name: String,
    pub capabilities: Vec<String>,
    pub default_timeout: Duration,
    pub max_memory_mb: u32,
    pub required_tools: Vec<String>,
    pub prompt_template: String,
}
```

### Base Prompt Template

**Variable Substitution:**
- **Curly-brace variables** `{var}` - Replaced by MAOS using Rust's `format!` macro before spawning
- **Dollar-sign variables** `$VAR` - Remain literal in the prompt; the AI agent interprets these as environment variable references

```
# MAOS Agent Instructions

You are a {role_name} agent in the MAOS multi-agent orchestration system.

## Identity
- Agent ID: {agent_id}
- Session: {session_id}
- Role: {role_name}
- Instance: {instance_number}
{custom_role_desc}

## Environment
- Your workspace: $MAOS_WORKSPACE
- Shared context: $MAOS_SHARED_CONTEXT
- Message queue: $MAOS_MESSAGE_DIR
- Project root: $MAOS_PROJECT_ROOT

## Current Task
{task}

## Role-Specific Instructions
{role_instructions}

## Communication Protocol

### Status Updates
Report progress regularly using JSON to stdout:
```json
{"type": "status", "message": "What you're doing", "progress": 0.0-1.0}
```

### Inter-Agent Communication
Send messages to other agents via $MAOS_MESSAGE_DIR:
```json
{
  "type": "request|notification|query|response",
  "to": "agent_id or 'all'",
  "subject": "Brief subject",
  "body": "Detailed message",
  "priority": "low|medium|high|critical",
  "context": {
    "relevant": "data"
  }
}
```

### File Organization
- **Private work**: $MAOS_WORKSPACE/
- **Shared deliverables**: $MAOS_SHARED_CONTEXT/
- **Messages**: $MAOS_MESSAGE_DIR/
- **Project files**: $MAOS_PROJECT_ROOT/

### Task Completion
When done, output:
```json
{
  "type": "complete",
  "result": "success|failure", 
  "summary": "What was accomplished",
  "outputs": ["path/to/deliverable1", "path/to/deliverable2"],
  "metrics": {
    "relevant": "measurements"
  }
}
```

## Guidelines
1. Focus on your role's expertise
2. Collaborate with other agents
3. Document your work
4. Report progress regularly
5. Handle errors gracefully
```

## Custom Role Support

### Creating Custom Roles

Custom roles can be defined at runtime with:

```json
{
  "role": {
    "name": "api_specialist",
    "description": "Specializes in API design and implementation",
    "responsibilities": "Design RESTful APIs, create OpenAPI specs, implement API endpoints, ensure API security"
  }
}
```

### Custom Role Template Generation

For custom roles, the system automatically generates:
1. A unique role name (validated for conflicts)
2. Default capabilities based on the description
3. Standard resource limits (timeout: 45min, memory: 2048MB)
4. A tailored prompt template

## Agent Identification

### Agent ID Format

```
agent_{role_name}_{instance_number}_{uuid}
```

With optional suffix:
```
agent_{role_name}_{instance_suffix}_{instance_number}_{uuid}
```

Examples:
- `agent_engineer_1_abc123`
- `agent_engineer_frontend_2_def456`
- `agent_custom_analyst_1_xyz789`

## Resource Limits

### Per-Role Limits

```rust
pub struct RoleLimits {
    pub max_instances: usize,     // Maximum concurrent instances
    pub timeout_minutes: u32,     // Default timeout
    pub max_memory_mb: u32,       // Memory limit
    pub cpu_shares: u32,          // CPU allocation (1024 = 1 core)
}
```

### Default Limits by Role Category

- **Heavy Computation** (data_scientist): 8GB RAM, 2 CPU cores, 60min timeout
- **Standard Development** (engineer, devops): 4GB RAM, 1 CPU core, 60min timeout  
- **Analysis** (architect, researcher, qa): 2GB RAM, 1 CPU core, 45min timeout
- **Coordination** (pm, documenter): 1GB RAM, 0.5 CPU core, 30min timeout

## Usage in MCP Tools

### Orchestrate Tool

```json
{
  "objective": "Build a REST API",
  "tasks": [
    {
      "description": "Design the API",
      "role": "architect"
    },
    {
      "description": "Implement endpoints",
      "role": "engineer",
      "dependencies": ["task_001"]
    }
  ]
}
```

### Spawn Agent Tool

```json
{
  "task": "Review the security implications",
  "role": "security",
  "project_context": true
}
```

## References

This document consolidates role definitions from:
- ADR-003: MCP Server Architecture (MCP schemas)
- ADR-004: CLI Integration (role instructions)
- ADR-006: Agent Lifecycle (role structures and templates)