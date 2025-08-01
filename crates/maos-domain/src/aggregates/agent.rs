use crate::value_objects::{AgentId, AgentRole};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Agent aggregate - represents an AI agent in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Available,
    Busy,
    Offline,
    Error,
}

impl Agent {
    pub fn new(name: String, role: AgentRole, capabilities: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: AgentId::new(),
            name,
            role,
            status: AgentStatus::Available,
            capabilities,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn set_busy(&mut self) {
        self.status = AgentStatus::Busy;
        self.updated_at = Utc::now();
    }

    pub fn set_available(&mut self) {
        self.status = AgentStatus::Available;
        self.updated_at = Utc::now();
    }

    pub fn set_offline(&mut self) {
        self.status = AgentStatus::Offline;
        self.updated_at = Utc::now();
    }

    pub fn set_error(&mut self) {
        self.status = AgentStatus::Error;
        self.updated_at = Utc::now();
    }
}
