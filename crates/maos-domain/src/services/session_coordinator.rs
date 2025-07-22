use crate::aggregates::{Session, SessionError, SessionMetadata, SessionStatus};
use crate::events::{
    domain_event::{EventDispatcher, DomainEvent},
    session_events::*,
};
use crate::repositories::{SessionRepository, SessionRepositoryError};
use crate::value_objects::{AgentId, SessionId};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during session coordination
#[derive(Debug, Error)]
pub enum SessionCoordinatorError {
    #[error("Session error: {0}")]
    SessionError(#[from] SessionError),
    
    #[error("Repository error: {0}")]
    RepositoryError(#[from] SessionRepositoryError),
    
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: SessionId },
    
    #[error("Invalid session state for operation: current={current:?}, required={required:?}")]
    InvalidSessionState { 
        current: SessionStatus, 
        required: Vec<SessionStatus> 
    },
    
    #[error("Session already exists: {session_id}")]
    SessionAlreadyExists { session_id: SessionId },
    
    #[error("Resource constraint violated: {message}")]
    ResourceConstraint { message: String },
    
    #[error("Orchestration error: {message}")]
    OrchestrationError { message: String },
    
    #[error("Event dispatch error: {message}")]
    EventDispatchError { message: String },
}

/// Configuration for session coordinator
#[derive(Debug, Clone)]
pub struct SessionCoordinatorConfig {
    pub max_concurrent_sessions: usize,
    pub default_session_timeout_minutes: u32,
    pub default_max_agents_per_session: usize,
    pub enable_event_dispatch: bool,
    pub cleanup_completed_sessions_after_hours: u32,
}

impl Default for SessionCoordinatorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_sessions: 50,
            default_session_timeout_minutes: 60,
            default_max_agents_per_session: 10,
            enable_event_dispatch: true,
            cleanup_completed_sessions_after_hours: 24,
        }
    }
}

/// Result of session creation operation
#[derive(Debug, Clone)]
pub struct SessionCreationResult {
    pub session: Session,
    pub session_id: SessionId,
    pub created_at: DateTime<Utc>,
}

/// Result of session operation with metrics
#[derive(Debug, Clone)]
pub struct SessionOperationResult {
    pub session_id: SessionId,
    pub old_status: SessionStatus,
    pub new_status: SessionStatus,
    pub operation_duration_ms: u64,
    pub events_dispatched: usize,
}

/// Session coordinator service - orchestrates session lifecycle
/// 
/// This is the primary service for managing session operations,
/// coordinating between aggregates and ensuring business rules.
pub struct SessionCoordinator {
    session_repository: Arc<dyn SessionRepository>,
    event_dispatcher: Arc<EventDispatcher>,
    config: SessionCoordinatorConfig,
}

impl SessionCoordinator {
    pub fn new(
        session_repository: Arc<dyn SessionRepository>,
        event_dispatcher: Arc<EventDispatcher>,
        config: SessionCoordinatorConfig,
    ) -> Self {
        Self {
            session_repository,
            event_dispatcher,
            config,
        }
    }

    /// Create a new session with validation and event dispatch
    pub async fn create_session(
        &self,
        task_description: String,
        metadata: SessionMetadata,
    ) -> Result<SessionCreationResult, SessionCoordinatorError> {
        let start_time = std::time::Instant::now();

        // Validate resource constraints
        self.validate_resource_constraints().await?;

        // Create session aggregate
        let session = Session::new(task_description.clone(), metadata.clone())?;
        let session_id = session.id();
        let created_at = session.created_at();

        // Save to repository
        self.session_repository.save(&session).await?;

        // Dispatch events
        let mut events_dispatched = 0;
        if self.config.enable_event_dispatch {
            let event = SessionCreated::new(
                session_id,
                task_description,
                metadata.workspace_path.to_string_lossy().to_string(),
                metadata.max_agents,
                metadata.timeout_minutes,
            );
            
            self.dispatch_event(event).await?;
            events_dispatched += 1;
        }

        Ok(SessionCreationResult {
            session,
            session_id,
            created_at,
        })
    }

