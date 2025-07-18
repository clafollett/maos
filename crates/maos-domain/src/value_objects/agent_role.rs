use serde::{Deserialize, Serialize};

/// AgentRole value object - represents the role and capabilities of an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentRole {
    pub name: String,
    pub description: String,
    pub responsibilities: String,
    pub is_predefined: bool,
    pub instance_suffix: Option<String>,
}

impl AgentRole {
    pub fn new(
        name: String,
        description: String,
        responsibilities: String,
        is_predefined: bool,
        instance_suffix: Option<String>,
    ) -> Self {
        Self {
            name,
            description,
            responsibilities,
            is_predefined,
            instance_suffix,
        }
    }

    pub fn predefined(name: String, description: String, responsibilities: String) -> Self {
        Self::new(name, description, responsibilities, true, None)
    }

    pub fn custom(name: String, description: String, responsibilities: String) -> Self {
        Self::new(name, description, responsibilities, false, None)
    }
}
