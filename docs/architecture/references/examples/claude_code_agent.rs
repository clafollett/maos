// docs/architecture/references/examples/claude_code_agent.rs
// -----------------------------------------------------------------------------
// Claude Code Agent - ACP Server Implementation
// 
// This agent manages multiple Claude CLI processes with different roles and
// sessions. It implements the full ACP protocol for process lifecycle management.
// -----------------------------------------------------------------------------

use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::RwLock,
    time::interval,
};
use uuid::Uuid;

// ===== Core Types =====

#[derive(Debug, Clone, Serialize)]
struct AgentManifest {
    name: String,
    description: String,
    version: String,
    capabilities: Vec<String>,
    stateful: bool,
    supports_sessions: bool,
    input_content_types: Vec<String>,
    output_content_types: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RunRequest {
    agent_name: String,
    input: Vec<Message>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Message {
    role: String,
    parts: Vec<MessagePart>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct MessagePart {
    content: serde_json::Value,
    content_type: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ClaudeParams {
    agent_id: String,
    agent_role: String,
    session_id: Option<String>,
    task: String,
    context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
struct RunInfo {
    run_id: String,
    status: RunStatus,
    metadata: RunMetadata,
}

#[derive(Debug, Clone, Serialize)]
struct RunMetadata {
    agent_id: String,
    session_id: String,
    process_pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum RunStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

// ===== Claude Code Agent =====

struct ClaudeCodeAgent {
    active_runs: Arc<RwLock<HashMap<String, ClaudeRun>>>,
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
}

struct ClaudeRun {
    run_id: String,
    agent_id: String,
    session_id: String,
    process: Child,
    status: RunStatus,
    output: Arc<RwLock<Vec<String>>>,
}

#[derive(Clone)]
struct SessionInfo {
    session_id: String,
    agent_id: String,
    agent_role: String,
    created_at: chrono::DateTime<chrono::Utc>,
    last_used: chrono::DateTime<chrono::Utc>,
}

impl ClaudeCodeAgent {
    fn new() -> Self {
        Self {
            active_runs: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn start_run(&self, request: RunRequest) -> Result<RunInfo, String> {
        // Parse Claude-specific parameters
        let params = self.parse_claude_params(&request)?;
        
        // Build claude command
        let mut cmd = Command::new("claude");
        cmd.arg("-p").arg(&params.agent_role);
        
        // Add session if continuing work
        let session_id = if let Some(sid) = params.session_id {
            cmd.arg("--session-id").arg(&sid);
            sid
        } else {
            // Generate new session ID
            let new_session = format!("sess_{}", Uuid::new_v4());
            new_session
        };
        
        // Configure process
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        // Spawn the process
        let mut process = cmd.spawn()
            .map_err(|e| format!("Failed to spawn claude: {}", e))?;
        
        let pid = process.id();
        let run_id = format!("run_{}", Uuid::new_v4());
        
        // Set up output streaming
        let output = Arc::new(RwLock::new(Vec::new()));
        if let Some(stdout) = process.stdout.take() {
            let output_clone = output.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    output_clone.write().await.push(line);
                }
            });
        }
        
        // Create run record
        let run = ClaudeRun {
            run_id: run_id.clone(),
            agent_id: params.agent_id.clone(),
            session_id: session_id.clone(),
            process,
            status: RunStatus::Running,
            output,
        };
        
        // Store run
        self.active_runs.write().await.insert(run_id.clone(), run);
        
        // Update session info
        let session_info = SessionInfo {
            session_id: session_id.clone(),
            agent_id: params.agent_id.clone(),
            agent_role: params.agent_role,
            created_at: chrono::Utc::now(),
            last_used: chrono::Utc::now(),
        };
        self.sessions.write().await.insert(session_id.clone(), session_info);
        
        // Send initial task to claude
        if let Some(stdin) = self.active_runs.read().await.get(&run_id)
            .and_then(|r| r.process.stdin.as_ref()) {
            // Write task to stdin (simplified - real impl would be more robust)
            // tokio::io::copy(&mut params.task.as_bytes(), stdin).await.ok();
        }
        
        Ok(RunInfo {
            run_id,
            status: RunStatus::Running,
            metadata: RunMetadata {
                agent_id: params.agent_id,
                session_id,
                process_pid: pid,
            },
        })
    }
    
    fn parse_claude_params(&self, request: &RunRequest) -> Result<ClaudeParams, String> {
        let content = &request.input.first()
            .ok_or("No input provided")?
            .parts.first()
            .ok_or("No message parts")?
            .content;
        
        serde_json::from_value(content.clone())
            .map_err(|e| format!("Invalid Claude parameters: {}", e))
    }
    
    async fn get_run_status(&self, run_id: &str) -> Result<RunInfo, String> {
        let runs = self.active_runs.read().await;
        let run = runs.get(run_id)
            .ok_or_else(|| "Run not found".to_string())?;
        
        Ok(RunInfo {
            run_id: run.run_id.clone(),
            status: run.status.clone(),
            metadata: RunMetadata {
                agent_id: run.agent_id.clone(),
                session_id: run.session_id.clone(),
                process_pid: Some(run.process.id().unwrap_or(0)),
            },
        })
    }
}

// ===== HTTP Handlers =====

async fn agents_handler() -> Json<serde_json::Value> {
    let manifest = AgentManifest {
        name: "claude-code".into(),
        description: "Manages Claude CLI processes for various roles".into(),
        version: "1.0.0".into(),
        capabilities: vec![
            "solution_architect".into(),
            "backend_engineer".into(),
            "frontend_engineer".into(),
            "qa_engineer".into(),
            "researcher".into(),
        ],
        stateful: true,
        supports_sessions: true,
        input_content_types: vec!["application/json".into()],
        output_content_types: vec!["text/plain".into(), "application/json".into()],
    };
    
    Json(serde_json::json!({ "agents": [manifest] }))
}

async fn start_run(
    State(agent): State<Arc<ClaudeCodeAgent>>,
    Json(request): Json<RunRequest>,
) -> Result<Json<RunInfo>, String> {
    let run_info = agent.start_run(request).await?;
    Ok(Json(run_info))
}

async fn get_run(
    State(agent): State<Arc<ClaudeCodeAgent>>,
    Path(run_id): Path<String>,
) -> Result<Json<RunInfo>, String> {
    let run_info = agent.get_run_status(&run_id).await?;
    Ok(Json(run_info))
}

async fn stream_run(
    State(agent): State<Arc<ClaudeCodeAgent>>,
    Path(run_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = async_stream::stream! {
        let mut interval = interval(Duration::from_millis(100));
        
        loop {
            interval.tick().await;
            
            // Get output from run
            if let Ok(runs) = agent.active_runs.read().await {
                if let Some(run) = runs.get(&run_id) {
                    let output = run.output.read().await;
                    for line in output.iter() {
                        yield Ok(Event::default()
                            .event("message")
                            .data(line));
                    }
                }
            }
        }
    };
    
    Sse::new(stream)
}

// ===== Main =====

#[tokio::main]
async fn main() {
    let agent = Arc::new(ClaudeCodeAgent::new());
    
    let app = Router::new()
        .route("/agents", get(agents_handler))
        .route("/runs", post(start_run))
        .route("/runs/:run_id", get(get_run))
        .route("/runs/:run_id/stream", get(stream_run))
        .with_state(agent);
    
    let addr = "0.0.0.0:8001".parse().unwrap();
    println!("Claude Code Agent listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}