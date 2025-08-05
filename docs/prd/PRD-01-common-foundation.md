# PRD-01: MAOS Common Foundation

## Executive Summary

The MAOS Common Foundation establishes the core types, error handling framework, configuration management, and shared utilities that all other MAOS components depend on. This foundational layer ensures consistency, maintainability, and performance across the entire system while enabling <10ms execution times through optimized data structures and zero-allocation patterns.

**Key Deliverable**: A robust `maos-core` crate containing all shared types, error handling, configuration management, JSON schemas, path utilities, and constants that enable seamless integration between all MAOS components.

## Problem Statement

Without a solid common foundation, MAOS components would suffer from:
- **Type Inconsistency**: Each component defining its own versions of core types (SessionId, AgentType, etc.)
- **Error Handling Chaos**: Inconsistent error propagation and exit code standards
- **Configuration Fragmentation**: Multiple configuration formats and loading mechanisms
- **Performance Overhead**: Repeated JSON parsing, string allocations, and path validation
- **Maintenance Nightmare**: Changes to core concepts requiring updates across multiple crates

We need a unified foundation that all other PRDs can build upon with confidence.

## Goals & Success Metrics

### Primary Goals

1. **Type Safety**: Zero-cost abstractions for all core domain types
2. **Performance Foundation**: Sub-millisecond operations for all common utilities
3. **Error Clarity**: Comprehensive error handling with actionable messages
4. **Configuration Simplicity**: Single source of truth for all MAOS configuration
5. **Developer Experience**: Clear APIs that prevent common mistakes

### Success Metrics

- **Performance**: All common operations complete in <1ms
- **Memory Efficiency**: Zero-allocation patterns for hot paths
- **API Stability**: Breaking changes require major version bumps only
- **Documentation Coverage**: 100% documented public APIs
- **Test Coverage**: >95% coverage for all foundational code
- **Integration Success**: All other PRDs build without type conflicts

## User Personas & Use Cases

### Primary User: MAOS Component Developer
**Profile**: Implements specific MAOS functionality (security, TTS, sessions, etc.)
**Use Case**: Reliable, fast, well-documented foundation types and utilities
**Success Criteria**: Never needs to implement core types or utilities from scratch

### Secondary User: MAOS CLI Developer  
**Profile**: Implements CLI commands and argument parsing
**Use Case**: Consistent error handling and configuration access patterns
**Success Criteria**: All commands have uniform behavior and error messages

### Tertiary User: MAOS Maintainer
**Profile**: Maintains and evolves the MAOS system
**Use Case**: Single place to modify core behavior and data structures
**Success Criteria**: Changes propagate consistently throughout the system

## Functional Requirements

### 1. Core Domain Types

#### 1.1 Session Management Types
```rust
/// Unique identifier for a MAOS session
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

/// Session metadata and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub status: SessionStatus,
    pub workspace_root: PathBuf,
    pub active_agents: Vec<AgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Failed { reason: String },
}
```

#### 1.2 Agent Management Types
```rust
/// Agent type - flexible string to support any user-defined agent
pub type AgentType = String;

/// Agent information and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: AgentId,
    pub agent_type: AgentType,  // e.g. "code-reviewer", "frontend-engineer", "my-custom-agent"
    pub session_id: SessionId,
    pub workspace_path: PathBuf,
    pub status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Initializing,
    Active,
    Waiting,
    Completed,
    Failed { error: String },
}
```

#### 1.3 Tool Integration Types
```rust
/// Tool call metadata from Claude Code hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<SessionId>,
    pub agent_id: Option<AgentId>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
}
```

### 2. Comprehensive Error Handling Framework

#### 2.1 Error Type Hierarchy
```rust
/// Root error type for all MAOS operations
#[derive(thiserror::Error, Debug)]
pub enum MaosError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Session error: {0}")]
    Session(#[from] SessionError),
    
    #[error("Security validation failed: {0}")]
    Security(#[from] SecurityError),
    
    #[error("File system error: {0}")]
    FileSystem(#[from] FileSystemError),
    
    #[error("Git operation failed: {0}")]
    Git(#[from] GitError),
    
    #[error("JSON processing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
    
    #[error("Operation timeout: {operation} took longer than {timeout_ms}ms")]
    Timeout { operation: String, timeout_ms: u64 },
}
```

