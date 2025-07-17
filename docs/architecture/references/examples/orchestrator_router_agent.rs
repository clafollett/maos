// docs/architecture/references/examples/orchestrator_router_agent.rs
// -----------------------------------------------------------------------------
// Orchestrator Router Agent - Phase-by-Phase Planning
//
// The Orchestrator uses Claude Code to:
// 1. Analyze the user request and plan ONE phase at a time
// 2. Execute that phase with appropriate agents
// 3. Review outputs and discoveries from the phase
// 4. Plan the next phase based on actual learnings
// 5. Continue until the project is complete
//
// This approach ensures each phase builds on real discoveries rather than
// assumptions made during upfront planning.
// -----------------------------------------------------------------------------

use axum::{routing::{get, post}, Json, Router};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

// ===== Core Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Phase {
    name: String,
    number: usize,
    parallel: bool,
    agents: Vec<AgentSpec>,
    rationale: String, // Why this phase is needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentSpec {
    role: String,
    task: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PhaseResult {
    phase_name: String,
    outputs: Vec<String>,
    key_findings: Vec<String>,
    recommendations: Vec<String>,
    blockers: Vec<String>,
}

// ===== Orchestrator Configuration =====

#[derive(Clone)]
struct OrchestratorConfig {
    planning_agent_url: String,
    project_workspace: String,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            planning_agent_url: "http://localhost:8001".into(),
            project_workspace: "./workspace".into(),
        }
    }
}

// ===== Orchestrator =====

struct Orchestrator {
    config: OrchestratorConfig,
    agent_registry: Arc<RwLock<HashMap<String, AgentInfo>>>,
    sessions: Arc<RwLock<HashMap<String, OrchestrationSession>>>,
    http_client: Client,
}

#[derive(Clone)]
struct AgentInfo {
    agent_type: String,
    base_url: String,
    capabilities: Vec<String>,
}

struct OrchestrationSession {
    session_id: String,
    original_request: String,
    current_phase: Option<Phase>,
    phase_history: Vec<(Phase, PhaseResult)>,
    status: SessionStatus,
    orchestrator_session_id: Option<String>, // Orchestrator's own Claude session
    agent_sessions: HashMap<String, String>, // agent_id -> claude_session_id
}

#[derive(Clone, Serialize)]
enum SessionStatus {
    Initializing,
    PlanningPhase,
    ExecutingPhase,
    ReviewingPhase,
    Complete,
    Failed,
}

// ===== Core Orchestrator Implementation =====

impl Orchestrator {
    fn new(config: OrchestratorConfig) -> Self {
        Self {
            config,
            agent_registry: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            http_client: Client::new(),
        }
    }
    
    /// Start a new orchestration session
    async fn orchestrate(&self, user_request: String) -> Result<String, String> {
        let session_id = format!("orch_{}", Uuid::new_v4());
        
        let session = OrchestrationSession {
            session_id: session_id.clone(),
            original_request: user_request.clone(),
            current_phase: None,
            phase_history: Vec::new(),
            status: SessionStatus::Initializing,
            orchestrator_session_id: None,
            agent_sessions: HashMap::new(),
        };
        
        self.sessions.write().await.insert(session_id.clone(), session);
        
        // Start the orchestration loop
        self.orchestration_loop(&session_id).await?;
        
        Ok(session_id)
    }
    
    /// Main orchestration loop - plan phase, execute, review, repeat
    async fn orchestration_loop(&self, session_id: &str) -> Result<(), String> {
        loop {
            // Step 1: Plan next phase based on current context
            let next_phase = self.plan_next_phase(session_id).await?;
            
            if let Some(phase) = next_phase {
                // Step 2: Execute the phase
                self.execute_phase(session_id, &phase).await?;
                
                // Step 3: Review phase outputs
                let phase_result = self.review_phase_outputs(session_id, &phase).await?;
                
                // Step 4: Update session with results
                let mut sessions = self.sessions.write().await;
                if let Some(session) = sessions.get_mut(session_id) {
                    session.phase_history.push((phase.clone(), phase_result));
                    session.current_phase = None;
                }
                drop(sessions);
                
                // Continue to next iteration
                println!("Phase {} complete, planning next phase...", phase.number);
            } else {
                // No more phases needed
                let mut sessions = self.sessions.write().await;
                if let Some(session) = sessions.get_mut(session_id) {
                    session.status = SessionStatus::Complete;
                }
                println!("Orchestration complete!");
                break;
            }
        }
        
        Ok(())
    }
    
