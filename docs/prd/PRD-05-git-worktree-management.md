# PRD-05: Git Worktree Management

## Executive Summary

The Git Worktree Management system provides isolated, high-performance workspace creation for parallel AI agent development in MAOS. This system automatically creates and manages git worktrees on-demand, ensuring complete isolation between agents while maintaining Git history integrity and enabling seamless parallel development workflows.

**Key Deliverable**: A robust worktree management system that creates isolated git worktrees lazily, manages branch naming strategies, executes git commands safely, and provides automatic cleanup—all while maintaining <10ms execution times and bulletproof data safety.

## Problem Statement

Without proper git worktree management, MAOS agents would face critical development challenges:

- **Agent Interference**: Multiple agents modifying the same files simultaneously, causing conflicts and data corruption
- **Git State Chaos**: Parallel operations creating inconsistent git history and branch management nightmare
- **Resource Waste**: Pre-creating worktrees when they may never be used, consuming unnecessary disk space
- **Cleanup Complexity**: Orphaned worktrees and branches accumulating over time, polluting the repository
- **Performance Degradation**: Slow git operations blocking the <10ms MAOS execution target
- **Data Loss Risk**: Improper worktree removal potentially destroying uncommitted work

We need a intelligent worktree management system that creates isolation when needed, maintains performance, and ensures data safety throughout the agent lifecycle.

## Goals & Success Metrics

### Primary Goals

1. **Lazy Isolation**: Create worktrees only when agents actually need them
2. **Branch Safety**: Implement safe branch naming and management strategies
3. **Performance Excellence**: All git operations complete within MAOS performance targets
4. **Data Protection**: Zero risk of data loss during worktree operations
5. **Automatic Cleanup**: Clean up unused worktrees without user intervention

### Success Metrics

- **Performance**: All worktree operations complete in <50ms (5x buffer under MAOS <10ms target)
- **Isolation**: 100% agent workspace isolation with zero cross-contamination
- **Reliability**: Zero data loss incidents during worktree lifecycle management
- **Efficiency**: Lazy creation reduces unnecessary disk usage by >90%
- **Cleanup**: Automatic cleanup removes 100% of stale worktrees within 24 hours
- **Git Compatibility**: Works correctly across all supported Git versions (2.5+)

## User Personas & Use Cases

### Primary User: MAOS Agent System
**Profile**: Automated agent requesting isolated workspace for development tasks
**Use Case**: Seamless workspace creation for parallel development without interference
**Success Criteria**: Gets isolated worktree instantly when needed, never aware of cleanup

### Secondary User: MAOS Session Manager
**Profile**: Session coordination system managing multiple concurrent agents
**Use Case**: Coordinate worktree allocation and prevent resource conflicts
**Success Criteria**: Perfect resource allocation with no agent workspace collisions

### Tertiary User: Developer Using MAOS
**Profile**: Engineer using MAOS-enhanced Claude Code for parallel development
**Use Case**: Transparent multi-agent development with clean git history
**Success Criteria**: Git history remains clean, no manual worktree management required

## Functional Requirements

### 1. Lazy Worktree Creation

#### 1.1 On-Demand Creation Strategy
```rust
/// Lazy worktree creation manager
pub struct WorktreeManager {
    repo_root: PathBuf,
    worktree_root: PathBuf,
    git_command: GitCommandExecutor,
    active_worktrees: HashMap<AgentId, WorktreeInfo>,
}

impl WorktreeManager {
    /// Create worktree only when agent first requests file access
    pub async fn ensure_worktree(&mut self, agent_id: &AgentId) -> MaosResult<PathBuf> {
        if let Some(worktree) = self.active_worktrees.get(agent_id) {
            return Ok(worktree.path.clone());
        }
        
        let worktree_info = self.create_worktree(agent_id).await?;
        self.active_worktrees.insert(agent_id.clone(), worktree_info.clone());
        Ok(worktree_info.path)
    }
    
    /// Check if worktree creation is needed based on agent activity
    pub fn requires_worktree(&self, agent_id: &AgentId, tool_call: &ToolCall) -> bool {
        // Only create worktree if agent is accessing files in workspace
        match tool_call.tool_name.as_str() {
            "Read" | "Write" | "Edit" | "MultiEdit" => true,
            "Grep" | "Glob" if tool_call.has_path_param() => true,
            _ => false,
        }
    }
}
```

#### 1.2 Creation Triggers
- **File Access**: Agent attempts Read/Write/Edit operations
- **Search Operations**: Grep/Glob with path parameters  
- **Explicit Request**: Agent requests workspace via session API
- **Dependency Chain**: Agent spawns child agents needing isolation

#### 1.3 Creation Validation
```rust
/// Validate worktree creation is safe and necessary
pub struct CreationValidator {
    max_worktrees: usize,
    min_disk_space_mb: u64,
    blacklisted_branches: HashSet<String>,
}

impl CreationValidator {
    pub fn validate_creation(&self, request: &WorktreeRequest) -> ValidationResult<()> {
        // Check resource limits
        self.check_disk_space()?;
        self.check_worktree_count()?;
        
        // Validate branch naming
        self.validate_branch_name(&request.branch_name)?;
        
        // Check git repository state
        self.validate_git_state()?;
        
        Ok(())
    }
}
```

### 2. Branch Naming Strategy

