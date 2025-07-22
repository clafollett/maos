use crate::aggregates::{Instance, InstanceStatus};
use crate::value_objects::{AgentId, SessionId};
use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

/// Repository errors for instance operations
#[derive(Debug, Error)]
pub enum InstanceRepositoryError {
    #[error("Instance not found: {instance_id}")]
    NotFound { instance_id: Uuid },
    
    #[error("Instance already exists: {instance_id}")]
    AlreadyExists { instance_id: Uuid },
    
    #[error("Database error: {message}")]
    DatabaseError { message: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Connection error: {message}")]
    ConnectionError { message: String },
    
    #[error("Validation error: {message}")]
    ValidationError { message: String },
}

/// Query criteria for filtering instances
#[derive(Debug, Clone, Default)]
pub struct InstanceQueryCriteria {
    pub agent_id_filter: Option<AgentId>,
    pub session_id_filter: Option<SessionId>,
    pub status_filter: Option<Vec<InstanceStatus>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl InstanceQueryCriteria {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_agent_id(mut self, agent_id: AgentId) -> Self {
        self.agent_id_filter = Some(agent_id);
        self
    }

    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id_filter = Some(session_id);
        self
    }

    pub fn with_statuses(mut self, statuses: Vec<InstanceStatus>) -> Self {
        self.status_filter = Some(statuses);
        self
    }

    pub fn with_status(mut self, status: InstanceStatus) -> Self {
        self.status_filter = Some(vec![status]);
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

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn running_only(self) -> Self {
        self.with_status(InstanceStatus::Running)
    }

    pub fn active_only(self) -> Self {
        self.with_statuses(vec![InstanceStatus::Starting, InstanceStatus::Running])
    }

    pub fn completed_only(self) -> Self {
        self.with_statuses(vec![InstanceStatus::Stopped, InstanceStatus::Failed])
    }
}

/// Instance statistics for monitoring and resource management
#[derive(Debug, Clone)]
pub struct InstanceStatistics {
    pub total_instances: usize,
    pub starting_instances: usize,
    pub running_instances: usize,
    pub stopping_instances: usize,
    pub stopped_instances: usize,
    pub failed_instances: usize,
    pub instances_by_agent: std::collections::HashMap<AgentId, usize>,
    pub instances_by_session: std::collections::HashMap<SessionId, usize>,
    pub average_runtime_minutes: Option<f64>,
}

/// Async repository trait for Instance aggregate
#[async_trait::async_trait]
pub trait InstanceRepository: Send + Sync {
    /// Save a new instance or update an existing one
    async fn save(&self, instance: &Instance) -> Result<(), InstanceRepositoryError>;

    /// Find instance by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Instance>, InstanceRepositoryError>;

    /// Get instance by ID (returns error if not found)
    async fn get_by_id(&self, id: Uuid) -> Result<Instance, InstanceRepositoryError> {
        self.find_by_id(id).await?.ok_or(InstanceRepositoryError::NotFound { instance_id: id })
    }

    /// Delete instance by ID
    async fn delete(&self, id: Uuid) -> Result<(), InstanceRepositoryError>;

    /// Find all instances matching criteria
    async fn find_by_criteria(&self, criteria: &InstanceQueryCriteria) -> Result<Vec<Instance>, InstanceRepositoryError>;

    /// Find instances by agent ID
    async fn find_by_agent_id(&self, agent_id: AgentId) -> Result<Vec<Instance>, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new().with_agent_id(agent_id);
        self.find_by_criteria(&criteria).await
    }

    /// Find instances by session ID
    async fn find_by_session_id(&self, session_id: SessionId) -> Result<Vec<Instance>, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new().with_session_id(session_id);
        self.find_by_criteria(&criteria).await
    }

