# PRD-03: Security Validation System

## Executive Summary

The MAOS Security Validation System provides comprehensive protection against dangerous operations while maintaining sub-10ms execution performance. This system acts as the primary security layer for all Claude Code tool executions, preventing destructive commands, protecting sensitive files, and enforcing workspace isolation through a high-performance, extensible rule engine.

**Key Deliverable**: A production-ready `maos-security` crate that validates all tool operations with zero false negatives for critical security violations while maintaining minimal false positives to preserve developer productivity.

## Problem Statement

Claude Code's AI agents can inadvertently execute dangerous operations that could:
- **Destroy Data**: Commands like `rm -rf /` or `rm -rf ~` can wipe entire systems
- **Expose Secrets**: Reading `.env` files could leak sensitive credentials to AI models
- **Break Repositories**: Destructive git operations could corrupt version control
- **Cross-Workspace Contamination**: Agents accessing files outside their assigned workspace
- **Path Traversal Attacks**: Maliciously crafted paths accessing system files

Without comprehensive security validation, MAOS users face unacceptable risks of accidental or malicious damage to their development environments.

## Goals & Success Metrics

### Primary Goals

1. **Zero Critical Security Failures**: Prevent all dangerous operations with 100% detection rate
2. **Ultra-Fast Validation**: Complete all security checks in <5ms (50% of total execution budget)
3. **Minimal False Positives**: <1% legitimate operations blocked incorrectly
4. **Extensible Rule System**: Support custom security policies per project
5. **Clear Security Feedback**: Provide actionable error messages for blocked operations

### Success Metrics

- **Security Coverage**: 100% of identified dangerous patterns blocked
- **Performance**: All validation operations complete in <5ms
- **False Positive Rate**: <1% of legitimate operations blocked
- **Rule Engine Efficiency**: O(1) average case for pattern matching
- **Memory Footprint**: <100KB memory usage during validation
- **Error Clarity**: 95% of users understand why operations were blocked

## User Personas & Use Cases

### Primary User: AI Agent (Claude Code)
**Profile**: Executes tool calls based on user instructions and context
**Use Case**: Attempts potentially dangerous operations without malicious intent
**Success Criteria**: Receives clear feedback about why operations are blocked and what alternatives exist

### Secondary User: Developer Using MAOS
**Profile**: Relies on MAOS to protect their development environment
**Use Case**: Wants comprehensive protection without productivity impact
**Success Criteria**: Works confidently knowing dangerous operations are prevented

### Tertiary User: Security-Conscious Organization
**Profile**: Requires customizable security policies for different projects
**Use Case**: Needs to enforce organization-specific security rules
**Success Criteria**: Can define and deploy custom security policies across teams

## Functional Requirements

### 1. Dangerous Command Detection

#### 1.1 Critical Command Patterns
```rust
/// High-priority dangerous commands that must be blocked
pub const CRITICAL_PATTERNS: &[&str] = &[
    // Recursive removal patterns
    r"rm\s+-[rf]*r[rf]*\s+/",           // rm -rf /
    r"rm\s+-[rf]*r[rf]*\s+/\*",         // rm -rf /*
    r"rm\s+-[rf]*r[rf]*\s+~",           // rm -rf ~
    r"rm\s+-[rf]*r[rf]*\s+\$HOME",      // rm -rf $HOME
    r"sudo\s+rm\s+-[rf]*r[rf]*",        // sudo rm -rf
    
    // System directory access
    r"rm\s+.*(/etc|/var|/usr|/bin|/sbin|/lib|/boot)",
    
    // Wildcard dangers
    r"rm\s+-[rf]*r[rf]*\s+\*",          // rm -rf *
    r"chmod\s+-R\s+000",                 // Make everything unreadable
    
    // Process killers
    r"kill\s+-9\s+-1",                  // Kill all processes
    r"pkill\s+-f\s+'.*'",               // Kill all matching processes
];

/// Command validation with context
#[derive(Debug, Clone)]
pub struct CommandValidation {
    pub command: String,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
    pub tool_name: String,
}

impl SecurityValidator {
    /// Validate command against dangerous patterns
    pub fn validate_command(&self, cmd: &CommandValidation) -> SecurityResult<()> {
        // Fast regex matching against critical patterns
        for pattern in CRITICAL_PATTERNS {
            if self.compiled_patterns[pattern].is_match(&cmd.command) {
                return Err(SecurityError::DangerousCommand {
                    command: cmd.command.clone(),
                    pattern: pattern.to_string(),
                    reason: self.get_pattern_explanation(pattern),
                    safe_alternative: self.suggest_alternative(&cmd.command),
                });
            }
        }
        Ok(())
    }
}
```

#### 1.2 Context-Aware Validation
```rust
/// Enhanced validation considering execution context
pub struct CommandContext {
    pub current_directory: PathBuf,
    pub session_id: SessionId,
    pub agent_id: AgentId,
    pub workspace_root: PathBuf,
    pub environment_vars: HashMap<String, String>,
}

impl SecurityValidator {
    /// Validate command with full context awareness
    pub fn validate_with_context(
        &self, 
        cmd: &str, 
        context: &CommandContext
    ) -> SecurityResult<ValidationResult> {
        let mut warnings = Vec::new();
        
        // 1. Critical pattern check
        self.validate_command(&CommandValidation {
            command: cmd.to_string(),
            working_directory: context.current_directory.clone(),
            environment: context.environment_vars.clone(),
            tool_name: "Bash".to_string(), // Tool type from context
        })?;
        
        // 2. Workspace boundary check
        if let Some(path_arg) = self.extract_path_argument(cmd) {
            self.validate_workspace_boundary(&path_arg, &context.workspace_root)?;
        }
        
        // 3. Environment variable expansion check
        let expanded_cmd = self.expand_environment_variables(cmd, &context.environment_vars);
        if expanded_cmd != cmd {
            self.validate_command(&CommandValidation {
                command: expanded_cmd,
                working_directory: context.current_directory.clone(),
                environment: context.environment_vars.clone(),
                tool_name: "Bash".to_string(),
            })?;
        }
        
        Ok(ValidationResult {
            allowed: true,
            warnings,
            modifications: None,
        })
    }
}
```

### 2. Environment File Protection

