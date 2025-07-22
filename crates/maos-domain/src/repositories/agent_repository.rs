use crate::aggregates::{Agent, AgentStatus};
use crate::value_objects::{AgentId, AgentRole, SessionId};
use chrono::{DateTime, Utc};
use thiserror::Error;

/// Repository errors for agent operations
#[derive(Debug, Error)]
pub enum AgentRepositoryError {
    #[error("Agent not found: {agent_id}")]
    NotFound { agent_id: AgentId },
    
    #[error("Agent already exists: {agent_id}")]
    AlreadyExists { agent_id: AgentId },
    
    #[error("Database error: {message}")]
    DatabaseError { message: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Connection error: {message}")]
    ConnectionError { message: String },
    
    #[error("Validation error: {message}")]
    ValidationError { message: String },
}

/// Query criteria for filtering agents
#[derive(Debug, Clone, Default)]
pub struct AgentQueryCriteria {
    pub role_filter: Option<Vec<AgentRole>>,
    pub status_filter: Option<Vec<AgentStatus>>,
    pub capability_filter: Option<Vec<String>>, // Agents must have ALL these capabilities
    pub capability_any_filter: Option<Vec<String>>, // Agents must have ANY of these capabilities
    pub name_contains: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl AgentQueryCriteria {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_roles(mut self, roles: Vec<AgentRole>) -> Self {
        self.role_filter = Some(roles);
        self
    }

    pub fn with_role(mut self, role: AgentRole) -> Self {
        self.role_filter = Some(vec![role]);
        self
    }

    pub fn with_statuses(mut self, statuses: Vec<AgentStatus>) -> Self {
        self.status_filter = Some(statuses);
        self
    }

    pub fn with_status(mut self, status: AgentStatus) -> Self {
        self.status_filter = Some(vec![status]);
        self
    }

    pub fn with_capabilities_all(mut self, capabilities: Vec<String>) -> Self {
        self.capability_filter = Some(capabilities);
        self
    }

    pub fn with_capabilities_any(mut self, capabilities: Vec<String>) -> Self {
        self.capability_any_filter = Some(capabilities);
        self
    }

    pub fn with_name_contains(mut self, name: String) -> Self {
        self.name_contains = Some(name);
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

    pub fn available_only(self) -> Self {
        self.with_status(AgentStatus::Available)
    }

    pub fn busy_only(self) -> Self {
        self.with_status(AgentStatus::Busy)
    }
}

/// Agent statistics for monitoring and load balancing
#[derive(Debug, Clone)]
pub struct AgentStatistics {
    pub total_agents: usize,
    pub available_agents: usize,
    pub busy_agents: usize,
    pub offline_agents: usize,
    pub error_agents: usize,
    pub agents_by_role: std::collections::HashMap<String, usize>,
    pub most_common_capabilities: Vec<(String, usize)>,
}

/// Async repository trait for Agent aggregate
#[async_trait::async_trait]
pub trait AgentRepository: Send + Sync {
    /// Save a new agent or update an existing one
    async fn save(&self, agent: &Agent) -> Result<(), AgentRepositoryError>;

    /// Find agent by ID
    async fn find_by_id(&self, id: AgentId) -> Result<Option<Agent>, AgentRepositoryError>;

    /// Get agent by ID (returns error if not found)
    async fn get_by_id(&self, id: AgentId) -> Result<Agent, AgentRepositoryError> {
        self.find_by_id(id).await?.ok_or(AgentRepositoryError::NotFound { agent_id: id })
    }

    /// Delete agent by ID
    async fn delete(&self, id: AgentId) -> Result<(), AgentRepositoryError>;

    /// Find all agents matching criteria
    async fn find_by_criteria(&self, criteria: &AgentQueryCriteria) -> Result<Vec<Agent>, AgentRepositoryError>;

    /// Find agents by role
    async fn find_by_role(&self, role: AgentRole) -> Result<Vec<Agent>, AgentRepositoryError> {
        let criteria = AgentQueryCriteria::new().with_role(role);
        self.find_by_criteria(&criteria).await
    }

    /// Find agents by status
    async fn find_by_status(&self, status: AgentStatus) -> Result<Vec<Agent>, AgentRepositoryError> {
        let criteria = AgentQueryCriteria::new().with_status(status);
        self.find_by_criteria(&criteria).await
    }

    /// Find available agents
    async fn find_available_agents(&self) -> Result<Vec<Agent>, AgentRepositoryError> {
        self.find_by_status(AgentStatus::Available).await
    }

    /// Find agents with specific capability
    async fn find_by_capability(&self, capability: &str) -> Result<Vec<Agent>, AgentRepositoryError> {
        let criteria = AgentQueryCriteria::new()
            .with_capabilities_any(vec![capability.to_string()]);
        self.find_by_criteria(&criteria).await
    }

    /// Find agents with all required capabilities
    async fn find_by_capabilities_all(&self, capabilities: Vec<String>) -> Result<Vec<Agent>, AgentRepositoryError> {
        let criteria = AgentQueryCriteria::new()
            .with_capabilities_all(capabilities);
        self.find_by_criteria(&criteria).await
    }

    /// Find agents with any of the specified capabilities
    async fn find_by_capabilities_any(&self, capabilities: Vec<String>) -> Result<Vec<Agent>, AgentRepositoryError> {
        let criteria = AgentQueryCriteria::new()
            .with_capabilities_any(capabilities);
        self.find_by_criteria(&criteria).await
    }

    /// Find available agents with specific role
    async fn find_available_by_role(&self, role: AgentRole) -> Result<Vec<Agent>, AgentRepositoryError> {
        let criteria = AgentQueryCriteria::new()
            .with_role(role)
            .with_status(AgentStatus::Available);
        self.find_by_criteria(&criteria).await
    }

    /// Find available agents with required capabilities
    async fn find_available_with_capabilities(
        &self,
        capabilities: Vec<String>,
    ) -> Result<Vec<Agent>, AgentRepositoryError> {
        let criteria = AgentQueryCriteria::new()
            .with_status(AgentStatus::Available)
            .with_capabilities_all(capabilities);
        self.find_by_criteria(&criteria).await
    }

    /// Find best agent for role and capabilities (returns the most suitable available agent)
    async fn find_best_available(
        &self,
        role: AgentRole,
        required_capabilities: Vec<String>,
    ) -> Result<Option<Agent>, AgentRepositoryError> {
        // First try exact match
        let criteria = AgentQueryCriteria::new()
            .with_role(role)
            .with_status(AgentStatus::Available)
            .with_capabilities_all(required_capabilities.clone())
            .with_limit(1);
        
        let agents = self.find_by_criteria(&criteria).await?;
        if !agents.is_empty() {
            return Ok(Some(agents.into_iter().next().unwrap()));
        }

        // Then try role match without all capabilities
        let criteria = AgentQueryCriteria::new()
            .with_role(role)
            .with_status(AgentStatus::Available)
            .with_limit(1);
        
        let agents = self.find_by_criteria(&criteria).await?;
        Ok(agents.into_iter().next())
    }

    /// Count agents matching criteria
    async fn count_by_criteria(&self, criteria: &AgentQueryCriteria) -> Result<usize, AgentRepositoryError>;

    /// Count all agents
    async fn count_all(&self) -> Result<usize, AgentRepositoryError> {
        let criteria = AgentQueryCriteria::new();
        self.count_by_criteria(&criteria).await
    }

    /// Get agent statistics
    async fn get_statistics(&self) -> Result<AgentStatistics, AgentRepositoryError>;

    /// Check if agent exists
    async fn exists(&self, id: AgentId) -> Result<bool, AgentRepositoryError> {
        match self.find_by_id(id).await? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Update agent status
    async fn update_status(&self, id: AgentId, status: AgentStatus) -> Result<(), AgentRepositoryError> {
        let mut agent = self.get_by_id(id).await?;
        match status {
            AgentStatus::Available => agent.set_available(),
            AgentStatus::Busy => agent.set_busy(),
            AgentStatus::Offline => agent.set_offline(),
            AgentStatus::Error => agent.set_error(),
        }
        self.save(&agent).await
    }

    /// Get all unique roles in the system
    async fn get_all_roles(&self) -> Result<Vec<AgentRole>, AgentRepositoryError>;

    /// Get all unique capabilities in the system
    async fn get_all_capabilities(&self) -> Result<Vec<String>, AgentRepositoryError>;

    /// Find agents that worked on a specific session (future use with session tracking)
    async fn find_by_session(&self, session_id: SessionId) -> Result<Vec<Agent>, AgentRepositoryError>;

    /// Batch operations for efficiency
    async fn save_batch(&self, agents: &[Agent]) -> Result<(), AgentRepositoryError> {
        for agent in agents {
            self.save(agent).await?;
        }
        Ok(())
    }

    /// Update multiple agent statuses (useful for bulk offline/online operations)
    async fn update_statuses_batch(
        &self,
        agent_ids: &[AgentId],
        status: AgentStatus,
    ) -> Result<(), AgentRepositoryError> {
        for &agent_id in agent_ids {
            self.update_status(agent_id, status.clone()).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_query_criteria_builder() {
        let role = AgentRole::from("backend_engineer");
        let criteria = AgentQueryCriteria::new()
            .with_role(role.clone())
            .with_status(AgentStatus::Available)
            .with_capabilities_all(vec!["rust".to_string(), "api".to_string()])
            .with_limit(5);

        assert_eq!(criteria.role_filter, Some(vec![role]));
        assert_eq!(criteria.status_filter, Some(vec![AgentStatus::Available]));
        assert_eq!(criteria.capability_filter, Some(vec!["rust".to_string(), "api".to_string()]));
        assert_eq!(criteria.limit, Some(5));
    }

    #[test]
    fn test_agent_query_criteria_capabilities_any() {
        let criteria = AgentQueryCriteria::new()
            .with_capabilities_any(vec!["rust".to_string(), "python".to_string()]);

        assert_eq!(criteria.capability_any_filter, Some(vec!["rust".to_string(), "python".to_string()]));
        assert_eq!(criteria.capability_filter, None);
    }

    #[test]
    fn test_agent_query_criteria_convenience_methods() {
        let available_criteria = AgentQueryCriteria::new().available_only();
        assert_eq!(available_criteria.status_filter, Some(vec![AgentStatus::Available]));

        let busy_criteria = AgentQueryCriteria::new().busy_only();
        assert_eq!(busy_criteria.status_filter, Some(vec![AgentStatus::Busy]));
    }

    #[test]
    fn test_agent_query_criteria_time_range() {
        let start = Utc::now();
        let end = start + chrono::Duration::hours(1);
        
        let criteria = AgentQueryCriteria::new()
            .with_created_after(start)
            .with_created_before(end);

        assert_eq!(criteria.created_after, Some(start));
        assert_eq!(criteria.created_before, Some(end));
    }

    #[test]
    fn test_agent_repository_error_display() {
        let agent_id = AgentId::new();
        let error = AgentRepositoryError::NotFound { agent_id };
        
        assert!(error.to_string().contains("Agent not found"));
        assert!(error.to_string().contains(&agent_id.to_string()));
    }

    #[test]
    fn test_agent_statistics_structure() {
        let mut agents_by_role = std::collections::HashMap::new();
        agents_by_role.insert("backend_engineer".to_string(), 5);
        agents_by_role.insert("frontend_engineer".to_string(), 3);

        let stats = AgentStatistics {
            total_agents: 10,
            available_agents: 6,
            busy_agents: 3,
            offline_agents: 1,
            error_agents: 0,
            agents_by_role,
            most_common_capabilities: vec![
                ("rust".to_string(), 5),
                ("javascript".to_string(), 3),
            ],
        };

        assert_eq!(stats.total_agents, 10);
        assert_eq!(stats.available_agents, 6);
        assert_eq!(stats.agents_by_role.get("backend_engineer"), Some(&5));
    }
}