# PRD: MAOS Rust CLI Implementation

## Executive Summary

MAOS (Multi-Agent Orchestration System) is a high-performance Rust CLI that replaces the existing Python hook scripts with a blazing-fast binary (<10ms execution) to enhance Claude Code's sub-agent capabilities. This document specifies the complete migration from Python implementation to production-ready Rust CLI with zero runtime dependencies, professional distribution channels, and bulletproof security.

**Key Deliverable**: A single compiled `maos` binary that integrates seamlessly with Claude Code's hook system to enable parallel AI development through git worktree isolation, security enforcement, and session coordination.

## Problem Statement

The current Python-based MAOS implementation has several production limitations:
- **Performance**: Python startup overhead (50-200ms) slows every tool operation
- **Dependencies**: Requires Python runtime, uv, and package management
- **Reliability**: Script failures can corrupt sessions or block operations
- **Distribution**: Complex installation requiring Python ecosystem
- **Maintenance**: Multiple `.py` files scattered across hook directory

We need a production-grade solution that maintains full feature parity while delivering enterprise reliability and performance.

## Goals & Success Metrics

### Primary Goals

1. **Performance Excellence**: <10ms execution time for all commands
2. **Zero Dependencies**: Self-contained binary with no runtime requirements  
3. **Feature Parity**: 100% compatibility with Python implementation
4. **Professional Distribution**: NPM, Homebrew, and direct download channels
5. **Production Reliability**: Bulletproof error handling and graceful degradation

### Success Metrics

- **Startup Time**: `maos --version` completes in <10ms
- **Hook Performance**: Pre/post-tool hooks add <5ms overhead to Claude operations
- **Binary Size**: Optimized release binary <10MB
- **Memory Usage**: <5MB resident memory during operation
- **Distribution**: Available via `npx @maos/cli`, `brew install maos`, direct download
- **Compatibility**: Passes 100% of existing Python integration tests

## User Personas & Use Cases

### Primary User: AI Developer
**Profile**: Uses Claude Code for development projects, wants parallel agent capabilities
**Use Case**: Invisible enhancement - configures hooks once, then experiences faster multi-agent development
**Success Criteria**: Never needs to think about MAOS after initial setup

### Secondary User: DevOps Engineer  
**Profile**: Manages development tooling across teams
**Use Case**: Easy installation and updates via package managers
**Success Criteria**: Standard deployment patterns work seamlessly

### Tertiary User: MAOS Developer
**Profile**: Maintains and extends MAOS functionality
**Use Case**: Clear architecture, comprehensive testing, easy debugging
**Success Criteria**: Can add features and fix issues efficiently

## Functional Requirements

### 1. Core Commands (All <10ms execution)

#### 1.1 `maos pre-tool-use`
**Input**: JSON from stdin containing tool call data
**Processing**:
- Parse tool call parameters and metadata
- Execute security validation (rm -rf, .env protection)
- Handle Task tool for sub-agent workspace creation
- Update session state and file locks
- Enforce workspace boundaries for file operations

**Output**: 
- Exit code 0: Continue with tool execution
- Exit code 2: Block tool execution with error message
- Stderr: Informational messages and warnings

**Critical Behaviors**:
- BLOCKING security violations (rm -rf patterns, .env access)
- NON-BLOCKING MAOS orchestration errors
- Lazy workspace creation (only when file operations occur)
- Workspace path enforcement with clear error messages

#### 1.2 `maos post-tool-use`
**Input**: JSON from stdin with tool call results
**Processing**:
- Update performance metrics and execution logs
- Release file locks for completed operations
- Update session progress tracking
- Schedule cleanup for completed sessions

**Output**: Exit code 0 (always succeeds)

#### 1.3 `maos notify`
**Input**: JSON notification data from stdin
**Processing**:
- Auto-detect best available TTS provider (ElevenLabs > OpenAI > pyttsx3 > macOS say)
- Generate notification message with optional engineer name
- Execute TTS with timeout and error handling

**Output**: Exit code 0 (always succeeds, fails silently)

#### 1.4 `maos stop`
**Input**: JSON session completion data
**Processing**:
- Trigger completion TTS announcement
- Process transcript for response TTS (if enabled)
- Generate completion message via LLM or fallback
- Execute session cleanup and archiving

**Output**: Exit code 0 (always succeeds)

#### 1.5 `maos subagent-stop`
**Input**: JSON subagent completion data
**Processing**:
- Announce subagent completion via TTS
- Log subagent session data
- Update session state for orchestrator

**Output**: Exit code 0 (always succeeds)

