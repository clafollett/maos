# ADR-009: Session Management

## Status
Accepted

## Context
MAOS orchestration happens within sessions - logical groupings of agents working together on a specific objective. Sessions need to:

- Track the overall orchestration objective and progress
- Manage agent lifecycles within the session
- Coordinate dependencies and execution order
- Handle failures and recovery
- Provide visibility into session state
- Clean up resources when complete

Key requirements:
- Sessions persist across MAOS restarts
- Multiple sessions can run concurrently
- Sessions can be paused and resumed
- Clear session history for debugging

## Decision
We will implement comprehensive session management with state tracking, persistence, and lifecycle control.

### Session State Model

```rust
pub enum SessionState {
    Created,      // Initial state, planning phase
    Running,      // Agents are actively working
    Paused,       // Temporarily suspended
    Completed,    // Successfully finished
    Failed,       // Terminated due to errors
    Cancelled,    // User-initiated termination
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

pub enum ExecutionStrategy {
    Parallel,     // All agents run concurrently
    Sequential,   // Agents run one after another
    Adaptive,     // Dynamic scheduling based on dependencies
    Pipeline,     // Agents in stages with handoffs
}
```

### Session Persistence

SQLite schema for session tracking:

```sql
-- In project's SQLite database
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    workspace_hash TEXT NOT NULL,
    objective TEXT NOT NULL,
    strategy TEXT NOT NULL,
    state TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    agent_count INTEGER DEFAULT 0,
    completed_agents INTEGER DEFAULT 0,
    failed_agents INTEGER DEFAULT 0,
    metadata JSON,
    error_message TEXT,
    
    INDEX idx_state (state),
    INDEX idx_workspace (workspace_hash),
    INDEX idx_created (created_at)
);

-- Agent assignments
CREATE TABLE session_agents (
    session_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    role TEXT NOT NULL,
    task TEXT NOT NULL,
    state TEXT NOT NULL,
    dependencies JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    exit_code INTEGER,
    error_message TEXT,
    
    PRIMARY KEY (session_id, agent_id),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Session events for audit trail
CREATE TABLE session_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    event_type TEXT NOT NULL,
    agent_id TEXT,
    data JSON,
    
    FOREIGN KEY (session_id) REFERENCES sessions(id),
    INDEX idx_session_time (session_id, timestamp)
);
```

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
        
        // Create session directory structure
        let session_dir = self.prepare_session_directory(&session_id)?;
        
        // Initialize shared directories
        fs::create_dir_all(session_dir.join("shared/context")).await?;
        fs::create_dir_all(session_dir.join("shared/messages/inbox")).await?;
        fs::create_dir_all(session_dir.join("shared/messages/outbox")).await?;
        fs::create_dir_all(session_dir.join("agents")).await?;
        
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
        
        // Pause all running agents
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

### Session Recovery

```rust
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
            
            // Check agent states
            let agents = self.get_session_agents(&session.id).await?;
            
            let mut has_running = false;
            for agent in agents {
                if agent.state == "running" {
                    // Check if process still exists
                    if self.process_manager.is_agent_alive(&agent.agent_id).await {
                        has_running = true;
                    } else {
                        // Mark as failed
                        self.handle_agent_completion(&session.id, &agent.agent_id, -1).await?;
                    }
                }
            }
            
            if !has_running {
                // Resume session if all agents are stopped
                self.resume_session(&session.id).await?;
            }
        }
        
        Ok(())
    }
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
- **Resilience**: Sessions survive MAOS restarts
- **Flexibility**: Multiple execution strategies
- **Visibility**: Clear progress tracking with role distribution
- **Debugging**: Complete session history
- **Control**: Pause/resume capabilities
- **Role Tracking**: Monitor multiple instances of same role
- **Instance Management**: Clear view of agent distribution

### Negative
- **Complexity**: State management logic
- **Storage**: Session data accumulates
- **Overhead**: Database operations
- **Recovery**: Complex failure scenarios
- **ID Parsing**: Must extract role from agent IDs

### Mitigation
- Implement session templates for common patterns
- Automatic cleanup policies
- Efficient state queries with proper indexing
- Clear recovery procedures and documentation

## References
- Workflow orchestration patterns
- State machine design
- Job scheduling systems
- Database transaction management

---
*Date: 2025-07-09*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollettLaFollett Labs LLC)*