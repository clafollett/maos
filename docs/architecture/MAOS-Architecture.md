# MAOS True Architecture - Rust CLI for Claude Code Orchestration

## What MAOS Actually Is

MAOS (Multi-Agent Orchestration System) is a **high-performance Rust CLI** that enhances Claude Code's native sub-agent capabilities through hook integration and git worktree isolation. It replaces fragile Python scripts with a fast, reliable binary.

> **Development Note**: MAOS is currently in a bootstrap phase, using Python hooks to build the Rust CLI. Once the CLI reaches feature parity, it will replace the Python implementation. The architecture described here represents the target state.

## System Requirements

- **MAOS CLI**: Installed via NPX, Homebrew, or direct download
- **Git**: 2.5+ (for worktree support)
- **Claude Code**: Latest version with hook support
- **No Runtime Dependencies**: Single compiled binary
- **Operating Systems**: 
  - Linux (x86_64, ARM64)
  - macOS (Intel, Apple Silicon)
  - Windows (x86_64)

## Key Principle: MAOS is Invisible

Users never interact with MAOS directly. They simply:
1. Talk to Claude Code normally
2. Claude Code decides when to parallelize work
3. Everything else happens automatically behind the scenes

## The Real Architecture

```
┌──────────────────────────────────────────────────────────┐
│                         USER                             │
│                   (Uses Claude Code normally)            │
└───────────────────────────┬──────────────────────────────┘
                            │ Natural language
                            ▼
┌──────────────────────────────────────────────────────────┐
│                     CLAUDE CODE                          │
│              (Main instance / Orchestrator)              │
│  • Receives user request                                 │
│  • Decides to parallelize work                           │
│  • Uses Task tool to spawn sub-agents                    │
└──────────┬──────────────────┬────────────────────────────┘
           │                  │
           │ settings.json    │ Task tool
           ▼                  ▼
┌─────────────────┐  ┌─────────────────────────────────────┐
│ Hook Commands   │  │          SUB-AGENTS                 │
│                 │  │  • Backend Engineer (in worktree)   │
│ maos pre-tool   │  │  • Frontend Engineer (in worktree)  │
│ maos post-tool  │  │  • QA Engineer (in worktree)        │
│ maos notify     │  └─────────────────────────────────────┘
└────────┬────────┘
         │ Executes
         ▼
┌──────────────────────────────────────────────────────────┐
│                    MAOS CLI (Rust Binary)                │
│  • Security validation (rm -rf, .env protection)         │
│  • Git worktree management                               │
│  • Session coordination                                  │
│  • TTS integration (ElevenLabs, OpenAI, macOS)          │
│  • Lock management                                       │
│  • Performance logging                                   │
└──────────────────────────────────────────────────────────┘
```

## How It Actually Works

### 1. User Request
```
User: "Build me a complete authentication system"
```

### 2. Claude Code Orchestration
Claude (orchestrator role) analyzes the request and decides to parallelize:
```markdown
I'll build this authentication system using multiple specialized agents working in parallel:
- Backend engineer for API
- Frontend engineer for UI
- Security specialist for review
- QA engineer for testing
```

### 3. Hook Configuration (settings.json)
Claude Code's hooks are configured to use MAOS CLI:

```json
{
  "hooks": {
    "PreToolUse": [{
      "command": "maos pre-tool-use"
    }],
    "PostToolUse": [{
      "command": "maos post-tool-use"
    }]
  }
}
```

When the Task tool is used, MAOS CLI automatically:
- Creates isolated git worktree for the agent
- Modifies the agent prompt to include workspace path
- Sets up coordination tracking

### 4. Sub-Agent Spawning
Claude uses native Task tool with MAOS automatically injecting workspace paths into the agent prompts.

### 5. Parallel Work in Isolation
Each agent works in its own git worktree:
```
main/                          # Original repository
worktrees/
├── backend-auth-123/         # Backend engineer workspace
├── frontend-auth-123/        # Frontend engineer workspace
├── security-auth-123/        # Security specialist workspace
└── qa-auth-123/             # QA engineer workspace
```

