use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Errors for AgentRole operations
#[derive(Debug, Error)]
pub enum AgentRoleError {
    #[error("Unknown agent role: {0}")]
    UnknownRole(String),
    #[error("Role name cannot be empty")]
    EmptyName,
    #[error("Role description cannot be empty")]
    EmptyDescription,
}

/// Predefined agent roles based on MAOS architecture
/// This enum provides type safety and better performance compared to string-based roles
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    // Predefined roles
    Orchestrator,
    SolutionArchitect,
    ApplicationArchitect,
    DataArchitect,
    ApiArchitect,
    SecurityArchitect,
    BackendEngineer,
    FrontendEngineer,
    MobileEngineer,
    Researcher,
    Qa,
    Pm,
    Devops,
    Security,
    DataScientist,
    UxDesigner,
    Documenter,
    Reviewer,
    Analyst,
    Tester,
    // Custom role support
    Custom {
        name: String,
        description: String,
        responsibilities: String,
        instance_suffix: Option<String>,
    },
}

impl AgentRole {
    /// Get the display name for this role
    pub fn name(&self) -> String {
        match self {
            AgentRole::Orchestrator => "orchestrator".to_string(),
            AgentRole::SolutionArchitect => "solution_architect".to_string(),
            AgentRole::ApplicationArchitect => "application_architect".to_string(),
            AgentRole::DataArchitect => "data_architect".to_string(),
            AgentRole::ApiArchitect => "api_architect".to_string(),
            AgentRole::SecurityArchitect => "security_architect".to_string(),
            AgentRole::BackendEngineer => "backend_engineer".to_string(),
            AgentRole::FrontendEngineer => "frontend_engineer".to_string(),
            AgentRole::MobileEngineer => "mobile_engineer".to_string(),
            AgentRole::Researcher => "researcher".to_string(),
            AgentRole::Qa => "qa".to_string(),
            AgentRole::Pm => "pm".to_string(),
            AgentRole::Devops => "devops".to_string(),
            AgentRole::Security => "security".to_string(),
            AgentRole::DataScientist => "data_scientist".to_string(),
            AgentRole::UxDesigner => "ux_designer".to_string(),
            AgentRole::Documenter => "documenter".to_string(),
            AgentRole::Reviewer => "reviewer".to_string(),
            AgentRole::Analyst => "analyst".to_string(),
            AgentRole::Tester => "tester".to_string(),
            AgentRole::Custom {
                name,
                instance_suffix,
                ..
            } => match instance_suffix {
                Some(suffix) => format!("{name}-{suffix}"),
                None => name.clone(),
            },
        }
    }

    /// Get the description for this role
    pub fn description(&self) -> String {
        match self {
            AgentRole::Orchestrator => "Coordinates multi-agent workflows by planning phases, spawning agents, and managing orchestration".to_string(),
            AgentRole::SolutionArchitect => "Designs end-to-end solutions across multiple systems and domains".to_string(),
            AgentRole::ApplicationArchitect => "Designs internal structure and patterns for single applications".to_string(), 
            AgentRole::DataArchitect => "Designs data models, storage systems, and data flow architecture".to_string(),
            AgentRole::ApiArchitect => "Designs API interfaces, service contracts, and integration patterns".to_string(),
            AgentRole::SecurityArchitect => "Designs security controls, threat models, and compliance frameworks".to_string(),
            AgentRole::BackendEngineer => "Implements server-side logic, APIs, and data processing".to_string(),
            AgentRole::FrontendEngineer => "Implements user interfaces and client-side application logic".to_string(),
            AgentRole::MobileEngineer => "Implements mobile applications for iOS, Android, or cross-platform".to_string(),
            AgentRole::Researcher => "Investigates technologies and provides recommendations".to_string(),
            AgentRole::Qa => "Reviews code and specifications for quality".to_string(),
            AgentRole::Pm => "Coordinates agents and tracks progress".to_string(),
            AgentRole::Devops => "Manages infrastructure and deployment".to_string(),
            AgentRole::Security => "Analyzes security vulnerabilities and compliance".to_string(),
            AgentRole::DataScientist => "Analyzes data requirements and develops models".to_string(),
            AgentRole::UxDesigner => "Creates user interface designs and user experiences".to_string(),
            AgentRole::Documenter => "Creates and maintains documentation".to_string(),
            AgentRole::Reviewer => "Reviews code and design decisions".to_string(),
            AgentRole::Analyst => "Analyzes requirements and business logic".to_string(),
            AgentRole::Tester => "Focuses on comprehensive testing strategies".to_string(),
            AgentRole::Custom { description, .. } => description.clone(),
        }
    }

