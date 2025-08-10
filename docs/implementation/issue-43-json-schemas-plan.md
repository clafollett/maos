# 🚀 TDD Implementation Plan for Issue #43: JSON Schemas & Integration Types

## Overview
Implement JSON message formats for Claude Code hook integration and session state persistence using strict Test-Driven Development (Red/Green/Refactor) methodology with idiomatic Rust best practices.

## 📋 Critical Discovery: Actual Claude Code Hook Format

Based on analysis of the official docs and our Python implementation, we discovered a **discrepancy** between documented and actual formats:

### Actual Format (Used by Our Python Hooks):
```json
{
  "tool_name": "Edit",           // The tool being called
  "tool_input": {                // Tool-specific parameters
    "file_path": "/path/to/file",
    "old_string": "...",
    "new_string": "..."
  },
  "metadata": {                   // Hook metadata (undocumented but exists!)
    // Additional context passed by hooks
  }
}
```

### Documented Format (From Claude Code Docs):
```json
{
  "session_id": "string",
  "transcript_path": "string",
  "cwd": "string",
  "hook_event_name": "PreToolUse",
  "tool_name": "string",
  "tool_input": { ... }
}
```

### Post-Tool Hook Addition:
```json
{
  // ... all fields above plus:
  "tool_response": {              // Result of tool execution
    "success": true,
    "filePath": "/path/to/file"
  }
}
```

**Critical Note:** We must support BOTH formats for compatibility and future-proofing!

## 📦 Implementation Phases

### Phase 1: Foundation & Setup (TDD Prep)
1. **Create feature branch**: `feature/issue-43/json-schemas-integration`
2. **Add dependencies to Cargo.toml**:
   ```toml
   jsonschema = "0.26"
   schemars = "0.8"  # for deriving JSON schemas
   dirs = "5.0"      # for home directory
   ```
3. **Create module structure**:
   - `src/messages/mod.rs` - Main module
   - `src/messages/hook.rs` - Hook message types
   - `src/messages/notification.rs` - Notification types  
   - `src/messages/session_state.rs` - Session state files
   - `src/messages/validation.rs` - Schema validation
   - `src/messages/compatibility.rs` - Handle both formats

### Phase 2: Hook Messages (RED → GREEN → REFACTOR)

#### 2.1 Hook Input Messages Tests (RED)
```rust
// tests/messages_tests.rs
#[test]
fn test_hook_input_legacy_format() { /* Our Python format */ }
#[test]
fn test_hook_input_documented_format() { /* Claude docs format */ }
#[test]
fn test_pre_tool_message_serialization() { /* expect compile fail */ }
#[test]
fn test_post_tool_message_with_response() { /* expect compile fail */ }
#[test]
fn test_metadata_extraction() { /* expect compile fail */ }
```

#### 2.2 Implement Hook Messages (GREEN)
```rust
/// Unified hook input that handles both formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HookInput {
    /// Format used by our Python hooks
    Legacy {
        tool_name: String,
        tool_input: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_response: Option<Value>,
        #[serde(default)]
        metadata: HashMap<String, Value>,
    },
    /// Format documented by Claude Code
    Documented {
        session_id: String,
        transcript_path: PathBuf,
        cwd: PathBuf,
        hook_event_name: HookEventName,
        tool_name: String,
        tool_input: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_response: Option<Value>,
    }
}

/// Hook event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum HookEventName {
    PreToolUse,
    PostToolUse,
}

/// Pre-tool message for MAOS processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreToolMessage {
    pub tool_call: ToolCall,
    pub session_context: SessionContext,
    pub workspace_constraints: Vec<PathConstraint>,
    #[serde(flatten)]
    pub raw_input: HookInput,
}

/// Post-tool message for MAOS processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToolMessage {
    pub tool_call: ToolCall,
    pub tool_result: ToolResult,
    pub session_context: SessionContext,
    #[serde(flatten)]
    pub raw_input: HookInput,
}
```

#### 2.3 Refactor & Optimize
- Extract common types to reduce duplication
- Add builder pattern for complex types
- Optimize serialization with custom serializers if needed

### Phase 3: Session Context & Constraints (RED → GREEN → REFACTOR)

#### 3.1 Context Tests (RED)
```rust
#[test]
fn test_session_context_from_metadata() { /* extract from metadata */ }
#[test]
fn test_workspace_constraints_validation() { /* path constraints */ }
#[test]
fn test_agent_identification() { /* agent ID extraction */ }
#[test]
fn test_security_path_validation() { /* dangerous paths */ }
```

