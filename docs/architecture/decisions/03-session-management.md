# ADR-03: Session Orchestration and State Management

## Status
Accepted

## Context
MAOS orchestration happens within sessions - logical groupings of agents working together on a specific objective. Sessions need comprehensive state management and recovery capabilities to handle the complex orchestration scenarios revealed by our POC.

### Core Session Requirements
- Track the overall orchestration objective and progress
- Manage agent lifecycles within the session
- Coordinate dependencies and execution order
- Handle failures and recovery at multiple levels
- Provide visibility into session state and progress
- Clean up resources when complete
- Support interrupted session recovery

### POC-Informed Recovery Requirements
During POC development, we discovered that orchestration sessions could be interrupted due to various factors including model timeouts, system crashes, network issues, and user intervention. Without proper state management and recovery mechanisms, these interruptions resulted in lost work and required complete restarts.

Key requirements evolved to include:
- Sessions persist across MAOS restarts with full state recovery
- Multiple sessions can run concurrently without interference  
- Sessions can be paused and resumed at agent, phase, or session level
- Complete session history for debugging and recovery
- Integration with Claude CLI `--resume` functionality
- Hierarchical recovery strategies for different interruption types

## Decision
We will implement comprehensive session management with state tracking, persistence, and lifecycle control.

### Architectural Layering

This ADR focuses on high-level session orchestration and coordination, while delegating process-level management to ADR-08:

- **ADR-03 handles**: Session planning, agent coordination, dependency tracking, execution strategies, and recovery
- **ADR-08 handles**: Process spawning, resource management, health monitoring, and lifecycle states
- **Relationship**: ADR-03 orchestrates sessions by coordinating with ADR-08's process management infrastructure

### Unified State Model

This ADR uses the unified state model that provides consistent state definitions across all orchestration layers:

```rust
// Session-level orchestration states
pub enum SessionState {
    Created,      // Planning phase
    Running,      // Active execution
    Paused,       // Temporarily suspended
    Completed,    // Successfully finished
    Failed,       // Terminated due to errors
    Cancelled,    // User-initiated termination
}

// Agent execution states (shared across ADR-03, ADR-04, ADR-08)
pub enum AgentExecutionState {
    Pending,      // Waiting to start
    Running,      // Currently executing
    Completed,    // Successfully finished
    Failed,       // Terminated due to error
    Resumable,    // Interrupted but can resume (for recovery)
    Cancelled,    // Explicitly stopped
}

// Phase states for orchestration
pub enum PhaseState {
    Planned,      // Phase defined but not started
    Running,      // Phase actively executing
    Completed,    // Phase successfully finished
    Failed,       // Phase failed
    Skipped,      // Phase skipped due to conditions
}

// Execution strategies
pub enum ExecutionStrategy {
    Parallel,     // All agents run concurrently
    Sequential,   // Agents run one after another
    Adaptive,     // Dynamic scheduling based on dependencies
    Pipeline,     // Agents in stages with handoffs
}

pub struct Session {
    pub id: String,
    pub workspace_hash: String,
    pub objective: String,
    pub strategy: ExecutionStrategy,
    pub state: SessionState,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub agent_count: usize,
    pub completed_agents: usize,
    pub failed_agents: usize,
    pub metadata: HashMap<String, Value>,
}
```

### Session Persistence

