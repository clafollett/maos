use crate::aggregates::{Session, SessionStatus, SessionAgentStatus, AgentStatus};
use crate::repositories::{
    AgentRepository, AgentRepositoryError, InstanceRepository, InstanceRepositoryError,
    SessionRepository, SessionRepositoryError,
};
use crate::value_objects::{AgentId, SessionId};
use chrono::{DateTime, Utc, Duration};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;

/// Validation errors for orchestration rules
#[derive(Debug, Error)]
pub enum OrchestrationValidationError {
    #[error("Repository error: {source}")]
    RepositoryError { source: String },
    
    #[error("Session consistency violation: {message}")]
    SessionConsistencyViolation { message: String },
    
    #[error("Agent assignment violation: {message}")]
    AgentAssignmentViolation { message: String },
    
    #[error("Resource constraint violation: {message}")]
    ResourceConstraintViolation { message: String },
    
    #[error("Temporal constraint violation: {message}")]
    TemporalConstraintViolation { message: String },
    
    #[error("State transition violation: {message}")]
    StateTransitionViolation { message: String },
    
    #[error("Business rule violation: {message}")]
    BusinessRuleViolation { message: String },
    
    #[error("Data integrity violation: {message}")]
    DataIntegrityViolation { message: String },
}

impl From<AgentRepositoryError> for OrchestrationValidationError {
    fn from(err: AgentRepositoryError) -> Self {
        OrchestrationValidationError::RepositoryError {
            source: err.to_string(),
        }
    }
}

impl From<SessionRepositoryError> for OrchestrationValidationError {
    fn from(err: SessionRepositoryError) -> Self {
        OrchestrationValidationError::RepositoryError {
            source: err.to_string(),
        }
    }
}

impl From<InstanceRepositoryError> for OrchestrationValidationError {
    fn from(err: InstanceRepositoryError) -> Self {
        OrchestrationValidationError::RepositoryError {
            source: err.to_string(),
        }
    }
}

/// Validation rule severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,    // Must be fixed immediately
    Warning,  // Should be fixed but not blocking
    Info,     // Informational, no action required
}

/// Individual validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub rule_name: String,
    pub severity: ValidationSeverity,
    pub message: String,
    pub entity_id: Option<String>, // Session ID, Agent ID, etc.
    pub suggested_action: Option<String>,
}

impl ValidationResult {
    pub fn error(rule_name: String, message: String) -> Self {
        Self {
            rule_name,
            severity: ValidationSeverity::Error,
            message,
            entity_id: None,
            suggested_action: None,
        }
    }

    pub fn warning(rule_name: String, message: String) -> Self {
        Self {
            rule_name,
            severity: ValidationSeverity::Warning,
            message,
            entity_id: None,
            suggested_action: None,
        }
    }

    pub fn info(rule_name: String, message: String) -> Self {
        Self {
            rule_name,
            severity: ValidationSeverity::Info,
            message,
            entity_id: None,
            suggested_action: None,
        }
    }

    pub fn with_entity_id(mut self, entity_id: String) -> Self {
        self.entity_id = Some(entity_id);
        self
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggested_action = Some(suggestion);
        self
    }
}

/// Comprehensive validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub timestamp: DateTime<Utc>,
    pub errors: Vec<ValidationResult>,
    pub warnings: Vec<ValidationResult>,
    pub info: Vec<ValidationResult>,
    pub total_rules_checked: usize,
    pub validation_duration_ms: u64,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            total_rules_checked: 0,
            validation_duration_ms: 0,
        }
    }

    pub fn add_result(&mut self, result: ValidationResult) {
        match result.severity {
            ValidationSeverity::Error => self.errors.push(result),
            ValidationSeverity::Warning => self.warnings.push(result),
            ValidationSeverity::Info => self.info.push(result),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }
}

/// Configuration for orchestration validator
#[derive(Debug, Clone)]
pub struct OrchestrationValidatorConfig {
    pub max_session_duration_hours: u32,
    pub max_agents_per_session: usize,
    pub max_instances_per_agent: usize,
    pub session_timeout_warning_threshold_minutes: u32,
    pub check_resource_constraints: bool,
    pub check_temporal_constraints: bool,
    pub check_state_consistency: bool,
    pub check_business_rules: bool,
}