    /// Start a session with validation and coordination
    pub async fn start_session(&self, session_id: SessionId) -> Result<SessionOperationResult, SessionCoordinatorError> {
        let start_time = std::time::Instant::now();
        
        let mut session = self.get_session(session_id).await?;
        let old_status = session.status().clone();

        // Validate session can be started
        if !matches!(session.status(), SessionStatus::Created) {
            return Err(SessionCoordinatorError::InvalidSessionState {
                current: old_status,
                required: vec![SessionStatus::Created],
            });
        }

        // Start the session
        session.start()?;
        let new_status = session.status().clone();

        // Save updated session
        self.session_repository.save(&session).await?;

        // Dispatch events
        let mut events_dispatched = 0;
        if self.config.enable_event_dispatch {
            let event = SessionStarted::new(session_id, session.updated_at().timestamp() as u64);
            self.dispatch_event(event).await?;
            events_dispatched += 1;
        }

        let operation_duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(SessionOperationResult {
            session_id,
            old_status,
            new_status,
            operation_duration_ms,
            events_dispatched,
        })
    }

    /// Pause a session
    pub async fn pause_session(
        &self, 
        session_id: SessionId,
        reason: Option<String>,
    ) -> Result<SessionOperationResult, SessionCoordinatorError> {
        let start_time = std::time::Instant::now();
        
        let mut session = self.get_session(session_id).await?;
        let old_status = session.status().clone();

        // Validate session can be paused
        if !matches!(session.status(), SessionStatus::InProgress) {
            return Err(SessionCoordinatorError::InvalidSessionState {
                current: old_status,
                required: vec![SessionStatus::InProgress],
            });
        }

        // Pause the session
        session.pause()?;
        let new_status = session.status().clone();

        // Save updated session
        self.session_repository.save(&session).await?;

        // Dispatch events
        let mut events_dispatched = 0;
        if self.config.enable_event_dispatch {
            let event = SessionPaused::new(
                session_id, 
                session.updated_at().timestamp() as u64,
                reason,
            );
            self.dispatch_event(event).await?;
            events_dispatched += 1;
        }

        let operation_duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(SessionOperationResult {
            session_id,
            old_status,
            new_status,
            operation_duration_ms,
            events_dispatched,
        })
    }

    /// Resume a paused session
    pub async fn resume_session(&self, session_id: SessionId) -> Result<SessionOperationResult, SessionCoordinatorError> {
        let start_time = std::time::Instant::now();
        
        let mut session = self.get_session(session_id).await?;
        let old_status = session.status().clone();

        // Validate session can be resumed
        if !matches!(session.status(), SessionStatus::Paused) {
            return Err(SessionCoordinatorError::InvalidSessionState {
                current: old_status,
                required: vec![SessionStatus::Paused],
            });
        }

        // Resume the session
        session.resume()?;
        let new_status = session.status().clone();

        // Save updated session
        self.session_repository.save(&session).await?;

        // Dispatch events
        let mut events_dispatched = 0;
        if self.config.enable_event_dispatch {
            let event = SessionResumed::new(session_id, session.updated_at().timestamp() as u64);
            self.dispatch_event(event).await?;
            events_dispatched += 1;
        }

        let operation_duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(SessionOperationResult {
            session_id,
            old_status,
            new_status,
            operation_duration_ms,
            events_dispatched,
        })
    }

    /// Complete a session with metrics calculation
    pub async fn complete_session(&self, session_id: SessionId) -> Result<SessionOperationResult, SessionCoordinatorError> {
        let start_time = std::time::Instant::now();
        
        let mut session = self.get_session(session_id).await?;
        let old_status = session.status().clone();

        // Validate session can be completed
        if !matches!(session.status(), SessionStatus::InProgress | SessionStatus::Paused) {
            return Err(SessionCoordinatorError::InvalidSessionState {
                current: old_status,
                required: vec![SessionStatus::InProgress, SessionStatus::Paused],
            });
        }

        // Calculate metrics before completion
        let duration_seconds = session.duration()
            .map(|d| d.num_seconds())
            .unwrap_or(0);
        let agents_used = session.active_agents().len();
        let total_phases = session.phase_count();

        // Complete the session
        session.complete()?;
        let new_status = session.status().clone();

        // Save updated session
        self.session_repository.save(&session).await?;

        // Dispatch events
        let mut events_dispatched = 0;
        if self.config.enable_event_dispatch {
            let event = SessionCompleted::new(
                session_id,
                session.updated_at().timestamp() as u64,
                total_phases,
                duration_seconds,
                agents_used,
            );
            self.dispatch_event(event).await?;
            events_dispatched += 1;
        }

        let operation_duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(SessionOperationResult {
            session_id,
            old_status,
            new_status,
            operation_duration_ms,
            events_dispatched,
        })
    }