    /// Get detailed responsibilities for this role
    pub fn responsibilities(&self) -> String {
        match self {
            AgentRole::Orchestrator => "Analyze user requests and break them into executable phases; Determine optimal agent roles and task assignments; Plan sequential vs parallel execution strategies; Adapt plans based on phase completion and new information; Coordinate agent communication and dependency management; Handle dynamic re-planning when requirements evolve".to_string(),
            AgentRole::SolutionArchitect => "Design cross-system integration strategies; Select appropriate technologies and platforms; Create enterprise-level solution blueprints; Coordinate multiple system architectures; Ensure solution alignment with business requirements".to_string(),
            AgentRole::ApplicationArchitect => "Design application component architecture; Define internal APIs and service boundaries; Establish application-level design patterns; Create modular application structures; Optimize application performance patterns".to_string(),
            AgentRole::DataArchitect => "Design database schemas and data models; Plan data storage and retrieval strategies; Design data pipelines and ETL processes; Establish data governance and quality standards; Optimize data access patterns".to_string(),
            AgentRole::ApiArchitect => "Design REST/GraphQL API specifications; Define service contracts and interfaces; Establish API governance and standards; Design API versioning and evolution strategies; Create API documentation and integration guides".to_string(),
            AgentRole::SecurityArchitect => "Design security architecture and controls; Perform threat modeling and risk assessment; Define authentication and authorization patterns; Establish security compliance frameworks; Create security integration patterns".to_string(),
            AgentRole::BackendEngineer => "Implement server-side application logic; Build REST/GraphQL APIs and microservices; Design and implement database interactions; Handle authentication, authorization, and security; Create unit and integration tests for backend systems; Optimize server performance and scalability".to_string(),
            AgentRole::FrontendEngineer => "Implement responsive user interfaces; Build client-side application logic and state management; Integrate with APIs and backend services; Ensure cross-browser compatibility and accessibility; Create unit and integration tests for frontend components; Optimize frontend performance and user experience".to_string(),
            AgentRole::MobileEngineer => "Develop native or cross-platform mobile applications; Implement mobile-specific UI patterns and interactions; Handle device capabilities (camera, GPS, sensors); Integrate with mobile backend services and APIs; Optimize for mobile performance and battery life; Create mobile-specific testing strategies".to_string(),
            AgentRole::Researcher => "Research technology options; Evaluate tools and frameworks; Document findings and trade-offs; Provide recommendations; Create proof-of-concepts".to_string(),
            AgentRole::Qa => "Review code for quality and standards; Write and execute test cases; Document bugs and issues; Verify requirements are met; Ensure test coverage".to_string(),
            AgentRole::Pm => "Coordinate between agents; Track task progress; Update project status; Manage dependencies; Ensure timely delivery".to_string(),
            AgentRole::Devops => "Set up CI/CD pipelines; Manage infrastructure as code; Configure deployment environments; Monitor system health; Automate operational tasks".to_string(),
            AgentRole::Security => "Perform security analysis; Identify vulnerabilities; Conduct threat modeling; Recommend security measures; Ensure compliance requirements".to_string(),
            AgentRole::DataScientist => "Analyze data requirements; Develop ML/AI models; Create data pipelines; Provide data insights; Optimize model performance".to_string(),
            AgentRole::UxDesigner => "Design user interfaces; Create user experience flows; Develop design systems; Create mockups and prototypes; Ensure accessibility standards".to_string(),
            AgentRole::Documenter => "Write technical documentation; Create user guides; Maintain API documentation; Document processes and procedures; Ensure documentation accuracy".to_string(),
            AgentRole::Reviewer => "Review code changes; Assess architectural decisions; Provide feedback and suggestions; Ensure coding standards; Validate best practices".to_string(),
            AgentRole::Analyst => "Analyze business requirements; Document use cases; Create process flows; Identify edge cases; Validate business logic".to_string(),
            AgentRole::Tester => "Create test strategies; Execute test plans; Perform various testing types; Track test metrics; Ensure quality standards".to_string(),
            AgentRole::Custom { responsibilities, .. } => responsibilities.clone(),
        }
    }

