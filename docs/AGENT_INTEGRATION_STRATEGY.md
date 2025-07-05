# Agent Integration Strategy

## Overview

MAOS orchestrates AI agents through their command-line interfaces (CLIs) rather than direct API calls. This approach provides several advantages:

- **Standardized Interface**: All agents expose consistent CLI patterns
- **Feature Parity**: Access to full CLI capabilities, not just basic API
- **Tool Integration**: Leverage built-in tools and workflows
- **Session Management**: Utilize CLI session persistence features
- **Authentication**: Leverage existing CLI authentication flows

## Current AI CLI Landscape (2024-2025)

### Primary Integration: Claude Code CLI

**Provider**: Anthropic  
**Repository**: https://github.com/anthropics/claude-code  
**Command**: `claude`  
**Release**: February-May 2025 (Limited Research Preview)  
**Models**: Claude 3.7 Sonnet (hybrid reasoning), Claude Opus 4, Claude Sonnet 4, Claude Haiku 3.5

**Key Features**:
- Agentic coding tool with substantial engineering task delegation
- Deep codebase understanding and context management
- File operations and code editing capabilities
- Session management and context persistence
- MCP server integration for extended capabilities
- Natural language command interface

**Integration Points**:
```bash
# Task execution with file context
claude "Analyze this code and suggest improvements" --file src/main.rs

# Session-based workflows
claude --continue session-123 "Continue the refactoring"

# MCP mode for server integration
claude mcp serve --port 8080

# Comprehensive task delegation
claude "Fix the authentication bug and add tests"
```

### Google Gemini CLI

**Provider**: Google  
**Repository**: https://github.com/google-gemini/gemini-cli  
**Command**: `gemini`  
**Release**: June 2025  
**License**: Apache 2.0 (Open Source)  
**Models**: Gemini 2.5 Pro

**Key Features**:
- ReAct (Reason and Act) loop with local and remote MCP servers
- 1M+ token context window for large codebase analysis
- Multimodal capabilities (images, PDFs, sketches)
- Integration with Google services (Search, Veo 3, Deep Research)
- Built-in media generation capabilities
- Free tier: 60 requests/min, 1,000 requests/day

**Integration Points**:
```bash
# Code analysis with large context
gemini "Analyze this entire codebase structure"

# Multimodal feature generation
gemini "Generate an app from this sketch" --image wireframe.png

# Research integration
gemini "Research the latest best practices for this architecture" --search

# Media generation
gemini "Create a demo video for this feature" --veo
```

### OpenAI Codex CLI

**Provider**: OpenAI  
**Repository**: https://github.com/openai/codex  
**Command**: `codex`  
**Release**: April 2025  
**Models**: o3, o4-mini  
**Installation**: `npm install -g @openai/codex`

**Key Features**:
- Lightweight coding agent for terminal workflows
- Zero-setup installation and configuration
- Multimodal support (screenshots, diagrams)
- Local file manipulation and version control integration
- Chat-driven development with repository understanding
- $1M in API grants available for eligible projects

**Integration Points**:
```bash
# Quick installation
npm install -g @openai/codex

# Feature implementation from visual input
codex "Implement this UI from screenshot" --image mockup.png

# Repository-wide refactoring
codex "Refactor the authentication system to use JWT"

# Code review and optimization
codex "Review this PR and suggest improvements"
```

### GitHub Copilot CLI

**Provider**: Microsoft/GitHub  
**Command**: `gh copilot`  
**Release**: General Availability March 2024  
**Requirements**: GitHub Copilot subscription

**Key Features**:
- Integrated with GitHub CLI ecosystem
- Command explanation and suggestion
- Shell command assistance
- Windows Terminal native integration
- Available to Individual, Business, and Enterprise users

**Integration Points**:
```bash
# Install and setup
gh extension install github/gh-copilot

# Command explanation
gh copilot explain "docker run -p 8080:80 nginx"

# Command suggestion
gh copilot suggest "list all running containers"

# Git workflow assistance
gh copilot suggest "create a feature branch for user auth"
```

### Warp 2.0 Agentic Development Environment

**Provider**: Warp  
**Repository**: https://github.com/warpdotdev/Warp  
**Command**: `warp` (terminal application)  
**Release**: June 2025  
**Platform**: Unified ADE (Agentic Development Environment)

**Key Features**:
- Multi-agent management and threading
- #1 on Terminal-Bench (52%), top-5 on SWE-bench Verified (71%)
- 95% code acceptance rate
- Parallel agent execution (saves 6-7 hours/week for heavy AI users)
- Integrated Code, Agents, Terminal, and Drive modules
- 75 million+ lines of code generated

