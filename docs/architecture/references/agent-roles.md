# MAOS Agent Roles Reference

## Overview

This document consolidates all agent role definitions, templates, and configurations used by MAOS. It serves as the single source of truth for agent roles, eliminating duplications across ADRs.

MAOS currently defines **20 predefined agent roles**: 1 meta-role (Orchestrator), 5 specialized architect roles, 3 specialized engineer roles, and 11 other domain-specific roles.

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
      "enum": ["orchestrator", "solution_architect", "application_architect", "data_architect", 
               "api_architect", "security_architect", "backend_engineer", "frontend_engineer", 
               "mobile_engineer", "researcher", "qa", "pm", "devops", "security", 
               "data_scientist", "ux_designer", "documenter", "reviewer", "analyst", "tester"],
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
    pub const ORCHESTRATOR: &str = "orchestrator";
    pub const SOLUTION_ARCHITECT: &str = "solution_architect";
    pub const APPLICATION_ARCHITECT: &str = "application_architect";
    pub const DATA_ARCHITECT: &str = "data_architect";
    pub const API_ARCHITECT: &str = "api_architect";
    pub const SECURITY_ARCHITECT: &str = "security_architect";
    pub const BACKEND_ENGINEER: &str = "backend_engineer";
    pub const FRONTEND_ENGINEER: &str = "frontend_engineer";
    pub const MOBILE_ENGINEER: &str = "mobile_engineer";
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

#### 0. Orchestrator (Meta-Role)
- **Description**: Coordinates multi-agent workflows by planning phases, spawning agents, and managing orchestration
- **Responsibilities**: 
  - Analyze user requests and break them into executable phases
  - Determine optimal agent roles and task assignments
  - Plan sequential vs parallel execution strategies
  - Adapt plans based on phase completion and new information
  - Coordinate agent communication and dependency management
  - Handle dynamic re-planning when requirements evolve
- **Capabilities**: strategic-planning, agent-coordination, adaptive-orchestration, phase-management
- **Default Timeout**: 20 minutes (planning and coordination tasks)
- **Max Memory**: 4096 MB (large orchestration context and planning)
- **Detailed Template**: [orchestrator.md](../../../assets/agent-roles/orchestrator.md)
- **Special Properties**: 
  - **Always Auto-Spawned**: Never explicitly requested by users
  - **Meta-Role**: Coordinates other agents rather than performing domain work
  - **Adaptive**: Uses phase-based planning with continuous re-evaluation
  - **Omnipresent**: Active throughout entire orchestration lifecycle

#### 1. Solution Architect
- **Description**: Designs end-to-end solutions across multiple systems and domains
- **Responsibilities**:
  - Design cross-system integration strategies
  - Select appropriate technologies and platforms
  - Create enterprise-level solution blueprints
  - Coordinate multiple system architectures
  - Ensure solution alignment with business requirements
- **Capabilities**: solution-design, technology-selection, enterprise-integration
- **Default Timeout**: 45 minutes
- **Max Memory**: 3072 MB
- **Detailed Template**: [solution_architect.md](../../../assets/agent-roles/solution_architect.md)

#### 2. Application Architect
- **Description**: Designs internal structure and patterns for single applications
- **Responsibilities**:
  - Design application component architecture
  - Define internal APIs and service boundaries
  - Establish application-level design patterns
  - Create modular application structures
  - Optimize application performance patterns
- **Capabilities**: component-design, internal-apis, design-patterns
- **Default Timeout**: 35 minutes
- **Max Memory**: 2048 MB
- **Detailed Template**: [application_architect.md](../../../assets/agent-roles/application_architect.md)

#### 3. Data Architect
- **Description**: Designs data models, storage systems, and data flow architecture
- **Responsibilities**:
  - Design database schemas and data models
  - Plan data storage and retrieval strategies
  - Design data pipelines and ETL processes
  - Establish data governance and quality standards
  - Optimize data access patterns
- **Capabilities**: data-modeling, database-design, data-pipelines
- **Default Timeout**: 40 minutes
- **Max Memory**: 3072 MB
- **Detailed Template**: [data_architect.md](../../../assets/agent-roles/data_architect.md)

