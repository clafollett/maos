use crate::events::domain_event::{BaseEvent, DomainEvent};
use crate::value_objects::{AgentId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Orchestration process started for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStarted {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub session_id: SessionId,
    pub orchestration_id: Uuid,
    pub initial_strategy: String,
    pub expected_phases: Option<u32>,
    pub started_at: DateTime<Utc>,
}

impl OrchestrationStarted {
    pub fn new(
        session_id: SessionId,
        orchestration_id: Uuid,
        initial_strategy: String,
        expected_phases: Option<u32>,
    ) -> Self {
        Self {
            base: BaseEvent::new(orchestration_id.to_string(), 1),
            session_id,
            orchestration_id,
            initial_strategy,
            expected_phases,
            started_at: Utc::now(),
        }
    }
}

impl DomainEvent for OrchestrationStarted {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "OrchestrationStarted"
    }

    fn aggregate_id(&self) -> String {
        self.base.aggregate_id.clone()
    }

    fn aggregate_version(&self) -> u64 {
        self.base.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.occurred_at
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }
}

/// Agent assignment was made during orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAssigned {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub orchestration_id: Uuid,
    pub session_id: SessionId,
    pub agent_id: AgentId,
    pub role_name: String,
    pub assignment_reason: String,
    pub priority: u32,
    pub assigned_at: DateTime<Utc>,
}

impl AgentAssigned {
    pub fn new(
        orchestration_id: Uuid,
        version: u64,
        session_id: SessionId,
        agent_id: AgentId,
        role_name: String,
        assignment_reason: String,
        priority: u32,
    ) -> Self {
        Self {
            base: BaseEvent::new(orchestration_id.to_string(), version),
            orchestration_id,
            session_id,
            agent_id,
            role_name,
            assignment_reason,
            priority,
            assigned_at: Utc::now(),
        }
    }
}

impl DomainEvent for AgentAssigned {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "AgentAssigned"
    }

    fn aggregate_id(&self) -> String {
        self.base.aggregate_id.clone()
    }

    fn aggregate_version(&self) -> u64 {
        self.base.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.occurred_at
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }
}

/// Orchestration strategy was changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStrategyChanged {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub orchestration_id: Uuid,
    pub session_id: SessionId,
    pub old_strategy: String,
    pub new_strategy: String,
    pub change_reason: String,
    pub changed_at: DateTime<Utc>,
}

impl OrchestrationStrategyChanged {
    pub fn new(
        orchestration_id: Uuid,
        version: u64,
        session_id: SessionId,
        old_strategy: String,
        new_strategy: String,
        change_reason: String,
    ) -> Self {
        Self {
            base: BaseEvent::new(orchestration_id.to_string(), version),
            orchestration_id,
            session_id,
            old_strategy,
            new_strategy,
            change_reason,
            changed_at: Utc::now(),
        }
    }
}

impl DomainEvent for OrchestrationStrategyChanged {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "OrchestrationStrategyChanged"
    }

    fn aggregate_id(&self) -> String {
        self.base.aggregate_id.clone()
    }

    fn aggregate_version(&self) -> u64 {
        self.base.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.occurred_at
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }
}

/// Orchestration phase completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationPhaseCompleted {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub orchestration_id: Uuid,
    pub session_id: SessionId,
    pub phase_number: u32,
    pub phase_name: String,
    pub agents_involved: Vec<AgentId>,
    pub phase_duration_seconds: i64,
    pub success: bool,
    pub output_summary: Option<String>,
    pub completed_at: DateTime<Utc>,
}

impl OrchestrationPhaseCompleted {
    pub fn new(
        orchestration_id: Uuid,
        version: u64,
        session_id: SessionId,
        phase_number: u32,
        phase_name: String,
        agents_involved: Vec<AgentId>,
        phase_duration_seconds: i64,
        success: bool,
        output_summary: Option<String>,
    ) -> Self {
        Self {
            base: BaseEvent::new(orchestration_id.to_string(), version),
            orchestration_id,
            session_id,
            phase_number,
            phase_name,
            agents_involved,
            phase_duration_seconds,
            success,
            output_summary,
            completed_at: Utc::now(),
        }
    }
}

impl DomainEvent for OrchestrationPhaseCompleted {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "OrchestrationPhaseCompleted"
    }

    fn aggregate_id(&self) -> String {
        self.base.aggregate_id.clone()
    }

    fn aggregate_version(&self) -> u64 {
        self.base.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.occurred_at
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }
}

