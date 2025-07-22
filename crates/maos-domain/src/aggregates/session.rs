use crate::value_objects::{AgentId, AgentRole, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Domain errors for Session aggregate
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session validation failed: {0}")]
    ValidationError(String),
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: SessionStatus,
        to: SessionStatus,
    },
    #[error("Agent {agent_id} not found in session")]
    AgentNotFound { agent_id: AgentId },
    #[error("Task description cannot be empty")]
    EmptyTaskDescription,
    #[error("Workspace path is required")]
    MissingWorkspace,
}

/// Session metadata containing environment and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub workspace_path: PathBuf,
    pub shared_context_path: PathBuf,
    pub project_root: PathBuf,
    pub environment_vars: HashMap<String, String>,
    pub max_agents: usize,
    pub timeout_minutes: u32,
}

impl SessionMetadata {
    pub fn new(
        workspace_path: PathBuf,
        shared_context_path: PathBuf,
        project_root: PathBuf,
    ) -> Result<Self, SessionError> {
        if workspace_path.as_os_str().is_empty() {
            return Err(SessionError::MissingWorkspace);
        }

        Ok(Self {
            workspace_path,
            shared_context_path,
            project_root,
            environment_vars: HashMap::new(),
            max_agents: 10,      // Default limit
            timeout_minutes: 60, // Default timeout
        })
    }

    pub fn with_env_var(mut self, key: String, value: String) -> Self {
        self.environment_vars.insert(key, value);
        self
    }

    pub fn with_max_agents(mut self, max_agents: usize) -> Self {
        self.max_agents = max_agents;
        self
    }

    pub fn with_timeout(mut self, timeout_minutes: u32) -> Self {
        self.timeout_minutes = timeout_minutes;
        self
    }
}

/// Session aggregate - represents a multi-agent orchestration session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    id: SessionId,
    task_description: String,
    status: SessionStatus,
    metadata: SessionMetadata,
    active_agents: HashMap<AgentId, AgentInstanceInfo>,
    phase_count: u32,
    total_phases: Option<u32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

/// Information about active agent instances in the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstanceInfo {
    pub role: AgentRole,
    pub instance_number: u32,
    pub spawned_at: DateTime<Utc>,
    pub status: SessionAgentStatus,
}