#### 4. API Architect
- **Description**: Designs API interfaces, service contracts, and integration patterns
- **Responsibilities**:
  - Design REST/GraphQL API specifications
  - Define service contracts and interfaces
  - Establish API governance and standards
  - Design API versioning and evolution strategies
  - Create API documentation and integration guides
- **Capabilities**: api-design, service-contracts, api-governance
- **Default Timeout**: 35 minutes
- **Max Memory**: 2048 MB
- **Detailed Template**: [api_architect.md](../../../assets/agent-roles/api_architect.md)

#### 5. Security Architect
- **Description**: Designs security controls, threat models, and compliance frameworks
- **Responsibilities**:
  - Design security architecture and controls
  - Perform threat modeling and risk assessment
  - Define authentication and authorization patterns
  - Establish security compliance frameworks
  - Create security integration patterns
- **Capabilities**: security-design, threat-modeling, compliance-frameworks
- **Default Timeout**: 40 minutes
- **Max Memory**: 2048 MB
- **Detailed Template**: [security_architect.md](../../../assets/agent-roles/security_architect.md)

#### 6. Backend Engineer
- **Description**: Implements server-side logic, APIs, and data processing
- **Responsibilities**:
  - Implement server-side application logic
  - Build REST/GraphQL APIs and microservices
  - Design and implement database interactions
  - Handle authentication, authorization, and security
  - Create unit and integration tests for backend systems
  - Optimize server performance and scalability
- **Capabilities**: server-side-development, api-implementation, database-integration
- **Default Timeout**: 60 minutes
- **Max Memory**: 4096 MB
- **Detailed Template**: [backend_engineer.md](../../../assets/agent-roles/backend_engineer.md)

#### 7. Frontend Engineer
- **Description**: Implements user interfaces and client-side application logic
- **Responsibilities**:
  - Implement responsive user interfaces
  - Build client-side application logic and state management
  - Integrate with APIs and backend services
  - Ensure cross-browser compatibility and accessibility
  - Create unit and integration tests for frontend components
  - Optimize frontend performance and user experience
- **Capabilities**: frontend-development, ui-implementation, client-side-optimization
- **Default Timeout**: 60 minutes
- **Max Memory**: 4096 MB
- **Detailed Template**: [frontend_engineer.md](../../../assets/agent-roles/frontend_engineer.md)

#### 8. Mobile Engineer
- **Description**: Implements mobile applications for iOS, Android, or cross-platform
- **Responsibilities**:
  - Develop native or cross-platform mobile applications
  - Implement mobile-specific UI patterns and interactions
  - Handle device capabilities (camera, GPS, sensors)
  - Integrate with mobile backend services and APIs
  - Optimize for mobile performance and battery life
  - Create mobile-specific testing strategies
- **Capabilities**: mobile-development, native-platforms, mobile-optimization
- **Default Timeout**: 60 minutes
- **Max Memory**: 4096 MB
- **Detailed Template**: [mobile_engineer.md](../../../assets/agent-roles/mobile_engineer.md)

#### 9. Researcher
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
- **Detailed Template**: [researcher.md](../../../assets/agent-roles/researcher.md)

#### 10. QA (Quality Assurance)
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
- **Detailed Template**: [qa.md](../../../assets/agent-roles/qa.md)

#### 11. PM (Project Manager)
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
- **Detailed Template**: [pm.md](../../../assets/agent-roles/pm.md)

#### 12. DevOps
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
- **Detailed Template**: [devops.md](../../../assets/agent-roles/devops.md)

#### 13. Security
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
- **Detailed Template**: [security.md](../../../assets/agent-roles/security.md)

#### 14. Data Scientist
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
- **Detailed Template**: [data_scientist.md](../../../assets/agent-roles/data_scientist.md)

#### 15. UX Designer
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
- **Detailed Template**: [ux_designer.md](../../../assets/agent-roles/ux_designer.md)

#### 16. Documenter
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
- **Detailed Template**: [documenter.md](../../../assets/agent-roles/documenter.md)