#### 2.1 Protected File Patterns
```rust
/// Environment file protection rules
#[derive(Debug, Clone)]
pub struct EnvFileProtection {
    /// Patterns for files that should be completely blocked
    blocked_patterns: Vec<Regex>,
    /// Patterns for files that are allowed (exceptions)
    allowed_patterns: Vec<Regex>,
    /// Custom project-specific blocks
    custom_blocks: Vec<String>,
}

impl Default for EnvFileProtection {
    fn default() -> Self {
        Self {
            blocked_patterns: vec![
                Regex::new(r"\.env$").unwrap(),
                Regex::new(r"\.env\.local$").unwrap(),
                Regex::new(r"\.env\.production$").unwrap(),
                Regex::new(r"\.env\.staging$").unwrap(),
                Regex::new(r"\.env\.development$").unwrap(),
                Regex::new(r"\.env\.test$").unwrap(),
                Regex::new(r".*\.key$").unwrap(),
                Regex::new(r".*\.pem$").unwrap(),
                Regex::new(r".*\.p12$").unwrap(),
                Regex::new(r"config/secrets\.yml$").unwrap(),
            ],
            allowed_patterns: vec![
                Regex::new(r"\.env\.example$").unwrap(),
                Regex::new(r"\.env\.sample$").unwrap(),
                Regex::new(r"\.env\.template$").unwrap(),
                Regex::new(r"stack\.env$").unwrap(),  // Allow MAOS stack.env
            ],
            custom_blocks: Vec::new(),
        }
    }
}

impl EnvFileProtection {
    /// Check if file access should be blocked
    pub fn is_file_blocked(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();
        
        // Check allowed patterns first (exceptions)
        for allowed_pattern in &self.allowed_patterns {
            if allowed_pattern.is_match(&path_str) {
                return false;
            }
        }
        
        // Check blocked patterns
        for blocked_pattern in &self.blocked_patterns {
            if blocked_pattern.is_match(&path_str) {
                return true;
            }
        }
        
        // Check custom blocks
        for custom_block in &self.custom_blocks {
            if path_str.contains(custom_block) {
                return true;
            }
        }
        
        false
    }
    
    /// Get explanation for why file is blocked
    pub fn get_block_reason(&self, file_path: &Path) -> String {
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
            
        match file_name {
            name if name.starts_with(".env") => {
                format!(
                    "Environment file '{}' contains sensitive configuration. \
                    Use '.env.example' or '.env.template' for documentation.",
                    file_name
                )
            },
            name if name.ends_with(".key") || name.ends_with(".pem") => {
                format!(
                    "Private key file '{}' contains sensitive cryptographic material.",
                    file_name
                )
            },
            _ => format!(
                "File '{}' is protected by security policy.",
                file_name
            )
        }
    }
}
```

#### 2.2 Tool-Specific File Access Validation
```rust
/// Validate file access based on tool type and operation
impl SecurityValidator {
    pub fn validate_file_access(
        &self,
        tool_call: &ToolCall,
        file_paths: &[PathBuf],
    ) -> SecurityResult<()> {
        for file_path in file_paths {
            // Check environment file protection
            if self.env_protection.is_file_blocked(file_path) {
                return Err(SecurityError::ProtectedFileAccess {
                    file_path: file_path.clone(),
                    tool_name: tool_call.tool_name.clone(),
                    reason: self.env_protection.get_block_reason(file_path),
                    suggestion: self.suggest_file_alternative(file_path),
                });
            }
            
            // Check workspace boundaries
            if let Some(workspace_root) = &tool_call.workspace_root {
                self.validate_workspace_boundary(file_path, workspace_root)?;
            }
            
            // Tool-specific validations
            match tool_call.tool_name.as_str() {
                "Read" | "Edit" | "Write" => {
                    self.validate_editor_access(file_path)?;
                },
                "Bash" => {
                    self.validate_bash_file_access(file_path)?;
                },
                _ => {
                    // Default validation for other tools
                }
            }
        }
        
        Ok(())
    }
    
    fn suggest_file_alternative(&self, blocked_path: &Path) -> Option<String> {
        let file_name = blocked_path.file_name()?.to_str()?;
        
        match file_name {
            ".env" => Some("Use '.env.example' to document required variables".to_string()),
            name if name.starts_with(".env.") => {
                Some("Create '.env.example' with placeholder values".to_string())
            },
            _ => None,
        }
    }
}
```

### 3. Path Traversal Prevention

#### 3.1 Path Validation Engine
```rust
/// Comprehensive path validation and sanitization
#[derive(Debug, Clone)]
pub struct PathValidator {
    /// Allowed workspace roots
    allowed_roots: Vec<PathBuf>,
    /// Blocked path patterns (beyond workspace boundaries)
    blocked_patterns: Vec<Regex>,
    /// Maximum path traversal depth
    max_traversal_depth: usize,
}

impl PathValidator {
    pub fn new(workspace_roots: Vec<PathBuf>) -> Self {
        Self {
            allowed_roots: workspace_roots,
            blocked_patterns: vec![
                // System directories
                Regex::new(r"^/(etc|var|usr|bin|sbin|lib|boot|sys|dev|proc)(/.*)?$").unwrap(),
                // User sensitive directories
                Regex::new(r"^/Users/[^/]+/\.(ssh|gnupg|config)(/.*)?$").unwrap(),
                Regex::new(r"^/home/[^/]+/\.(ssh|gnupg|config)(/.*)?$").unwrap(),
                // Windows system directories
                Regex::new(r"^[A-Z]:\\(Windows|System32|Program Files)\\.*$").unwrap(),
            ],
            max_traversal_depth: 10,
        }
    }
    
    /// Validate and canonicalize path
    pub fn validate_path(
        &self, 
        path: &Path, 
        workspace_root: &Path
    ) -> SecurityResult<PathBuf> {
        // 1. Resolve to canonical path
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // If path doesn't exist, validate the intended path
                self.validate_intended_path(path, workspace_root)?
            }
        };
        
        // 2. Check against blocked system paths
        let canonical_str = canonical_path.to_string_lossy();
        for blocked_pattern in &self.blocked_patterns {
            if blocked_pattern.is_match(&canonical_str) {
                return Err(SecurityError::BlockedSystemPath {
                    attempted_path: path.to_path_buf(),
                    canonical_path: canonical_path.clone(),
                    reason: "Access to system directory blocked".to_string(),
                });
            }
        }
        
        // 3. Validate workspace boundary
        self.validate_workspace_boundary(&canonical_path, workspace_root)?;
        
        // 4. Check traversal depth
        self.validate_traversal_depth(path)?;
        
        Ok(canonical_path)
    }
    
    fn validate_workspace_boundary(
        &self,
        canonical_path: &Path,
        workspace_root: &Path,
    ) -> SecurityResult<()> {
        let canonical_workspace = workspace_root.canonicalize()
            .map_err(|e| SecurityError::InvalidWorkspace {
                workspace_path: workspace_root.to_path_buf(),
                error: e.to_string(),
            })?;
        
        if !canonical_path.starts_with(&canonical_workspace) {
            return Err(SecurityError::WorkspaceBoundaryViolation {
                attempted_path: canonical_path.to_path_buf(),
                workspace_root: canonical_workspace,
                suggestion: format!(
                    "Access files within workspace: {}",
                    canonical_workspace.display()
                ),
            });
        }
        
        Ok(())
    }
    
    fn validate_traversal_depth(&self, path: &Path) -> SecurityResult<()> {
        let traversal_count = path.components()
            .filter(|c| matches!(c, std::path::Component::ParentDir))
            .count();
            
        if traversal_count > self.max_traversal_depth {
            return Err(SecurityError::ExcessiveTraversal {
                path: path.to_path_buf(),
                traversal_count,
                max_allowed: self.max_traversal_depth,
            });
        }
        
        Ok(())
    }
    
    fn validate_intended_path(
        &self, 
        path: &Path, 
        workspace_root: &Path
    ) -> SecurityResult<PathBuf> {
        // For non-existent paths, manually resolve relative components
        let mut resolved = workspace_root.to_path_buf();
        
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    if !resolved.pop() {
                        return Err(SecurityError::PathTraversalAttempt {
                            path: path.to_path_buf(),
                            reason: "Attempted to traverse above filesystem root".to_string(),
                        });
                    }
                },
                std::path::Component::Normal(name) => {
                    resolved.push(name);
                },
                std::path::Component::RootDir => {
                    return Err(SecurityError::PathTraversalAttempt {
                        path: path.to_path_buf(),
                        reason: "Absolute paths not allowed in workspace context".to_string(),
                    });
                },
                _ => {} // Current dir, prefix, etc.
            }
        }
        
        Ok(resolved)
    }
}
```