#### 2.1 Deterministic Branch Names
```rust
/// Generate safe, unique branch names for agent worktrees
pub fn generate_branch_name(
    session_id: &SessionId,
    agent_id: &AgentId,
    agent_type: &AgentType,
) -> String {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let session_short = &session_id.to_string()[..8];
    let agent_short = &agent_id.to_string()[..8];
    
    format!(
        "maos/{}/{}/{}_{}_{}",
        agent_type.to_string().to_lowercase(),
        session_short,
        agent_short,
        timestamp,
        thread_rng().gen::<u32>() % 1000
    )
    // Example: maos/code-review/a1b2c3d4/e5f6g7h8_20240804-143025_742
}
```

#### 2.2 Branch Lifecycle Management
```rust
/// Branch state tracking and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub base_commit: String,
    pub agent_id: AgentId,
    pub session_id: SessionId,
    pub status: BranchStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchStatus {
    Active,
    Merged,
    Abandoned,
    PendingCleanup,
}
```

#### 2.3 Branch Conflict Resolution
```rust
/// Handle branch name conflicts gracefully
impl WorktreeManager {
    pub fn resolve_branch_conflict(&self, preferred_name: &str) -> MaosResult<String> {
        let mut attempt = 0;
        let base_name = preferred_name;
        
        loop {
            let candidate = if attempt == 0 {
                base_name.to_string()
            } else {
                format!("{}_retry_{}", base_name, attempt)
            };
            
            if !self.branch_exists(&candidate)? {
                return Ok(candidate);
            }
            
            attempt += 1;
            if attempt > 10 {
                return Err(MaosError::Git(GitError::BranchConflict(
                    "Unable to resolve branch name conflict".to_string()
                )));
            }
        }
    }
}
```

### 3. Safe Git Command Execution

#### 3.1 Git Command Wrapper
```rust
/// Safe git command execution with comprehensive error handling
pub struct GitCommandExecutor {
    repo_path: PathBuf,
    timeout_ms: u64,
    max_retries: u32,
}

impl GitCommandExecutor {
    /// Execute git command with safety checks and timeout
    pub async fn execute(&self, args: &[&str]) -> GitResult<CommandOutput> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo_path)
           .args(args)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // Apply timeout to prevent hanging
        let result = timeout(Duration::from_millis(self.timeout_ms), cmd.output()).await
            .map_err(|_| GitError::Timeout(self.timeout_ms))?;
        
        match result {
            Ok(output) => self.process_command_output(output, args),
            Err(e) => Err(GitError::ExecutionFailed(e.to_string())),
        }
    }
    
    /// Validate git command safety before execution
    fn validate_command_safety(&self, args: &[&str]) -> GitResult<()> {
        // Block dangerous operations by matching exact argument patterns
        let dangerous_patterns: &[&[&str]] = &[
            &["reset", "--hard"],
            &["clean", "-fd"],
            &["push", "--force"],
        ];
        
        for pattern in dangerous_patterns {
            if args.len() >= pattern.len() && args[..pattern.len()] == *pattern {
                return Err(GitError::DangerousOperation(args.join(" ")));
            }
        }
        
        Ok(())
    }
}
```

#### 3.2 Atomic Operations
```rust
/// Atomic worktree operations with rollback capability
pub struct AtomicWorktreeOperation {
    steps: Vec<GitOperation>,
    completed_steps: Vec<GitOperation>,
}

impl AtomicWorktreeOperation {
    pub async fn execute(mut self) -> GitResult<()> {
        for step in &self.steps {
            match self.execute_step(step).await {
                Ok(()) => self.completed_steps.push(step.clone()),
                Err(e) => {
                    self.rollback().await?;
                    return Err(e);
                }
            }
        }
        Ok(())
    }
    
    async fn rollback(&self) -> GitResult<()> {
        // Execute rollback operations in reverse order
        for step in self.completed_steps.iter().rev() {
            if let Err(e) = step.rollback().await {
                error!("Rollback failed for step {:?}: {}", step, e);
            }
        }
        Ok(())
    }
}
```

#### 3.3 Git Repository Validation
```rust
/// Comprehensive git repository validation
pub struct GitRepositoryValidator;

impl GitRepositoryValidator {
    pub fn validate_repository(path: &Path) -> GitResult<RepositoryInfo> {
        // Check if it's a git repository
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return Err(GitError::NotARepository(path.to_path_buf()));
        }
        
        // Validate git version compatibility
        Self::check_git_version()?;
        
        // Check repository health
        Self::check_repository_health(path)?;
        
        // Validate worktree support
        Self::check_worktree_support(path)?;
        
        Ok(RepositoryInfo {
            root_path: path.to_path_buf(),
            git_version: Self::get_git_version()?,
            worktree_capable: true,
            current_branch: Self::get_current_branch(path)?,
        })
    }
    
    fn check_git_version() -> GitResult<()> {
        let version = Self::get_git_version()?;
        let required = Version::parse("2.5.0").unwrap();
        
        if version < required {
            return Err(GitError::IncompatibleVersion(version, required));
        }
        
        Ok(())
    }
}
```

### 4. Worktree Lifecycle Management

