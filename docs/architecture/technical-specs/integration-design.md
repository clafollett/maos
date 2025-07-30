# MAOS Integration Design

## Overview

This document details how MAOS integrates with Claude Code, Git worktrees, and external systems to provide seamless multi-agent orchestration.

## Claude Code Integration

### Environment Detection and Setup

```rust
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ClaudeEnvironment {
    pub session_id: Option<String>,
    pub agent_type: Option<String>,
    pub workspace_root: PathBuf,
    pub config_dir: PathBuf,
    pub hooks_enabled: bool,
}

impl ClaudeEnvironment {
    pub fn detect() -> Self {
        Self {
            session_id: env::var("CLAUDE_SESSION_ID").ok(),
            agent_type: env::var("CLAUDE_AGENT_TYPE").ok(),
            workspace_root: env::current_dir().unwrap_or_default(),
            config_dir: dirs::home_dir()
                .unwrap_or_default()
                .join(".claude"),
            hooks_enabled: env::var("CLAUDE_HOOKS_ENABLED")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }
    
    pub fn is_claude_session(&self) -> bool {
        self.session_id.is_some()
    }
    
    pub fn setup_maos_integration(&self) -> Result<()> {
        // Set MAOS environment variables
        env::set_var("MAOS_WORKSPACE", &self.workspace_root);
        
        if let Some(ref session) = self.session_id {
            env::set_var("MAOS_SESSION", session);
        }
        
        if let Some(ref agent) = self.agent_type {
            env::set_var("MAOS_AGENT", agent);
        }
        
        // Initialize MAOS directories
        let maos_dir = self.workspace_root.join(".maos");
        fs::create_dir_all(&maos_dir.join("channels"))?;
        fs::create_dir_all(&maos_dir.join("locks"))?;
        fs::create_dir_all(&maos_dir.join("shared"))?;
        
        Ok(())
    }
}
```

### Hook Registration

```rust
pub struct HookManager {
    claude_config: PathBuf,
    hooks_dir: PathBuf,
}

impl HookManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Home directory not found"))?
            .join(".claude");
        
        Ok(Self {
            claude_config: config_dir.join("config.json"),
            hooks_dir: config_dir.join("hooks"),
        })
    }
    
    pub fn register_maos_hooks(&self) -> Result<()> {
        // Read existing Claude config
        let mut config = self.read_claude_config()?;
        
        // Add MAOS hooks
        let maos_hook = ClaudeHook {
            name: "maos-security".to_string(),
            path: self.get_hook_binary_path()?,
            events: vec![
                "PreToolUse".to_string(),
                "PostToolUse".to_string(),
                "UserPromptSubmit".to_string(),
            ],
            enabled: true,
        };
        
        config.hooks.push(maos_hook);
        
        // Write updated config
        self.write_claude_config(&config)?;
        
        // Create symlink for hook binary
        let hook_binary = self.get_hook_binary_path()?;
        let hook_link = self.hooks_dir.join("maos-security");
        
        if !hook_link.exists() {
            std::os::unix::fs::symlink(&hook_binary, &hook_link)?;
        }
        
        Ok(())
    }
    
    fn get_hook_binary_path(&self) -> Result<PathBuf> {
        // Check multiple locations
        let locations = vec![
            PathBuf::from("/usr/local/bin/maos-hooks"),
            dirs::home_dir().unwrap().join(".cargo/bin/maos-hooks"),
            env::current_exe()?.parent().unwrap().join("maos-hooks"),
        ];
        
        for path in locations {
            if path.exists() {
                return Ok(path);
            }
        }
        
        Err(anyhow!("MAOS hooks binary not found"))
    }
}
```

### Agent Communication Protocol

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: Uuid,
    pub timestamp: SystemTime,
    pub from_agent: String,
    pub to_agent: String,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Notification,
    Progress,
    Error,
}

pub struct AgentCommunicator {
    workspace: PathBuf,
    agent_id: String,
}

impl AgentCommunicator {
    pub async fn send_to_agent(
        &self,
        target: &str,
        message_type: MessageType,
        payload: impl Serialize,
    ) -> Result<Uuid> {
        let message = AgentMessage {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            from_agent: self.agent_id.clone(),
            to_agent: target.to_string(),
            message_type,
            payload: serde_json::to_value(payload)?,
        };
        
        // Write to target agent's inbox
        let inbox_path = self.workspace
            .join(".maos")
            .join("agents")
            .join(target)
            .join("inbox");
        
        fs::create_dir_all(&inbox_path).await?;
        
        let message_file = inbox_path.join(format!("{}.json", message.id));
        let content = serde_json::to_string_pretty(&message)?;
        fs::write(&message_file, content).await?;
        
        // Send notification through Claude's Task tool
        self.notify_via_task_tool(target, &message.id).await?;
        
        Ok(message.id)
    }
    