#### 2.2 Exit Code Standards
```rust
/// Standard exit codes for MAOS operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    BlockingError = 2,    // Used to block tool execution
    ConfigError = 3,
    SecurityError = 4,
    TimeoutError = 5,
    InternalError = 99,
}

impl From<&MaosError> for ExitCode {
    fn from(error: &MaosError) -> Self {
        match error {
            MaosError::Security(_) => ExitCode::SecurityError,
            MaosError::Config(_) => ExitCode::ConfigError,
            MaosError::Timeout { .. } => ExitCode::TimeoutError,
            _ => ExitCode::GeneralError,
        }
    }
}
```

### 3. Configuration Management System

#### 3.1 Configuration Structure
```rust
/// Global MAOS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaosConfig {
    /// General system settings
    pub system: SystemConfig,
    
    /// Security validation settings
    pub security: SecurityConfig,
    
    /// TTS provider settings
    pub tts: TtsConfig,
    
    /// Session management settings
    pub session: SessionConfig,
    
    /// Git worktree settings
    pub worktree: WorktreeConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Maximum execution time for any operation (ms)
    pub max_execution_time_ms: u64,
    
    /// Maximum memory usage per operation (MB)
    pub max_memory_mb: usize,
    
    /// Default workspace root directory
    pub workspace_root: PathBuf,
    
    /// Enable performance monitoring
    pub enable_metrics: bool,
}
```

#### 3.2 Configuration Loading
```rust
/// Configuration loader with environment variable support
pub struct ConfigLoader {
    config_path: PathBuf,
    env_prefix: String,
}

impl ConfigLoader {
    /// Load configuration from file with environment overrides
    pub fn load() -> Result<MaosConfig, ConfigError> {
        // Implementation loads from:
        // 1. Default values
        // 2. ~/.maos/config.json
        // 3. Environment variables (MAOS_*)
        // 4. Command-line overrides
    }
    
    /// Validate configuration consistency
    pub fn validate(config: &MaosConfig) -> Result<(), ConfigError> {
        // Validate all configuration values
    }
}
```

### 4. JSON Schema Definitions

#### 4.1 Hook Message Formats
```rust
/// Pre-tool-use hook input message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreToolMessage {
    pub tool_call: ToolCall,
    pub session_context: Option<SessionContext>,
    pub workspace_constraints: Vec<PathConstraint>,
}

/// Post-tool-use hook input message  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToolMessage {
    pub tool_call: ToolCall,
    pub tool_result: ToolResult,
    pub session_context: Option<SessionContext>,
}

/// Notification message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub message: String,
    pub notification_type: NotificationType,
    pub engineer_name: Option<String>,
    pub session_id: Option<SessionId>,
    pub urgency: NotificationUrgency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    UserInputRequest,
    TaskCompletion,
    AgentSpawned,
    AgentCompleted,
    SecurityAlert,
    SystemError,
}
```

#### 4.2 Session State Formats
```rust
/// Session directory structure and file formats
pub mod session_files {
    /// .maos/sessions/{session_id}/session.json
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SessionFile {
        pub session: Session,
        pub configuration: SessionConfig,
        pub created_by: String,
        pub workspace_assignments: HashMap<AgentId, PathBuf>,
    }
    
    /// .maos/sessions/{session_id}/agents.json
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AgentsFile {
        pub agents: HashMap<AgentId, AgentInfo>,
        pub spawn_tree: AgentSpawnTree,
        pub coordination_state: CoordinationState,
    }
    
    /// .maos/sessions/{session_id}/locks.json
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LocksFile {
        pub file_locks: HashMap<PathBuf, FileLock>,
        pub resource_locks: HashMap<String, ResourceLock>,
        pub lock_timeout_ms: u64,
    }
    
    /// Session log directory structure
    /// .maos/sessions/{session_id}/logs/
    /// ├── session-{id}.log          # Current log (up to 10MB)
    /// ├── session-{id}.1.log.gz     # First rolled log (compressed)
    /// ├── session-{id}.2.log.gz     # Second rolled log
    /// └── session-{id}.10.log.gz    # Oldest kept log (max 10 files)
}
```

### 5. Path Utilities and Validation

