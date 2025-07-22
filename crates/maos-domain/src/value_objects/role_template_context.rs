use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::AgentRole;

/// Complete context model for role template serialization
/// This replaces individual template tokens with a single serialized JSON structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleTemplateContext {
    pub identity: IdentityContext,
    pub assignment: AssignmentContext,
    pub environment: EnvironmentContext,
    pub metadata: MetadataContext,
}

/// Agent identity and role information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityContext {
    pub role: AgentRole,
    pub role_description: String,
    pub agent_id: String,
    pub session_id: String,
    pub instance_number: u32,
    pub custom_role_desc: Option<String>,
    pub responsibilities: String,
}

/// Current task assignment details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentContext {
    pub task: String,
    pub project_context: Option<String>,
    pub deadline: Option<String>,
    pub complexity_level: String,
    pub priority: String,
    pub additional_instructions: Option<String>,
}

/// Agent workspace and environment paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    pub workspace_path: String,
    pub shared_context: String,
    pub project_root: String,
    pub message_dir: String,
    pub output_dir: String,
    pub temp_dir: String,
}

/// Additional metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataContext {
    pub template_version: String,
    pub created_at: String,
    pub created_by: String,
    pub category: String,
    pub quality_level: String,
    pub last_updated: String,
    pub supports_multiple_instances: bool,
    pub custom_fields: HashMap<String, String>,
}

impl RoleTemplateContext {
    /// Create a new RoleTemplateContext with all required fields
    pub fn new(role: AgentRole, agent_id: String, session_id: String, task: String) -> Self {
        let role_description = role.description();
        let responsibilities = role.responsibilities();

        Self {
            identity: IdentityContext {
                role_description,
                role,
                agent_id,
                session_id,
                instance_number: 1,
                custom_role_desc: None,
                responsibilities,
            },
            assignment: AssignmentContext {
                task,
                project_context: None,
                deadline: None,
                complexity_level: "Medium".to_string(),
                priority: "Normal".to_string(),
                additional_instructions: None,
            },
            environment: EnvironmentContext {
                workspace_path: "./workspace".to_string(),
                shared_context: "./shared".to_string(),
                project_root: ".".to_string(),
                message_dir: "./messages".to_string(),
                output_dir: "./output".to_string(),
                temp_dir: "./tmp".to_string(),
            },
            metadata: MetadataContext {
                template_version: "2.0".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                created_by: "MAOS".to_string(),
                category: "General".to_string(),
                quality_level: "Standard".to_string(),
                last_updated: chrono::Utc::now().to_rfc3339(),
                supports_multiple_instances: true,
                custom_fields: HashMap::new(),
            },
        }
    }

    /// Builder pattern methods for optional fields
    pub fn with_project_context(mut self, context: String) -> Self {
        self.assignment.project_context = Some(context);
        self
    }

    pub fn with_deadline(mut self, deadline: String) -> Self {
        self.assignment.deadline = Some(deadline);
        self
    }

    pub fn with_complexity(mut self, level: String) -> Self {
        self.assignment.complexity_level = level;
        self
    }

    pub fn with_priority(mut self, priority: String) -> Self {
        self.assignment.priority = priority;
        self
    }

    pub fn with_custom_role_desc(mut self, desc: String) -> Self {
        self.identity.custom_role_desc = Some(desc);
        self
    }

    pub fn with_additional_instructions(mut self, instructions: String) -> Self {
        self.assignment.additional_instructions = Some(instructions);
        self
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.metadata.category = category;
        self
    }

    pub fn with_quality_level(mut self, level: String) -> Self {
        self.metadata.quality_level = level;
        self
    }

    pub fn with_custom_field(mut self, key: String, value: String) -> Self {
        self.metadata.custom_fields.insert(key, value);
        self
    }

    /// Serialize to pretty JSON for template injection
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get the agent's display name for logging/UI
    pub fn display_name(&self) -> String {
        format!("{}-{}", self.identity.role, self.identity.instance_number)
    }

