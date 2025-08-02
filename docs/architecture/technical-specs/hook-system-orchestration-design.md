# Hook System Design for Claude Code Sub-Agent Orchestration

## Executive Summary

This document presents a comprehensive hook system architecture for orchestrating Claude Code sub-agents in MAOS. The design balances security, performance, and coordination needs while keeping implementation complexity appropriate for a single developer's machine.

**Key Design Principles**:
- Minimal overhead (sub-10ms per hook)
- Progressive security (not over-engineered)
- Coordination-first approach
- Native Claude Code integration

## Architecture Overview

### Hook System Layers

```
┌─────────────────────────────────────────────────────┐
│                   Application Layer                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │   Agent 1   │  │   Agent 2   │  │   Agent 3   │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘ │
│         │                 │                 │        │
├─────────┴─────────────────┴─────────────────┴───────┤
│                    Hook Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │  Pre-Tool   │  │ Post-Tool   │  │User Prompt  │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘ │
│         │                 │                 │        │
├─────────┴─────────────────┴─────────────────┴───────┤
│                Coordination Layer                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │Lock Manager │  │Progress Mgr │  │Comm Channel │ │
│  └─────────────┘  └─────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────┘
```

## Essential Hooks for Orchestration

### 1. Pre-Tool Hooks

**Purpose**: Prevent conflicts, validate operations, coordinate resource access

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PreToolHook {
    pub name: &'static str,
    pub priority: i32,
    pub handler: fn(&ToolRequest) -> HookResult,
}

// Essential pre-tool hooks
pub const PRE_TOOL_HOOKS: &[PreToolHook] = &[
    PreToolHook {
        name: "file_lock_check",
        priority: 100,  // Highest priority
        handler: check_file_locks,
    },
    PreToolHook {
        name: "resource_allocation",
        priority: 90,
        handler: allocate_resources,
    },
    PreToolHook {
        name: "conflict_detection",
        priority: 80,
        handler: detect_conflicts,
    },
    PreToolHook {
        name: "permission_validation",
        priority: 70,
        handler: validate_permissions,
    },
];
```

### 2. Post-Tool Hooks

**Purpose**: Update coordination state, track progress, release resources

```rust
pub const POST_TOOL_HOOKS: &[PostToolHook] = &[
    PostToolHook {
        name: "progress_update",
        priority: 100,
        handler: update_progress,
    },
    PostToolHook {
        name: "resource_release",
        priority: 90,
        handler: release_resources,
    },
    PostToolHook {
        name: "state_synchronization",
        priority: 80,
        handler: sync_agent_state,
    },
    PostToolHook {
        name: "audit_logging",
        priority: 70,
        handler: log_operation,
    },
];
```

### 3. User Prompt Hooks

**Purpose**: Task assignment, agent coordination, security filtering

```rust
pub const USER_PROMPT_HOOKS: &[UserPromptHook] = &[
    UserPromptHook {
        name: "task_extraction",
        priority: 100,
        handler: extract_tasks,
    },
    UserPromptHook {
        name: "agent_assignment",
        priority: 90,
        handler: assign_to_agents,
    },
    UserPromptHook {
        name: "security_filter",
        priority: 80,
        handler: filter_dangerous_prompts,
    },
];
```

## Coordination Patterns

### 1. File Lock Coordination

```rust
use dashmap::DashMap;
use std::time::{Duration, Instant};

pub struct FileLockCoordinator {
    locks: Arc<DashMap<PathBuf, LockInfo>>,
    timeout: Duration,
}

#[derive(Debug, Clone)]
struct LockInfo {
    agent_id: String,
    acquired_at: Instant,
    operation: String,
}

impl FileLockCoordinator {
    pub async fn check_file_locks(request: &ToolRequest) -> HookResult {
        match request.tool_name.as_str() {
            "Write" | "Edit" | "MultiEdit" => {
                let file_path = extract_file_path(request)?;
                
                // Check if file is locked by another agent
                if let Some(lock_info) = self.locks.get(&file_path) {
                    if lock_info.agent_id != request.agent_id {
                        return HookResult::Block {
                            reason: format!(
                                "File {} is locked by agent {} for {}",
                                file_path.display(),
                                lock_info.agent_id,
                                lock_info.operation
                            ),
                        };
                    }
                }
                
                // Acquire lock
                self.locks.insert(file_path.clone(), LockInfo {
                    agent_id: request.agent_id.clone(),
                    acquired_at: Instant::now(),
                    operation: request.tool_name.clone(),
                });
                
                HookResult::Allow
            }
            _ => HookResult::Allow,
        }
    }
    