/// Orchestration completed successfully
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationCompleted {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub orchestration_id: Uuid,
    pub session_id: SessionId,
    pub total_phases: u32,
    pub total_agents_used: usize,
    pub total_duration_seconds: i64,
    pub final_strategy: String,
    pub success_rate: f32,
    pub completed_at: DateTime<Utc>,
}

impl OrchestrationCompleted {
    pub fn new(
        orchestration_id: Uuid,
        version: u64,
        session_id: SessionId,
        total_phases: u32,
        total_agents_used: usize,
        total_duration_seconds: i64,
        final_strategy: String,
        success_rate: f32,
    ) -> Self {
        Self {
            base: BaseEvent::new(orchestration_id.to_string(), version),
            orchestration_id,
            session_id,
            total_phases,
            total_agents_used,
            total_duration_seconds,
            final_strategy,
            success_rate,
            completed_at: Utc::now(),
        }
    }
}

impl DomainEvent for OrchestrationCompleted {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "OrchestrationCompleted"
    }

    fn aggregate_id(&self) -> String {
        self.base.aggregate_id.clone()
    }

    fn aggregate_version(&self) -> u64 {
        self.base.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.occurred_at
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }
}

/// Orchestration failed with error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationFailed {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub orchestration_id: Uuid,
    pub session_id: SessionId,
    pub failed_phase: Option<u32>,
    pub error_message: String,
    pub error_category: String,
    pub phases_completed: u32,
    pub agents_involved: Vec<AgentId>,
    pub failed_at: DateTime<Utc>,
}

impl OrchestrationFailed {
    pub fn new(
        orchestration_id: Uuid,
        version: u64,
        session_id: SessionId,
        failed_phase: Option<u32>,
        error_message: String,
        error_category: String,
        phases_completed: u32,
        agents_involved: Vec<AgentId>,
    ) -> Self {
        Self {
            base: BaseEvent::new(orchestration_id.to_string(), version),
            orchestration_id,
            session_id,
            failed_phase,
            error_message,
            error_category,
            phases_completed,
            agents_involved,
            failed_at: Utc::now(),
        }
    }
}

impl DomainEvent for OrchestrationFailed {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "OrchestrationFailed"
    }

    fn aggregate_id(&self) -> String {
        self.base.aggregate_id.clone()
    }

    fn aggregate_version(&self) -> u64 {
        self.base.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.occurred_at
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }
}

/// Resource constraint encountered during orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConstraintEncountered {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub orchestration_id: Uuid,
    pub session_id: SessionId,
    pub constraint_type: String, // e.g., "max_agents", "timeout", "memory"
    pub current_usage: String,
    pub limit_reached: String,
    pub resolution_action: Option<String>,
    pub encountered_at: DateTime<Utc>,
}

impl ResourceConstraintEncountered {
    pub fn new(
        orchestration_id: Uuid,
        version: u64,
        session_id: SessionId,
        constraint_type: String,
        current_usage: String,
        limit_reached: String,
        resolution_action: Option<String>,
    ) -> Self {
        Self {
            base: BaseEvent::new(orchestration_id.to_string(), version),
            orchestration_id,
            session_id,
            constraint_type,
            current_usage,
            limit_reached,
            resolution_action,
            encountered_at: Utc::now(),
        }
    }
}