#### 1.6 `maos prompt-submit`
**Input**: JSON user prompt data
**Processing**:
- Log user prompts for analysis
- Optional prompt validation (configurable)
- Session context injection (if enabled)

**Output**: 
- Exit code 0: Continue with prompt processing
- Exit code 2: Block prompt with validation error

#### 1.7 `maos session-info`
**Input**: Optional session ID argument
**Processing**:
- Display current session status and active agents
- Show workspace assignments and progress
- List file locks and coordination state

**Output**: Human-readable session information

#### 1.8 `maos worktree-list`
**Input**: None
**Processing**:
- Enumerate all active git worktrees
- Show MAOS-managed workspace assignments
- Display cleanup candidates

**Output**: Structured worktree listing

### 2. Security Validation System

#### 2.1 Command Pattern Blocking
**Requirement**: Block dangerous rm commands with comprehensive pattern matching
**Implementation**: 
- Parse command tokens to distinguish flags from filenames
- Detect `-rf`, `--recursive --force` combinations
- Block recursive deletion of dangerous paths (/, ~, .)
- Handle edge cases: `--`, quoted arguments, variable expansion

#### 2.2 Environment File Protection
**Requirement**: Prevent access to .env files containing secrets
**Implementation**:
- Block Read/Write/Edit operations on .env files (but allow .env.sample)
- Scan Bash commands for .env file manipulation
- Support configurable filename patterns

#### 2.3 Workspace Boundary Enforcement
**Requirement**: Ensure agents work only in assigned workspaces
**Implementation**:
- Block file operations outside assigned workspace paths
- Provide clear error messages with correct workspace paths
- Support both absolute and relative path validation

### 3. Git Worktree Management

#### 3.1 Workspace Creation
**Requirement**: Create isolated git worktrees for parallel agent work
**Implementation**:
- Generate unique workspace names: `agent-type-session-id`
- Create worktree branches: `agent/session-{id}/{agent-type}`
- Set up workspace directory structure
- Handle git repository detection and validation

#### 3.2 Workspace Cleanup
**Requirement**: Remove completed workspaces and branches
**Implementation**:
- Identify inactive workspaces (no recent file operations)
- Remove git worktrees and associated branches
- Archive workspace logs and metrics
- Configurable retention policies

### 4. Session Coordination

#### 4.1 Session Management
**Requirement**: Track multi-agent sessions and coordination
**File Structure**:
```
.maos/
├── config.json              # Global configuration
├── active_session.json      # Current session pointer
└── sessions/
    └── {session_id}/
        ├── session.json     # Session metadata
        ├── agents.json      # Active agents registry
        ├── locks.json       # File locks
        ├── progress.json    # Task progress
        ├── timeline.json    # Event log
        └── metrics.json     # Performance data
```

#### 4.2 File Locking
**Requirement**: Prevent concurrent file access conflicts
**Implementation**:
- Track file locks per session with agent ownership
- Warn about concurrent access attempts
- Automatic lock cleanup on tool completion
- Timeout-based stale lock removal

### 5. TTS Integration

#### 5.1 Multi-Provider Support
**Requirement**: Support multiple TTS providers with automatic fallback
**Priority Order**: ElevenLabs → OpenAI → pyttsx3 → macOS say
**Implementation**:
- Auto-detect available providers based on API keys and system capabilities
- Unified interface for all providers
- Timeout handling and error recovery
- Configurable voice settings per provider

#### 5.2 Smart Notifications
**Requirement**: Context-aware TTS announcements
**Implementation**:
- Completion announcements with LLM-generated messages
- Subagent completion notifications
- User input request alerts with engineer name
- Response TTS for conversational content (configurable)

## Non-Functional Requirements

### Performance Requirements
- **Startup Time**: <10ms for any command
- **Memory Usage**: <5MB RAM during operation
- **CPU Usage**: <5% CPU during normal operation
- **I/O Performance**: <1ms for JSON file read/write operations

### Reliability Requirements
- **Uptime**: 99.9% success rate for hook operations
- **Error Handling**: Graceful degradation, no hanging processes
- **Recovery**: Automatic cleanup of corrupted state files
- **Timeout Handling**: All operations timeout within reasonable limits

### Security Requirements
- **Input Validation**: All JSON inputs validated and sanitized
- **Path Security**: Protection against path traversal attacks
- **Command Injection**: Safe handling of shell commands and arguments
- **Privilege Escalation**: Run with minimal required permissions