    /// Fail a session with error information
    pub async fn fail_session(
        &self, 
        session_id: SessionId,
        error_message: String,
    ) -> Result<SessionOperationResult, SessionCoordinatorError> {
        let start_time = std::time::Instant::now();
        
        let mut session = self.get_session(session_id).await?;
        let old_status = session.status().clone();

        // Calculate metrics before failure
        let agents_used = session.active_agents().len();
        let phase_count = session.phase_count();

        // Fail the session
        session.fail(Some(error_message.clone()))?;
        let new_status = session.status().clone();

        // Save updated session
        self.session_repository.save(&session).await?;

        // Dispatch events
        let mut events_dispatched = 0;
        if self.config.enable_event_dispatch {
            let event = SessionFailed::new(
                session_id,
                session.updated_at().timestamp() as u64,
                error_message,
                phase_count,
                agents_used,
            );
            self.dispatch_event(event).await?;
            events_dispatched += 1;
        }

        let operation_duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(SessionOperationResult {
            session_id,
            old_status,
            new_status,
            operation_duration_ms,
            events_dispatched,
        })
    }

    /// Cancel a session
    pub async fn cancel_session(
        &self, 
        session_id: SessionId,
        reason: Option<String>,
    ) -> Result<SessionOperationResult, SessionCoordinatorError> {
        let start_time = std::time::Instant::now();
        
        let mut session = self.get_session(session_id).await?;
        let old_status = session.status().clone();

        // Validate session can be cancelled
        if matches!(session.status(), SessionStatus::Completed | SessionStatus::Failed) {
            return Err(SessionCoordinatorError::InvalidSessionState {
                current: old_status,
                required: vec![
                    SessionStatus::Created,
                    SessionStatus::InProgress,
                    SessionStatus::Paused,
                ],
            });
        }

        // Cancel the session
        session.cancel()?;
        let new_status = session.status().clone();

        // Save updated session
        self.session_repository.save(&session).await?;

        // Dispatch events
        let mut events_dispatched = 0;
        if self.config.enable_event_dispatch {
            let event = SessionCancelled::new(
                session_id,
                session.updated_at().timestamp() as u64,
                reason,
            );
            self.dispatch_event(event).await?;
            events_dispatched += 1;
        }

        let operation_duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(SessionOperationResult {
            session_id,
            old_status,
            new_status,
            operation_duration_ms,
            events_dispatched,
        })
    }

    /// Add agent to session with validation
    pub async fn add_agent_to_session(
        &self,
        session_id: SessionId,
        agent_id: AgentId,
        role_name: String,
        instance_number: u32,
    ) -> Result<(), SessionCoordinatorError> {
        let mut session = self.get_session(session_id).await?;

        // Validate session can accept new agents
        if !session.can_accept_new_agents() {
            return Err(SessionCoordinatorError::ResourceConstraint {
                message: format!(
                    "Session {} cannot accept new agents: status={:?}, current_agents={}, max_agents={}",
                    session_id,
                    session.status(),
                    session.active_agents().len(),
                    session.metadata().max_agents
                ),
            });
        }

        // Add agent to session
        session.spawn_agent(agent_id, role_name.clone(), instance_number)?;

        // Save updated session
        self.session_repository.save(&session).await?;

        // Dispatch events
        if self.config.enable_event_dispatch {
            let event = SessionAgentSpawned::new(
                session_id,
                session.updated_at().timestamp() as u64,
                agent_id,
                role_name,
                instance_number,
            );
            self.dispatch_event(event).await?;
        }

        Ok(())
    }

    /// Update session phase with event dispatch
    pub async fn advance_session_phase(&self, session_id: SessionId) -> Result<(), SessionCoordinatorError> {
        let mut session = self.get_session(session_id).await?;
        let previous_phase = session.phase_count();
        
        // Advance phase
        session.advance_phase();
        let new_phase = session.phase_count();
        let total_phases = session.total_phases();

        // Save updated session
        self.session_repository.save(&session).await?;

        // Dispatch events
        if self.config.enable_event_dispatch {
            let event = SessionPhaseAdvanced::new(
                session_id,
                session.updated_at().timestamp() as u64,
                previous_phase,
                new_phase,
                total_phases,
            );
            self.dispatch_event(event).await?;
        }

        Ok(())
    }