### 4. Security Rule Engine

#### 4.1 Extensible Rule System
```rust
/// Trait for custom security rules
pub trait SecurityRule: Send + Sync + std::fmt::Debug {
    /// Unique identifier for this rule
    fn id(&self) -> &str;
    
    /// Human-readable description of what this rule does
    fn description(&self) -> &str;
    
    /// Validate a security context
    fn validate(&self, context: &SecurityContext) -> SecurityResult<RuleResult>;
    
    /// Priority level (0 = highest priority)
    fn priority(&self) -> u8 { 50 }
    
    /// Whether this rule can be disabled by configuration
    fn can_disable(&self) -> bool { true }
}

/// Result of a security rule evaluation
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub action: RuleAction,
    pub message: Option<String>,
    pub suggestion: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuleAction {
    Allow,
    Block,
    Warn,
    Modify(String), // Modified command to execute instead
}

/// Security context passed to rules
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub tool_call: ToolCall,
    pub session_context: Option<SessionContext>,
    pub workspace_root: PathBuf,
    pub file_paths: Vec<PathBuf>,
    pub command: Option<String>,
    pub environment: HashMap<String, String>,
}

/// High-performance rule engine
pub struct SecurityRuleEngine {
    rules: Vec<Box<dyn SecurityRule>>,
    rule_cache: DashMap<String, RuleResult>,
    config: SecurityConfig,
}

impl SecurityRuleEngine {
    pub fn new(config: SecurityConfig) -> Self {
        let mut rules: Vec<Box<dyn SecurityRule>> = vec![
            Box::new(DangerousCommandRule::new()),
            Box::new(EnvironmentFileRule::new()),
            Box::new(PathTraversalRule::new()),
            Box::new(WorkspaceBoundaryRule::new()),
        ];
        
        // Add custom rules from config
        for custom_rule in config.custom_rules {
            rules.push(Box::new(CustomPatternRule::new(custom_rule)));
        }
        
        // Sort by priority
        rules.sort_by_key(|rule| rule.priority());
        
        Self {
            rules,
            rule_cache: DashMap::new(),
            config,
        }
    }
    
    /// Validate security context against all rules
    pub fn validate(&self, context: &SecurityContext) -> SecurityResult<ValidationResult> {
        let mut warnings = Vec::new();
        let mut modifications = Vec::new();
        
        // Generate cache key for this validation
        let cache_key = self.generate_cache_key(context);
        
        // Check cache first (for performance)
        if let Some(cached_result) = self.rule_cache.get(&cache_key) {
            return Ok(cached_result.clone().into());
        }
        
        // Execute rules in priority order
        for rule in &self.rules {
            // Skip disabled rules
            if self.config.disabled_rules.contains(rule.id()) && rule.can_disable() {
                continue;
            }
            
            let start_time = std::time::Instant::now();
            let result = rule.validate(context)?;
            let execution_time = start_time.elapsed();
            
            // Log slow rules
            if execution_time > std::time::Duration::from_millis(1) {
                tracing::warn!(
                    rule_id = rule.id(),
                    execution_time_ms = execution_time.as_millis(),
                    "Slow security rule execution"
                );
            }
            
            match result.action {
                RuleAction::Block => {
                    let validation_result = ValidationResult {
                        allowed: false,
                        block_reason: result.message.unwrap_or_else(|| {
                            format!("Blocked by security rule: {}", rule.id())
                        }),
                        suggestion: result.suggestion,
                        warnings: Vec::new(),
                        modifications: None,
                    };
                    
                    // Cache the result
                    self.rule_cache.insert(cache_key, validation_result.clone().into());
                    
                    return Ok(validation_result);
                },
                RuleAction::Warn => {
                    if let Some(message) = result.message {
                        warnings.push(SecurityWarning {
                            rule_id: rule.id().to_string(),
                            message,
                            suggestion: result.suggestion,
                        });
                    }
                },
                RuleAction::Modify(new_command) => {
                    modifications.push(SecurityModification {
                        rule_id: rule.id().to_string(),
                        original_command: context.command.clone(),
                        modified_command: new_command,
                        reason: result.message.unwrap_or_default(),
                    });
                },
                RuleAction::Allow => {
                    // Continue to next rule
                }
            }
        }
        
        let validation_result = ValidationResult {
            allowed: true,
            block_reason: String::new(),
            suggestion: None,
            warnings,
            modifications: if modifications.is_empty() { None } else { Some(modifications) },
        };
        
        // Cache the result
        self.rule_cache.insert(cache_key, validation_result.clone().into());
        
        Ok(validation_result)
    }
    
    fn generate_cache_key(&self, context: &SecurityContext) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        context.tool_call.tool_name.hash(&mut hasher);
        if let Some(cmd) = &context.command {
            cmd.hash(&mut hasher);
        }
        context.workspace_root.hash(&mut hasher);
        context.file_paths.hash(&mut hasher);
        
        format!("sec_val_{:x}", hasher.finish())
    }
}
```

#### 4.2 Built-in Security Rules
```rust
/// Dangerous command detection rule
#[derive(Debug)]
pub struct DangerousCommandRule {
    patterns: Vec<(Regex, &'static str, &'static str)>, // pattern, reason, suggestion
}

impl DangerousCommandRule {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                (
                    Regex::new(r"rm\s+-[rf]*r[rf]*\s+(/|\*|~|\$HOME)").unwrap(),
                    "Dangerous rm -rf command detected",
                    "Use specific file paths instead of wildcards or system directories"
                ),
                (
                    Regex::new(r"sudo\s+rm\s+-[rf]*r[rf]*").unwrap(),
                    "Privileged destructive operation blocked",
                    "Avoid using sudo with destructive commands"
                ),
                (
                    Regex::new(r"chmod\s+-R\s+000").unwrap(),
                    "Command would make files unreadable",
                    "Use specific permissions like 644 or 755"
                ),
            ],
        }
    }
}

impl SecurityRule for DangerousCommandRule {
    fn id(&self) -> &str { "dangerous_command" }
    fn description(&self) -> &str { "Blocks commands that could cause system damage" }
    fn priority(&self) -> u8 { 0 } // Highest priority
    fn can_disable(&self) -> bool { false } // Cannot be disabled
    
    fn validate(&self, context: &SecurityContext) -> SecurityResult<RuleResult> {
        if let Some(command) = &context.command {
            for (pattern, reason, suggestion) in &self.patterns {
                if pattern.is_match(command) {
                    return Ok(RuleResult {
                        action: RuleAction::Block,
                        message: Some(reason.to_string()),
                        suggestion: Some(suggestion.to_string()),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        Ok(RuleResult {
            action: RuleAction::Allow,
            message: None,
            suggestion: None,
            metadata: HashMap::new(),
        })
    }
}

/// Environment file protection rule
#[derive(Debug)]
pub struct EnvironmentFileRule {
    protection: EnvFileProtection,
}

impl SecurityRule for EnvironmentFileRule {
    fn id(&self) -> &str { "env_file_protection" }
    fn description(&self) -> &str { "Protects sensitive environment files from access" }
    fn priority(&self) -> u8 { 10 }
    
    fn validate(&self, context: &SecurityContext) -> SecurityResult<RuleResult> {
        for file_path in &context.file_paths {
            if self.protection.is_file_blocked(file_path) {
                return Ok(RuleResult {
                    action: RuleAction::Block,
                    message: Some(self.protection.get_block_reason(file_path)),
                    suggestion: Some(format!(
                        "Use '.env.example' or create a template file instead of '{}'",
                        file_path.display()
                    )),
                    metadata: HashMap::new(),
                });
            }
        }
        
        Ok(RuleResult {
            action: RuleAction::Allow,
            message: None,
            suggestion: None,
            metadata: HashMap::new(),
        })
    }
}
```

