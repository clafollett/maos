# ADR-05: PTY-Based Agent Communication Patterns

## Status
Accepted (Supersedes ACP approach)

## Context
Agents in MAOS need to communicate and coordinate their work effectively. Since each agent runs as a separate Claude CLI process, we need well-defined patterns for:

- Sending messages between agents
- Coordinating task handoffs
- Sharing context and results
- Broadcasting status updates
- Maintaining conversation flow
- Ensuring message delivery

### Previous Approaches
1. **File-Based Messaging**: Had latency issues and no real-time communication
2. **ACP Network Protocol**: Added unnecessary complexity with REST APIs and port management
3. **MCP Tool Integration**: Hit timeout limitations (10s/60s) that broke long-running operations

### PTY Solution
Direct PTY (pseudo-terminal) control provides immediate, reliable communication through the same interface humans use - the terminal. This approach is inspired by tmux-orchestrator's proven success.

## Decision
We will implement a **hub-and-spoke communication pattern** where the MAOS PTY Multiplexer acts as the central router for all agent messages. All inter-agent communication flows through the multiplexer using PTY I/O.

### Communication Architecture

```
                    User Terminal
                         │
                         ▼
┌────────────────────────────────────────────┐
│            MAOS CLI (PTY Multiplexer)      │
│                                            │
│          ┌──────────────────┐              │
│          │   Message Router  │              │
│          │   (Hub & Spoke)   │              │
│          └────────┬─────────┘              │
│                  │                         │
│     ┌────────────┼────────────┐           │
│     ▼            ▼            ▼           │
│  ┌──────┐    ┌──────┐    ┌──────┐        │
│  │ PTY  │    │ PTY  │    │ PTY  │        │
│  │Orch. │    │Backend│    │Front │        │
│  └───┬──┘    └───┬──┘    └───┬──┘        │
└──────┼───────────┼───────────┼────────────┘
       │           │           │
       ▼           ▼           ▼
    Claude      Claude      Claude
    (orch)     (backend)   (frontend)
```

### Communication Patterns

1. **Direct Messaging**
   ```
   User → CLI: maos message backend "implement auth endpoint"
   CLI → PTY: [writes to backend PTY with proper timing]
   Backend → CLI: [output captured from PTY]
   ```

2. **Orchestrator Routing**
   ```
   Orchestrator: "Backend, please implement the API"
   CLI → Backend PTY: [routes message from orchestrator]
   Backend: "Working on it..."
   CLI → Orchestrator PTY: [routes response back]
   ```

3. **Broadcast Messages**
   ```
   User → CLI: maos broadcast "switching to phase 2"
   CLI → All PTYs: [sends to all active agents]
   ```

### Key Design Principles

1. **Hub-and-Spoke Pattern**: All messages route through the central multiplexer
2. **Direct PTY Control**: No network protocols, just terminal I/O
3. **Timing Awareness**: Critical 500ms delay for Claude UI registration
4. **Session Continuity**: Claude CLI's --session-id preserves context
5. **Output Capture**: Full visibility into agent responses
6. **Message History**: Multiplexer maintains audit trail
7. **Error Handling**: Direct detection of process failures
8. **Extensibility**: Easy to add new agent types

## Message Implementation

### 1. Message Timing
The most critical aspect is the 500ms delay between sending the message and pressing Enter:

```rust
pub async fn send_message(pty: &mut PtyHandle, message: &str) -> Result<()> {
    // Send the message text
    pty.write(message.as_bytes())?;
    
    // CRITICAL: Wait for Claude UI to register the text
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Send Enter key
    pty.write(b"\r")?;
    
    Ok(())
}
```

### 2. Message Types

**Task Assignment**
```
maos message backend "Please implement the user authentication API with the following requirements:
- JWT token-based auth
- Email/password login
- Session management
- Password reset flow"
```

**Status Check**
```
maos message backend "STATUS: What's your current progress?"
```

**Cross-Agent Coordination**
```
# Orchestrator to Backend
"Backend, the Frontend team needs the API endpoint documentation"

# Orchestrator to Frontend  
"Frontend, Backend has completed the auth API at /api/v1/auth"
```

**Phase Transitions**
```
maos broadcast "Phase 1 complete. Moving to integration testing phase."
```

### 3. Session Management

Each agent maintains context through Claude's session system:

```rust
pub struct AgentSession {
    pub id: String,
    pub role: AgentRole,
    pub claude_session_id: String,  // For --session-id
    pub pty: PtyHandle,
    pub created_at: DateTime<Utc>,
    pub message_history: Vec<Message>,
}
```

### 4. Output Handling

The multiplexer captures all agent output for:
- **Real-time monitoring**: Stream to user terminal
- **Message routing**: Parse for inter-agent messages
- **Audit trail**: Complete conversation history
- **Error detection**: Process crashes or issues

### 5. Coordination Patterns

**Sequential Execution**
```
Orchestrator → Backend: "Create the database schema"
[waits for completion]
Orchestrator → Backend: "Now implement the API endpoints"
```

**Parallel Execution**
```
Orchestrator → Backend: "Build the API"
Orchestrator → Frontend: "Start on the UI components"
Orchestrator → QA: "Prepare test scenarios"
[all work simultaneously]
```

**Dependency Handling**
```
Orchestrator monitors output and coordinates:
- Frontend waiting for API? Route endpoint info when ready
- Backend needs requirements? Forward from Business Analyst
- QA needs deployment? Coordinate with DevOps
```

## Consequences

### Positive
- **Simplicity**: Direct PTY control eliminates network complexity
- **Reliability**: No timeouts, no port conflicts, no network failures
- **Visibility**: Complete audit trail of all communications
- **Performance**: Direct I/O faster than network protocols
- **Debugging**: Can observe agent conversations like tmux
- **Session Continuity**: Claude's --session-id works perfectly
- **Cross-Platform**: Works anywhere PTYs work

### Negative
- **Centralized Control**: All messages must route through multiplexer
- **No Direct Agent Communication**: Agents can't talk without orchestrator
- **Learning Curve**: PTY concepts less familiar than REST APIs

### Neutral
- **Different from Industry Standards**: Not using typical microservice patterns
- **Inspired by tmux-orchestrator**: Proven approach but adapted for our needs

## Implementation Notes

### Critical Success Factors
1. **Message Timing**: The 500ms delay is non-negotiable for Claude UI
2. **Output Parsing**: Must handle ANSI codes and partial outputs
3. **Process Monitoring**: Detect crashes and hung processes
4. **Session Persistence**: Always use --session-id for context

### Example Implementation
```rust
impl MessageRouter {
    pub async fn route_message(&mut self, from: &str, to: &str, msg: &str) -> Result<()> {
        // Find target agent
        let target = self.agents.get_mut(to)
            .ok_or_else(|| anyhow!("Agent {} not found", to))?;
        
        // Format message with sender context
        let formatted = format!("[From {}]: {}", from, msg);
        
        // Send via PTY with proper timing
        send_message(&mut target.pty, &formatted).await?;
        
        // Log for audit trail
        self.log_message(from, to, msg);
        
        Ok(())
    }
}
```

## References
- [tmux-orchestrator](https://github.com/Jedward23/Tmux-Orchestrator) - Inspiration for PTY-based communication
- ADR-02: PTY Multiplexer Architecture - Core PTY design
- ADR-03: Terminal-Agnostic Design - Terminal compatibility
- Issue #5: Architectural pivot from ACP to PTY