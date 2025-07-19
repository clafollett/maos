use crate::value_objects::{AgentId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Instance aggregate - represents a running instance of an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: uuid::Uuid,
    pub agent_id: AgentId,
    pub session_id: SessionId,
    pub status: InstanceStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

impl Instance {
    pub fn new(agent_id: AgentId, session_id: SessionId) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4(),
            agent_id,
            session_id,
            status: InstanceStatus::Starting,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn start(&mut self) {
        self.status = InstanceStatus::Running;
        self.updated_at = Utc::now();
    }

    pub fn stop(&mut self) {
        self.status = InstanceStatus::Stopping;
        self.updated_at = Utc::now();
    }

    pub fn stopped(&mut self) {
        self.status = InstanceStatus::Stopped;
        self.updated_at = Utc::now();
    }

    pub fn fail(&mut self) {
        self.status = InstanceStatus::Failed;
        self.updated_at = Utc::now();
    }
}