### 5. Security Error Handling and Exit Codes

#### 5.1 Comprehensive Security Error Types
```rust
/// Security-specific error types extending MaosError from PRD-01
#[derive(thiserror::Error, Debug)]
pub enum SecurityError {
    #[error("Dangerous command blocked: {command}")]
    DangerousCommand {
        command: String,
        pattern: String,
        reason: String,
        safe_alternative: Option<String>,
    },
    
    #[error("Protected file access denied: {file_path}")]
    ProtectedFileAccess {
        file_path: PathBuf,
        tool_name: String,
        reason: String,
        suggestion: Option<String>,
    },
    
    #[error("Path traversal attempt blocked: {path}")]
    PathTraversalAttempt {
        path: PathBuf,
        reason: String,
    },
    
    #[error("Workspace boundary violation: {attempted_path}")]
    WorkspaceBoundaryViolation {
        attempted_path: PathBuf,
        workspace_root: PathBuf,
        suggestion: String,
    },
    
    #[error("Blocked system path access: {attempted_path}")]
    BlockedSystemPath {
        attempted_path: PathBuf,
        canonical_path: PathBuf,
        reason: String,
    },
    
    #[error("Excessive path traversal: {path} contains {traversal_count} '..' components")]
    ExcessiveTraversal {
        path: PathBuf,
        traversal_count: usize,
        max_allowed: usize,
    },
    
    #[error("Invalid workspace: {workspace_path}")]
    InvalidWorkspace {
        workspace_path: PathBuf,
        error: String,
    },
    
    #[error("Security rule engine error: {message}")]
    RuleEngineError {
        message: String,
        rule_id: Option<String>,
    },
    
    #[error("Security configuration error: {message}")]
    ConfigurationError {
        message: String,
    },
}

impl From<SecurityError> for MaosError {
    fn from(error: SecurityError) -> Self {
        MaosError::Security(error)
    }
}

/// Ensure security errors map to exit code 2 (blocking)
impl From<&SecurityError> for ExitCode {
    fn from(_error: &SecurityError) -> Self {
        ExitCode::BlockingError // Exit code 2
    }
}
```

#### 5.2 User-Friendly Error Messages
```rust
impl SecurityError {
    /// Generate user-friendly error message with suggestions
    pub fn user_message(&self) -> String {
        match self {
            SecurityError::DangerousCommand { command, reason, safe_alternative, .. } => {
                let mut message = format!("🚫 Security Alert: {}\n\nBlocked command: {}", reason, command);
                if let Some(alternative) = safe_alternative {
                    message.push_str(&format!("\n\n💡 Try instead: {}", alternative));
                }
                message.push_str("\n\nThis protection prevents accidental system damage.");
                message
            },
            
            SecurityError::ProtectedFileAccess { file_path, reason, suggestion, .. } => {
                let mut message = format!("🔒 File Access Blocked: {}\n\nFile: {}", reason, file_path.display());
                if let Some(suggestion) = suggestion {
                    message.push_str(&format!("\n\n💡 Suggestion: {}", suggestion));
                }
                message.push_str("\n\nThis protection prevents exposure of sensitive data.");
                message
            },
            
            SecurityError::WorkspaceBoundaryViolation { attempted_path, workspace_root, suggestion } => {
                format!(
                    "🚧 Workspace Boundary Violation\n\n\
                    Attempted to access: {}\n\
                    Allowed workspace: {}\n\n\
                    💡 {}\n\n\
                    This protection prevents agents from interfering with each other.",
                    attempted_path.display(),
                    workspace_root.display(),
                    suggestion
                )
            },
            
            SecurityError::PathTraversalAttempt { path, reason } => {
                format!(
                    "🚨 Path Traversal Blocked\n\n\
                    Blocked path: {}\n\
                    Reason: {}\n\n\
                    This protection prevents unauthorized file system access.",
                    path.display(),
                    reason
                )
            },
            
            _ => format!("🛡️ Security Policy Violation: {}", self),
        }
    }
    
    /// Generate JSON-formatted error for programmatic consumption
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "error_type": "security_violation",
            "error_code": "MAOS_SEC_001",
            "message": self.to_string(),
            "user_message": self.user_message(),
            "suggestions": self.get_suggestions(),
            "metadata": self.get_metadata(),
        })
    }
    
    fn get_suggestions(&self) -> Vec<String> {
        match self {
            SecurityError::DangerousCommand { safe_alternative, .. } => {
                let mut suggestions = vec!["Review the command for accuracy".to_string()];
                if let Some(alt) = safe_alternative {
                    suggestions.push(format!("Use: {}", alt));
                }
                suggestions
            },
            SecurityError::ProtectedFileAccess { suggestion, .. } => {
                let mut suggestions = vec!["Use template files instead of actual config files".to_string()];
                if let Some(s) = suggestion {
                    suggestions.push(s.clone());
                }
                suggestions
            },
            SecurityError::WorkspaceBoundaryViolation { suggestion, .. } => {
                vec![suggestion.clone()]
            },
            _ => vec!["Contact administrator if this block is incorrect".to_string()],
        }
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        match self {
            SecurityError::DangerousCommand { command, pattern, .. } => {
                serde_json::json!({
                    "blocked_command": command,
                    "matched_pattern": pattern,
                    "severity": "critical"
                })
            },
            SecurityError::ProtectedFileAccess { file_path, tool_name, .. } => {
                serde_json::json!({
                    "blocked_file": file_path,
                    "tool_name": tool_name,
                    "severity": "high"
                })
            },
            _ => serde_json::json!({
                "severity": "medium"
            }),
        }
    }
}
```

## Non-Functional Requirements

### Performance Requirements
- **Validation Speed**: Complete all security checks in <5ms (50% of total execution budget)
- **Memory Usage**: <100KB additional memory consumption during validation
- **Startup Overhead**: Security module initialization in <1ms
- **Rule Engine Efficiency**: O(1) average case pattern matching through pre-compiled regex
- **Cache Hit Rate**: >80% cache hit rate for repeated similar operations

### Reliability Requirements
- **Zero False Negatives**: 100% detection rate for all defined dangerous patterns
- **Minimal False Positives**: <1% legitimate operations incorrectly blocked
- **Graceful Degradation**: Continue with warnings if non-critical security rules fail
- **Error Recovery**: Clear error messages that help users understand and fix issues
- **Audit Trail**: Complete logging of all security decisions for forensic analysis

### Security Requirements
- **Pattern Completeness**: Comprehensive coverage of dangerous command patterns
- **Evasion Resistance**: Resilient against common command obfuscation techniques
- **Configuration Security**: Secure handling of custom security rules and configurations
- **Information Disclosure**: No sensitive information leaked in error messages
- **Default Deny**: Fail securely - block operations when validation is uncertain