#### 5.1 Safe Path Operations
```rust
/// Safe path operations with validation
pub struct PathValidator {
    allowed_roots: Vec<PathBuf>,
    blocked_patterns: Vec<String>,
}

impl PathValidator {
    /// Validate path is within allowed workspace boundaries
    pub fn validate_workspace_path(
        &self,
        path: &Path,
        workspace_root: &Path,
    ) -> Result<PathBuf, PathValidationError> {
        // Validate path is within workspace
        // Resolve symlinks and relative paths
        // Check for path traversal attempts
        // Return canonicalized safe path
    }
    
    /// Check if path matches blocked patterns
    pub fn is_blocked_path(&self, path: &Path) -> bool {
        // Check against .env, .git, and other sensitive patterns
    }
    
    /// Generate unique workspace path
    pub fn generate_workspace_path(
        &self,
        root: &Path,
        session_id: &SessionId,
        agent_type: &AgentType,
    ) -> PathBuf {
        // Generate: {root}/agent-{type}-{session-id}
    }
}
```

#### 5.2 Cross-Platform Path Handling
```rust
/// Cross-platform path utilities
pub mod path_utils {
    /// Convert path to platform-specific format
    pub fn normalize_path(path: &Path) -> PathBuf {
        // Handle Windows vs Unix path separators
        // Normalize case sensitivity
        // Remove redundant separators
    }
    
    /// Check if two paths refer to the same location
    pub fn paths_equal(a: &Path, b: &Path) -> bool {
        // Handle case sensitivity differences
        // Resolve symlinks
        // Compare canonical paths
    }
    
    /// Get relative path from base to target
    pub fn relative_path(base: &Path, target: &Path) -> Option<PathBuf> {
        // Calculate relative path if possible
        // Return None if paths are on different roots
    }
}
```

### 6. Constants and Defaults

#### 6.1 System Constants
```rust
/// System-wide constants
pub mod constants {
    use std::time::Duration;
    
    /// Default configuration directory: ~/.maos
    pub const DEFAULT_CONFIG_DIR: &str = ".maos";
    
    /// Default configuration file name
    pub const CONFIG_FILE_NAME: &str = "config.json";
    
    /// Default session directory name
    pub const SESSIONS_DIR_NAME: &str = "sessions";
    
    /// Default workspace directory name
    pub const WORKSPACES_DIR_NAME: &str = "workspaces";
    
    /// Default logs directory name within session
    pub const LOGS_DIR_NAME: &str = "logs";
    
    /// Performance targets
    pub const MAX_EXECUTION_TIME_MS: u64 = 10;
    pub const MAX_MEMORY_USAGE_MB: usize = 5;
    pub const MAX_BINARY_SIZE_MB: usize = 10;
    
    /// Timeout values
    pub const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_millis(5000);
    pub const FILE_LOCK_TIMEOUT: Duration = Duration::from_millis(1000);
    pub const TTS_TIMEOUT: Duration = Duration::from_millis(10000);
    
    /// File naming patterns
    pub const SESSION_FILE_NAME: &str = "session.json";
    pub const AGENTS_FILE_NAME: &str = "agents.json";
    pub const LOCKS_FILE_NAME: &str = "locks.json";
    pub const PROGRESS_FILE_NAME: &str = "progress.json";
    pub const TIMELINE_FILE_NAME: &str = "timeline.json";
    pub const METRICS_FILE_NAME: &str = "metrics.json";
}
```

#### 6.2 Default Configuration Values
```rust
/// Default configuration factory
impl Default for MaosConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                max_execution_time_ms: constants::MAX_EXECUTION_TIME_MS,
                max_memory_mb: constants::MAX_MEMORY_USAGE_MB,
                workspace_root: dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("maos-workspaces"),
                enable_metrics: true,
            },
            security: SecurityConfig::default(),
            tts: TtsConfig::default(),
            session: SessionConfig::default(),
            worktree: WorktreeConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}
```

### 7. Logging and Observability Foundation

#### 7.1 Structured Logging Setup
```rust
/// Logging configuration and setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub format: LogFormat,
    pub output: LogOutput,
    pub enable_performance_logs: bool,
    pub enable_security_logs: bool,
    pub rolling: RollingLogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug, 
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,       // Structured JSON for machine parsing
    Plain,      // Simple text format
    Pretty,     // Human-readable with colors (TTY only)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    SessionFile,    // Per-session log files
    Both,
}

/// Rolling log configuration for per-session logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingLogConfig {
    /// Maximum size per log file before rolling (10MB default)
    pub max_file_size_bytes: usize,
    
    /// Maximum number of rolled log files to keep per session
    pub max_files_per_session: usize,
    
    /// Compress rolled logs (.gz)
    pub compress_on_roll: bool,
    
    /// Log file name pattern for session logs
    pub file_pattern: String,  // e.g., "session-{session_id}.log"
}

impl Default for RollingLogConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 10 * 1024 * 1024,  // 10MB
            max_files_per_session: 10,
            compress_on_roll: true,
            file_pattern: "session-{session_id}.log".to_string(),
        }
    }
}

/// Initialize structured logging for MAOS
pub fn init_logging(config: &LoggingConfig) -> Result<(), LoggingError> {
    // Set up tracing with structured JSON output
    // Configure performance and security log streams
    // Set appropriate filtering levels
}
```