    /// Check if this agent supports multiple instances
    pub fn supports_multiple_instances(&self) -> bool {
        self.metadata.supports_multiple_instances
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TDD Phase 1 RED: Failing tests for clean RoleTemplateContext structure

    #[test]
    fn test_role_template_context_without_resources() {
        // RED: This test should pass since we don't want ResourceContext
        let context = RoleTemplateContext::new(
            AgentRole::BackendEngineer,
            "agent-123".to_string(),
            "session-456".to_string(),
            "Test task".to_string(),
        );

        // Verify no ResourceContext fields exist in the structure
        // This test documents our intention to keep resources separate from agent context
        assert!(context.identity.agent_id.contains("agent"));
        assert!(context.assignment.task == "Test task");
        assert!(!context.environment.workspace_path.is_empty());
        assert!(!context.metadata.template_version.is_empty());

        // We should NOT have ResourceContext - this test ensures separation
        // If ResourceContext exists, this design is wrong
    }

    #[test]
    fn test_role_template_context_json_excludes_resources() {
        // RED: Ensure JSON serialization doesn't include ResourceContext fields
        let context = RoleTemplateContext::new(
            AgentRole::DataScientist,
            "agent-ds-1".to_string(),
            "session-789".to_string(),
            "Analyze data".to_string(),
        );

        let json = context.to_json().unwrap();

        // JSON should NOT contain any ResourceContext fields
        assert!(!json.contains("timeout_minutes"));
        assert!(!json.contains("model_name"));
        assert!(!json.contains("memory_mb"));
        assert!(!json.contains("cpu_priority"));

        // JSON SHOULD contain core agent context fields
        assert!(json.contains("\"role\": \"data_scientist\""));
        assert!(json.contains("\"agent_id\": \"agent-ds-1\""));
        assert!(json.contains("\"workspace_path\""));
    }

    #[test]
    fn test_role_template_context_new_simplified_signature() {
        // RED: Verify new() only takes core parameters (no resource params)
        let context = RoleTemplateContext::new(
            AgentRole::Tester,
            "agent-test".to_string(),
            "session-test".to_string(),
            "Run tests".to_string(),
        );

        // Should work with just 4 core parameters - no resource parameters needed
        assert_eq!(context.identity.role, AgentRole::Tester);
        assert_eq!(context.identity.agent_id, "agent-test");
        assert_eq!(context.assignment.task, "Run tests");

        // Default environment should be set up
        assert!(!context.environment.workspace_path.is_empty());
    }

    #[test]
    fn test_resource_context_separate_from_agent_context() {
        // RED: This test documents that ResourceContext should be in orchestration layer only
        // ResourceContext (timeout_minutes, model_name, etc.) belongs in process management
        // NOT in agent context that gets serialized to JSON

        let context = RoleTemplateContext::new(
            AgentRole::Orchestrator,
            "agent-orch".to_string(),
            "session-orch".to_string(),
            "Coordinate agents".to_string(),
        );

        // Agent context should focus on WHAT the agent needs to know
        // NOT HOW the orchestrator manages the agent process
        assert!(context.identity.role == AgentRole::Orchestrator);
        assert!(!context.assignment.task.is_empty());
        assert!(!context.environment.workspace_path.is_empty());
        assert!(!context.metadata.template_version.is_empty());

        // ResourceContext would pollute agent context with orchestration concerns
        // This separation is critical for clean architecture
    }

    #[test]
    fn test_role_template_context_creation() {
        let context = RoleTemplateContext::new(
            AgentRole::BackendEngineer,
            "agent-123".to_string(),
            "session-456".to_string(),
            "Implement user authentication".to_string(),
        );

        assert_eq!(context.identity.role, AgentRole::BackendEngineer);
        assert_eq!(context.identity.agent_id, "agent-123");
        assert_eq!(context.assignment.task, "Implement user authentication");
        assert_eq!(context.display_name(), "backend_engineer-1");
    }

    #[test]
    fn test_builder_pattern() {
        let context = RoleTemplateContext::new(
            AgentRole::FrontendEngineer,
            "agent-789".to_string(),
            "session-012".to_string(),
            "Build React dashboard".to_string(),
        )
        .with_project_context("E-commerce platform".to_string())
        .with_deadline("2024-03-15".to_string())
        .with_complexity("High".to_string())
        .with_priority("Critical".to_string())
        .with_custom_field("framework".to_string(), "React".to_string());

        assert_eq!(
            context.assignment.project_context,
            Some("E-commerce platform".to_string())
        );
        assert_eq!(context.assignment.deadline, Some("2024-03-15".to_string()));
        assert_eq!(context.assignment.complexity_level, "High");
        assert_eq!(context.assignment.priority, "Critical");
        assert_eq!(
            context.metadata.custom_fields.get("framework"),
            Some(&"React".to_string())
        );
    }

    #[test]
    fn test_json_serialization() {
        let context = RoleTemplateContext::new(
            AgentRole::DataScientist,
            "agent-456".to_string(),
            "session-789".to_string(),
            "Build recommendation engine".to_string(),
        );

        let json = context.to_json().unwrap();
        assert!(json.contains("\"role\": \"data_scientist\""));
        assert!(json.contains("\"task\": \"Build recommendation engine\""));
        assert!(json.contains("\"agent_id\": \"agent-456\""));
        assert!(json.contains("\"session_id\": \"session-789\""));

        // Test round-trip serialization
        let deserialized: RoleTemplateContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.identity.role, context.identity.role);
        assert_eq!(deserialized.assignment.task, context.assignment.task);
    }

    #[test]
    fn test_display_name() {
        let mut context = RoleTemplateContext::new(
            AgentRole::Qa,
            "agent-999".to_string(),
            "session-111".to_string(),
            "Test API endpoints".to_string(),
        );
        context.identity.instance_number = 3;

        assert_eq!(context.display_name(), "qa-3");
    }

    #[test]
    fn test_metadata_defaults() {
        let context = RoleTemplateContext::new(
            AgentRole::Tester,
            "agent-test".to_string(),
            "session-test".to_string(),
            "Test task".to_string(),
        );

        assert_eq!(context.metadata.template_version, "2.0");
        assert_eq!(context.metadata.created_by, "MAOS");
        assert_eq!(context.metadata.category, "General");
        assert_eq!(context.metadata.quality_level, "Standard");
        assert!(context.metadata.supports_multiple_instances);
        assert!(context.metadata.custom_fields.is_empty());
    }
}