**Integration Points**:
```bash
# Launch Warp 2.0 environment
warp

# Multi-agent task execution
warp agent create --name "backend-dev" --task "API development"
warp agent create --name "frontend-dev" --task "UI implementation"

# Agent monitoring and management
warp agents list
warp agent status backend-dev
```

### Pieces CLI Agent

**Provider**: Pieces for Developers  
**Command**: `pieces`  
**Release**: 2024-2025  
**Focus**: Developer productivity and snippet management

**Key Features**:
- AI-powered code snippet management
- Workflow context capture and retrieval
- In-project AI chat integration
- Personalized AI copilot for development tasks
- Terminal-based access to snippet library

**Integration Points**:
```bash
# Save and manage code snippets
pieces save "function implementation" --file util.rs

# AI-powered code search
pieces ask "How do I implement authentication in Rust?"

# Workflow context integration
pieces context save --project "maos" --description "Agent integration work"
```

## Architecture Design

### Agent Adapter Pattern

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;

/// Core agent abstraction for all CLI-based agents
#[async_trait::async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Execute a task with the agent
    async fn execute_task(&self, task: &Task) -> Result<TaskResult, AgentError>;
    
    /// Check if agent is available and responsive
    async fn check_availability(&self) -> Result<AgentStatus, AgentError>;
    
    /// Get agent capabilities
    async fn get_capabilities(&self) -> Result<Vec<Capability>, AgentError>;
    
    /// Health check for monitoring
    async fn health_check(&self) -> Result<HealthStatus, AgentError>;
    
    /// Get agent metadata
    fn get_metadata(&self) -> AgentMetadata;
}

/// Claude Code CLI adapter implementation
pub struct ClaudeCodeAdapter {
    command_path: PathBuf,
    session_manager: SessionManager,
    config: ClaudeConfig,
}

#[async_trait::async_trait]
impl AgentAdapter for ClaudeCodeAdapter {
    async fn execute_task(&self, task: &Task) -> Result<TaskResult, AgentError> {
        let mut cmd = CommandBuilder::claude()
            .with_timeout(Duration::from_secs(300));
        
        // Configure command based on task type
        match task.task_type() {
            TaskType::CodeAnalysis => {
                if let Some(file_path) = task.file_path() {
                    cmd = cmd.with_file(file_path);
                }
                cmd = cmd.with_prompt(task.prompt());
            }
            TaskType::CodeGeneration => {
                if let Some(output_path) = task.output_path() {
                    cmd = cmd.with_output(output_path);
                }
                cmd = cmd.with_prompt(task.prompt());
            }
            TaskType::Interactive => {
                if let Some(session_id) = task.session_id() {
                    cmd = cmd.with_session(session_id);
                }
                cmd = cmd.with_prompt(task.prompt());
            }
            TaskType::MCPIntegration => {
                cmd = cmd.with_mcp_mode()
                    .with_prompt(task.prompt());
            }
        }
        
        // Execute and capture output
        let output = cmd.execute().await?;
        TaskResult::from_command_output(output)
    }

    async fn get_capabilities(&self) -> Result<Vec<Capability>, AgentError> {
        Ok(vec![
            Capability::CodeAnalysis,
            Capability::CodeGeneration,
            Capability::FileOperations,
            Capability::InteractiveSessions,
            Capability::MCPIntegration,
            Capability::ContextManagement,
        ])
    }
}

/// Google Gemini CLI adapter
pub struct GeminiAdapter {
    command_path: PathBuf,
    config: GeminiConfig,
    api_key: Option<String>,
}

#[async_trait::async_trait]
impl AgentAdapter for GeminiAdapter {
    async fn execute_task(&self, task: &Task) -> Result<TaskResult, AgentError> {
        let mut cmd = CommandBuilder::gemini()
            .with_timeout(Duration::from_secs(600)); // Longer timeout for large contexts
        
        match task.task_type() {
            TaskType::CodeAnalysis => {
                cmd = cmd.with_context_window(task.context_size().unwrap_or(1_000_000))
                    .with_prompt(task.prompt());
            }
            TaskType::MultimodalAnalysis => {
                if let Some(image_path) = task.image_path() {
                    cmd = cmd.with_image(image_path);
                }
                cmd = cmd.with_prompt(task.prompt());
            }
            TaskType::ResearchIntegration => {
                cmd = cmd.with_search()
                    .with_prompt(task.prompt());
            }
            TaskType::MediaGeneration => {
                cmd = cmd.with_veo()
                    .with_prompt(task.prompt());
            }
            _ => {
                cmd = cmd.with_prompt(task.prompt());
            }
        }
        
        let output = cmd.execute().await?;
        TaskResult::from_command_output(output)
    }