#### 7.2 Performance Monitoring
```rust
/// Performance metrics collection
pub struct PerformanceMetrics {
    execution_times: HashMap<String, Vec<Duration>>,
    memory_usage: HashMap<String, Vec<usize>>,
    error_counts: HashMap<String, usize>,
}

impl PerformanceMetrics {
    /// Record operation execution time
    pub fn record_execution_time(&mut self, operation: &str, duration: Duration) {
        // Record timing data for performance analysis
    }
    
    /// Record memory usage for operation
    pub fn record_memory_usage(&mut self, operation: &str, bytes: usize) {
        // Track memory consumption patterns
    }
    
    /// Export metrics for analysis
    pub fn export_metrics(&self) -> MetricsReport {
        // Generate performance report
    }
}
```

## Non-Functional Requirements

### Performance Requirements
- **Type Creation**: All type constructors complete in <1μs
- **JSON Parsing**: Standard message parsing in <100μs  
- **Path Validation**: Path operations complete in <50μs
- **Configuration Loading**: Full config load in <1ms
- **Memory Allocation**: Zero allocations in hot paths

### Reliability Requirements
- **Error Propagation**: 100% error cases properly typed and handled
- **Configuration Validation**: Invalid configs rejected with clear messages
- **Path Safety**: All path operations protected against traversal attacks
- **Type Safety**: Runtime panics impossible with proper API usage

### Security Requirements
- **Input Validation**: All external input validated and sanitized
- **Path Security**: Complete protection against path traversal
- **Configuration Security**: Sensitive values properly protected
- **Error Information**: No sensitive data leaked in error messages

### Compatibility Requirements
- **Rust Edition**: Compatible with Rust 2024 edition
- **Platform Support**: Linux, macOS, Windows (all architectures)
- **Serialization**: Forward/backward compatible JSON formats
- **API Stability**: Semantic versioning for all public APIs

## Technical Design

### 1. Crate Architecture
```
maos-core/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── types/              # Core domain types
│   │   ├── mod.rs
│   │   ├── session.rs      # Session and agent types
│   │   ├── tool.rs         # Tool call types
│   │   └── message.rs      # Hook message types
│   ├── error/              # Error handling framework
│   │   ├── mod.rs
│   │   ├── types.rs        # Error type definitions
│   │   └── codes.rs        # Exit code mappings
│   ├── config/             # Configuration management
│   │   ├── mod.rs
│   │   ├── types.rs        # Config type definitions
│   │   ├── loader.rs       # Config loading logic
│   │   └── validation.rs   # Config validation
│   ├── path/               # Path utilities
│   │   ├── mod.rs
│   │   ├── validator.rs    # Path validation
│   │   └── utils.rs        # Cross-platform utilities
│   ├── constants.rs        # System constants
│   ├── logging.rs          # Logging setup
│   └── metrics.rs          # Performance monitoring
└── tests/
    ├── integration/        # Integration tests
    └── unit/              # Unit tests per module
```

### 2. API Design Principles

#### 2.1 Zero-Cost Abstractions
- Use `&str` instead of `String` where possible
- Implement `Copy` for lightweight types
- Use `Cow<'_, str>` for optional allocations
- Lazy evaluation for expensive operations

#### 2.2 Error Handling Patterns
```rust
/// Consistent Result type aliases
pub type MaosResult<T> = Result<T, MaosError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Error context extension trait
pub trait ErrorContext<T> {
    fn with_context<F>(self, f: F) -> MaosResult<T>
    where
        F: FnOnce() -> String;
}
```

#### 2.3 Builder Patterns for Complex Types
```rust
/// Session builder for safe construction
pub struct SessionBuilder {
    id: Option<SessionId>,
    workspace_root: Option<PathBuf>,
    config: Option<SessionConfig>,
}

impl SessionBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn with_id(mut self, id: SessionId) -> Self { /* ... */ }
    pub fn with_workspace_root(mut self, root: PathBuf) -> Self { /* ... */ }
    pub fn build(self) -> MaosResult<Session> { /* ... */ }
}
```