    pub async fn release_locks(response: &ToolResponse) -> HookResult {
        if let Some(file_path) = extract_file_path_from_response(response) {
            self.locks.remove(&file_path);
        }
        HookResult::Allow
    }
}
```

### 2. Progress Tracking

```rust
pub struct ProgressTracker {
    tasks: Arc<DashMap<TaskId, TaskProgress>>,
    subscribers: Arc<RwLock<Vec<ProgressSubscriber>>>,
}

#[derive(Debug, Clone)]
struct TaskProgress {
    task_id: TaskId,
    agent_id: String,
    status: TaskStatus,
    steps_completed: u32,
    total_steps: u32,
    last_update: Instant,
}

impl ProgressTracker {
    pub async fn update_progress(response: &ToolResponse) -> HookResult {
        if let Some(task_id) = extract_task_id(response) {
            self.tasks.alter(&task_id, |_, mut progress| {
                progress.steps_completed += 1;
                progress.last_update = Instant::now();
                
                // Notify subscribers
                self.notify_subscribers(&progress).await;
                
                progress
            });
        }
        HookResult::Allow
    }
    
    async fn notify_subscribers(&self, progress: &TaskProgress) {
        let subscribers = self.subscribers.read().await;
        for subscriber in subscribers.iter() {
            subscriber.notify(progress).await.ok();
        }
    }
}
```

### 3. Inter-Agent Communication

```rust
pub struct AgentCommunicator {
    channels: Arc<DashMap<String, MessageChannel>>,
    broadcast: Arc<broadcast::Sender<AgentMessage>>,
}

impl AgentCommunicator {
    pub async fn handle_communication(request: &ToolRequest) -> HookResult {
        // Check if this is a communication request
        if request.tool_name == "Write" {
            if let Some(path) = extract_file_path(request) {
                if path.starts_with(".maos/messages/") {
                    // This is an inter-agent message
                    let message = parse_agent_message(request)?;
                    
                    // Route to appropriate channel
                    if let Some(channel) = self.channels.get(&message.target_agent) {
                        channel.send(message).await?;
                    }
                    
                    // Also broadcast for monitoring
                    self.broadcast.send(message.clone()).ok();
                }
            }
        }
        HookResult::Allow
    }
}
```

## Security Guidelines

### 1. Minimal Security Layer

```rust
pub struct SecurityValidator {
    workspace_root: PathBuf,
    allowed_commands: HashSet<String>,
    sensitive_patterns: Vec<Regex>,
}

impl SecurityValidator {
    pub async fn validate_operation(request: &ToolRequest) -> HookResult {
        match request.tool_name.as_str() {
            "Bash" => self.validate_command(request),
            "Write" | "Edit" => self.validate_file_access(request),
            _ => HookResult::Allow,
        }
    }
    
    fn validate_command(&self, request: &ToolRequest) -> HookResult {
        let command = extract_command(request)?;
        let base_cmd = command.split_whitespace().next().unwrap_or("");
        
        // Block obviously dangerous commands
        const DANGEROUS_COMMANDS: &[&str] = &[
            "rm -rf /",
            "mkfs",
            "dd if=/dev/zero",
            ":(){ :|:& };:",  // Fork bomb
        ];
        
        for dangerous in DANGEROUS_COMMANDS {
            if command.contains(dangerous) {
                return HookResult::Block {
                    reason: format!("Dangerous command pattern detected: {}", dangerous),
                };
            }
        }
        
        // Allow everything else (developer machine context)
        HookResult::Allow
    }
    
    fn validate_file_access(&self, request: &ToolRequest) -> HookResult {
        let file_path = extract_file_path(request)?;
        
        // Ensure path is within workspace
        if !file_path.starts_with(&self.workspace_root) {
            return HookResult::Block {
                reason: "File access outside workspace not allowed".to_string(),
            };
        }
        
        // Check for sensitive files
        const SENSITIVE_PATTERNS: &[&str] = &[
            ".env",
            ".ssh/",
            ".aws/credentials",
            ".git/config",
        ];
        
        for pattern in SENSITIVE_PATTERNS {
            if file_path.to_string_lossy().contains(pattern) {
                return HookResult::Modify {
                    reason: format!("Sensitive file access: {}", pattern),
                    modifications: json!({
                        "add_warning": "This file contains sensitive information"
                    }),
                };
            }
        }
        
        HookResult::Allow
    }
}
```

### 2. Resource Limits

```rust
pub struct ResourceLimiter {
    file_size_limit: usize,
    command_timeout: Duration,
    max_concurrent_operations: usize,
    current_operations: Arc<AtomicUsize>,
}