    async fn notify_via_task_tool(&self, target: &str, message_id: &Uuid) -> Result<()> {
        // This would integrate with Claude's Task tool
        // For now, we use file-based notification
        let notification_path = self.workspace
            .join(".maos")
            .join("notifications")
            .join(format!("{}.notify", target));
        
        fs::write(&notification_path, message_id.to_string()).await?;
        Ok(())
    }
}
```

## Git Worktree Automation

### Worktree Management

```rust
use git2::{Repository, BranchType, Signature};

pub struct WorktreeManager {
    repo: Repository,
    base_path: PathBuf,
    config: WorktreeConfig,
}

#[derive(Debug, Clone)]
pub struct WorktreeConfig {
    pub branch_prefix: String,
    pub worktree_prefix: String,
    pub auto_fetch: bool,
    pub cleanup_merged: bool,
}

impl WorktreeManager {
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = Repository::open(repo_path)?;
        let base_path = repo_path.parent()
            .ok_or_else(|| anyhow!("Invalid repository path"))?
            .to_path_buf();
        
        Ok(Self {
            repo,
            base_path,
            config: WorktreeConfig::default(),
        })
    }
    
    pub async fn create_for_issue(&self, issue_id: &str) -> Result<Worktree> {
        // Fetch latest from origin
        if self.config.auto_fetch {
            self.fetch_origin().await?;
        }
        
        // Generate branch name
        let branch_name = self.generate_branch_name(issue_id).await?;
        
        // Create branch from main
        let main_branch = self.repo.find_branch("main", BranchType::Local)?;
        let target_commit = main_branch.get().target()
            .ok_or_else(|| anyhow!("Main branch has no target"))?;
        
        let commit = self.repo.find_commit(target_commit)?;
        self.repo.branch(&branch_name, &commit, false)?;
        
        // Create worktree
        let worktree_path = self.base_path.join(format!(
            "{}-{}",
            self.config.worktree_prefix,
            issue_id
        ));
        
        self.create_worktree(&branch_name, &worktree_path)?;
        
        // Initialize MAOS structure in worktree
        self.initialize_maos_structure(&worktree_path).await?;
        
        Ok(Worktree {
            path: worktree_path,
            branch: branch_name,
            issue_id: issue_id.to_string(),
            created_at: SystemTime::now(),
        })
    }
    
    async fn generate_branch_name(&self, issue_id: &str) -> Result<String> {
        // Fetch issue details from GitHub
        let github = GithubClient::from_env()?;
        let issue = github.get_issue(issue_id).await?;
        
        // Generate branch name
        let slug = slugify(&issue.title);
        Ok(format!(
            "{}/issue-{}/{}",
            self.config.branch_prefix,
            issue_id,
            slug
        ))
    }
    
    fn create_worktree(&self, branch: &str, path: &Path) -> Result<()> {
        use std::process::Command;
        
        let output = Command::new("git")
            .current_dir(self.repo.path())
            .args(&["worktree", "add", path.to_str().unwrap(), branch])
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create worktree: {}", stderr));
        }
        
        Ok(())
    }
    
    pub async fn switch_worktree(&self, issue_id: &str) -> Result<PathBuf> {
        let worktrees = self.list_worktrees().await?;
        
        let worktree = worktrees.iter()
            .find(|w| w.issue_id == issue_id)
            .ok_or_else(|| anyhow!("Worktree not found for issue {}", issue_id))?;
        
        // Update Claude's working directory
        env::set_current_dir(&worktree.path)?;
        
        // Notify MAOS of context switch
        self.notify_context_switch(issue_id).await?;
        
        Ok(worktree.path.clone())
    }
}
```

### Automated Worktree Lifecycle

```rust
pub struct WorktreeLifecycle {
    manager: WorktreeManager,
    cleanup_config: CleanupConfig,
}

impl WorktreeLifecycle {
    pub async fn auto_cleanup(&self) -> Result<CleanupReport> {
        let mut report = CleanupReport::default();
        let worktrees = self.manager.list_worktrees().await?;
        
        for worktree in worktrees {
            // Check if branch is merged
            if self.is_branch_merged(&worktree.branch).await? {
                if self.cleanup_config.remove_merged {
                    self.remove_worktree(&worktree).await?;
                    report.removed_merged += 1;
                }
            }
            
            // Check if worktree is stale
            if let Some(stale_days) = self.cleanup_config.stale_after_days {
                let age = SystemTime::now()
                    .duration_since(worktree.created_at)?;
                
                if age > Duration::from_secs(stale_days * 24 * 60 * 60) {
                    self.archive_worktree(&worktree).await?;
                    report.archived_stale += 1;
                }
            }
        }
        
        Ok(report)
    }
    
