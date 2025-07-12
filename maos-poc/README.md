# MAOS Proof of Concept

This is a simple POC to demonstrate the core MAOS concepts:

1. **Orchestrator Agent** - Automatically spawned to analyze user requests
2. **Execution Plans** - JSON-structured plans with phases and agents
3. **Parallel Execution** - Agents can work simultaneously when appropriate
4. **Real-time Progress** - Stream output from agents as they work

## Running the POC

```bash
cd maos-poc
cargo run
```

## What It Demonstrates

The POC shows:
- **Real Claude Integration**: Actually spawns Claude CLI for orchestrator and agents
- **Streaming Output**: Real-time progress updates as agents work
- **Parallel Execution**: When in parallel mode, agents run simultaneously
- **Progress Tracking**: Shows line counts and highlights important actions
- **File Creation**: Tracks files created by each agent
- **Timeout Handling**: 5-minute timeout per agent with graceful termination
- **Workspace Management**: Each agent gets its own workspace in `target/tmp/maos-workspace/`
- **Agent Collaboration**: Agents can share work via the shared context directory

## Features

### Real-time Progress
- Shows dots (`.`) as orchestrator processes
- Displays line count every 5 lines for agents
- Highlights lines containing "Creating", "Writing", or "Generated"
- Shows files created by each agent

### Error Handling
- Falls back to a default plan if orchestrator fails
- Captures and reports agent errors
- Implements 5-minute timeout per agent
- Gracefully terminates hanging processes

### Workspace Structure
```
target/tmp/maos-workspace/
├── agents/           # Individual agent workspaces
│   ├── architect_0/
│   ├── engineer_0/
│   └── engineer_1/
├── shared_context/   # Shared files between agents
└── messages/         # Inter-agent communication (future)
```

## Next Steps

In the real implementation:
- MCP server integration for tool exposure
- Better inter-agent communication protocols
- Session persistence and resumption
- More sophisticated orchestration patterns
- Integration with other CLI tools (aider, continue, etc)

This POC proves that real-time orchestration with progress feedback is achievable!