    /// Get the default timeout in minutes for this role
    pub fn default_timeout_minutes(&self) -> u32 {
        match self {
            AgentRole::Orchestrator => 20,
            AgentRole::Pm | AgentRole::Documenter | AgentRole::Reviewer => 30,
            AgentRole::ApplicationArchitect | AgentRole::ApiArchitect => 35,
            AgentRole::DataArchitect
            | AgentRole::SecurityArchitect
            | AgentRole::Devops
            | AgentRole::Security => 40,
            AgentRole::SolutionArchitect
            | AgentRole::Researcher
            | AgentRole::Qa
            | AgentRole::UxDesigner
            | AgentRole::Analyst
            | AgentRole::Tester => 45,
            AgentRole::BackendEngineer
            | AgentRole::FrontendEngineer
            | AgentRole::MobileEngineer
            | AgentRole::DataScientist => 60,
            AgentRole::Custom { .. } => 45, // Default for custom roles
        }
    }

    /// Get the default memory allocation in MB for this role
    pub fn default_memory_mb(&self) -> u32 {
        match self {
            AgentRole::Pm | AgentRole::Documenter => 1024,
            AgentRole::Orchestrator
            | AgentRole::Researcher
            | AgentRole::UxDesigner
            | AgentRole::Reviewer => 2048,
            AgentRole::FrontendEngineer
            | AgentRole::MobileEngineer
            | AgentRole::Qa
            | AgentRole::Analyst
            | AgentRole::Tester => 3072,
            AgentRole::SolutionArchitect
            | AgentRole::ApplicationArchitect
            | AgentRole::ApiArchitect
            | AgentRole::SecurityArchitect
            | AgentRole::BackendEngineer
            | AgentRole::Devops
            | AgentRole::Security => 4096,
            AgentRole::DataArchitect => 6144,
            AgentRole::DataScientist => 8192,
            AgentRole::Custom { .. } => 2048, // Default for custom roles
        }
    }

    /// Check if this role supports multiple instances
    pub fn supports_multiple_instances(&self) -> bool {
        match self {
            AgentRole::Orchestrator => false, // Only one orchestrator per session
            AgentRole::Pm => false,           // Only one PM per session
            _ => true,
        }
    }

    /// Check if this is a predefined role
    pub fn is_predefined(&self) -> bool {
        !matches!(self, AgentRole::Custom { .. })
    }

    /// Get instance suffix for custom roles
    pub fn instance_suffix(&self) -> Option<&String> {
        match self {
            AgentRole::Custom {
                instance_suffix, ..
            } => instance_suffix.as_ref(),
            _ => None,
        }
    }

    /// Create a predefined role by name
    pub fn predefined(name: &str) -> Result<Self, AgentRoleError> {
        name.parse()
    }

