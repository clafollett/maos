use crate::value_objects::{AgentId, AgentRole, SessionId, Workspace};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Domain errors for Agent aggregate
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent validation failed: {0}")]
    ValidationError(String),
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition { from: AgentStatus, to: AgentStatus },
    #[error("Agent name cannot be empty")]
    EmptyName,
    #[error("Agent is not assigned to session {session_id}")]
    NotAssignedToSession { session_id: SessionId },
    #[error("Agent context cannot be empty")]
    EmptyContext,
    #[error("Work history entry cannot be empty")]
    EmptyWorkHistoryEntry,
}

/// Agent context information for maintaining state across tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub current_task: Option<String>,
    pub workspace: Option<Workspace>,
    pub session_id: Option<SessionId>,
    pub shared_context: HashMap<String, String>,
    pub last_interaction: Option<DateTime<Utc>>,
}

impl AgentContext {
    pub fn new() -> Self {
        Self {
            current_task: None,
            workspace: None,
            session_id: None,
            shared_context: HashMap::new(),
            last_interaction: None,
        }
    }

    pub fn with_task(mut self, task: String) -> Self {
        self.current_task = Some(task);
        self.last_interaction = Some(Utc::now());
        self
    }

    pub fn with_workspace(mut self, workspace: Workspace) -> Self {
        self.workspace = Some(workspace);
        self
    }

    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn add_context(&mut self, key: String, value: String) {
        self.shared_context.insert(key, value);
        self.last_interaction = Some(Utc::now());
    }

    pub fn clear_task(&mut self) {
        self.current_task = None;
        self.last_interaction = Some(Utc::now());
    }
}

