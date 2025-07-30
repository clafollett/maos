# MAOS Rust Hooks System Design

## Overview

The Rust hooks system replaces the Python-based security hooks with a high-performance, capability-based security model. It provides fine-grained control over Claude Code's tool usage while maintaining near-zero overhead.

## Architecture

### Hook System Components

```rust
maos-hooks/
├── src/
│   ├── lib.rs               // Library interface
│   ├── main.rs              // Standalone hook binary
│   ├── core/
│   │   ├── mod.rs           // Core types and traits
│   │   ├── event.rs         // Hook event definitions
│   │   ├── response.rs      // Hook response types
│   │   └── context.rs       // Execution context
│   ├── security/
│   │   ├── mod.rs           // Security module
│   │   ├── capability.rs    // Capability definitions
│   │   ├── policy.rs        // Security policies
│   │   ├── validator.rs     // Validation engine
│   │   └── rules/           // Validation rules
│   │       ├── mod.rs
│   │       ├── filesystem.rs
│   │       ├── execution.rs
│   │       ├── network.rs
│   │       └── content.rs
│   ├── performance/
│   │   ├── mod.rs           // Performance optimizations
│   │   ├── cache.rs         // Decision caching
│   │   ├── batch.rs         // Batch validation
│   │   └── metrics.rs       // Performance metrics
│   └── audit/
│       ├── mod.rs           // Audit system
│       ├── logger.rs        // Structured logging
│       ├── storage.rs       // Log storage
│       └── analysis.rs      // Log analysis
```

## Core Types

### Capability Model

```rust
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Capabilities: u32 {
        // Filesystem capabilities
        const READ_WORKSPACE    = 0b00000001;
        const WRITE_WORKSPACE   = 0b00000010;
        const READ_SYSTEM       = 0b00000100;
        const WRITE_SYSTEM      = 0b00001000;
        
        // Execution capabilities
        const EXECUTE_SAFE      = 0b00010000;
        const EXECUTE_SHELL     = 0b00100000;
        const EXECUTE_SUDO      = 0b01000000;
        
        // Network capabilities
        const NETWORK_LOCAL     = 0b00100000;
        const NETWORK_EXTERNAL  = 0b01000000;
        const NETWORK_LISTEN    = 0b10000000;
        
        // Special capabilities
        const ACCESS_SECRETS    = 0b000100000000;
        const MODIFY_SYSTEM     = 0b001000000000;
        const DEBUG_TOOLS       = 0b010000000000;
        
        // Predefined sets
        const BASIC = Self::READ_WORKSPACE.bits() | Self::WRITE_WORKSPACE.bits();
        const STANDARD = Self::BASIC.bits() | Self::EXECUTE_SAFE.bits() | Self::READ_SYSTEM.bits();
        const EXTENDED = Self::STANDARD.bits() | Self::NETWORK_LOCAL.bits() | Self::EXECUTE_SHELL.bits();
        const FULL = 0xFFFFFFFF;
    }
}
```