### Scalability Requirements
- **Rule Engine Scale**: Support for 100+ custom security rules with minimal performance impact
- **Concurrent Validation**: Thread-safe validation for multiple simultaneous operations
- **Memory Scaling**: Linear memory usage scaling with number of active rules
- **Cache Efficiency**: Effective caching to handle repeated pattern validation

## Technical Design

### 1. Security Architecture Overview
```
Security Validation Flow:
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Tool Call     │───▶│  Security        │───▶│  Validation     │
│   Input         │    │  Context         │    │  Result         │
└─────────────────┘    │  Builder         │    └─────────────────┘
                       └──────────────────┘              │
                                │                        │
                                ▼                        ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │  Rule Engine     │    │   Exit Code 2   │
                       │  Execution       │    │   (if blocked)  │
                       └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │  Security Rules  │
                       │  - Commands      │
                       │  - Files         │
                       │  - Paths         │
                       │  - Custom        │
                       └──────────────────┘
```

### 2. Performance Optimization Strategy

#### 2.1 Pre-compiled Pattern Matching
```rust
/// High-performance pattern matching using pre-compiled regex
pub struct CompiledSecurityPatterns {
    dangerous_commands: RegexSet,
    protected_files: RegexSet,
    system_paths: RegexSet,
    pattern_explanations: HashMap<usize, PatternInfo>,
}

impl CompiledSecurityPatterns {
    pub fn new() -> Self {
        let dangerous_patterns = CRITICAL_PATTERNS.iter().map(|p| *p).collect::<Vec<_>>();
        let file_patterns = PROTECTED_FILE_PATTERNS.iter().map(|p| *p).collect::<Vec<_>>();
        let system_patterns = SYSTEM_PATH_PATTERNS.iter().map(|p| *p).collect::<Vec<_>>();
        
        Self {
            dangerous_commands: RegexSet::new(&dangerous_patterns).unwrap(),
            protected_files: RegexSet::new(&file_patterns).unwrap(),
            system_paths: RegexSet::new(&system_patterns).unwrap(),
            pattern_explanations: Self::build_explanations(),
        }
    }
    
    /// Fast pattern matching - O(1) average case
    pub fn check_dangerous_command(&self, command: &str) -> Option<PatternMatch> {
        let matches: Vec<_> = self.dangerous_commands.matches(command).into_iter().collect();
        
        if let Some(&first_match) = matches.first() {
            Some(PatternMatch {
                pattern_id: first_match,
                matched_text: command.to_string(),
                explanation: self.pattern_explanations[&first_match].clone(),
            })
        } else {
            None
        }
    }
}

/// Pattern matching optimized for minimal allocations
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_id: usize,
    pub matched_text: String,
    pub explanation: PatternInfo,
}

#[derive(Debug, Clone)]
pub struct PatternInfo {
    pub pattern: &'static str,
    pub reason: &'static str,
    pub suggestion: &'static str,
    pub severity: Severity,
}
```

#### 2.2 Intelligent Caching Strategy
```rust
/// Multi-level caching for security validation results
pub struct SecurityCache {
    /// L1: Recently validated commands (LRU cache)
    command_cache: LruCache<String, ValidationResult>,
    /// L2: File path validations (persistent across invocations)
    path_cache: DashMap<PathBuf, PathValidationResult>,
    /// L3: Rule execution results (shared across sessions)
    rule_cache: DashMap<String, RuleResult>,
}

impl SecurityCache {
    pub fn new() -> Self {
        Self {
            command_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            path_cache: DashMap::new(),
            rule_cache: DashMap::new(),
        }
    }
    
    /// Get cached validation result
    pub fn get_command_validation(&mut self, command: &str) -> Option<ValidationResult> {
        self.command_cache.get(command).cloned()
    }
    
    /// Cache validation result with TTL
    pub fn cache_command_validation(
        &mut self, 
        command: String, 
        result: ValidationResult
    ) {
        self.command_cache.put(command, result);
    }
    
    /// Batch cache invalidation for performance
    pub fn invalidate_workspace(&self, workspace_root: &Path) {
        self.path_cache.retain(|path, _| !path.starts_with(workspace_root));
    }
}
```

### 3. Rule Engine Implementation

#### 3.1 High-Performance Rule Execution
```rust
/// Optimized rule engine for sub-millisecond execution
pub struct OptimizedRuleEngine {
    /// Rules sorted by priority and execution cost
    priority_rules: Vec<Box<dyn SecurityRule>>,
    /// Fast-path rules (simple pattern matching)
    fast_rules: Vec<Box<dyn SecurityRule>>,
    /// Complex rules requiring full context
    complex_rules: Vec<Box<dyn SecurityRule>>,
    /// Performance metrics
    metrics: SecurityMetrics,
}

impl OptimizedRuleEngine {
    /// Execute rules with performance monitoring
    pub fn validate_optimized(&self, context: &SecurityContext) -> SecurityResult<ValidationResult> {
        let start_time = std::time::Instant::now();
        
        // 1. Fast-path rules first (regex matching only)
        for rule in &self.fast_rules {
            let rule_start = std::time::Instant::now();
            let result = rule.validate(context)?;
            let rule_duration = rule_start.elapsed();
            
            self.metrics.record_rule_execution(rule.id(), rule_duration);
            
            if result.action == RuleAction::Block {
                self.metrics.record_validation_complete(start_time.elapsed(), false);
                return Ok(ValidationResult::blocked(result));
            }
        }
        
        // 2. Priority rules (critical security checks)
        for rule in &self.priority_rules {
            let rule_start = std::time::Instant::now();
            let result = rule.validate(context)?;
            let rule_duration = rule_start.elapsed();
            
            self.metrics.record_rule_execution(rule.id(), rule_duration);
            
            if result.action == RuleAction::Block {
                self.metrics.record_validation_complete(start_time.elapsed(), false);
                return Ok(ValidationResult::blocked(result));
            }
        }
        
        // 3. Complex rules (if we have time budget remaining)
        let elapsed = start_time.elapsed();
        if elapsed < std::time::Duration::from_millis(3) { // Save 2ms for other operations
            for rule in &self.complex_rules {
                let rule_start = std::time::Instant::now();
                let result = rule.validate(context)?;
                let rule_duration = rule_start.elapsed();
                
                self.metrics.record_rule_execution(rule.id(), rule_duration);
                
                if result.action == RuleAction::Block {
                    self.metrics.record_validation_complete(start_time.elapsed(), false);
                    return Ok(ValidationResult::blocked(result));
                }
                
                // Check time budget
                if start_time.elapsed() > std::time::Duration::from_millis(4) {
                    tracing::warn!("Security validation approaching time limit, skipping remaining complex rules");
                    break;
                }
            }
        }
        
        self.metrics.record_validation_complete(start_time.elapsed(), true);
        Ok(ValidationResult::allowed())
    }
}
```

### 4. Integration with MAOS Core