    /// Plan the next phase using Claude Code
    async fn plan_next_phase(&self, session_id: &str) -> Result<Option<Phase>, String> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id).ok_or("Session not found")?;
        
        // Build context from all previous phases
        let context = self.build_planning_context(session);
        let phase_number = session.phase_history.len() + 1;
        let orchestrator_session = session.orchestrator_session_id.clone();
        drop(sessions);
        
        let planning_prompt = format!(
            r#"You are the Orchestrator for MAOS. Based on the context below, plan the NEXT SINGLE PHASE of work.

{}

Project Workspace: {}

Analyze what has been done and what still needs to be done. If the project is complete, output:
```json
{{"complete": true}}
```

Otherwise, output the next phase:
```json
{{
  "name": "Descriptive phase name",
  "number": {},
  "parallel": true/false,
  "agents": [
    {{"role": "role_name", "task": "specific task based on learnings"}}
  ],
  "rationale": "Why this phase is needed based on what we've learned"
}}
```

Available roles: researcher, solution_architect, backend_engineer, frontend_engineer, qa_engineer, documenter

IMPORTANT: 
- Plan only ONE phase
- Base tasks on actual discoveries from previous phases
- Be specific about what each agent should do
- Don't repeat completed work

Output ONLY the JSON."#,
            context,
            self.config.project_workspace,
            phase_number
        );
        
        // Call Claude Code Agent for planning
        let planning_request = serde_json::json!({
            "agent_name": "claude-code",
            "input": [{
                "role": "user",
                "parts": [{
                    "content": {
                        "agent_id": "orchestrator",
                        "agent_role": "orchestrator", 
                        "session_id": orchestrator_session,
                        "task": planning_prompt,
                        "context": {
                            "mode": "planning",
                            "phase_number": phase_number,
                        }
                    },
                    "content_type": "application/json"
                }]
            }]
        });
        
        println!("\nPlanning phase {} based on learnings...", phase_number);
        
        let response = self.http_client
            .post(format!("{}/runs", self.config.planning_agent_url))
            .json(&planning_request)
            .send()
            .await
            .map_err(|e| format!("Failed to reach Claude Code: {}", e))?;
        
        let run_info: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        // Save orchestrator's session ID
        if let Some(new_session) = run_info["metadata"]["session_id"].as_str() {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.orchestrator_session_id = Some(new_session.to_string());
            }
        }
        
        // Wait for planning to complete
        let plan_json = self.wait_for_run_completion(
            &run_info["run_id"].as_str().unwrap_or("")
        ).await?;
        
        // Check if project is complete
        if plan_json["complete"].as_bool() == Some(true) {
            return Ok(None);
        }
        
        // Parse the phase
        let phase: Phase = serde_json::from_value(plan_json)
            .map_err(|e| format!("Failed to parse phase: {}", e))?;
        
        println!("Planned: {} ({} agents, {})", 
            phase.name, 
            phase.agents.len(),
            if phase.parallel { "parallel" } else { "sequential" }
        );
        println!("Rationale: {}", phase.rationale);
        
        Ok(Some(phase))
    }
    
    /// Build context string from session history
    fn build_planning_context(&self, session: &OrchestrationSession) -> String {
        let mut context = format!(
            "Original Request: {}\n\n",
            session.original_request
        );
        
        if session.phase_history.is_empty() {
            context.push_str("This is the first phase. No previous work has been done yet.");
        } else {
            context.push_str("Completed Phases:\n");
            for (phase, result) in &session.phase_history {
                context.push_str(&format!("\n--- Phase {}: {} ---\n", phase.number, phase.name));
                
                if !result.key_findings.is_empty() {
                    context.push_str("Key Findings:\n");
                    for finding in &result.key_findings {
                        context.push_str(&format!("- {}\n", finding));
                    }
                }
                
                if !result.recommendations.is_empty() {
                    context.push_str("Recommendations for next steps:\n");
                    for rec in &result.recommendations {
                        context.push_str(&format!("- {}\n", rec));
                    }
                }
                
                if !result.outputs.is_empty() {
                    context.push_str("Deliverables created:\n");
                    for output in &result.outputs {
                        context.push_str(&format!("- {}\n", output));
                    }
                }
            }
        }
        
        context
    }
    
    /// Execute a phase with its agents
    async fn execute_phase(&self, session_id: &str, phase: &Phase) -> Result<(), String> {
        println!("\n=== Executing Phase {}: {} ===", phase.number, phase.name);
        
        // Update session status
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.status = SessionStatus::ExecutingPhase;
                session.current_phase = Some(phase.clone());
            }
        }
        
        if phase.parallel {
            // Run agents in parallel
            let mut handles = vec![];
            
            for (idx, agent) in phase.agents.iter().enumerate() {
                let agent_id = format!("phase{}_{}_{}", phase.number, agent.role, idx);
                let session_id = session_id.to_string();
                let agent = agent.clone();
                let orchestrator = self.clone();
                
                let handle = tokio::spawn(async move {
                    orchestrator.run_agent(&session_id, &agent_id, &agent.role, &agent.task).await
                });
                
                handles.push(handle);
            }
            
            // Wait for all agents to complete
            for handle in handles {
                handle.await.map_err(|e| format!("Agent failed: {}", e))??;
            }
        } else {
            // Run agents sequentially
            for (idx, agent) in phase.agents.iter().enumerate() {
                let agent_id = format!("phase{}_{}_{}", phase.number, agent.role, idx);
                self.run_agent(session_id, &agent_id, &agent.role, &agent.task).await?;
            }
        }
        
        Ok(())
    }
    
    /// Run a single agent
    async fn run_agent(
        &self,
        session_id: &str,
        agent_id: &str,
        role: &str,
        task: &str,
    ) -> Result<(), String> {
        // Get or create Claude session for this agent
        let claude_session = {
            let sessions = self.sessions.read().await;
            sessions.get(session_id)
                .and_then(|s| s.agent_sessions.get(agent_id))
                .cloned()
        };
        
        // Enhanced task with context from previous phases
        let enhanced_task = self.enhance_task_with_context(session_id, task).await?;
        
        // Call Claude Code Agent
        let request = serde_json::json!({
            "agent_name": "claude-code",
            "input": [{
                "role": "user",
                "parts": [{
                    "content": {
                        "agent_id": agent_id,
                        "agent_role": role,
                        "session_id": claude_session,
                        "task": enhanced_task,
                        "context": {
                            "orchestration_session": session_id,
                            "project_workspace": self.config.project_workspace,
                        }
                    },
                    "content_type": "application/json"
                }]
            }]
        });
        
        println!("[{}] Starting: {}", agent_id, 
            if task.len() > 60 { &format!("{}...", &task[..60]) } else { task }
        );
        
        let response = self.http_client
            .post(format!("{}/runs", self.config.planning_agent_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to dispatch agent: {}", e))?;
        
        let run_info: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        // Save agent's session ID
        if let Some(new_session) = run_info["metadata"]["session_id"].as_str() {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.agent_sessions.insert(agent_id.to_string(), new_session.to_string());
            }
        }
        
        // Wait for agent to complete (simplified - real impl would poll/stream)
        println!("[{}] Running...", agent_id);
        
        Ok(())
    }
    
    /// Add context from previous phases to the task
    async fn enhance_task_with_context(&self, session_id: &str, task: &str) -> Result<String, String> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id).ok_or("Session not found")?;
        
        if session.phase_history.is_empty() {
            return Ok(task.to_string());
        }
        
        let mut enhanced = format!("{}\n\nRELEVANT CONTEXT FROM PREVIOUS PHASES:\n", task);
        
        // Add key findings that might be relevant
        for (phase, result) in &session.phase_history {
            if !result.key_findings.is_empty() {
                enhanced.push_str(&format!("\nFrom {} phase:\n", phase.name));
                for finding in &result.key_findings.iter().take(3) {
                    enhanced.push_str(&format!("- {}\n", finding));
                }
            }
        }
        
        enhanced.push_str("\nEnsure your work aligns with these findings.");
        
        Ok(enhanced)
    }
    
    /// Review outputs from a completed phase
    async fn review_phase_outputs(
        &self,
        session_id: &str,
        phase: &Phase,
    ) -> Result<PhaseResult, String> {
        // In real implementation, this would:
        // 1. Collect all agent outputs
        // 2. Use Claude to summarize findings
        // 3. Extract recommendations
        
        // Mock implementation for example
        let mock_findings = match phase.name.as_str() {
            name if name.contains("Research") => vec![
                "Users need OAuth2 authentication".to_string(),
                "Mobile-first design is critical".to_string(),
                "Real-time updates required".to_string(),
            ],
            name if name.contains("Architecture") => vec![
                "Microservices architecture selected".to_string(),
                "PostgreSQL for data persistence".to_string(),
                "Redis for caching layer".to_string(),
            ],
            _ => vec!["Phase completed successfully".to_string()],
        };
        
        Ok(PhaseResult {
            phase_name: phase.name.clone(),
            outputs: vec![format!("{} deliverables created", phase.agents.len())],
            key_findings: mock_findings,
            recommendations: vec!["Continue with implementation".to_string()],
            blockers: vec![],
        })
    }
    
    /// Wait for a run to complete
    async fn wait_for_run_completion(&self, run_id: &str) -> Result<serde_json::Value, String> {
        // Simplified - real implementation would use SSE streaming
        for _ in 0..60 {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            let response = self.http_client
                .get(format!("{}/runs/{}", self.config.planning_agent_url, run_id))
                .send()
                .await
                .map_err(|e| format!("Failed to check status: {}", e))?;
            
            let status: serde_json::Value = response.json().await
                .map_err(|e| format!("Failed to parse status: {}", e))?;
            
            if status["status"].as_str() == Some("completed") {
                if let Some(output) = status["output"].as_array()
                    .and_then(|msgs| msgs.first())
                    .and_then(|msg| msg["parts"].as_array())
                    .and_then(|parts| parts.first())
                    .and_then(|part| part["content"].as_str()) {
                    
                    let json_str = output
                        .trim_start_matches("```json")
                        .trim_end_matches("```")
                        .trim();
                    
                    return serde_json::from_str(json_str)
                        .map_err(|e| format!("Failed to parse JSON: {}", e));
                }
            }
        }
        
        Err("Run timed out".to_string())
    }
    
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            agent_registry: self.agent_registry.clone(),
            sessions: self.sessions.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

