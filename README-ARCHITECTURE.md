# MAOS Architecture Summary

## What is MAOS?

MAOS (Multi-Agent Orchestration System) is a **backend utility** that enhances Claude Code's native sub-agent capabilities through hooks and git worktree isolation. 

**Key Point**: Users never interact with MAOS directly - it works invisibly in the background.

## How Users Experience It

1. User talks to Claude Code normally: *"Build me a REST API with authentication"*
2. Claude decides to use multiple agents for parallel development
3. Agents automatically work in isolated environments without conflicts
4. Results are seamlessly integrated back

That's it. No new commands to learn, no complex setup.

## Architecture Components

### 1. Claude Code (Orchestrator)
- Receives user requests
- Decides when to parallelize work  
- Spawns sub-agents using native Task tool
- Manages overall workflow

### 2. MAOS Hooks
Located in `.claude/hooks/`:
- **pre_tool_use.py**: Intercepts Task tool to prepare workspaces
- **post_tool_use.py**: Cleans up and tracks progress
- Prevents file conflicts between agents
- Completely automatic

### 3. Backend Utilities  
Python modules in `.claude/hooks/utils/` (not user-facing):
- Create/remove git worktrees
- Manage coordination files
- Track agent progress
- Clean up when done
- No shell scripts = no chmod issues

### 4. Git Worktrees
Each agent works in isolation:
```
main/                    # Original repository
worktrees/
├── backend-123/        # Backend engineer workspace
├── frontend-123/       # Frontend engineer workspace
└── qa-123/            # QA engineer workspace
```

## File Structure

```
your-project/
├── .claude/
│   ├── agents/         # Agent definitions (built-in)
│   └── hooks/          # MAOS hooks (our addition)
├── .maos/              # Coordination files (auto-created)
│   ├── session.json
│   └── coordination/
└── worktrees/          # Agent workspaces (auto-managed)
```

## Key Design Principles

1. **Invisible to Users**: No user-facing commands or interfaces
2. **Leverages Native Features**: Uses Claude Code's existing capabilities
3. **Simple Implementation**: ~200 lines of hook code total
4. **Zero Configuration**: Works out of the box

## What MAOS is NOT

- ❌ NOT a CLI tool for users
- ❌ NOT a complex orchestration framework
- ❌ NOT an agent platform
- ❌ NOT enterprise software

It's just glue that makes parallel agents work better.

## Implementation Status

### ✅ Completed
- Agent definitions for 22 specialized roles
- Hook system design and examples
- Git worktree patterns documentation
- Architecture documentation

### 🚧 In Progress  
- Basic hook implementation
- Backend utility scripts
- Testing with real Claude Code sessions

### 📋 TODO
- Automatic cleanup mechanisms
- Progress visualization
- Performance optimizations

## Getting Started

1. Copy the hooks to `.claude/hooks/`
2. Make them executable: `chmod +x .claude/hooks/*.py`
3. Use Claude Code normally - MAOS works automatically

## More Information

- [True Architecture](docs/architecture/MAOS-True-Architecture.md) - Detailed architecture
- [Implementation Guide](docs/architecture/MAOS-Implementation-Guide.md) - Step-by-step setup
- [Hook Orchestration](docs/architecture/research/hook-based-orchestration.md) - Hook patterns

## Summary

MAOS makes Claude Code's parallel agents work better by:
- Automatically isolating work in git worktrees
- Preventing conflicts through hooks
- Tracking progress in simple files
- Cleaning up when done

All invisible to the user - just better parallel development that works.