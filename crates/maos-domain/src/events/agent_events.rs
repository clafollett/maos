use crate::aggregates::{AgentStatus, InstanceStatus};
use crate::events::domain_event::{BaseEvent, DomainEvent, EventError, impl_domain_event_as_json};
use crate::value_objects::{AgentId, AgentRole, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Agent was registered in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistered {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub agent_id: AgentId,
    pub name: String,
    pub role: AgentRole,
    pub capabilities: Vec<String>,
    pub registered_at: DateTime<Utc>,
}

impl AgentRegistered {
    pub fn new(
        agent_id: AgentId,
        name: String,
        role: AgentRole,
        capabilities: Vec<String>,
    ) -> Self {
        Self {
            base: BaseEvent::new(agent_id.to_string(), 1),
            agent_id,
            name,
            role,
            capabilities,
            registered_at: Utc::now(),
        }
    }
}

impl DomainEvent for AgentRegistered {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "AgentRegistered"
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

/// Agent status changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusChanged {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub agent_id: AgentId,
    pub old_status: AgentStatus,
    pub new_status: AgentStatus,
    pub changed_at: DateTime<Utc>,
    pub reason: Option<String>,
}

impl AgentStatusChanged {
    pub fn new(
        agent_id: AgentId,
        version: u64,
        old_status: AgentStatus,
        new_status: AgentStatus,
        reason: Option<String>,
    ) -> Self {
        Self {
            base: BaseEvent::new(agent_id.to_string(), version),
            agent_id,
            old_status,
            new_status,
            changed_at: Utc::now(),
            reason,
        }
    }
}

impl DomainEvent for AgentStatusChanged {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "AgentStatusChanged"
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

/// Agent instance was spawned for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstanceSpawned {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub instance_id: Uuid,
    pub agent_id: AgentId,
    pub session_id: SessionId,
    pub spawned_at: DateTime<Utc>,
}

impl AgentInstanceSpawned {
    pub fn new(instance_id: Uuid, agent_id: AgentId, session_id: SessionId) -> Self {
        Self {
            base: BaseEvent::new(instance_id.to_string(), 1),
            instance_id,
            agent_id,
            session_id,
            spawned_at: Utc::now(),
        }
    }
}

impl DomainEvent for AgentInstanceSpawned {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "AgentInstanceSpawned"
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

/// Agent instance status changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstanceStatusChanged {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub instance_id: Uuid,
    pub agent_id: AgentId,
    pub session_id: SessionId,
    pub old_status: InstanceStatus,
    pub new_status: InstanceStatus,
    pub changed_at: DateTime<Utc>,
}

impl AgentInstanceStatusChanged {
    pub fn new(
        instance_id: Uuid,
        version: u64,
        agent_id: AgentId,
        session_id: SessionId,
        old_status: InstanceStatus,
        new_status: InstanceStatus,
    ) -> Self {
        Self {
            base: BaseEvent::new(instance_id.to_string(), version),
            instance_id,
            agent_id,
            session_id,
            old_status,
            new_status,
            changed_at: Utc::now(),
        }
    }
}

impl DomainEvent for AgentInstanceStatusChanged {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "AgentInstanceStatusChanged"
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

/// Agent instance completed its work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstanceCompleted {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub instance_id: Uuid,
    pub agent_id: AgentId,
    pub session_id: SessionId,
    pub completed_at: DateTime<Utc>,
    pub success: bool,
    pub output_summary: Option<String>,
    pub error_message: Option<String>,
}

impl AgentInstanceCompleted {
    pub fn new(
        instance_id: Uuid,
        version: u64,
        agent_id: AgentId,
        session_id: SessionId,
        success: bool,
        output_summary: Option<String>,
        error_message: Option<String>,
    ) -> Self {
        Self {
            base: BaseEvent::new(instance_id.to_string(), version),
            instance_id,
            agent_id,
            session_id,
            completed_at: Utc::now(),
            success,
            output_summary,
            error_message,
        }
    }
}

impl DomainEvent for AgentInstanceCompleted {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "AgentInstanceCompleted"
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

/// Agent capabilities were updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilitiesUpdated {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub agent_id: AgentId,
    pub old_capabilities: Vec<String>,
    pub new_capabilities: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

impl AgentCapabilitiesUpdated {
    pub fn new(
        agent_id: AgentId,
        version: u64,
        old_capabilities: Vec<String>,
        new_capabilities: Vec<String>,
    ) -> Self {
        Self {
            base: BaseEvent::new(agent_id.to_string(), version),
            agent_id,
            old_capabilities,
            new_capabilities,
            updated_at: Utc::now(),
        }
    }
}

impl DomainEvent for AgentCapabilitiesUpdated {
    fn event_id(&self) -> Uuid {
        self.base.event_id
    }

    fn event_type(&self) -> &'static str {
        "AgentCapabilitiesUpdated"
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
    use crate::value_objects::AgentRole;

    #[test]
    fn test_agent_registered_event() {
        let agent_id = AgentId::new();
        let role = AgentRole::from("backend_engineer");
        let capabilities = vec!["rust".to_string(), "api_design".to_string()];
        
        let event = AgentRegistered::new(
            agent_id,
            "Backend Agent 1".to_string(),
            role.clone(),
            capabilities.clone(),
        );

        assert_eq!(event.event_type(), "AgentRegistered");
        assert_eq!(event.agent_id, agent_id);
        assert_eq!(event.name, "Backend Agent 1");
        assert_eq!(event.role, role);
        assert_eq!(event.capabilities, capabilities);
        assert_eq!(event.aggregate_version(), 1);
    }

    #[test]
    fn test_agent_status_changed_event() {
        let agent_id = AgentId::new();
        let event = AgentStatusChanged::new(
            agent_id,
            2,
            AgentStatus::Available,
            AgentStatus::Busy,
            Some("Assigned to session".to_string()),
        );

        assert_eq!(event.event_type(), "AgentStatusChanged");
        assert_eq!(event.agent_id, agent_id);
        assert!(matches!(event.old_status, AgentStatus::Available));
        assert!(matches!(event.new_status, AgentStatus::Busy));
        assert_eq!(event.reason, Some("Assigned to session".to_string()));
        assert_eq!(event.aggregate_version(), 2);
    }

    #[test]
    fn test_agent_instance_spawned_event() {
        let instance_id = Uuid::new_v4();
        let agent_id = AgentId::new();
        let session_id = SessionId::new();
        
        let event = AgentInstanceSpawned::new(instance_id, agent_id, session_id);

        assert_eq!(event.event_type(), "AgentInstanceSpawned");
        assert_eq!(event.instance_id, instance_id);
        assert_eq!(event.agent_id, agent_id);
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.aggregate_version(), 1);
    }

    #[test]
    fn test_agent_instance_completed_event() {
        let instance_id = Uuid::new_v4();
        let agent_id = AgentId::new();
        let session_id = SessionId::new();
        
        let event = AgentInstanceCompleted::new(
            instance_id,
            3,
            agent_id,
            session_id,
            true,
            Some("API implementation completed".to_string()),
            None,
        );

        assert_eq!(event.event_type(), "AgentInstanceCompleted");
        assert_eq!(event.instance_id, instance_id);
        assert_eq!(event.agent_id, agent_id);
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.success, true);
        assert_eq!(event.output_summary, Some("API implementation completed".to_string()));
        assert_eq!(event.error_message, None);
        assert_eq!(event.aggregate_version(), 3);
    }

    #[test]
    fn test_agent_instance_completed_with_error() {
        let instance_id = Uuid::new_v4();
        let agent_id = AgentId::new();
        let session_id = SessionId::new();
        
        let event = AgentInstanceCompleted::new(
            instance_id,
            3,
            agent_id,
            session_id,
            false,
            None,
            Some("Database connection failed".to_string()),
        );

        assert_eq!(event.success, false);
        assert_eq!(event.output_summary, None);
        assert_eq!(event.error_message, Some("Database connection failed".to_string()));
    }

    #[test]
    fn test_agent_capabilities_updated_event() {
        let agent_id = AgentId::new();
        let old_capabilities = vec!["rust".to_string()];
        let new_capabilities = vec!["rust".to_string(), "docker".to_string()];
        
        let event = AgentCapabilitiesUpdated::new(
            agent_id,
            4,
            old_capabilities.clone(),
            new_capabilities.clone(),
        );

        assert_eq!(event.event_type(), "AgentCapabilitiesUpdated");
        assert_eq!(event.agent_id, agent_id);
        assert_eq!(event.old_capabilities, old_capabilities);
        assert_eq!(event.new_capabilities, new_capabilities);
        assert_eq!(event.aggregate_version(), 4);
    }

    #[test]
    fn test_domain_event_trait_implementation() {
        let agent_id = AgentId::new();
        let role = AgentRole::from("tester");
        let event = AgentRegistered::new(
            agent_id,
            "Test Agent".to_string(),
            role,
            vec!["testing".to_string()],
        );

        // Test DomainEvent trait methods
        assert_eq!(event.aggregate_id(), agent_id.to_string());
        assert_eq!(event.aggregate_version(), 1);
        assert!(event.metadata().is_empty());
        
        // Ensure event_id is unique
        let event2 = AgentRegistered::new(
            agent_id,
            "Test Agent 2".to_string(),
            AgentRole::from("tester"),
            vec!["testing".to_string()],
        );
        assert_ne!(event.event_id(), event2.event_id());
    }
}