// ===== HTTP Handlers =====

async fn agents_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "agents": [{
            "name": "orchestrator",
            "description": "Router agent that plans phases incrementally based on discoveries",
            "version": "1.0.0",
            "capabilities": [
                "phase_by_phase_planning",
                "context_aware_task_assignment", 
                "output_based_learning",
                "agent_coordination"
            ],
            "router": true,
            "stateful": true,
            "input_content_types": ["text/plain", "application/json"],
            "output_content_types": ["application/json"]
        }]
    }))
}

async fn start_orchestration(
    axum::extract::State(orchestrator): axum::extract::State<Arc<Orchestrator>>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, String> {
    let task = request["input"][0]["parts"][0]["content"]
        .as_str()
        .ok_or("No task provided")?;
    
    let session_id = orchestrator.orchestrate(task.to_string()).await?;
    
    Ok(Json(serde_json::json!({
        "run_id": session_id,
        "status": "running",
        "metadata": {
            "type": "orchestration",
            "approach": "phase_by_phase",
            "task": task
        }
    })))
}

// ===== Main =====

#[tokio::main]
async fn main() {
    let config = OrchestratorConfig::default();
    let orchestrator = Arc::new(Orchestrator::new(config));
    
    // Register Claude Code Agent
    orchestrator.agent_registry.write().await.insert(
        "claude-code".into(),
        AgentInfo {
            agent_type: "claude-code".into(),
            base_url: "http://localhost:8001".into(),
            capabilities: vec![
                "solution_architect".into(),
                "backend_engineer".into(), 
                "frontend_engineer".into(),
                "qa_engineer".into(),
                "researcher".into(),
                "documenter".into(),
            ],
        },
    );
    
    let app = Router::new()
        .route("/agents", get(agents_handler))
        .route("/runs", post(start_orchestration))
        .with_state(orchestrator);
    
    let addr = "0.0.0.0:8000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Orchestrator listening on {}", addr);
    println!("Planning phases incrementally based on discoveries!");
    
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}