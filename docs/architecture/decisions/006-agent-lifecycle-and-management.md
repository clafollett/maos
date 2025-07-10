# ADR-006: Agent Lifecycle and Management

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

### Agent Lifecycle States

```
┌─────────┐      ┌─────────┐      ┌─────────┐
│ Pending │ ───► │ Running │ ───► │Complete│
└─────────┘      └────┬────┘      └─────────┘
                      │
                      ▼
                 ┌─────────┐
                 │ Failed  │
                 └─────────┘
```

### Agent Role Definitions

```rust
#[derive(Clone, Debug)]
pub struct AgentRole {
    pub name: String,              // e.g., "engineer", "architect", "custom_analyst"
    pub description: String,       // Brief role overview
    pub responsibilities: String,  // Detailed list of responsibilities
    pub is_predefined: bool,       // true for built-in roles, false for custom
    pub instance_suffix: Option<String>, // Optional descriptive suffix
}

// Predefined role constants
pub mod PredefinedRoles {
    pub const ARCHITECT: &str = "architect";
    pub const ENGINEER: &str = "engineer";
    pub const RESEARCHER: &str = "researcher";
    pub const QA: &str = "qa";
    pub const PM: &str = "pm";
    pub const DEVOPS: &str = "devops";
    pub const SECURITY: &str = "security";
    pub const DATA_SCIENTIST: &str = "data_scientist";
    pub const DESIGNER: &str = "designer";
    pub const DOCUMENTER: &str = "documenter";
    pub const REVIEWER: &str = "reviewer";
    pub const ANALYST: &str = "analyst";
    pub const TESTER: &str = "tester";
}

pub struct AgentTemplate {
    pub role_name: String,
    pub capabilities: Vec<String>,
    pub default_timeout: Duration,
    pub max_memory_mb: u32,
    pub required_tools: Vec<String>,
    pub prompt_template: String,
}

lazy_static! {
    static ref PREDEFINED_TEMPLATES: HashMap<String, AgentTemplate> = {
        let mut templates = HashMap::new();
        
        templates.insert(PredefinedRoles::ARCHITECT.to_string(), AgentTemplate {
            role_name: PredefinedRoles::ARCHITECT.to_string(),
            capabilities: vec![
                "system-design".to_string(),
                "technical-specifications".to_string(),
                "architecture-diagrams".to_string(),
            ],
            default_timeout: Duration::from_secs(1800), // 30 minutes
            max_memory_mb: 2048,
            required_tools: vec!["read", "write"],
            prompt_template: include_str!("templates/architect.txt"),
        });
        
        templates.insert(PredefinedRoles::ENGINEER.to_string(), AgentTemplate {
            role_name: PredefinedRoles::ENGINEER.to_string(),
            capabilities: vec![
                "code-implementation".to_string(),
                "testing".to_string(),
                "debugging".to_string(),
            ],
            default_timeout: Duration::from_secs(3600), // 60 minutes
            max_memory_mb: 4096,
            required_tools: vec!["read", "write", "bash"],
            prompt_template: include_str!("templates/engineer.txt"),
        });
        
        // Additional predefined roles
        templates.insert(PredefinedRoles::DEVOPS.to_string(), AgentTemplate {
            role_name: PredefinedRoles::DEVOPS.to_string(),
            capabilities: vec![
                "infrastructure".to_string(),
                "deployment".to_string(),
                "ci-cd".to_string(),
            ],
            default_timeout: Duration::from_secs(2400), // 40 minutes
            max_memory_mb: 3072,
            required_tools: vec!["read", "write", "bash"],
            prompt_template: include_str!("templates/devops.txt"),
        });
        
        templates.insert(PredefinedRoles::SECURITY.to_string(), AgentTemplate {
            role_name: PredefinedRoles::SECURITY.to_string(),
            capabilities: vec![
                "security-analysis".to_string(),
                "vulnerability-assessment".to_string(),
                "compliance-checking".to_string(),
            ],
            default_timeout: Duration::from_secs(2400),
            max_memory_mb: 2048,
            required_tools: vec!["read", "write"],
            prompt_template: include_str!("templates/security.txt"),
        });
        
        // ... other predefined roles
        
        templates
    };
}
```

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

```rust
pub struct TemplateGenerator {
    base_timeout: Duration,
    base_memory_mb: u32,
}

impl TemplateGenerator {
    pub async fn generate_custom_template(&self, role: &AgentRole) -> Result<AgentTemplate> {
        // Generate a template based on role description
        let capabilities = self.infer_capabilities(&role.description);
        let required_tools = self.infer_required_tools(&capabilities);
        
        let template = AgentTemplate {
            role_name: role.name.clone(),
            capabilities,
            default_timeout: self.base_timeout,
            max_memory_mb: self.base_memory_mb,
            required_tools,
            prompt_template: self.generate_prompt_template(role),
        };
        
        Ok(template)
    }
    
    fn generate_prompt_template(&self, role: &AgentRole) -> String {
        format!(r#"
You are a {} agent in the MAOS multi-agent orchestration system.

Role Description: {}

Your responsibilities:
{}

Work within your isolated workspace while collaborating through shared context.
Follow the standard MAOS communication protocols for inter-agent messaging.
"#, 
            role.name,
            role.description,
            role.responsibilities
        )
    }
}
```

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
- Tokio process management documentation
- Linux resource limits (setrlimit)
- Process supervision patterns

---
*Date: 2025-07-09*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollettLaFollett Labs LLC)*