    async fn get_capabilities(&self) -> Result<Vec<Capability>, AgentError> {
        Ok(vec![
            Capability::CodeAnalysis,
            Capability::LargeContextProcessing,
            Capability::MultimodalAnalysis,
            Capability::ResearchIntegration,
            Capability::MediaGeneration,
            Capability::MCPIntegration,
        ])
    }
}
```

### Command Builder Framework

```rust
/// Flexible command builder for different CLI agents
pub struct CommandBuilder {
    base_command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    working_dir: Option<PathBuf>,
    timeout: Option<Duration>,
}

impl CommandBuilder {
    /// Create Claude Code command builder
    pub fn claude() -> Self {
        Self {
            base_command: "claude".to_string(),
            args: Vec::new(),
            env: HashMap::new(),
            working_dir: None,
            timeout: Some(Duration::from_secs(300)),
        }
    }
    
    /// Create Gemini CLI command builder
    pub fn gemini() -> Self {
        Self {
            base_command: "gemini".to_string(),
            args: Vec::new(),
            env: HashMap::new(),
            working_dir: None,
            timeout: Some(Duration::from_secs(600)),
        }
    }
    
    /// Create OpenAI Codex command builder
    pub fn codex() -> Self {
        Self {
            base_command: "codex".to_string(),
            args: Vec::new(),
            env: HashMap::new(),
            working_dir: None,
            timeout: Some(Duration::from_secs(300)),
        }
    }
    
    /// Create GitHub Copilot command builder
    pub fn gh_copilot() -> Self {
        Self {
            base_command: "gh".to_string(),
            args: vec!["copilot".to_string()],
            env: HashMap::new(),
            working_dir: None,
            timeout: Some(Duration::from_secs(60)),
        }
    }
    
    /// Add file parameter
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.args.push("--file".to_string());
        self.args.push(path.as_ref().to_string_lossy().to_string());
        self
    }
    
    /// Add prompt/query
    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.args.push(prompt.to_string());
        self
    }
    
    /// Add session continuation
    pub fn with_session(mut self, session_id: &str) -> Self {
        self.args.push("--continue".to_string());
        self.args.push(session_id.to_string());
        self
    }
    
    /// Add image input (multimodal)
    pub fn with_image<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.args.push("--image".to_string());
        self.args.push(path.as_ref().to_string_lossy().to_string());
        self
    }
    
    /// Enable search integration
    pub fn with_search(mut self) -> Self {
        self.args.push("--search".to_string());
        self
    }
    
    /// Enable MCP mode
    pub fn with_mcp_mode(mut self) -> Self {
        self.args.push("mcp".to_string());
        self.args.push("serve".to_string());
        self
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Execute the command
    pub async fn execute(self) -> Result<CommandOutput, AgentError> {
        let mut cmd = Command::new(self.base_command);
        cmd.args(self.args);
        
        if let Some(dir) = self.working_dir {
            cmd.current_dir(dir);
        }
        
        for (key, value) in self.env {
            cmd.env(key, value);
        }
        
        // Execute with timeout
        let output = match self.timeout {
            Some(timeout_duration) => {
                timeout(timeout_duration, cmd.output()).await
                    .map_err(|_| AgentError::Timeout)?
                    .map_err(AgentError::CommandFailed)?
            }
            None => cmd.output().await.map_err(AgentError::CommandFailed)?,
        };
        
        Ok(CommandOutput::from(output))
    }
}
```

## Configuration Management

### Agent Discovery and Validation

```rust
/// Discovers available AI CLI agents on the system
pub struct AgentDiscovery {
    search_paths: Vec<PathBuf>,
    known_agents: HashMap<String, AgentMetadata>,
}

