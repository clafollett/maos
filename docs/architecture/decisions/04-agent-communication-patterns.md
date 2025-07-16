# ADR-04: Orchestrator-as-PTY-Multiplexer Communication Patterns

## Status
Accepted

## Context
Agents in MAOS need to communicate and coordinate their work effectively. Since each agent runs as a separate Claude CLI process, we need well-defined patterns for:

- Sharing work artifacts (specifications, code, test results)
- Sending messages between agents via central coordination
- Coordinating dependencies and handoffs
- Broadcasting status updates through orchestrator
- Requesting help or clarification from other agents
- Maintaining clean audit trails of all communications

### Previous Approaches

**File-Based Messaging:**
Our initial approach used file-based messaging with shared directories, but this had critical limitations:
- **Latency**: File system polling delays
- **No real-time communication**: Agents had to poll for messages
- **Scalability issues**: File system as message queue doesn't scale
- **No standardized format**: Ad-hoc message structures
- **Discovery problems**: Complex agent discovery mechanisms

**Network-Based Approaches:**
We considered peer-to-peer networking approaches but these introduced significant complexity:
- **Network management**: Port allocation, discovery, health monitoring
- **Cross-platform issues**: Network stack differences across OS
- **Debugging complexity**: Distributed logs across multiple servers
- **Deployment overhead**: Multiple embedded servers per orchestration

### PTY Multiplexer Solution
Inspired by Tmux-Orchestrator's proven architecture, we implement a **hub-and-spoke communication model** where the **Orchestrator Agent acts as a PTY multiplexer**, managing multiple Claude CLI processes and routing all inter-agent communication.

## Decision
We will implement **Orchestrator-as-PTY-Multiplexer** where the Orchestrator Agent owns and manages multiple Claude CLI processes via PTY, providing centralized communication routing, audit trails, and process lifecycle management.

### Architecture: Orchestrator-as-PTY-Multiplexer

```
┌─────────────────────────────────────────────────────────────────┐
│                    Claude Code (MCP Client)                     │
│                                                                 │
│  maos/orchestrate ──► Start orchestration session              │
│  maos/session-status ──► Monitor progress                      │
│  maos/list-roles ──► List available agent roles                │
└─────────────────────┬───────────────────────────────────────────┘
                      │ MCP Protocol
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                     MAOS MCP Server                            │
│                                                                 │
│  • Spawns Orchestrator Agent (PTY Multiplexer)                 │
│  • Streams unified output from Orchestrator only               │
│  • Tracks high-level session state                             │
│  • No direct agent communication tools                         │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Spawns Orchestrator process
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│               Orchestrator Agent (PTY Multiplexer)             │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                Agent Registry & Message Router          │   │
│  │  • agent_backend_engineer_1_abc123 → session_xyz789    │   │
│  │  • agent_frontend_engineer_1_def456 → session_mno345   │   │  
│  │  • agent_solution_architect_1_ghi789 → session_pqr012  │   │
│  │  • Centralized message routing and audit logging       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                            │                                   │
│            Hub-and-Spoke Communication                         │
│                            │                                   │
│    ┌───────────────────────┼───────────────────────┐           │
│    ▼                       ▼                       ▼           │
│ ┌─────────────┐     ┌─────────────┐     ┌─────────────┐       │
│ │ Backend     │     │ Frontend    │     │ Solution    │       │
│ │ Engineer    │     │ Engineer    │     │ Architect   │       │
│ │ (Claude CLI)│     │ (Claude CLI)│     │ (Claude CLI)│       │
│ │ PTY Control │     │ PTY Control │     │ PTY Control │       │
│ └─────────────┘     └─────────────┘     └─────────────┘       │
│                                                               │
│  • Orchestrator controls all agents via PTY read/write       │
│  • All inter-agent messages route through Orchestrator       │
│  • Centralized audit trail and logging                       │
│  • Cross-platform compatibility (portable-pty)              │
│  • Session persistence via Claude CLI --session-id           │
└─────────────────────────────────────────────────────────────────┘
```

### Core PTY Multiplexer Principles

1. **Hub-and-Spoke Communication**: All agent messages route through Orchestrator
2. **Single Process Model**: Each agent is just a Claude CLI process (no embedded servers)
3. **PTY Process Control**: Direct stdin/stdout/stderr control via portable-pty
4. **Centralized Audit**: Complete message log and communication history
5. **Session Persistence**: Leverage Claude CLI `--session-id` for memory continuity
6. **Cross-Platform**: Works on Windows, macOS, Linux without dependencies
7. **Simplified Deployment**: No network configuration or port management
8. **Isolation by Default**: Agents work in focused isolation unless coordination needed