### Hook Events

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hook_event_name")]
pub enum HookEvent {
    PreToolUse {
        tool_name: String,
        tool_input: serde_json::Value,
        context: ExecutionContext,
    },
    PostToolUse {
        tool_name: String,
        tool_input: serde_json::Value,
        tool_response: serde_json::Value,
        context: ExecutionContext,
    },
    UserPromptSubmit {
        prompt: String,
        context: ExecutionContext,
    },
    ConversationStart {
        metadata: HashMap<String, String>,
    },
    ConversationEnd {
        summary: ConversationSummary,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub session_id: String,
    pub agent_type: Option<String>,
    pub workspace_root: String,
    pub timestamp: u64,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResponse {
    pub decision: Decision,
    pub reason: Option<String>,
    pub modifications: Option<serde_json::Value>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Allow,
    Block,
    Modify,
}
```

## Security Policies

### Policy Definition

```rust
use std::path::PathBuf;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub name: String,
    pub description: String,
    pub capabilities: Capabilities,
    pub rules: PolicyRules,
    pub metadata: PolicyMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRules {
    pub filesystem: FilesystemRules,
    pub execution: ExecutionRules,
    pub network: NetworkRules,
    pub content: ContentRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemRules {
    pub allowed_paths: Vec<PathPattern>,
    pub denied_paths: Vec<PathPattern>,
    pub sensitive_patterns: Vec<Regex>,
    pub max_file_size: usize,
    pub allowed_extensions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRules {
    pub allowed_commands: Vec<CommandPattern>,
    pub denied_commands: Vec<CommandPattern>,
    pub environment_whitelist: Vec<String>,
    pub max_execution_time: u64,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone)]
pub enum PathPattern {
    Exact(PathBuf),
    Prefix(PathBuf),
    Glob(glob::Pattern),
    Regex(Regex),
}

impl PolicyManager {
    pub fn load_policy(name: &str) -> Result<SecurityPolicy> {
        match name {
            "strict" => Ok(Self::strict_policy()),
            "standard" => Ok(Self::standard_policy()),
            "permissive" => Ok(Self::permissive_policy()),
            custom => Self::load_custom_policy(custom),
        }
    }
    
    fn strict_policy() -> SecurityPolicy {
        SecurityPolicy {
            name: "strict".to_string(),
            description: "Minimal capabilities for safe operations".to_string(),
            capabilities: Capabilities::BASIC,
            rules: PolicyRules {
                filesystem: FilesystemRules {
                    allowed_paths: vec![
                        PathPattern::Prefix(PathBuf::from(".")),
                    ],
                    denied_paths: vec![
                        PathPattern::Glob(glob::Pattern::new("**/.env*").unwrap()),
                        PathPattern::Glob(glob::Pattern::new("**/.git/config").unwrap()),
                        PathPattern::Regex(Regex::new(r".*\.(key|pem|crt)$").unwrap()),
                    ],
                    sensitive_patterns: vec![
                        Regex::new(r"(?i)(api[_-]?key|password|secret|token)").unwrap(),
                    ],
                    max_file_size: 10 * 1024 * 1024, // 10MB
                    allowed_extensions: None,
                },
                execution: ExecutionRules {
                    allowed_commands: vec![
                        CommandPattern::Exact("ls"),
                        CommandPattern::Exact("cat"),
                        CommandPattern::Exact("grep"),
                        CommandPattern::Exact("echo"),
                        CommandPattern::Prefix("git"),
                    ],
                    denied_commands: vec![
                        CommandPattern::Regex(Regex::new(r"rm\s+-rf").unwrap()),
                        CommandPattern::Exact("sudo"),
                    ],
                    environment_whitelist: vec!["PATH", "HOME", "USER"],
                    max_execution_time: 30,
                    resource_limits: Default::default(),
                },
                // ... other rules
            },
            metadata: Default::default(),
        }
    }
}
```

## Validation Engine

### High-Performance Validator

```rust
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::RwLock;

pub struct SecurityValidator {
    policy: Arc<SecurityPolicy>,
    cache: Arc<DashMap<u64, CachedDecision>>,
    metrics: Arc<RwLock<ValidationMetrics>>,
}

#[derive(Clone)]
struct CachedDecision {
    decision: Decision,
    reason: Option<String>,
    timestamp: u64,
}

impl SecurityValidator {
    pub async fn validate(&self, event: &HookEvent) -> Result<HookResponse> {
        // Check cache first
        let cache_key = self.compute_cache_key(event);
        if let Some(cached) = self.cache.get(&cache_key) {
            if cached.timestamp + CACHE_TTL > current_timestamp() {
                self.metrics.write().await.cache_hits += 1;
                return Ok(HookResponse {
                    decision: cached.decision,
                    reason: cached.reason.clone(),
                    modifications: None,
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Perform validation
        let start = std::time::Instant::now();
        let result = match event {
            HookEvent::PreToolUse { tool_name, tool_input, context } => {
                self.validate_tool_use(tool_name, tool_input, context).await
            },
            HookEvent::UserPromptSubmit { prompt, context } => {
                self.validate_prompt(prompt, context).await
            },
            _ => Ok(HookResponse::allow()),
        };
        
        // Update metrics
        self.metrics.write().await.validation_time += start.elapsed();
        
        // Cache decision
        if let Ok(ref response) = result {
            self.cache.insert(cache_key, CachedDecision {
                decision: response.decision,
                reason: response.reason.clone(),
                timestamp: current_timestamp(),
            });
        }
        
        result
    }
    
    async fn validate_tool_use(
        &self,
        tool_name: &str,
        tool_input: &serde_json::Value,
        context: &ExecutionContext,
    ) -> Result<HookResponse> {
        // Parallel validation of different aspects
        let (fs_result, exec_result, content_result) = tokio::join!(
            self.validate_filesystem_access(tool_name, tool_input),
            self.validate_execution(tool_name, tool_input),
            self.validate_content(tool_name, tool_input),
        );
        
        // Combine results
        if let Err(reason) = fs_result {
            return Ok(HookResponse::block(reason));
        }
        if let Err(reason) = exec_result {
            return Ok(HookResponse::block(reason));
        }
        if let Err(reason) = content_result {
            return Ok(HookResponse::block(reason));
        }
        
        Ok(HookResponse::allow())
    }
}
```

### Validation Rules

```rust
// filesystem.rs
pub struct FilesystemValidator<'a> {
    rules: &'a FilesystemRules,
    capabilities: Capabilities,
}

impl<'a> FilesystemValidator<'a> {
    pub fn validate_path_access(
        &self,
        path: &str,
        access_type: AccessType,
    ) -> Result<(), String> {
        let path = Path::new(path);
        
        // Check capabilities
        match access_type {
            AccessType::Read => {
                if !self.capabilities.contains(Capabilities::READ_WORKSPACE) {
                    if !self.is_workspace_path(path) {
                        return Err("Read access outside workspace not permitted".into());
                    }
                }
            }
            AccessType::Write => {
                if !self.capabilities.contains(Capabilities::WRITE_WORKSPACE) {
                    return Err("Write access not permitted".into());
                }
            }
        }
        
        // Check against denied paths
        for pattern in &self.rules.denied_paths {
            if pattern.matches(path) {
                return Err(format!("Access to {} is denied", path.display()));
            }
        }
        
        // Check against allowed paths
        let mut allowed = false;
        for pattern in &self.rules.allowed_paths {
            if pattern.matches(path) {
                allowed = true;
                break;
            }
        }
        
        if !allowed && !self.rules.allowed_paths.is_empty() {
            return Err(format!("Path {} not in allowed list", path.display()));
        }
        
        Ok(())
    }
    
    pub fn validate_content(
        &self,
        content: &str,
        file_path: Option<&str>,
    ) -> Result<(), String> {
        // Check for sensitive patterns
        for pattern in &self.rules.sensitive_patterns {
            if pattern.is_match(content) {
                return Err("Content contains sensitive information".into());
            }
        }
        
        // Check file size
        if content.len() > self.rules.max_file_size {
            return Err(format!(
                "Content exceeds maximum size of {} bytes",
                self.rules.max_file_size
            ));
        }
        
        Ok(())
    }
}
```

## Performance Optimizations

### 1. Zero-Allocation Parsing

```rust
use nom::{
    bytes::complete::{tag, take_while},
    character::complete::space1,
    IResult,
};

pub fn parse_command_zero_alloc(input: &str) -> IResult<&str, CommandInfo> {
    let (input, command) = take_while(|c: char| !c.is_whitespace())(input)?;
    let (input, _) = space1(input)?;
    let (input, args) = take_while(|_| true)(input)?;
    
    Ok((input, CommandInfo {
        command,
        args,
        full_command: input,
    }))
}
```

### 2. Batch Validation

```rust
pub struct BatchValidator {
    validator: Arc<SecurityValidator>,
    batch_size: usize,
}

impl BatchValidator {
    pub async fn validate_batch(
        &self,
        events: Vec<HookEvent>,
    ) -> Vec<Result<HookResponse>> {
        use futures::stream::{self, StreamExt};
        
        stream::iter(events)
            .chunks(self.batch_size)
            .flat_map(|chunk| {
                stream::iter(chunk).map(|event| {
                    let validator = self.validator.clone();
                    async move { validator.validate(&event).await }
                })
            })
            .buffer_unordered(10)
            .collect()
            .await
    }
}
```

### 3. Compiled Regex Cache

```rust
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Mutex;

static REGEX_CACHE: Lazy<Mutex<HashMap<String, Regex>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn get_or_compile_regex(pattern: &str) -> Result<Regex> {
    let mut cache = REGEX_CACHE.lock().unwrap();
    
    if let Some(regex) = cache.get(pattern) {
        return Ok(regex.clone());
    }
    
    let regex = Regex::new(pattern)?;
    cache.insert(pattern.to_string(), regex.clone());
    Ok(regex)
}
```

## Audit System

### Structured Logging

```rust
use tracing::{info, warn, error, instrument};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub session_id: String,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub decision: Decision,
    pub reason: Option<String>,
    pub duration_ns: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct AuditLogger {
    writer: Arc<Mutex<BufWriter<File>>>,
    buffer: Arc<Mutex<Vec<AuditEntry>>>,
}

impl AuditLogger {
    #[instrument(skip(self, entry))]
    pub async fn log(&self, entry: AuditEntry) {
        // Buffer entries for batch writing
        let mut buffer = self.buffer.lock().await;
        buffer.push(entry);
        
        if buffer.len() >= BATCH_SIZE {
            self.flush_buffer().await;
        }
    }
    
    async fn flush_buffer(&self) {
        let mut buffer = self.buffer.lock().await;
        let mut writer = self.writer.lock().await;
        
        for entry in buffer.drain(..) {
            if let Ok(json) = serde_json::to_string(&entry) {
                writeln!(writer, "{}", json).ok();
            }
        }
        
        writer.flush().ok();
    }
}
```

## Configuration

### Hook Configuration Format

```toml
# ~/.config/maos/hooks.toml

[global]
enabled = true
default_policy = "standard"
cache_ttl = 300
batch_size = 10

[policies.custom]
name = "project-specific"
base = "standard"
capabilities = ["read_system", "network_local"]

[[policies.custom.rules.filesystem.allowed_paths]]
type = "prefix"
path = "/usr/local/share"

[[policies.custom.rules.execution.allowed_commands]]
type = "regex"
pattern = "^(npm|yarn|pnpm)\\s+"

[audit]
enabled = true
path = "~/.maos/audit/hooks.jsonl"
rotation = "daily"
retention_days = 30

[performance]
enable_caching = true
max_cache_size = 10000
enable_metrics = true
metrics_interval = 60
```

## Integration with MAOS CLI

```rust
// In maos-cli
use maos_hooks::{HookEvent, SecurityValidator, PolicyManager};

pub async fn execute_with_hooks<F, T>(
    tool_name: &str,
    tool_input: serde_json::Value,
    operation: F,
) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    // Create hook event
    let event = HookEvent::PreToolUse {
        tool_name: tool_name.to_string(),
        tool_input: tool_input.clone(),
        context: ExecutionContext::current(),
    };
    
    // Validate with hooks
    let policy = PolicyManager::load_current()?;
    let validator = SecurityValidator::new(policy);
    let response = validator.validate(&event).await?;
    
    match response.decision {
        Decision::Allow => {
            // Execute operation
            let result = operation()?;
            
            // Post-execution hook
            let post_event = HookEvent::PostToolUse {
                tool_name: tool_name.to_string(),
                tool_input,
                tool_response: serde_json::to_value(&result)?,
                context: ExecutionContext::current(),
            };
            validator.validate(&post_event).await?;
            
            Ok(result)
        }
        Decision::Block => {
            Err(anyhow!("Operation blocked: {}", response.reason.unwrap_or_default()))
        }
        Decision::Modify => {
            // Apply modifications and retry
            todo!("Implement modification logic")
        }
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_filesystem_validation() {
        let policy = PolicyManager::strict_policy();
        let validator = SecurityValidator::new(Arc::new(policy));
        
        let event = HookEvent::PreToolUse {
            tool_name: "Write".to_string(),
            tool_input: json!({
                "file_path": "/etc/passwd",
                "content": "test"
            }),
            context: ExecutionContext::test(),
        };
        
        let response = validator.validate(&event).await.unwrap();
        assert_eq!(response.decision, Decision::Block);
    }
    
    #[tokio::test]
    async fn test_performance() {
        let policy = PolicyManager::standard_policy();
        let validator = Arc::new(SecurityValidator::new(Arc::new(policy)));
        
        // Generate 1000 events
        let events: Vec<_> = (0..1000)
            .map(|i| HookEvent::PreToolUse {
                tool_name: "Read".to_string(),
                tool_input: json!({ "file_path": format!("file_{}.txt", i) }),
                context: ExecutionContext::test(),
            })
            .collect();
        
        let start = std::time::Instant::now();
        let batch = BatchValidator::new(validator, 100);
        let results = batch.validate_batch(events).await;
        let duration = start.elapsed();
        
        println!("Validated 1000 events in {:?}", duration);
        assert!(duration.as_millis() < 100); // Should complete in under 100ms
    }
}
```

This Rust hooks system provides superior performance, type safety, and security compared to the Python implementation while maintaining compatibility with Claude Code's hook interface.