#### 17. Reviewer
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
- **Detailed Template**: [reviewer.md](../../../assets/agent-roles/reviewer.md)

#### 18. Analyst
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
- **Detailed Template**: [analyst.md](../../../assets/agent-roles/analyst.md)

#### 19. Tester
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
- **Detailed Template**: [tester.md](../../../assets/agent-roles/tester.md)

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

## Architect Role Selection Guidance

### When to Use Multiple Architect Roles

The specialized architect roles are designed to work together on complex projects. Here's guidance on when to use each:

#### Single Architect Scenarios
- **Architect (General)**: Simple applications with straightforward requirements
- **Application Architect**: Single-application development with moderate complexity
- **API Architect**: API-first projects or simple service development

#### Multi-Architect Scenarios

**Large Enterprise Applications:**
- **Solution Architect**: Overall solution design and technology selection
- **Application Architect**: Internal application structure and components
- **Data Architect**: Database design and data flow optimization
- **Security Architect**: Security controls and compliance requirements

**Microservices or Service-Oriented Architecture:**
- **Solution Architect**: Service topology and integration patterns
- **API Architect**: Service contracts and API governance
- **Data Architect**: Data consistency and distributed data patterns
- **Security Architect**: Inter-service security and API security

**Data-Heavy Applications (Analytics, ML, etc.):**
- **Solution Architect**: Platform and infrastructure selection
- **Data Architect**: Data pipelines, storage, and modeling
- **Security Architect**: Data governance and privacy controls

**Integration Projects:**
- **Solution Architect**: Integration strategy and platform selection
- **API Architect**: Integration APIs and service contracts
- **Data Architect**: Data transformation and synchronization

### Orchestration Patterns

#### Sequential Architecture Development
```json
{
  "objective": "Design enterprise application",
  "tasks": [
    {
      "description": "Design overall solution architecture",
      "role": "solution_architect"
    },
    {
      "description": "Design application internal structure",
      "role": "application_architect",
      "dependencies": ["task_001"]
    },
    {
      "description": "Design data architecture and schemas",
      "role": "data_architect",
      "dependencies": ["task_001"]
    },
    {
      "description": "Design API contracts and governance",
      "role": "api_architect",
      "dependencies": ["task_002"]
    },
    {
      "description": "Design security architecture",
      "role": "security_architect",
      "dependencies": ["task_001", "task_003", "task_004"]
    }
  ]
}
```

#### Parallel Architecture Development (for experienced teams)
```json
{
  "objective": "Design microservices platform",
  "tasks": [
    {
      "description": "Design platform architecture",
      "role": "solution_architect"
    },
    {
      "description": "Design service internal patterns",
      "role": "application_architect"
    },
    {
      "description": "Design data architecture",
      "role": "data_architect"
    },
    {
      "description": "Design API contracts",
      "role": "api_architect"
    }
  ],
  "strategy": "parallel"
}
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
- **Standard Development** (backend_engineer, frontend_engineer, mobile_engineer, devops): 4GB RAM, 1 CPU core, 60min timeout  
- **Analysis** (solution_architect, application_architect, data_architect, api_architect, security_architect, researcher, qa): 2GB RAM, 1 CPU core, 45min timeout
- **Meta-Role** (orchestrator): 4GB RAM, 1 CPU core, 20min timeout
- **Coordination** (pm, documenter): 1GB RAM, 0.5 CPU core, 30min timeout

## Usage in MCP Tools

### Orchestrate Tool

```json
{
  "objective": "Build a REST API",
  "tasks": [
    {
      "description": "Design the API",
      "role": "api_architect"
    },
    {
      "description": "Implement endpoints",
      "role": "backend_engineer",
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
- ADR-10: MCP Server Architecture (MCP schemas)
- ADR-05: CLI Integration (role instructions)  
- ADR-08: Agent Lifecycle and Management (role structures and templates)
- ADR-03: Session Orchestration and State Management (Orchestrator role)
- ADR-11: Adaptive Phase-Based Orchestration (Orchestrator behavior)