impl Default for OrchestrationValidatorConfig {
    fn default() -> Self {
        Self {
            max_session_duration_hours: 24,
            max_agents_per_session: 10,
            max_instances_per_agent: 3,
            session_timeout_warning_threshold_minutes: 50, // Warn at 50 min if 60 min timeout
            check_resource_constraints: true,
            check_temporal_constraints: true,
            check_state_consistency: true,
            check_business_rules: true,
        }
    }
}

/// Orchestration validator service - ensures consistency and business rules
pub struct OrchestrationValidator {
    agent_repository: Arc<dyn AgentRepository>,
    session_repository: Arc<dyn SessionRepository>,
    instance_repository: Arc<dyn InstanceRepository>,
    config: OrchestrationValidatorConfig,
}

impl OrchestrationValidator {
    pub fn new(
        agent_repository: Arc<dyn AgentRepository>,
        session_repository: Arc<dyn SessionRepository>,
        instance_repository: Arc<dyn InstanceRepository>,
        config: OrchestrationValidatorConfig,
    ) -> Self {
        Self {
            agent_repository,
            session_repository,
            instance_repository,
            config,
        }
    }

    /// Validate entire orchestration system state
    pub async fn validate_system(&self) -> Result<ValidationReport, OrchestrationValidationError> {
        let start_time = std::time::Instant::now();
        let mut report = ValidationReport::new();

        // Run all validation checks
        if self.config.check_state_consistency {
            self.validate_state_consistency(&mut report).await?;
        }

        if self.config.check_resource_constraints {
            self.validate_resource_constraints(&mut report).await?;
        }

        if self.config.check_temporal_constraints {
            self.validate_temporal_constraints(&mut report).await?;
        }

        if self.config.check_business_rules {
            self.validate_business_rules(&mut report).await?;
        }

        // Validate agent-session consistency
        self.validate_agent_session_consistency(&mut report).await?;

        // Validate instance consistency
        self.validate_instance_consistency(&mut report).await?;

        // Check for orphaned entities
        self.validate_orphaned_entities(&mut report).await?;

        report.validation_duration_ms = start_time.elapsed().as_millis() as u64;
        report.total_rules_checked = 15; // Update this as we add more rules

        Ok(report)
    }

    /// Validate a specific session
    pub async fn validate_session(&self, session_id: SessionId) -> Result<ValidationReport, OrchestrationValidationError> {
        let start_time = std::time::Instant::now();
        let mut report = ValidationReport::new();

        let session = self.session_repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| OrchestrationValidationError::DataIntegrityViolation {
                message: format!("Session {} not found", session_id),
            })?;

        // Validate session state
        self.validate_session_state(&session, &mut report).await?;

        // Validate session agents
        self.validate_session_agents(&session, &mut report).await?;

        // Validate session timing
        self.validate_session_timing(&session, &mut report).await?;

        // Validate session resources
        self.validate_session_resources(&session, &mut report).await?;

        report.validation_duration_ms = start_time.elapsed().as_millis() as u64;
        report.total_rules_checked = 5;