    /// Get session with proper error handling
    pub async fn get_session(&self, session_id: SessionId) -> Result<Session, SessionCoordinatorError> {
        self.session_repository
            .find_by_id(session_id)
            .await?
            .ok_or(SessionCoordinatorError::SessionNotFound { session_id })
    }

    /// Check if session exists
    pub async fn session_exists(&self, session_id: SessionId) -> Result<bool, SessionCoordinatorError> {
        Ok(self.session_repository.exists(session_id).await?)
    }

    /// Get active sessions count for monitoring
    pub async fn get_active_sessions_count(&self) -> Result<usize, SessionCoordinatorError> {
        let active_sessions = self.session_repository.find_active_sessions().await?;
        Ok(active_sessions.len())
    }

    /// Cleanup completed sessions older than configured threshold
    pub async fn cleanup_old_sessions(&self) -> Result<usize, SessionCoordinatorError> {
        let threshold = Utc::now() - chrono::Duration::hours(
            self.config.cleanup_completed_sessions_after_hours as i64
        );
        
        let criteria = crate::repositories::SessionQueryCriteria::new()
            .with_status(vec![SessionStatus::Completed, SessionStatus::Failed, SessionStatus::Cancelled])
            .with_created_before(threshold);

        let sessions_to_cleanup = self.session_repository.find_by_criteria(&criteria).await?;
        let count = sessions_to_cleanup.len();

        for session in sessions_to_cleanup {
            self.session_repository.delete(session.id()).await?;
        }

        Ok(count)
    }

    // Private helper methods
    
    async fn validate_resource_constraints(&self) -> Result<(), SessionCoordinatorError> {
        let active_count = self.get_active_sessions_count().await?;
        
        if active_count >= self.config.max_concurrent_sessions {
            return Err(SessionCoordinatorError::ResourceConstraint {
                message: format!(
                    "Maximum concurrent sessions ({}) reached. Current: {}",
                    self.config.max_concurrent_sessions,
                    active_count
                ),
            });
        }
        
        Ok(())
    }