impl Default for AgentContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Work history entry for tracking agent activities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkHistoryEntry {
    pub task_description: String,
    pub session_id: Option<SessionId>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<String>,
    pub status: WorkEntryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum WorkEntryStatus {
    #[default]
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl WorkHistoryEntry {
    pub fn new(
        task_description: String,
        session_id: Option<SessionId>,
    ) -> Result<Self, AgentError> {
        if task_description.trim().is_empty() {
            return Err(AgentError::EmptyWorkHistoryEntry);
        }

        Ok(Self {
            task_description: task_description.trim().to_string(),
            session_id,
            started_at: Utc::now(),
            completed_at: None,
            result: None,
            status: WorkEntryStatus::InProgress,
        })
    }

    pub fn complete(mut self, result: String) -> Self {
        self.completed_at = Some(Utc::now());
        self.result = Some(result);
        self.status = WorkEntryStatus::Completed;
        self
    }

    pub fn fail(mut self, error: String) -> Self {
        self.completed_at = Some(Utc::now());
        self.result = Some(error);
        self.status = WorkEntryStatus::Failed;
        self
    }

    pub fn cancel(mut self) -> Self {
        self.completed_at = Some(Utc::now());
        self.status = WorkEntryStatus::Cancelled;
        self
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        match self.completed_at {
            Some(end) => Some(end - self.started_at),
            None => Some(Utc::now() - self.started_at),
        }
    }
}

/// Agent status with proper state machine semantics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent is created but not yet assigned to any work
    Idle,
    /// Agent is actively working on a task
    Running,
    /// Agent has completed its current work successfully
    Completed,
    /// Agent encountered an error and cannot continue
    Failed,
    /// Agent has been terminated (by system or user)
    Terminated,
}

/// Agent aggregate - represents an AI agent with proper domain behaviors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    id: AgentId,
    name: String,
    role: AgentRole,
    status: AgentStatus,
    context: AgentContext,
    work_history: Vec<WorkHistoryEntry>,
    capabilities: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Agent {
    /// Create a new Agent with validation
    pub fn new(name: String, role: AgentRole) -> Result<Self, AgentError> {
        if name.trim().is_empty() {
            return Err(AgentError::EmptyName);
        }

        let now = Utc::now();
        Ok(Self {
            id: AgentId::new(),
            name: name.trim().to_string(),
            role,
            status: AgentStatus::Idle,
            context: AgentContext::new(),
            work_history: Vec::new(),
            capabilities: Vec::new(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Create an Agent with predefined capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self.updated_at = Utc::now();
        self
    }

    // Getters
    pub fn id(&self) -> AgentId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> &AgentRole {
        &self.role
    }

    pub fn status(&self) -> &AgentStatus {
        &self.status
    }

    pub fn context(&self) -> &AgentContext {
        &self.context
    }

    pub fn work_history(&self) -> &[WorkHistoryEntry] {
        &self.work_history
    }

    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // State transitions with business rules
    pub fn assign_task(
        &mut self,
        task: String,
        session_id: Option<SessionId>,
        workspace: Option<Workspace>,
    ) -> Result<(), AgentError> {
        // Only idle or completed agents can be assigned new tasks
        if !matches!(self.status, AgentStatus::Idle | AgentStatus::Completed) {
            return Err(AgentError::InvalidTransition {
                from: self.status.clone(),
                to: AgentStatus::Running,
            });
        }

        if task.trim().is_empty() {
            return Err(AgentError::ValidationError(
                "Task cannot be empty".to_string(),
            ));
        }

        // Update context
        self.context.current_task = Some(task.clone());
        if let Some(session_id) = session_id {
            self.context.session_id = Some(session_id);
        }
        if let Some(workspace) = workspace {
            self.context.workspace = Some(workspace);
        }
        self.context.last_interaction = Some(Utc::now());

        // Add work history entry
        let work_entry = WorkHistoryEntry::new(task, session_id)?;
        self.work_history.push(work_entry);

        // Transition to running state
        self.status = AgentStatus::Running;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn complete_task(&mut self, result: String) -> Result<(), AgentError> {
        if !matches!(self.status, AgentStatus::Running) {
            return Err(AgentError::InvalidTransition {
                from: self.status.clone(),
                to: AgentStatus::Completed,
            });
        }

        // Update the most recent work history entry
        if let Some(last_entry) = self.work_history.last_mut() {
            *last_entry = std::mem::take(last_entry).complete(result);
        }

        // Clear current task and transition state
        self.context.clear_task();
        self.status = AgentStatus::Completed;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn fail_task(&mut self, error: String) -> Result<(), AgentError> {
        if !matches!(self.status, AgentStatus::Running) {
            return Err(AgentError::InvalidTransition {
                from: self.status.clone(),
                to: AgentStatus::Failed,
            });
        }

        // Update the most recent work history entry
        if let Some(last_entry) = self.work_history.last_mut() {
            *last_entry = std::mem::take(last_entry).fail(error);
        }

        // Clear current task and transition state
        self.context.clear_task();
        self.status = AgentStatus::Failed;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn terminate(&mut self) -> Result<(), AgentError> {
        // Agents can be terminated from any state except already terminated
        if matches!(self.status, AgentStatus::Terminated) {
            return Err(AgentError::InvalidTransition {
                from: self.status.clone(),
                to: AgentStatus::Terminated,
            });
        }

        // If currently running, cancel the current work entry
        if matches!(self.status, AgentStatus::Running) {
            if let Some(last_entry) = self.work_history.last_mut() {
                *last_entry = std::mem::take(last_entry).cancel();
            }
        }

        self.context.clear_task();
        self.status = AgentStatus::Terminated;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn reset_to_idle(&mut self) -> Result<(), AgentError> {
        // Only completed or failed agents can be reset to idle
        if !matches!(self.status, AgentStatus::Completed | AgentStatus::Failed) {
            return Err(AgentError::InvalidTransition {
                from: self.status.clone(),
                to: AgentStatus::Idle,
            });
        }

        self.context.clear_task();
        self.status = AgentStatus::Idle;
        self.updated_at = Utc::now();
        Ok(())
    }

    // Context management
    pub fn add_context(&mut self, key: String, value: String) {
        self.context.add_context(key, value);
        self.updated_at = Utc::now();
    }

    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.shared_context.get(key)
    }

    // Business logic helpers
    pub fn is_available_for_work(&self) -> bool {
        matches!(self.status, AgentStatus::Idle | AgentStatus::Completed)
    }

    pub fn is_working(&self) -> bool {
        matches!(self.status, AgentStatus::Running)
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            AgentStatus::Completed | AgentStatus::Failed | AgentStatus::Terminated
        )
    }

    pub fn current_session(&self) -> Option<SessionId> {
        self.context.session_id
    }

    pub fn current_workspace(&self) -> Option<&Workspace> {
        self.context.workspace.as_ref()
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }

    pub fn total_completed_tasks(&self) -> usize {
        self.work_history
            .iter()
            .filter(|entry| matches!(entry.status, WorkEntryStatus::Completed))
            .count()
    }

    pub fn total_failed_tasks(&self) -> usize {
        self.work_history
            .iter()
            .filter(|entry| matches!(entry.status, WorkEntryStatus::Failed))
            .count()
    }

    pub fn success_rate(&self) -> f32 {
        let completed = self.total_completed_tasks() as f32;
        let total_finished = (self.total_completed_tasks() + self.total_failed_tasks()) as f32;

        if total_finished == 0.0 {
            0.0
        } else {
            completed / total_finished
        }
    }

    pub fn current_task_duration(&self) -> Option<chrono::Duration> {
        if let Some(last_entry) = self.work_history.last() {
            if matches!(last_entry.status, WorkEntryStatus::InProgress) {
                return last_entry.duration();
            }
        }
        None
    }

    // Additional state transition methods needed by repository
    pub fn set_available(&mut self) -> Result<(), AgentError> {
        self.status = AgentStatus::Idle;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_busy(&mut self) -> Result<(), AgentError> {
        self.status = AgentStatus::Running;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_offline(&mut self) -> Result<(), AgentError> {
        self.status = AgentStatus::Terminated;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_error(&mut self) -> Result<(), AgentError> {
        self.status = AgentStatus::Failed;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_role() -> AgentRole {
        AgentRole::BackendEngineer
    }

    fn create_test_workspace() -> Workspace {
        Workspace::from_path(PathBuf::from("/tmp/test/workspace")).unwrap()
    }

    #[test]
    fn test_agent_creation_success() {
        let role = create_test_role();
        let agent = Agent::new("TestAgent".to_string(), role.clone()).unwrap();

        assert_eq!(agent.name(), "TestAgent");
        assert_eq!(agent.role(), &role);
        assert!(matches!(agent.status(), AgentStatus::Idle));
        assert_eq!(agent.work_history().len(), 0);
        assert_eq!(agent.capabilities().len(), 0);
        assert!(agent.is_available_for_work());
        assert!(!agent.is_working());
        assert!(!agent.is_finished());
    }

    #[test]
    fn test_agent_creation_empty_name() {
        let role = create_test_role();
        let result = Agent::new("  ".to_string(), role);

        assert!(matches!(result, Err(AgentError::EmptyName)));
    }

    #[test]
    fn test_agent_creation_name_trimming() {
        let role = create_test_role();
        let agent = Agent::new("  TestAgent  ".to_string(), role).unwrap();

        assert_eq!(agent.name(), "TestAgent");
    }

    #[test]
    fn test_agent_with_capabilities() {
        let role = create_test_role();
        let capabilities = vec!["api-development".to_string(), "testing".to_string()];
        let agent = Agent::new("TestAgent".to_string(), role)
            .unwrap()
            .with_capabilities(capabilities.clone());

        assert_eq!(agent.capabilities(), &capabilities);
        assert!(agent.has_capability("api-development"));
        assert!(agent.has_capability("testing"));
        assert!(!agent.has_capability("frontend"));
    }

    #[test]
    fn test_assign_task_success() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();
        let session_id = SessionId::new();
        let workspace = create_test_workspace();

        let result = agent.assign_task(
            "Build REST API".to_string(),
            Some(session_id),
            Some(workspace.clone()),
        );

        assert!(result.is_ok());
        assert!(matches!(agent.status(), AgentStatus::Running));
        assert_eq!(
            agent.context().current_task,
            Some("Build REST API".to_string())
        );
        assert_eq!(agent.context().session_id, Some(session_id));
        assert_eq!(agent.context().workspace, Some(workspace));
        assert_eq!(agent.work_history().len(), 1);
        assert!(agent.is_working());
        assert!(!agent.is_available_for_work());
    }

    #[test]
    fn test_assign_task_empty() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        let result = agent.assign_task("  ".to_string(), None, None);

        assert!(matches!(result, Err(AgentError::ValidationError(_))));
        assert!(matches!(agent.status(), AgentStatus::Idle));
    }

    #[test]
    fn test_assign_task_invalid_state() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        // Assign first task to make agent running
        agent.assign_task("Task 1".to_string(), None, None).unwrap();

        // Try to assign another task while running
        let result = agent.assign_task("Task 2".to_string(), None, None);

        assert!(matches!(result, Err(AgentError::InvalidTransition { .. })));
        assert!(matches!(agent.status(), AgentStatus::Running));
        assert_eq!(agent.work_history().len(), 1); // Should still have only one task
    }

    #[test]
    fn test_complete_task_success() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        agent
            .assign_task("Build API".to_string(), None, None)
            .unwrap();
        let result = agent.complete_task("API built successfully".to_string());

        assert!(result.is_ok());
        assert!(matches!(agent.status(), AgentStatus::Completed));
        assert_eq!(agent.context().current_task, None);
        assert_eq!(agent.work_history().len(), 1);
        assert!(matches!(
            agent.work_history()[0].status,
            WorkEntryStatus::Completed
        ));
        assert_eq!(
            agent.work_history()[0].result,
            Some("API built successfully".to_string())
        );
        assert!(agent.is_finished());
        assert!(agent.is_available_for_work()); // Completed agents can accept new work
    }

    #[test]
    fn test_complete_task_invalid_state() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        let result = agent.complete_task("Result".to_string());

        assert!(matches!(result, Err(AgentError::InvalidTransition { .. })));
        assert!(matches!(agent.status(), AgentStatus::Idle));
    }

    #[test]
    fn test_fail_task_success() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        agent
            .assign_task("Build API".to_string(), None, None)
            .unwrap();
        let result = agent.fail_task("Network error occurred".to_string());

        assert!(result.is_ok());
        assert!(matches!(agent.status(), AgentStatus::Failed));
        assert_eq!(agent.context().current_task, None);
        assert_eq!(agent.work_history().len(), 1);
        assert!(matches!(
            agent.work_history()[0].status,
            WorkEntryStatus::Failed
        ));
        assert_eq!(
            agent.work_history()[0].result,
            Some("Network error occurred".to_string())
        );
        assert!(agent.is_finished());
        assert!(!agent.is_available_for_work()); // Failed agents are not available until reset
    }

    #[test]
    fn test_fail_task_invalid_state() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        let result = agent.fail_task("Error".to_string());

        assert!(matches!(result, Err(AgentError::InvalidTransition { .. })));
        assert!(matches!(agent.status(), AgentStatus::Idle));
    }

    #[test]
    fn test_terminate_success() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        let result = agent.terminate();

        assert!(result.is_ok());
        assert!(matches!(agent.status(), AgentStatus::Terminated));
        assert!(agent.is_finished());
        assert!(!agent.is_available_for_work());
    }

    #[test]
    fn test_terminate_running_agent() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        agent.assign_task("Task".to_string(), None, None).unwrap();
        let result = agent.terminate();

        assert!(result.is_ok());
        assert!(matches!(agent.status(), AgentStatus::Terminated));
        assert_eq!(agent.work_history().len(), 1);
        assert!(matches!(
            agent.work_history()[0].status,
            WorkEntryStatus::Cancelled
        ));
    }

    #[test]
    fn test_terminate_already_terminated() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        agent.terminate().unwrap();
        let result = agent.terminate();

        assert!(matches!(result, Err(AgentError::InvalidTransition { .. })));
    }

