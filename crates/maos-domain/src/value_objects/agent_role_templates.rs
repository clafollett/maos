use crate::value_objects::{AgentRole, RoleTemplateContext};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use std::collections::HashMap;

/// Embedded role templates at compile time
#[derive(RustEmbed)]
#[folder = "../../assets/agent-roles/"]
#[exclude = "README.md"]
struct RoleTemplates;

const AGENT_CONTEXT_TOKEN: &str = "{AGENT_CONTEXT}";

/// Template variable substitution context
#[derive(Debug, Clone)]
pub struct TemplateContext {
    // Identity variables
    pub role_name: String,
    pub agent_id: String,
    pub session_id: String,
    pub instance_number: u32,
    pub custom_role_desc: Option<String>,

    // Task variables
    pub task: String,
    pub project_context: Option<String>,
    pub deadline: Option<String>,
    pub complexity_level: Option<String>,
    pub priority: Option<String>,

    // Environment variables
    pub workspace_path: String,
    pub shared_context: String,
    pub message_dir: String,
    pub project_root: String,

    // Resource variables
    pub timeout_minutes: u32,
    pub memory_mb: u32,
    pub model_name: Option<String>,
}

impl TemplateContext {
    pub fn new(
        role_name: String,
        agent_id: String,
        session_id: String,
        instance_number: u32,
        task: String,
        workspace_path: String,
        shared_context: String,
        project_root: String,
    ) -> Self {
        Self {
            role_name,
            agent_id,
            session_id,
            instance_number,
            custom_role_desc: None,
            task,
            project_context: None,
            deadline: None,
            complexity_level: None,
            priority: None,
            workspace_path,
            message_dir: format!("{shared_context}/messages"),
            shared_context,
            project_root,
            timeout_minutes: 45, // Default
            memory_mb: 2048,     // Default
            model_name: None,
        }
    }

    /// Builder pattern methods for optional fields
    pub fn with_custom_role_desc(mut self, desc: String) -> Self {
        self.custom_role_desc = Some(desc);
        self
    }

    pub fn with_project_context(mut self, context: String) -> Self {
        self.project_context = Some(context);
        self
    }

    pub fn with_deadline(mut self, deadline: String) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn with_complexity(mut self, complexity: String) -> Self {
        self.complexity_level = Some(complexity);
        self
    }

    pub fn with_priority(mut self, priority: String) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn with_resources(mut self, timeout_minutes: u32, memory_mb: u32) -> Self {
        self.timeout_minutes = timeout_minutes;
        self.memory_mb = memory_mb;
        self
    }

    pub fn with_model(mut self, model_name: String) -> Self {
        self.model_name = Some(model_name);
        self
    }

    /// Convert TemplateContext to RoleTemplateContext for new template system
    pub fn to_role_template_context(&self, role: &AgentRole) -> RoleTemplateContext {
        RoleTemplateContext::new(
            role.clone(),
            self.agent_id.clone(),
            self.session_id.clone(),
            self.task.clone(),
        )
        .with_project_context(self.project_context.clone().unwrap_or_default())
        .with_deadline(self.deadline.clone().unwrap_or("Not specified".to_string()))
        .with_complexity(
            self.complexity_level
                .clone()
                .unwrap_or("Medium".to_string()),
        )
        .with_priority(self.priority.clone().unwrap_or("Normal".to_string()))
        .with_custom_role_desc(self.custom_role_desc.clone().unwrap_or_default())
    }
}