### 6. Coordination (File-Based)
Agents coordinate through session-scoped files:
```
.maos/
├── active_session.json       # Points to current session
├── sessions/
│   └── {session_id}/        # Per-session coordination
│       ├── session.json     # Session metadata
│       ├── agents.json      # Active agents in session
│       ├── locks.json       # File locks for session
│       ├── progress.json    # Task completion tracking
│       ├── orchestration.json # Agent relationships
│       ├── timeline.json    # Event timeline
│       └── metrics.json     # Performance metrics
└── logs/                    # Audit trail
```

## What MAOS is NOT

1. **NOT a user command** - Users configure hooks once, then forget about MAOS
2. **NOT an agent spawner** - Claude Code's Task tool does that
3. **NOT a complex orchestrator** - Just fast backend utilities
4. **NOT fragile scripts** - Compiled binary for reliability

## MAOS Components

### 1. Rust CLI Binary
A single compiled binary with subcommands:
- **maos pre-tool-use**: Security validation and workspace preparation
- **maos post-tool-use**: State updates and cleanup
- **maos notify**: TTS notifications with provider selection
- **maos stop**: Session completion with optional announcements
- **maos session-info**: Current session status
- **maos worktree-list**: Active workspace listing

### 2. Hook Configuration
Simple settings.json entries replace complex Python scripts:
```json
{
  "hooks": {
    "PreToolUse": [{"command": "maos pre-tool-use"}],
    "PostToolUse": [{"command": "maos post-tool-use"}],
    "Notification": [{"command": "maos notify"}],
    "Stop": [{"command": "maos stop"}]
  }
}
```

### 3. Coordination Files
Simple JSON files in `.maos/`:
- No database needed
- File system is the source of truth
- Atomic operations for consistency

## Example: Complete Flow

1. **User**: "Add password reset feature"

2. **Claude Orchestrator**: 
   - Decides to spawn backend and frontend agents
   - Task tool usage triggers MAOS via hooks

3. **Automatic MAOS Actions**:
   ```bash
   # Called by settings.json hook configuration
   maos pre-tool-use  # Validates and prepares workspaces
   # Creates: worktrees/backend-sess-789/
   # Creates: worktrees/frontend-sess-789/
   ```

4. **Agents Work**:
   - Backend engineer implements API in isolated worktree
   - Frontend engineer builds UI in separate worktree
   - MAOS prevents file conflicts via locks

5. **Completion**:
   - Orchestrator merges work back to main
   - `maos post-tool-use` handles cleanup automatically

## Technical Details

For implementation details, see:
- [Rust CLI Architecture](./rust-cli-architecture.md) - Technical design and crate structure
- [Implementation PRD](./maos-implementation-prd.md) - Requirements and timeline

## Workspace Enforcement Strategy

Since Claude Code hooks cannot intercept and modify tool parameters, MAOS uses a "block-and-educate" pattern:

1. **Detection**: Pre-hook checks if file operations target the assigned workspace
2. **Blocking**: Operations outside workspace are blocked (exit code 2)
3. **Education**: Clear error messages show the correct workspace path
4. **Retry**: Agents learn to use workspace paths through feedback

Example enforcement:
```
❌ BLOCKED: Edit /project/src/main.py
✅ Use instead: /project/worktrees/agent-123/src/main.py
```

This approach achieves workspace isolation within hook architecture constraints.

## Key Design Decisions

1. **Hooks Over Commands**: Users never need to learn MAOS
2. **Files Over Database**: Simple, debuggable, portable
3. **Git Native**: Leverage git's power for isolation
4. **Minimal Dependencies**: Just git and basic scripting
5. **Block-and-Educate**: Enforce isolation through intelligent blocking

## Success Criteria

1. **Invisible to Users**: They just talk to Claude normally
2. **Prevents Conflicts**: Agents don't step on each other
3. **Easy to Debug**: Just look at files and git branches
4. **Fast**: Minimal overhead on operations

## What We're NOT Building

- User-facing commands
- Complex orchestration engine  
- Enterprise features
- Distributed systems
- Agent spawning (Claude does that)

## Summary

MAOS is a high-performance Rust CLI that makes Claude Code's native sub-agents work better through:
1. Fast, compiled binary execution (<10ms)
2. Professional distribution (NPX, Homebrew)
3. Security-first design (rm -rf protection, .env blocking)
4. Git worktree isolation for parallel development
5. Simple hook configuration in settings.json

Users get blazing-fast parallel AI development without learning anything new.