#### 3.2 Implement Context Types (GREEN)
```rust
/// Session context passed to hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: SessionId,
    pub agent_id: Option<AgentId>,
    pub agent_type: Option<AgentType>,
    pub workspace_root: PathBuf,
    pub active_agents: Vec<AgentId>,
    pub parent_agent: Option<AgentId>,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
}

impl SessionContext {
    /// Extract context from hook input
    pub fn from_hook_input(input: &HookInput) -> Result<Self> {
        match input {
            HookInput::Legacy { metadata, .. } => {
                // Extract from metadata like Python does
                Self::from_metadata(metadata)
            },
            HookInput::Documented { session_id, cwd, transcript_path, .. } => {
                Ok(Self {
                    session_id: SessionId::from_str(session_id)?,
                    cwd: cwd.clone(),
                    transcript_path: Some(transcript_path.clone()),
                    // ... other fields from environment or defaults
                })
            }
        }
    }
}

/// Path constraints for workspace isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConstraint {
    pub allowed_paths: Vec<PathBuf>,
    pub blocked_patterns: Vec<String>,
    pub max_depth: Option<usize>,
}
```

### Phase 4: Hook Response Types (RED → GREEN → REFACTOR)

#### 4.1 Response Tests (RED)
```rust
#[test]
fn test_hook_response_allow() { /* simple allow */ }
#[test]
fn test_hook_response_block_security() { /* block with reason */ }
#[test]
fn test_hook_response_modify_params() { /* parameter modification */ }
#[test]
fn test_hook_response_redirect_tool() { /* tool redirection */ }
#[test]
fn test_exit_code_mapping() { /* correct exit codes */ }
```

#### 4.2 Implement Responses (GREEN)
```rust
/// Hook response format - what we send back to Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "data")]
pub enum HookResponse {
    /// Allow tool execution (exit 0)
    Allow,
    
    /// Block tool execution (exit 2)
    Block { reason: String },
    
    /// Modify tool parameters (not yet supported by Claude Code)
    Modify { parameters: Value },
    
    /// Redirect to different tool (future feature)
    Redirect { 
        tool_name: String,
        parameters: Value,
    },
}

impl HookResponse {
    /// Convert to exit code for hook script
    pub fn to_exit_code(&self) -> i32 {
        match self {
            HookResponse::Allow => 0,
            HookResponse::Block { .. } => 2,
            _ => 0, // Unsupported actions default to allow
        }
    }
}
```

### Phase 5: Notification Messages (RED → GREEN → REFACTOR)

#### 5.1 Notification Tests (RED)
```rust
#[test]
fn test_notification_serialization() { /* JSON format */ }
#[test]
fn test_tts_formatting_with_engineer() { /* TTS with name */ }
#[test]
fn test_urgency_levels() { /* priority handling */ }
#[test]
fn test_notification_types() { /* all notification types */ }
#[test]
fn test_config_integration() { /* read engineer name from config */ }
```

#### 5.2 Implement Notifications (GREEN)
```rust
/// Notification message for TTS and logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub message: String,
    pub notification_type: NotificationType,
    pub engineer_name: Option<String>,
    pub session_id: Option<SessionId>,
    pub urgency: NotificationUrgency,
    #[serde(default = "chrono::Utc::now")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    UserInputRequest,
    TaskCompletion,
    AgentSpawned,
    AgentCompleted,
    SecurityAlert,
    SystemError,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationUrgency {
    Low,      // Can be batched/delayed
    Normal,   // Standard notification
    High,     // Immediate attention
    Critical, // Interrupt current work
}

impl NotificationMessage {
    /// Format for TTS announcement (matches Python implementation)
    pub fn to_tts_string(&self) -> String {
        let engineer = self.engineer_name.as_deref()
            .unwrap_or_else(|| {
                // Try to get from config like Python does
                // Read from .claude/hooks/maos/config.json
                "Engineer"
            });
        
        match self.notification_type {
            NotificationType::UserInputRequest => {
                format!("{}, I need your input: {}", engineer, self.message)
            }
            NotificationType::TaskCompletion => {
                format!("{}, task completed: {}", engineer, self.message)
            }
            NotificationType::AgentSpawned => {
                format!("New agent spawned: {}", self.message)
            }
            NotificationType::AgentCompleted => {
                format!("Agent finished: {}", self.message)
            }
            NotificationType::SecurityAlert => {
                format!("Security alert! {}", self.message)
            }
            NotificationType::SystemError => {
                format!("System error: {}", self.message)
            }
        }
    }
}
```

