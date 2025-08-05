# PRD-02: MAOS CLI Structure

## Executive Summary

The MAOS CLI Structure establishes a high-performance command-line interface using the Clap framework that serves as the primary entry point for all MAOS hook operations. This CLI replaces Python hook scripts with a sub-5ms startup time, routing hook commands to appropriate handlers while maintaining clean separation between CLI concerns and business logic.

**Key Deliverable**: A robust CLI binary that efficiently routes Claude Code hook commands (pre-tool-use, post-tool-use, notify, stop, subagent-stop, user-prompt-submit) with JSON stdin/stdout handling and proper exit code management.

## Problem Statement

Claude Code currently relies on Python hook scripts that introduce significant overhead:
- **Startup Latency**: Python scripts take 50-200ms to initialize, violating the <10ms execution target
- **Maintenance Complexity**: Multiple Python files with inconsistent error handling and argument parsing
- **Resource Overhead**: Python interpreter initialization consumes unnecessary memory and CPU
- **Integration Fragmentation**: Each hook script handles JSON parsing and configuration loading independently
- **Error Handling Inconsistency**: No standardized exit codes or error message formatting

We need a unified, high-performance CLI entry point that provides consistent behavior across all hook operations while maintaining extensibility for future commands.

## Goals & Success Metrics

### Primary Goals

1. **Lightning-Fast Startup**: CLI initialization completes in <5ms
2. **Clean Command Structure**: Extensible architecture supporting current and future commands
3. **Efficient JSON Processing**: Parse stdin JSON input in <1ms for typical hook messages
4. **Proper Exit Code Management**: Consistent exit codes that properly signal Claude Code behavior
5. **Separation of Concerns**: CLI handles argument parsing and routing; business logic stays in handlers

### Success Metrics

- **CLI Startup Time**: <5ms from process start to command dispatch
- **JSON Parse Time**: <1ms for typical hook messages (1-10KB)
- **Memory Footprint**: <2MB resident memory during execution
- **Binary Size**: <2MB final binary size
- **Error Rate**: Zero panics or crashes during normal operation
- **Developer Experience**: Adding new commands requires <10 lines of boilerplate

## User Personas & Use Cases

### Primary User: Claude Code Hook System
**Profile**: Automated system calling MAOS commands through hook configuration
**Use Case**: Execute pre-tool-use, post-tool-use, notify, stop commands with JSON input
**Success Criteria**: Commands execute successfully with appropriate exit codes and performance

### Secondary User: MAOS Component Developer
**Profile**: Implements specific MAOS functionality using the CLI framework
**Use Case**: Add new commands with minimal boilerplate while leveraging shared infrastructure
**Success Criteria**: Can implement new commands by focusing on business logic, not CLI concerns

### Tertiary User: Operations Engineer
**Profile**: Troubleshoots MAOS behavior in production environments
**Use Case**: Clear error messages, proper exit codes, and consistent logging for debugging
**Success Criteria**: Can quickly identify issues from error messages and exit codes

## Functional Requirements

### 1. Command Structure and Routing

#### 1.1 Core Hook Commands
```rust
/// Main CLI command structure
#[derive(Parser)]
#[command(name = "maos")]
#[command(about = "Multi-Agent Orchestration System")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Process pre-tool-use hook
    #[command(name = "pre-tool-use")]
    PreToolUse,
    
    /// Process post-tool-use hook
    #[command(name = "post-tool-use")]
    PostToolUse,
    
    /// Handle notification messages
    Notify,
    
    /// Process session stop events
    Stop {
        /// Export chat transcript
        #[arg(long)]
        chat: bool,
    },
    
    /// Handle subagent stop events
    #[command(name = "subagent-stop")]
    SubagentStop,
    
    /// Process user prompt submissions
    #[command(name = "user-prompt-submit")]
    UserPromptSubmit {
        /// Validate prompt before processing
        #[arg(long)]
        validate: bool,
    },
}
```

