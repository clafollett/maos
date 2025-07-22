use crate::aggregates::session::{SessionStatus, SessionAgentStatus};
use crate::events::domain_event::{BaseEvent, DomainEvent, EventError, impl_domain_event_as_json};
use crate::value_objects::{AgentId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Session was created with initial task and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreated {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub session_id: SessionId,
    pub task_description: String,
    pub workspace_path: String,
    pub max_agents: usize,
    pub timeout_minutes: u32,
}

impl SessionCreated {
    pub fn new(
        session_id: SessionId,
        task_description: String,
        workspace_path: String,
        max_agents: usize,
        timeout_minutes: u32,
    ) -> Self {
        Self {
            base: BaseEvent::new(session_id.to_string(), 1),
            session_id,
            task_description,
            workspace_path,
            max_agents,
            timeout_minutes,
        }
    }
}

impl DomainEvent for SessionCreated {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionCreated"
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

    impl_domain_event_as_json!();
}

/// Session was started and is now in progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStarted {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub session_id: SessionId,
    pub started_at: DateTime<Utc>,
}

impl SessionStarted {
    pub fn new(session_id: SessionId, version: u64) -> Self {
        Self {
            base: BaseEvent::new(session_id.to_string(), version),
            session_id,
            started_at: Utc::now(),
        }
    }
}

impl DomainEvent for SessionStarted {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionStarted"
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

    impl_domain_event_as_json!();
}

/// Session was paused
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPaused {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub session_id: SessionId,
    pub paused_at: DateTime<Utc>,
    pub reason: Option<String>,
}

impl SessionPaused {
    pub fn new(session_id: SessionId, version: u64, reason: Option<String>) -> Self {
        Self {
            base: BaseEvent::new(session_id.to_string(), version),
            session_id,
            paused_at: Utc::now(),
            reason,
        }
    }
}

impl DomainEvent for SessionPaused {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionPaused"
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

    impl_domain_event_as_json!();
}

/// Session was resumed from paused state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResumed {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub session_id: SessionId,
    pub resumed_at: DateTime<Utc>,
}

impl SessionResumed {
    pub fn new(session_id: SessionId, version: u64) -> Self {
        Self {
            base: BaseEvent::new(session_id.to_string(), version),
            session_id,
            resumed_at: Utc::now(),
        }
    }
}

impl DomainEvent for SessionResumed {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionResumed"
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

    impl_domain_event_as_json!();
}

/// Session completed successfully
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCompleted {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub session_id: SessionId,
    pub completed_at: DateTime<Utc>,
    pub total_phases: u32,
    pub duration_seconds: i64,
    pub agents_used: usize,
}

impl SessionCompleted {
    pub fn new(
        session_id: SessionId,
        version: u64,
        total_phases: u32,
        duration_seconds: i64,
        agents_used: usize,
    ) -> Self {
        Self {
            base: BaseEvent::new(session_id.to_string(), version),
            session_id,
            completed_at: Utc::now(),
            total_phases,
            duration_seconds,
            agents_used,
        }
    }
}

impl DomainEvent for SessionCompleted {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionCompleted"
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

    impl_domain_event_as_json!();
}

/// Session failed with error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFailed {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub session_id: SessionId,
    pub failed_at: DateTime<Utc>,
    pub error_message: String,
    pub phase_count: u32,
    pub agents_used: usize,
}

impl SessionFailed {
    pub fn new(
        session_id: SessionId,
        version: u64,
        error_message: String,
        phase_count: u32,
        agents_used: usize,
    ) -> Self {
        Self {
            base: BaseEvent::new(session_id.to_string(), version),
            session_id,
            failed_at: Utc::now(),
            error_message,
            phase_count,
            agents_used,
        }
    }
}

impl DomainEvent for SessionFailed {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionFailed"
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

    impl_domain_event_as_json!();
}

/// Session was cancelled by user or system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCancelled {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub session_id: SessionId,
    pub cancelled_at: DateTime<Utc>,
    pub reason: Option<String>,
}

impl SessionCancelled {
    pub fn new(session_id: SessionId, version: u64, reason: Option<String>) -> Self {
        Self {
            base: BaseEvent::new(session_id.to_string(), version),
            session_id,
            cancelled_at: Utc::now(),
            reason,
        }
    }
}

impl DomainEvent for SessionCancelled {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionCancelled"
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

    impl_domain_event_as_json!();
}

/// Agent was spawned in the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAgentSpawned {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub session_id: SessionId,
    pub agent_id: AgentId,
    pub role_name: String,
    pub instance_number: u32,
    pub spawned_at: DateTime<Utc>,
}

impl SessionAgentSpawned {
    pub fn new(
        session_id: SessionId,
        version: u64,
        agent_id: AgentId,
        role_name: String,
        instance_number: u32,
    ) -> Self {
        Self {
            base: BaseEvent::new(session_id.to_string(), version),
            session_id,
            agent_id,
            role_name,
            instance_number,
            spawned_at: Utc::now(),
        }
    }
}