### Compatibility Requirements
- **Operating Systems**: Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), Windows (x86_64)
- **Git Versions**: Git 2.5+ for worktree support
- **Claude Code**: Latest version with hook support
- **Backward Compatibility**: Support existing Python hook format during transition

### Maintainability Requirements
- **Code Coverage**: >90% test coverage across all modules
- **Documentation**: Comprehensive API docs and architecture guide
- **Logging**: Structured logging with configurable levels
- **Debugging**: Clear error messages and diagnostic information

## Python to Rust Feature Mapping

### Security Features

| Python Implementation | Rust Equivalent | Priority |
|----------------------|-----------------|----------|
| `is_dangerous_rm_command()` | `SecurityValidator::validate_rm_command()` | P0 |
| `is_env_file_access()` | `SecurityValidator::validate_env_access()` | P0 |
| Workspace enforcement | `WorkspaceValidator::enforce_boundaries()` | P0 |

### Hook Processing

| Python Hook | Rust Command | Key Functions |
|-------------|--------------|---------------|
| `pre_tool_use.py` | `maos pre-tool-use` | Security validation, workspace creation, lock management |
| `post_tool_use.py` | `maos post-tool-use` | Metrics logging, lock cleanup, progress updates |
| `notification.py` | `maos notify` | TTS notifications with provider detection |
| `stop.py` | `maos stop` | Completion announcements, response TTS, session cleanup |
| `subagent_stop.py` | `maos subagent-stop` | Subagent completion notifications |
| `user_prompt_submit.py` | `maos prompt-submit` | Prompt logging and validation |

### MAOS Backend Features

| Python Component | Rust Module | Description |
|------------------|-------------|-------------|
| `MAOSBackend` | `maos-core::Orchestrator` | Main coordination logic |
| `pre_tool_handler.py` | `maos-core::hooks::PreToolHandler` | Task tool processing |
| `post_tool_handler.py` | `maos-core::hooks::PostToolHandler` | Completion processing |
| File locks | `maos-session::LockManager` | Concurrent access control |
| Progress tracking | `maos-session::ProgressTracker` | Agent activity monitoring |

### TTS Integration

| Python TTS Script | Rust Provider | Configuration |
|-------------------|---------------|---------------|
| `elevenlabs_tts.py` | `ElevenLabsProvider` | `ELEVENLABS_API_KEY` |
| `openai_tts.py` | `OpenAIProvider` | `OPENAI_API_KEY` |
| `pyttsx3_tts.py` | `Pyttsx3Provider` | System TTS |
| macOS `say` | `MacOSProvider` | Built-in TTS |

## Success Criteria

### Functional Success Criteria

1. **Complete Feature Parity**: All Python functionality replicated in Rust
2. **Hook Integration**: Seamless replacement in Claude Code settings.json
3. **Security Enforcement**: 100% detection rate for blocked patterns
4. **Workspace Isolation**: Zero file conflicts between parallel agents
5. **TTS Functionality**: All notification types working across providers

### Performance Success Criteria

1. **Sub-10ms Execution**: All commands complete within performance targets
2. **Memory Efficiency**: Minimal memory footprint during operation
3. **Binary Size**: Optimized release binary under size targets
4. **Startup Performance**: Cold start times meet requirements

### Distribution Success Criteria

1. **Package Managers**: Available via NPM (`npx @maos/cli`) and Homebrew
2. **Direct Downloads**: GitHub releases for all target platforms
3. **Auto-Updates**: Built-in update mechanism with version checking
4. **Documentation**: Complete installation and usage guides

### Quality Success Criteria

1. **Test Coverage**: >90% code coverage with comprehensive test suite
2. **Integration Tests**: Full end-to-end testing with Claude Code
3. **Performance Benchmarks**: Automated performance regression testing
4. **Error Handling**: Graceful failure modes with clear error messages

## Implementation Phases

### Phase 1: Core CLI Foundation (Weeks 1-2)
**Scope**: Basic CLI structure and core commands
**Deliverables**:
- Workspace setup with multi-crate architecture
- Basic CLI parsing with clap
- Core command structure (`pre-tool-use`, `post-tool-use`)
- JSON input/output handling
- Error handling framework
- Basic security validation (rm -rf, .env protection)

**Acceptance Criteria**:
- `maos --version` executes in <10ms
- All core commands parse arguments correctly
- Security validation blocks dangerous operations
- JSON parsing handles malformed input gracefully

### Phase 2: Security & Git Integration (Weeks 3-4)
**Scope**: Complete security system and git worktree management
**Deliverables**:
- Comprehensive security validation rules
- Git worktree creation and management
- Workspace boundary enforcement
- Path validation and sanitization

