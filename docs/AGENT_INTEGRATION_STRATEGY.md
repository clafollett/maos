# Agent Integration Strategy

## Overview

MAOS (Multi-Agent Orchestration System) operates as an MCP (Model Context Protocol) server that orchestrates AI agents through process spawning. Claude Code (or any MCP-compatible client) connects to MAOS, which then spawns and manages specialized agent processes to accomplish complex tasks.

## Architecture Overview

```
┌─────────────────────────┐
│   Claude Code Client    │
│   (User Interface)      │
└───────────┬─────────────┘
            │ MCP Protocol
            ▼
┌─────────────────────────┐
│   MAOS MCP Server       │
│   - Orchestration       │
│   - Process Management  │
│   - Session Tracking    │
└───────────┬─────────────┘
            │ Spawns
            ▼
┌─────────────────────────┐
│   Agent Processes       │
│   - claude -p           │
│   - Multiple Instances  │
│   - Role Specialization │
└─────────────────────────┘
```

## Agent Role System

MAOS supports both predefined and custom agent roles, allowing flexibility in task specialization:

### Predefined Roles

| Role | Description | Primary Responsibilities |
|------|-------------|-------------------------|
| **Architect** | System design specialist | Design architecture, create technical specifications, document system structure |
| **Engineer** | Code implementation | Write code, implement features, debug issues, optimize performance |
| **Researcher** | Technology investigation | Research libraries, evaluate options, provide recommendations |
| **QA** | Quality assurance | Test code, verify requirements, ensure quality standards |
| **PM** | Project management | Track progress, coordinate agents, manage dependencies |
| **DevOps** | Infrastructure specialist | Setup CI/CD, manage deployments, infrastructure as code |
| **Security** | Security analysis | Vulnerability assessment, security recommendations, threat modeling |
| **DataScientist** | Data analysis | Analyze data, develop ML models, provide insights |
| **Designer** | UI/UX design | Create mockups, design user interfaces, improve UX |
| **Documenter** | Technical writing | Write documentation, API docs, user guides |
| **Reviewer** | Code review | Review PRs, suggest improvements, ensure standards |
| **Analyst** | Business analysis | Analyze requirements, define specifications |
| **Tester** | Specialized testing | Integration testing, performance testing, edge cases |

### Custom Roles

Users can define custom roles on-the-fly:

```json
{
  "role": {
    "name": "api_specialist",
    "description": "Expert in REST API design and OpenAPI specifications",
    "responsibilities": "Design RESTful APIs, create OpenAPI specs, ensure API consistency"
  },
  "task": "Design the authentication API endpoints"
}
```

## Integration Patterns

### 1. Process Spawning

MAOS spawns agent processes with specific environment variables:

```rust
let child = Command::new("claude")
    .current_dir(&workspace)
    .arg("-p")
    .arg(&prompt)
    .env("MAOS_AGENT_ROLE", role_name)
    .env("MAOS_AGENT_ROLE_DESC", role_description)
    .env("MAOS_AGENT_INSTANCE", instance_number)
    .env("MAOS_SESSION_ID", session_id)
    .env("MAOS_AGENT_ID", agent_id)
    .env("MAOS_WORKSPACE", workspace_path)
    .env("MAOS_SHARED_CONTEXT", shared_context_path)
    .env("MAOS_MESSAGE_DIR", message_dir_path)
    .env("MAOS_PROJECT_ROOT", project_root)
    .spawn()?;
```

### 2. Agent Communication

Agents communicate through:
- **Shared Context Directory**: For specifications, designs, and artifacts
- **Message Queue**: For inter-agent communication
- **JSON stdout**: For status updates to MAOS

#### Message Routing Patterns

```python
# Send to specific agent
communicator.send_message("agent_engineer_1_abc123", "request", subject, body)

# Send to all agents of a role
communicator.broadcast_to_role("engineer", "notification", subject, body)

# Send to pattern-matched agents
communicator.send_message("engineer_*", "announcement", subject, body)

# Broadcast to all agents
communicator.send_message("all", "alert", subject, body)
```

### 3. Multiple Instance Support

MAOS can spawn multiple agents of the same role:

```
agent_engineer_1_abc123    # First engineer instance
agent_engineer_2_def456    # Second engineer instance
agent_engineer_frontend_3_ghi789  # Third engineer with suffix
```

## MCP Tool Interface

### Orchestrate Tool

Start a multi-agent session with specific roles and tasks:

```json
{
  "objective": "Build a REST API with authentication",
  "agents": [
    {
      "role": "architect",
      "task": "Design the API architecture and data models"
    },
    {
      "role": "engineer",
      "task": "Implement the authentication endpoints",
      "instance_suffix": "backend"
    },
    {
      "role": "engineer", 
      "task": "Create the frontend authentication UI",
      "instance_suffix": "frontend"
    },
    {
      "role": {
        "name": "api_tester",
        "description": "API testing specialist",
        "responsibilities": "Test API endpoints, verify responses, check edge cases"
      },
      "task": "Create comprehensive API tests"
    }
  ],
  "strategy": "parallel",
  "max_agents_per_role": {
    "engineer": 3,
    "architect": 1
  }
}
```

### Spawn Agent Tool

Add agents dynamically during orchestration:

```json
{
  "role": "security",
  "task": "Review the authentication implementation for vulnerabilities",
  "session_id": "sess_abc123",
  "dependencies": ["agent_engineer_backend_1_def456"],
  "template_override": {
    "timeout_seconds": 1800,
    "max_memory_mb": 4096
  }
}
```

## Session Management

### Session Lifecycle

1. **Creation**: Initialize session with objective and agent specifications
2. **Agent Spawning**: Launch agents based on strategy (parallel/sequential/adaptive)
3. **Execution**: Agents work on tasks, communicate, share artifacts
4. **Monitoring**: Track progress, handle failures, manage dependencies
5. **Completion**: Collect results, cleanup resources

### Execution Strategies

| Strategy | Description | Use Case |
|----------|-------------|----------|
| **Parallel** | All agents run simultaneously | Independent tasks |
| **Sequential** | Agents run one after another | Dependent workflows |
| **Adaptive** | Dynamic scheduling based on dependencies | Complex projects |
| **Pipeline** | Staged execution with handoffs | Multi-phase projects |

## Resource Management

### Agent Limits

```yaml
resource_limits:
  max_total_agents: 10
  max_agents_per_role:
    engineer: 5
    architect: 2
    default: 3
  max_total_memory_mb: 16384
  default_timeout_seconds: 3600
```

### Instance Tracking

MAOS tracks all running instances:

```sql
CREATE TABLE instances (
    id TEXT PRIMARY KEY,
    pid INTEGER NOT NULL,
    port INTEGER NOT NULL,
    workspace_hash TEXT NOT NULL,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_heartbeat TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## Agent Templates

### Dynamic Template Generation

For custom roles, MAOS generates appropriate templates:

```rust
impl TemplateGenerator {
    fn generate_custom_template(&self, role: &AgentRole) -> AgentTemplate {
        // Infer capabilities from role description
        let capabilities = self.infer_capabilities(&role.description);
        
        // Suggest appropriate timeout
        let timeout = self.suggest_timeout(&role);
        
        // Determine required tools
        let tools = self.infer_required_tools(&role);
        
        AgentTemplate {
            role_name: role.name.clone(),
            capabilities,
            default_timeout: timeout,
            max_memory_mb: self.base_memory_mb,
            required_tools: tools,
            prompt_template: self.generate_prompt_template(role),
        }
    }
}
```

## Error Handling and Recovery

### Agent Failure Handling

1. **Health Monitoring**: Regular process checks every 5 seconds
2. **Automatic Restart**: For transient failures (configurable)
3. **Dependency Resolution**: Notify dependent agents of failures
4. **Session Recovery**: Resume interrupted sessions on MAOS restart

### Graceful Shutdown

```rust
impl ProcessManager {
    async fn shutdown_agent(&self, agent_id: &str, timeout: Duration) -> Result<()> {
        // Send graceful shutdown signal
        // Wait for completion with timeout
        // Force kill if necessary
        // Cleanup resources
    }
}
```

## Security Considerations

### Process Isolation

- Each agent runs in an isolated workspace
- Limited file system access (workspace + shared context)
- Resource limits enforced at OS level
- No direct access to MAOS internals

### Authentication Flow

1. User authenticates with Claude Code
2. Claude Code connects to MAOS via MCP
3. MAOS spawns agents with inherited permissions
4. Agents use CLI's existing authentication

## Development Workflow

### Local Development

```bash
# Start MAOS MCP server
maos serve --port 3000

# Configure Claude Code to connect
claude --mcp-server http://localhost:3000

# Use MAOS tools through Claude Code
"Orchestrate a new feature implementation with architect and 2 engineers"
```

### Testing Strategy

1. **Unit Tests**: Core orchestration logic
2. **Integration Tests**: MCP protocol compliance
3. **Process Tests**: Agent spawning and management
4. **End-to-End Tests**: Full orchestration scenarios

## Future Enhancements

### Phase 1: Core Implementation
- Basic MCP server with orchestration tools
- Process spawning for predefined roles
- Simple session management
- File-based communication

### Phase 2: Advanced Features
- Custom role support
- Multiple instance management
- Advanced routing patterns
- Session persistence and recovery

### Phase 3: Production Features
- Multi-instance MAOS support
- Resource pooling
- Performance optimization
- Advanced monitoring

### Phase 4: Extended Capabilities
- Support for other CLI tools (when available)
- Cross-session agent collaboration
- Machine learning for role optimization
- Automated task decomposition

## References

- [ADR-10: MCP Server Architecture](./architecture/decisions/10-mcp-server-architecture.md)
- [ADR-05: CLI Integration and Process Spawning](./architecture/decisions/05-cli-integration-and-process-spawning.md)
- [ADR-08: Agent Lifecycle and Management](./architecture/decisions/08-agent-lifecycle-and-management.md)
- [ADR-04: Agent Communication Patterns](./architecture/decisions/04-agent-communication-patterns.md)
- [ADR-07: Orchestration Guardrails and Coordination Protocols](./architecture/decisions/07-orchestration-guardrails.md)
- [ADR-11: Adaptive Phase-Based Orchestration](./architecture/decisions/11-adaptive-phase-based-orchestration.md)
- [POC Learnings and Findings](./architecture/references/poc-learnings.md)
- [Model Context Protocol Specification](https://modelcontextprotocol.io)