#### 4.1 Security Validator Public API
```rust
/// Main security validator interface for MAOS core
pub struct SecurityValidator {
    rule_engine: OptimizedRuleEngine,
    path_validator: PathValidator,
    env_protection: EnvFileProtection,
    cache: Mutex<SecurityCache>,
    config: SecurityConfig,
    metrics: Arc<SecurityMetrics>,
}

impl SecurityValidator {
    /// Create new security validator with configuration
    pub fn new(config: SecurityConfig) -> SecurityResult<Self> {
        let rule_engine = OptimizedRuleEngine::new(&config)?;
        let path_validator = PathValidator::new(config.allowed_workspaces.clone());
        let env_protection = EnvFileProtection::from_config(&config);
        
        Ok(Self {
            rule_engine,
            path_validator,
            env_protection,
            cache: Mutex::new(SecurityCache::new()),
            config,
            metrics: Arc::new(SecurityMetrics::new()),
        })
    }
    
    /// Main validation entry point
    pub async fn validate_tool_call(&self, tool_call: &ToolCall) -> SecurityResult<ValidationResult> {
        let context = SecurityContext::from_tool_call(tool_call)?;
        
        // Check cache first
        let cache_key = self.generate_cache_key(&context);
        if let Some(cached_result) = self.cache.lock().unwrap().get_command_validation(&cache_key) {
            self.metrics.record_cache_hit();
            return Ok(cached_result);
        }
        
        // Full validation
        let result = self.rule_engine.validate_optimized(&context)?;
        
        // Cache the result
        self.cache.lock().unwrap().cache_command_validation(cache_key, result.clone());
        
        Ok(result)
    }
    
    /// Validate file access specifically
    pub fn validate_file_access(
        &self,
        file_paths: &[PathBuf],
        workspace_root: &Path,
        tool_name: &str,
    ) -> SecurityResult<()> {
        for file_path in file_paths {
            // Check environment file protection
            if self.env_protection.is_file_blocked(file_path) {
                return Err(SecurityError::ProtectedFileAccess {
                    file_path: file_path.clone(),
                    tool_name: tool_name.to_string(),
                    reason: self.env_protection.get_block_reason(file_path),
                    suggestion: Some("Use template files instead".to_string()),
                });
            }
            
            // Validate path traversal
            self.path_validator.validate_path(file_path, workspace_root)?;
        }
        
        Ok(())
    }
    
    /// Get security metrics for monitoring
    pub fn get_metrics(&self) -> SecurityMetricsReport {
        self.metrics.generate_report()
    }
}
```

## Dependencies & Constraints

### Dependencies on Other PRDs

#### PRD-01: Common Foundation (Critical Dependency)
- **Required Types**: `MaosError`, `ExitCode`, `ToolCall`, `SessionContext`
- **Required Utilities**: Path validation utilities, configuration management
- **Required Constants**: Performance targets, timeout values
- **Integration Point**: Security errors extend the common error hierarchy

#### PRD-02: Session Management (Expected Dependency)
- **Required Types**: `SessionId`, `AgentId`, `WorktreeSpec`
- **Required Services**: Session context for security decisions
- **Integration Point**: Security validation considers session boundaries

### External Dependencies

#### Core Dependencies
```toml
[dependencies]
# From PRD-01 common foundation
maos-common = { path = "../maos-common" }

# Performance-critical dependencies
regex = "1.10"          # Pattern matching
dashmap = "5.5"         # Concurrent hashmaps
lru = "0.12"           # LRU cache
once_cell = "1.19"     # Lazy static initialization

# Standard dependencies
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"
```

#### Development Dependencies
```toml
[dev-dependencies]
criterion = "0.5"       # Performance benchmarking
proptest = "1.4"        # Property-based testing
tempfile = "3.8"        # Temporary files for testing
tokio-test = "0.4"      # Async testing utilities
```

### Technical Constraints

- **Performance Budget**: Total security validation must complete in <5ms
- **Memory Limit**: Additional memory usage must stay under 100KB
- **Thread Safety**: All security operations must be thread-safe for concurrent use
- **Deterministic Results**: Same input must always produce same security decision
- **Zero External Dependencies**: No network calls or external service dependencies

### Design Constraints

- **Fail-Safe Design**: When in doubt, block the operation rather than allow it
- **Clear Error Messages**: Users must understand why operations were blocked
- **Extensibility**: Custom rules must be easy to add without core changes
- **Backward Compatibility**: Security policy changes should not break existing workflows

## Success Criteria & Acceptance Tests

### Functional Acceptance Tests

#### 1. Dangerous Command Detection
```rust
#[cfg(test)]
mod dangerous_command_tests {
    use super::*;
    
    #[test]
    fn test_rm_rf_root_blocked() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        let tool_call = ToolCall::bash_command("rm -rf /");
        
        let result = validator.validate_tool_call(&tool_call).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            SecurityError::DangerousCommand { command, .. } => {
                assert_eq!(command, "rm -rf /");
            },
            _ => panic!("Expected DangerousCommand error"),
        }
    }
    
    #[test]
    fn test_safe_rm_allowed() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        let tool_call = ToolCall::bash_command("rm specific_file.txt");
        
        let result = validator.validate_tool_call(&tool_call).await;
        assert!(result.is_ok());
        assert!(result.unwrap().allowed);
    }
    
    #[test]
    fn test_environment_variable_expansion() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        let mut tool_call = ToolCall::bash_command("rm -rf $HOME");
        tool_call.environment.insert("HOME".to_string(), "/home/user".to_string());
        
        let result = validator.validate_tool_call(&tool_call).await;
        assert!(result.is_err());
    }
}
```

#### 2. Environment File Protection
```rust
#[cfg(test)]
mod env_file_tests {
    use super::*;
    
    #[test]
    fn test_env_file_blocked() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        let file_paths = vec![PathBuf::from(".env")];
        
        let result = validator.validate_file_access(
            &file_paths,
            &PathBuf::from("/workspace"),
            "Read"
        );
        
        assert!(result.is_err());
        match result.unwrap_err() {
            SecurityError::ProtectedFileAccess { file_path, .. } => {
                assert_eq!(file_path, PathBuf::from(".env"));
            },
            _ => panic!("Expected ProtectedFileAccess error"),
        }
    }
    
    #[test]
    fn test_env_example_allowed() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        let file_paths = vec![PathBuf::from(".env.example")];
        
        let result = validator.validate_file_access(
            &file_paths,
            &PathBuf::from("/workspace"),
            "Read"
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_stack_env_allowed() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        let file_paths = vec![PathBuf::from("stack.env")];
        
        let result = validator.validate_file_access(
            &file_paths,
            &PathBuf::from("/workspace"),
            "Read"
        );
        
        assert!(result.is_ok());
    }
}
```

#### 3. Path Traversal Prevention
```rust
#[cfg(test)]
mod path_traversal_tests {
    use super::*;
    
    #[test]
    fn test_path_traversal_blocked() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        let file_paths = vec![PathBuf::from("../../../etc/passwd")];
        
        let result = validator.validate_file_access(
            &file_paths,
            &PathBuf::from("/workspace"),
            "Read"
        );
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_workspace_relative_allowed() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        let file_paths = vec![PathBuf::from("src/main.rs")];
        
        let result = validator.validate_file_access(
            &file_paths,
            &PathBuf::from("/workspace"),
            "Read"
        );
        
        assert!(result.is_ok());
    }
}
```

### Performance Acceptance Tests

