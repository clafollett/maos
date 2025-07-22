use crate::aggregates::{Agent, AgentStatus};
use crate::events::{
    domain_event::{EventDispatcher, DomainEvent},
    agent_events::*,
};
use crate::repositories::{
    AgentRepository, AgentRepositoryError, InstanceRepository, InstanceRepositoryError,
    SessionRepository, SessionRepositoryError,
};
use crate::value_objects::{AgentId, AgentRole, SessionId};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during agent scheduling
#[derive(Debug, Error)]
pub enum AgentSchedulerError {
    #[error("Agent repository error: {0}")]
    AgentRepositoryError(#[from] AgentRepositoryError),
    
    #[error("Instance repository error: {0}")]
    InstanceRepositoryError(#[from] InstanceRepositoryError),
    
    #[error("Session repository error: {0}")]
    SessionRepositoryError(#[from] SessionRepositoryError),
    
    #[error("No available agents for role: {role}")]
    NoAvailableAgents { role: String },
    
    #[error("No agents with required capabilities: {capabilities:?}")]
    NoAgentsWithCapabilities { capabilities: Vec<String> },
    
    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: AgentId },
    
    #[error("Agent is not available: {agent_id}, current_status={status:?}")]
    AgentNotAvailable { 
        agent_id: AgentId, 
        status: AgentStatus 
    },
    
    #[error("Resource constraint: {message}")]
    ResourceConstraint { message: String },
    
    #[error("Scheduling conflict: {message}")]
    SchedulingConflict { message: String },
    
    #[error("Event dispatch error: {message}")]
    EventDispatchError { message: String },
    
    #[error("Invalid assignment request: {message}")]
    InvalidAssignmentRequest { message: String },
}

/// Configuration for agent scheduler
#[derive(Debug, Clone)]
pub struct AgentSchedulerConfig {
    pub max_agents_per_session: usize,
    pub max_instances_per_agent: usize,
    pub enable_load_balancing: bool,
    pub enable_capability_matching: bool,
    pub enable_event_dispatch: bool,
    pub scheduling_timeout_seconds: u64,
}

impl Default for AgentSchedulerConfig {
    fn default() -> Self {
        Self {
            max_agents_per_session: 10,
            max_instances_per_agent: 3,
            enable_load_balancing: true,
            enable_capability_matching: true,
            enable_event_dispatch: true,
            scheduling_timeout_seconds: 30,
        }
    }
}

/// Agent assignment request
#[derive(Debug, Clone)]
pub struct AgentAssignmentRequest {
    pub session_id: SessionId,
    pub role: AgentRole,
    pub required_capabilities: Vec<String>,
    pub preferred_capabilities: Vec<String>,
    pub priority: u32, // 1=highest, 10=lowest
    pub timeout_seconds: Option<u64>,
    pub allow_busy_agents: bool,
}

impl AgentAssignmentRequest {
    pub fn new(session_id: SessionId, role: AgentRole) -> Self {
        Self {
            session_id,
            role,
            required_capabilities: Vec::new(),
            preferred_capabilities: Vec::new(),
            priority: 5,
            timeout_seconds: None,
            allow_busy_agents: false,
        }
    }

    pub fn with_required_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.required_capabilities = capabilities;
        self
    }

    pub fn with_preferred_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.preferred_capabilities = capabilities;
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }

    pub fn allow_busy(mut self) -> Self {
        self.allow_busy_agents = true;
        self
    }
}

/// Result of agent assignment
#[derive(Debug, Clone)]
pub struct AgentAssignmentResult {
    pub agent_id: AgentId,
    pub session_id: SessionId,
    pub role: AgentRole,
    pub assignment_score: f64, // 0.0-1.0, higher is better match
    pub capabilities_matched: Vec<String>,
    pub capabilities_missing: Vec<String>,
    pub load_factor: f64, // Current utilization of the agent
}

/// Agent utilization metrics
#[derive(Debug, Clone)]
pub struct AgentUtilization {
    pub agent_id: AgentId,
    pub active_instances: usize,
    pub max_instances: usize,
    pub utilization_percentage: f64,
    pub current_sessions: Vec<SessionId>,
}

/// Agent scheduler service - coordinates agent assignment and resource management
pub struct AgentScheduler {
    agent_repository: Arc<dyn AgentRepository>,
    instance_repository: Arc<dyn InstanceRepository>,
    session_repository: Arc<dyn SessionRepository>,
    event_dispatcher: Arc<EventDispatcher>,
    config: AgentSchedulerConfig,
}