### Phase 6: Session State Files (RED → GREEN → REFACTOR)

#### 6.1 State File Tests (RED)
```rust
#[test]
fn test_session_file_roundtrip() { /* serialize/deserialize */ }
#[test]
fn test_agents_file_spawn_tree() { /* agent hierarchy */ }
#[test]
fn test_locks_file_conflicts() { /* lock management */ }
#[test]
fn test_atomic_file_operations() { /* safe writes */ }
#[test]
fn test_pending_agents() { /* lazy workspace creation */ }
```

#### 6.2 Implement State Files (GREEN)
Based on Python's actual session structure:
```rust
/// Session state matching Python implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFile {
    pub session_id: SessionId,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub workspace_root: PathBuf,
    pub active_agents: HashMap<AgentId, AgentInfo>,
    pub pending_agents: Vec<PendingAgent>,
}

/// Pending agent waiting for workspace (lazy creation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAgent {
    pub agent_id: String,
    pub agent_type: String,
    pub workspace_created: bool,
    pub workspace_path: Option<PathBuf>,
}

/// Agents file for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsFile {
    pub agents: HashMap<AgentId, AgentInfo>,
    pub spawn_tree: AgentSpawnTree,
}

/// Agent spawn hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpawnTree {
    pub root_agent: AgentId,
    pub children: HashMap<AgentId, Vec<AgentId>>,
    pub spawn_times: HashMap<AgentId, DateTime<Utc>>,
}

/// File locks for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocksFile {
    pub file_locks: HashMap<PathBuf, FileLock>,
    pub lock_timeout_ms: u64,
}

/// File lock information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLock {
    pub agent: AgentId,
    pub tool: String,
    pub acquired_at: DateTime<Utc>,
    pub file_path: PathBuf,
}

/// Session directory helper
pub struct SessionDirectory {
    root: PathBuf,
}

impl SessionDirectory {
    pub fn new(session_id: &SessionId) -> Result<Self> {
        let root = Path::new(".maos")
            .join("sessions")
            .join(session_id.as_str());
            
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }
    
    pub fn session_file_path(&self) -> PathBuf {
        self.root.join("session.json")
    }
    
    pub fn agents_file_path(&self) -> PathBuf {
        self.root.join("agents.json")
    }
    
    pub fn locks_file_path(&self) -> PathBuf {
        self.root.join("locks.json")
    }
    
    pub fn pending_agents_path(&self) -> PathBuf {
        self.root.join("pending_agents.json")
    }
    
    pub fn activity_path(&self) -> PathBuf {
        self.root.join("activity.json")
    }
}
```

### Phase 7: Schema Validation (RED → GREEN → REFACTOR)

#### 7.1 Validation Tests (RED)
```rust
#[test]
fn test_validate_hook_input_legacy() { /* Python format */ }
#[test]
fn test_validate_hook_input_documented() { /* Claude format */ }
#[test]
fn test_schema_performance() { /* < 1ms target */ }
#[test]
fn test_invalid_messages() { /* error handling */ }
#[test]
fn test_schema_caching() { /* compiled schema caching */ }
```

#### 7.2 Implement Validation (GREEN)
```rust
/// Schema validator with cached compiled schemas
pub struct SchemaValidator {
    hook_input_schema: JSONSchema,
    notification_schema: JSONSchema,
    session_schema: JSONSchema,
}

impl SchemaValidator {
    pub fn new() -> Result<Self, SchemaError> {
        // Compile schemas once for performance
        let hook_input_schema = JSONSchema::compile(&HOOK_INPUT_SCHEMA)?;
        let notification_schema = JSONSchema::compile(&NOTIFICATION_SCHEMA)?;
        let session_schema = JSONSchema::compile(&SESSION_SCHEMA)?;
        
        Ok(Self {
            hook_input_schema,
            notification_schema,
            session_schema,
        })
    }
    
    /// Validate any hook input format
    pub fn validate_hook_input(&self, value: &Value) -> Result<(), SchemaError> {
        // Validates both legacy and documented formats
        self.hook_input_schema.validate(value)
            .map_err(|e| SchemaError::ValidationFailed(e.collect()))
    }
}

/// JSON Schema for hook input (supports both formats)
const HOOK_INPUT_SCHEMA: &str = r#"
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "oneOf": [
    {
      "type": "object",
      "required": ["tool_name", "tool_input"],
      "properties": {
        "tool_name": { "type": "string" },
        "tool_input": { "type": "object" },
        "tool_response": { "type": ["object", "null"] },
        "metadata": { "type": "object" }
      }
    },
    {
      "type": "object",
      "required": ["session_id", "transcript_path", "cwd", "hook_event_name", "tool_name", "tool_input"],
      "properties": {
        "session_id": { "type": "string" },
        "transcript_path": { "type": "string" },
        "cwd": { "type": "string" },
        "hook_event_name": { "enum": ["PreToolUse", "PostToolUse"] },
        "tool_name": { "type": "string" },
        "tool_input": { "type": "object" },
        "tool_response": { "type": ["object", "null"] }
      }
    }
  ]
}
"#;
```