#### 4.1 Worktree Creation Process
```rust
/// Complete worktree creation workflow
impl WorktreeManager {
    pub async fn create_worktree(&mut self, agent_id: &AgentId) -> MaosResult<WorktreeInfo> {
        let session = self.get_session_for_agent(agent_id)?;
        
        // Generate unique branch name
        let branch_name = generate_branch_name(
            &session.id,
            agent_id,
            &self.get_agent_type(agent_id)?,
        );
        
        // Validate creation is safe
        self.validator.validate_creation(&WorktreeRequest {
            agent_id: agent_id.clone(),
            branch_name: branch_name.clone(),
            base_branch: "main".to_string(),
        })?;
        
        // Create worktree path
        let worktree_path = self.generate_worktree_path(agent_id)?;
        
        // Execute atomic worktree creation
        let operation = AtomicWorktreeOperation::new(vec![
            GitOperation::CreateBranch(branch_name.clone()),
            GitOperation::AddWorktree {
                path: worktree_path.clone(),
                branch: branch_name.clone(),
            },
            GitOperation::InitializeWorktree(worktree_path.clone()),
        ]);
        
        operation.execute().await
            .map_err(|e| MaosError::Git(e))?;
        
        // Track worktree state
        let worktree_info = WorktreeInfo {
            path: worktree_path,
            branch_name,
            agent_id: agent_id.clone(),
            created_at: Utc::now(),
            status: WorktreeStatus::Active,
        };
        
        self.persist_worktree_info(&worktree_info)?;
        
        Ok(worktree_info)
    }
}
```

#### 4.2 Worktree Status Tracking
```rust
/// Comprehensive worktree status and health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch_name: String,
    pub agent_id: AgentId,
    pub session_id: SessionId,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub status: WorktreeStatus,
    pub uncommitted_changes: bool,
    pub disk_usage_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorktreeStatus {
    Active,
    Idle,
    PendingCleanup,
    CleanupInProgress,
    Removed,
}

impl WorktreeInfo {
    /// Check if worktree is eligible for cleanup
    pub fn is_cleanup_eligible(&self, max_idle_hours: u64) -> bool {
        if self.uncommitted_changes {
            return false;
        }
        
        let idle_duration = Utc::now().signed_duration_since(self.last_activity);
        idle_duration.num_hours() as u64 > max_idle_hours
    }
    
    /// Update activity timestamp
    pub fn touch_activity(&mut self) {
        self.last_activity = Utc::now();
    }
}
```

### 5. Automatic Cleanup Operations

#### 5.1 Cleanup Strategy Engine
```rust
/// Intelligent cleanup strategy based on usage patterns and resource constraints
pub struct CleanupStrategy {
    max_idle_hours: u64,
    max_worktrees: usize,
    min_free_disk_gb: u64,
    preserve_uncommitted: bool,
}

impl CleanupStrategy {
    pub fn evaluate_cleanup_candidates(
        &self,
        worktrees: &[WorktreeInfo],
        system_info: &SystemInfo,
    ) -> Vec<CleanupCandidate> {
        let mut candidates = Vec::new();
        
        for worktree in worktrees {
            let priority = self.calculate_cleanup_priority(worktree, system_info);
            
            if priority > 0 {
                candidates.push(CleanupCandidate {
                    worktree_info: worktree.clone(),
                    priority,
                    reason: self.determine_cleanup_reason(worktree, system_info),
                });
            }
        }
        
        // Sort by priority (highest first)
        candidates.sort_by(|a, b| b.priority.cmp(&a.priority));
        candidates
    }
    
    fn calculate_cleanup_priority(&self, worktree: &WorktreeInfo, system: &SystemInfo) -> u32 {
        let mut priority = 0;
        
        // Age-based priority
        let hours_idle = Utc::now()
            .signed_duration_since(worktree.last_activity)
            .num_hours() as u64;
        
        if hours_idle > self.max_idle_hours {
            priority += (hours_idle - self.max_idle_hours) as u32;
        }
        
        // Resource pressure priority
        if system.free_disk_gb < self.min_free_disk_gb {
            priority += 100;
        }
        
        if system.worktree_count > self.max_worktrees {
            priority += 50;
        }
        
        // Never cleanup with uncommitted changes
        if worktree.uncommitted_changes && self.preserve_uncommitted {
            priority = 0;
        }
        
        priority
    }
}
```

#### 5.2 Safe Cleanup Operations
```rust
/// Safe worktree cleanup with comprehensive data protection
pub struct WorktreeCleanup {
    strategy: CleanupStrategy,
    git_executor: GitCommandExecutor,
    backup_manager: BackupManager,
}

impl WorktreeCleanup {
    pub async fn cleanup_worktree(&self, worktree: &WorktreeInfo) -> MaosResult<CleanupResult> {
        // Pre-cleanup validation
        self.validate_cleanup_safety(worktree).await?;
        
        // Create backup if requested
        if self.should_backup(worktree) {
            self.backup_manager.create_backup(worktree).await?;
        }
        
        // Execute cleanup atomically
        let operation = AtomicWorktreeOperation::new(vec![
            GitOperation::CheckUncommittedChanges(worktree.path.clone()),
            GitOperation::RemoveWorktree(worktree.path.clone()),
            GitOperation::DeleteBranch(worktree.branch_name.clone()),
            GitOperation::CleanupGitRefs(worktree.branch_name.clone()),
        ]);
        
        operation.execute().await
            .map_err(|e| MaosError::Git(e))?;
        
        Ok(CleanupResult {
            worktree_path: worktree.path.clone(),
            branch_name: worktree.branch_name.clone(),
            disk_freed_bytes: worktree.disk_usage_bytes,
            cleanup_time: Utc::now(),
        })
    }
    
    async fn validate_cleanup_safety(&self, worktree: &WorktreeInfo) -> MaosResult<()> {
        // Check for uncommitted changes
        let status = self.git_executor.execute(&[
            "-C", worktree.path.to_str().unwrap(),
            "status", "--porcelain"
        ]).await?;
        
        if !status.stdout.is_empty() && !self.strategy.preserve_uncommitted {
            return Err(MaosError::Git(GitError::UncommittedChanges(
                worktree.path.clone()
            )));
        }
        
        // Verify agent is no longer active
        if self.is_agent_active(&worktree.agent_id)? {
            return Err(MaosError::InvalidInput {
                message: format!("Cannot cleanup worktree for active agent: {}", worktree.agent_id)
            });
        }
        
        Ok(())
    }
}
```