#### 1. Validation Speed Requirements
```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_validation_speed_requirement() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        let tool_call = ToolCall::bash_command("ls -la");
        
        let start = Instant::now();
        let result = validator.validate_tool_call(&tool_call).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration < std::time::Duration::from_millis(5), 
               "Validation took {:?}, expected <5ms", duration);
    }
    
    #[tokio::test]
    async fn test_concurrent_validation_performance() {
        let validator = Arc::new(SecurityValidator::new(SecurityConfig::default()).unwrap());
        let mut handles = vec![];
        
        let start = Instant::now();
        
        // Simulate 10 concurrent validations
        for i in 0..10 {
            let validator = validator.clone();
            let handle = tokio::spawn(async move {
                let tool_call = ToolCall::bash_command(&format!("echo 'test {}'", i));
                validator.validate_tool_call(&tool_call).await
            });
            handles.push(handle);
        }
        
        // Wait for all validations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
        
        let duration = start.elapsed();
        assert!(duration < std::time::Duration::from_millis(20), 
               "Concurrent validation took {:?}, expected <20ms", duration);
    }
}
```

#### 2. Memory Usage Requirements
```rust
#[cfg(test)]
mod memory_tests {
    use super::*;
    
    #[test]
    fn test_memory_usage_limit() {
        let initial_memory = get_memory_usage();
        
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        
        // Perform multiple validations
        for i in 0..1000 {
            let tool_call = ToolCall::bash_command(&format!("echo 'test {}'", i));
            let _ = validator.validate_tool_call(&tool_call).await;
        }
        
        let final_memory = get_memory_usage();
        let memory_increase = final_memory - initial_memory;
        
        assert!(memory_increase < 100_000, // 100KB limit
               "Memory usage increased by {} bytes, expected <100KB", memory_increase);
    }
    
    fn get_memory_usage() -> usize {
        // Platform-specific memory usage measurement
        #[cfg(target_os = "linux")]
        {
            let status = std::fs::read_to_string("/proc/self/status").unwrap();
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let kb: usize = line.split_whitespace().nth(1).unwrap().parse().unwrap();
                    return kb * 1024; // Convert to bytes
                }
            }
        }
        0 // Fallback for other platforms
    }
}
```

### Security Acceptance Tests

#### 1. Zero False Negatives
```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    const DANGEROUS_COMMANDS: &[&str] = &[
        "rm -rf /",
        "rm -rf /*",
        "rm -rf ~",
        "rm -rf $HOME",
        "sudo rm -rf /tmp",
        "rm -rf .",
        "rm -rf *",
        "chmod -R 000 /",
        "kill -9 -1",
    ];
    
    #[test]
    fn test_zero_false_negatives() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        
        for dangerous_cmd in DANGEROUS_COMMANDS {
            let tool_call = ToolCall::bash_command(dangerous_cmd);
            let result = validator.validate_tool_call(&tool_call).await;
            
            assert!(result.is_err(), 
                   "Command '{}' should be blocked but was allowed", dangerous_cmd);
        }
    }
    
    const LEGITIMATE_COMMANDS: &[&str] = &[
        "ls -la",
        "git status",
        "cargo build",
        "npm install",
        "rm specific_file.txt",
        "chmod 644 file.txt",
        "mkdir new_directory",
    ];
    
    #[test]
    fn test_minimal_false_positives() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        let mut blocked_count = 0;
        
        for legitimate_cmd in LEGITIMATE_COMMANDS {
            let tool_call = ToolCall::bash_command(legitimate_cmd);
            let result = validator.validate_tool_call(&tool_call).await;
            
            if result.is_err() {
                blocked_count += 1;
                eprintln!("Legitimate command '{}' was blocked: {:?}", 
                         legitimate_cmd, result.unwrap_err());
            }
        }
        
        let false_positive_rate = blocked_count as f64 / LEGITIMATE_COMMANDS.len() as f64;
        assert!(false_positive_rate < 0.01, // <1% false positive rate
               "False positive rate is {:.2}%, expected <1%", false_positive_rate * 100.0);
    }
}
```

## Testing Strategy

### 1. Comprehensive Unit Testing

#### Test Categories
- **Pattern Matching Tests**: Verify regex patterns catch all dangerous variants
- **Path Validation Tests**: Comprehensive traversal and boundary testing
- **File Protection Tests**: Verify all protected file patterns work correctly
- **Rule Engine Tests**: Test custom rule execution and performance
- **Error Handling Tests**: Verify proper error messages and exit codes

#### Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_path_validation_security(
        path in ".*{1,100}",
        workspace in "/[a-zA-Z0-9/]{1,50}"
    ) {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        let workspace_path = PathBuf::from(workspace);
        let test_path = PathBuf::from(path);
        
        let result = validator.validate_file_access(
            &[test_path.clone()], 
            &workspace_path, 
            "Read"
        );
        
        // If validation succeeds, path must be within workspace or allowed exception
        if result.is_ok() {
            // Additional invariant checking
            if let Ok(canonical_test) = test_path.canonicalize() {
                if let Ok(canonical_workspace) = workspace_path.canonicalize() {
                    assert!(
                        canonical_test.starts_with(&canonical_workspace) ||
                        is_allowed_exception(&canonical_test),
                        "Path {:?} should not be allowed outside workspace {:?}",
                        canonical_test, canonical_workspace
                    );
                }
            }
        }
    }
}
```

### 2. Integration Testing

#### MAOS Core Integration
```rust
#[tokio::test]
async fn test_maos_core_integration() {
    let security_config = SecurityConfig::default();
    let validator = SecurityValidator::new(security_config).unwrap();
    
    // Simulate real MAOS pre-tool-use hook
    let tool_call = ToolCall {
        tool_name: "Bash".to_string(),
        parameters: serde_json::json!({
            "command": "rm -rf /tmp/test"
        }),
        workspace_root: Some(PathBuf::from("/workspace")),
        session_id: Some("test-session".to_string()),
        agent_id: Some("test-agent".to_string()),
    };
    
    let result = validator.validate_tool_call(&tool_call).await;
    
    // Should block dangerous command
    assert!(result.is_err());
    
    // Should return exit code 2
    let error = result.unwrap_err();
    let exit_code: ExitCode = (&error).into();
    assert_eq!(exit_code as i32, 2);
}
```

### 3. Performance Benchmarking

#### Micro-benchmarks
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_security_validation(c: &mut Criterion) {
    let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
    
    c.bench_function("validate_safe_command", |b| {
        b.iter(|| {
            let tool_call = ToolCall::bash_command(black_box("ls -la"));
            black_box(validator.validate_tool_call(&tool_call))
        })
    });
    
    c.bench_function("validate_dangerous_command", |b| {
        b.iter(|| {
            let tool_call = ToolCall::bash_command(black_box("rm -rf /"));
            black_box(validator.validate_tool_call(&tool_call))
        })
    });
    
    c.bench_function("validate_file_access", |b| {
        b.iter(|| {
            let paths = vec![PathBuf::from(black_box("src/main.rs"))];
            let workspace = PathBuf::from(black_box("/workspace"));
            black_box(validator.validate_file_access(&paths, &workspace, "Read"))
        })
    });
}

criterion_group!(benches, benchmark_security_validation);
criterion_main!(benches);
```

### 4. Security Testing