impl AgentScheduler {
    pub fn new(
        agent_repository: Arc<dyn AgentRepository>,
        instance_repository: Arc<dyn InstanceRepository>,
        session_repository: Arc<dyn SessionRepository>,
        event_dispatcher: Arc<EventDispatcher>,
        config: AgentSchedulerConfig,
    ) -> Self {
        Self {
            agent_repository,
            instance_repository,
            session_repository,
            event_dispatcher,
            config,
        }
    }

    /// Find and assign the best available agent for a request
    pub async fn assign_agent(
        &self,
        request: AgentAssignmentRequest,
    ) -> Result<AgentAssignmentResult, AgentSchedulerError> {
        // Validate the request
        self.validate_assignment_request(&request).await?;

        // Find candidate agents
        let candidates = self.find_candidate_agents(&request).await?;
        
        if candidates.is_empty() {
            return Err(AgentSchedulerError::NoAvailableAgents {
                role: request.role.to_string(),
            });
        }

        // Score and rank candidates
        let mut scored_candidates = Vec::new();
        for agent in candidates {
            let score = self.calculate_assignment_score(&agent, &request).await?;
            let utilization = self.get_agent_utilization(agent.id).await?;
            
            scored_candidates.push((agent, score, utilization));
        }

        // Sort by score (highest first)
        scored_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Select the best candidate
        let (best_agent, assignment_score, utilization) = scored_candidates
            .into_iter()
            .next()
            .unwrap();

        // Check capabilities
        let capabilities_matched = self.get_matched_capabilities(&best_agent, &request);
        let capabilities_missing = self.get_missing_capabilities(&best_agent, &request);

        // Mark agent as busy
        self.agent_repository
            .update_status(best_agent.id, AgentStatus::Busy)
            .await?;

        // Dispatch assignment event
        if self.config.enable_event_dispatch {
            let event = AgentStatusChanged::new(
                best_agent.id,
                2, // TODO: Get actual version from agent
                AgentStatus::Available,
                AgentStatus::Busy,
                Some(format!("Assigned to session {}", request.session_id)),
            );
            self.dispatch_event(event).await?;
        }

        Ok(AgentAssignmentResult {
            agent_id: best_agent.id,
            session_id: request.session_id,
            role: request.role,
            assignment_score,
            capabilities_matched,
            capabilities_missing,
            load_factor: utilization.utilization_percentage,
        })
    }

    /// Release an agent from assignment
    pub async fn release_agent(
        &self,
        agent_id: AgentId,
        session_id: SessionId,
    ) -> Result<(), AgentSchedulerError> {
        // Validate agent exists
        let agent = self.agent_repository
            .find_by_id(agent_id)
            .await?
            .ok_or(AgentSchedulerError::AgentNotFound { agent_id })?;

        // Stop all instances for this agent in the session
        let instances = self.instance_repository
            .find_by_agent_id(agent_id)
            .await?;
        
        for instance in instances {
            if instance.session_id == session_id {
                self.instance_repository
                    .update_status(instance.id, crate::aggregates::InstanceStatus::Stopped)
                    .await?;
            }
        }

        // Check if agent has other active instances
        let active_instances = self.instance_repository
            .find_active_by_agent(agent_id)
            .await?;

        // If no other active instances, mark agent as available
        if active_instances.is_empty() {
            self.agent_repository
                .update_status(agent_id, AgentStatus::Available)
                .await?;

            // Dispatch status change event
            if self.config.enable_event_dispatch {
                let event = AgentStatusChanged::new(
                    agent_id,
                    3, // TODO: Get actual version
                    AgentStatus::Busy,
                    AgentStatus::Available,
                    Some(format!("Released from session {}", session_id)),
                );
                self.dispatch_event(event).await?;
            }
        }

        Ok(())
    }

    /// Get agent utilization metrics
    pub async fn get_agent_utilization(
        &self,
        agent_id: AgentId,
    ) -> Result<AgentUtilization, AgentSchedulerError> {
        // Get active instances for agent
        let active_instances = self.instance_repository
            .find_active_by_agent(agent_id)
            .await?;

        let active_count = active_instances.len();
        let max_instances = self.config.max_instances_per_agent;
        let utilization_percentage = (active_count as f64 / max_instances as f64) * 100.0;

        // Get current sessions
        let mut current_sessions = Vec::new();
        for instance in &active_instances {
            if !current_sessions.contains(&instance.session_id) {
                current_sessions.push(instance.session_id);
            }
        }

        Ok(AgentUtilization {
            agent_id,
            active_instances: active_count,
            max_instances,
            utilization_percentage,
            current_sessions,
        })
    }