### Phase 8: Integration & Performance (RED → GREEN → REFACTOR)

#### 8.1 Integration Tests (RED)
```rust
#[test]
fn test_python_compatibility() { 
    // Read actual Python hook JSON logs
    let log_path = Path::new(".maos/logs/pre_tool_use.jsonl");
    // Parse and validate each line
}

#[test]
fn test_claude_code_compatibility() { 
    // Test with both format variations
}

#[test]
fn test_session_persistence() { 
    // Full session lifecycle with atomic writes
}

#[test]
fn test_concurrent_agents() { 
    // Multi-agent coordination with locks
}

#[test]
fn test_workspace_isolation() {
    // Verify path constraints work
}
```

#### 8.2 Performance Benchmarks
```rust
// benches/messages_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_hook_input_parsing(c: &mut Criterion) {
    c.bench_function("parse_legacy_format", |b| {
        let json = r#"{"tool_name":"Edit","tool_input":{"file_path":"test.rs"}}"#;
        b.iter(|| {
            let _: HookInput = serde_json::from_str(black_box(json)).unwrap();
        });
    });
}

fn bench_schema_validation(c: &mut Criterion) {
    let validator = SchemaValidator::new().unwrap();
    let value = json!({"tool_name": "Edit", "tool_input": {}});
    
    c.bench_function("validate_hook_input", |b| {
        b.iter(|| {
            validator.validate_hook_input(black_box(&value)).unwrap();
        });
    });
}

fn bench_tts_formatting(c: &mut Criterion) {
    let msg = NotificationMessage {
        message: "Test completed".into(),
        notification_type: NotificationType::TaskCompletion,
        engineer_name: Some("Marvin".into()),
        // ...
    };
    
    c.bench_function("format_tts", |b| {
        b.iter(|| {
            black_box(msg.to_tts_string());
        });
    });
}

criterion_group!(benches, bench_hook_input_parsing, bench_schema_validation, bench_tts_formatting);
criterion_main!(benches);
```

### Phase 9: Documentation & Examples

#### 9.1 Comprehensive Rustdocs
Every public type and method needs documentation following our existing patterns:

```rust
/// Hook input message from Claude Code
///
/// This enum handles both the legacy format used by our Python hooks
/// and the documented format from Claude Code documentation.
///
/// # Format Compatibility
///
/// The untagged deserialization allows us to accept either format:
///
/// ## Legacy Format (Python hooks)
/// ```json
/// {
///   "tool_name": "Edit",
///   "tool_input": { "file_path": "..." },
///   "metadata": { }
/// }
/// ```
///
/// ## Documented Format (Claude Code docs)
/// ```json
/// {
///   "session_id": "sess_123",
///   "transcript_path": "/path/to/transcript",
///   "cwd": "/current/directory",
///   "hook_event_name": "PreToolUse",
///   "tool_name": "Edit",
///   "tool_input": { "file_path": "..." }
/// }
/// ```
///
/// # Example
///
/// ```
/// use maos_core::messages::HookInput;
///
/// // Parse legacy format
/// let legacy = r#"{"tool_name": "Edit", "tool_input": {}}"#;
/// let input: HookInput = serde_json::from_str(legacy)?;
///
/// // Parse documented format  
/// let documented = r#"{
///   "session_id": "sess_123",
///   "transcript_path": "/tmp/transcript",
///   "cwd": "/home/user",
///   "hook_event_name": "PreToolUse",
///   "tool_name": "Edit",
///   "tool_input": {}
/// }"#;
/// let input: HookInput = serde_json::from_str(documented)?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HookInput {
    // ...
}
```

#### 9.2 Migration Guide
Create `docs/migration/python-to-rust-hooks.md`:
- How to transition from Python hooks
- Compatibility guarantees
- Performance improvements
- Future roadmap

## 🎯 Success Metrics

- ✅ 100% compatibility with existing Python hooks
- ✅ Support for documented Claude Code format
- ✅ All tests passing (unit + integration)
- ✅ 100% test coverage
- ✅ Schema validation < 1ms
- ✅ JSON parsing < 100μs  
- ✅ Atomic file operations (no data loss)
- ✅ Thread-safe implementations
- ✅ Zero clippy warnings
- ✅ Comprehensive rustdocs on all public APIs
- ✅ Performance benchmarks meet targets

## 📦 Key Implementation Details

1. **Dual format support**: Use `#[serde(untagged)]` enum to handle both formats transparently
2. **Backward compatibility**: Ensure Python hooks continue working without changes
3. **Forward compatibility**: Support documented Claude Code format for future
4. **Config integration**: Read from same `.claude/hooks/maos/config.json` as Python
5. **Session management**: Match Python's `.maos/sessions/` structure exactly
6. **Error handling**: Extend `MaosError` with new `SchemaError` variant
7. **Atomic operations**: Use tempfile + rename pattern for all file writes
8. **Performance**: Pre-compile schemas, use `&str` where possible, minimize allocations
9. **Security**: Validate paths, check dangerous commands (rm -rf, .env access)

## 🔧 Development Commands

```bash
# Create feature branch
git checkout -b feature/issue-43/json-schemas-integration

# Run tests in watch mode
cargo watch -x "test --package maos-core messages"

# Run specific test module
cargo test messages_tests

# Run benchmarks
cargo bench messages

# Check Python compatibility
python -m pytest .claude/hooks/maos/tests/

# Check coverage
cargo tarpaulin --all-features --out Html

# Full validation
just fmt && just test && just clippy-fix
```

## 📋 TDD Task Checklist

- [ ] Setup module structure and dependencies
- [ ] Write failing tests for HookInput enum (both formats)
- [ ] Implement HookInput with untagged enum
- [ ] Write failing tests for SessionContext extraction
- [ ] Implement SessionContext from both formats
- [ ] Write failing tests for PathConstraint validation
- [ ] Implement PathConstraint with security checks
- [ ] Write failing tests for HookResponse
- [ ] Implement HookResponse with exit codes
- [ ] Write failing tests for NotificationMessage
- [ ] Implement NotificationMessage with TTS formatting
- [ ] Write failing tests for session state files
- [ ] Implement session state matching Python structure
- [ ] Write failing tests for schema validation
- [ ] Implement SchemaValidator with both formats
- [ ] Add integration tests for Python compatibility
- [ ] Add performance benchmarks
- [ ] Document all public APIs with examples
- [ ] Test with actual Claude Code hooks
- [ ] Run full test suite and fix any issues
- [ ] Create PR with comprehensive description

## 🚨 Critical Security Considerations

From Python hook analysis, we must handle:
1. **Dangerous rm commands**: Block `rm -rf /`, `rm -rf ~`, etc.
2. **.env file access**: Block access to `.env` files (but allow `.env.sample` and `stack.env`)
3. **Path constraints**: Enforce workspace isolation for agents
4. **File locks**: Prevent concurrent edits to same file

## 📚 References

- Issue #43: https://github.com/[org]/maos/issues/43
- Claude Code Hooks Docs: https://docs.anthropic.com/en/docs/claude-code/hooks
- Python Hook Implementation: `.claude/hooks/maos/`
- Existing Types: `crates/maos-core/src/types/`
- PRD-01: `docs/prd/PRD-01-common-foundation.md` (lines 286-363)

## 🔄 Future Considerations

1. **Hook versioning**: May need to support v1, v2 formats in future
2. **Streaming support**: For large tool responses
3. **Batch operations**: Multiple tool calls in one hook
4. **Custom tool support**: Extensible tool definitions
5. **WebSocket support**: Real-time hook communication

---

This plan ensures we handle the **actual** message formats used by our Python hooks while also supporting the documented Claude Code format for future compatibility. The strict TDD approach with Red/Green/Refactor cycles ensures high quality, idiomatic Rust code with comprehensive test coverage.