### 3. Memory Management Strategy

#### 3.1 Allocation Minimization
- String interning for common values
- Object pooling for frequently created types
- Stack allocation for small, known-size data
- Memory-mapped files for large data sets

#### 3.2 Lifetime Management
- Explicit lifetime parameters where beneficial
- Owned types for data crossing async boundaries
- Borrowed types for temporary computations
- Smart pointers only when necessary

## Dependencies & Constraints

### External Dependencies
- **serde**: JSON serialization/deserialization (essential)
- **thiserror**: Error type derivation (essential)
- **chrono**: Date/time handling (essential)
- **dirs**: Cross-platform directory paths (essential)
- **tracing**: Structured logging (essential)

### Technical Constraints
- **Performance Budget**: Each utility function <1ms execution
- **Memory Budget**: <1MB memory usage for all common types
- **Binary Size Impact**: <500KB contribution to final binary
- **API Stability**: Public API must be stable across minor versions

### Design Constraints
- **Zero Runtime Dependencies**: No dynamic linking requirements
- **Thread Safety**: All types must be Send + Sync where appropriate
- **Error Transparency**: Errors must provide actionable information
- **Configuration Flexibility**: Support both file and environment configuration

## Success Criteria & Acceptance Tests

### Functional Success Criteria

1. **Type System Completeness**
   - All domain concepts have corresponding Rust types
   - Type relationships correctly model business logic
   - Serialization round-trip tests pass for all types

2. **Error Handling Robustness**
   - All error paths properly typed and tested
   - Error messages provide actionable information
   - Exit codes correctly map to error categories

3. **Configuration System Reliability**
   - Configuration loading handles all edge cases
   - Environment variable overrides work correctly
   - Invalid configurations rejected with clear errors

4. **Path Operations Security**
   - Path traversal attacks completely blocked
   - Workspace boundary enforcement works correctly
   - Cross-platform path handling consistent

### Performance Success Criteria

1. **Sub-Millisecond Operations**
   - Type construction: <1μs
   - JSON parsing: <100μs
   - Path validation: <50μs
   - Configuration access: <10μs

2. **Memory Efficiency**
   - Zero allocations in hot paths
   - Minimal heap usage for common operations
   - No memory leaks in long-running scenarios

3. **Binary Size Impact**
   - Foundation contributes <500KB to final binary
   - Dead code elimination removes unused features
   - Minimal dependency impact

### Quality Success Criteria

1. **Test Coverage**: >95% line coverage
2. **Documentation**: 100% documented public APIs
3. **API Stability**: No breaking changes without major version
4. **Cross-Platform**: All tests pass on Linux, macOS, Windows

### Integration Success Criteria

1. **Downstream Compatibility**: All other PRDs build successfully
2. **Runtime Integration**: No type conflicts or version mismatches
3. **Error Propagation**: Errors flow correctly through component boundaries
4. **Configuration Consistency**: All components use same config format

## Testing Strategy

### 1. Unit Testing Approach
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    #[test]
    fn test_session_id_generation() {
        let id = SessionId::generate();
        assert!(id.is_valid());
        assert_ne!(id, SessionId::generate());
    }
    
    proptest! {
        #[test]
        fn test_path_validation_security(
            path in ".*",
            workspace in "/[a-zA-Z0-9/]*"
        ) {
            let validator = PathValidator::new();
            let result = validator.validate_workspace_path(
                Path::new(&path), 
                Path::new(&workspace)
            );
            
            // Should never allow path traversal
            if let Ok(validated_path) = result {
                assert!(validated_path.starts_with(&workspace));
            }
        }
    }
}
```

### 2. Integration Testing
- **Configuration Loading**: Test all config sources and overrides
- **Error Propagation**: Verify errors flow correctly between components
- **Serialization Compatibility**: Round-trip tests for all JSON formats
- **Cross-Platform**: Platform-specific path and file system behavior

### 3. Performance Testing
```rust
#[cfg(test)]
mod performance_tests {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_session_creation(c: &mut Criterion) {
        c.bench_function("session_creation", |b| {
            b.iter(|| {
                let session = SessionBuilder::new()
                    .with_id(black_box(SessionId::generate()))
                    .build()
                    .unwrap();
                black_box(session)
            })
        });
    }
    