    /// Find instances by status
    async fn find_by_status(&self, status: InstanceStatus) -> Result<Vec<Instance>, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new().with_status(status);
        self.find_by_criteria(&criteria).await
    }

    /// Find all active instances (Starting or Running)
    async fn find_active_instances(&self) -> Result<Vec<Instance>, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new().active_only();
        self.find_by_criteria(&criteria).await
    }

    /// Find running instances
    async fn find_running_instances(&self) -> Result<Vec<Instance>, InstanceRepositoryError> {
        self.find_by_status(InstanceStatus::Running).await
    }

    /// Find active instances for specific agent
    async fn find_active_by_agent(&self, agent_id: AgentId) -> Result<Vec<Instance>, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new()
            .with_agent_id(agent_id)
            .active_only();
        self.find_by_criteria(&criteria).await
    }

    /// Find active instances for specific session
    async fn find_active_by_session(&self, session_id: SessionId) -> Result<Vec<Instance>, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new()
            .with_session_id(session_id)
            .active_only();
        self.find_by_criteria(&criteria).await
    }

    /// Find instances created within time range
    async fn find_instances_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Instance>, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new()
            .with_created_after(start)
            .with_created_before(end);
        self.find_by_criteria(&criteria).await
    }

    /// Count instances matching criteria
    async fn count_by_criteria(&self, criteria: &InstanceQueryCriteria) -> Result<usize, InstanceRepositoryError>;

    /// Count all instances
    async fn count_all(&self) -> Result<usize, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new();
        self.count_by_criteria(&criteria).await
    }

    /// Count active instances for resource monitoring
    async fn count_active(&self) -> Result<usize, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new().active_only();
        self.count_by_criteria(&criteria).await
    }

    /// Count instances by agent (useful for load balancing)
    async fn count_by_agent(&self, agent_id: AgentId) -> Result<usize, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new().with_agent_id(agent_id);
        self.count_by_criteria(&criteria).await
    }

    /// Count instances by session
    async fn count_by_session(&self, session_id: SessionId) -> Result<usize, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new().with_session_id(session_id);
        self.count_by_criteria(&criteria).await
    }

    /// Get instance statistics
    async fn get_statistics(&self) -> Result<InstanceStatistics, InstanceRepositoryError>;

    /// Check if instance exists
    async fn exists(&self, id: Uuid) -> Result<bool, InstanceRepositoryError> {
        match self.find_by_id(id).await? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Update instance status
    async fn update_status(&self, id: Uuid, status: InstanceStatus) -> Result<(), InstanceRepositoryError> {
        let mut instance = self.get_by_id(id).await?;
        match status {
            InstanceStatus::Starting => {
                return Err(InstanceRepositoryError::ValidationError {
                    message: "Cannot transition to Starting status".to_string(),
                });
            }
            InstanceStatus::Running => instance.start(),
            InstanceStatus::Stopping => instance.stop(),
            InstanceStatus::Stopped => instance.stopped(),
            InstanceStatus::Failed => instance.fail(),
        }
        self.save(&instance).await
    }

    /// Get most recent instances for monitoring
    async fn find_recent(&self, limit: usize) -> Result<Vec<Instance>, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new().with_limit(limit);
        self.find_by_criteria(&criteria).await
    }

    /// Find long-running instances (potential resource leaks)
    async fn find_long_running(&self, threshold_minutes: i64) -> Result<Vec<Instance>, InstanceRepositoryError> {
        let threshold_time = Utc::now() - chrono::Duration::minutes(threshold_minutes);
        let criteria = InstanceQueryCriteria::new()
            .with_status(InstanceStatus::Running)
            .with_created_before(threshold_time);
        self.find_by_criteria(&criteria).await
    }

    /// Find orphaned instances (instances with no active session)
    async fn find_orphaned_instances(&self) -> Result<Vec<Instance>, InstanceRepositoryError>;

    /// Batch operations for efficiency
    async fn save_batch(&self, instances: &[Instance]) -> Result<(), InstanceRepositoryError> {
        for instance in instances {
            self.save(instance).await?;
        }
        Ok(())
    }

    /// Cleanup completed instances older than threshold
    async fn cleanup_old_instances(&self, older_than: DateTime<Utc>) -> Result<usize, InstanceRepositoryError> {
        let criteria = InstanceQueryCriteria::new()
            .completed_only()
            .with_created_before(older_than);
        
        let instances = self.find_by_criteria(&criteria).await?;
        let count = instances.len();
        
        for instance in instances {
            self.delete(instance.id).await?;
        }
        
        Ok(count)
    }

    /// Force stop all instances (emergency use)
    async fn emergency_stop_all(&self) -> Result<usize, InstanceRepositoryError> {
        let active_instances = self.find_active_instances().await?;
        let count = active_instances.len();
        
        for instance in active_instances {
            self.update_status(instance.id, InstanceStatus::Stopped).await?;
        }
        
        Ok(count)
    }

    /// Force stop instances by session (session cleanup)
    async fn stop_instances_by_session(&self, session_id: SessionId) -> Result<usize, InstanceRepositoryError> {
        let active_instances = self.find_active_by_session(session_id).await?;
        let count = active_instances.len();
        
        for instance in active_instances {
            self.update_status(instance.id, InstanceStatus::Stopped).await?;
        }
        
        Ok(count)
    }

    /// Get resource utilization by agent
    async fn get_agent_utilization(&self) -> Result<std::collections::HashMap<AgentId, usize>, InstanceRepositoryError> {
        let active_instances = self.find_active_instances().await?;
        let mut utilization = std::collections::HashMap::new();
        
        for instance in active_instances {
            *utilization.entry(instance.agent_id).or_insert(0) += 1;
        }
        
        Ok(utilization)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_query_criteria_builder() {
        let agent_id = AgentId::new();
        let session_id = SessionId::new();
        let criteria = InstanceQueryCriteria::new()
            .with_agent_id(agent_id)
            .with_session_id(session_id)
            .with_status(InstanceStatus::Running)
            .with_limit(10);

        assert_eq!(criteria.agent_id_filter, Some(agent_id));
        assert_eq!(criteria.session_id_filter, Some(session_id));
        assert_eq!(criteria.status_filter, Some(vec![InstanceStatus::Running]));
        assert_eq!(criteria.limit, Some(10));
    }

    #[test]
    fn test_instance_query_criteria_convenience_methods() {
        let running_criteria = InstanceQueryCriteria::new().running_only();
        assert_eq!(running_criteria.status_filter, Some(vec![InstanceStatus::Running]));

        let active_criteria = InstanceQueryCriteria::new().active_only();
        assert_eq!(active_criteria.status_filter, Some(vec![InstanceStatus::Starting, InstanceStatus::Running]));

        let completed_criteria = InstanceQueryCriteria::new().completed_only();
        assert_eq!(completed_criteria.status_filter, Some(vec![InstanceStatus::Stopped, InstanceStatus::Failed]));
    }

    #[test]
    fn test_instance_query_criteria_time_range() {
        let start = Utc::now();
        let end = start + chrono::Duration::hours(1);
        
        let criteria = InstanceQueryCriteria::new()
            .with_created_after(start)
            .with_created_before(end);

        assert_eq!(criteria.created_after, Some(start));
        assert_eq!(criteria.created_before, Some(end));
    }

    #[test]
    fn test_instance_repository_error_display() {
        let instance_id = Uuid::new_v4();
        let error = InstanceRepositoryError::NotFound { instance_id };
        
        assert!(error.to_string().contains("Instance not found"));
        assert!(error.to_string().contains(&instance_id.to_string()));
    }

    #[test]
    fn test_instance_statistics_structure() {
        let mut instances_by_agent = std::collections::HashMap::new();
        let agent_id = AgentId::new();
        instances_by_agent.insert(agent_id, 3);

        let mut instances_by_session = std::collections::HashMap::new();
        let session_id = SessionId::new();
        instances_by_session.insert(session_id, 2);

        let stats = InstanceStatistics {
            total_instances: 10,
            starting_instances: 1,
            running_instances: 5,
            stopping_instances: 1,
            stopped_instances: 2,
            failed_instances: 1,
            instances_by_agent,
            instances_by_session,
            average_runtime_minutes: Some(45.5),
        };

        assert_eq!(stats.total_instances, 10);
        assert_eq!(stats.running_instances, 5);
        assert_eq!(stats.instances_by_agent.get(&agent_id), Some(&3));
    }
}