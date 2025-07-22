use crate::aggregates::{Session, SessionStatus};
use crate::value_objects::{AgentId, SessionId};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use thiserror::Error;

/// Repository errors for session operations
#[derive(Debug, Error)]
pub enum SessionRepositoryError {
    #[error("Session not found: {session_id}")]
    NotFound { session_id: SessionId },
    
    #[error("Session already exists: {session_id}")]
    AlreadyExists { session_id: SessionId },
    
    #[error("Database error: {message}")]
    DatabaseError { message: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Connection error: {message}")]
    ConnectionError { message: String },
    
    #[error("Validation error: {message}")]
    ValidationError { message: String },
}

/// Query criteria for filtering sessions
#[derive(Debug, Clone, Default)]
pub struct SessionQueryCriteria {
    pub status_filter: Option<Vec<SessionStatus>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub has_active_agents: Option<bool>,
    pub workspace_path_contains: Option<String>,
    pub task_description_contains: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl SessionQueryCriteria {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status(mut self, statuses: Vec<SessionStatus>) -> Self {
        self.status_filter = Some(statuses);
        self
    }

    pub fn with_created_after(mut self, after: DateTime<Utc>) -> Self {
        self.created_after = Some(after);
        self
    }

    pub fn with_created_before(mut self, before: DateTime<Utc>) -> Self {
        self.created_before = Some(before);
        self
    }

    pub fn with_active_agents(mut self, has_active: bool) -> Self {
        self.has_active_agents = Some(has_active);
        self
    }

    pub fn with_workspace_filter(mut self, workspace_contains: String) -> Self {
        self.workspace_path_contains = Some(workspace_contains);
        self
    }

    pub fn with_task_filter(mut self, task_contains: String) -> Self {
        self.task_description_contains = Some(task_contains);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Session statistics for monitoring and reporting
#[derive(Debug, Clone)]
pub struct SessionStatistics {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub completed_sessions: usize,
    pub failed_sessions: usize,
    pub cancelled_sessions: usize,
    pub average_duration_minutes: Option<f64>,
    pub average_agents_per_session: Option<f64>,
}

/// Async repository trait for Session aggregate
#[async_trait::async_trait]
pub trait SessionRepository: Send + Sync {
    /// Save a new session or update an existing one
    async fn save(&self, session: &Session) -> Result<(), SessionRepositoryError>;

    /// Find session by ID
    async fn find_by_id(&self, id: SessionId) -> Result<Option<Session>, SessionRepositoryError>;

    /// Get session by ID (returns error if not found)
    async fn get_by_id(&self, id: SessionId) -> Result<Session, SessionRepositoryError> {
        self.find_by_id(id).await?.ok_or(SessionRepositoryError::NotFound { session_id: id })
    }

    /// Delete session by ID
    async fn delete(&self, id: SessionId) -> Result<(), SessionRepositoryError>;

    /// Find all sessions matching criteria
    async fn find_by_criteria(&self, criteria: &SessionQueryCriteria) -> Result<Vec<Session>, SessionRepositoryError>;

    /// Find sessions by status
    async fn find_by_status(&self, status: SessionStatus) -> Result<Vec<Session>, SessionRepositoryError> {
        let criteria = SessionQueryCriteria::new().with_status(vec![status]);
        self.find_by_criteria(&criteria).await
    }

    /// Find all active sessions (InProgress or Paused)
    async fn find_active_sessions(&self) -> Result<Vec<Session>, SessionRepositoryError> {
        let criteria = SessionQueryCriteria::new()
            .with_status(vec![SessionStatus::InProgress, SessionStatus::Paused]);
        self.find_by_criteria(&criteria).await
    }

    /// Find sessions with specific agent
    async fn find_sessions_with_agent(&self, agent_id: AgentId) -> Result<Vec<Session>, SessionRepositoryError>;

    /// Find sessions created within time range
    async fn find_sessions_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Session>, SessionRepositoryError> {
        let criteria = SessionQueryCriteria::new()
            .with_created_after(start)
            .with_created_before(end);
        self.find_by_criteria(&criteria).await
    }

    /// Count sessions matching criteria
    async fn count_by_criteria(&self, criteria: &SessionQueryCriteria) -> Result<usize, SessionRepositoryError>;

    /// Count all sessions
    async fn count_all(&self) -> Result<usize, SessionRepositoryError> {
        let criteria = SessionQueryCriteria::new();
        self.count_by_criteria(&criteria).await
    }

    /// Get session statistics
    async fn get_statistics(&self) -> Result<SessionStatistics, SessionRepositoryError>;

    /// Check if session exists
    async fn exists(&self, id: SessionId) -> Result<bool, SessionRepositoryError> {
        match self.find_by_id(id).await? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Find sessions by workspace path
    async fn find_by_workspace_path(&self, workspace_path: &str) -> Result<Vec<Session>, SessionRepositoryError> {
        let criteria = SessionQueryCriteria::new()
            .with_workspace_filter(workspace_path.to_string());
        self.find_by_criteria(&criteria).await
    }

    /// Get most recent sessions (up to limit)
    async fn find_recent(&self, limit: usize) -> Result<Vec<Session>, SessionRepositoryError> {
        let criteria = SessionQueryCriteria::new().with_limit(limit);
        self.find_by_criteria(&criteria).await
    }

    /// Update session status
    async fn update_status(&self, id: SessionId, status: SessionStatus) -> Result<(), SessionRepositoryError> {
        let mut session = self.get_by_id(id).await?;
        match status {
            SessionStatus::InProgress => session.start(),
            SessionStatus::Paused => session.pause(),
            SessionStatus::Completed => session.complete(),
            SessionStatus::Failed => session.fail(None),
            SessionStatus::Cancelled => session.cancel(),
            SessionStatus::Created => {
                return Err(SessionRepositoryError::ValidationError {
                    message: "Cannot transition to Created status".to_string(),
                });
            }
        }.map_err(|e| SessionRepositoryError::ValidationError { 
            message: e.to_string() 
        })?;
        
        self.save(&session).await
    }

    /// Batch operations for efficiency
    async fn save_batch(&self, sessions: &[Session]) -> Result<(), SessionRepositoryError> {
        for session in sessions {
            self.save(session).await?;
        }
        Ok(())
    }

    /// Delete sessions matching criteria (use with caution!)
    async fn delete_by_criteria(&self, criteria: &SessionQueryCriteria) -> Result<usize, SessionRepositoryError> {
        let sessions = self.find_by_criteria(criteria).await?;
        let count = sessions.len();
        
        for session in sessions {
            self.delete(session.id()).await?;
        }
        
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_query_criteria_builder() {
        let criteria = SessionQueryCriteria::new()
            .with_status(vec![SessionStatus::InProgress])
            .with_active_agents(true)
            .with_limit(10)
            .with_offset(0);

        assert_eq!(criteria.status_filter, Some(vec![SessionStatus::InProgress]));
        assert_eq!(criteria.has_active_agents, Some(true));
        assert_eq!(criteria.limit, Some(10));
        assert_eq!(criteria.offset, Some(0));
    }

    #[test]
    fn test_session_query_criteria_time_range() {
        let start = Utc::now();
        let end = start + chrono::Duration::hours(1);
        
        let criteria = SessionQueryCriteria::new()
            .with_created_after(start)
            .with_created_before(end);

        assert_eq!(criteria.created_after, Some(start));
        assert_eq!(criteria.created_before, Some(end));
    }

    #[test]
    fn test_session_query_criteria_filters() {
        let criteria = SessionQueryCriteria::new()
            .with_workspace_filter("/workspace".to_string())
            .with_task_filter("API".to_string());

        assert_eq!(criteria.workspace_path_contains, Some("/workspace".to_string()));
        assert_eq!(criteria.task_description_contains, Some("API".to_string()));
    }

    #[test]
    fn test_session_repository_error_display() {
        let session_id = SessionId::new();
        let error = SessionRepositoryError::NotFound { session_id };
        
        assert!(error.to_string().contains("Session not found"));
        assert!(error.to_string().contains(&session_id.to_string()));
    }

    #[test]
    fn test_session_statistics_structure() {
        let stats = SessionStatistics {
            total_sessions: 100,
            active_sessions: 10,
            completed_sessions: 80,
            failed_sessions: 8,
            cancelled_sessions: 2,
            average_duration_minutes: Some(45.5),
            average_agents_per_session: Some(2.3),
        };

        assert_eq!(stats.total_sessions, 100);
        assert_eq!(stats.active_sessions, 10);
        assert_eq!(stats.completed_sessions, 80);
        assert_eq!(stats.failed_sessions, 8);
        assert_eq!(stats.cancelled_sessions, 2);
    }
}