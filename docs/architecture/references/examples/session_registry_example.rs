// docs/architecture/references/examples/session_registry_example.rs
// -----------------------------------------------------------------------------
// Session Registry Example - Intelligent Agent Selection
// 
// Shows how the Orchestrator maintains a registry of agent sessions and
// intelligently selects or creates agents based on work context.
// -----------------------------------------------------------------------------

use std::collections::HashMap;

#[derive(Debug, Clone)]
struct SessionEntry {
    agent_id: String,
    session_id: String,
    role: String,
    work_context: Vec<String>,
}

struct SessionRegistry {
    entries: Vec<SessionEntry>,
    next_ordinal: HashMap<String, u32>,
}

impl SessionRegistry {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_ordinal: HashMap::new(),
        }
    }
    
    /// Find an agent that matches the role and has worked on related context
    fn find_agent(&self, role: &str, context: &str) -> Option<&SessionEntry> {
        self.entries.iter()
            .filter(|e| e.role == role)
            .find(|e| e.work_context.iter().any(|ctx| ctx.contains(context)))
    }
    
    /// Create a new agent with ordinal ID
    fn create_agent(&mut self, role: &str, session_id: String, initial_context: String) -> String {
        let ordinal = self.next_ordinal.entry(role.to_string()).or_insert(1);
        let agent_id = format!("{}_{}", role, ordinal);
        *ordinal += 1;
        
        self.entries.push(SessionEntry {
            agent_id: agent_id.clone(),
            session_id,
            role: role.to_string(),
            work_context: vec![initial_context],
        });
        
        agent_id
    }
    
    /// Update agent's work context after completing a task
    fn update_context(&mut self, agent_id: &str, new_context: String) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.agent_id == agent_id) {
            entry.work_context.push(new_context);
        }
    }
}

// Example usage in Orchestrator
async fn orchestrator_example() {
    let mut registry = SessionRegistry::new();
    
    // Phase 1: Initial backend work
    let auth_engineer = registry.create_agent(
        "backend_eng",
        "session_def456".to_string(),
        "auth_service".to_string()
    );
    println!("Created {} for auth service", auth_engineer);
    
    // Phase 3: Need auth updates - finds existing engineer
    if let Some(engineer) = registry.find_agent("backend_eng", "auth") {
        println!("Reusing {} (session {}) for auth updates", 
                 engineer.agent_id, engineer.session_id);
        // Use existing session_id for context continuity
    }
    
    // Phase 4: Need user service work - no match, create new
    if registry.find_agent("backend_eng", "user").is_none() {
        let user_engineer = registry.create_agent(
            "backend_eng",
            "session_ghi789".to_string(),
            "user_service".to_string()
        );
        println!("Created {} for user service", user_engineer);
    }
    
    // Update context after work completion
    registry.update_context("backend_eng_1", "oauth_implementation".to_string());
    registry.update_context("backend_eng_1", "jwt_tokens".to_string());
}