impl DomainEvent for ResourceConstraintEncountered {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "ResourceConstraintEncountered"
    }

    fn aggregate_id(&self) -> String {
        self.base.aggregate_id.clone()
    }

    fn aggregate_version(&self) -> u64 {
        self.base.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.occurred_at
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestration_started_event() {
        let session_id = SessionId::new();
        let orchestration_id = Uuid::new_v4();
        let event = OrchestrationStarted::new(
            session_id,
            orchestration_id,
            "adaptive_phase_based".to_string(),
            Some(3),
        );

        assert_eq!(event.event_type(), "OrchestrationStarted");
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.orchestration_id, orchestration_id);
        assert_eq!(event.initial_strategy, "adaptive_phase_based");
        assert_eq!(event.expected_phases, Some(3));
        assert_eq!(event.aggregate_version(), 1);
    }

    #[test]
    fn test_agent_assigned_event() {
        let orchestration_id = Uuid::new_v4();
        let session_id = SessionId::new();
        let agent_id = AgentId::new();
        
        let event = AgentAssigned::new(
            orchestration_id,
            2,
            session_id,
            agent_id,
            "backend_engineer".to_string(),
            "API implementation needed".to_string(),
            1,
        );

        assert_eq!(event.event_type(), "AgentAssigned");
        assert_eq!(event.orchestration_id, orchestration_id);
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.agent_id, agent_id);
        assert_eq!(event.role_name, "backend_engineer");
        assert_eq!(event.assignment_reason, "API implementation needed");
        assert_eq!(event.priority, 1);
        assert_eq!(event.aggregate_version(), 2);
    }

    #[test]
    fn test_orchestration_phase_completed_event() {
        let orchestration_id = Uuid::new_v4();
        let session_id = SessionId::new();
        let agents = vec![AgentId::new(), AgentId::new()];
        
        let event = OrchestrationPhaseCompleted::new(
            orchestration_id,
            3,
            session_id,
            1,
            "Analysis Phase".to_string(),
            agents.clone(),
            120,
            true,
            Some("Requirements analyzed successfully".to_string()),
        );

        assert_eq!(event.event_type(), "OrchestrationPhaseCompleted");
        assert_eq!(event.orchestration_id, orchestration_id);
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.phase_number, 1);
        assert_eq!(event.phase_name, "Analysis Phase");
        assert_eq!(event.agents_involved, agents);
        assert_eq!(event.phase_duration_seconds, 120);
        assert_eq!(event.success, true);
        assert_eq!(event.output_summary, Some("Requirements analyzed successfully".to_string()));
    }

    #[test]
    fn test_orchestration_completed_event() {
        let orchestration_id = Uuid::new_v4();
        let session_id = SessionId::new();
        
        let event = OrchestrationCompleted::new(
            orchestration_id,
            5,
            session_id,
            3,
            4,
            300,
            "adaptive_phase_based".to_string(),
            0.95,
        );

        assert_eq!(event.event_type(), "OrchestrationCompleted");
        assert_eq!(event.orchestration_id, orchestration_id);
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.total_phases, 3);
        assert_eq!(event.total_agents_used, 4);
        assert_eq!(event.total_duration_seconds, 300);
        assert_eq!(event.final_strategy, "adaptive_phase_based");
        assert_eq!(event.success_rate, 0.95);
    }

    #[test]
    fn test_orchestration_failed_event() {
        let orchestration_id = Uuid::new_v4();
        let session_id = SessionId::new();
        let agents = vec![AgentId::new()];
        
        let event = OrchestrationFailed::new(
            orchestration_id,
            4,
            session_id,
            Some(2),
            "Agent communication timeout".to_string(),
            "timeout".to_string(),
            1,
            agents.clone(),
        );

        assert_eq!(event.event_type(), "OrchestrationFailed");
        assert_eq!(event.orchestration_id, orchestration_id);
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.failed_phase, Some(2));
        assert_eq!(event.error_message, "Agent communication timeout");
        assert_eq!(event.error_category, "timeout");
        assert_eq!(event.phases_completed, 1);
        assert_eq!(event.agents_involved, agents);
    }

    #[test]
    fn test_resource_constraint_encountered_event() {
        let orchestration_id = Uuid::new_v4();
        let session_id = SessionId::new();
        
        let event = ResourceConstraintEncountered::new(
            orchestration_id,
            3,
            session_id,
            "max_agents".to_string(),
            "10".to_string(),
            "10".to_string(),
            Some("Queue additional agent requests".to_string()),
        );

        assert_eq!(event.event_type(), "ResourceConstraintEncountered");
        assert_eq!(event.orchestration_id, orchestration_id);
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.constraint_type, "max_agents");
        assert_eq!(event.current_usage, "10");
        assert_eq!(event.limit_reached, "10");
        assert_eq!(event.resolution_action, Some("Queue additional agent requests".to_string()));
    }

    #[test]
    fn test_domain_event_trait_implementation() {
        let session_id = SessionId::new();
        let orchestration_id = Uuid::new_v4();
        let event = OrchestrationStarted::new(
            session_id,
            orchestration_id,
            "test_strategy".to_string(),
            None,
        );

        // Test DomainEvent trait methods
        assert_eq!(event.aggregate_id(), orchestration_id.to_string());
        assert_eq!(event.aggregate_version(), 1);
        assert!(event.metadata().is_empty());
        
        // Ensure event_id is unique
        let event2 = OrchestrationStarted::new(
            session_id,
            orchestration_id,
            "test_strategy2".to_string(),
            None,
        );
        assert_ne!(event.event_id(), event2.event_id());
    }
}