impl ResourceLimiter {
    pub async fn check_limits(request: &ToolRequest) -> HookResult {
        // Check concurrent operations
        let current = self.current_operations.load(Ordering::Relaxed);
        if current >= self.max_concurrent_operations {
            return HookResult::Block {
                reason: format!(
                    "Too many concurrent operations ({}/{})",
                    current, self.max_concurrent_operations
                ),
            };
        }
        
        // Check file size for write operations
        if request.tool_name == "Write" {
            if let Some(content) = extract_content(request) {
                if content.len() > self.file_size_limit {
                    return HookResult::Block {
                        reason: format!(
                            "File size {} exceeds limit of {}",
                            content.len(), self.file_size_limit
                        ),
                    };
                }
            }
        }
        
        // Increment operation counter
        self.current_operations.fetch_add(1, Ordering::Relaxed);
        
        HookResult::Allow
    }
}
```

## Performance Optimization

### 1. Fast Path Optimization

```rust
#[inline(always)]
pub fn fast_hook_check(request: &ToolRequest) -> Option<HookResult> {
    // Quick checks that don't need full processing
    match request.tool_name.as_str() {
        "Read" | "Grep" | "Glob" => Some(HookResult::Allow),  // Read-only, always safe
        "Bash" if request.is_simple_command() => Some(HookResult::Allow),
        _ => None,  // Needs full processing
    }
}

pub async fn process_hook(request: &ToolRequest) -> HookResult {
    // Try fast path first
    if let Some(result) = fast_hook_check(request) {
        return result;
    }
    
    // Full processing for complex cases
    let start = Instant::now();
    let result = full_hook_processing(request).await;
    
    // Log if slow
    if start.elapsed() > Duration::from_millis(10) {
        warn!("Slow hook processing: {:?}ms", start.elapsed().as_millis());
    }
    
    result
}
```

### 2. Caching Strategy

```rust
pub struct HookCache {
    cache: Arc<DashMap<u64, CachedDecision>>,
    ttl: Duration,
}

impl HookCache {
    pub fn get_cached_decision(&self, request: &ToolRequest) -> Option<HookResult> {
        let key = self.compute_cache_key(request);
        
        if let Some(cached) = self.cache.get(&key) {
            if cached.timestamp.elapsed() < self.ttl {
                return Some(cached.result.clone());
            }
        }
        
        None
    }
    
    fn compute_cache_key(&self, request: &ToolRequest) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        request.tool_name.hash(&mut hasher);
        
        // Only hash relevant fields for caching
        match request.tool_name.as_str() {
            "Read" | "Grep" => {
                request.get_path().hash(&mut hasher);
            }
            _ => return 0,  // Don't cache write operations
        }
        
        hasher.finish()
    }
}
```

### 3. Async Processing

```rust
pub struct AsyncHookProcessor {
    runtime: Arc<Runtime>,
    thread_pool: Arc<ThreadPool>,
}

impl AsyncHookProcessor {
    pub async fn process_hooks_parallel(
        &self,
        request: &ToolRequest,
        hooks: &[Hook],
    ) -> HookResult {
        let mut handles = Vec::new();
        
        // Spawn parallel tasks for independent hooks
        for hook in hooks {
            let request = request.clone();
            let hook = hook.clone();
            
            let handle = self.runtime.spawn(async move {
                hook.process(&request).await
            });
            
            handles.push(handle);
        }
        
        // Collect results
        for handle in handles {
            let result = handle.await?;
            
            // First block/modify wins
            match result {
                HookResult::Block { .. } | HookResult::Modify { .. } => {
                    return result;
                }
                HookResult::Allow => continue,
            }
        }
        
        HookResult::Allow
    }
}
```

## Implementation Patterns

### 1. Hook Configuration

```toml
# ~/.config/maos/hooks.toml

[hooks]
enabled = true
timeout_ms = 10
cache_ttl_seconds = 60

[hooks.pre_tool]
enabled = ["file_lock_check", "resource_allocation", "permission_validation"]
disabled = []  # Explicitly disabled hooks

[hooks.post_tool]
enabled = ["progress_update", "resource_release", "audit_logging"]
async = true  # Run post-hooks asynchronously

[hooks.user_prompt]
enabled = ["task_extraction", "security_filter"]
priority_override = { task_extraction = 150 }  # Override default priority

[coordination]
lock_timeout_seconds = 300
max_concurrent_agents = 5
progress_update_interval_ms = 1000

[security]
workspace_only = true
sensitive_file_warning = true
command_timeout_seconds = 30
```

### 2. Hook Binary Implementation

```rust
// maos-hooks/src/main.rs
use clap::Parser;
use serde_json;
use std::io::{self, Read};

#[derive(Parser)]
struct Args {
    #[clap(long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Read hook request from stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    
    let request: HookRequest = serde_json::from_str(&buffer)?;
    
    // Load configuration
    let config = load_config(args.config)?;
    
    // Process hook
    let processor = HookProcessor::new(config);
    let response = processor.process(request).await?;
    
    // Write response to stdout
    println!("{}", serde_json::to_string(&response)?);
    