#### 1.2 Command Dispatch Architecture
```rust
/// Command dispatcher that routes to appropriate handlers
pub struct CommandDispatcher {
    config: Arc<MaosConfig>,
    metrics: Arc<PerformanceMetrics>,
}

impl CommandDispatcher {
    /// Dispatch command to appropriate handler
    pub async fn dispatch(&self, command: Commands) -> MaosResult<ExitCode> {
        let start_time = Instant::now();
        
        let result = match command {
            Commands::PreToolUse => self.handle_pre_tool_use().await,
            Commands::PostToolUse => self.handle_post_tool_use().await,
            Commands::Notify => self.handle_notify().await,
            Commands::Stop { chat } => self.handle_stop(chat).await,
            Commands::SubagentStop => self.handle_subagent_stop().await,
            Commands::UserPromptSubmit { validate } => self.handle_user_prompt_submit(validate).await,
        };
        
        self.metrics.record_execution_time(
            &format!("{:?}", command),
            start_time.elapsed()
        );
        
        result
    }
}
```

### 2. JSON Input/Output Handling

#### 2.1 Stdin Processing
```rust
/// High-performance JSON input processor
pub struct StdinProcessor {
    buffer: Vec<u8>,
    max_size: usize,
}

impl StdinProcessor {
    /// Read and parse JSON from stdin with timeout
    pub async fn read_json<T>(&mut self) -> MaosResult<T>
    where
        T: DeserializeOwned,
    {
        // Read from stdin with 100ms timeout
        let input = tokio::time::timeout(
            Duration::from_millis(100),
            self.read_to_buffer()
        ).await.map_err(|_| MaosError::Timeout {
            operation: "stdin_read".to_string(),
            timeout_ms: 100,
        })??;
        
        // Parse JSON using simd-json for performance
        serde_json::from_slice(&input)
            .map_err(|e| MaosError::Json(e))
    }
    
    async fn read_to_buffer(&mut self) -> io::Result<&[u8]> {
        self.buffer.clear();
        let mut stdin = tokio::io::stdin();
        stdin.read_to_end(&mut self.buffer).await?;
        Ok(&self.buffer)
    }
}
```

#### 2.2 Hook Message Types Integration
```rust
/// Hook input message variants from PRD-01
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hook_event_name")]
pub enum HookMessage {
    PreToolUse {
        session_id: String,
        transcript_path: PathBuf,
        cwd: PathBuf,
        tool_name: String,
        tool_input: serde_json::Value,
    },
    PostToolUse {
        session_id: String,
        transcript_path: PathBuf,
        cwd: PathBuf,
        tool_name: String,
        tool_result: serde_json::Value,
    },
    Notification {
        session_id: String,
        transcript_path: PathBuf,
        cwd: PathBuf,
        message: String,
    },
    Stop {
        session_id: String,
        transcript_path: PathBuf,
        cwd: PathBuf,
    },
    UserPromptSubmit {
        session_id: String,
        transcript_path: PathBuf,
        cwd: PathBuf,
        prompt: String,
    },
}
```

### 3. Exit Code Management

#### 3.1 Command-Specific Exit Codes
```rust
/// Command-specific exit code handling
pub trait CommandHandler {
    async fn execute(&self, input: HookMessage) -> MaosResult<CommandResult>;
}

#[derive(Debug)]
pub struct CommandResult {
    pub exit_code: ExitCode,
    pub output: Option<String>,
    pub metrics: ExecutionMetrics,
}

/// Exit code mapping for hook commands
impl From<CommandResult> for ExitCode {
    fn from(result: CommandResult) -> Self {
        result.exit_code
    }
}
```

#### 3.2 Error-to-Exit Code Mapping
```rust
/// Convert MAOS errors to appropriate exit codes
pub fn error_to_exit_code(error: &MaosError) -> ExitCode {
    match error {
        MaosError::Security(_) => ExitCode::BlockingError,  // Block tool execution
        MaosError::Config(_) => ExitCode::ConfigError,
        MaosError::Timeout { .. } => ExitCode::TimeoutError,
        MaosError::Json(_) => ExitCode::GeneralError,
        MaosError::InvalidInput { .. } => ExitCode::GeneralError,
        _ => ExitCode::InternalError,
    }
}
```

### 4. Logging and Observability

#### 4.1 Structured Logging Setup
```rust
/// Initialize logging for CLI operations
pub fn init_cli_logging(config: &LoggingConfig) -> MaosResult<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_timer(tracing_subscriber::fmt::time::uptime())
                .with_level(true)
                .json()
        );
        
    subscriber.init();
    Ok(())
}
```