impl AgentDiscovery {
    pub async fn discover_agents(&self) -> Result<Vec<DiscoveredAgent>, DiscoveryError> {
        let mut agents = Vec::new();
        
        // Claude Code CLI
        if let Some(claude_path) = self.find_command("claude").await? {
            let version = self.get_claude_version(&claude_path).await?;
            agents.push(DiscoveredAgent {
                name: "claude".to_string(),
                command: "claude".to_string(),
                path: claude_path,
                version,
                capabilities: vec![
                    Capability::CodeAnalysis,
                    Capability::CodeGeneration,
                    Capability::FileOperations,
                    Capability::InteractiveSessions,
                    Capability::MCPIntegration,
                ],
                status: AgentStatus::Available,
            });
        }
        
        // Google Gemini CLI
        if let Some(gemini_path) = self.find_command("gemini").await? {
            let version = self.get_gemini_version(&gemini_path).await?;
            agents.push(DiscoveredAgent {
                name: "gemini".to_string(),
                command: "gemini".to_string(),
                path: gemini_path,
                version,
                capabilities: vec![
                    Capability::CodeAnalysis,
                    Capability::LargeContextProcessing,
                    Capability::MultimodalAnalysis,
                    Capability::ResearchIntegration,
                    Capability::MediaGeneration,
                ],
                status: AgentStatus::Available,
            });
        }
        
        // OpenAI Codex CLI
        if let Some(codex_path) = self.find_command("codex").await? {
            let version = self.get_codex_version(&codex_path).await?;
            agents.push(DiscoveredAgent {
                name: "codex".to_string(),
                command: "codex".to_string(),
                path: codex_path,
                version,
                capabilities: vec![
                    Capability::CodeAnalysis,
                    Capability::CodeGeneration,
                    Capability::MultimodalAnalysis,
                    Capability::VersionControl,
                ],
                status: AgentStatus::Available,
            });
        }
        
        // GitHub Copilot CLI
        if let Some(gh_path) = self.find_command("gh").await? {
            if self.check_copilot_extension(&gh_path).await? {
                agents.push(DiscoveredAgent {
                    name: "github-copilot".to_string(),
                    command: "gh copilot".to_string(),
                    path: gh_path,
                    version: None,
                    capabilities: vec![
                        Capability::CommandSuggestion,
                        Capability::CommandExplanation,
                        Capability::GitIntegration,
                    ],
                    status: AgentStatus::Available,
                });
            }
        }
        
        // Pieces CLI
        if let Some(pieces_path) = self.find_command("pieces").await? {
            let version = self.get_pieces_version(&pieces_path).await?;
            agents.push(DiscoveredAgent {
                name: "pieces".to_string(),
                command: "pieces".to_string(),
                path: pieces_path,
                version,
                capabilities: vec![
                    Capability::SnippetManagement,
                    Capability::ContextManagement,
                    Capability::WorkflowIntegration,
                ],
                status: AgentStatus::Available,
            });
        }
        
        Ok(agents)
    }
}
```

### Agent Configuration

```yaml
# maos.yaml - Configuration file
agents:
  claude:
    command: "claude"
    path: "/usr/local/bin/claude"  # Auto-detected if not specified
    priority: 1  # Highest priority for task routing
    capabilities:
      - code_analysis
      - code_generation
      - file_operations
      - interactive_sessions
      - mcp_integration
    config:
      model: "claude-3-7-sonnet"
      session_persistence: true
      max_concurrent_tasks: 3
      timeout: 300
    
  gemini:
    command: "gemini"
    path: "/usr/local/bin/gemini"
    priority: 2
    capabilities:
      - code_analysis
      - large_context_processing
      - multimodal_analysis
      - research_integration
      - media_generation
    config:
      model: "gemini-2.5-pro"
      context_window: 1000000
      max_concurrent_tasks: 2
      timeout: 600
      
  codex:
    command: "codex"
    path: "/usr/local/bin/codex"
    priority: 3
    capabilities:
      - code_analysis
      - code_generation
      - multimodal_analysis
      - version_control
    config:
      model: "o3"
      max_concurrent_tasks: 2
      timeout: 300
      
  github-copilot:
    command: "gh copilot"
    path: "/usr/local/bin/gh"
    priority: 4
    capabilities:
      - command_suggestion
      - command_explanation
      - git_integration
    config:
      max_concurrent_tasks: 5
      timeout: 60
      
  pieces:
    command: "pieces"
    path: "/usr/local/bin/pieces"
    priority: 5
    capabilities:
      - snippet_management
      - context_management
      - workflow_integration
    config:
      max_concurrent_tasks: 3
      timeout: 120
```

## Task Routing Strategy

### Capability-Based Routing

```rust
/// Routes tasks to appropriate agents based on capabilities and priority
pub struct TaskRouter {
    agents: HashMap<AgentId, Box<dyn AgentAdapter>>,
    capability_map: HashMap<Capability, Vec<AgentId>>,
    agent_priority: HashMap<AgentId, u8>,
    load_balancer: LoadBalancer,
}

