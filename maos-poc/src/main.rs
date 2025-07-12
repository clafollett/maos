use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct ExecutionPlan {
    phases: Vec<Phase>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Phase {
    name: String,
    parallel: bool,
    agents: Vec<AgentSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentSpec {
    role: String,
    task: String,
}

#[derive(Debug, Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(flatten)]
    data: Value,
}

/// Simple POC to demonstrate the Orchestrator concept
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Set up workspace in target/tmp
    let workspace_root = setup_workspace().await?;
    println!("MAOS POC - Orchestrator Demo");
    println!("Workspace: {}\n", workspace_root.display());

    // Example user request
    let user_request = "Research and build a secure ERP web application using Vue.js and Nuxt for Muralists and Painters to manage their projects, customers, paints, tools, supplies, and inventory.";
    println!("User request: {}\n", user_request);

    // Step 1: Spawn Orchestrator agent
    println!("Spawning Orchestrator agent...");
    let plan = spawn_orchestrator(user_request).await?;

    // Step 2: Execute the plan
    println!("\nExecuting plan...");
    execute_plan(plan, &workspace_root).await?;

    println!("\n✅ Orchestration complete!");
    println!(
        "Check the generated project: {}",
        workspace_root.join("project").display()
    );
    println!(
        "Agent outputs: {}",
        workspace_root.join("shared_context").display()
    );

    Ok(())
}

async fn setup_workspace() -> Result<PathBuf> {
    // Get the target directory relative to the binary
    let exe_path = std::env::current_exe()?;
    let target_dir = exe_path
        .parent() // directory containing the binary
        .and_then(|p| p.parent()) // debug or release
        .and_then(|p| p.parent()) // target
        .ok_or_else(|| anyhow::anyhow!("Could not find target directory"))?;

    let workspace_root = target_dir.join("target").join("tmp").join("maos-workspace");

    // Create workspace directories
    fs::create_dir_all(&workspace_root).await?;
    fs::create_dir_all(workspace_root.join("shared_context")).await?;
    fs::create_dir_all(workspace_root.join("agents")).await?;
    fs::create_dir_all(workspace_root.join("messages")).await?;
    fs::create_dir_all(workspace_root.join("project")).await?; // Shared project directory

    Ok(workspace_root)
}