#### 4.2 Performance Instrumentation
```rust
/// Instrument CLI operations for performance monitoring
#[tracing::instrument(skip(processor, command))]
pub async fn execute_command_with_metrics(
    processor: &mut StdinProcessor,
    command: Commands,
    dispatcher: &CommandDispatcher,
) -> MaosResult<ExitCode> {
    let start = Instant::now();
    
    // Parse input JSON
    let input = processor.read_json::<HookMessage>().await?;
    let parse_time = start.elapsed();
    
    tracing::info!(
        parse_time_ms = parse_time.as_millis(),
        command = ?command,
        "JSON parsing completed"
    );
    
    // Execute command
    let result = dispatcher.dispatch(command).await;
    let total_time = start.elapsed();
    
    tracing::info!(
        total_time_ms = total_time.as_millis(),
        result = ?result,
        "Command execution completed"
    );
    
    result
}
```

## Non-Functional Requirements

### Performance Requirements
- **CLI Startup Time**: Process initialization to command dispatch in <5ms
- **JSON Parsing**: Standard hook messages (1-10KB) parsed in <1ms
- **Memory Usage**: <2MB resident memory during typical operation
- **Binary Size**: Final binary size <2MB with all dependencies
- **Command Execution**: Total execution time <10ms (including business logic)

### Reliability Requirements
- **Error Handling**: All errors properly typed and converted to appropriate exit codes
- **Input Validation**: Malformed JSON handled gracefully with clear error messages
- **Timeout Protection**: Stdin reading protected with configurable timeouts
- **Resource Cleanup**: Proper cleanup of resources on both success and failure paths

### Security Requirements
- **Input Sanitization**: All JSON input validated before processing
- **Path Validation**: Working directory and file paths validated against traversal attacks
- **Resource Limits**: Memory and execution time limits enforced
- **Error Information**: No sensitive configuration data leaked in error messages

### Compatibility Requirements
- **Platform Support**: Linux, macOS, Windows (x86_64, aarch64)
- **Claude Code Integration**: Compatible with existing hook configuration format
- **Shell Integration**: Proper signal handling and exit code behavior
- **Environment Variables**: Respect standard environment variable conventions

## Technical Design

### 1. CLI Architecture

#### 1.1 Main Binary Structure
```
src/
├── main.rs              # CLI entry point and argument parsing
├── cli/                 # CLI-specific modules
│   ├── mod.rs
│   ├── commands.rs      # Command definitions and parsing
│   ├── dispatcher.rs    # Command routing and execution
│   ├── input.rs         # JSON input processing
│   └── output.rs        # Output formatting and exit codes
├── handlers/            # Command handler implementations
│   ├── mod.rs
│   ├── pre_tool_use.rs  # Pre-tool-use command handler
│   ├── post_tool_use.rs # Post-tool-use command handler
│   ├── notify.rs        # Notification handler
│   ├── stop.rs          # Stop command handler
│   └── user_prompt.rs   # User prompt handler
└── lib.rs               # Public API exports
```

#### 1.2 Dependency Injection Pattern
```rust
/// Dependency container for CLI operations
pub struct CliContext {
    pub config: Arc<MaosConfig>,
    pub metrics: Arc<PerformanceMetrics>,
    pub stdin_processor: StdinProcessor,
    pub handlers: HandlerRegistry,
}

impl CliContext {
    /// Build CLI context with configuration
    pub async fn build() -> MaosResult<Self> {
        let config = Arc::new(ConfigLoader::load()?);
        let metrics = Arc::new(PerformanceMetrics::new());
        let stdin_processor = StdinProcessor::new(config.system.max_memory_mb * 1024 * 1024);
        let handlers = HandlerRegistry::build(&config).await?;
        
        Ok(Self {
            config,
            metrics,
            stdin_processor,
            handlers,
        })
    }
}
```

### 2. Command Handler Pattern