impl TaskRouter {
    pub async fn route_task(&self, task: &Task) -> Result<AgentId, RoutingError> {
        let required_capabilities = task.required_capabilities();
        
        // Find agents that can handle all required capabilities
        let mut candidate_agents = Vec::new();
        
        for agent_id in self.agents.keys() {
            let agent = self.agents.get(agent_id).unwrap();
            let agent_capabilities = agent.get_capabilities().await?;
            
            if required_capabilities.iter().all(|cap| agent_capabilities.contains(cap)) {
                candidate_agents.push((
                    agent_id.clone(),
                    self.agent_priority.get(agent_id).copied().unwrap_or(99)
                ));
            }
        }
        
        if candidate_agents.is_empty() {
            return Err(RoutingError::NoCapableAgent);
        }
        
        // Sort by priority (lower number = higher priority)
        candidate_agents.sort_by_key(|(_, priority)| *priority);
        
        // Apply load balancing among agents with same priority
        let top_priority = candidate_agents[0].1;
        let top_agents: Vec<AgentId> = candidate_agents
            .into_iter()
            .filter(|(_, priority)| *priority == top_priority)
            .map(|(agent_id, _)| agent_id)
            .collect();
        
        let selected_agent = self.load_balancer.select_agent(&top_agents).await?;
        Ok(selected_agent)
    }
}
```

## Error Handling and Resilience

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Command execution failed: {0}")]
    CommandFailed(#[from] std::io::Error),
    
    #[error("Agent not available: {0}")]
    AgentUnavailable(String),
    
    #[error("Timeout executing command")]
    Timeout,
    
    #[error("Invalid agent response: {0}")]
    InvalidResponse(String),
    
    #[error("Authentication failed for agent: {0}")]
    AuthenticationFailed(String),
    
    #[error("Agent capability not supported: {capability} by {agent}")]
    UnsupportedCapability { agent: String, capability: String },
    
    #[error("Session management error: {0}")]
    SessionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Retry policy for agent operations
pub struct RetryPolicy {
    max_attempts: usize,
    backoff_strategy: BackoffStrategy,
    retry_conditions: Vec<Box<dyn Fn(&AgentError) -> bool + Send + Sync>>,
}

impl RetryPolicy {
    pub fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_strategy: BackoffStrategy::ExponentialBackoff {
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(10),
                multiplier: 2.0,
            },
            retry_conditions: vec![
                Box::new(|e| matches!(e, AgentError::Timeout)),
                Box::new(|e| matches!(e, AgentError::CommandFailed(_))),
            ],
        }
    }
}
```

## Testing Strategy

### Integration Testing with Real CLIs

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_claude_code_integration() {
        if !agent_available("claude").await {
            return;
        }
        
        let adapter = ClaudeCodeAdapter::new();
        let task = Task::new(
            TaskType::CodeAnalysis,
            "Analyze this function for potential improvements".to_string(),
        ).with_file("src/test_file.rs");
        
        let result = adapter.execute_task(&task).await.unwrap();
        assert!(result.success());
        assert!(!result.output().is_empty());
    }
    
    #[tokio::test]
    async fn test_gemini_multimodal() {
        if !agent_available("gemini").await {
            return;
        }
        
        let adapter = GeminiAdapter::new();
        let task = Task::new(
            TaskType::MultimodalAnalysis,
            "Analyze this architecture diagram".to_string(),
        ).with_image("docs/architecture.png");
        
        let result = adapter.execute_task(&task).await.unwrap();
        assert!(result.success());
    }
    
    #[tokio::test]
    async fn test_agent_discovery() {
        let discovery = AgentDiscovery::new();
        let agents = discovery.discover_agents().await.unwrap();
        
        // Should find at least one agent
        assert!(!agents.is_empty());
        
        // Verify agent metadata
        for agent in agents {
            assert!(!agent.capabilities.is_empty());
            assert!(agent.path.exists());
        }
    }
}
```

## Development Priorities

### Phase 1: Foundation (Weeks 2-3)
1. Implement `ClaudeCodeAdapter` (primary integration)
2. Create `AgentDiscovery` system
3. Build `CommandBuilder` framework
4. Add configuration management

### Phase 2: Multi-Agent Support (Weeks 4-5)
1. Implement `GeminiAdapter`
2. Add `CodexAdapter`
3. Create task routing system
4. Implement load balancing

### Phase 3: Enhanced Integration (Weeks 6-7)
1. Add `GitHubCopilotAdapter`
2. Implement `PiecesAdapter`
3. Add session management
4. Create monitoring and health checks

### Phase 4: Advanced Features (Weeks 8-9)
1. Multi-agent collaboration workflows
2. Agent-specific optimizations
3. Performance monitoring
4. Advanced error recovery

This CLI-centric approach provides a robust, future-proof foundation for integrating the latest AI agents through their standardized command-line interfaces! 🚀