# Learnings from Claude-Flow and Claude Code SDK for MAOS Design

## Executive Summary

After analyzing claude-flow and Claude Code documentation, we've identified critical patterns and anti-patterns that should guide MAOS development. The key insight: **avoid simulation and focus on real execution with proper governance**.

## Critical Anti-Patterns to Avoid (From Claude-Flow)

### 1. "Bridge to Nowhere" Code
```typescript
// DON'T DO THIS - claude-flow's simulated execution
private async simulateTaskExecution(task: SwarmTask): Promise<any> {
    setTimeout(() => resolve({result: "fake success"}), Math.random() * 5000);
}
```

**MAOS Approach**: Only implement real functionality. If something isn't ready, throw "Not Implemented" errors rather than fake results.

### 2. Overlapping State Management
Claude-flow tracks agents in multiple places (agent-manager, swarm-coordinator, orchestrator) leading to inconsistencies.

**MAOS Approach**: Single source of truth using event sourcing pattern.

### 3. Insufficient Quality Control
Tasks marked "complete" without validation of outputs.

**MAOS Approach**: Implement quality gates with validation before task completion.

## Valuable Patterns to Adopt

### 1. Circuit Breaker Pattern (From Claude-Flow)
```rust
pub struct CircuitBreaker {
    state: State,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

enum State {
    Closed,      // Normal operation
    Open,        // Failing, reject requests
    HalfOpen,    // Testing if service recovered
}
```

### 2. SDK Patterns (From Claude Code)

#### Task Execution with Limits
```rust
pub struct TaskConfig {
    max_turns: Option<u32>,      // Limit agent iterations
    timeout: Duration,           // Hard timeout
    output_format: OutputFormat, // text, json, stream
}
```

#### Session Management
```rust
pub struct Session {
    id: Uuid,
    context: Vec<Message>,
    model: Model,
    tools: ToolPermissions,
}

// Support resumption
impl Session {
    pub fn resume(id: Uuid) -> Result<Self> { }
    pub fn continue_with(&mut self, prompt: String) -> Result<Response> { }
}
```

### 3. Configuration Hierarchy (From Claude Code)
```
1. Enterprise policies (highest priority)
2. Command line flags
3. Local project settings
4. Shared project settings  
5. User settings (lowest priority)
```

## MAOS Architecture Recommendations

### 1. Real Process Execution
```rust
// MAOS: Actual Claude process spawning
pub async fn execute_task(task: Task) -> Result<TaskOutput> {
    let process = Command::new("claude")
        .arg("--max-turns").arg(task.config.max_turns.to_string())
        .arg("--format").arg("json")
        .arg("--message").arg(build_task_prompt(&task))
        .stdout(Stdio::piped())
        .spawn()?;
        
    // Monitor real process
    let output = tokio::time::timeout(
        task.config.timeout,
        process.wait_with_output()
    ).await??;
    
    // Validate real output
    validate_task_output(&output)?
}
```

### 2. Quality Governance Framework
```rust
pub trait QualityGate {
    async fn validate(&self, output: &TaskOutput) -> Result<ValidationResult>;
}

pub struct GovernanceEngine {
    gates: Vec<Box<dyn QualityGate>>,
}

impl GovernanceEngine {
    pub async fn check_output(&self, output: &TaskOutput) -> Result<()> {
        for gate in &self.gates {
            match gate.validate(output).await? {
                ValidationResult::Pass => continue,
                ValidationResult::Fail(reason) => {
                    return Err(GovernanceError::QualityGateFailed(reason));
                }
                ValidationResult::NeedsReview => {
                    // Trigger peer review by another agent
                    self.request_peer_review(output).await?;
                }
            }
        }
        Ok(())
    }
}
```

### 3. Tool Permission System
```rust
// From Claude Code's security model
pub struct ToolPermissions {
    allowed: HashSet<ToolType>,
    denied: HashSet<ToolType>,
    require_confirmation: HashSet<ToolType>,
}

impl ToolPermissions {
    pub fn check(&self, tool: &ToolType) -> PermissionResult {
        if self.denied.contains(tool) {
            return PermissionResult::Denied;
        }
        if self.require_confirmation.contains(tool) {
            return PermissionResult::RequiresConfirmation;
        }
        if self.allowed.contains(tool) {
            return PermissionResult::Allowed;
        }
        PermissionResult::Denied // Default deny
    }
}
```

### 4. Structured Communication
```rust
// Avoid claude-flow's loose event bus
pub enum AgentMessage {
    TaskAssignment { task: Task, deadline: Instant },
    StatusUpdate { task_id: Uuid, status: TaskStatus },
    OutputReady { task_id: Uuid, output: TaskOutput },
    QualityCheckRequest { output: TaskOutput, reviewer: AgentId },
}

// Type-safe communication channels
pub struct AgentChannel {
    tx: mpsc::Sender<AgentMessage>,
    rx: mpsc::Receiver<AgentMessage>,
}
```

### 5. Real Metrics and Monitoring
```rust
pub struct TaskMetrics {
    start_time: Instant,
    end_time: Option<Instant>,
    token_usage: TokenUsage,
    quality_score: Option<f64>,
    validation_results: Vec<ValidationResult>,
    actual_output: String, // Not simulated!
}
```

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- Real Claude process spawning
- Basic task execution with timeouts
- Simple quality validation (non-empty output)
- Session management

### Phase 2: Governance (Week 3-4)
- Quality gate framework
- Peer review mechanism
- Output validation rules
- Metrics collection

### Phase 3: Orchestration (Week 5-6)
- Parallel task execution
- Dependency management
- Circuit breaker implementation
- Tool permission system

## Key Principles for MAOS

1. **No Fake Results**: Every task execution must spawn real processes
2. **Validate Everything**: Outputs must pass quality gates
3. **Type Safety**: Use Rust's type system to prevent errors
4. **Clear Ownership**: Single source of truth for state
5. **Observable**: Rich metrics and logging
6. **Secure by Default**: Explicit tool permissions

## Configuration Example

```toml
# maos.toml
[orchestration]
max_concurrent_agents = 4
default_timeout = "5m"
retry_attempts = 3

[quality]
min_output_length = 100
require_structured_output = true
peer_review_threshold = 0.8

[tools]
allowed = ["read", "write", "search"]
denied = ["bash", "web_fetch"]
require_confirmation = ["edit"]

[models]
default = "claude-3-opus"
fallback = "claude-3-sonnet"
```

## Avoiding Common Pitfalls

1. **Don't Mock in Production Code**
   - Test with real processes or clearly separate test code
   
2. **Don't Hide Failures**
   - Surface all errors with context
   - Never simulate success

3. **Don't Overcomplicate State**
   - Event sourcing provides audit trail
   - Keep runtime state minimal

4. **Don't Trust Without Verification**
   - Validate all agent outputs
   - Implement peer review for critical tasks

## Conclusion

By learning from claude-flow's mistakes and Claude Code's patterns, MAOS can deliver a robust multi-agent orchestration system that produces real, validated results with proper governance. The key is to start simple with real execution and build governance incrementally.