#### 2.1 Handler Registry
```rust
/// Registry for command handlers with lazy initialization
pub struct HandlerRegistry {
    pre_tool_use: Box<dyn CommandHandler>,
    post_tool_use: Box<dyn CommandHandler>,
    notify: Box<dyn CommandHandler>,
    stop: Box<dyn CommandHandler>,
    subagent_stop: Box<dyn CommandHandler>,
    user_prompt_submit: Box<dyn CommandHandler>,
}

impl HandlerRegistry {
    /// Get handler for specific command
    pub fn get_handler(&self, command: &Commands) -> &dyn CommandHandler {
        match command {
            Commands::PreToolUse => &*self.pre_tool_use,
            Commands::PostToolUse => &*self.post_tool_use,
            Commands::Notify => &*self.notify,
            Commands::Stop { .. } => &*self.stop,
            Commands::SubagentStop => &*self.subagent_stop,
            Commands::UserPromptSubmit { .. } => &*self.user_prompt_submit,
        }
    }
}
```

#### 2.2 Handler Interface
```rust
/// Trait for command handlers with async execution
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// Execute command with hook message input
    async fn execute(&self, input: HookMessage) -> MaosResult<CommandResult>;
    
    /// Get command name for logging/metrics
    fn name(&self) -> &'static str;
    
    /// Validate input before execution (optional)
    fn validate_input(&self, input: &HookMessage) -> MaosResult<()> {
        Ok(())
    }
}
```

### 3. Performance Optimization Strategy

#### 3.1 Startup Time Optimization
- **Lazy Initialization**: Only initialize components when needed
- **Minimal Dependencies**: Use feature flags to exclude unused functionality
- **Static Linking**: Reduce dynamic library loading overhead
- **Profile-Guided Optimization**: Use PGO for binary optimization

#### 3.2 JSON Processing Optimization
- **simd-json**: Use SIMD-accelerated JSON parsing where available
- **Zero-Copy Deserialization**: Avoid unnecessary string allocations
- **Buffer Reuse**: Reuse input buffers across multiple operations
- **Streaming Parser**: Process large JSON inputs incrementally

#### 3.3 Memory Management
- **Arena Allocation**: Use arena allocators for short-lived objects
- **Object Pooling**: Pool frequently created objects
- **Stack Allocation**: Prefer stack allocation for small, fixed-size data
- **Memory Mapping**: Use memory-mapped files for large data sets

## Dependencies & Constraints

### External Dependencies (PRD-01 Common Foundation)
- **Core Types**: Session, Agent, Tool types from maos-core
- **Error Handling**: MaosError hierarchy and exit code mappings
- **Configuration**: MaosConfig and configuration loading system
- **JSON Schemas**: Hook message format definitions
- **Path Utilities**: Safe path validation and workspace management
- **Logging**: Structured logging setup and performance metrics

### Additional Dependencies
- **clap**: Command-line argument parsing (v4.0+)
- **tokio**: Async runtime for I/O operations (v1.0+)
- **serde_json**: JSON serialization/deserialization
- **tracing**: Structured logging and instrumentation
- **async-trait**: Async trait support

### Technical Constraints
- **Performance Budget**: <5ms CLI startup, <1ms JSON parsing
- **Memory Budget**: <2MB resident memory usage
- **Binary Size**: <2MB final binary size
- **Platform Support**: Linux, macOS, Windows compatibility

### Design Constraints
- **Single Binary**: All functionality in one executable
- **No Runtime Dependencies**: Statically linked binary
- **Backwards Compatibility**: Maintain hook configuration compatibility
- **Error Transparency**: Clear error messages for troubleshooting

## Success Criteria & Acceptance Tests

### Functional Success Criteria

1. **Command Parsing Completeness**
   - All required hook commands properly parsed and routed
   - Command-line arguments correctly handled for stop and user-prompt-submit
   - Help and version information properly displayed

2. **JSON Input Processing**
   - All hook message formats correctly deserialized
   - Malformed JSON handled gracefully with appropriate exit codes
   - Large JSON inputs (up to 1MB) processed within timeout limits

3. **Exit Code Consistency**
   - Security violations return exit code 2 (blocking)
   - Configuration errors return exit code 3
   - Timeout errors return exit code 5
   - Success operations return exit code 0

4. **Handler Integration**
   - All commands successfully dispatch to appropriate handlers
   - Error propagation from handlers to CLI layer works correctly
   - Command context properly passed through to business logic

