// docs/architecture/references/examples/intelligent_agent_selection.rs
// -----------------------------------------------------------------------------
// Intelligent Agent Selection Using Claude
// 
// Shows how the Orchestrator uses Claude to make smart decisions about
// which agent should handle a task based on the session registry.
// -----------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
struct SessionEntry {
    agent_id: String,
    session_id: String,
    role: String,
    work_context: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AgentDecision {
    agent_id: Option<String>,     // Existing agent ID or None for new
    create_new: bool,             // Whether to create a new agent
    reasoning: String,            // Claude's reasoning for the decision
    suggested_context: String,    // Context to record for this work
}

struct IntelligentOrchestrator {
    registry: Vec<SessionEntry>,
    orchestrator_session_id: String,  // Orchestrator's own Claude session
}

impl IntelligentOrchestrator {
    async fn select_agent_for_task(&self, role: &str, task: &str) -> Result<AgentDecision, String> {
        // Build context for Claude
        let registry_summary = self.build_registry_summary();
        
        let decision_prompt = format!(
            r#"You are the Orchestrator managing a team of AI agents. 
            
Current Agent Registry:
{}

New Task Assignment:
- Required Role: {}
- Task Description: {}

Analyze which agent should handle this task. Consider:
1. Which agents have worked on related components?
2. Would reusing an existing agent provide valuable context?
3. Are there dependency relationships between components?
4. Is this task sufficiently different to warrant a new specialist?

Output JSON:
{{
  "agent_id": "backend_eng_1" or null,
  "create_new": true/false,
  "reasoning": "explanation of your decision",
  "suggested_context": "brief description of component/area"
}}"#,
            registry_summary, role, task
        );
        
        // Call Claude Code Agent with orchestrator's session
        let response = self.call_claude_for_decision(&decision_prompt).await?;
        
        // Parse Claude's decision
        let decision: AgentDecision = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse decision: {}", e))?;
            
        Ok(decision)
    }
    
    fn build_registry_summary(&self) -> String {
        self.registry.iter()
            .map(|entry| {
                format!("{}: {} - worked on: {}", 
                    entry.agent_id,
                    entry.role,
                    entry.work_context.join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    async fn call_claude_for_decision(&self, prompt: &str) -> Result<String, String> {
        // This would call Claude Code Agent with the orchestrator's session
        // Using the same pattern as agent execution
        let request = serde_json::json!({
            "agent_name": "claude-code",
            "input": [{
                "role": "user",
                "parts": [{
                    "content": {
                        "agent_id": "orchestrator",
                        "agent_role": "orchestrator",
                        "session_id": self.orchestrator_session_id,
                        "task": prompt,
                        "context": {
                            "mode": "agent_selection",
                        }
                    },
                    "content_type": "application/json"
                }]
            }]
        });
        
        // Simplified - actual implementation would make HTTP request
        Ok(r#"{"agent_id": "backend_eng_1", "create_new": false, "reasoning": "backend_eng_1 has already worked on auth_service and this task involves extending authentication", "suggested_context": "auth_2fa"}"#.to_string())
    }
}

// Example usage
async fn example_smart_assignment() {
    let orchestrator = IntelligentOrchestrator {
        registry: vec![
            SessionEntry {
                agent_id: "backend_eng_1".to_string(),
                session_id: "session_def456".to_string(),
                role: "backend_eng".to_string(),
                work_context: vec!["auth_service".to_string(), "oauth_impl".to_string()],
            },
            SessionEntry {
                agent_id: "backend_eng_2".to_string(),
                session_id: "session_ghi789".to_string(),
                role: "backend_eng".to_string(),
                work_context: vec!["user_service".to_string(), "crud_apis".to_string()],
            },
        ],
        orchestrator_session_id: "session_orch123".to_string(),
    };
    
    // Need to add 2FA to auth - Claude will intelligently pick backend_eng_1
    let decision = orchestrator.select_agent_for_task(
        "backend_eng",
        "Add two-factor authentication support to the auth service"
    ).await.unwrap();
    
    println!("Decision: Use agent {} because: {}", 
             decision.agent_id.unwrap_or("NEW".to_string()),
             decision.reasoning);
}