# ADR-01: File-Based Agent Communication

## Status
Accepted (Updated for Claude Code native sub-agents)

## Context
With Claude Code's native sub-agent feature, agents need a simple way to share context and communicate. File-based communication provides:

- **Transparency**: Easy to debug and inspect
- **Persistence**: Survives agent restarts
- **Simplicity**: No complex protocols or infrastructure
- **Compatibility**: Works with all Claude Code tools

## Decision
We will use file system patterns for agent communication and shared context.

### Communication Patterns

#### 1. Shared Context Directory
```
.maos/
└── shared/
    ├── api-spec.json      # Backend → Frontend
    ├── test-results.json  # QA → All
    └── deployment.yaml    # DevOps → All
```

#### 2. Agent Work Directories
```
.maos/
└── agents/
    ├── backend/
    │   └── api-implementation.py
    ├── frontend/
    │   └── ui-components.tsx
    └── qa/
        └── test-suite.py
```

#### 3. Communication Flow
1. Agents write outputs to shared directories
2. Other agents read from shared locations
3. Use file watching for real-time updates
4. Maintain clean file naming conventions

### Best Practices

1. **Clear Naming**: Use descriptive file names
2. **Format Standards**: JSON for structured data, Markdown for docs
3. **Directory Organization**: Group by purpose, not by agent
4. **Cleanup**: Remove temporary files after use

## Consequences

### Positive
- Simple to implement and debug
- No additional infrastructure needed
- Works with all Claude Code file tools
- Easy to version control

### Negative
- No real-time messaging (use file watching)
- Manual cleanup may be needed
- Limited to file-based patterns

## References
- Claude Code file tools documentation
- Agent communication patterns