    /// Get utilization for all agents
    pub async fn get_all_agent_utilization(
        &self,
    ) -> Result<Vec<AgentUtilization>, AgentSchedulerError> {
        let all_agents = self.agent_repository
            .find_by_criteria(&crate::repositories::AgentQueryCriteria::new())
            .await?;

        let mut utilizations = Vec::new();
        for agent in all_agents {
            let utilization = self.get_agent_utilization(agent.id).await?;
            utilizations.push(utilization);
        }

        Ok(utilizations)
    }

    /// Find agents available for role
    pub async fn find_available_agents_for_role(
        &self,
        role: AgentRole,
    ) -> Result<Vec<Agent>, AgentSchedulerError> {
        let agents = self.agent_repository
            .find_available_by_role(role)
            .await?;

        if self.config.enable_load_balancing {
            // Filter out overloaded agents
            let mut filtered_agents = Vec::new();
            for agent in agents {
                let utilization = self.get_agent_utilization(agent.id).await?;
                if utilization.active_instances < self.config.max_instances_per_agent {
                    filtered_agents.push(agent);
                }
            }
            Ok(filtered_agents)
        } else {
            Ok(agents)
        }
    }

    /// Find agents with specific capabilities
    pub async fn find_agents_with_capabilities(
        &self,
        capabilities: Vec<String>,
        require_all: bool,
    ) -> Result<Vec<Agent>, AgentSchedulerError> {
        let agents = if require_all {
            self.agent_repository
                .find_by_capabilities_all(capabilities)
                .await?
        } else {
            self.agent_repository
                .find_by_capabilities_any(capabilities)
                .await?
        };

        Ok(agents)
    }

    /// Get load balancing recommendations
    pub async fn get_load_balancing_recommendations(
        &self,
    ) -> Result<Vec<String>, AgentSchedulerError> {
        let utilizations = self.get_all_agent_utilization().await?;
        let mut recommendations = Vec::new();

        // Find overutilized agents
        let overutilized: Vec<_> = utilizations
            .iter()
            .filter(|u| u.utilization_percentage > 80.0)
            .collect();

        // Find underutilized agents
        let underutilized: Vec<_> = utilizations
            .iter()
            .filter(|u| u.utilization_percentage < 20.0)
            .collect();

        if !overutilized.is_empty() {
            recommendations.push(format!(
                "{} agents are overutilized (>80%): {:?}",
                overutilized.len(),
                overutilized
                    .iter()
                    .map(|u| u.agent_id)
                    .collect::<Vec<_>>()
            ));
        }

        if !underutilized.is_empty() {
            recommendations.push(format!(
                "{} agents are underutilized (<20%): {:?}",
                underutilized.len(),
                underutilized
                    .iter()
                    .map(|u| u.agent_id)
                    .collect::<Vec<_>>()
            ));
        }

        if overutilized.is_empty() && underutilized.is_empty() {
            recommendations.push("Agent utilization is well balanced".to_string());
        }

        Ok(recommendations)
    }

    // Private helper methods

    async fn validate_assignment_request(
        &self,
        request: &AgentAssignmentRequest,
    ) -> Result<(), AgentSchedulerError> {
        // Validate session exists
        let session_exists = self.session_repository
            .exists(request.session_id)
            .await?;
        
        if !session_exists {
            return Err(AgentSchedulerError::InvalidAssignmentRequest {
                message: format!("Session {} does not exist", request.session_id),
            });
        }

        // Validate priority range
        if request.priority < 1 || request.priority > 10 {
            return Err(AgentSchedulerError::InvalidAssignmentRequest {
                message: format!("Priority must be between 1-10, got {}", request.priority),
            });
        }

        Ok(())
    }

    async fn find_candidate_agents(
        &self,
        request: &AgentAssignmentRequest,
    ) -> Result<Vec<Agent>, AgentSchedulerError> {
        let mut criteria = crate::repositories::AgentQueryCriteria::new()
            .with_role(request.role.clone());

        // Filter by availability unless busy agents are allowed
        if !request.allow_busy_agents {
            criteria = criteria.with_status(AgentStatus::Available);
        }

        // Filter by required capabilities if specified
        if !request.required_capabilities.is_empty() && self.config.enable_capability_matching {
            criteria = criteria.with_capabilities_all(request.required_capabilities.clone());
        }

        let candidates = self.agent_repository
            .find_by_criteria(&criteria)
            .await?;

        // Additional load balancing filter
        if self.config.enable_load_balancing {
            let mut filtered_candidates = Vec::new();
            for candidate in candidates {
                let utilization = self.get_agent_utilization(candidate.id).await?;
                if utilization.active_instances < self.config.max_instances_per_agent {
                    filtered_candidates.push(candidate);
                }
            }
            Ok(filtered_candidates)
        } else {
            Ok(candidates)
        }
    }