**Acceptance Criteria**:
- Security tests pass 100% detection rate
- Git worktrees created with correct branch naming
- Workspace enforcement blocks out-of-bounds operations
- Path validation prevents traversal attacks

### Phase 3: Session Management (Weeks 5-6)
**Scope**: Multi-agent coordination and state management
**Deliverables**:
- Session creation and tracking
- File locking system
- Progress monitoring
- Agent registry and coordination

**Acceptance Criteria**:
- Sessions persist across tool calls
- File locks prevent conflicts
- Progress tracking captures agent activity
- Session cleanup removes stale data

### Phase 4: TTS Integration (Weeks 7-8)
**Scope**: Multi-provider TTS system
**Deliverables**:
- TTS provider abstraction
- ElevenLabs, OpenAI, pyttsx3, macOS providers
- Notification and completion announcements
- Configuration and fallback logic

**Acceptance Criteria**:
- TTS providers auto-detect based on available APIs
- Notifications work across all supported providers
- Completion messages generated via LLM or fallback
- Error handling prevents TTS failures from blocking operations

### Phase 5: Performance Optimization (Weeks 9-10)
**Scope**: Performance tuning and optimization
**Deliverables**:
- Binary size optimization
- Startup time optimization
- Memory usage optimization
- Performance benchmarking suite

**Acceptance Criteria**:
- All commands execute within performance targets
- Binary size meets requirements
- Memory usage stays within bounds
- Performance regression tests automated

### Phase 6: Distribution & Production (Weeks 11-12)
**Scope**: Distribution channels and production readiness
**Deliverables**:
- NPM package for `npx @maos/cli`
- Homebrew formula
- GitHub releases for all platforms
- Auto-update mechanism
- Production documentation

**Acceptance Criteria**:
- Installation works via all distribution channels
- Auto-update mechanism functions correctly
- Production documentation complete
- Migration guide from Python implementation

## Testing Requirements

### Unit Testing
**Framework**: Rust built-in testing with `cargo test`
**Coverage Target**: >90% line coverage
**Scope**:
- Security validation logic
- JSON parsing and serialization  
- Git operations
- File system operations
- TTS provider implementations