#### 5.3 Scheduled Cleanup
```rust
/// Background cleanup scheduling and execution
pub struct CleanupScheduler {
    cleanup_interval_hours: u64,
    last_cleanup: Option<DateTime<Utc>>,
    cleanup_engine: WorktreeCleanup,
}

impl CleanupScheduler {
    pub async fn run_scheduled_cleanup(&mut self) -> MaosResult<CleanupReport> {
        if !self.should_run_cleanup() {
            return Ok(CleanupReport::skipped());
        }
        
        let all_worktrees = self.list_all_worktrees().await?;
        let system_info = self.get_system_info().await?;
        
        let candidates = self.cleanup_engine.strategy.evaluate_cleanup_candidates(
            &all_worktrees,
            &system_info,
        );
        
        let mut results = Vec::new();
        for candidate in candidates {
            match self.cleanup_engine.cleanup_worktree(&candidate.worktree_info).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Failed to cleanup worktree {:?}: {}", candidate.worktree_info.path, e);
                }
            }
        }
        
        self.last_cleanup = Some(Utc::now());
        
        Ok(CleanupReport {
            cleaned_worktrees: results.len(),
            total_disk_freed: results.iter().map(|r| r.disk_freed_bytes).sum(),
            cleanup_time: Utc::now(),
        })
    }
}
```

### 6. Error Recovery Mechanisms

#### 6.1 Corrupted Worktree Recovery
```rust
/// Recovery operations for corrupted or inconsistent worktrees
pub struct WorktreeRecovery {
    git_executor: GitCommandExecutor,
    validator: GitRepositoryValidator,
}

impl WorktreeRecovery {
    pub async fn recover_worktree(&self, worktree_path: &Path) -> MaosResult<RecoveryResult> {
        let recovery_plan = self.assess_corruption(worktree_path).await?;
        
        match recovery_plan.corruption_level {
            CorruptionLevel::Minor => self.repair_minor_issues(worktree_path).await,
            CorruptionLevel::Major => self.rebuild_worktree(worktree_path).await,
            CorruptionLevel::Critical => self.remove_and_recreate(worktree_path).await,
        }
    }
    
    async fn assess_corruption(&self, path: &Path) -> MaosResult<RecoveryPlan> {
        let mut issues = Vec::new();
        
        // Check git integrity
        if let Err(e) = self.git_executor.execute(&[
            "-C", path.to_str().unwrap(),
            "fsck", "--no-progress"
        ]).await {
            issues.push(CorruptionIssue::GitIntegrity(e.to_string()));
        }
        
        // Check worktree registration
        if !self.is_worktree_registered(path).await? {
            issues.push(CorruptionIssue::UnregisteredWorktree);
        }
        
        // Check file system consistency
        if !path.exists() {
            issues.push(CorruptionIssue::MissingDirectory);
        }
        
        Ok(RecoveryPlan::from_issues(issues))
    }
}
```

#### 6.2 Git State Synchronization
```rust
/// Synchronize git state after recovery or inconsistency detection
pub struct GitStateSynchronizer {
    main_repo: PathBuf,
    git_executor: GitCommandExecutor,
}

impl GitStateSynchronizer {
    pub async fn synchronize_worktree_state(&self, worktree_path: &Path) -> MaosResult<()> {
        // Fetch latest changes from main repository
        self.git_executor.execute(&[
            "-C", worktree_path.to_str().unwrap(),
            "fetch", "origin"
        ]).await?;
        
        // Update references
        self.git_executor.execute(&[
            "-C", worktree_path.to_str().unwrap(),
            "remote", "update"
        ]).await?;
        
        // Verify consistency
        self.verify_worktree_consistency(worktree_path).await?;
        
        Ok(())
    }
    
    async fn verify_worktree_consistency(&self, path: &Path) -> MaosResult<()> {
        // Check that worktree points to correct commit
        let main_head = self.get_main_head().await?;
        let worktree_base = self.get_worktree_base(path).await?;
        
        // Verify branch relationships are correct
        self.validate_branch_relationships(path).await?;
        
        Ok(())
    }
}
```

## Non-Functional Requirements

### Performance Requirements
- **Worktree Creation**: Complete in <50ms including all git operations
- **Branch Generation**: Unique branch names generated in <1ms  
- **Git Commands**: Individual git operations timeout after 30 seconds
- **Cleanup Operations**: Process 100 worktrees in <5 seconds
- **Status Queries**: Worktree status checks complete in <10ms