    /// Create a custom role
    pub fn custom(
        name: String,
        description: String,
        responsibilities: String,
    ) -> Result<Self, AgentRoleError> {
        if name.trim().is_empty() {
            return Err(AgentRoleError::EmptyName);
        }
        if description.trim().is_empty() {
            return Err(AgentRoleError::EmptyDescription);
        }

        Ok(AgentRole::Custom {
            name: name.trim().to_string(),
            description: description.trim().to_string(),
            responsibilities: responsibilities.trim().to_string(),
            instance_suffix: None,
        })
    }

    /// Create a role with instance suffix (builder pattern)
    pub fn with_suffix(self, suffix: String) -> Self {
        match self {
            AgentRole::Custom {
                name,
                description,
                responsibilities,
                ..
            } => AgentRole::Custom {
                name,
                description,
                responsibilities,
                instance_suffix: Some(suffix),
            },
            // Predefined roles don't support suffixes in the enum variant
            // but we could handle this differently if needed
            _ => self,
        }
    }

    /// Get all available agent roles
    pub fn all() -> &'static [AgentRole] {
        &[
            AgentRole::Orchestrator,
            AgentRole::SolutionArchitect,
            AgentRole::ApplicationArchitect,
            AgentRole::DataArchitect,
            AgentRole::ApiArchitect,
            AgentRole::SecurityArchitect,
            AgentRole::BackendEngineer,
            AgentRole::FrontendEngineer,
            AgentRole::MobileEngineer,
            AgentRole::Researcher,
            AgentRole::Qa,
            AgentRole::Pm,
            AgentRole::Devops,
            AgentRole::Security,
            AgentRole::DataScientist,
            AgentRole::UxDesigner,
            AgentRole::Documenter,
            AgentRole::Reviewer,
            AgentRole::Analyst,
            AgentRole::Tester,
        ]
    }
}

impl fmt::Display for AgentRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for AgentRole {
    type Err = AgentRoleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "orchestrator" => Ok(AgentRole::Orchestrator),
            "solution_architect" => Ok(AgentRole::SolutionArchitect),
            "application_architect" => Ok(AgentRole::ApplicationArchitect),
            "data_architect" => Ok(AgentRole::DataArchitect),
            "api_architect" => Ok(AgentRole::ApiArchitect),
            "security_architect" => Ok(AgentRole::SecurityArchitect),
            "backend_engineer" => Ok(AgentRole::BackendEngineer),
            "frontend_engineer" => Ok(AgentRole::FrontendEngineer),
            "mobile_engineer" => Ok(AgentRole::MobileEngineer),
            "researcher" => Ok(AgentRole::Researcher),
            "qa" => Ok(AgentRole::Qa),
            "pm" => Ok(AgentRole::Pm),
            "devops" => Ok(AgentRole::Devops),
            "security" => Ok(AgentRole::Security),
            "data_scientist" => Ok(AgentRole::DataScientist),
            "ux_designer" => Ok(AgentRole::UxDesigner),
            "documenter" => Ok(AgentRole::Documenter),
            "reviewer" => Ok(AgentRole::Reviewer),
            "analyst" => Ok(AgentRole::Analyst),
            "tester" => Ok(AgentRole::Tester),
            _ => {
                // For custom roles, we can't parse from string without additional context
                // This is a limitation of the enum approach vs struct approach
                Err(AgentRoleError::UnknownRole(s.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_role_display() {
        assert_eq!(AgentRole::BackendEngineer.to_string(), "backend_engineer");
        assert_eq!(AgentRole::FrontendEngineer.to_string(), "frontend_engineer");
        assert_eq!(AgentRole::Qa.to_string(), "qa");
    }

    #[test]
    fn test_agent_role_from_str() {
        assert_eq!(
            "backend_engineer".parse::<AgentRole>().unwrap(),
            AgentRole::BackendEngineer
        );
        assert_eq!(
            "frontend_engineer".parse::<AgentRole>().unwrap(),
            AgentRole::FrontendEngineer
        );
        assert_eq!("qa".parse::<AgentRole>().unwrap(), AgentRole::Qa);
    }

    #[test]
    fn test_agent_role_from_str_invalid() {
        let result = "invalid_role".parse::<AgentRole>();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unknown agent role: invalid_role"
        );
    }

    #[test]
    fn test_agent_role_properties() {
        assert_eq!(AgentRole::BackendEngineer.default_timeout_minutes(), 60);
        assert_eq!(AgentRole::DataScientist.default_memory_mb(), 8192);
        assert!(!AgentRole::Orchestrator.supports_multiple_instances());
        assert!(AgentRole::BackendEngineer.supports_multiple_instances());
    }

    #[test]
    fn test_agent_role_description() {
        assert!(
            AgentRole::BackendEngineer
                .description()
                .contains("server-side")
        );
        assert!(
            AgentRole::FrontendEngineer
                .description()
                .contains("user interface")
        );
    }

    #[test]
    fn test_all_roles() {
        let all_roles = AgentRole::all();
        assert_eq!(all_roles.len(), 20); // Only predefined roles
        assert!(all_roles.contains(&AgentRole::BackendEngineer));
        assert!(all_roles.contains(&AgentRole::Orchestrator));
        // Custom roles are not included in all()
    }

    #[test]
    fn test_serialization() {
        let role = AgentRole::BackendEngineer;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"backend_engineer\"");

        let deserialized: AgentRole = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, role);
    }

