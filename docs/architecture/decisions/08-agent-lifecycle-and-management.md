# ADR-08: Agent Lifecycle and Management

## Status
Accepted

## Context
MAOS needs to manage the complete lifecycle of AI agent processes, from spawning to termination. With our architecture of MAOS as an MCP server spawning CLI processes, we need:

- Reliable process spawning and monitoring
- Agent role specialization with both predefined and custom roles
- Support for multiple instances of the same role
- Resource management and limits
- Graceful shutdown and cleanup
- Health monitoring and recovery
- Output capture and streaming

Key insights:
- Each agent is a separate CLI process (`claude -p`, etc.)
- Agents can have predefined roles (architect, engineer, etc.) or custom roles
- Multiple agents of the same role can work in parallel
- Agents work in isolated workspaces with shared context
- Output streams back to the MCP client in real-time

## Decision
We will implement comprehensive agent lifecycle management with flexible role-based specialization supporting both predefined and custom roles, multiple instances, and robust process handling.

### Architectural Layering

This ADR provides the low-level process management infrastructure that higher-level orchestration builds upon:

- **ADR-08 provides**: Process spawning, resource management, health monitoring, and lifecycle state management
- **ADR-03 uses**: This infrastructure for session-level orchestration and agent coordination
- **ADR-05 provides**: CLI configurations and integration patterns that this ADR uses for spawning
- **Relationship**: ADR-08 handles the "how" of agent processes, ADR-03 handles the "when/why" of orchestration, ADR-05 handles the "what CLIs are available"

### Unified State Model Integration

This ADR uses the unified state model for consistent agent lifecycle management across all orchestration layers. The process management layer uses `AgentExecutionState` for tracking individual agent lifecycle:

```
┌─────────┐      ┌─────────┐      ┌───────────┐
│ Pending │ ───► │ Running │ ───► │ Completed │
└─────────┘      └────┬────┘      └───────────┘
                      │
                      ├─────► ┌─────────┐
                      │       │ Failed  │
                      │       └─────────┘
                      │
                      ├─────► ┌───────────┐
                      │       │ Resumable │ (for recovery)
                      │       └───────────┘
                      │
                      └─────► ┌───────────┐
                              │ Cancelled │
                              └───────────┘
```

The process manager also tracks low-level process states separately for technical monitoring.

### Agent Role Definitions

Agent roles and templates are comprehensively documented in the [Agent Roles Reference](../references/agent-roles.md). This includes:

- **AgentRole structure**: Core properties for both predefined and custom roles
- **Predefined roles**: 13 built-in roles with specific capabilities and resource limits
- **AgentTemplate**: Runtime configuration including timeouts, memory limits, and required tools
- **Custom role support**: Dynamic template generation for user-defined roles

### Process Management