    #[test]
    fn test_reset_to_idle_from_completed() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        agent.assign_task("Task".to_string(), None, None).unwrap();
        agent.complete_task("Done".to_string()).unwrap();
        let result = agent.reset_to_idle();

        assert!(result.is_ok());
        assert!(matches!(agent.status(), AgentStatus::Idle));
        assert!(agent.is_available_for_work());
        assert_eq!(agent.context().current_task, None);
    }

    #[test]
    fn test_reset_to_idle_from_failed() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        agent.assign_task("Task".to_string(), None, None).unwrap();
        agent.fail_task("Error".to_string()).unwrap();
        let result = agent.reset_to_idle();

        assert!(result.is_ok());
        assert!(matches!(agent.status(), AgentStatus::Idle));
        assert!(agent.is_available_for_work());
    }

    #[test]
    fn test_reset_to_idle_invalid_state() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        agent.assign_task("Task".to_string(), None, None).unwrap();
        let result = agent.reset_to_idle();

        assert!(matches!(result, Err(AgentError::InvalidTransition { .. })));
        assert!(matches!(agent.status(), AgentStatus::Running));
    }

    #[test]
    fn test_context_management() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        agent.add_context("key1".to_string(), "value1".to_string());
        agent.add_context("key2".to_string(), "value2".to_string());

        assert_eq!(agent.get_context("key1"), Some(&"value1".to_string()));
        assert_eq!(agent.get_context("key2"), Some(&"value2".to_string()));
        assert_eq!(agent.get_context("nonexistent"), None);
    }

    #[test]
    fn test_work_metrics() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        // Start with no tasks
        assert_eq!(agent.total_completed_tasks(), 0);
        assert_eq!(agent.total_failed_tasks(), 0);
        assert_eq!(agent.success_rate(), 0.0);

        // Complete two tasks
        agent.assign_task("Task 1".to_string(), None, None).unwrap();
        agent.complete_task("Done 1".to_string()).unwrap();
        agent.reset_to_idle().unwrap();

        agent.assign_task("Task 2".to_string(), None, None).unwrap();
        agent.complete_task("Done 2".to_string()).unwrap();
        agent.reset_to_idle().unwrap();

        // Fail one task
        agent.assign_task("Task 3".to_string(), None, None).unwrap();
        agent.fail_task("Error".to_string()).unwrap();

        assert_eq!(agent.total_completed_tasks(), 2);
        assert_eq!(agent.total_failed_tasks(), 1);
        assert_eq!(agent.success_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_current_task_duration() {
        let role = create_test_role();
        let mut agent = Agent::new("TestAgent".to_string(), role).unwrap();

        // No current task
        assert!(agent.current_task_duration().is_none());

        // Start task
        agent.assign_task("Task".to_string(), None, None).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let duration = agent.current_task_duration().unwrap();
        assert!(duration.num_milliseconds() >= 0);

        // Complete task
        agent.complete_task("Done".to_string()).unwrap();
        assert!(agent.current_task_duration().is_none()); // No longer current
    }

    #[test]
    fn test_agent_context_builders() {
        let session_id = SessionId::new();
        let workspace = create_test_workspace();

        let context = AgentContext::new()
            .with_task("Test task".to_string())
            .with_session(session_id)
            .with_workspace(workspace.clone());

        assert_eq!(context.current_task, Some("Test task".to_string()));
        assert_eq!(context.session_id, Some(session_id));
        assert_eq!(context.workspace, Some(workspace));
        assert!(context.last_interaction.is_some());
    }

    #[test]
    fn test_work_history_entry_lifecycle() {
        let session_id = SessionId::new();

        // Test successful completion
        let entry = WorkHistoryEntry::new("Test task".to_string(), Some(session_id)).unwrap();
        assert!(matches!(entry.status, WorkEntryStatus::InProgress));
        assert!(entry.duration().is_some());

        let completed = entry.complete("Success".to_string());
        assert!(matches!(completed.status, WorkEntryStatus::Completed));
        assert_eq!(completed.result, Some("Success".to_string()));
        assert!(completed.completed_at.is_some());

        // Test failure
        let entry = WorkHistoryEntry::new("Test task".to_string(), Some(session_id)).unwrap();
        let failed = entry.fail("Error occurred".to_string());
        assert!(matches!(failed.status, WorkEntryStatus::Failed));
        assert_eq!(failed.result, Some("Error occurred".to_string()));

        // Test cancellation
        let entry = WorkHistoryEntry::new("Test task".to_string(), Some(session_id)).unwrap();
        let cancelled = entry.cancel();
        assert!(matches!(cancelled.status, WorkEntryStatus::Cancelled));
        assert!(cancelled.completed_at.is_some());
    }

    #[test]
    fn test_work_history_entry_empty_task() {
        let result = WorkHistoryEntry::new("  ".to_string(), None);
        assert!(matches!(result, Err(AgentError::EmptyWorkHistoryEntry)));
    }

    #[test]
    fn test_agent_serialization() {
        let role = create_test_role();
        let agent = Agent::new("TestAgent".to_string(), role).unwrap();

        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: Agent = serde_json::from_str(&json).unwrap();

        assert_eq!(agent.name(), deserialized.name());
        assert_eq!(agent.status(), deserialized.status());
        assert_eq!(
            agent.work_history().len(),
            deserialized.work_history().len()
        );
    }

    #[test]
    fn test_complex_agent_workflow() {
        let role = create_test_role();
        let mut agent = Agent::new("ComplexAgent".to_string(), role)
            .unwrap()
            .with_capabilities(vec!["api-development".to_string(), "testing".to_string()]);

        let session_id = SessionId::new();
        let workspace = create_test_workspace();

        // Task 1: Success
        agent
            .assign_task(
                "Build user authentication".to_string(),
                Some(session_id),
                Some(workspace.clone()),
            )
            .unwrap();
        agent.add_context("auth_method".to_string(), "jwt".to_string());
        agent
            .complete_task("JWT auth implemented".to_string())
            .unwrap();
        agent.reset_to_idle().unwrap();

        // Task 2: Failure
        agent
            .assign_task(
                "Integrate payment gateway".to_string(),
                Some(session_id),
                Some(workspace.clone()),
            )
            .unwrap();
        agent
            .fail_task("Payment gateway API unreachable".to_string())
            .unwrap();
        agent.reset_to_idle().unwrap();

        // Task 3: Success
        agent
            .assign_task(
                "Write unit tests".to_string(),
                Some(session_id),
                Some(workspace),
            )
            .unwrap();
        agent
            .complete_task("100% test coverage achieved".to_string())
            .unwrap();

        // Verify final state
        assert!(matches!(agent.status(), AgentStatus::Completed));
        assert_eq!(agent.total_completed_tasks(), 2);
        assert_eq!(agent.total_failed_tasks(), 1);
        assert_eq!(agent.success_rate(), 2.0 / 3.0);
        assert_eq!(agent.work_history().len(), 3);
        assert_eq!(agent.current_session(), Some(session_id));
        assert_eq!(agent.get_context("auth_method"), Some(&"jwt".to_string()));
    }
}