impl AgentRole {
    /// Get the raw template content for this role
    pub fn raw_template(&self) -> Result<String, TemplateError> {
        let template_filename = match self {
            AgentRole::Orchestrator => "orchestrator.md",
            AgentRole::SolutionArchitect => "solution_architect.md",
            AgentRole::ApplicationArchitect => "application_architect.md",
            AgentRole::DataArchitect => "data_architect.md",
            AgentRole::ApiArchitect => "api_architect.md",
            AgentRole::SecurityArchitect => "security_architect.md",
            AgentRole::BackendEngineer => "backend_engineer.md",
            AgentRole::FrontendEngineer => "frontend_engineer.md",
            AgentRole::MobileEngineer => "mobile_engineer.md",
            AgentRole::Researcher => "researcher.md",
            AgentRole::Qa => "qa.md",
            AgentRole::Pm => "pm.md",
            AgentRole::Devops => "devops.md",
            AgentRole::Security => "security.md",
            AgentRole::DataScientist => "data_scientist.md",
            AgentRole::UxDesigner => "ux_designer.md",
            AgentRole::Documenter => "documenter.md",
            AgentRole::Reviewer => "reviewer.md",
            AgentRole::Analyst => "analyst.md",
            AgentRole::Tester => "tester.md",
            AgentRole::Custom { .. } => {
                return Err(TemplateError::CustomRoleTemplate(
                    "Custom roles must provide their own template content".to_string(),
                ));
            }
        };

        RoleTemplates::get(template_filename)
            .ok_or_else(|| TemplateError::TemplateNotFound(template_filename.to_string()))
            .map(|file| String::from_utf8_lossy(&file.data).to_string())
    }

    /// Get the populated template with RoleTemplateContext JSON injection
    pub fn populated_template_with_context(
        &self,
        template_context: &RoleTemplateContext,
    ) -> Result<String, TemplateError> {
        let template = match self {
            AgentRole::Custom {
                name,
                description,
                responsibilities,
                ..
            } => {
                // For custom roles, create a basic template with {AGENT_CONTEXT} token
                format!(
                    r#"# {name} Agent Template

## Agent Context
```json
{AGENT_CONTEXT_TOKEN}
```

## Role Identity & Mindset
**Role Name**: {name}
**Primary Focus**: {description}
**Problem-Solving Approach**: Custom role implementation

You are a custom agent role with specialized expertise in your defined area.

## Core Responsibilities
{responsibilities}

## Current Assignment
Your current assignment details are defined in the Agent Context JSON above. Apply your expertise to deliver results that meet the specified requirements while following best practices.

## Remember
- Focus on your specialized responsibilities
- Communicate clearly with other agents
- Document your work and decisions
- Ask for clarification when needed

---
*Template Version: 2.0*
*Role Category: Custom*
"#,
                )
            }
            _ => self.raw_template()?,
        };

        // Phase 1: Replace runtime tokens with actual values
        let template_with_tokens_replaced = replace_runtime_tokens(&template, template_context)?;

        // Phase 2: Replace {AGENT_CONTEXT} with JSON
        let context_json = template_context.to_json().map_err(|e| {
            TemplateError::ProcessingError(format!("JSON serialization failed: {e}"))
        })?;
        let result = template_with_tokens_replaced.replace(AGENT_CONTEXT_TOKEN, &context_json);

        Ok(result)
    }

    /// Get the populated template with variable substitution (legacy method)
    pub fn populated_template(&self, context: &TemplateContext) -> Result<String, TemplateError> {
        // Convert to RoleTemplateContext and use the modern approach
        let template_context = context.to_role_template_context(self);
        self.populated_template_with_context(&template_context)
    }

    /// List all available template files
    pub fn available_templates() -> Vec<String> {
        RoleTemplates::iter().map(|s| s.to_string()).collect()
    }

    /// Get all supported runtime tokens
    pub fn get_supported_tokens() -> Vec<String> {
        TOKEN_MAPPINGS
            .iter()
            .map(|(token, _)| token.to_string())
            .collect()
    }

    /// Replace runtime tokens in template content with actual values from context
    pub fn replace_runtime_tokens(
        template_content: &str,
        context: &RoleTemplateContext,
    ) -> Result<String, TemplateError> {
        replace_runtime_tokens(template_content, context)
    }
}

/// Template processing errors
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Custom role must provide template content: {0}")]
    CustomRoleTemplate(String),

    #[error("Unsubstituted variables found: {0:?}")]
    UnsubstitutedVariables(Vec<String>),

    #[error("Template processing error: {0}")]
    ProcessingError(String),

    #[error("Token replacement error: {0}")]
    TokenReplacementError(String),
}

/// Token mappings for runtime token replacement
/// Maps token strings to functions that extract values from RoleTemplateContext
/// This design allows for easy extension of supported tokens
type TokenExtractor = fn(&RoleTemplateContext) -> String;

