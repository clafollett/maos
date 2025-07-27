# ADR-02: PTY Multiplexer Architecture

## Status
Accepted

## Context
MAOS needs to orchestrate multiple AI agents (Claude CLI instances) reliably across different platforms. Our initial attempts with network-based communication (ACP) and MCP servers revealed fundamental limitations:

- **Timeout Issues**: MCP Inspector enforces 10s initial response and 60s total timeouts
- **Network Complexity**: Port management, discovery, and REST APIs add unnecessary complexity
- **Platform Limitations**: Network approaches require platform-specific handling
- **Process Control**: Indirect control through network protocols is less reliable than direct process management

After studying successful orchestration systems like tmux-orchestrator, we identified that direct process control via pseudo-terminals (PTY) provides the reliability and simplicity we need.

## Decision
MAOS will implement an **Orchestrator-as-PTY-Multiplexer** architecture where a single Rust process directly manages multiple Claude CLI instances through pseudo-terminal interfaces.

### Core Architecture

```
            User Terminal (any)
                      |
                      ▼
┌──────────────────────────────────────────┐
|             MAOS CLI (Rust)              |
|                                          |
|  ┌────────────────────────────────────┐  |
|  |     PTY Multiplexer Core           |  |
|  |                                    |  |
|  |  * Process Lifecycle Manager       |  |
|  |  * Message Router (Hub-Spoke)      |  |
|  |  * Session State Tracker           |  |
|  |  * Output Stream Handler           |  |
|  └────────────────────────────────────┘  |
|                     |                    |
|      ┌──────────────|─────────────┐      |
|      ▼              ▼             ▼      |
|  ┌───────┐      ┌───────┐    ┌────────┐  |
|  |  PTY  |      |  PTY  |    |  PTY   |  |
|  |  Orch |      |Backend|    |Frontend|  |
|  └───────┘      └───────┘    └────────┘  |
└──────────────────────────────────────────┘
       |              |             |
       ▼              ▼             ▼
    Claude         Claude        Claude
    (orch)        (backend)    (frontend)
```

### Key Components

1. **PTY Multiplexer Core**
   - Creates and manages pseudo-terminals for each agent
   - Provides direct I/O control for each Claude process
   - Handles process lifecycle (spawn, monitor, terminate)

2. **Message Router**
   - Implements hub-and-spoke communication pattern
   - All inter-agent messages route through the orchestrator
   - Ensures proper timing for Claude UI (500ms delay)

3. **Session Manager**
   - Tracks Claude CLI `--session-id` for each agent
   - Enables session persistence and resume
   - Maintains agent role assignments

4. **Output Stream Handler**
   - Captures agent output in real-time
   - Provides unified logging and monitoring
   - Enables output streaming to external tools

### Implementation Strategy

```rust
// Core abstraction
pub trait PtyBackend {
    fn spawn_process(&mut self, cmd: &[&str]) -> Result<PtyHandle>;
    fn write(&mut self, data: &[u8]) -> Result<()>;
    fn read(&mut self, timeout: Duration) -> Result<Vec<u8>>;
    fn resize(&mut self, rows: u16, cols: u16) -> Result<()>;
}

// Cross-platform implementation
pub struct PortablePtyBackend; // Uses portable-pty crate

// Optional Unix enhancement
pub struct TmuxBackend; // Leverages tmux for additional features
```

## Consequences

### Positive
- **Reliability**: Direct process control eliminates network timeouts and failures
- **Simplicity**: No ports, no discovery, no REST APIs to maintain
- **Cross-Platform**: Works anywhere portable-pty works (Windows, macOS, Linux)
- **Performance**: Direct I/O is faster than network protocols
- **Debugging**: Can inspect PTY output directly, similar to tmux
- **Compatibility**: Works in any terminal environment (SSH, Docker, CI/CD)

### Negative
- **Learning Curve**: PTY concepts may be unfamiliar to some developers
- **Platform Differences**: PTY behavior varies slightly between OS versions
- **Binary Size**: Rust binary larger than simple scripts
- **Testing Complexity**: PTY interactions harder to unit test than REST APIs

### Neutral
- **Architecture Shift**: Moving from distributed to centralized control
- **Tool Choice**: Rust provides safety and performance for system programming
- **Dependency**: Relies on portable-pty crate for cross-platform support

## References
- [portable-pty documentation](https://docs.rs/portable-pty/)
- [tmux-orchestrator](https://github.com/Jedward23/Tmux-Orchestrator) - Inspiration for PTY-based orchestration
- Issue #5: Architectural Pivot from ACP to PTY Multiplexer