Session tracking is managed through the project database schema documented in the [Storage Schema Reference](../references/storage-schema.md#project-database-schema). Key tables include:

- **sessions**: Tracks orchestration sessions with state, strategy, and progress
- **session_agents**: Manages agent assignments, tasks, and dependencies
- **session_events**: Provides audit trail for session lifecycle events

### Session Lifecycle Manager

```rust
pub struct SessionManager {
    db: SqlitePool,
    process_manager: Arc<ProcessManager>,
    logger: Arc<SessionLogger>,
}

impl SessionManager {
    pub async fn create_session(&self, request: CreateSessionRequest) -> Result<Session> {
        // Generate session ID
        let session_id = format!("sess_{}", Uuid::new_v4().simple());
        
        // Create session directory structure (delegates to storage layer)
        let session_dir = self.prepare_session_directory(&session_id)?;
        
        // Create session record
        let session = Session {
            id: session_id.clone(),
            workspace_hash: self.workspace_hash.clone(),
            objective: request.objective,
            strategy: request.strategy,
            state: SessionState::Created,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            agent_count: request.agents.len(),
            completed_agents: 0,
            failed_agents: 0,
            metadata: request.metadata,
        };
        
        // Save to database
        sqlx::query!(
            r#"
            INSERT INTO sessions 
            (id, workspace_hash, objective, strategy, state, agent_count, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            session.id,
            session.workspace_hash,
            session.objective,
            session.strategy.to_string(),
            session.state.to_string(),
            session.agent_count as i32,
            serde_json::to_string(&session.metadata)?
        )
        .execute(&self.db)
        .await?;
        
        // Plan agent tasks
        for agent_spec in request.agents {
            self.register_agent(&session_id, agent_spec).await?;
        }
        
        // Log session creation
        self.logger.log_event("session_created", json!({
            "session_id": session_id,
            "objective": session.objective,
            "strategy": session.strategy,
            "agent_count": session.agent_count,
        }))?;
        
        Ok(session)
    }
    
    pub async fn start_session(&self, session_id: &str) -> Result<()> {
        // Update session state
        sqlx::query!(
            "UPDATE sessions SET state = ?, started_at = ? WHERE id = ?",
            SessionState::Running.to_string(),
            Utc::now(),
            session_id
        )
        .execute(&self.db)
        .await?;
        
        // Get execution strategy
        let strategy = self.get_session_strategy(session_id).await?;
        
        match strategy {
            ExecutionStrategy::Parallel => {
                self.start_all_agents(session_id).await?;
            }
            ExecutionStrategy::Sequential => {
                self.start_next_agent(session_id).await?;
            }
            ExecutionStrategy::Adaptive => {
                self.start_ready_agents(session_id).await?;
            }
            ExecutionStrategy::Pipeline => {
                self.start_pipeline_stage(session_id, 0).await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn pause_session(&self, session_id: &str) -> Result<()> {
        // Update state
        sqlx::query!(
            "UPDATE sessions SET state = ? WHERE id = ?",
            SessionState::Paused.to_string(),
            session_id
        )
        .execute(&self.db)
        .await?;
        
        // Pause all running agents (delegates to ADR-08 ProcessManager)
        let agents = self.get_running_agents(session_id).await?;
        for agent in agents {
            self.process_manager.pause_agent(&agent.agent_id).await?;
        }
        
        self.logger.log_event("session_paused", json!({
            "session_id": session_id,
        }))?;
        
        Ok(())
    }
    
    pub async fn resume_session(&self, session_id: &str) -> Result<()> {
        // Update state
        sqlx::query!(
            "UPDATE sessions SET state = ? WHERE id = ?",
            SessionState::Running.to_string(),
            session_id
        )
        .execute(&self.db)
        .await?;
        
        // Resume paused agents
        let agents = self.get_paused_agents(session_id).await?;
        for agent in agents {
            self.process_manager.resume_agent(&agent.agent_id).await?;
        }
        
        // Start any pending agents
        self.start_ready_agents(session_id).await?;
        
        Ok(())
    }
}
```

### Agent Coordination

```rust
impl SessionManager {
    async fn start_ready_agents(&self, session_id: &str) -> Result<()> {
        // Get all pending agents
        let pending_agents = sqlx::query!(
            r#"
            SELECT agent_id, role, task, dependencies
            FROM session_agents
            WHERE session_id = ? AND state = 'pending'
            "#,
            session_id
        )
        .fetch_all(&self.db)
        .await?;
        
        for agent in pending_agents {
            // Check dependencies
            let deps: Vec<String> = serde_json::from_str(&agent.dependencies)?;
            if self.check_dependencies_met(session_id, &deps).await? {
                self.start_agent(session_id, &agent.agent_id).await?;
            }
        }
        
        Ok(())
    }
    
    async fn check_dependencies_met(&self, session_id: &str, deps: &[String]) -> Result<bool> {
        if deps.is_empty() {
            return Ok(true);
        }
        
        let completed_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM session_agents
            WHERE session_id = ? 
            AND agent_id IN (SELECT value FROM json_each(?))
            AND state = 'completed'
            "#,
            session_id,
            serde_json::to_string(deps)?
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(completed_count == deps.len() as i64)
    }
    
    pub async fn handle_agent_completion(&self, session_id: &str, agent_id: &str, exit_code: i32) -> Result<()> {
        // Update agent state
        let state = if exit_code == 0 { "completed" } else { "failed" };
        sqlx::query!(
            r#"
            UPDATE session_agents 
            SET state = ?, completed_at = ?, exit_code = ?
            WHERE session_id = ? AND agent_id = ?
            "#,
            state,
            Utc::now(),
            exit_code,
            session_id,
            agent_id
        )
        .execute(&self.db)
        .await?;
        
        // Update session counters
        if exit_code == 0 {
            sqlx::query!(
                "UPDATE sessions SET completed_agents = completed_agents + 1 WHERE id = ?",
                session_id
            )
            .execute(&self.db)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE sessions SET failed_agents = failed_agents + 1 WHERE id = ?",
                session_id
            )
            .execute(&self.db)
            .await?;
        }
        
        // Check if session is complete
        self.check_session_completion(session_id).await?;
        
        // Start dependent agents if using adaptive strategy
        let strategy = self.get_session_strategy(session_id).await?;
        if matches!(strategy, ExecutionStrategy::Adaptive) {
            self.start_ready_agents(session_id).await?;
        }
        
        Ok(())
    }
}
```

### Session Monitoring

```rust
pub struct SessionMonitor {
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    db: SqlitePool,
}

impl SessionMonitor {
    pub async fn get_session_status(&self, session_id: &str) -> Result<SessionStatus> {
        let session = sqlx::query!(
            "SELECT * FROM sessions WHERE id = ?",
            session_id
        )
        .fetch_one(&self.db)
        .await?;
        
        let agents = sqlx::query!(
            r#"
            SELECT agent_id, role, state, task,
                   started_at, completed_at, exit_code
            FROM session_agents
            WHERE session_id = ?
            ORDER BY created_at
            "#,
            session_id
        )
        .fetch_all(&self.db)
        .await?;
        
        // Calculate role distribution
        let role_distribution = self.calculate_role_distribution(&agents);
        
        Ok(SessionStatus {
            session: session.into(),
            agents: agents.into_iter().map(Into::into).collect(),
            role_distribution,
            progress: self.calculate_progress(&agents),
            estimated_completion: self.estimate_completion(&session, &agents),
        })
    }
    
    pub async fn stream_session_updates(&self, session_id: &str) -> impl Stream<Item = SessionUpdate> {
        // Watch for changes in session state and agent status
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        let session_id = session_id.to_string();
        let db = self.db.clone();
        
        tokio::spawn(async move {
            let mut last_update = Utc::now();
            loop {
                // Query for updates since last check
                let updates = sqlx::query!(
                    r#"
                    SELECT * FROM session_events
                    WHERE session_id = ? AND timestamp > ?
                    ORDER BY timestamp
                    "#,
                    session_id,
                    last_update
                )
                .fetch_all(&db)
                .await
                .unwrap_or_default();
                
                for update in updates {
                    if let Ok(data) = serde_json::from_str(&update.data) {
                        let _ = tx.send(SessionUpdate {
                            timestamp: update.timestamp,
                            event_type: update.event_type,
                            agent_id: update.agent_id,
                            data,
                        }).await;
                    }
                    last_update = update.timestamp;
                }
                
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
        
        ReceiverStream::new(rx)
    }
    
    fn calculate_role_distribution(&self, agents: &[AgentRecord]) -> HashMap<String, RoleStats> {
        let mut distribution: HashMap<String, RoleStats> = HashMap::new();
        
        for agent in agents {
            // Parse role from agent_id format: agent_{role}_{instance}_{id}
            let parts: Vec<&str> = agent.agent_id.split('_').collect();
            if parts.len() >= 3 && parts[0] == "agent" {
                let role_name = parts[1].to_string();
                let stats = distribution.entry(role_name).or_insert(RoleStats {
                    total: 0,
                    running: 0,
                    completed: 0,
                    failed: 0,
                });
                
                stats.total += 1;
                match agent.state.as_str() {
                    "running" => stats.running += 1,
                    "completed" => stats.completed += 1,
                    "failed" => stats.failed += 1,
                    _ => {}
                }
            }
        }
        
        distribution
    }
}

#[derive(Debug, Clone)]
pub struct RoleStats {
    pub total: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
}
```

### Comprehensive State Management and Recovery

#### State Persistence Strategy
We maintain persistent state for all orchestration sessions including:
- **Current phase information**: Active phase, completed phases, planned phases
- **Agent execution state**: Running agents, completed agents, pending agents  
- **Orchestrator plan state**: Adaptive plan evolution, phase decisions, dependencies
- **Shared context state**: Phase outputs, summaries, shared documents

#### Hierarchical Resume Support
Support resume at multiple levels:
- **Agent-Level Resume**: Individual agents resume using Claude CLI `--resume` flag
- **Phase-Level Resume**: Resume orchestration from the beginning of an interrupted phase
- **Session-Level Resume**: Resume entire orchestration session from last checkpoint

#### State Checkpointing Strategy
Create state checkpoints at key orchestration points:
- **Phase Boundaries**: Before starting each new phase
- **Agent Completion**: After each agent completes successfully
- **Plan Updates**: When orchestrator modifies or extends the plan
- **Error Conditions**: Before attempting error recovery

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrchestrationState {
    session_id: String,
    current_phase: Option<Phase>,
    completed_phases: Vec<Phase>,
    planned_phases: Vec<PlannedPhase>,
    orchestrator_context: OrchestratorContext,
    shared_context: SharedContext,
    last_checkpoint: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Phase {
    phase_id: String,
    phase_name: String,
    state: PhaseState, // Uses unified state model
    agents: Vec<AgentExecution>,
    outputs: Vec<PhaseOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentExecution {
    agent_id: String,
    role: String,
    state: AgentExecutionState, // Uses unified state model
    claude_session_id: Option<String>, // For --resume integration
    start_time: Option<DateTime<Utc>>,
    completion_time: Option<DateTime<Utc>>,
}

impl SessionManager {
    pub async fn recover_sessions(&self) -> Result<()> {
        // Find sessions that were running when MAOS stopped
        let interrupted_sessions = sqlx::query!(
            r#"
            SELECT id FROM sessions 
            WHERE state = 'Running' 
            AND workspace_hash = ?
            "#,
            self.workspace_hash
        )
        .fetch_all(&self.db)
        .await?;
        
        for session in interrupted_sessions {
            info!("Recovering session: {}", session.id);
            
            // Load orchestration state from checkpoint
            let state = self.load_orchestration_state(&session.id).await?;
            
            // Assess recovery scope based on interruption context
            let recovery_plan = self.assess_recovery_scope(&session.id, &state).await?;
            
            match recovery_plan {
                RecoveryPlan::ResumeAgent(agent_id) => {
                    // Resume specific agent with Claude CLI --resume
                    self.resume_agent_with_cli(&agent_id, &state).await?;
                }
                RecoveryPlan::RestartPhase(phase_id) => {
                    // Restart the current phase with all context
                    self.restart_phase_with_context(&session.id, &phase_id, &state).await?;
                }
                RecoveryPlan::ResumeSession => {
                    // Resume session from last checkpoint
                    self.resume_session_from_checkpoint(&session.id, &state).await?;
                }
                RecoveryPlan::MarkFailed(reason) => {
                    // Mark session as failed if recovery isn't possible
                    self.mark_session_failed(&session.id, &reason).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn create_checkpoint(&self, session_id: &str) -> Result<()> {
        let state = self.capture_orchestration_state(session_id).await?;
        
        // Store state in structured format for easy inspection
        let checkpoint_path = format!("~/.maos/projects/{}/sessions/{}/checkpoint.json", 
            self.workspace_hash, session_id);
        
        let checkpoint_data = serde_json::to_string_pretty(&state)?;
        fs::write(checkpoint_path, checkpoint_data).await?;
        
        // Update last checkpoint timestamp
        sqlx::query!(
            "UPDATE sessions SET last_checkpoint = ? WHERE id = ?",
            Utc::now(),
            session_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    async fn assess_recovery_scope(&self, session_id: &str, state: &OrchestrationState) -> Result<RecoveryPlan> {
        // Implement intelligent recovery decisions based on interruption context
        
        // Check if individual agents can be resumed
        for agent in &state.current_phase.as_ref().unwrap_or(&Phase::default()).agents {
            if agent.state == AgentExecutionState::Resumable && agent.claude_session_id.is_some() {
                return Ok(RecoveryPlan::ResumeAgent(agent.agent_id.clone()));
            }
        }
        
        // Check if we should restart the current phase
        if let Some(current_phase) = &state.current_phase {
            if current_phase.state == PhaseState::Running {
                return Ok(RecoveryPlan::RestartPhase(current_phase.phase_id.clone()));
            }
        }
        
        // Default to session-level resume
        Ok(RecoveryPlan::ResumeSession)
    }
}

#[derive(Debug, Clone)]
enum RecoveryPlan {
    ResumeAgent(String),
    RestartPhase(String), 
    ResumeSession,
    MarkFailed(String),
}
```

### Session Cleanup

```rust
pub struct SessionCleaner {
    retention_days: u32,
    archive_completed: bool,
}

impl SessionCleaner {
    pub async fn cleanup_old_sessions(&self) -> Result<()> {
        let cutoff = Utc::now() - Duration::days(self.retention_days as i64);
        
        // Find old completed/failed sessions
        let old_sessions = sqlx::query!(
            r#"
            SELECT id, workspace_hash 
            FROM sessions 
            WHERE state IN ('Completed', 'Failed', 'Cancelled')
            AND completed_at < ?
            "#,
            cutoff
        )
        .fetch_all(&self.db)
        .await?;
        
        for session in old_sessions {
            if self.archive_completed {
                self.archive_session(&session.id).await?;
            }
            
            // Remove session directory
            let session_dir = format!("~/.maos/projects/{}/sessions/{}", 
                session.workspace_hash, session.id);
            fs::remove_dir_all(session_dir).await?;
            
            // Remove from database
            sqlx::query!("DELETE FROM sessions WHERE id = ?", session.id)
                .execute(&self.db)
                .await?;
        }
        
        Ok(())
    }
}
```

## Consequences

### Positive
- **Resilient Orchestration**: Sessions survive various types of interruptions with comprehensive recovery
- **Reduced Waste**: No need to restart completed work after interruptions
- **Better User Experience**: Users can confidently start long-running orchestrations
- **Fault Tolerance**: System gracefully handles expected failure modes with intelligent recovery
- **Multiple Execution Strategies**: Parallel, Sequential, Adaptive, and Pipeline orchestration modes
- **Comprehensive Visibility**: Clear progress tracking with role distribution and phase-aware monitoring
- **Development Efficiency**: Faster iteration during development and testing with resume capability
- **Complete Session History**: Full audit trail for debugging and analysis

### Negative
- **Implementation Complexity**: Additional state management and recovery logic across multiple abstraction levels
- **Storage Requirements**: Persistent state storage for all orchestration sessions and checkpoints
- **Performance Overhead**: Checkpointing adds latency to orchestration execution
- **Recovery Edge Cases**: Complex scenarios where automated recovery isn't possible
- **Database Operations**: Ongoing overhead for state tracking and updates

### Mitigation
- Implement incremental state storage to minimize performance impact
- Provide clear user feedback on recovery options and limitations
- Create comprehensive testing for various interruption scenarios
- Design graceful degradation when automated recovery isn't possible
- Automatic cleanup policies for completed sessions
- Efficient state queries with proper indexing

## References
- ADR-02: Hybrid Storage Strategy - Session data persistence
- ADR-07: Orchestration Guardrails and Coordination Protocols - Session coordination requirements
- ADR-08: Agent Lifecycle and Management - Process-level coordination with session management
- ADR-11: Adaptive Phase-Based Orchestration - Phase-aware session planning
- Claude CLI `--resume` functionality documentation
- POC analysis of session interruption patterns
- Workflow orchestration patterns
- State machine design
- Fault tolerance design patterns

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*