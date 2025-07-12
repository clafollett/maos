# ADR-004: CLI Integration and Process Spawning

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
We adopt a **process spawning strategy** that:
1. **Uses `claude -p` pattern** for non-interactive execution
2. **Configures agents via MAOS environment variables**
3. **Isolates agent workspaces** while enabling controlled sharing
4. **Streams output back to MCP client** in real-time

### Process Spawning Architecture

```rust
pub async fn spawn_agent(
    &self,
    role: AgentRole,
    task: &str,
    session_id: &str,
    agent_id: &str,
) -> Result<Child> {
    // Prepare isolated workspace
    let workspace = self.prepare_agent_workspace(session_id, agent_id)?;
    
    // Build environment variables
    let mut env_vars = HashMap::new();
    env_vars.insert("MAOS_AGENT_ROLE", role.to_string());
    env_vars.insert("MAOS_SESSION_ID", session_id.to_string());
    env_vars.insert("MAOS_AGENT_ID", agent_id.to_string());
    env_vars.insert("MAOS_WORKSPACE", workspace.to_string());
    env_vars.insert("MAOS_SHARED_CONTEXT", format!("~/.maos/projects/{}/sessions/{}/shared/context", 
        self.workspace_hash, session_id));
    env_vars.insert("MAOS_MESSAGE_DIR", format!("~/.maos/projects/{}/sessions/{}/shared/messages",
        self.workspace_hash, session_id));
    env_vars.insert("MAOS_PROJECT_ROOT", self.project_root.to_string());
    
    // Build the prompt with role-specific instructions
    let role_instructions = RoleInstructions::new();
    let instructions = role_instructions.get_instructions(&role);
    let prompt = self.build_agent_prompt(role, task, &instructions, &env_vars)?;
    
    // Spawn the process
    let mut cmd = Command::new("claude");
    cmd.current_dir(&workspace)
        .arg("-p")
        .arg(&prompt)
        .arg("--output-format").arg("json")
        .arg("--max-turns").arg("10")
        .envs(&env_vars)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    let child = cmd.spawn()?;
    
    // Start output streaming
    self.start_output_streaming(session_id, agent_id, &child).await?;
    
    Ok(child)
}
```

### Agent Prompt Template

The base agent prompt template and communication protocol are documented in the [Agent Roles Reference](../references/agent-roles.md#base-prompt-template).

### Role-Specific Instructions

Predefined role instructions and custom role generation are documented in the [Agent Roles Reference](../references/agent-roles.md#predefined-roles). The system includes instructions for all 13 predefined roles and automatically generates appropriate instructions for custom roles.

### Output Streaming

```rust
async fn start_output_streaming(
    &self,
    session_id: &str,
    agent_id: &str,
    child: &Child,
) -> Result<()> {
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    // Stream stdout
    let stdout_path = format!("~/.maos/projects/{}/sessions/{}/agents/{}/stdout.log",
        self.workspace_hash, session_id, agent_id);
    tokio::spawn(stream_to_file_and_mcp(stdout, stdout_path, self.mcp_server.clone()));
    
    // Stream stderr
    let stderr_path = format!("~/.maos/projects/{}/sessions/{}/agents/{}/stderr.log",
        self.workspace_hash, session_id, agent_id);
    tokio::spawn(stream_to_file_and_mcp(stderr, stderr_path, self.mcp_server.clone()));
    
    Ok(())
}

async fn stream_to_file_and_mcp(
    reader: impl AsyncRead + Unpin,
    file_path: String,
    mcp_server: Arc<McpServer>,
) {
    let mut reader = BufReader::new(reader);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .await
        .unwrap();
    
    let mut line = String::new();
    while reader.read_line(&mut line).await.unwrap() > 0 {
        // Write to file
        file.write_all(line.as_bytes()).await.unwrap();
        
        // Stream to MCP client
        if let Ok(json) = serde_json::from_str::<Value>(&line) {
            mcp_server.send_agent_update(agent_id, json).await;
        } else {
            mcp_server.send_agent_output(agent_id, &line).await;
        }
        
        line.clear();
    }
}
```

### Dynamic Template Generation

For custom roles, we generate appropriate prompts and configurations:

```rust
pub struct DynamicTemplateBuilder {
    default_timeout: Duration,
    default_memory_mb: u32,
}

impl DynamicTemplateBuilder {
    pub fn build_custom_prompt(
        &self,
        role: &AgentRole,
        task: &str,
        agent_id: &str,
        session_id: &str,
        instance_num: usize,
    ) -> String {
        let custom_desc = if !role.is_predefined {
            format!("\n- Role Description: {}", role.description)
        } else {
            String::new()
        };
        
        format!(
            AGENT_PROMPT_TEMPLATE,
            role_name = role.name,
            agent_id = agent_id,
            session_id = session_id,
            instance_number = instance_num,
            custom_role_desc = custom_desc,
            task = task,
            role_instructions = RoleInstructions::new().get_instructions(role),
        )
    }
    
    pub fn infer_required_tools(&self, role: &AgentRole) -> Vec<String> {
        let mut tools = vec!["read".to_string(), "write".to_string()];
        
        // Infer tools based on role name and description
        let role_text = format!("{} {}", role.name, role.description).to_lowercase();
        
        if role_text.contains("code") || role_text.contains("implement") 
            || role_text.contains("engineer") || role_text.contains("develop") {
            tools.push("bash".to_string());
        }
        
        if role_text.contains("test") || role_text.contains("qa") {
            tools.push("bash".to_string());
        }
        
        tools
    }
    
    pub fn suggest_timeout(&self, role: &AgentRole) -> Duration {
        // Longer timeouts for complex roles
        let role_text = format!("{} {}", role.name, role.description).to_lowercase();
        
        if role_text.contains("architect") || role_text.contains("design") {
            Duration::from_secs(3600) // 1 hour
        } else if role_text.contains("implement") || role_text.contains("code") {
            Duration::from_secs(7200) // 2 hours
        } else {
            self.default_timeout
        }
    }
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
- **CLI Agnostic**: Works with any CLI supporting non-interactive mode
- **Clean Separation**: Environment variables don't pollute prompts
- **Debuggable**: All I/O captured in log files
- **Flexible Roles**: Support for both predefined and custom agent roles
- **Dynamic Templates**: Automatic generation for custom roles
- **Multiple Instances**: Clear identification of agent instances
- **Streaming**: Real-time visibility of agent work

### Negative
- **Process Overhead**: Spawning processes has startup cost
- **CLI Dependency**: Requires CLIs to be installed
- **Platform Specific**: Some CLI behaviors vary by OS
- **Template Quality**: Generated templates may need refinement

### Mitigation
- Process pooling for frequently used configurations
- Clear error messages for missing CLIs
- Platform-specific handling where needed

## References
- [Agent Roles Reference](../references/agent-roles.md) - Role definitions and templates
- [Environment Variables Reference](../references/environment-variables.md) - Complete variable list
- Claude Code CLI documentation
- Unix process model and environment variables
- MCP streaming specifications

---
*Date: 2025-07-09*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*