### Reliability Requirements
- **Data Safety**: Zero risk of data loss during any worktree operation
- **Atomic Operations**: All multi-step operations are fully atomic with rollback
- **Corruption Recovery**: Automatic detection and recovery of corrupted worktrees
- **Consistency**: Git repository remains in consistent state after any failure
- **Retry Logic**: Transient failures automatically retried with exponential backoff

### Scalability Requirements
- **Concurrent Worktrees**: Support up to 50 simultaneous worktrees per repository
- **Repository Size**: Handle repositories up to 10GB with acceptable performance
- **Branch Management**: Efficiently manage thousands of MAOS branches
- **Cleanup Scale**: Clean up hundreds of worktrees without blocking operations

### Security Requirements
- **Path Validation**: All worktree paths validated against traversal attacks
- **Git Command Safety**: Block dangerous git operations that could corrupt repository
- **Branch Isolation**: Complete isolation between agent branches
- **Access Control**: Agents can only access their assigned worktrees

### Compatibility Requirements
- **Git Versions**: Support Git 2.5+ through latest versions
- **Platform Support**: Linux, macOS, Windows with consistent behavior
- **Repository Types**: Support standard git repositories and bare repositories
- **Network Operations**: Handle both local and remote git repositories

## Technical Design

### 1. Component Architecture

```
maos-core::worktree
├── src/
│   ├── lib.rs              # Public API exports
│   ├── manager.rs          # WorktreeManager main logic
│   ├── creator.rs          # Worktree creation operations
│   ├── cleanup.rs          # Cleanup and maintenance
│   ├── branch.rs           # Branch naming and management
│   ├── git/                # Git command execution
│   │   ├── mod.rs
│   │   ├── executor.rs     # GitCommandExecutor
│   │   ├── operations.rs   # Atomic git operations
│   │   └── validator.rs    # Repository validation
│   ├── recovery.rs         # Error recovery mechanisms
│   ├── scheduler.rs        # Background cleanup scheduling
│   └── types.rs           # Worktree-specific types
└── tests/
    ├── integration/        # Full workflow tests
    ├── unit/              # Component unit tests
    └── performance/       # Performance benchmarks
```

### 2. Core Interfaces

#### 2.1 Main Worktree Manager API
```rust
/// Primary interface for all worktree operations
pub trait WorktreeManager {
    /// Ensure agent has an isolated worktree (lazy creation)
    async fn ensure_worktree(&mut self, agent_id: &AgentId) -> MaosResult<PathBuf>;
    
    /// Get existing worktree for agent (returns None if not created)
    fn get_worktree(&self, agent_id: &AgentId) -> Option<&WorktreeInfo>;
    
    /// List all active worktrees
    fn list_worktrees(&self) -> Vec<WorktreeInfo>;
    
    /// Clean up worktree for completed agent
    async fn remove_worktree(&mut self, agent_id: &AgentId) -> MaosResult<CleanupResult>;
    
    /// Run maintenance and cleanup operations
    async fn run_maintenance(&mut self) -> MaosResult<MaintenanceReport>;
}
```

#### 2.2 Git Command Interface
```rust
/// Safe git command execution with comprehensive error handling
pub trait GitCommandExecutor {
    /// Execute git command with safety validation and timeout
    async fn execute(&self, args: &[&str]) -> GitResult<CommandOutput>;
    
    /// Check if git repository is healthy and ready
    async fn validate_repository(&self) -> GitResult<RepositoryInfo>;
    
    /// Get current git version for compatibility checking
    fn get_git_version(&self) -> GitResult<Version>;
}
```

#### 2.3 Cleanup Strategy Interface
```rust
/// Pluggable cleanup strategy for different usage patterns
pub trait CleanupStrategy {
    /// Evaluate which worktrees should be cleaned up
    fn evaluate_cleanup_candidates(
        &self,
        worktrees: &[WorktreeInfo],
        system_info: &SystemInfo,
    ) -> Vec<CleanupCandidate>;
    
    /// Determine if immediate cleanup is required
    fn requires_immediate_cleanup(&self, system_info: &SystemInfo) -> bool;
}
```

### 3. Data Models

#### 3.1 Configuration Types
```rust
/// Worktree management configuration (extends PRD-01 WorktreeConfig)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeConfig {
    /// Root directory for all worktrees
    pub worktree_root: PathBuf,
    
    /// Maximum number of concurrent worktrees
    pub max_worktrees: usize,
    
    /// Hours of inactivity before cleanup eligibility
    pub cleanup_idle_hours: u64,
    
    /// Minimum free disk space to maintain (GB)
    pub min_free_disk_gb: u64,
    
    /// Whether to preserve worktrees with uncommitted changes
    pub preserve_uncommitted: bool,
    
    /// Git command timeout in milliseconds
    pub git_timeout_ms: u64,
    
    /// Background cleanup interval in hours
    pub cleanup_interval_hours: u64,
    
    /// Enable automatic cleanup scheduling
    pub enable_auto_cleanup: bool,
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            worktree_root: PathBuf::from("worktrees"),
            max_worktrees: 20,
            cleanup_idle_hours: 24,
            min_free_disk_gb: 5,
            preserve_uncommitted: true,
            git_timeout_ms: 30000,
            cleanup_interval_hours: 6,
            enable_auto_cleanup: true,
        }
    }
}
```