### Performance Success Criteria

1. **Startup Performance**
   - Cold start (first execution): <5ms to command dispatch
   - Warm start (subsequent executions): <3ms to command dispatch
   - Memory allocation during startup: <1MB heap usage

2. **JSON Processing Performance**
   - Small messages (1KB): <100μs parsing time
   - Medium messages (10KB): <500μs parsing time
   - Large messages (100KB): <2ms parsing time

3. **Memory Efficiency**
   - Peak memory usage: <2MB for typical operations
   - Memory cleanup: No memory leaks after 1000 operations
   - Buffer reuse: Input buffers properly recycled

### Quality Success Criteria

1. **Test Coverage**: >90% line coverage for CLI modules
2. **Documentation**: All public APIs documented with examples
3. **Error Handling**: No panics during normal or error conditions
4. **Cross-Platform**: All tests pass on Linux, macOS, Windows

### Integration Success Criteria

1. **Claude Code Compatibility**: All existing hook configurations work unchanged
2. **Handler Integration**: All command handlers receive correctly formatted input
3. **Configuration Integration**: CLI respects all relevant configuration options
4. **Logging Integration**: Structured logs properly formatted and routed

## Testing Strategy

### 1. Unit Testing Approach
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use predicates::prelude::*;
    
    #[test]
    fn test_command_parsing() {
        let cli = Cli::try_parse_from(["maos", "pre-tool-use"]).unwrap();
        match cli.command {
            Commands::PreToolUse => (),
            _ => panic!("Expected PreToolUse command"),
        }
    }
    
    #[tokio::test]
    async fn test_json_input_parsing() {
        let input = r#"{
            "session_id": "test-session",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/tmp",
            "hook_event_name": "PreToolUse",
            "tool_name": "Read",
            "tool_input": {"file_path": "/tmp/test.txt"}
        }"#;
        
        let mut processor = StdinProcessor::new(1024 * 1024);
        // Simulate stdin input...
        let message: HookMessage = serde_json::from_str(input).unwrap();
        
        match message {
            HookMessage::PreToolUse { tool_name, .. } => {
                assert_eq!(tool_name, "Read");
            }
            _ => panic!("Expected PreToolUse message"),
        }
    }
}
```

### 2. Integration Testing
- **End-to-End CLI Testing**: Full command execution with real JSON input
- **Handler Integration**: Verify commands properly route to business logic
- **Error Path Testing**: Test all error conditions and exit codes
- **Performance Integration**: Verify performance targets in realistic scenarios

### 3. Performance Testing
```rust
#[cfg(test)]
mod performance_tests {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_cli_startup(c: &mut Criterion) {
        c.bench_function("cli_startup", |b| {
            b.iter(|| {
                let cli = black_box(Cli::try_parse_from(["maos", "pre-tool-use"]));
                black_box(cli)
            })
        });
    }
    
    fn benchmark_json_parsing(c: &mut Criterion) {
        let input = generate_test_hook_message(1024); // 1KB message
        
        c.bench_function("json_parse_1kb", |b| {
            b.iter(|| {
                let parsed: HookMessage = black_box(
                    serde_json::from_str(&input).unwrap()
                );
                black_box(parsed)
            })
        });
    }
    