/// Agent status within a session (distinct from the Agent aggregate's status)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionAgentStatus {
    Spawning,
    Active,
    Completed,
    Failed,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Created,
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl Session {
    /// Create a new session with the given task description and metadata
    pub fn new(task_description: String, metadata: SessionMetadata) -> Result<Self, SessionError> {
        if task_description.trim().is_empty() {
            return Err(SessionError::EmptyTaskDescription);
        }

        let now = Utc::now();
        Ok(Self {
            id: SessionId::new(),
            task_description: task_description.trim().to_string(),
            status: SessionStatus::Created,
            metadata,
            active_agents: HashMap::new(),
            phase_count: 0,
            total_phases: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
        })
    }

    // Getters
    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn task_description(&self) -> &str {
        &self.task_description
    }

    pub fn status(&self) -> &SessionStatus {
        &self.status
    }

    pub fn metadata(&self) -> &SessionMetadata {
        &self.metadata
    }

    pub fn active_agents(&self) -> &HashMap<AgentId, AgentInstanceInfo> {
        &self.active_agents
    }

    pub fn phase_count(&self) -> u32 {
        self.phase_count
    }

    pub fn total_phases(&self) -> Option<u32> {
        self.total_phases
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.started_at
    }

    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            (Some(start), None) => Some(Utc::now() - start),
            _ => None,
        }
    }

    // State transitions with business invariants
    pub fn start(&mut self) -> Result<(), SessionError> {
        self.validate_transition(&SessionStatus::InProgress)?;
        self.status = SessionStatus::InProgress;
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), SessionError> {
        if !matches!(self.status, SessionStatus::InProgress) {
            return Err(SessionError::InvalidTransition {
                from: self.status.clone(),
                to: SessionStatus::Paused,
            });
        }
        self.status = SessionStatus::Paused;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), SessionError> {
        if !matches!(self.status, SessionStatus::Paused) {
            return Err(SessionError::InvalidTransition {
                from: self.status.clone(),
                to: SessionStatus::InProgress,
            });
        }
        self.status = SessionStatus::InProgress;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), SessionError> {
        if !matches!(
            self.status,
            SessionStatus::InProgress | SessionStatus::Paused
        ) {
            return Err(SessionError::InvalidTransition {
                from: self.status.clone(),
                to: SessionStatus::Completed,
            });
        }
        self.status = SessionStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn fail(&mut self, _reason: Option<String>) -> Result<(), SessionError> {
        if matches!(self.status, SessionStatus::Completed) {
            return Err(SessionError::InvalidTransition {
                from: self.status.clone(),
                to: SessionStatus::Failed,
            });
        }
        self.status = SessionStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        // TODO: Store failure reason in a proper field
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), SessionError> {
        if matches!(
            self.status,
            SessionStatus::Completed | SessionStatus::Failed
        ) {
            return Err(SessionError::InvalidTransition {
                from: self.status.clone(),
                to: SessionStatus::Cancelled,
            });
        }
        self.status = SessionStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    // Agent management
    pub fn spawn_agent(
        &mut self,
        agent_id: AgentId,
        role: AgentRole,
        instance_number: u32,
    ) -> Result<(), SessionError> {
        if self.active_agents.len() >= self.metadata.max_agents {
            return Err(SessionError::ValidationError(format!(
                "Maximum agents ({}) reached",
                self.metadata.max_agents
            )));
        }

        let agent_info = AgentInstanceInfo {
            role,
            instance_number,
            spawned_at: Utc::now(),
            status: SessionAgentStatus::Spawning,
        };

        self.active_agents.insert(agent_id, agent_info);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_agent_status(
        &mut self,
        agent_id: AgentId,
        status: SessionAgentStatus,
    ) -> Result<(), SessionError> {
        let agent = self
            .active_agents
            .get_mut(&agent_id)
            .ok_or(SessionError::AgentNotFound { agent_id })?;

        agent.status = status;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn terminate_agent(&mut self, agent_id: AgentId) -> Result<(), SessionError> {
        let agent = self
            .active_agents
            .get_mut(&agent_id)
            .ok_or(SessionError::AgentNotFound { agent_id })?;

        agent.status = SessionAgentStatus::Terminated;
        self.updated_at = Utc::now();
        Ok(())
    }

    // Phase management
    pub fn set_total_phases(&mut self, total: u32) {
        self.total_phases = Some(total);
        self.updated_at = Utc::now();
    }

    pub fn advance_phase(&mut self) {
        self.phase_count += 1;
        self.updated_at = Utc::now();
    }

    pub fn progress_percentage(&self) -> Option<f32> {
        self.total_phases
            .map(|total| (self.phase_count as f32 / total as f32 * 100.0).min(100.0))
    }

    // Business logic helpers
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            SessionStatus::InProgress | SessionStatus::Paused
        )
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            SessionStatus::Completed | SessionStatus::Failed | SessionStatus::Cancelled
        )
    }

    pub fn has_active_agents(&self) -> bool {
        self.active_agents.values().any(|info| {
            matches!(
                info.status,
                SessionAgentStatus::Spawning | SessionAgentStatus::Active
            )
        })
    }

    pub fn can_accept_new_agents(&self) -> bool {
        self.active_agents.len() < self.metadata.max_agents && self.is_active()
    }

    // Private helpers
    fn validate_transition(&self, to: &SessionStatus) -> Result<(), SessionError> {
        let valid = matches!(
            (&self.status, to),
            (SessionStatus::Created, SessionStatus::InProgress)
                | (SessionStatus::InProgress, SessionStatus::Paused)
                | (SessionStatus::InProgress, SessionStatus::Completed)
                | (SessionStatus::InProgress, SessionStatus::Failed)
                | (SessionStatus::InProgress, SessionStatus::Cancelled)
                | (SessionStatus::Paused, SessionStatus::InProgress)
                | (SessionStatus::Paused, SessionStatus::Completed)
                | (SessionStatus::Paused, SessionStatus::Failed)
                | (SessionStatus::Paused, SessionStatus::Cancelled)
        );

        if !valid {
            return Err(SessionError::InvalidTransition {
                from: self.status.clone(),
                to: to.clone(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn create_test_metadata() -> SessionMetadata {
        SessionMetadata::new(
            Path::new("/tmp/workspace").to_path_buf(),
            Path::new("/tmp/context").to_path_buf(),
            Path::new("/tmp/project").to_path_buf(),
        )
        .unwrap()
    }

    #[test]
    fn test_session_creation_success() {
        let metadata = create_test_metadata();
        let session = Session::new("Build a web API".to_string(), metadata).unwrap();

        assert_eq!(session.task_description(), "Build a web API");
        assert!(matches!(session.status(), SessionStatus::Created));
        assert_eq!(session.active_agents().len(), 0);
        assert_eq!(session.phase_count(), 0);
        assert!(session.total_phases().is_none());
        assert!(session.started_at().is_none());
        assert!(session.completed_at().is_none());
    }

    #[test]
    fn test_session_creation_empty_task() {
        let metadata = create_test_metadata();
        let result = Session::new("   ".to_string(), metadata);

        assert!(matches!(result, Err(SessionError::EmptyTaskDescription)));
    }

    #[test]
    fn test_session_lifecycle_happy_path() {
        let metadata = create_test_metadata();
        let mut session = Session::new("Test task".to_string(), metadata).unwrap();

        // Start session
        assert!(session.start().is_ok());
        assert!(matches!(session.status(), SessionStatus::InProgress));
        assert!(session.started_at().is_some());
        assert!(session.is_active());
        assert!(!session.is_finished());

        // Complete session
        assert!(session.complete().is_ok());
        assert!(matches!(session.status(), SessionStatus::Completed));
        assert!(session.completed_at().is_some());
        assert!(!session.is_active());
        assert!(session.is_finished());
    }

    #[test]
    fn test_session_pause_resume() {
        let metadata = create_test_metadata();
        let mut session = Session::new("Test task".to_string(), metadata).unwrap();

        // Start and then pause
        session.start().unwrap();
        assert!(session.pause().is_ok());
        assert!(matches!(session.status(), SessionStatus::Paused));
        assert!(session.is_active());

        // Resume
        assert!(session.resume().is_ok());
        assert!(matches!(session.status(), SessionStatus::InProgress));
    }

    #[test]
    fn test_invalid_state_transitions() {
        let metadata = create_test_metadata();
        let mut session = Session::new("Test task".to_string(), metadata).unwrap();

        // Cannot pause before starting
        assert!(matches!(
            session.pause(),
            Err(SessionError::InvalidTransition { .. })
        ));

        // Cannot resume before pausing
        session.start().unwrap();
        assert!(matches!(
            session.resume(),
            Err(SessionError::InvalidTransition { .. })
        ));

        // Cannot complete after already completed
        session.complete().unwrap();
        assert!(matches!(
            session.complete(),
            Err(SessionError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn test_agent_management() {
        let metadata = create_test_metadata();
        let mut session = Session::new("Test task".to_string(), metadata).unwrap();
        session.start().unwrap();

        let agent_id = AgentId::new();

        // Spawn agent
        assert!(
            session
                .spawn_agent(agent_id, AgentRole::BackendEngineer, 1)
                .is_ok()
        );
        assert_eq!(session.active_agents().len(), 1);
        assert!(session.has_active_agents());
        assert!(session.can_accept_new_agents());

        // Update agent status
        assert!(
            session
                .update_agent_status(agent_id, SessionAgentStatus::Active)
                .is_ok()
        );

        // Terminate agent
        assert!(session.terminate_agent(agent_id).is_ok());
        assert!(!session.has_active_agents());
    }

    #[test]
    fn test_agent_not_found() {
        let metadata = create_test_metadata();
        let mut session = Session::new("Test task".to_string(), metadata).unwrap();
        let agent_id = AgentId::new();

        assert!(matches!(
            session.update_agent_status(agent_id, SessionAgentStatus::Active),
            Err(SessionError::AgentNotFound { .. })
        ));
    }

    #[test]
    fn test_max_agents_limit() {
        let metadata = create_test_metadata().with_max_agents(1);
        let mut session = Session::new("Test task".to_string(), metadata).unwrap();
        session.start().unwrap();

        // First agent should succeed
        assert!(
            session
                .spawn_agent(AgentId::new(), AgentRole::BackendEngineer, 1)
                .is_ok()
        );

        // Second agent should fail
        assert!(matches!(
            session.spawn_agent(AgentId::new(), AgentRole::FrontendEngineer, 2),
            Err(SessionError::ValidationError(_))
        ));
    }

    #[test]
    fn test_phase_management() {
        let metadata = create_test_metadata();
        let mut session = Session::new("Test task".to_string(), metadata).unwrap();

        session.set_total_phases(3);
        assert_eq!(session.total_phases(), Some(3));
        assert_eq!(session.progress_percentage(), Some(0.0));

        session.advance_phase();
        assert_eq!(session.phase_count(), 1);
        assert_eq!(session.progress_percentage(), Some(33.333334));

        session.advance_phase();
        session.advance_phase();
        assert_eq!(session.phase_count(), 3);
        assert_eq!(session.progress_percentage(), Some(100.0));
    }

    #[test]
    fn test_session_metadata_creation() {
        let metadata = SessionMetadata::new(
            Path::new("/workspace").to_path_buf(),
            Path::new("/context").to_path_buf(),
            Path::new("/project").to_path_buf(),
        )
        .unwrap()
        .with_env_var("TEST".to_string(), "value".to_string())
        .with_max_agents(5)
        .with_timeout(30);

        assert_eq!(metadata.max_agents, 5);
        assert_eq!(metadata.timeout_minutes, 30);
        assert_eq!(
            metadata.environment_vars.get("TEST"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_session_metadata_empty_workspace() {
        let result = SessionMetadata::new(
            PathBuf::new(), // Empty path
            Path::new("/context").to_path_buf(),
            Path::new("/project").to_path_buf(),
        );

        assert!(matches!(result, Err(SessionError::MissingWorkspace)));
    }

    #[test]
    fn test_session_duration() {
        let metadata = create_test_metadata();
        let mut session = Session::new("Test task".to_string(), metadata).unwrap();

        // No duration when not started
        assert!(session.duration().is_none());

        // Has duration when started
        session.start().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = session.duration().unwrap();
        assert!(duration.num_milliseconds() >= 0);

        // Fixed duration when completed
        session.complete().unwrap();
        let final_duration = session.duration().unwrap();
        assert!(final_duration.num_milliseconds() > 0);
    }
}