```rust
pub struct AgentProcess {
    pub agent_id: String,
    pub role: AgentRole,
    pub instance_number: usize,  // 1, 2, 3, etc. for multiple instances
    pub child: Child,
    pub spawned_at: Instant,
    pub workspace: PathBuf,
    pub stdout_handle: JoinHandle<()>,
    pub stderr_handle: JoinHandle<()>,
    pub health_check_handle: JoinHandle<()>,
}

// Track instance counts per role
pub struct InstanceTracker {
    role_instances: Arc<RwLock<HashMap<String, usize>>>,
}

impl InstanceTracker {
    pub async fn get_next_instance_number(&self, role_name: &str) -> usize {
        let mut instances = self.role_instances.write().await;
        let count = instances.entry(role_name.to_string()).or_insert(0);
        *count += 1;
        *count
    }
    
    pub async fn release_instance_number(&self, role_name: &str) {
        let mut instances = self.role_instances.write().await;
        if let Some(count) = instances.get_mut(role_name) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
}

pub struct ProcessManager {
    agents: Arc<RwLock<HashMap<String, AgentProcess>>>,
    resource_limiter: ResourceLimiter,
    health_monitor: HealthMonitor,
    instance_tracker: InstanceTracker,
    template_generator: TemplateGenerator,
}

impl ProcessManager {
    pub async fn spawn_agent(
        &self,
        session_id: &str,
        role: AgentRole,
        task: &str,
        dependencies: Vec<String>,
    ) -> Result<String> {
        // Get or generate template for role
        let template = if role.is_predefined {
            PREDEFINED_TEMPLATES.get(&role.name)
                .ok_or_else(|| anyhow!("Unknown predefined role: {}", role.name))?
                .clone()
        } else {
            // Generate template for custom role
            self.template_generator.generate_custom_template(&role).await?
        };
        
        // Check resource limits
        self.resource_limiter.check_can_spawn(&role.name, &template).await?;
        
        // Get instance number for this role
        let instance_num = self.instance_tracker.get_next_instance_number(&role.name).await;
        
        // Generate agent ID with role and instance info
        let agent_id = generate_agent_id_with_role(&role, instance_num);
        let workspace = self.prepare_workspace(session_id, &agent_id)?;
        
        // Build environment
        let env = self.build_agent_env(session_id, &agent_id, &role, instance_num, &workspace)?;
        
        // Build prompt from template
        let prompt = self.build_prompt(&template, &role, task, dependencies)?;
        
        // Spawn process
        let mut cmd = Command::new("claude");
        cmd.current_dir(&workspace)
            .arg("-p")
            .arg(&prompt)
            .arg("--output-format").arg("json")
            .arg("--max-turns").arg(template.default_timeout.as_secs().to_string())
            .envs(&env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
            
        // Apply resource limits
        #[cfg(target_os = "linux")]
        {
            cmd.pre_exec(move || {
                // Set memory limit
                let rlimit = libc::rlimit {
                    rlim_cur: (template.max_memory_mb * 1024 * 1024) as u64,
                    rlim_max: (template.max_memory_mb * 1024 * 1024) as u64,
                };
                unsafe {
                    libc::setrlimit(libc::RLIMIT_AS, &rlimit);
                }
                Ok(())
            });
        }
        
        let mut child = cmd.spawn()?;
        
        // Start output streaming
        let stdout_handle = self.start_stdout_streaming(&agent_id, child.stdout.take().unwrap());
        let stderr_handle = self.start_stderr_streaming(&agent_id, child.stderr.take().unwrap());
        
        // Start health monitoring
        let health_handle = self.start_health_monitoring(&agent_id, child.id());
        
        // Register agent
        let agent = AgentProcess {
            agent_id: agent_id.clone(),
            role,
            instance_number: instance_num,
            child,
            spawned_at: Instant::now(),
            workspace,
            stdout_handle,
            stderr_handle,
            health_check_handle: health_handle,
        };
        
        self.agents.write().await.insert(agent_id.clone(), agent);
        
        // Log spawn event
        self.logger.log_event("agent_spawned", json!({
            "agent_id": agent_id,
            "role_name": role.name,
            "role_type": if role.is_predefined { "predefined" } else { "custom" },
            "instance_number": instance_num,
            "task": task,
            "pid": child.id(),
        }))?;
        
        Ok(agent_id)
    }
}

// Helper function to generate agent IDs with role and instance info
fn generate_agent_id_with_role(role: &AgentRole, instance_num: usize) -> String {
    let base_id = Uuid::new_v4().simple().to_string();
    let short_id = &base_id[..8];
    
    if let Some(suffix) = &role.instance_suffix {
        format!("agent_{}_{}_{}_{}", role.name, suffix, instance_num, short_id)
    } else {
        format!("agent_{}_{}_{}", role.name, instance_num, short_id)
    }
}
```

### Health Monitoring

```rust
impl ProcessManager {
    async fn start_health_monitoring(&self, agent_id: &str, pid: u32) -> JoinHandle<()> {
        let agent_id = agent_id.to_string();
        let agents = self.agents.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                // Check if process still exists
                match System::new_with_specifics(RefreshKind::new().with_processes()) {
                    sys if sys.process(Pid::from(pid as usize)).is_some() => {
                        // Process healthy
                        continue;
                    }
                    _ => {
                        // Process died unexpectedly
                        if let Some(mut agent) = agents.write().await.remove(&agent_id) {
                            logger.log_event("agent_died", json!({
                                "agent_id": agent_id,
                                "pid": pid,
                                "uptime_seconds": agent.spawned_at.elapsed().as_secs(),
                            }));
                            
                            // Cancel streaming handles
                            agent.stdout_handle.abort();
                            agent.stderr_handle.abort();
                        }
                        break;
                    }
                }
            }
        })
    }
}
```

### Template Generation for Custom Roles

Template generation for custom roles is handled by the TemplateGenerator, which automatically:
- Infers capabilities from the role description
- Determines required tools based on capabilities
- Generates appropriate prompt templates

