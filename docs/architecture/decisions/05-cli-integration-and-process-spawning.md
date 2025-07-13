# ADR-05: CLI Integration and Process Spawning

## Status
Accepted

## Context
MAOS needs to spawn and manage AI CLI processes (Claude Code, GPT, Gemini, etc.) to execute orchestrated tasks. Each CLI has different:
- Command-line interfaces and arguments
- Output formats and streaming behaviors
- Authentication mechanisms
- Error patterns and exit codes

Key insights from our architecture:
- MAOS is an MCP server that spawns CLI processes
- Agents need isolated workspaces and shared context
- Environment variables provide configuration without modifying prompts
- JSON output enables structured communication

## Decision
We adopt a **CLI integration strategy** that supports multiple external AI tools through standardized patterns:
1. **Detects available AI CLIs** and their capabilities
2. **Abstracts CLI differences** through unified configuration
3. **Provides CLI-specific error handling** and authentication patterns
4. **Integrates with ADR-08's process management** for agent execution

### Architectural Layering

This ADR focuses on external CLI integration patterns, while delegating process management to ADR-08:

- **ADR-05 provides**: CLI detection, tool-specific configurations, command-line patterns, error handling
- **ADR-08 uses**: These CLI configurations for actual agent process spawning and management
- **Relationship**: ADR-05 provides the "what CLIs are available", ADR-08 handles the "how to run them"

### Agent Prompt Template

The base agent prompt template and communication protocol are documented in the [Agent Roles Reference](../references/agent-roles.md#base-prompt-template).

### CLI-Specific Configuration Patterns

Each CLI has different command-line interfaces and capabilities. This ADR defines standardized configuration patterns that ADR-08 uses for process spawning:

```rust
#[derive(Debug, Clone)]
pub struct CliConfiguration {
    pub cli_type: CliType,
    pub command: String,
    pub supports_json: bool,
    pub max_turns_flag: Option<String>,
    pub timeout_flag: Option<String>,
    pub output_format_flag: Option<String>,
    pub authentication_check: fn() -> Result<bool>,
    pub error_parser: fn(&str, i32) -> AgentError,
}

pub enum CliType {
    Claude,
    GPT,
    Gemini,
    Ollama,
    // Extensible for new CLIs
}
```

### CLI Detection and Registry

```rust
pub struct CliRegistry {
    available_clis: HashMap<CliType, CliInfo>,
}

impl CliRegistry {
    pub async fn detect_available_clis(&mut self) -> Result<()> {
        // Check for Claude
        if let Ok(output) = Command::new("which").arg("claude").output().await {
            if output.status.success() {
                self.available_clis.insert(CliType::Claude, CliInfo {
                    command: "claude",
                    version: self.get_claude_version().await?,
                    supports_json: true,
                    max_turns_flag: "--max-turns",
                });
            }
        }
        
        // Check for other CLIs...
        // Similar patterns for GPT, Gemini, Ollama
        
        Ok(())
    }
}
```

### Error Handling

```rust
pub fn parse_cli_error(cli_type: CliType, stderr: &str, exit_code: i32) -> AgentError {
    match cli_type {
        CliType::Claude => {
            if stderr.contains("Not authenticated") {
                AgentError::AuthRequired("Claude Code not authenticated. Run: claude login")
            } else if stderr.contains("Rate limit") {
                AgentError::RateLimit("Claude API rate limit exceeded")
            } else {
                AgentError::CliError(format!("Claude exited with code {}: {}", exit_code, stderr))
            }
        }
        // Similar patterns for other CLIs...
    }
}
```

## Environment Variables

MAOS uses environment variables to configure spawned agents. The complete list of variables and their usage is documented in the [Environment Variables Reference](../references/environment-variables.md).

## Consequences

### Positive
- **Multi-CLI Support**: Works with any AI CLI supporting non-interactive mode
- **Extensible**: Easy to add support for new AI tools
- **Unified Interface**: Abstracts CLI differences through standardized configuration
- **Robust Error Handling**: CLI-specific error parsing and authentication checks
- **Foundation for Process Management**: Provides configuration that ADR-08 uses for execution

### Negative
- **CLI Dependency**: Requires external AI CLIs to be installed
- **Platform Specific**: Some CLI behaviors vary by OS
- **Version Compatibility**: CLI changes may require configuration updates
- **Authentication Complexity**: Each CLI has different auth mechanisms

### Mitigation
- Clear error messages for missing or misconfigured CLIs
- Graceful degradation when CLIs are unavailable
- Platform-specific handling where needed
- Version detection and compatibility warnings

## References
- **ADR-08: Agent Lifecycle and Management** - Uses CLI configurations from this ADR for process spawning
- ADR-02: Hybrid Storage Strategy - Environment variable configuration  
- Claude Code CLI documentation and command-line patterns
- External AI CLI documentation (GPT, Gemini, Ollama)
- Cross-platform process execution patterns

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*