## PTY Multiplexer Communication Architecture

### 1. Orchestrator Agent Process Management

The Orchestrator Agent manages multiple Claude CLI processes using portable-pty:

**Agent Lifecycle:**
- **Spawn**: Create PTY pair and launch Claude CLI process
- **Initialize**: Send role briefing and establish session ID
- **Register**: Record agent ID → Claude session ID mapping
- **Communicate**: Send/receive messages via PTY read/write
- **Monitor**: Track agent status and health
- **Sleep/Wake**: Preserve session context across activation cycles
- **Terminate**: Clean shutdown with session preservation

**PTY Integration:**
- Each agent gets dedicated PTY pair (master/slave)
- Orchestrator controls master side (read/write)
- Claude CLI process connected to slave side
- Non-blocking I/O for concurrent agent management
- Message parsing and routing through Orchestrator

### 2. Agent Registry and Session Management

**Agent ID Structure**: `agent_{role}_{instance}_{unique_id}`
- Example: `agent_backend_engineer_1_abc123`
- Supports multiple instances of same role
- Unique identifier prevents collisions

**Session Binding Registry:**
```
Agent Registry Mapping:
┌─────────────────────────────────────────────────────────────┐
│ agent_id                    → claude_session_id → status   │
├─────────────────────────────────────────────────────────────┤
│ agent_backend_engineer_1    → session_xyz789    → [ACTIVE] │
│ agent_frontend_engineer_1   → session_mno345    → [ACTIVE] │  
│ agent_solution_architect_1  → session_pqr012    → [SLEEP]  │
│ agent_backend_engineer_2    → session_stu678    → [ACTIVE] │
└─────────────────────────────────────────────────────────────┘
```

**Perfect Context Continuity:**
- Agents bound to Claude CLI session IDs never lose memory
- Sleep/wake cycles preserve full conversation history
- Multiple sessions can work on different aspects simultaneously
- Session persistence across orchestrator restarts

### 3. Message Types and Communication Patterns

Communication follows **minimal, essential-only** coordination patterns:

**Essential Communication Triggers:**
- **Task Assignment**: Orchestrator assigns work to specific agents
- **Completion Status**: Agents report finished work to Orchestrator
- **Inter-Agent Handoffs**: Coordinated work transfer via Orchestrator
- **Assistance Requests**: Agents request help from other specialists
- **Status Inquiries**: Orchestrator checks agent progress
- **Direction Changes**: Orchestrator redirects agent focus
- **Error Resolution**: Agents report blocking issues
- **Phase Transitions**: Orchestration phase changes

**Message Categories:**
- **Direct Commands**: Orchestrator → Agent task assignments
- **Status Reports**: Agent → Orchestrator progress updates  
- **Coordination Messages**: Agent → Orchestrator → Agent handoffs
- **Broadcast Updates**: Orchestrator → All Agents announcements
- **Assistance Flows**: Agent → Orchestrator → Specialist Agent

**No Chatter Principle**: Messages sent only when **essential for coordination**, not for status broadcasts or social communication.

### 4. Hub-and-Spoke Message Routing

**Central Message Router in Orchestrator:**

All agent communication flows through the Orchestrator's message routing system:

**Inbound Message Processing:**
- Parse messages from agent PTY output
- Identify message type and target recipient
- Log all communications for audit trail
- Route to appropriate destination

**Outbound Message Delivery:**
- Format messages for target agent role/context
- Send via PTY write to specific agent
- Confirm delivery and log transmission
- Handle delivery failures and retries

**Message Flow Examples:**

*Task Assignment:*
```
User Request → MCP Server → Orchestrator → Backend Engineer
                                     ↓
                               [Message Router]
                                     ↓
                     "Please implement auth API endpoints 
                      per specification in shared/specs/auth.md"
```

*Inter-Agent Coordination:*
```
Backend Engineer → Orchestrator → Frontend Engineer
     ↓                ↓                ↓
"Auth API ready"  [Route Message]  "Backend APIs available
                                   at localhost:8080/api/"
```

*Assistance Request:*
```
Frontend Engineer → Orchestrator → Solution Architect
        ↓               ↓               ↓
   "Need guidance   [Route &        "Please review frontend
    on state mgmt"   Log]           architecture questions"
```

