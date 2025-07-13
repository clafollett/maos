# ADR-04: Agent Communication Patterns

## Status
Accepted

## Context
Agents in MAOS need to communicate and coordinate their work effectively. Since each agent runs as a separate CLI process, we need well-defined patterns for:

- Sharing work artifacts (specifications, code, test results)
- Sending messages between agents
- Coordinating dependencies and handoffs
- Broadcasting status updates
- Requesting help or clarification

Key constraints:
- Agents are separate OS processes (not threads)
- Communication must work across different CLI tools
- Need both synchronous and asynchronous patterns
- Must maintain agent isolation for stability

## Decision
We will implement a hybrid communication system using:
1. **Shared file system** for artifacts and persistent data
2. **Message files** for inter-agent messaging
3. **JSON stdout** for status updates to MAOS
4. **Environment variables** for configuration

### Communication Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    MAOS Server                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │           Message Router & Monitor                │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                           │
    ┌──────────────────────┼──────────────────────┐
    │                      │                      │
    ▼                      ▼                      ▼
┌─────────┐          ┌─────────┐          ┌─────────┐
│ Agent 1 │          │ Agent 2 │          │ Agent 3 │
└────┬────┘          └────┬────┘          └────┬────┘
     │                    │                    │
     └────────────────────┴────────────────────┘
                          │
                          ▼
         ┌──────────────────────────────┐
         │     Shared File System       │
         ├──────────────────────────────┤
         │ • shared/context/            │
         │ • shared/messages/           │
         │ • agents/{id}/workspace/     │
         └──────────────────────────────┘
```

### 1. Shared Context Directory

Agents share artifacts through a structured directory:

The shared directory structure for context and messages is documented in the [Storage Schema Reference](../references/storage-schema.md#file-system-structure). Key directories include:
- `shared/context/` - Shared specifications, designs, and artifacts
- `shared/messages/` - Inter-agent message queues

### 2. Message Format

Inter-agent messages follow a standardized JSON format documented in the [Storage Schema Reference](../references/storage-schema.md#message-file-format). Messages include sender/receiver identification, type classification, content body, and metadata for routing and correlation.

### 3. Message Types and Patterns

```rust
pub enum MessageType {
    // Work coordination
    Request,      // Ask another agent for something
    Response,     // Reply to a request
    Handoff,      // Transfer responsibility
    
    // Status updates
    Progress,     // Report progress on a task
    Completed,    // Signal task completion
    Blocked,      // Report being blocked
    
    // Collaboration
    Review,       // Request review of work
    Feedback,     // Provide feedback
    Question,     // Ask for clarification
    
    // Broadcast
    Announcement, // Broadcast to all agents
    Alert,        // Important notification
}

### Message Routing Architecture

The MessageRouter coordinates inter-agent communication through these key capabilities:

**Routing Patterns**:
- **Direct Messaging**: Send to specific agent by ID
- **Role-based Broadcasting**: Send to all agents of a specific role
- **Pattern Matching**: Send to agents matching ID patterns (wildcards)
- **System-wide Broadcasting**: Announce to all agents in session

**Message Delivery**:
- Messages written to recipient agent's inbox directory
- Structured JSON format with timestamp, sender, type, and content
- Automatic validation of sender registration
- Comprehensive logging for debugging and audit
```

### 4. Agent Registry and Discovery

Agents can discover other agents in the session:

```rust
pub struct AgentInfo {
    pub agent_id: String,
    pub role_name: String,
    pub role_description: Option<String>,
    pub instance_number: usize,
    pub execution_state: AgentExecutionState, // Uses unified state model
    pub capabilities: Vec<String>,
}

### Agent Discovery Patterns

Agents can discover other agents through multiple mechanisms:

**Discovery Methods**:
- **By Role**: Find all agents with specific role (e.g., "engineer", "architect")
- **By Capability**: Locate agents with specific capabilities (e.g., "code_review", "database_design")
- **By Pattern**: Match agent IDs using wildcards for flexible queries
- **Session Summary**: Get overview of all active agents and their roles