The complete template generation logic and prompt format are documented in the [Agent Roles Reference](../references/agent-roles.md#custom-role-support).

### Resource Management

```rust
pub struct ResourceLimiter {
    max_total_agents: usize,
    max_agents_per_role: HashMap<String, usize>,  // role_name -> limit
    default_max_per_role: usize,  // For undefined roles
    max_total_memory_mb: u32,
    current_usage: Arc<RwLock<ResourceUsage>>,
}

struct ResourceUsage {
    agent_count: usize,
    role_counts: HashMap<String, usize>,  // role_name -> count
    total_memory_mb: u32,
}

impl ResourceLimiter {
    pub async fn check_can_spawn(&self, role_name: &str, template: &AgentTemplate) -> Result<()> {
        let mut usage = self.current_usage.write().await;
        
        // Check total agent limit
        if usage.agent_count >= self.max_total_agents {
            return Err(anyhow!("Maximum agent limit reached"));
        }
        
        // Check role-specific limit
        let limit = self.max_agents_per_role.get(role_name)
            .copied()
            .unwrap_or(self.default_max_per_role);
            
        let current_count = usage.role_counts.get(role_name).copied().unwrap_or(0);
        if current_count >= limit {
            return Err(anyhow!("Maximum {} agents reached ({}/{})", role_name, current_count, limit));
        }
        
        // Check memory limit
        if usage.total_memory_mb + template.max_memory_mb > self.max_total_memory_mb {
            return Err(anyhow!("Insufficient memory for new agent"));
        }
        
        // Update usage
        usage.agent_count += 1;
        *usage.role_counts.entry(role_name.to_string()).or_insert(0) += 1;
        usage.total_memory_mb += template.max_memory_mb;
        
        Ok(())
    }
}
```

### Graceful Shutdown

```rust
impl ProcessManager {
    pub async fn shutdown_all(&self, timeout: Duration) -> Result<()> {
        let agents = self.agents.read().await;
        let shutdown_futures: Vec<_> = agents.values()
            .map(|agent| self.shutdown_agent(&agent.agent_id, timeout))
            .collect();
            
        // Shutdown all agents in parallel
        let results = future::join_all(shutdown_futures).await;
        
        // Check for failures
        for (i, result) in results.into_iter().enumerate() {
            if let Err(e) = result {
                error!("Failed to shutdown agent: {}", e);
            }
        }
        
        Ok(())
    }
    
    async fn shutdown_agent(&self, agent_id: &str, timeout: Duration) -> Result<()> {
        if let Some(mut agent) = self.agents.write().await.remove(agent_id) {
            // Send graceful shutdown signal
            if let Some(stdin) = agent.child.stdin.take() {
                // Send shutdown command if CLI supports it
                writeln!(stdin, "{{\"command\": \"shutdown\"}}")?;
            }
            
            // Wait for graceful exit
            match tokio::time::timeout(timeout, agent.child.wait()).await {
                Ok(Ok(status)) => {
                    info!("Agent {} exited with status: {}", agent_id, status);
                }
                Ok(Err(e)) => {
                    error!("Error waiting for agent {}: {}", agent_id, e);
                }
                Err(_) => {
                    // Timeout - force kill
                    warn!("Agent {} didn't exit gracefully, forcing kill", agent_id);
                    agent.child.kill().await?;
                }
            }
            
            // Cleanup handles
            agent.stdout_handle.abort();
            agent.stderr_handle.abort();
            agent.health_check_handle.abort();
        }
        
        Ok(())
    }
}
```

## Consequences

### Positive
- **Flexible Roles**: Support for both predefined and custom agent roles
- **Multiple Instances**: Can run multiple agents of the same role concurrently
- **Role Specialization**: Clear agent responsibilities with optimized prompts
- **Dynamic Templates**: Custom roles get automatically generated templates
- **Resource Control**: Per-role limits with defaults for custom roles
- **Health Monitoring**: Automatic detection of crashed agents
- **Graceful Shutdown**: Clean termination with timeout fallback
- **Isolated Workspaces**: Agents can't interfere with each other
- **Clear Identification**: Agent IDs include role and instance information

### Negative
- **Process Overhead**: Each agent is a full process
- **Platform Differences**: Resource limits vary by OS
- **Recovery Complexity**: Restarting failed agents needs careful state management
- **Template Quality**: Custom role templates may be less optimized than predefined ones

### Mitigation
- Process pooling for frequently used configurations
- Platform-specific resource management code
- Simple retry policies with exponential backoff
- Template caching and refinement over time
- Allow users to provide custom templates for their roles

## References
- **ADR-03: Session Orchestration and State Management** - Uses this process management infrastructure for session-level coordination
- ADR-05: CLI Integration and Process Spawning - CLI-specific spawning patterns
- Process supervision patterns and best practices
- Tokio process management documentation
- Linux resource limits (setrlimit)
- Container orchestration patterns

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*