impl DomainEvent for SessionAgentSpawned {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionAgentSpawned"
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

    impl_domain_event_as_json!();
}

/// Agent status changed within the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAgentStatusChanged {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub session_id: SessionId,
    pub agent_id: AgentId,
    pub old_status: SessionAgentStatus,
    pub new_status: SessionAgentStatus,
    pub changed_at: DateTime<Utc>,
}

impl SessionAgentStatusChanged {
    pub fn new(
        session_id: SessionId,
        version: u64,
        agent_id: AgentId,
        old_status: SessionAgentStatus,
        new_status: SessionAgentStatus,
    ) -> Self {
        Self {
            base: BaseEvent::new(session_id.to_string(), version),
            session_id,
            agent_id,
            old_status,
            new_status,
            changed_at: Utc::now(),
        }
    }
}

impl DomainEvent for SessionAgentStatusChanged {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionAgentStatusChanged"
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

    impl_domain_event_as_json!();
}

/// Session phase advanced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPhaseAdvanced {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub session_id: SessionId,
    pub previous_phase: u32,
    pub new_phase: u32,
    pub total_phases: Option<u32>,
    pub advanced_at: DateTime<Utc>,
}

impl SessionPhaseAdvanced {
    pub fn new(
        session_id: SessionId,
        version: u64,
        previous_phase: u32,
        new_phase: u32,
        total_phases: Option<u32>,
    ) -> Self {
        Self {
            base: BaseEvent::new(session_id.to_string(), version),
            session_id,
            previous_phase,
            new_phase,
            total_phases,
            advanced_at: Utc::now(),
        }
    }
}

impl DomainEvent for SessionPhaseAdvanced {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionPhaseAdvanced"
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

    impl_domain_event_as_json!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_session_created_event() {
        let session_id = SessionId::new();
        let event = SessionCreated::new(
            session_id,
            "Build a web API".to_string(),
            "/workspace".to_string(),
            10,
            60,
        );

        assert_eq!(event.event_type(), "SessionCreated");
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.task_description, "Build a web API");
        assert_eq!(event.max_agents, 10);
        assert_eq!(event.timeout_minutes, 60);
        assert_eq!(event.aggregate_version(), 1);
    }

    #[test]
    fn test_session_started_event() {
        let session_id = SessionId::new();
        let event = SessionStarted::new(session_id, 2);

        assert_eq!(event.event_type(), "SessionStarted");
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.aggregate_version(), 2);
    }

    #[test]
    fn test_session_completed_event() {
        let session_id = SessionId::new();
        let event = SessionCompleted::new(session_id, 5, 3, 120, 2);

        assert_eq!(event.event_type(), "SessionCompleted");
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.total_phases, 3);
        assert_eq!(event.duration_seconds, 120);
        assert_eq!(event.agents_used, 2);
    }

    #[test]
    fn test_session_failed_event() {
        let session_id = SessionId::new();
        let event = SessionFailed::new(
            session_id,
            3,
            "Agent communication timeout".to_string(),
            1,
            1,
        );

        assert_eq!(event.event_type(), "SessionFailed");
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.error_message, "Agent communication timeout");
        assert_eq!(event.phase_count, 1);
        assert_eq!(event.agents_used, 1);
    }

    #[test]
    fn test_session_agent_spawned_event() {
        let session_id = SessionId::new();
        let agent_id = AgentId::new();
        let event = SessionAgentSpawned::new(
            session_id,
            4,
            agent_id,
            "backend_engineer".to_string(),
            1,
        );

        assert_eq!(event.event_type(), "SessionAgentSpawned");
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.agent_id, agent_id);
        assert_eq!(event.role_name, "backend_engineer");
        assert_eq!(event.instance_number, 1);
    }

    #[test]
    fn test_session_phase_advanced_event() {
        let session_id = SessionId::new();
        let event = SessionPhaseAdvanced::new(session_id, 6, 1, 2, Some(3));

        assert_eq!(event.event_type(), "SessionPhaseAdvanced");
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.previous_phase, 1);
        assert_eq!(event.new_phase, 2);
        assert_eq!(event.total_phases, Some(3));
    }

    #[test]
    fn test_domain_event_trait_implementation() {
        let session_id = SessionId::new();
        let event = SessionCreated::new(
            session_id,
            "Test".to_string(),
            "/workspace".to_string(),
            10,
            60,
        );

        // Test DomainEvent trait methods
        assert_eq!(event.aggregate_id(), session_id.to_string());
        assert_eq!(event.aggregate_version(), 1);
        assert!(event.metadata().is_empty());
        
        // Ensure event_id is unique
        let event2 = SessionCreated::new(
            session_id,
            "Test2".to_string(),
            "/workspace".to_string(),
            10,
            60,
        );
        assert_ne!(event.event_id(), event2.event_id());
    }
}