    async fn is_branch_merged(&self, branch: &str) -> Result<bool> {
        use std::process::Command;
        
        let output = Command::new("git")
            .args(&["branch", "--merged", "main"])
            .output()?;
        
        let merged_branches = String::from_utf8_lossy(&output.stdout);
        Ok(merged_branches.contains(branch))
    }
}
```

## GitHub Integration

### Issue-Driven Development

```rust
use octocrab::{Octocrab, models};

pub struct GithubIntegration {
    client: Octocrab,
    owner: String,
    repo: String,
}

impl GithubIntegration {
    pub fn from_env() -> Result<Self> {
        let token = env::var("GITHUB_TOKEN")?;
        let client = Octocrab::builder()
            .personal_token(token)
            .build()?;
        
        let owner = env::var("GITHUB_OWNER")
            .unwrap_or_else(|_| "clafollett".to_string());
        let repo = env::var("GITHUB_REPO")
            .unwrap_or_else(|_| "maos".to_string());
        
        Ok(Self { client, owner, repo })
    }
    
    pub async fn get_issue(&self, issue_number: u64) -> Result<Issue> {
        let issue = self.client
            .issues(&self.owner, &self.repo)
            .get(issue_number)
            .await?;
        
        Ok(Issue {
            number: issue.number,
            title: issue.title,
            body: issue.body.unwrap_or_default(),
            labels: issue.labels.into_iter()
                .map(|l| l.name)
                .collect(),
            assignees: issue.assignees.into_iter()
                .map(|a| a.login)
                .collect(),
        })
    }
    
    pub async fn create_pr_for_worktree(
        &self,
        worktree: &Worktree,
    ) -> Result<PullRequest> {
        let issue = self.get_issue(worktree.issue_id.parse()?).await?;
        
        let pr = self.client
            .pulls(&self.owner, &self.repo)
            .create(
                &issue.title,
                &worktree.branch,
                "main",
            )
            .body(&format!(
                "Closes #{}\n\n{}",
                worktree.issue_id,
                issue.body
            ))
            .draft(true)
            .send()
            .await?;
        
        Ok(PullRequest {
            number: pr.number,
            url: pr.html_url.unwrap_or_default(),
            branch: worktree.branch.clone(),
        })
    }
    
    pub async fn update_issue_status(
        &self,
        issue_number: u64,
        status: &str,
    ) -> Result<()> {
        // Add a comment with status update
        self.client
            .issues(&self.owner, &self.repo)
            .create_comment(issue_number, status)
            .await?;
        
        // Update labels if needed
        match status {
            "in-progress" => {
                self.add_label(issue_number, "status: in progress").await?;
            }
            "ready-for-review" => {
                self.remove_label(issue_number, "status: in progress").await?;
                self.add_label(issue_number, "status: review").await?;
            }
            _ => {}
        }
        
        Ok(())
    }
}
```

## Session Management

### MAOS Session Lifecycle

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub issue_id: String,
    pub worktree: Worktree,
    pub agents: Vec<AgentInstance>,
    pub started_at: SystemTime,
    pub status: SessionStatus,
}

pub struct SessionManager {
    workspace: PathBuf,
    github: GithubIntegration,
    worktree_manager: WorktreeManager,
}

impl SessionManager {
    pub async fn start_session(
        &self,
        issue_id: &str,
        agents: Vec<&str>,
    ) -> Result<Session> {
        // Create or switch to worktree
        let worktree = if let Ok(existing) = self.find_worktree(issue_id).await {
            existing
        } else {
            self.worktree_manager.create_for_issue(issue_id).await?
        };
        
        // Switch to worktree directory
        env::set_current_dir(&worktree.path)?;
        
        // Initialize session
        let session = Session {
            id: SessionId::new(),
            issue_id: issue_id.to_string(),
            worktree: worktree.clone(),
            agents: Vec::new(),
            started_at: SystemTime::now(),
            status: SessionStatus::Active,
        };
        
        // Save session state
        self.save_session(&session).await?;
        
        // Update GitHub issue
        self.github.update_issue_status(
            issue_id.parse()?,
            "MAOS session started 🚀",
        ).await?;
        
        // Spawn requested agents
        for agent_type in agents {
            self.spawn_agent(&session.id, agent_type).await?;
        }
        
        Ok(session)
    }
    
    pub async fn end_session(
        &self,
        session_id: &SessionId,
        merge: bool,
    ) -> Result<SessionSummary> {
        let mut session = self.load_session(session_id).await?;
        
        // Collect session metrics
        let summary = self.generate_summary(&session).await?;
        
        // If merge requested, create PR
        if merge {
            let pr = self.github
                .create_pr_for_worktree(&session.worktree)
                .await?;
            
            self.github.update_issue_status(
                session.issue_id.parse()?,
                &format!("PR created: {}", pr.url),
            ).await?;
        }
        
        // Update session status
        session.status = SessionStatus::Completed;
        self.save_session(&session).await?;
        
        // Cleanup temporary files
        self.cleanup_session_artifacts(&session).await?;
        
        Ok(summary)
    }
    
    async fn spawn_agent(
        &self,
        session_id: &SessionId,
        agent_type: &str,
    ) -> Result<AgentInstance> {
        // This would integrate with Claude's sub-agent spawning
        // For now, we track the agent metadata
        
        let agent = AgentInstance {
            id: AgentId::new(),
            agent_type: agent_type.to_string(),
            session_id: session_id.clone(),
            spawned_at: SystemTime::now(),
            status: AgentStatus::Active,
        };
        
        // Create agent workspace
        let agent_dir = self.workspace
            .join(".maos")
            .join("agents")
            .join(&agent.id.to_string());
        
        fs::create_dir_all(&agent_dir).await?;
        
        // Initialize agent communication channels
        fs::create_dir_all(&agent_dir.join("inbox")).await?;
        fs::create_dir_all(&agent_dir.join("outbox")).await?;
        
        Ok(agent)
    }
}
```

