# MAOS - Multi-Agent Orchestration System

**Backend orchestration for Claude Code's native sub-agent capabilities through hooks and git worktree isolation.**

## What is MAOS?

MAOS (Multi-Agent Orchestration System) is a lightweight backend system that enhances Claude Code's ability to work with multiple AI agents in parallel. It operates entirely through Claude Code's hook system, providing automatic workspace isolation and coordination for sub-agents.

**Key Point**: MAOS is invisible to end users. They simply use Claude Code normally, and MAOS handles the backend orchestration automatically.

## How It Works

1. **User** talks to Claude Code: *"Build me a complete authentication system"*
2. **Claude Code** decides to use multiple agents for parallel work
3. **MAOS hooks** automatically:
   - Create isolated git worktrees for each agent
   - Prevent file conflicts between agents
   - Track progress and coordinate work
   - Clean up when complete
4. **Results** are seamlessly integrated back

No new commands to learn. No complex setup. Just better parallel AI development.

## Architecture

```
User → Claude Code → Hooks → Backend Orchestration
                       ↓
              ┌────────┴────────┐
              │   MAOS Hooks    │
              │ • pre_tool_use  │
              │ • post_tool_use │
              └────────┬────────┘
                       ↓
         ┌─────────────┴─────────────┐
         │     Git Worktrees         │
         │ • backend-engineer/       │
         │ • frontend-engineer/      │
         │ • qa-engineer/           │
         └───────────────────────────┘
```

## For MAOS Developers

If you're contributing to MAOS itself:

### Prerequisites

- Python 3.8+
- Git 2.5+ (for worktree support)
- Claude Code installed

### Development Setup

```bash
# Clone the repository
git clone https://github.com/clafollett/maos.git
cd maos

# Set up hooks (make them executable)
chmod +x .claude/hooks/*.py

# Test the setup
just check-hooks
```

### Testing Hooks

To test MAOS hooks with Claude Code:

1. Make a request that would spawn multiple agents
2. Check that worktrees are created automatically
3. Monitor coordination files in `.maos/`

```bash
# Debug commands
just worktree-list    # List active worktrees
just session-info     # Show current session
just watch-maos       # Monitor coordination files
```

## Key Features

### 🔒 **Automatic Isolation**
Each agent works in its own git worktree, preventing conflicts and enabling true parallel development.

### 📁 **File-Based Coordination**
Simple JSON files track agent state, locks, and progress. No database needed.

### 🪝 **Hook-Based Design**
Integrates seamlessly with Claude Code's existing hook system. No new APIs to learn.

### 🐍 **Python Only**
Pure Python implementation avoids shell script permission issues.

### 👻 **Invisible to Users**
Users just talk to Claude normally. MAOS works behind the scenes.

## Documentation

### Core Architecture
- **[True Architecture](docs/architecture/MAOS-True-Architecture.md)**: How MAOS really works
- **[Implementation Guide](docs/architecture/MAOS-Implementation-Guide.md)**: Step-by-step implementation
- **[Worktree System](docs/guides/worktree-quick-start.md)**: Git worktree management details

### Research & Design
- **[Hook-Based Orchestration](docs/architecture/research/hook-based-orchestration.md)**: Hook system design
- **[Git Worktree Integration](docs/architecture/research/git-worktree-integration-design.md)**: Worktree patterns
- **[Local Orchestration Patterns](docs/architecture/research/local-orchestration-patterns.md)**: Coordination strategies

## Implementation Status

### ✅ Completed
- Architecture documentation
- Hook system design
- Worktree management patterns
- File-based coordination design

### 🚧 In Progress
- Basic hook implementation
- MAOSBackend Python class
- Coordination file management

### 📋 Next Steps
1. Implement pre_tool_use.py hook for Task interception
2. Create MAOSBackend for worktree management
3. Add coordination file tracking
4. Test with real Claude Code workflows

## Why MAOS?

### **Prevent Agent Conflicts**
When Claude Code spawns multiple agents, they might edit the same files simultaneously. MAOS prevents this through automatic worktree isolation and file locking.

### **Enable True Parallelism**
Each agent gets its own complete workspace (git worktree), allowing them to work on different features without stepping on each other.

### **Zero User Friction**
Users don't need to learn new commands or change their workflow. MAOS operates invisibly through Claude Code's hook system.

### **Simple & Reliable**
No complex infrastructure. Just Python scripts, git commands, and JSON files. Easy to debug and understand.

## Project Structure

```
maos/
├── .claude/
│   └── hooks/                    # Claude Code hooks
│       ├── pre_tool_use.py       # Intercepts operations
│       ├── post_tool_use.py      # Cleanup and tracking
│       └── utils/                # Backend utilities
│           └── maos_backend.py   # Core orchestration logic
├── docs/
│   └── architecture/             # Technical documentation
├── worktrees/                    # Auto-created agent workspaces
└── .maos/                        # Coordination files
    ├── session.json
    └── coordination/
        ├── agents.json
        ├── locks.json
        └── progress.json
```

## Contributing

MAOS is open source and welcomes contributions. Key principles:

1. **Keep it invisible** - Users should never know MAOS exists
2. **Python only** - No shell scripts that require chmod
3. **Simple is better** - File-based coordination, no databases
4. **Hook-first design** - Everything happens through Claude Code hooks

## License

MIT License

---

*MAOS: Making Claude Code's sub-agents work better together, invisibly.*