    #[test]
    fn test_round_trip_conversion() {
        for role in AgentRole::all() {
            let name = role.to_string();
            let parsed = name.parse::<AgentRole>().unwrap();
            assert_eq!(*role, parsed);
        }
    }

    #[test]
    fn test_custom_role_creation() {
        let role = AgentRole::custom(
            "custom_analyst".to_string(),
            "Custom analysis role".to_string(),
            "Perform custom analysis tasks".to_string(),
        )
        .unwrap();

        assert_eq!(role.name(), "custom_analyst");
        assert_eq!(role.description(), "Custom analysis role");
        assert_eq!(role.responsibilities(), "Perform custom analysis tasks");
        assert!(!role.is_predefined());
        assert!(role.supports_multiple_instances());
        assert_eq!(role.default_memory_mb(), 2048); // Default for custom
        assert_eq!(role.default_timeout_minutes(), 45); // Default for custom
    }

    #[test]
    fn test_custom_role_validation() {
        // Empty name
        assert!(matches!(
            AgentRole::custom("".to_string(), "desc".to_string(), "resp".to_string()),
            Err(AgentRoleError::EmptyName)
        ));

        // Empty description
        assert!(matches!(
            AgentRole::custom("name".to_string(), "".to_string(), "resp".to_string()),
            Err(AgentRoleError::EmptyDescription)
        ));
    }

    #[test]
    fn test_custom_role_with_suffix() {
        let role = AgentRole::custom(
            "custom_engineer".to_string(),
            "Custom engineering role".to_string(),
            "Custom engineering tasks".to_string(),
        )
        .unwrap()
        .with_suffix("api".to_string());

        assert_eq!(role.name(), "custom_engineer-api");
        assert_eq!(role.instance_suffix(), Some(&"api".to_string()));
    }

    #[test]
    fn test_predefined_role_creation() {
        let role = AgentRole::predefined("backend_engineer").unwrap();
        assert_eq!(role, AgentRole::BackendEngineer);

        assert!(matches!(
            AgentRole::predefined("unknown_role"),
            Err(AgentRoleError::UnknownRole(_))
        ));
    }

    #[test]
    fn test_predefined_vs_custom_properties() {
        let predefined = AgentRole::BackendEngineer;
        let custom = AgentRole::custom(
            "custom_role".to_string(),
            "Custom role".to_string(),
            "Custom responsibilities".to_string(),
        )
        .unwrap();

        assert!(predefined.is_predefined());
        assert!(!custom.is_predefined());
        assert!(predefined.instance_suffix().is_none());
        assert!(custom.instance_suffix().is_none());
    }
}