### 5. Communication Protocol Format

**Structured Message Format:**
Messages between Orchestrator and Agents follow consistent formatting:

**Task Assignment Messages:**
```
TASK ASSIGNMENT from Orchestrator:
Objective: [Clear, specific goal]
Context: [Relevant background and constraints] 
Resources: [Available specs, files, dependencies]
Success Criteria: [Measurable completion requirements]
```

**Status Update Messages:**
```
STATUS UPDATE to Orchestrator:
Progress: [Current work and completion percentage]
Completed: [Finished deliverables]
Next: [Planned immediate work]
Blockers: [Issues requiring assistance or coordination]
```

**Inter-Agent Coordination Messages:**
```
COORDINATION MESSAGE from [Source Agent]:
Context: [Why this communication is needed]
Information: [Specific details, decisions, or deliverables]
Next Steps: [What the receiving agent should do]
```

### 6. Shared Context Integration

**File-Based Artifact Sharing:**
- Agents continue to share specifications, code, and results via shared file system
- PTY messages announce artifact creation/updates for awareness
- Combination of real-time messaging + persistent artifacts
- Orchestrator tracks artifact dependencies and notifications

**Artifact Notification Flow:**
```
Agent creates/updates file → Notifies Orchestrator → Broadcasts to relevant agents
```

**Workspace Structure Maintained:**
```
shared/
├── context/           # Shared specifications and designs
├── artifacts/         # Generated code and deliverables  
└── logs/             # Centralized communication audit trail
```

### 7. Process Lifecycle and Health Monitoring

**Agent Health Monitoring:**
- Orchestrator periodically checks PTY process health
- Monitors Claude CLI responsiveness via ping/status commands
- Detects hung or crashed processes
- Automatic restart with session restoration when possible

**Graceful Shutdown:**
- Orchestrator coordinates clean agent shutdown
- Preserves Claude session state for future restoration
- Logs final status and deliverables
- Cleans up PTY resources

**Error Recovery:**
- Failed agent processes restart with preserved session ID
- Communication history maintained across failures  
- Orchestrator maintains continuity during agent issues
- Fallback to file-based communication if PTY fails

## Consequences

### Positive
- **Simplified Architecture**: Single multiplexer vs distributed network
- **Cross-Platform Compatibility**: Works on Windows, macOS, Linux without dependencies
- **Centralized Audit Trail**: Complete communication history in one place
- **Easier Debugging**: All messages flow through central router
- **No Network Complexity**: No port management, discovery, or network configuration
- **Direct Process Control**: PTY provides reliable stdin/stdout/stderr access
- **Session Persistence**: Leverage Claude CLI's native session management
- **Proven Pattern**: Based on successful Tmux-Orchestrator architecture
- **Enterprise Ready**: Centralized logging, monitoring, and control
- **IDE Integration Potential**: PTY works in any environment
- **Faster Development**: Less complexity means faster implementation
- **Better Reliability**: Fewer moving parts and failure modes

### Negative
- **Central Point of Coordination**: Orchestrator handles all message routing
- **PTY Dependency**: Requires portable-pty for cross-platform compatibility
- **Process Management Complexity**: Managing multiple Claude CLI processes
- **Memory Usage**: Multiple Claude CLI processes consume more memory

### Mitigation
- **Orchestrator Resilience**: Robust error handling and recovery in Orchestrator
- **PTY Abstraction**: portable-pty handles cross-platform PTY differences
- **Process Monitoring**: Health checks and automatic restart capabilities
- **Resource Management**: Configurable limits on concurrent agents and memory usage
- **Graceful Degradation**: Fallback mechanisms for communication failures

## References
- [Tmux-Orchestrator](https://github.com/Jedward23/Tmux-Orchestrator) - Inspiration for hub-and-spoke architecture
- [portable-pty](https://docs.rs/portable-pty/) - Cross-platform PTY implementation
- **ADR-08: Agent Lifecycle and Management** - PTY-based process lifecycle
- **ADR-10: MCP Server Architecture** - Simplified Orchestrator-only streaming
- **ADR-11: Adaptive Phase-Based Orchestration** - PTY multiplexer coordination
- Process management patterns in distributed systems
- Terminal multiplexer design principles
- Hub-and-spoke communication architectures

---
*Date: 2025-07-16*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*