    async fn calculate_assignment_score(
        &self,
        agent: &Agent,
        request: &AgentAssignmentRequest,
    ) -> Result<f64, AgentSchedulerError> {
        let mut score = 0.0;

        // Role match (base score)
        if agent.role == request.role {
            score += 1.0;
        }

        // Capability matching
        if self.config.enable_capability_matching {
            let matched_required = self.get_matched_capabilities(agent, request).len();
            let total_required = request.required_capabilities.len();
            
            if total_required > 0 {
                let capability_score = matched_required as f64 / total_required as f64;
                score += capability_score * 0.8;
            }

            let matched_preferred = request.preferred_capabilities
                .iter()
                .filter(|cap| agent.capabilities.contains(cap))
                .count();
            let total_preferred = request.preferred_capabilities.len();
            
            if total_preferred > 0 {
                let preferred_score = matched_preferred as f64 / total_preferred as f64;
                score += preferred_score * 0.2;
            }
        }

        // Load balancing factor
        if self.config.enable_load_balancing {
            let utilization = self.get_agent_utilization(agent.id).await?;
            let load_factor = 1.0 - (utilization.utilization_percentage / 100.0);
            score += load_factor * 0.3;
        }

        // Agent status bonus
        match agent.status {
            AgentStatus::Available => score += 0.2,
            AgentStatus::Busy => score -= 0.1,
            AgentStatus::Offline => score -= 1.0,
            AgentStatus::Error => score -= 2.0,
        }

        Ok(score.max(0.0).min(5.0)) // Cap score between 0 and 5
    }

    fn get_matched_capabilities(&self, agent: &Agent, request: &AgentAssignmentRequest) -> Vec<String> {
        request.required_capabilities
            .iter()
            .filter(|cap| agent.capabilities.contains(cap))
            .cloned()
            .collect()
    }

    fn get_missing_capabilities(&self, agent: &Agent, request: &AgentAssignmentRequest) -> Vec<String> {
        request.required_capabilities
            .iter()
            .filter(|cap| !agent.capabilities.contains(cap))
            .cloned()
            .collect()
    }

    async fn dispatch_event<T: DomainEvent>(&self, event: T) -> Result<(), AgentSchedulerError> {
        self.event_dispatcher
            .dispatch(event)
            .await
            .map_err(|e| AgentSchedulerError::EventDispatchError {
                message: e.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::AgentRole;

    #[test]
    fn test_agent_assignment_request_builder() {
        let session_id = SessionId::new();
        let role = AgentRole::from("backend_engineer");
        
        let request = AgentAssignmentRequest::new(session_id, role.clone())
            .with_required_capabilities(vec!["rust".to_string(), "api".to_string()])
            .with_preferred_capabilities(vec!["docker".to_string()])
            .with_priority(1)
            .with_timeout(60)
            .allow_busy();

        assert_eq!(request.session_id, session_id);
        assert_eq!(request.role, role);
        assert_eq!(request.required_capabilities, vec!["rust".to_string(), "api".to_string()]);
        assert_eq!(request.preferred_capabilities, vec!["docker".to_string()]);
        assert_eq!(request.priority, 1);
        assert_eq!(request.timeout_seconds, Some(60));
        assert_eq!(request.allow_busy_agents, true);
    }

    #[test]
    fn test_agent_scheduler_config_default() {
        let config = AgentSchedulerConfig::default();
        
        assert_eq!(config.max_agents_per_session, 10);
        assert_eq!(config.max_instances_per_agent, 3);
        assert_eq!(config.enable_load_balancing, true);
        assert_eq!(config.enable_capability_matching, true);
        assert_eq!(config.enable_event_dispatch, true);
        assert_eq!(config.scheduling_timeout_seconds, 30);
    }

    #[test]
    fn test_agent_utilization_calculation() {
        let agent_id = AgentId::new();
        let session1 = SessionId::new();
        let session2 = SessionId::new();
        
        let utilization = AgentUtilization {
            agent_id,
            active_instances: 2,
            max_instances: 3,
            utilization_percentage: (2.0 / 3.0) * 100.0,
            current_sessions: vec![session1, session2],
        };

        assert_eq!(utilization.active_instances, 2);
        assert_eq!(utilization.max_instances, 3);
        assert!((utilization.utilization_percentage - 66.666).abs() < 0.1);
        assert_eq!(utilization.current_sessions.len(), 2);
    }
}