    async fn dispatch_event<T: DomainEvent>(&self, event: T) -> Result<(), SessionCoordinatorError> {
        self.event_dispatcher
            .dispatch(event)
            .await
            .map_err(|e| SessionCoordinatorError::EventDispatchError {
                message: e.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregates::SessionMetadata;
    use std::path::PathBuf;
    use tokio;

    // Mock implementations for testing
    struct MockSessionRepository {
        sessions: std::sync::RwLock<std::collections::HashMap<SessionId, Session>>,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                sessions: std::sync::RwLock::new(std::collections::HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn save(&self, session: &Session) -> Result<(), SessionRepositoryError> {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session.id(), session.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: SessionId) -> Result<Option<Session>, SessionRepositoryError> {
            let sessions = self.sessions.read().unwrap();
            Ok(sessions.get(&id).cloned())
        }

        async fn delete(&self, id: SessionId) -> Result<(), SessionRepositoryError> {
            let mut sessions = self.sessions.write().unwrap();
            sessions.remove(&id);
            Ok(())
        }

        // Minimal implementation for other required methods
        async fn find_by_criteria(&self, _criteria: &crate::repositories::SessionQueryCriteria) -> Result<Vec<Session>, SessionRepositoryError> { Ok(Vec::new()) }
        async fn find_sessions_with_agent(&self, _agent_id: AgentId) -> Result<Vec<Session>, SessionRepositoryError> { Ok(Vec::new()) }
        async fn count_by_criteria(&self, _criteria: &crate::repositories::SessionQueryCriteria) -> Result<usize, SessionRepositoryError> { Ok(0) }
        async fn get_statistics(&self) -> Result<crate::repositories::SessionStatistics, SessionRepositoryError> {
            Ok(crate::repositories::SessionStatistics {
                total_sessions: 0,
                active_sessions: 0,
                completed_sessions: 0,
                failed_sessions: 0,
                cancelled_sessions: 0,
                average_duration_minutes: None,
                average_agents_per_session: None,
            })
        }
    }

    fn create_test_metadata() -> SessionMetadata {
        SessionMetadata::new(
            PathBuf::from("/workspace"),
            PathBuf::from("/context"),
            PathBuf::from("/project"),
        ).unwrap()
    }

    fn create_test_coordinator() -> SessionCoordinator {
        let repository = Arc::new(MockSessionRepository::new());
        let event_dispatcher = Arc::new(EventDispatcher::new());
        let config = SessionCoordinatorConfig::default();
        
        SessionCoordinator::new(repository, event_dispatcher, config)
    }

    #[tokio::test]
    async fn test_create_session_success() {
        let coordinator = create_test_coordinator();
        let metadata = create_test_metadata();
        
        let result = coordinator
            .create_session("Test task".to_string(), metadata)
            .await;
        
        assert!(result.is_ok());
        let creation_result = result.unwrap();
        assert_eq!(creation_result.session.task_description(), "Test task");
    }

    #[tokio::test]
    async fn test_session_lifecycle_happy_path() {
        let coordinator = create_test_coordinator();
        let metadata = create_test_metadata();
        
        // Create session
        let creation_result = coordinator
            .create_session("Test task".to_string(), metadata)
            .await
            .unwrap();
        
        let session_id = creation_result.session_id;
        
        // Start session
        let start_result = coordinator.start_session(session_id).await.unwrap();
        assert_eq!(start_result.old_status, SessionStatus::Created);
        assert_eq!(start_result.new_status, SessionStatus::InProgress);
        
        // Complete session
        let complete_result = coordinator.complete_session(session_id).await.unwrap();
        assert_eq!(complete_result.old_status, SessionStatus::InProgress);
        assert_eq!(complete_result.new_status, SessionStatus::Completed);
    }

    #[tokio::test]
    async fn test_pause_resume_session() {
        let coordinator = create_test_coordinator();
        let metadata = create_test_metadata();
        
        // Create and start session
        let creation_result = coordinator
            .create_session("Test task".to_string(), metadata)
            .await
            .unwrap();
        
        let session_id = creation_result.session_id;
        coordinator.start_session(session_id).await.unwrap();
        
        // Pause session
        let pause_result = coordinator
            .pause_session(session_id, Some("Test pause".to_string()))
            .await
            .unwrap();
        assert_eq!(pause_result.old_status, SessionStatus::InProgress);
        assert_eq!(pause_result.new_status, SessionStatus::Paused);
        
        // Resume session
        let resume_result = coordinator.resume_session(session_id).await.unwrap();
        assert_eq!(resume_result.old_status, SessionStatus::Paused);
        assert_eq!(resume_result.new_status, SessionStatus::InProgress);
    }

    #[tokio::test]
    async fn test_fail_session() {
        let coordinator = create_test_coordinator();
        let metadata = create_test_metadata();
        
        // Create and start session
        let creation_result = coordinator
            .create_session("Test task".to_string(), metadata)
            .await
            .unwrap();
        
        let session_id = creation_result.session_id;
        coordinator.start_session(session_id).await.unwrap();
        
        // Fail session
        let fail_result = coordinator
            .fail_session(session_id, "Test error".to_string())
            .await
            .unwrap();
        assert_eq!(fail_result.old_status, SessionStatus::InProgress);
        assert_eq!(fail_result.new_status, SessionStatus::Failed);
    }

    #[tokio::test]
    async fn test_invalid_state_transitions() {
        let coordinator = create_test_coordinator();
        let metadata = create_test_metadata();
        
        // Create session
        let creation_result = coordinator
            .create_session("Test task".to_string(), metadata)
            .await
            .unwrap();
        
        let session_id = creation_result.session_id;
        
        // Try to pause before starting (should fail)
        let pause_result = coordinator
            .pause_session(session_id, None)
            .await;
        
        assert!(pause_result.is_err());
        assert!(matches!(pause_result.unwrap_err(), SessionCoordinatorError::InvalidSessionState { .. }));
    }

    #[tokio::test]
    async fn test_session_not_found() {
        let coordinator = create_test_coordinator();
        let non_existent_id = SessionId::new();
        
        let result = coordinator.start_session(non_existent_id).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SessionCoordinatorError::SessionNotFound { .. }));
    }
}