#### 3.2 Error Types
```rust
/// Comprehensive git and worktree error types
#[derive(thiserror::Error, Debug)]
pub enum GitError {
    #[error("Git command execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Git operation timed out after {0}ms")]
    Timeout(u64),
    
    #[error("Repository not found at path: {0}")]
    NotARepository(PathBuf),
    
    #[error("Git version {0} is incompatible (requires {1}+)")]
    IncompatibleVersion(Version, Version),
    
    #[error("Branch name conflict: {0}")]
    BranchConflict(String),
    
    #[error("Worktree has uncommitted changes: {0}")]
    UncommittedChanges(PathBuf),
    
    #[error("Dangerous git operation blocked: {0}")]
    DangerousOperation(String),
    
    #[error("Worktree corruption detected: {0}")]
    CorruptedWorktree(String),
    
    #[error("Repository is in invalid state: {0}")]
    InvalidRepositoryState(String),
}

impl From<GitError> for MaosError {
    fn from(err: GitError) -> Self {
        MaosError::Git(err)
    }
}
```

### 4. Performance Optimizations

#### 4.1 Command Batching
```rust
/// Batch multiple git operations for better performance
pub struct GitCommandBatch {
    commands: Vec<GitCommand>,
    executor: GitCommandExecutor,
}

impl GitCommandBatch {
    pub fn add_command(&mut self, args: Vec<String>) -> &mut Self {
        self.commands.push(GitCommand::new(args));
        self
    }
    
    /// Execute all commands in optimal order
    pub async fn execute_all(self) -> GitResult<Vec<CommandOutput>> {
        // Optimize command order for performance
        let optimized_commands = self.optimize_command_order();
        
        let mut results = Vec::new();
        for command in optimized_commands {
            let result = self.executor.execute(&command.args).await?;
            results.push(result);
        }
        
        Ok(results)
    }
}
```

#### 4.2 Caching Layer
```rust
/// Cache frequently accessed git information
pub struct GitInfoCache {
    repository_info: RwLock<HashMap<PathBuf, CachedRepositoryInfo>>,
    branch_cache: RwLock<HashMap<String, CachedBranchInfo>>,
    cache_ttl: Duration,
}

impl GitInfoCache {
    pub async fn get_repository_info(&self, path: &Path) -> Option<RepositoryInfo> {
        let cache = self.repository_info.read().await;
        cache.get(path)
            .filter(|info| !info.is_expired())
            .map(|info| info.data.clone())
    }
    
    pub async fn cache_repository_info(&self, path: PathBuf, info: RepositoryInfo) {
        let mut cache = self.repository_info.write().await;
        cache.insert(path, CachedRepositoryInfo {
            data: info,
            cached_at: Utc::now(),
            ttl: self.cache_ttl,
        });
    }
}
```

## Dependencies & Constraints

### External Dependencies
- **git**: Git command-line tool version 2.5+ (essential)
- **tokio**: Async runtime for git command execution (essential)
- **serde**: Serialization for worktree state persistence (from PRD-01)
- **chrono**: Date/time handling for cleanup scheduling (from PRD-01)
- **thiserror**: Error type derivation (from PRD-01)
- **semver**: Git version parsing and comparison (essential)

### Internal Dependencies
- **PRD-01 Common Foundation**: Core types (SessionId, AgentId, MaosError, etc.)
- **PRD-02 Security Validation**: Path validation and command safety checks
- **PRD-04 Session Management**: Agent lifecycle and session state tracking

### Technical Constraints
- **Git Version**: Requires Git 2.5+ for full worktree support
- **File System**: Sufficient disk space for parallel worktrees
- **Performance Budget**: All operations must complete within MAOS <10ms target
- **Cross-Platform**: Consistent behavior across Linux, macOS, Windows

### Platform Constraints
- **Windows**: Handle path length limitations and permission differences
- **macOS**: Respect case sensitivity settings in APFS/HFS+
- **Linux**: Handle different filesystem types and permissions
- **Network**: Support both local and remote git repositories

## Success Criteria & Acceptance Tests

### Functional Success Criteria

1. **Lazy Creation Excellence**
   - Worktrees created only when agents need file access
   - Zero overhead for agents that don't require isolation
   - Creation triggers work correctly for all tool types

2. **Branch Management Perfection**
   - Unique branch names generated with zero conflicts
   - Branch cleanup removes all traces correctly
   - Branch naming follows predictable patterns for debugging

3. **Git Command Safety**
   - All git operations complete safely without repository corruption
   - Dangerous commands blocked before execution
   - Atomic operations with complete rollback on failure

4. **Cleanup Automation**
   - Scheduled cleanup removes stale worktrees automatically
   - Uncommitted changes preserved when configured
   - Resource pressure triggers immediate cleanup

### Performance Success Criteria

1. **Sub-50ms Operations**
   - Worktree creation: <50ms end-to-end
   - Status queries: <10ms response time
   - Branch generation: <1ms for unique names
   - Cleanup operations: <5s for 100 worktrees

2. **Resource Efficiency**
   - Lazy creation reduces disk usage by >90%
   - Git commands execute within timeout limits
   - Memory usage stays under 10MB per operation

3. **Scalability Targets**
   - Support 50 concurrent worktrees without degradation
   - Handle repositories up to 10GB
   - Process thousands of cleanup candidates efficiently

### Security Success Criteria

