# Claude Code Agent Design - The ACP Revolution

## Overview

A **Claude Code Agent** is an ACP server that manages multiple Claude CLI processes with different roles and contexts. This design is modular - we can easily add Gemini Agent, Codex Agent, or any other CLI-based AI agent with minimal effort!

## Revolutionary Modular Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Claude Code (MCP Client)                 │
└─────────────────────┬───────────────────────────────────────┘
                      │ MCP Protocol
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                    MAOS MCP Server                          │
└─────────────────────┬───────────────────────────────────────┘
                      │ Spawns
                      ▼
┌─────────────────────────────────────────────────────────────┐
│           Orchestrator (Router Agent) - ACP Server          │
│  • Analyzes tasks                                           │
│  • Determines required roles & agent types                  │
│  • Routes to appropriate agent (Claude/Gemini/Codex)        │
│  • Tracks all agent sessions                                │
└─────────┬────────────────────┬────────────────────┬─────────┘
          │ ACP Requests        │                    │
          ▼                     ▼                    ▼
┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
│ Claude Code     │   │ Gemini         │   │ Codex          │
│ Agent           │   │ Agent          │   │ Agent          │
│ (ACP Server)    │   │ (ACP Server)   │   │ (ACP Server)   │
│                 │   │                 │   │                 │
│ Manages:        │   │ Manages:        │   │ Manages:        │
│ • claude CLI    │   │ • gemini CLI    │   │ • codex CLI    │
│ • Sessions      │   │ • Sessions      │   │ • Sessions      │
│ • Roles         │   │ • Contexts      │   │ • Tasks        │
└─────────────────┘   └─────────────────┘   └─────────────────┘
```

## The Claude Code Agent

### Core Responsibilities
- **Process Management**: Spawns and manages `claude` CLI processes
- **Session Tracking**: Maintains Claude session IDs for context continuity
- **Role Assignment**: Dynamically assigns roles via `-p` flag
- **Status Reporting**: Reports run status via ACP protocol
- **Output Streaming**: Streams Claude output via Server-Sent Events

### ACP Endpoints
```
GET  /agents           → Returns Claude Code Agent capabilities
POST /runs             → Start new Claude process with role
GET  /runs/{id}        → Get run status and results
GET  /runs/{id}/stream → Stream real-time output
POST /runs/{id}/resume → Continue with additional input
```

### Agent Manifest
```json
{
  "name": "claude-code",
  "description": "Manages Claude CLI processes for various roles",
  "version": "1.0.0",
  "capabilities": [
    "solution_architect",
    "backend_engineer", 
    "frontend_engineer",
    "qa_engineer",
    "researcher",
    "documenter"
  ],
  "stateful": true,
  "supports_sessions": true,
  "input_content_types": ["application/json"],
  "output_content_types": ["text/plain", "application/json"]
}
```

## Request/Response Flow

### Start New Claude Agent
```json
POST /runs
{
  "agent_name": "claude-code",
  "input": [{
    "role": "user",
    "parts": [{
      "content": {
        "agent_id": "architect_1",
        "agent_role": "solution_architect",
        "session_id": null,  // New session
        "task": "Design authentication system",
        "context": {
          "project_type": "web_app",
          "requirements": ["OAuth2", "MFA"]
        }
      },
      "content_type": "application/json"
    }]
  }]
}

Response:
{
  "run_id": "run_abc123",
  "status": "running",
  "metadata": {
    "agent_id": "architect_1",
    "session_id": "sess_def456",  // New Claude session created
    "process_pid": 12345
  }
}
```

### Continue Existing Session
```json
POST /runs
{
  "agent_name": "claude-code",
  "input": [{
    "role": "user", 
    "parts": [{
      "content": {
        "agent_id": "architect_1",
        "agent_role": "solution_architect",
        "session_id": "sess_def456",  // Continue previous work
        "task": "Refine the authentication design for mobile",
        "context": {
          "previous_work": "run_abc123"
        }
      },
      "content_type": "application/json"
    }]
  }]
}
```

## Implementation Details

### Rust Structure
```rust
// Core trait all AI agents implement
trait AiAgent: Send + Sync {
    fn manifest(&self) -> AgentManifest;
    async fn start_run(&self, request: RunRequest) -> Result<RunInfo>;
    async fn get_run_status(&self, run_id: &str) -> Result<RunStatus>;
    async fn stream_output(&self, run_id: &str) -> Result<EventStream>;
}

// Claude Code Agent implementation
struct ClaudeCodeAgent {
    active_runs: Arc<RwLock<HashMap<String, ClaudeRun>>>,
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    config: ClaudeConfig,
}

struct ClaudeRun {
    run_id: String,
    agent_id: String,
    session_id: String,
    process: Child,
    status: RunStatus,
    output_buffer: Vec<u8>,
}

impl AiAgent for ClaudeCodeAgent {
    async fn start_run(&self, request: RunRequest) -> Result<RunInfo> {
        let params = parse_claude_params(&request)?;
        
        // Build command
        let mut cmd = Command::new("claude");
        cmd.arg("-p").arg(&params.agent_role);
        
        if let Some(session_id) = &params.session_id {
            cmd.arg("--session-id").arg(session_id);
        }
        
        // Spawn process
        let process = cmd.spawn()?;
        
        // Track run
        let run = ClaudeRun {
            run_id: Uuid::new_v4().to_string(),
            agent_id: params.agent_id.clone(),
            session_id: params.session_id.unwrap_or_else(generate_session_id),
            process,
            status: RunStatus::Running,
            output_buffer: Vec::new(),
        };
        
        self.active_runs.write().await.insert(run.run_id.clone(), run);
        
        Ok(RunInfo {
            run_id: run.run_id,
            session_id: run.session_id,
            status: RunStatus::Running,
        })
    }
}
```

## Why This Design Wins

### 1. Modular & Extensible
```rust
// Adding a new AI agent is trivial
struct GeminiAgent { /* ... */ }
impl AiAgent for GeminiAgent { /* ... */ }

// Register with orchestrator
orchestrator.register_agent("gemini", Box::new(GeminiAgent::new()));
```

### 2. Clean Separation
- **Orchestrator**: Business logic and routing
- **Claude Code Agent**: Claude-specific process management
- **Gemini Agent**: Gemini-specific handling
- **Future Agents**: Just implement the trait!

### 3. Perfect Session Management
- Each agent type manages its own session semantics
- Claude uses `--session-id` for context
- Gemini might use different flags
- Orchestrator doesn't need to know the details!

### 4. Resource Efficiency
- One ACP server per agent type (not per role!)
- Processes spawned only when needed
- Clean process lifecycle management

## The ACP Train is Rolling! 🚂

This architecture fully embraces ACP's strengths:
- **Stateful agents** with session management
- **Async execution** with status polling
- **Streaming outputs** via SSE
- **Rich metadata** for discovery
- **Modular design** for future growth

Next steps:
1. Implement Claude Code Agent with full ACP compliance
2. Create integration tests with Orchestrator
3. Add Gemini Agent when ready
4. Watch MAOS revolutionize multi-agent orchestration!

## Conclusion

The Claude Code Agent design is the perfect balance of:
- **Simplicity**: One agent type per CLI tool
- **Flexibility**: Easy to add new AI providers
- **Power**: Full session management and streaming
- **Compliance**: 100% ACP-compliant implementation

This is THE architecture that will make MAOS legendary!