async fn spawn_orchestrator(user_request: &str) -> Result<ExecutionPlan> {
    let prompt = format!(
        r#"You are the Orchestrator agent for MAOS. Analyze this request and create an execution plan.

User Request: {user_request}

Output a JSON execution plan with phases and agents. Available roles: researcher, architect, engineer, qa_engineer, documenter.

Example format:
```
{{
  "phases": [
    {{
      "name": "Research and Design",
      "parallel": false,
      "agents": [
        {{"role": "researcher", "task": "Research best practices for todo APIs"}},
        {{"role": "architect", "task": "Design API structure"}}
      ]
    }},
    {{
      "name": "Implementation",
      "parallel": true,
      "agents": [
        {{"role": "engineer", "task": "Implement API endpoints"}},
        {{"role": "engineer", "task": "Set up database"}},
        {{"role": "qa_engineer", "task": "Write tests"}}
      ]
    }}
  ]
}}
```

Output ONLY the JSON, no other text."#
    );

    println!("Orchestrator analyzing request...");

    // Use regular JSON output format for the orchestrator (simpler and works)
    let cmd = Command::new("claude")
        .arg("-p")
        .arg(&prompt)
        .arg("--output-format")
        .arg("json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Read the JSON output
    let output = cmd.wait_with_output().await?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        println!("Claude not available: {}", error_msg);
        return Err(anyhow::anyhow!("Claude not available"));
    }

    let json_output: Value = serde_json::from_slice(&output.stdout)?;

    // Debug: show what we got
    println!(
        "JSON envelope from Claude: {}",
        serde_json::to_string_pretty(&json_output)?
    );

    // Check if this was an error
    if json_output["subtype"].as_str() == Some("error_during_execution") {
        return Err(anyhow::anyhow!("Claude error during execution"));
    }

    // The field is called "result" based on your test
    let response_text = json_output["result"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No 'result' field in JSON output"))?;

    // Claude's response should contain JSON. Let's try to parse it directly first
    let response_text = response_text
        .trim_start_matches("```json\n")
        .trim_end_matches("\n```");

    println!("Got response from Claude!");
    println!("Claude's response text: {}", response_text);

    match serde_json::from_str::<ExecutionPlan>(response_text) {
        Ok(plan) => {
            println!(
                "Orchestrator created plan with {} phases",
                plan.phases.len()
            );
            return Ok(plan);
        }
        Err(_) => Err(anyhow::anyhow!("Failed to parse JSON from Claude response")),
    }
}

async fn execute_plan(plan: ExecutionPlan, workspace_root: &PathBuf) -> Result<()> {
    for (idx, phase) in plan.phases.iter().enumerate() {
        println!("\n=== Phase {}: {} ===", idx + 1, phase.name);

        if phase.parallel {
            println!("Executing {} agents in parallel...", phase.agents.len());

            // Spawn all agents in parallel
            let mut handles = vec![];
            for (i, agent) in phase.agents.iter().enumerate() {
                let agent = agent.clone();
                let agent_workspace = workspace_root
                    .join("agents")
                    .join(format!("{}_{}", agent.role, i));
                let shared_context = workspace_root.join("shared_context");
                let project_dir = workspace_root.join("project");

                let handle = tokio::spawn(async move {
                    spawn_agent(
                        &agent.role,
                        &agent.task,
                        &agent_workspace,
                        &shared_context,
                        &project_dir,
                    )
                    .await
                });
                handles.push(handle);
            }

            // Wait for all to complete
            for handle in handles {
                handle.await??;
            }
        } else {
            println!("Executing {} agents sequentially...", phase.agents.len());

            for (i, agent) in phase.agents.iter().enumerate() {
                let agent_workspace = workspace_root
                    .join("agents")
                    .join(format!("{}_{}", agent.role, i));
                let shared_context = workspace_root.join("shared_context");
                let project_dir = workspace_root.join("project");
                spawn_agent(
                    &agent.role,
                    &agent.task,
                    &agent_workspace,
                    &shared_context,
                    &project_dir,
                )
                .await?;
            }
        }
    }

    Ok(())
}

async fn spawn_agent(
    role: &str,
    task: &str,
    workspace: &PathBuf,
    shared_context: &PathBuf,
    project_dir: &PathBuf,
) -> Result<()> {
    let start = std::time::Instant::now();
    println!("\n  [{:>12}] Starting: {}", role, task);

    // Create agent workspace
    fs::create_dir_all(workspace).await?;

    let messages_dir = workspace.parent().unwrap().parent().unwrap().join("messages");
    
    // Add role-specific guidance
    let role_guidance = match role {
        "researcher" => "\n         RESEARCHER SPECIFIC:\n\
         - Place all research documents in project/docs/research/\n\
         - Name files descriptively (e.g., rest-api-best-practices.md)\n\
         - Create an index or summary document\n",
        "architect" => "\n         ARCHITECT SPECIFIC:\n\
         - Place design documents in project/docs/design/\n\
         - Place API specifications in project/docs/api/\n\
         - Architecture decision records go in project/docs/adr/\n",
        "engineer" => "\n         ENGINEER SPECIFIC:\n\
         - Source code goes in project/src/ (or appropriate source directory)\n\
         - Follow the project structure from architect's design\n\
         - Configuration files in project root\n",
        "qa_engineer" => "\n         QA ENGINEER SPECIFIC:\n\
         - Place tests in the appropriate location for the project language/framework\n\
         - Common patterns: tests/, test/, src/test/, or alongside source files\n\
         - Test documentation in project/docs/testing/\n",
        "documenter" => "\n         DOCUMENTER SPECIFIC:\n\
         - User documentation in project/docs/\n\
         - API documentation in project/docs/api/\n\
         - README.md in project root\n",
        _ => "",
    };
    
    let prompt = format!(
        "You are a {} agent in the MAOS system. Your task: {}\n\n\
         Project directory (WORK HERE): {}\n\
         Your private workspace: {}\n\
         Shared context: {}\n\
         Messages from other agents: {}\n\n\
         CRITICAL INSTRUCTIONS:\n\
         1. Work in the PROJECT DIRECTORY - this is where all agents collaborate\n\
         2. Check the shared context for other agents' summaries: ls {}\n\
         3. Check messages directory for updates from other agents: ls {}\n\
         4. Create all code/files in the project directory, not your private workspace\n\
         5. Your private workspace is only for temporary files or notes\n\
         6. Write a summary of your work to the shared context when done\n\n\
         FILE ORGANIZATION RULES:\n\
         - Documentation files (.md, .txt): Place in project/docs/ directory\n\
         - API specs (openapi.yaml, swagger.json): Place in project root or project/docs/\n\
         - Source code: Place in appropriate directory for the language/framework\n\
         - Tests: Place in appropriate test directory for the language/framework\n\
         - Configuration files: Place in project root\n\
         - Create directories if they don't exist yet\n\
{}\n\
         You are working with other agents simultaneously. Be aware that:\n\
         - Other agents may be creating files in the project directory\n\
         - Check existing project structure before creating new files\n\
         - Read messages from completed agents to understand their work\n\
         - Coordinate through the shared context summaries\n\n\
         Focus on creating actual working code and artifacts in the project directory.",
        role,
        task,
        project_dir.display(),
        workspace.display(),
        shared_context.display(),
        messages_dir.display(),
        shared_context.display(),
        messages_dir.display(),
        role_guidance
    );

    // Spawn Claude with streaming JSON output (requires --verbose with -p)
    let mut cmd = Command::new("claude")
        .arg("-p")
        .arg(&prompt)
        .arg("--verbose")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--add-dir")
        .arg(project_dir.as_os_str())
        .arg("--add-dir")
        .arg(shared_context.as_os_str())
        .arg("--add-dir")
        .arg(messages_dir.as_os_str())
        .arg("--dangerously-skip-permissions")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = cmd.stdout.take().expect("Failed to capture stdout");
    let stderr = cmd.stderr.take().expect("Failed to capture stderr");

    // Read streaming JSON events
    let role_clone = role.to_string();
    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        let mut full_response = String::new();
        let mut event_count = 0;
        let mut session_id = None;
        let mut last_progress_time = std::time::Instant::now();
        let mut tool_count = 0;

        while let Some(line) = reader.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse as JSON event
            if let Ok(event) = serde_json::from_str::<StreamEvent>(&line) {
                event_count += 1;

                match event.event_type.as_str() {
                    "system" => {
                        // Capture session ID from init message
                        if event.data.get("subtype").and_then(|s| s.as_str()) == Some("init") {
                            session_id = event
                                .data
                                .get("session_id")
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_string());
                            if let Some(sid) = &session_id {
                                println!("  [{:>12}] Session ID: {}", role_clone, sid);
                            }
                        }
                    }
                    "assistant" => {
                        // Extract content from the message
                        if let Some(message) = event.data.get("message") {
                            if let Some(content) = message.get("content").and_then(|c| c.as_array())
                            {
                                for item in content {
                                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                        full_response.push_str(text);
                                        // Add newline after each text block to preserve formatting
                                        if !text.ends_with('\n') {
                                            full_response.push('\n');
                                        }

                                        // Show progress for interesting content
                                        if text.contains("Creating")
                                            || text.contains("Writing")
                                            || text.contains("Generated")
                                            || text.contains("File:")
                                        {
                                            let preview = if text.len() > 60 {
                                                format!("{}...", &text[..60])
                                            } else {
                                                text.to_string()
                                            };
                                            println!("  [{:>12}] > {}", role_clone, preview.trim());
                                        }
                                    } else if let Some(tool_use) = item.get("name") {
                                        let tool_name = tool_use.as_str().unwrap_or("unknown");
                                        tool_count += 1;
                                        println!(
                                            "  [{:>12}] Using tool: {} (#{} tools used)",
                                            role_clone, tool_name, tool_count
                                        );
                                        
                                        // Show periodic progress updates
                                        if last_progress_time.elapsed().as_secs() >= 30 {
                                            println!(
                                                "  [{:>12}] 🔄 Still working... ({} tools used so far)",
                                                role_clone, tool_count
                                            );
                                            last_progress_time = std::time::Instant::now();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "result" => {
                        // Final message, we can ignore or log completion
                        if event.data.get("subtype").and_then(|s| s.as_str()) == Some("success") {
                            // Success - response is complete
                        }
                    }
                    _ => {
                        if event_count % 5 == 0 {
                            use std::io::Write;
                            std::io::stdout().flush()?;
                        }
                    }
                }
            }
        }

        if event_count % 10 != 0 {
            println!(); // New line after dots
        }

        Ok::<(String, Option<String>), anyhow::Error>((full_response, session_id))
    });

    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        let mut errors = Vec::new();

        while let Some(line) = reader.next_line().await? {
            errors.push(line);
        }
        Ok::<Vec<String>, anyhow::Error>(errors)
    });

    // Set a timeout for the agent
    let timeout = tokio::time::timeout(
        std::time::Duration::from_secs(1800), // 30 minutes max
        cmd.wait(),
    );

    match timeout.await {
        Ok(Ok(status)) => {
            let (full_response, session_id) = stdout_task.await??;
            let errors = stderr_task.await??;

            if status.success() {
                // Save session ID for potential resumption
                if let Some(sid) = &session_id {
                    let session_file = workspace.join("session_id.txt");
                    fs::write(&session_file, sid).await?;
                }

                // Save agent output to workspace
                let output_file = workspace.join("agent_output.md");
                fs::write(&output_file, &full_response).await?;

                // Also save to shared context for other agents
                // Extract agent ID from workspace path (e.g., "engineer_0" -> "0")
                let agent_id = workspace
                    .file_name()
                    .and_then(|n| n.to_str())
                    .and_then(|n| n.split('_').last())
                    .unwrap_or("0");

                // Use actual session ID from Claude (or fallback to timestamp)
                let session_id_short = if let Some(sid) = &session_id {
                    // Take first 8 chars of the session ID
                    sid.chars().take(8).collect::<String>()
                } else {
                    // Fallback to timestamp-based ID
                    format!(
                        "{:x}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    )[..8]
                        .to_string()
                };

                let shared_file =
                    shared_context.join(format!("{}_{}_{}.md", role, agent_id, session_id_short));
                fs::write(&shared_file, &full_response).await?;
                
                // Create a structured message for other agents
                let message = format!(
                    "# Agent: {} (ID: {})\n## Session: {}\n## Status: Completed\n\n{}",
                    role, agent_id, session_id_short, full_response
                );
                let message_file = workspace.parent().unwrap().parent().unwrap()
                    .join("messages")
                    .join(format!("{}_{}_completed.md", role, agent_id));
                fs::write(&message_file, &message).await?;

                // Log a summary
                println!(
                    "  [{:>12}] Summary saved to: {}",
                    role,
                    shared_file.file_name().unwrap().to_str().unwrap()
                );

                // Check if agent created any files
                if let Ok(mut entries) = fs::read_dir(workspace).await {
                    let mut files = Vec::new();
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        if let Ok(name) = entry.file_name().into_string() {
                            files.push(name);
                        }
                    }

                    if !files.is_empty() {
                        println!("  [{:>12}] Workspace files: {:?}", role, files);
                    }
                }

                // Check if agent created any files in project directory
                if let Ok(mut entries) = fs::read_dir(project_dir).await {
                    let mut project_files = Vec::new();
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        if let Ok(name) = entry.file_name().into_string() {
                            project_files.push(name);
                        }
                    }

                    if !project_files.is_empty() {
                        println!("  [{:>12}] PROJECT FILES: {:?}", role, project_files);
                    } else {
                        println!(
                            "  [{:>12}] ⚠️  No files created in project directory!",
                            role
                        );
                    }
                }

                // Check shared context updates
                if let Ok(mut entries) = fs::read_dir(shared_context).await {
                    let mut shared_files = Vec::new();
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        if let Ok(name) = entry.file_name().into_string() {
                            if name.contains(role) {
                                shared_files.push(name);
                            }
                        }
                    }

                    if !shared_files.is_empty() {
                        println!("  [{:>12}] Shared outputs: {:?}", role, shared_files);
                    }
                }

                let elapsed = start.elapsed();
                println!(
                    "  [{:>12}] ✓ Completed in {:.1}s",
                    role,
                    elapsed.as_secs_f32()
                );
                Ok(())
            } else {
                let error_msg = errors.join("\n");
                Err(anyhow::anyhow!("Agent failed: {}", error_msg))
            }
        }
        Ok(Err(e)) => Err(anyhow::anyhow!("Agent process error: {}", e)),
        Err(_) => {
            // Try to kill the process
            let _ = cmd.kill().await;
            
            // Save partial progress for potential resumption
            if let Ok(Ok((partial_response, session_id))) = stdout_task.await {
                if let Some(sid) = session_id {
                    let resume_file = workspace.join("resume_info.json");
                    let resume_info = serde_json::json!({
                        "session_id": sid,
                        "role": role,
                        "task": task,
                        "status": "timeout",
                        "partial_response": partial_response
                    });
                    let _ = fs::write(&resume_file, serde_json::to_string_pretty(&resume_info)?).await;
                    println!("  [{:>12}] ⚠️  Timeout - session saved for resumption: {}", role, sid);
                }
            }
            
            Err(anyhow::anyhow!("Agent timed out after 30 minutes"))
        }
    }
}