1. **Complete Isolation**: Zero cross-contamination between agent worktrees
2. **Path Safety**: All path operations validated against traversal attacks
3. **Command Validation**: Dangerous git operations blocked 100% of the time
4. **Data Protection**: Zero data loss incidents during any operation

### Integration Success Criteria

1. **Session Integration**: Perfect coordination with session management
2. **Agent Lifecycle**: Worktrees align with agent creation/destruction
3. **Error Propagation**: Git errors mapped correctly to MAOS error types
4. **Configuration**: All settings respected from unified config system

## Testing Strategy

### 1. Unit Testing Approach
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio_test;
    
    #[tokio::test]
    async fn test_lazy_worktree_creation() {
        let temp_repo = create_test_repository().await;
        let mut manager = WorktreeManager::new(temp_repo.path(), WorktreeConfig::default());
        
        let agent_id = AgentId::generate();
        
        // Initially no worktree should exist
        assert!(manager.get_worktree(&agent_id).is_none());
        
        // Requesting worktree should create it
        let worktree_path = manager.ensure_worktree(&agent_id).await.unwrap();
        assert!(worktree_path.exists());
        assert!(manager.get_worktree(&agent_id).is_some());
        
        // Second request should return same worktree
        let same_path = manager.ensure_worktree(&agent_id).await.unwrap();
        assert_eq!(worktree_path, same_path);
    }
    
    #[tokio::test]
    async fn test_branch_name_uniqueness() {
        let session_id = SessionId::generate();
        let agent_id1 = AgentId::generate();
        let agent_id2 = AgentId::generate();
        
        let branch1 = generate_branch_name(&session_id, &agent_id1, "code-reviewer");
        let branch2 = generate_branch_name(&session_id, &agent_id2, "code-reviewer");
        
        assert_ne!(branch1, branch2);
        assert!(branch1.contains("code-review"));
        assert!(branch2.contains("code-review"));
    }
    
    #[tokio::test]
    async fn test_cleanup_preserves_uncommitted_changes() {
        let temp_repo = create_test_repository_with_changes().await;
        let cleanup = WorktreeCleanup::new(CleanupStrategy::default());
        
        let worktree = create_test_worktree_with_changes(&temp_repo).await;
        
        // Cleanup should fail when uncommitted changes exist
        let result = cleanup.cleanup_worktree(&worktree).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MaosError::Git(GitError::UncommittedChanges(_))));
    }
}
```

### 2. Integration Testing
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_full_agent_lifecycle() {
        let test_session = create_test_session().await;
        let mut manager = WorktreeManager::new(test_session.repo_path(), WorktreeConfig::default());
        
        // Simulate agent spawning
        let agent_id = spawn_test_agent(&test_session).await;
        
        // Agent requests file access - should trigger worktree creation
        let worktree_path = manager.ensure_worktree(&agent_id).await.unwrap();
        
        // Simulate agent work
        perform_agent_file_operations(&worktree_path).await;
        
        // Agent completes - cleanup should occur
        let cleanup_result = manager.remove_worktree(&agent_id).await.unwrap();
        assert!(cleanup_result.disk_freed_bytes > 0);
        
        // Worktree should be completely removed
        assert!(!worktree_path.exists());
    }
    
    #[tokio::test]
    async fn test_parallel_agent_isolation() {
        let test_session = create_test_session().await;
        let mut manager = WorktreeManager::new(test_session.repo_path(), WorktreeConfig::default());
        
        // Create multiple agents simultaneously
        let agent_ids: Vec<_> = (0..5).map(|_| AgentId::generate()).collect();
        
        let worktree_futures: Vec<_> = agent_ids.iter()
            .map(|id| manager.ensure_worktree(id))
            .collect();
        
        let worktree_paths = futures::future::try_join_all(worktree_futures).await.unwrap();
        
        // All worktrees should be unique and isolated
        let unique_paths: HashSet<_> = worktree_paths.iter().collect();
        assert_eq!(unique_paths.len(), agent_ids.len());
        
        // Verify each agent can work independently
        for (agent_id, path) in agent_ids.iter().zip(worktree_paths.iter()) {
            simulate_independent_agent_work(agent_id, path).await;
        }
        
        // Verify no cross-contamination occurred
        verify_agent_isolation(&agent_ids, &worktree_paths).await;
    }
}
```

### 3. Performance Testing
```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_worktree_creation(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let temp_repo = rt.block_on(create_test_repository());
        let mut manager = WorktreeManager::new(temp_repo.path(), WorktreeConfig::default());
        
        c.bench_function("worktree_creation", |b| {
            b.to_async(&rt).iter(|| async {
                let agent_id = black_box(AgentId::generate());
                let path = manager.ensure_worktree(&agent_id).await.unwrap();
                manager.remove_worktree(&agent_id).await.unwrap();
                black_box(path)
            })
        });
    }
    
    fn benchmark_cleanup_performance(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (cleanup, worktrees) = rt.block_on(create_cleanup_benchmark_scenario(100));
        
        c.bench_function("cleanup_100_worktrees", |b| {
            b.to_async(&rt).iter(|| async {
                for worktree in &worktrees {
                    let _ = cleanup.cleanup_worktree(black_box(worktree)).await;
                }
            })
        });
    }
    
    criterion_group!(benches, benchmark_worktree_creation, benchmark_cleanup_performance);
    criterion_main!(benches);
}
```