        Ok(report)
    }

    /// Validate agent assignment before it happens
    pub async fn validate_agent_assignment(
        &self,
        session_id: SessionId,
        agent_id: AgentId,
    ) -> Result<ValidationReport, OrchestrationValidationError> {
        let mut report = ValidationReport::new();

        // Check if session exists and is in valid state
        let session = self.session_repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| OrchestrationValidationError::DataIntegrityViolation {
                message: format!("Session {} not found", session_id),
            })?;

        if !matches!(session.status(), SessionStatus::Created | SessionStatus::InProgress | SessionStatus::Paused) {
            report.add_result(
                ValidationResult::error(
                    "invalid_session_state".to_string(),
                    format!("Cannot assign agents to session in {:?} state", session.status()),
                )
                .with_entity_id(session_id.to_string())
                .with_suggestion("Only assign agents to active sessions".to_string()),
            );
        }

        // Check if agent exists and is available
        let agent = self.agent_repository
            .find_by_id(agent_id)
            .await?
            .ok_or_else(|| OrchestrationValidationError::DataIntegrityViolation {
                message: format!("Agent {} not found", agent_id),
            })?;

        if !matches!(agent.status, AgentStatus::Available | AgentStatus::Busy) {
            report.add_result(
                ValidationResult::error(
                    "agent_not_available".to_string(),
                    format!("Agent {} is in {:?} state", agent_id, agent.status),
                )
                .with_entity_id(agent_id.to_string())
                .with_suggestion("Use available or busy agents only".to_string()),
            );
        }

        // Check resource constraints
        if session.active_agents().len() >= session.metadata().max_agents {
            report.add_result(
                ValidationResult::error(
                    "max_agents_exceeded".to_string(),
                    format!(
                        "Session {} already has maximum agents ({})",
                        session_id,
                        session.metadata().max_agents
                    ),
                )
                .with_entity_id(session_id.to_string())
                .with_suggestion("Increase session agent limit or wait for agents to complete".to_string()),
            );
        }

        // Check agent utilization
        let active_instances = self.instance_repository
            .find_active_by_agent(agent_id)
            .await?;

        if active_instances.len() >= self.config.max_instances_per_agent {
            report.add_result(
                ValidationResult::warning(
                    "agent_overutilized".to_string(),
                    format!(
                        "Agent {} already has {} active instances (limit: {})",
                        agent_id,
                        active_instances.len(),
                        self.config.max_instances_per_agent
                    ),
                )
                .with_entity_id(agent_id.to_string())
                .with_suggestion("Consider load balancing or using different agent".to_string()),
            );
        }

        Ok(report)
    }

    // Private validation methods

    async fn validate_state_consistency(&self, report: &mut ValidationReport) -> Result<(), OrchestrationValidationError> {
        // Check for sessions with inconsistent agent states
        let active_sessions = self.session_repository.find_active_sessions().await?;
        
        for session in active_sessions {
            for (agent_id, agent_info) in session.active_agents() {
                // Verify agent still exists
                if let Some(agent) = self.agent_repository.find_by_id(*agent_id).await? {
                    // Check status consistency
                    if matches!(agent_info.status, SessionAgentStatus::Active | SessionAgentStatus::Spawning) {
                        if matches!(agent.status, AgentStatus::Offline | AgentStatus::Error) {
                            report.add_result(
                                ValidationResult::error(
                                    "agent_status_inconsistency".to_string(),
                                    format!(
                                        "Session {} shows agent {} as {:?} but agent is {:?}",
                                        session.id(),
                                        agent_id,
                                        agent_info.status,
                                        agent.status
                                    ),
                                )
                                .with_entity_id(session.id().to_string())
                                .with_suggestion("Sync agent status or mark session agent as failed".to_string()),
                            );
                        }
                    }
                } else {
                    report.add_result(
                        ValidationResult::error(
                            "missing_agent".to_string(),
                            format!("Session {} references non-existent agent {}", session.id(), agent_id),
                        )
                        .with_entity_id(session.id().to_string())
                        .with_suggestion("Remove orphaned agent reference from session".to_string()),
                    );
                }
            }
        }

        Ok(())
    }

    async fn validate_resource_constraints(&self, report: &mut ValidationReport) -> Result<(), OrchestrationValidationError> {
        // Check for resource limit violations
        let sessions = self.session_repository
            .find_by_criteria(&crate::repositories::SessionQueryCriteria::new())
            .await?;

        for session in sessions {
            // Check agent count limits
            if session.active_agents().len() > session.metadata().max_agents {
                report.add_result(
                    ValidationResult::error(
                        "session_agent_limit_exceeded".to_string(),
                        format!(
                            "Session {} has {} agents (limit: {})",
                            session.id(),
                            session.active_agents().len(),
                            session.metadata().max_agents
                        ),
                    )
                    .with_entity_id(session.id().to_string())
                    .with_suggestion("Terminate excess agents or increase session limit".to_string()),
                );
            }
        }

        // Check global agent utilization
        let all_agents = self.agent_repository
            .find_by_criteria(&crate::repositories::AgentQueryCriteria::new())
            .await?;

        for agent in all_agents {
            let active_instances = self.instance_repository.find_active_by_agent(agent.id).await?;
            if active_instances.len() > self.config.max_instances_per_agent {
                report.add_result(
                    ValidationResult::warning(
                        "agent_instance_limit_exceeded".to_string(),
                        format!(
                            "Agent {} has {} active instances (limit: {})",
                            agent.id,
                            active_instances.len(),
                            self.config.max_instances_per_agent
                        ),
                    )
                    .with_entity_id(agent.id.to_string())
                    .with_suggestion("Terminate some instances or increase agent limit".to_string()),
                );
            }
        }

        Ok(())
    }

    async fn validate_temporal_constraints(&self, report: &mut ValidationReport) -> Result<(), OrchestrationValidationError> {
        let now = Utc::now();
        let active_sessions = self.session_repository.find_active_sessions().await?;

        for session in active_sessions {
            // Check for long-running sessions
            if let Some(started_at) = session.started_at() {
                let duration = now - started_at;
                let max_duration = Duration::hours(self.config.max_session_duration_hours as i64);
                
                if duration > max_duration {
                    report.add_result(
                        ValidationResult::error(
                            "session_duration_exceeded".to_string(),
                            format!(
                                "Session {} has been running for {} hours (limit: {} hours)",
                                session.id(),
                                duration.num_hours(),
                                self.config.max_session_duration_hours
                            ),
                        )
                        .with_entity_id(session.id().to_string())
                        .with_suggestion("Complete or cancel long-running session".to_string()),
                    );
                }
            }

            // Check session timeout warnings
            let timeout_duration = Duration::minutes(session.metadata().timeout_minutes as i64);
            if let Some(started_at) = session.started_at() {
                let running_duration = now - started_at;
                let warning_threshold = Duration::minutes(self.config.session_timeout_warning_threshold_minutes as i64);
                
                if running_duration > warning_threshold && running_duration < timeout_duration {
                    report.add_result(
                        ValidationResult::warning(
                            "session_timeout_approaching".to_string(),
                            format!(
                                "Session {} approaching timeout in {} minutes",
                                session.id(),
                                (timeout_duration - running_duration).num_minutes()
                            ),
                        )
                        .with_entity_id(session.id().to_string())
                        .with_suggestion("Complete session soon or extend timeout".to_string()),
                    );
                }
            }
        }

        Ok(())
    }

    async fn validate_business_rules(&self, report: &mut ValidationReport) -> Result<(), OrchestrationValidationError> {
        let sessions = self.session_repository
            .find_by_criteria(&crate::repositories::SessionQueryCriteria::new())
            .await?;

        for session in sessions {
            // Rule: Sessions should have meaningful task descriptions
            if session.task_description().trim().is_empty() || session.task_description().len() < 5 {
                report.add_result(
                    ValidationResult::warning(
                        "inadequate_task_description".to_string(),
                        format!(
                            "Session {} has inadequate task description: '{}'",
                            session.id(),
                            session.task_description()
                        ),
                    )
                    .with_entity_id(session.id().to_string())
                    .with_suggestion("Provide more descriptive task description".to_string()),
                );
            }

            // Rule: Active sessions should have active agents
            if matches!(session.status(), SessionStatus::InProgress) && !session.has_active_agents() {
                report.add_result(
                    ValidationResult::warning(
                        "active_session_no_agents".to_string(),
                        format!("Session {} is active but has no active agents", session.id()),
                    )
                    .with_entity_id(session.id().to_string())
                    .with_suggestion("Assign agents to session or pause/complete session".to_string()),
                );
            }

            // Rule: Completed sessions should have some progress
            if matches!(session.status(), SessionStatus::Completed) && session.phase_count() == 0 {
                report.add_result(
                    ValidationResult::info(
                        "completed_session_no_progress".to_string(),
                        format!("Session {} completed without any phase progress", session.id()),
                    )
                    .with_entity_id(session.id().to_string()),
                );
            }
        }

        Ok(())
    }

    async fn validate_session_state(&self, session: &Session, report: &mut ValidationReport) -> Result<(), OrchestrationValidationError> {
        // Check state transitions make sense
        match session.status() {
            SessionStatus::InProgress => {
                if session.started_at().is_none() {
                    report.add_result(
                        ValidationResult::error(
                            "inconsistent_session_timestamps".to_string(),
                            "Session is InProgress but has no started_at timestamp".to_string(),
                        )
                        .with_entity_id(session.id().to_string()),
                    );
                }
            }
            SessionStatus::Completed | SessionStatus::Failed | SessionStatus::Cancelled => {
                if session.completed_at().is_none() {
                    report.add_result(
                        ValidationResult::error(
                            "inconsistent_session_timestamps".to_string(),
                            "Session is finished but has no completed_at timestamp".to_string(),
                        )
                        .with_entity_id(session.id().to_string()),
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn validate_session_agents(&self, session: &Session, report: &mut ValidationReport) -> Result<(), OrchestrationValidationError> {
        for (agent_id, agent_info) in session.active_agents() {
            // Verify agent exists
            if self.agent_repository.find_by_id(*agent_id).await?.is_none() {
                report.add_result(
                    ValidationResult::error(
                        "session_references_missing_agent".to_string(),
                        format!("Session references non-existent agent {}", agent_id),
                    )
                    .with_entity_id(session.id().to_string()),
                );
            }

            // Check agent role consistency
            if agent_info.role_name.is_empty() {
                report.add_result(
                    ValidationResult::warning(
                        "agent_missing_role".to_string(),
                        format!("Agent {} in session has no role assigned", agent_id),
                    )
                    .with_entity_id(session.id().to_string()),
                );
            }
        }

        Ok(())
    }

    async fn validate_session_timing(&self, session: &Session, report: &mut ValidationReport) -> Result<(), OrchestrationValidationError> {
        // Check for timing inconsistencies
        if let (Some(started_at), Some(completed_at)) = (session.started_at(), session.completed_at()) {
            if completed_at < started_at {
                report.add_result(
                    ValidationResult::error(
                        "invalid_session_timestamps".to_string(),
                        "Session completed_at is before started_at".to_string(),
                    )
                    .with_entity_id(session.id().to_string()),
                );
            }
        }

        Ok(())
    }

    async fn validate_session_resources(&self, session: &Session, report: &mut ValidationReport) -> Result<(), OrchestrationValidationError> {
        // Already covered in validate_resource_constraints, but session-specific checks here
        if session.metadata().max_agents == 0 {
            report.add_result(
                ValidationResult::warning(
                    "session_zero_agent_limit".to_string(),
                    "Session has zero agent limit, no agents can be assigned".to_string(),
                )
                .with_entity_id(session.id().to_string()),
            );
        }

        Ok(())
    }

    async fn validate_agent_session_consistency(&self, report: &mut ValidationReport) -> Result<(), OrchestrationValidationError> {
        // This checks for consistency between agent status and their session assignments
        let busy_agents = self.agent_repository.find_by_status(AgentStatus::Busy).await?;
        let active_sessions = self.session_repository.find_active_sessions().await?;

        // Create a map of which sessions each agent is supposed to be in
        let mut session_agent_map: HashMap<AgentId, Vec<SessionId>> = HashMap::new();
        for session in &active_sessions {
            for agent_id in session.active_agents().keys() {
                session_agent_map.entry(*agent_id).or_default().push(session.id());
            }
        }

        // Check if busy agents are actually assigned to sessions
        for agent in busy_agents {
            if !session_agent_map.contains_key(&agent.id) {
                report.add_result(
                    ValidationResult::warning(
                        "busy_agent_not_assigned".to_string(),
                        format!("Agent {} is marked as busy but not assigned to any active session", agent.id),
                    )
                    .with_entity_id(agent.id.to_string())
                    .with_suggestion("Mark agent as available or assign to session".to_string()),
                );
            }
        }

        Ok(())
    }

    async fn validate_instance_consistency(&self, report: &mut ValidationReport) -> Result<(), OrchestrationValidationError> {
        let active_instances = self.instance_repository.find_active_instances().await?;

        for instance in active_instances {
            // Check if referenced agent exists
            if self.agent_repository.find_by_id(instance.agent_id).await?.is_none() {
                report.add_result(
                    ValidationResult::error(
                        "instance_references_missing_agent".to_string(),
                        format!("Instance {} references non-existent agent {}", instance.id, instance.agent_id),
                    )
                    .with_suggestion("Clean up orphaned instances".to_string()),
                );
            }

            // Check if referenced session exists
            if self.session_repository.find_by_id(instance.session_id).await?.is_none() {
                report.add_result(
                    ValidationResult::error(
                        "instance_references_missing_session".to_string(),
                        format!("Instance {} references non-existent session {}", instance.id, instance.session_id),
                    )
                    .with_suggestion("Clean up orphaned instances".to_string()),
                );
            }
        }

        Ok(())
    }

    async fn validate_orphaned_entities(&self, report: &mut ValidationReport) -> Result<(), OrchestrationValidationError> {
        // Check for instances without corresponding session agent entries
        let active_instances = self.instance_repository.find_active_instances().await?;
        let sessions = self.session_repository
            .find_by_criteria(&crate::repositories::SessionQueryCriteria::new())
            .await?;

        let mut session_agents: HashSet<(SessionId, AgentId)> = HashSet::new();
        for session in sessions {
            for agent_id in session.active_agents().keys() {
                session_agents.insert((session.id(), *agent_id));
            }
        }

        for instance in active_instances {
            let key = (instance.session_id, instance.agent_id);
            if !session_agents.contains(&key) {
                report.add_result(
                    ValidationResult::warning(
                        "orphaned_instance".to_string(),
                        format!(
                            "Instance {} exists but session {} doesn't track agent {}",
                            instance.id, instance.session_id, instance.agent_id
                        ),
                    )
                    .with_suggestion("Sync instance with session state or clean up instance".to_string()),
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_creation() {
        let error_result = ValidationResult::error(
            "test_rule".to_string(),
            "Test error message".to_string(),
        );
        assert_eq!(error_result.severity, ValidationSeverity::Error);
        assert_eq!(error_result.rule_name, "test_rule");
        assert_eq!(error_result.message, "Test error message");

        let warning_result = ValidationResult::warning(
            "test_rule".to_string(),
            "Test warning message".to_string(),
        ).with_entity_id("entity_123".to_string())
         .with_suggestion("Fix this issue".to_string());
        
        assert_eq!(warning_result.severity, ValidationSeverity::Warning);
        assert_eq!(warning_result.entity_id, Some("entity_123".to_string()));
        assert_eq!(warning_result.suggested_action, Some("Fix this issue".to_string()));
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        
        report.add_result(ValidationResult::error("rule1".to_string(), "Error 1".to_string()));
        report.add_result(ValidationResult::warning("rule2".to_string(), "Warning 1".to_string()));
        report.add_result(ValidationResult::info("rule3".to_string(), "Info 1".to_string()));
        
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.warning_count(), 1);
        assert_eq!(report.total_issues(), 2);
        assert!(report.has_errors());
        assert!(report.has_warnings());
        assert!(!report.is_clean());
    }

    #[test]
    fn test_validation_report_clean() {
        let mut report = ValidationReport::new();
        report.add_result(ValidationResult::info("rule1".to_string(), "Just info".to_string()));
        
        assert_eq!(report.error_count(), 0);
        assert_eq!(report.warning_count(), 0);
        assert!(!report.has_errors());
        assert!(!report.has_warnings());
        assert!(report.is_clean());
    }

    #[test]
    fn test_orchestration_validator_config_default() {
        let config = OrchestrationValidatorConfig::default();
        
        assert_eq!(config.max_session_duration_hours, 24);
        assert_eq!(config.max_agents_per_session, 10);
        assert_eq!(config.max_instances_per_agent, 3);
        assert_eq!(config.session_timeout_warning_threshold_minutes, 50);
        assert!(config.check_resource_constraints);
        assert!(config.check_temporal_constraints);
        assert!(config.check_state_consistency);
        assert!(config.check_business_rules);
    }
}