**Registry Information**: Each agent maintains discoverable information including role, capabilities, execution state, and instance number.
```

### 5. Agent Communication Capabilities

Agents access communication through environment variables and can:
- **Send Messages**: Direct messages to specific agents or broadcast to role groups
- **Receive Messages**: Read from their inbox with automatic message parsing
- **Share Artifacts**: Place specifications, code, and results in shared context directories
- **Broadcast Announcements**: Send system-wide notifications about milestones or status
- **Request Assistance**: Ask for help from agents with specific roles or capabilities

### 6. Status Updates via stdout

Agents report status through structured JSON output:

```json
{"type": "status", "message": "Starting API design", "progress": 0.1}
{"type": "artifact", "path": "shared/context/architecture/api-spec.yaml", "description": "REST API specification"}
{"type": "dependency", "waiting_for": "agent_researcher_001", "reason": "Need database recommendations"}
{"type": "complete", "result": "success", "outputs": ["api-spec.yaml", "system-design.md"]}
```

### 7. Dependency Coordination

Agents coordinate work dependencies through message-based coordination:

**Dependency Patterns**:
- **Completion Notifications**: Agents announce when they finish work that others depend on
- **Blocking Messages**: Agents can signal when they're waiting for specific dependencies
- **Status Polling**: Dependent agents can check completion status of required work
- **Handoff Coordination**: Explicit work transfer between agents with validation

**Coordination Mechanisms**:
- Agents declare dependencies when spawned
- Status updates track completion of dependency requirements
- Message-based notifications for dependency resolution

### 8. Broadcast Patterns

System-wide coordination uses broadcast messaging for:

**Broadcast Types**:
- **Milestone Announcements**: Notify all agents of project milestones or phase completions
- **System Alerts**: Emergency notifications or important status changes
- **Coordination Updates**: Session-wide updates about strategy or priority changes
- **Resource Warnings**: Notifications about system resource limits or constraints

**Broadcast Mechanisms**:
- Orchestrator can send announcements to all agents in session
- Agents can broadcast status updates to entire team when appropriate
- Automatic exclusion of message sender from broadcast recipients

## File Access Coordination

Shared file access coordination prevents conflicts when multiple agents modify artifacts:

**Coordination Strategies**:
- **File Locking**: Acquire exclusive locks for files being modified
- **Lock Ownership**: Track which agent currently holds locks on specific files
- **Lock Timeouts**: Automatic release of locks after timeout to prevent deadlocks
- **Conflict Resolution**: Error handling when agents attempt to access locked files

**Shared Access Patterns**:
- Read-only access to shared specifications and documentation
- Exclusive write access for artifact creation and modification
- Atomic file operations to prevent partial updates

## Consequences

### Positive
- **Simple Integration**: File-based communication works with any tool
- **Persistence**: Messages and artifacts are automatically saved
- **Debugging**: Easy to inspect communication history
- **Flexibility**: Agents can use any language/framework
- **Resilience**: Survives agent crashes
- **Role-Based Routing**: Easy to message all agents of a specific role
- **Pattern Matching**: Flexible targeting with wildcards
- **Agent Discovery**: Agents can find others by role or capability
- **Foundation for Higher-Level Protocols**: Provides infrastructure for orchestration coordination (see ADR-07)

### Negative  
- **Latency**: File system operations slower than memory
- **Polling**: Agents must poll for new messages
- **Cleanup**: Old messages need periodic cleanup
- **Complexity**: Multiple communication channels
- **Role Parsing**: Must extract role from agent IDs

### Mitigation
- Implement file system watchers for real-time updates
- Add message expiration and archival
- Provide client libraries for common languages
- Clear documentation and examples

## References
- **ADR-07: Orchestration Guardrails and Coordination Protocols** - Uses this communication infrastructure for higher-level coordination
- ADR-02: Hybrid Storage Strategy - File system organization for communication
- ADR-03: Session Orchestration and State Management - Session-level message coordination
- Unix philosophy: Everything is a file
- Actor model for message passing
- Enterprise Integration Patterns
- File system notification APIs (inotify, FSEvents)

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*