#### Penetration Testing Scenarios
```rust
#[cfg(test)]
mod penetration_tests {
    use super::*;
    
    /// Test evasion techniques that attackers might use
    #[test]
    fn test_command_obfuscation_resistance() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        
        let obfuscated_commands = vec![
            "r\u{006d} -rf /",           // Unicode normalization
            "rm\t-rf\t/",                // Tab characters
            "rm  -rf  /",                // Multiple spaces
            "RM -RF /",                  // Case variations
            "rm -rf //",                 // Double slashes
            "/bin/rm -rf /",             // Full path
            "$(echo rm) -rf /",          // Command substitution
        ];
        
        for cmd in obfuscated_commands {
            let tool_call = ToolCall::bash_command(cmd);
            let result = validator.validate_tool_call(&tool_call).await;
            
            assert!(result.is_err(), 
                   "Obfuscated command '{}' should be blocked", cmd);
        }
    }
    
    /// Test sophisticated path traversal attempts
    #[test]
    fn test_advanced_path_traversal() {
        let validator = SecurityValidator::new(SecurityConfig::default()).unwrap();
        
        let traversal_attempts = vec![
            "../../../etc/passwd",
            "..\\..\\..\\Windows\\System32",
            "/../../etc/shadow",
            "workspace/../../../etc/hosts",
            "./../../etc/passwd",
            "src/../../.ssh/id_rsa",
        ];
        
        let workspace = PathBuf::from("/safe/workspace");
        
        for path_str in traversal_attempts {
            let path = PathBuf::from(path_str);
            let result = validator.validate_file_access(&[path.clone()], &workspace, "Read");
            
            assert!(result.is_err(), 
                   "Path traversal '{}' should be blocked", path_str);
        }
    }
}
```

## Timeline Estimate

### Week 1: Core Security Engine (Days 1-7)
**Days 1-2**: Security error types and exit code integration with PRD-01
**Days 3-4**: Dangerous command pattern matching with pre-compiled regex
**Days 5-6**: Environment file protection system
**Day 7**: Path traversal prevention and validation

**Deliverables**:
- Complete security error hierarchy
- High-performance pattern matching engine
- Basic rule engine framework
- Initial test suite with >90% coverage

### Week 2: Advanced Validation and Rule Engine (Days 8-14)
**Days 8-9**: Extensible security rule system and custom rule support
**Days 10-11**: Context-aware validation with session and workspace integration
**Days 12-13**: Performance optimization and caching system
**Day 14**: Comprehensive error messages and user feedback

**Deliverables**:
- Full-featured rule engine with custom rule support
- Context-aware security validation
- Performance-optimized validation pipeline
- User-friendly error messaging system

### Week 3: Integration and Performance Tuning (Days 15-21)
**Days 15-16**: MAOS core integration and API finalization
**Days 17-18**: Security configuration system and policy management
**Days 19-20**: Performance benchmarking and optimization
**Day 21**: Integration testing with other MAOS components

**Deliverables**:
- Seamless MAOS core integration
- Configurable security policies
- Performance targets achieved (<5ms validation)
- Complete integration test suite

### Week 4: Testing and Documentation (Days 22-28)
**Days 22-23**: Comprehensive security testing and penetration testing
**Days 24-25**: Property-based testing and edge case coverage
**Days 26-27**: Documentation, examples, and usage guides
**Day 28**: Final optimization and production readiness verification

**Deliverables**:
- Security-hardened validation system
- Complete test coverage including edge cases
- Production-ready documentation
- Performance and security validation reports

## Risk Assessment & Mitigation

### High-Priority Risks

**Risk**: Performance targets not met due to complex regex matching
**Probability**: Medium **Impact**: High
**Mitigation**: 
- Use pre-compiled `RegexSet` for O(1) pattern matching
- Implement tiered validation (fast rules first)
- Set strict time budgets with early termination
- Cache validation results aggressively

**Risk**: False positives blocking legitimate operations
**Probability**: High **Impact**: Medium
**Mitigation**:
- Extensive testing with real-world command corpus
- Whitelist patterns for common safe operations
- Configuration options to adjust sensitivity
- Clear error messages with suggested alternatives

**Risk**: Evasion of security rules through command obfuscation
**Probability**: Medium **Impact**: High
**Mitigation**:
- Comprehensive pattern testing including obfuscation techniques
- Environment variable expansion before validation
- Command normalization and canonicalization
- Multiple detection layers (command + file + path validation)

### Medium-Priority Risks

**Risk**: Memory usage exceeding budget due to caching
**Probability**: Medium **Impact**: Medium
**Mitigation**:
- LRU cache with strict size limits
- Cache invalidation strategies
- Memory usage monitoring and alerts
- Graceful degradation when memory limited

**Risk**: Integration complexity with PRD-01 and PRD-02
**Probability**: Low **Impact**: Medium
**Mitigation**:
- Clear interface definitions early in development
- Stub implementations for independent testing
- Regular integration testing during development
- Version compatibility checking

**Risk**: Custom security rules causing performance degradation
**Probability**: Medium **Impact**: Low
**Mitigation**:
- Rule execution time limits
- Performance profiling for custom rules
- Rule complexity scoring and warnings
- Documentation on writing efficient rules

## Dependencies for Other PRDs

This Security Validation PRD enables and supports:

### Direct Dependents
- **PRD-02: Session Management** - Provides security context for session operations
- **PRD-04: Git Worktree Management** - Validates git operations and workspace boundaries
- **PRD-05: CLI Command Framework** - Integrates security validation into all commands
- **PRD-06: TTS Integration** - May need security validation for TTS-triggered operations

### Security Integration Points
- **Pre-tool-use Hook**: Primary integration point for validating all tool calls
- **File Operations**: Security validation for Read, Write, Edit tools
- **Command Execution**: Bash command validation before execution
- **Path Resolution**: Safe path handling for all file system operations

## Implementation Notes

### 1. Development Priority
This PRD has **P1 Priority** - it must be implemented immediately after PRD-01 (Common Foundation) as it provides critical security capabilities required by all other components.

### 2. Security-First Design Philosophy
- **Default Deny**: When validation is uncertain, block the operation
- **Defense in Depth**: Multiple validation layers (command, file, path)
- **Clear Communication**: Users must understand why operations are blocked
- **Performance Balance**: Security cannot compromise the <10ms execution target

### 3. Configuration Philosophy
- **Secure Defaults**: Out-of-box configuration should be secure for most users
- **Customizable**: Organizations can define additional security policies
- **Auditable**: All security decisions should be logged and reviewable
- **Maintainable**: Security rules should be easy to understand and modify

### 4. Testing Philosophy
- **Zero Tolerance**: No false negatives for critical security violations
- **Minimal Friction**: Keep false positives under 1%
- **Real-World Testing**: Use actual commands from development workflows
- **Adversarial Testing**: Test against evasion and attack techniques

## Summary

The MAOS Security Validation System provides comprehensive, high-performance protection for AI-assisted development workflows. By implementing sophisticated pattern matching, path validation, and extensible rule engines, this system ensures that dangerous operations are blocked while maintaining developer productivity.

**Key Achievements**:
- **100% Detection Rate** for defined dangerous patterns
- **Sub-5ms Validation** for all security checks
- **<1% False Positive Rate** for legitimate operations
- **Extensible Rule System** supporting custom security policies
- **Clear User Feedback** with actionable suggestions

This security foundation enables developers to work confidently with AI agents, knowing that accidental destructive operations, sensitive file exposure, and system compromises are prevented at the tool execution level.

The system integrates seamlessly with MAOS core components and provides the security foundation that all other PRDs depend on for safe operation. 🛡️💯🚀