    Ok(())
}
```

### 3. Claude Code Integration

```json
// Claude Code configuration
{
  "hooks": {
    "pre_tool_use": [
      {
        "matcher": ".*",
        "command": "/usr/local/bin/maos-hooks",
        "args": ["--config", "~/.config/maos/hooks.toml"],
        "timeout": 0.01
      }
    ],
    "post_tool_use": [
      {
        "matcher": ".*",
        "command": "/usr/local/bin/maos-hooks",
        "async": true
      }
    ],
    "user_prompt_submit": [
      {
        "command": "/usr/local/bin/maos-hooks"
      }
    ]
  }
}
```

## Hook Chaining and Composition

### 1. Chain of Responsibility

```rust
pub struct HookChain {
    hooks: Vec<Box<dyn Hook>>,
}

impl HookChain {
    pub async fn process(&self, request: &ToolRequest) -> HookResult {
        let mut context = HookContext::new(request);
        
        for hook in &self.hooks {
            let result = hook.process(&mut context).await?;
            
            match result {
                HookResult::Allow => continue,
                HookResult::Block { .. } => return result,
                HookResult::Modify { modifications } => {
                    context.apply_modifications(modifications);
                }
            }
        }
        
        HookResult::Allow
    }
}
```

### 2. Conditional Hooks

```rust
pub struct ConditionalHook {
    condition: Box<dyn Fn(&ToolRequest) -> bool>,
    hook: Box<dyn Hook>,
}

impl ConditionalHook {
    pub fn new<F>(condition: F, hook: Box<dyn Hook>) -> Self
    where
        F: Fn(&ToolRequest) -> bool + 'static,
    {
        Self {
            condition: Box::new(condition),
            hook,
        }
    }
}

// Usage
let file_hook = ConditionalHook::new(
    |req| req.tool_name == "Write" || req.tool_name == "Edit",
    Box::new(FileLockHook::new()),
);
```

## Error Handling

### 1. Graceful Degradation

```rust
pub async fn process_with_fallback(request: &ToolRequest) -> HookResult {
    match tokio::time::timeout(Duration::from_millis(10), process_hook(request)).await {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => {
            error!("Hook processing error: {}", e);
            HookResult::Allow  // Fail open on developer machine
        }
        Err(_) => {
            warn!("Hook timeout, allowing operation");
            HookResult::Allow
        }
    }
}
```

### 2. Error Recovery

```rust
pub struct HookErrorRecovery {
    retry_count: usize,
    backoff: Duration,
}

impl HookErrorRecovery {
    pub async fn process_with_retry(&self, request: &ToolRequest) -> HookResult {
        let mut attempts = 0;
        let mut backoff = self.backoff;
        
        loop {
            match process_hook(request).await {
                Ok(result) => return result,
                Err(e) if attempts < self.retry_count => {
                    attempts += 1;
                    warn!("Hook error (attempt {}/{}): {}", attempts, self.retry_count, e);
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                }
                Err(e) => {
                    error!("Hook failed after {} attempts: {}", attempts, e);
                    return HookResult::Allow;  // Fail open
                }
            }
        }
    }
}
```

## Monitoring and Debugging

### 1. Hook Metrics

```rust
pub struct HookMetrics {
    execution_time: Histogram,
    decision_counter: IntCounterVec,
    error_counter: IntCounter,
}

impl HookMetrics {
    pub fn record(&self, hook_name: &str, duration: Duration, result: &HookResult) {
        self.execution_time
            .with_label_values(&[hook_name])
            .observe(duration.as_secs_f64());
        
        let decision = match result {
            HookResult::Allow => "allow",
            HookResult::Block { .. } => "block",
            HookResult::Modify { .. } => "modify",
        };
        
        self.decision_counter
            .with_label_values(&[hook_name, decision])
            .inc();
    }
}
```

### 2. Debug Mode

```rust
pub struct DebugHookWrapper {
    inner: Box<dyn Hook>,
    debug_output: bool,
}

impl DebugHookWrapper {
    pub async fn process(&self, request: &ToolRequest) -> HookResult {
        if self.debug_output {
            eprintln!("🪝 Processing hook for: {}", request.tool_name);
            eprintln!("   Request: {:?}", request);
        }
        
        let start = Instant::now();
        let result = self.inner.process(request).await;
        let duration = start.elapsed();
        
        if self.debug_output {
            eprintln!("   Result: {:?}", result);
            eprintln!("   Duration: {:?}", duration);
        }
        
        result
    }
}
```

## Conclusion

This hook system design provides:

1. **Essential Coordination**: File locking, progress tracking, and inter-agent communication
2. **Proportional Security**: Appropriate for developer machines without over-engineering
3. **High Performance**: Sub-10ms overhead with caching and fast paths
4. **Native Integration**: Works seamlessly with Claude Code's existing hook system

The architecture is extensible and can grow with MAOS needs while maintaining simplicity and performance.