### 4. Security Testing
```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dangerous_command_blocking() {
        let executor = GitCommandExecutor::new(PathBuf::from("/tmp"));
        
        let dangerous_commands = vec![
            &["reset", "--hard", "HEAD~10"],
            &["clean", "-fd"],
            &["push", "--force", "origin", "main"],
        ];
        
        for cmd in dangerous_commands {
            let result = executor.execute(cmd).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), GitError::DangerousOperation(_)));
        }
    }
    
    #[tokio::test]
    async fn test_path_traversal_prevention() {
        let manager = WorktreeManager::new(PathBuf::from("/safe/repo"), WorktreeConfig::default());
        
        let malicious_paths = vec![
            "../../etc/passwd",
            "../../../sensitive/file",
            "/absolute/path/escape",
        ];
        
        for malicious_path in malicious_paths {
            let result = manager.validate_worktree_path(Path::new(malicious_path));
            assert!(result.is_err());
        }
    }
}
```

## Timeline Estimate

### Week 1: Core Worktree Operations
**Days 1-2**: Git command executor and safety validation
**Days 3-4**: Worktree creation and branch management
**Days 5-7**: Basic cleanup operations and atomic transactions

**Deliverables**:
- Safe git command execution with timeout and validation
- Worktree creation with unique branch naming
- Basic cleanup operations with data protection

### Week 2: Advanced Management Features  
**Days 1-3**: Lazy creation strategy and triggers
**Days 4-5**: Advanced cleanup with intelligent scheduling
**Days 6-7**: Error recovery and corruption handling

**Deliverables**:
- Lazy worktree creation based on agent activity
- Intelligent cleanup with resource pressure handling
- Comprehensive error recovery mechanisms

### Week 3: Performance and Integration
**Days 1-2**: Performance optimization and caching
**Days 3-4**: Integration with session management and security
**Days 5-7**: Cross-platform testing and compatibility

**Deliverables**:
- Performance optimizations meeting <50ms targets
- Full integration with existing MAOS components
- Cross-platform compatibility validation

### Week 4: Testing and Production Readiness
**Days 1-2**: Comprehensive test suite development
**Days 3-4**: Security testing and attack scenario validation
**Days 5-7**: Documentation and production deployment preparation

**Deliverables**:
- Complete test suite with >95% coverage
- Security validation against attack vectors
- Production-ready worktree management system

## Risk Assessment & Mitigation

### Technical Risks

**Risk**: Git command execution failures causing repository corruption
**Probability**: Medium **Impact**: Critical
**Mitigation**: Atomic operations with rollback, comprehensive pre-flight validation, test coverage for all failure scenarios

**Risk**: Performance targets not met due to git command overhead
**Probability**: Medium **Impact**: High
**Mitigation**: Command batching, caching layer, parallel execution where safe, performance benchmarking throughout development

**Risk**: Cross-platform git behavior differences
**Probability**: High **Impact**: Medium
**Mitigation**: Extensive cross-platform testing, platform-specific handling, git version compatibility matrix

### Operational Risks

**Risk**: Cleanup operations removing important uncommitted work
**Probability**: Low **Impact**: Critical
**Mitigation**: Multiple safety checks, user configuration options, backup creation before cleanup, extensive testing

**Risk**: Worktree proliferation consuming excessive disk space
**Probability**: Medium **Impact**: Medium
**Mitigation**: Resource monitoring, aggressive cleanup policies, disk space warnings, configurable limits

### Integration Risks

**Risk**: Session management integration conflicts
**Probability**: Low **Impact**: High
**Mitigation**: Early integration testing, clear interface contracts, coordinated development with session PRD

**Risk**: Security validation bypass
**Probability**: Low **Impact**: High
**Mitigation**: Integration with PRD-02 security system, multiple validation layers, security-focused code review

## Dependencies for Other PRDs

This Git Worktree Management PRD enables and is required by:

### Direct Dependencies
- **PRD-02 Security Validation System** (validates worktree operations and git commands)
- **PRD-04 Session Management** (coordinates worktree lifecycle with agent sessions)
- **PRD-06 CLI Command Framework** (provides worktree management commands)

### Indirect Dependencies
- **PRD-07 Performance Monitoring** (monitors worktree operation performance)
- **PRD-08 Integration Testing** (validates worktree isolation in multi-agent scenarios)

## Implementation Notes

### 1. Development Priority
This PRD has **P1 Priority** as it provides the foundation for agent isolation. It must be completed before multi-agent orchestration can be safely implemented.

### 2. Git Version Compatibility
The implementation must gracefully handle different git versions, falling back to alternative approaches for older versions while maintaining full functionality on modern git.

### 3. Resource Management
All worktree operations must be resource-aware, preventing system overload through configurable limits and monitoring.

### 4. Integration Points
Clean integration with session management ensures worktree lifecycle aligns perfectly with agent lifecycle, preventing resource leaks.

## Summary

The Git Worktree Management system provides the crucial isolation foundation that enables safe, parallel AI agent development in MAOS. Through lazy creation, intelligent cleanup, and bulletproof data safety, this system ensures agents can work independently while maintaining repository integrity and system performance.

**Expected Outcome**: A rock-solid worktree management system that provides transparent isolation for agents, maintains repository health, and enables confident parallel development workflows. Agents get their own isolated workspace instantly when needed, and the system automatically maintains optimal resource usage through intelligent cleanup—all while guaranteeing zero data loss and maintaining the <10ms MAOS performance standard. 💯🚀