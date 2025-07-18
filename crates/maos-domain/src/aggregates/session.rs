use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Session aggregate - represents an orchestration session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub task_description: String,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Created,
    InProgress,
    Completed,
    Failed,
}

impl Session {
    pub fn new(task_description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            task_description,
            status: SessionStatus::Created,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn start(&mut self) {
        self.status = SessionStatus::InProgress;
        self.updated_at = Utc::now();
    }

    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
        self.updated_at = Utc::now();
    }

    pub fn fail(&mut self) {
        self.status = SessionStatus::Failed;
        self.updated_at = Utc::now();
    }
}