**Example Test Structure**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rm_rf_detection() {
        let validator = SecurityValidator::new();
        assert!(validator.validate_command("rm -rf /").is_err());
        assert!(validator.validate_command("rm file.txt").is_ok());
    }
    
    #[tokio::test]
    async fn test_worktree_creation() {
        let manager = WorktreeManager::new(temp_repo_path());
        let worktree = manager.create_worktree(spec).await.unwrap();
        assert!(worktree.path.exists());
    }
}
```

### Integration Testing
**Framework**: Custom integration test suite
**Scope**:
- End-to-end hook processing
- Claude Code integration scenarios
- Multi-agent coordination workflows
- Cross-platform compatibility

**Key Integration Tests**:
1. **Full Hook Cycle**: Pre-tool → Tool execution → Post-tool processing
2. **Multi-Agent Spawning**: Task tool creates multiple isolated workspaces
3. **File Conflict Prevention**: Concurrent file access properly handled
4. **Security Blocking**: Dangerous operations blocked with proper exit codes
5. **TTS Notifications**: All notification types function correctly

### Performance Testing
**Framework**: Criterion.rs for Rust benchmarks
**Metrics**:
- Command startup time (<10ms)
- JSON processing performance
- File system operation speed
- Memory usage profiling

**Benchmark Structure**:
```rust
fn benchmark_startup(c: &mut Criterion) {
    c.bench_function("maos startup", |b| {
        b.iter(|| {
            Command::new("maos")
                .arg("--version")
                .output()
                .unwrap()
        })
    });
}
```

### Security Testing
**Scope**: Comprehensive security validation testing
**Test Cases**:
- rm -rf pattern variations (quoted, variables, edge cases)
- .env file access attempts (direct and indirect)
- Path traversal attacks (../, symlinks, absolute paths)
- Command injection attempts
- Workspace boundary violations

### Compatibility Testing
**Platforms**: 
- Linux: Ubuntu 20.04+, CentOS 8+, Debian 11+
- macOS: 10.15+ (Intel and Apple Silicon)
- Windows: Windows 10+ (x86_64)

**Git Versions**: Test with Git 2.5, 2.20, 2.30, latest
**Claude Code Integration**: Test with multiple Claude Code versions

## Risk Assessment & Mitigation

### Technical Risks

**Risk**: Performance regression compared to Python
**Probability**: Low **Impact**: High
**Mitigation**: Comprehensive benchmarking, optimization focus, performance monitoring

**Risk**: Git worktree compatibility issues across versions
**Probability**: Medium **Impact**: Medium  
**Mitigation**: Extensive Git version testing, fallback mechanisms

**Risk**: TTS provider API changes breaking integration
**Probability**: Medium **Impact**: Low
**Mitigation**: Provider abstraction, graceful degradation, multiple fallbacks

### Operational Risks

**Risk**: Distribution channel failures (NPM, Homebrew)
**Probability**: Low **Impact**: Medium
**Mitigation**: Multiple distribution channels, direct download fallback

**Risk**: Migration complexity from Python implementation
**Probability**: Medium **Impact**: Medium
**Mitigation**: Comprehensive migration guide, backward compatibility support

### Security Risks

**Risk**: Security validation bypass
**Probability**: Low **Impact**: High
**Mitigation**: Extensive security testing, multiple validation layers, conservative defaults

**Risk**: Privilege escalation vulnerabilities
**Probability**: Low **Impact**: High
**Mitigation**: Minimal privilege principle, security audit, sandboxing

## Dependencies & Constraints

### External Dependencies
- **Git**: 2.5+ for worktree support (system dependency)
- **Claude Code**: Latest version with hook support (integration dependency)
- **API Keys**: ElevenLabs, OpenAI for premium TTS (optional)

### Technical Constraints
- **Hook Architecture**: Must work within Claude Code's hook system limitations
- **JSON Protocol**: Must maintain compatibility with existing hook JSON format
- **Process Model**: Stateless execution model for hook commands
- **Exit Codes**: Must use specific exit codes for blocking vs. non-blocking errors

### Resource Constraints
- **Development Time**: 12-week implementation timeline
- **Team Size**: Single developer primary implementation
- **Testing Resources**: Automated testing across multiple platforms
- **Distribution**: Limited to standard package manager channels

## Implementation Architecture

### Crate Structure
```
maos/
├── Cargo.toml (workspace)
├── crates/
│   ├── maos-cli/        # Binary entry point
│   ├── maos-core/       # Core orchestration logic
│   ├── maos-security/   # Security validation
│   ├── maos-worktree/   # Git worktree management
│   ├── maos-session/    # Session coordination
│   ├── maos-tts/        # TTS providers
│   └── maos-common/     # Shared types and utilities
└── target/              # Build artifacts
```

### Key Design Patterns
- **Command Pattern**: Each CLI command implemented as separate module
- **Provider Pattern**: TTS providers implement common trait
- **Strategy Pattern**: Security rules implement validation trait
- **Builder Pattern**: Configuration and complex object construction
- **Error Chain Pattern**: Comprehensive error handling with context

### Performance Optimization Strategy
1. **Lazy Initialization**: Delay expensive operations until needed
2. **Static Compilation**: Link dependencies statically for fast startup
3. **Memory Mapping**: Use mmap for large file operations
4. **Async I/O**: Non-blocking I/O for network operations
5. **Dead Code Elimination**: Compile-time feature flags for binary size

## Monitoring & Observability

### Logging Strategy
- **Structured Logging**: JSON-formatted logs with tracing crate
- **Log Levels**: TRACE, DEBUG, INFO, WARN, ERROR
- **Context Propagation**: Request tracing across hook operations
- **Performance Logging**: Execution time and resource usage metrics

### Metrics Collection
- **Command Execution Times**: Track performance across all commands
- **Error Rates**: Monitor success/failure rates by operation type
- **Resource Usage**: Memory and CPU utilization tracking
- **Session Statistics**: Agent spawning, workspace usage, coordination events

### Debugging Support
- **Verbose Mode**: Detailed execution tracing for troubleshooting
- **State Inspection**: Commands to examine session and agent state
- **Log Analysis**: Tools to analyze MAOS operation logs
- **Integration Debugging**: Special modes for Claude Code integration testing

## Summary

This PRD defines the complete migration from Python-based MAOS hooks to a production-ready Rust CLI that delivers blazing-fast performance (<10ms), zero runtime dependencies, and enterprise-grade reliability. The implementation will maintain 100% feature parity while providing professional distribution channels and comprehensive testing.

The phased approach ensures incremental delivery of value while maintaining system stability throughout the migration. Success will be measured by performance benchmarks, comprehensive testing, and seamless integration with Claude Code's multi-agent capabilities.

**Expected Outcome**: AI developers get invisible, high-performance parallel development capabilities through a single, professionally distributed `maos` binary that just works.