// Allow complex type for token mappings - this is intentional for extensibility
#[allow(clippy::type_complexity)]
/// Registry of all supported runtime tokens and their extractors
/// New tokens can be added here while maintaining backward compatibility
const TOKEN_MAPPINGS: &[(&str, TokenExtractor)] = &[
    // Environment tokens
    ("{workspace_path}", |ctx| {
        ctx.environment.workspace_path.clone()
    }),
    ("{shared_context}", |ctx| {
        ctx.environment.shared_context.clone()
    }),
    ("{project_root}", |ctx| ctx.environment.project_root.clone()),
    ("{output_dir}", |ctx| ctx.environment.output_dir.clone()),
    ("{temp_dir}", |ctx| ctx.environment.temp_dir.clone()),
    // Identity tokens
    ("{agent_id}", |ctx| ctx.identity.agent_id.clone()),
    ("{session_id}", |ctx| ctx.identity.session_id.clone()),
    ("{instance_number}", |ctx| {
        ctx.identity.instance_number.to_string()
    }),
    // Assignment tokens
    ("{task}", |ctx| ctx.assignment.task.clone()),
    ("{priority}", |ctx| ctx.assignment.priority.clone()),
    ("{complexity_level}", |ctx| {
        ctx.assignment.complexity_level.clone()
    }),
];

/// Compile-time regex for efficient token detection
/// Matches only simple {identifier} patterns, not complex JSON braces
static TOKEN_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)\}").expect("Invalid token regex"));