    criterion_group!(benches, benchmark_cli_startup, benchmark_json_parsing);
    criterion_main!(benches);
}
```

### 4. Security Testing
- **JSON Injection**: Test with malicious JSON payloads
- **Buffer Overflow**: Test with oversized JSON inputs
- **Path Traversal**: Test with malicious file paths in hook messages
- **Resource Exhaustion**: Test with memory and CPU intensive inputs

## Timeline Estimate

### Week 1: Core CLI Framework
**Days 1-2**: Clap command structure and argument parsing
**Days 3-4**: JSON input processing and stdin handling
**Days 5-7**: Command dispatcher and handler registry pattern

**Deliverables**:
- Complete CLI command structure with Clap
- JSON input processing with error handling
- Command routing infrastructure

### Week 2: Handler Integration and Exit Codes
**Days 1-3**: Command handler trait and registry implementation
**Days 4-5**: Exit code management and error-to-code mapping
**Days 6-7**: Integration with PRD-01 error handling and types

**Deliverables**:
- Handler interface with async execution
- Proper exit code handling for all scenarios
- Integration with maos-core error types

### Week 3: Performance Optimization and Testing
**Days 1-2**: Performance optimization (startup time, JSON parsing)
**Days 3-4**: Comprehensive unit and integration test suite
**Days 5-7**: Performance benchmarking and memory optimization

**Deliverables**:
- Sub-5ms startup time achieved
- >90% test coverage
- Performance benchmarks meeting targets

### Week 4: Documentation and Production Readiness
**Days 1-2**: API documentation and usage examples
**Days 3-4**: Cross-platform testing and compatibility verification
**Days 5-7**: Final optimization and stability improvements

**Deliverables**:
- Complete CLI documentation
- Cross-platform compatibility verified
- Production-ready CLI binary

## Risk Assessment & Mitigation

### Technical Risks

**Risk**: JSON parsing performance doesn't meet <1ms target
**Probability**: Medium **Impact**: High
**Mitigation**: Use simd-json, implement streaming parser, benchmark-driven optimization

**Risk**: CLI startup time exceeds 5ms on slower systems
**Probability**: Medium **Impact**: High
**Mitigation**: Lazy initialization, minimal dependencies, profile-guided optimization

**Risk**: Memory usage exceeds 2MB limit with large JSON inputs
**Probability**: Low **Impact**: Medium
**Mitigation**: Streaming JSON parser, memory limits, buffer management

### Design Risks

**Risk**: Handler interface becomes too complex for future commands
**Probability**: Medium **Impact**: Medium
**Mitigation**: Keep interface minimal, use trait composition, regular API review

**Risk**: Exit code semantics conflict with Claude Code expectations
**Probability**: Low **Impact**: High
**Mitigation**: Extensive testing with Claude Code, maintain backward compatibility

### Integration Risks

**Risk**: Changes to hook message format break CLI parsing
**Probability**: Medium **Impact**: High
**Mitigation**: Versioned message formats, backward compatibility testing, schema validation

**Risk**: Handler dependencies create circular references
**Probability**: Low **Impact**: Medium
**Mitigation**: Clear dependency injection, handler registry pattern, interface segregation

## Dependencies for Other PRDs

This CLI Structure PRD enables and is required by:

### Direct Dependencies
- **PRD-03: Security Validation System** (pre-tool-use command handler)
- **PRD-04: Session Management** (session context for all commands)
- **PRD-05: TTS Integration** (notify and stop command handlers)
- **PRD-06: Git Worktree Management** (workspace setup in pre-tool-use)

### Indirect Dependencies
- **PRD-07: Performance Monitoring** (CLI metrics collection)
- **PRD-08: Distribution & Installation** (binary distribution)
- **PRD-09: Integration Testing** (end-to-end CLI testing)

## Implementation Notes

### 1. Development Priority
This PRD has **P1 Priority** as it provides the main entry point for MAOS functionality. Handler implementations can proceed in parallel once the CLI framework is established.

### 2. Performance Monitoring
All CLI operations include performance instrumentation to ensure the <10ms total execution time requirement is met. Critical path operations (startup, JSON parsing) have dedicated metrics.

### 3. Backwards Compatibility
The CLI maintains compatibility with existing Claude Code hook configurations while providing a foundation for future command extensions.

### 4. Error Handling Philosophy
The CLI focuses on proper error classification and exit code mapping, allowing business logic in handlers to focus on domain-specific concerns while maintaining consistent error reporting.

## Summary

The MAOS CLI Structure provides a high-performance, extensible foundation for all MAOS hook operations. By leveraging Rust's performance characteristics and the Clap framework's robust argument parsing, this CLI achieves sub-5ms startup times while maintaining clean separation between command routing and business logic.

**Expected Outcome**: A lightning-fast CLI that makes MAOS hook operations invisible to Claude Code users while providing a solid foundation for all MAOS functionality. The CLI will serve as the primary touchpoint between Claude Code and MAOS, ensuring consistent behavior and performance across all operations.

This CLI structure enables rapid development of command handlers while guaranteeing that performance, error handling, and logging remain consistent across the entire MAOS system.