    criterion_group!(benches, benchmark_session_creation);
    criterion_main!(benches);
}
```

### 4. Security Testing
- **Path Traversal**: Comprehensive path injection attack tests
- **Input Validation**: Malformed JSON and invalid configuration tests
- **Buffer Overflow**: Large input handling tests
- **Resource Exhaustion**: Memory and file handle leak tests

## Timeline Estimate

### Week 1: Core Types and Error Framework
**Days 1-2**: Core domain types (Session, Agent, Tool)
**Days 3-4**: Error type hierarchy and exit code mapping
**Days 5-7**: JSON message format definitions and serialization tests

**Deliverables**:
- All core types defined with full serialization support
- Comprehensive error handling framework
- JSON schema validation for all message types

### Week 2: Configuration and Path Systems
**Days 1-3**: Configuration type definitions and loading system
**Days 4-5**: Path validation and cross-platform utilities
**Days 6-7**: Constants, defaults, and logging setup

**Deliverables**:
- Full configuration management system
- Secure path validation with traversal protection
- System constants and default value definitions

### Week 3: Testing and Optimization
**Days 1-2**: Comprehensive unit test suite
**Days 3-4**: Integration tests and property-based testing
**Days 5-7**: Performance optimization and benchmarking

**Deliverables**:
- >95% test coverage achieved
- Performance benchmarks meeting targets
- Security test suite with attack scenario coverage

### Week 4: Documentation and Integration
**Days 1-2**: API documentation and usage examples
**Days 3-4**: Integration testing with downstream components
**Days 5-7**: Final optimization and stability improvements

**Deliverables**:
- Complete API documentation
- Integration validation with other PRDs
- Production-ready foundation crate

## Risk Assessment & Mitigation

### Technical Risks

**Risk**: Performance targets not met due to serialization overhead
**Probability**: Medium **Impact**: High
**Mitigation**: Benchmark-driven development, zero-copy deserialization patterns, lazy evaluation

**Risk**: API design decisions limit future extensibility
**Probability**: Medium **Impact**: High  
**Mitigation**: Extensive design review, trait-based abstractions, version compatibility testing

**Risk**: Cross-platform path handling edge cases
**Probability**: High **Impact**: Medium
**Mitigation**: Comprehensive platform testing, existing library usage, extensive edge case testing

### Design Risks

**Risk**: Over-engineering leads to complex APIs
**Probability**: Medium **Impact**: Medium
**Mitigation**: Regular API review, simplicity-first design principle, user feedback integration

**Risk**: Breaking changes required during development
**Probability**: High **Impact**: Low
**Mitigation**: Pre-1.0 version, extensive prototyping, clear migration paths

### Integration Risks

**Risk**: Type conflicts with other crates
**Probability**: Low **Impact**: High
**Mitigation**: Single source of truth design, workspace-wide type consistency, integration testing

## Dependencies for Other PRDs

This Common Foundation PRD enables and is required by:

### Direct Dependencies
- **PRD 1: Security Validation System** (requires error types, path validation)
- **PRD 2: Session Management** (requires session types, configuration)
- **PRD 3: Git Worktree Management** (requires path utilities, error handling)
- **PRD 4: TTS Integration** (requires configuration, error types)
- **PRD 5: CLI Command Framework** (requires all types, error handling)

### Indirect Dependencies
- **PRD 6: Performance Monitoring** (builds on metrics foundation)
- **PRD 7: Distribution & Installation** (uses configuration system)
- **PRD 8: Integration Testing** (validates all foundation components)

## Implementation Notes

### 1. Development Priority
This PRD has **P0 Priority** as it blocks all other development work. No other PRD can begin implementation until the foundation types and error handling are established.

### 2. API Stability Commitment
Once this foundation reaches version 0.2.0, the public API will maintain backward compatibility within the 0.x series. Breaking changes will require careful migration planning.

### 3. Performance Monitoring
All foundation operations will include performance instrumentation to ensure they meet the <10ms execution time requirement for the overall system.

### 4. Documentation Standard
Every public type, function, and module must have comprehensive rustdoc documentation with usage examples before the PRD is considered complete.

## Summary

The MAOS Common Foundation establishes the bedrock upon which the entire MAOS system is built. By providing consistent types, robust error handling, flexible configuration management, and high-performance utilities, this foundation enables all other components to focus on their specific functionality while maintaining system-wide consistency and performance.

**Expected Outcome**: A rock-solid foundation that enables rapid, confident development of all other MAOS components while ensuring performance targets, security requirements, and maintainability goals are met across the entire system.

This foundation will make MAOS development feel effortless and ensure that performance, security, and reliability are built into every component from day one. 🚀💯