## Configuration Management

### Unified Configuration

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MaosConfig {
    pub core: CoreConfig,
    pub security: SecurityConfig,
    pub worktree: WorktreeConfig,
    pub github: GithubConfig,
    pub hooks: HooksConfig,
}

impl MaosConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }
    
    pub fn merge_with_env(&mut self) {
        // Override with environment variables
        if let Ok(token) = env::var("GITHUB_TOKEN") {
            self.github.token = Some(token);
        }
        
        if let Ok(policy) = env::var("MAOS_SECURITY_POLICY") {
            self.security.default_policy = policy;
        }
        
        if let Ok(prefix) = env::var("MAOS_BRANCH_PREFIX") {
            self.worktree.branch_prefix = prefix;
        }
    }
    
    fn config_path() -> Result<PathBuf> {
        Ok(dirs::config_dir()
            .ok_or_else(|| anyhow!("Config directory not found"))?
            .join("maos")
            .join("config.toml"))
    }
}
```

### Environment Variable Integration

```bash
# MAOS-specific variables
export MAOS_WORKSPACE="/path/to/workspace"
export MAOS_SECURITY_POLICY="standard"
export MAOS_BRANCH_PREFIX="feature"
export MAOS_AUTO_CLEANUP="true"
export MAOS_LOG_LEVEL="info"

# GitHub integration
export GITHUB_TOKEN="ghp_..."
export GITHUB_OWNER="clafollett"
export GITHUB_REPO="maos"

# Claude Code compatibility
export CLAUDE_SESSION_ID="..."
export CLAUDE_AGENT_TYPE="orchestrator"
export CLAUDE_HOOKS_ENABLED="true"
```

## Error Handling and Recovery

### Graceful Degradation

```rust
pub struct IntegrationManager {
    claude_env: Option<ClaudeEnvironment>,
    github: Option<GithubIntegration>,
    config: MaosConfig,
}

impl IntegrationManager {
    pub fn new() -> Self {
        let config = MaosConfig::load().unwrap_or_default();
        
        let claude_env = match ClaudeEnvironment::detect() {
            env if env.is_claude_session() => Some(env),
            _ => {
                eprintln!("Warning: Not running in Claude Code environment");
                None
            }
        };
        
        let github = match GithubIntegration::from_env() {
            Ok(gh) => Some(gh),
            Err(e) => {
                eprintln!("Warning: GitHub integration unavailable: {}", e);
                None
            }
        };
        
        Self {
            claude_env,
            github,
            config,
        }
    }
    
    pub async fn execute_with_fallback<T, F, G>(
        &self,
        primary: F,
        fallback: G,
    ) -> Result<T>
    where
        F: Future<Output = Result<T>>,
        G: Future<Output = Result<T>>,
    {
        match primary.await {
            Ok(result) => Ok(result),
            Err(e) => {
                eprintln!("Primary operation failed: {}, trying fallback", e);
                fallback.await
            }
        }
    }
}
```

This integration design ensures MAOS works seamlessly with Claude Code while providing robust automation for multi-agent development workflows.