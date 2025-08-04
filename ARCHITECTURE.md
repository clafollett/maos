# MAOS Architecture

This document describes the architecture of MAOS (Multi-Agent Orchestration System), a high-performance Rust CLI that enhances Claude Code with parallel AI development capabilities.

## Overview

MAOS is designed as a compiled binary that replaces Python hook scripts with fast, reliable CLI commands. It integrates seamlessly with Claude Code through the `settings.json` configuration file, providing security, orchestration, and notification features without any runtime overhead.

## Design Principles

1. **Performance First**: Eliminate interpreter overhead with compiled Rust
2. **Zero Configuration**: Works immediately after installation
3. **Invisible Operation**: Users interact with Claude Code normally
4. **Type Safety**: Leverage Rust's type system for reliability
5. **Modular Architecture**: Separate concerns into focused crates

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Claude Code                           │
│                                                              │
│  ┌─────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │    User     │───►│   Claude AI   │───►│ settings.json│  │
│  └─────────────┘    └──────────────┘    └───────┬──────┘  │
└──────────────────────────────────────────────────┼──────────┘
                                                   │
                                                   ▼
┌─────────────────────────────────────────────────────────────┐
│                         MAOS CLI                             │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Commands   │  │   Security   │  │   Orchestration  │  │
│  │              │  │              │  │                  │  │
│  │ • pre-tool   │  │ • rm -rf     │  │ • Worktrees     │  │
│  │ • post-tool  │  │ • .env       │  │ • Sessions      │  │
│  │ • notify     │  │ • Paths      │  │ • Locks         │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │     TTS      │  │    Config    │  │     Logging      │  │
│  │              │  │              │  │                  │  │
│  │ • ElevenLabs │  │ • JSON       │  │ • Structured    │  │
│  │ • OpenAI     │  │ • Env vars   │  │ • Audit trail   │  │
│  │ • macOS      │  │ • Defaults   │  │ • Debug info    │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Crate Structure

MAOS follows a multi-crate workspace architecture for modularity and parallel compilation:

### `maos-cli` - Main CLI Application
The entry point for all commands. Uses `clap` for argument parsing and dispatches to appropriate handlers.

```rust
// Example command structure
#[derive(Parser)]
#[command(name = "maos")]
#[command(about = "Multi-Agent Orchestration System for Claude Code")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    PreToolUse { /* args */ },
    PostToolUse { /* args */ },
    Notify { /* args */ },
    // ... other commands
}
```

### `maos-core` - Core Orchestration Logic
Handles the fundamental orchestration features:
- Session management
- Git worktree operations
- File locking mechanisms
- Agent coordination

### `maos-security` - Security Features
Implements all security validations:
- Dangerous command detection (rm -rf patterns)
- Sensitive file protection (.env files)
- Path traversal prevention
- Input sanitization

### `maos-tts` - Text-to-Speech Integration
Multi-provider TTS support with automatic fallback:
- Provider trait for extensibility
- ElevenLabs implementation
- OpenAI implementation
- macOS native implementation
- pyttsx3 fallback

### `maos-config` - Configuration Management
Handles all configuration sources:
- JSON config file parsing
- Environment variable fallbacks
- Platform-specific defaults
- Configuration validation

## Data Flow

### Hook Execution Flow

```
1. Claude Code executes tool
2. settings.json triggers MAOS command
3. MAOS CLI parses arguments
4. Security validation occurs
5. Core logic executes (if approved)
6. Results logged and returned
7. Exit code indicates success/failure
```

### Security Check Flow

```rust
// Simplified security flow
pub fn validate_tool_use(tool: &ToolInput) -> Result<(), SecurityError> {
    // Check for dangerous commands
    if is_dangerous_rm(&tool.command) {
        return Err(SecurityError::DangerousCommand);
    }
    
    // Check for sensitive file access
    if is_env_file_access(&tool.file_path) {
        return Err(SecurityError::SensitiveFileAccess);
    }
    
    // Additional validations...
    Ok(())
}
```

## Key Design Decisions

### 1. Single Binary Distribution
- **Decision**: Compile to a single static binary
- **Rationale**: Simplifies distribution and eliminates dependencies
- **Trade-off**: Larger binary size vs zero runtime dependencies

### 2. JSON for Coordination
- **Decision**: Use JSON files for state management
- **Rationale**: Human-readable, debuggable, no database required
- **Trade-off**: File I/O overhead vs operational simplicity

### 3. Git Worktrees for Isolation
- **Decision**: Use git worktrees for agent workspaces
- **Rationale**: Complete isolation with native git integration
- **Trade-off**: Disk space usage vs perfect isolation

### 4. Synchronous Execution
- **Decision**: Commands execute synchronously
- **Rationale**: Matches Claude Code's hook execution model
- **Trade-off**: Simplicity vs potential async benefits

## Performance Considerations

### Startup Time
- Target: <10ms for any command
- Achieved through:
  - Minimal dependencies
  - Lazy initialization
  - Compiled binary with no JIT

### Memory Usage
- Efficient string handling with Rust's ownership
- Streaming JSON parsing for large files
- Minimal allocations in hot paths

### Concurrency
- File locking for safe concurrent access
- Atomic operations for state updates
- No global mutable state

## Security Model

### Threat Model
1. **Accidental Damage**: Users running dangerous commands
2. **Sensitive Data**: Exposure of secrets in .env files
3. **Path Traversal**: Accessing files outside project
4. **Command Injection**: Malicious input in commands

### Mitigations
1. **Command Validation**: Comprehensive rm -rf detection
2. **File Access Control**: Block .env and sensitive patterns
3. **Path Sanitization**: Validate all file paths
4. **Input Validation**: Strict parsing and escaping

## Extension Points

### Adding New Commands
1. Define command in `maos-cli`
2. Implement logic in appropriate crate
3. Add tests and documentation
4. Update settings.json examples

### Adding TTS Providers
1. Implement the `TtsProvider` trait
2. Add provider configuration
3. Update provider selection logic
4. Document API key requirements

### Custom Security Rules
1. Extend validation functions
2. Add configuration options
3. Implement rule engine
4. Update security documentation

## Testing Strategy

### Unit Tests
- Each crate has comprehensive unit tests
- Mock external dependencies
- Test edge cases and error conditions

### Integration Tests
- Full command execution tests
- File system interaction tests
- Multi-agent scenario tests

### Performance Tests
- Benchmark critical paths
- Monitor startup time
- Profile memory usage

## Future Architecture Considerations

### Plugin System
- Dynamic loading of custom hooks
- Plugin API for extensions
- Sandboxed execution environment

### Distributed Coordination
- Network-based agent coordination
- Distributed lock management
- Cloud storage integration

### Web Dashboard
- Real-time monitoring
- Session visualization
- Performance metrics

## Conclusion

MAOS's architecture prioritizes performance, reliability, and user experience. By leveraging Rust's strengths and maintaining a modular design, we ensure the system remains fast, secure, and maintainable while providing seamless integration with Claude Code.