/// Replace runtime tokens in template content with actual values from context
/// Optimized for performance with pre-compiled regex and efficient replacement
fn replace_runtime_tokens(
    template_content: &str,
    context: &RoleTemplateContext,
) -> Result<String, TemplateError> {
    let mut result = template_content.to_string();

    // Pre-build token values map for efficient lookup
    let token_values: HashMap<String, String> = TOKEN_MAPPINGS
        .iter()
        .map(|(token, extractor)| (token.to_string(), extractor(context)))
        .collect();

    // Replace all known tokens efficiently
    for (token, value) in &token_values {
        result = result.replace(token, value);
    }

    // Use pre-compiled regex to detect any remaining undefined tokens
    let undefined_tokens: Vec<String> = TOKEN_REGEX
        .captures_iter(&result)
        .map(|cap| cap.get(0).unwrap().as_str().to_string())
        .filter(|token| {
            // Allow {AGENT_CONTEXT} as a special case - it's replaced in phase 2
            *token != "{AGENT_CONTEXT}" && !token_values.contains_key(token)
        })
        .collect();

    if !undefined_tokens.is_empty() {
        return Err(TemplateError::TokenReplacementError(format!(
            "Undefined tokens: {}",
            undefined_tokens.join(", ")
        )));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::AgentRole;

    fn create_test_context() -> TemplateContext {
        TemplateContext::new(
            "backend_engineer".to_string(),
            "agent_001".to_string(),
            "session_123".to_string(),
            1,
            "Implement user authentication API".to_string(),
            "/tmp/workspace".to_string(),
            "/tmp/shared".to_string(),
            "/tmp/project".to_string(),
        )
        .with_project_context("E-commerce platform development".to_string())
        .with_deadline("2025-07-25".to_string())
        .with_priority("High".to_string())
        .with_resources(60, 4096)
    }

    #[test]
    fn test_template_context_creation() {
        let context = create_test_context();
        assert_eq!(context.role_name, "backend_engineer");
        assert_eq!(context.agent_id, "agent_001");
        assert_eq!(context.timeout_minutes, 60);
        assert_eq!(context.memory_mb, 4096);
    }

    #[test]
    fn test_role_template_loading() {
        // Test that we can load templates for predefined roles
        let backend_role = AgentRole::BackendEngineer;
        let template_result = backend_role.raw_template();
        assert!(template_result.is_ok());

        let template = template_result.unwrap();
        assert!(template.contains("Backend Engineer"));
        assert!(!template.is_empty());
    }

    #[test]
    fn test_custom_role_template() {
        let custom_role = AgentRole::custom(
            "test_role".to_string(),
            "Test role for testing".to_string(),
            "Test responsibilities".to_string(),
        )
        .unwrap();

        let context = create_test_context();
        let result = custom_role.populated_template(&context);
        assert!(result.is_ok());

        let template = result.unwrap();
        assert!(template.contains("test_role"));
        assert!(template.contains("Test role for testing"));
        assert!(template.contains("Test responsibilities"));
    }

    #[test]
    fn test_available_templates_list() {
        let templates = AgentRole::available_templates();
        assert!(!templates.is_empty());
        assert!(templates.contains(&"backend_engineer.md".to_string()));
        assert!(templates.contains(&"data_scientist.md".to_string()));
        // Should not contain TEMPLATE_USAGE.md (excluded)
        assert!(!templates.contains(&"TEMPLATE_USAGE.md".to_string()));
    }

    #[test]
    fn test_populated_template_data_scientist() {
        let role = AgentRole::DataScientist;
        let context = create_test_context();

        let result = role.populated_template(&context);
        if let Err(e) = &result {
            println!("Template error: {e:?}");
        }
        assert!(result.is_ok());

        let template = result.unwrap();
        // Check that variables were substituted
        assert!(template.contains("agent_001"));
        assert!(template.contains("session_123"));
        assert!(template.contains("Implement user authentication API"));
        assert!(template.contains("2025-07-25"));
        // Check that no unsubstituted variables remain
        assert!(!template.contains("{agent_id}"));
        assert!(!template.contains("{task}"));
    }

    #[test]
    fn test_populated_template_with_role_template_context() {
        use crate::value_objects::RoleTemplateContext;

        let role = AgentRole::ApiArchitect;
        let template_context = RoleTemplateContext::new(
            AgentRole::ApiArchitect,
            "agent_123".to_string(),
            "session_456".to_string(),
            "Design REST API for user management".to_string(),
        )
        .with_project_context("E-commerce platform".to_string())
        .with_deadline("2025-07-30".to_string())
        .with_priority("High".to_string());

        let result = role.populated_template_with_context(&template_context);
        assert!(result.is_ok());

        let template = result.unwrap();
        // Check that Agent Context JSON is included
        assert!(template.contains("\"role\": \"api_architect\""));
        assert!(template.contains("\"agent_id\": \"agent_123\""));
        assert!(template.contains("\"task\": \"Design REST API for user management\""));
        assert!(template.contains("\"priority\": \"High\""));
        assert!(template.contains("\"session_id\": \"session_456\""));

        // Check template structure
        assert!(template.contains("# API Architect Agent Template"));
        assert!(template.contains("## Agent Context"));
        assert!(template.contains("## Role Identity & Mindset"));

        // Should not contain old variable placeholders
        assert!(!template.contains("{AGENT_CONTEXT}"));
    }

    #[test]
    fn test_template_context_to_role_template_context_conversion() {
        let role = AgentRole::BackendEngineer;
        let template_context = create_test_context();

        let role_template_context = template_context.to_role_template_context(&role);

        assert_eq!(
            role_template_context.identity.role,
            AgentRole::BackendEngineer
        );
        assert_eq!(role_template_context.identity.agent_id, "agent_001");
        assert_eq!(
            role_template_context.assignment.task,
            "Implement user authentication API"
        );
        assert_eq!(
            role_template_context.assignment.deadline,
            Some("2025-07-25".to_string())
        );
        assert_eq!(role_template_context.assignment.priority, "High");

        // Test JSON serialization works
        let json_result = role_template_context.to_json();
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert!(json.contains("\"role\": \"backend_engineer\""));
        assert!(json.contains("\"agent_id\": \"agent_001\""));
    }

    // TDD Phase 2 RED: Failing tests for token replacement system

    #[test]
    fn test_token_registry_maps_all_supported_tokens() {
        // RED: This will fail until we implement TOKEN_MAPPINGS
        // This test documents all tokens we want to support

        // Test that our token registry exists and has all expected mappings
        let supported_tokens = AgentRole::get_supported_tokens();

        // Environment tokens
        assert!(supported_tokens.contains(&"{workspace_path}".to_string()));
        assert!(supported_tokens.contains(&"{shared_context}".to_string()));
        assert!(supported_tokens.contains(&"{project_root}".to_string()));
        assert!(supported_tokens.contains(&"{output_dir}".to_string()));
        assert!(supported_tokens.contains(&"{temp_dir}".to_string()));

        // Identity tokens
        assert!(supported_tokens.contains(&"{agent_id}".to_string()));
        assert!(supported_tokens.contains(&"{session_id}".to_string()));
        assert!(supported_tokens.contains(&"{instance_number}".to_string()));

        // Assignment tokens
        assert!(supported_tokens.contains(&"{task}".to_string()));
        assert!(supported_tokens.contains(&"{priority}".to_string()));
        assert!(supported_tokens.contains(&"{complexity_level}".to_string()));

        // Should have all expected tokens (11 total)
        assert!(supported_tokens.len() >= 11);
    }

    #[test]
    fn test_replace_runtime_tokens_workspace_path() {
        // RED: This will fail until we implement replace_runtime_tokens()
        let template_content = "Save file to {workspace_path}/architecture/design.md";
        let context = create_test_context();
        let role = AgentRole::ApiArchitect;
        let role_template_context = context.to_role_template_context(&role);

        // This should fail since replace_runtime_tokens doesn't exist yet
        let result = AgentRole::replace_runtime_tokens(template_content, &role_template_context);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert!(processed.contains("./workspace/architecture/design.md"));
        assert!(!processed.contains("{workspace_path}"));
    }

    #[test]
    fn test_replace_runtime_tokens_agent_id() {
        // RED: This will fail until we implement token replacement
        let template_content = "Agent {agent_id} working on task";
        let context = create_test_context();
        let role = AgentRole::DataScientist;
        let role_template_context = context.to_role_template_context(&role);

        let result = AgentRole::replace_runtime_tokens(template_content, &role_template_context);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert!(processed.contains("Agent agent_001 working on task"));
        assert!(!processed.contains("{agent_id}"));
    }

    #[test]
    fn test_replace_runtime_tokens_multiple_tokens() {
        // RED: Test multiple token replacement in single template
        let template_content =
            "Agent {agent_id} in session {session_id} should save to {workspace_path}/output/";
        let context = create_test_context();
        let role = AgentRole::Tester;
        let role_template_context = context.to_role_template_context(&role);

        let result = AgentRole::replace_runtime_tokens(template_content, &role_template_context);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert!(processed.contains("Agent agent_001"));
        assert!(processed.contains("session session_123"));
        assert!(processed.contains("./workspace/output/"));
        assert!(!processed.contains("{agent_id}"));
        assert!(!processed.contains("{session_id}"));
        assert!(!processed.contains("{workspace_path}"));
    }

    #[test]
    fn test_replace_runtime_tokens_missing_token_error() {
        // RED: Test error handling for undefined tokens
        let template_content = "Invalid token: {undefined_token}";
        let context = create_test_context();
        let role = AgentRole::Orchestrator;
        let role_template_context = context.to_role_template_context(&role);

        let result = AgentRole::replace_runtime_tokens(template_content, &role_template_context);

        // Should return an error for undefined tokens
        assert!(result.is_err());
    }

    #[test]
    fn test_two_phase_processing() {
        // Test complete two-phase processing: tokens first, then {AGENT_CONTEXT}
        // This test uses the actual Solution Architect template which has {AGENT_CONTEXT}
        let context = create_test_context();
        let role = AgentRole::SolutionArchitect;
        let role_template_context = context.to_role_template_context(&role);

        // This should process both runtime tokens AND {AGENT_CONTEXT}
        let result = role.populated_template_with_context(&role_template_context);
        if let Err(ref e) = result {
            eprintln!("Two-phase processing error: {e:?}");
        }
        assert!(result.is_ok());

        let processed = result.unwrap();

        // {AGENT_CONTEXT} should be replaced with JSON
        assert!(processed.contains("\"agent_id\": \"agent_001\""));
        assert!(processed.contains("\"role\": \"solution_architect\""));
        assert!(!processed.contains("{AGENT_CONTEXT}"));

        // Template should contain the expected content
        assert!(processed.contains("Solution Architect Agent Template"));
        assert!(processed.contains("End-to-end solution design"));
    }
}
