# ADR-008: Agent Communication Patterns

## Status
Accepted

## Context
Agents in MAOS need to communicate and coordinate their work effectively. Since each agent runs as a separate CLI process, we need well-defined patterns for:

- Sharing work artifacts (specifications, code, test results)
- Sending messages between agents
- Coordinating dependencies and handoffs
- Broadcasting status updates
- Requesting help or clarification

Key constraints:
- Agents are separate OS processes (not threads)
- Communication must work across different CLI tools
- Need both synchronous and asynchronous patterns
- Must maintain agent isolation for stability

## Decision
We will implement a hybrid communication system using:
1. **Shared file system** for artifacts and persistent data
2. **Message files** for inter-agent messaging
3. **JSON stdout** for status updates to MAOS
4. **Environment variables** for configuration

### Communication Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    MAOS Server                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │           Message Router & Monitor                │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                           │
    ┌──────────────────────┼──────────────────────┐
    │                      │                      │
    ▼                      ▼                      ▼
┌─────────┐          ┌─────────┐          ┌─────────┐
│ Agent 1 │          │ Agent 2 │          │ Agent 3 │
└────┬────┘          └────┬────┘          └────┬────┘
     │                    │                    │
     └────────────────────┴────────────────────┘
                          │
                          ▼
         ┌──────────────────────────────┐
         │     Shared File System       │
         ├──────────────────────────────┤
         │ • shared/context/            │
         │ • shared/messages/           │
         │ • agents/{id}/workspace/     │
         └──────────────────────────────┘
```

### 1. Shared Context Directory

Agents share artifacts through a structured directory:

```
~/.maos/projects/{workspace-hash}/sessions/{session-id}/shared/
├── context/
│   ├── architecture/
│   │   ├── system-design.md
│   │   ├── api-spec.yaml
│   │   └── diagrams/
│   ├── research/
│   │   ├── tech-evaluation.md
│   │   └── benchmarks.json
│   ├── implementation/
│   │   ├── modules.json
│   │   └── interfaces.ts
│   └── qa/
│       ├── test-plan.md
│       └── coverage-report.html
└── messages/
    ├── inbox/
    │   └── {agent-id}/
    │       └── {timestamp}-{from}-{msgid}.json
    └── outbox/
        └── {agent-id}/
            └── {timestamp}-{to}-{msgid}.json
```

### 2. Message Format

Inter-agent messages use a standardized JSON format:

```json
{
  "id": "msg_abc123",
  "timestamp": "2024-01-15T10:30:00Z",
  "from": "agent_architect_001",
  "to": "agent_engineer_002",
  "type": "request",
  "subject": "API endpoint clarification",
  "body": "Can you clarify the authentication flow for the /api/users endpoint?",
  "context": {
    "file": "shared/context/architecture/api-spec.yaml",
    "line": 145
  },
  "priority": "normal",
  "requires_response": true,
  "thread_id": "thread_xyz789"
}
```

### 3. Message Types and Patterns

```rust
pub enum MessageType {
    // Work coordination
    Request,      // Ask another agent for something
    Response,     // Reply to a request
    Handoff,      // Transfer responsibility
    
    // Status updates
    Progress,     // Report progress on a task
    Completed,    // Signal task completion
    Blocked,      // Report being blocked
    
    // Collaboration
    Review,       // Request review of work
    Feedback,     // Provide feedback
    Question,     // Ask for clarification
    
    // Broadcast
    Announcement, // Broadcast to all agents
    Alert,        // Important notification
}

pub struct MessageRouter {
    session_dir: PathBuf,
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
}

impl MessageRouter {
    pub async fn route_message(&self, message: Message) -> Result<()> {
        // Validate sender is registered
        if !self.agents.read().await.contains_key(&message.from) {
            return Err(anyhow!("Unknown sender: {}", message.from));
        }
        
        // Handle broadcast messages
        if message.to == "all" {
            return self.broadcast_message(message).await;
        }
        
        // Write to recipient's inbox
        let inbox_dir = self.session_dir
            .join("shared/messages/inbox")
            .join(&message.to);
        
        fs::create_dir_all(&inbox_dir).await?;
        
        let filename = format!("{}-{}-{}.json", 
            message.timestamp.timestamp(),
            message.from,
            message.id
        );
        
        let filepath = inbox_dir.join(filename);
        let content = serde_json::to_string_pretty(&message)?;
        fs::write(&filepath, content).await?;
        
        // Log the message
        self.logger.log_event("message_sent", json!({
            "from": message.from,
            "to": message.to,
            "type": message.message_type,
            "id": message.id,
        }))?;
        
        Ok(())
    }
    
    async fn broadcast_to_role(&self, message: Message, role_name: &str) -> Result<()> {
        let agents = self.agents.read().await;
        
        // Find all agents with matching role
        let matching_agents: Vec<String> = agents
            .iter()
            .filter(|(_, info)| info.role_name == role_name)
            .map(|(id, _)| id.clone())
            .collect();
        
        if matching_agents.is_empty() {
            return Err(anyhow!("No agents found with role: {}", role_name));
        }
        
        // Send to each matching agent
        for agent_id in matching_agents {
            if agent_id != message.from {
                let mut routed_msg = message.clone();
                routed_msg.to = agent_id;
                self.route_direct(routed_msg, &agent_id).await?;
            }
        }
        
        Ok(())
    }
    
    async fn route_by_pattern(&self, message: Message, pattern: &str) -> Result<()> {
        let agents = self.agents.read().await;
        let regex_pattern = pattern.replace('*', ".*");
        let regex = regex::Regex::new(&regex_pattern)?;
        
        // Find all agents matching the pattern
        let matching_agents: Vec<String> = agents
            .keys()
            .filter(|id| regex.is_match(id))
            .cloned()
            .collect();
        
        if matching_agents.is_empty() {
            return Err(anyhow!("No agents found matching pattern: {}", pattern));
        }
        
        // Send to each matching agent
        for agent_id in matching_agents {
            if agent_id != message.from {
                let mut routed_msg = message.clone();
                routed_msg.to = agent_id.clone();
                self.route_direct(routed_msg, &agent_id).await?;
            }
        }
        
        Ok(())
    }
}
```

### 4. Agent Registry and Discovery

Agents can discover other agents in the session:

```rust
pub struct AgentInfo {
    pub agent_id: String,
    pub role_name: String,
    pub role_description: Option<String>,
    pub instance_number: usize,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
}

pub struct AgentDiscovery {
    registry: Arc<RwLock<HashMap<String, AgentInfo>>>,
}

impl AgentDiscovery {
    pub async fn find_agents_by_role(&self, role_name: &str) -> Vec<AgentInfo> {
        self.registry
            .read()
            .await
            .values()
            .filter(|info| info.role_name == role_name)
            .cloned()
            .collect()
    }
    
    pub async fn find_agent_by_capability(&self, capability: &str) -> Option<AgentInfo> {
        self.registry
            .read()
            .await
            .values()
            .find(|info| info.capabilities.contains(&capability.to_string()))
            .cloned()
    }
    
    pub async fn get_role_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for info in self.registry.read().await.values() {
            *summary.entry(info.role_name.clone()).or_insert(0) += 1;
        }
        summary
    }
}
```

### 5. Agent-Side Communication

Agents include communication helpers in their environment:

```python
# Example helper script available to agents
import json
import os
from pathlib import Path
from datetime import datetime

class MAOSCommunicator:
    def __init__(self):
        self.agent_id = os.environ['MAOS_AGENT_ID']
        self.agent_role = os.environ['MAOS_AGENT_ROLE']
        self.session_id = os.environ['MAOS_SESSION_ID']
        self.messages_dir = Path(os.environ['MAOS_MESSAGE_DIR'])
        self.shared_context = Path(os.environ['MAOS_SHARED_CONTEXT'])
    
    def send_message(self, to_agent, message_type, subject, body, **kwargs):
        """Send a message to another agent or role group
        
        Args:
            to_agent: Can be:
                - Specific agent ID: 'agent_engineer_1_abc123'
                - All agents: 'all'
                - All of a role: 'all_engineers'
                - Pattern match: 'engineer_*'
        """
        message = {
            "id": f"msg_{datetime.now().timestamp()}",
            "timestamp": datetime.utcnow().isoformat(),
            "from": self.agent_id,
            "to": to_agent,
            "type": message_type,
            "subject": subject,
            "body": body,
            **kwargs
        }
        
        # Write to outbox (MAOS will route it)
        outbox = self.messages_dir / "outbox" / self.agent_id
        outbox.mkdir(parents=True, exist_ok=True)
        
        filename = f"{message['timestamp']}-{to_agent}-{message['id']}.json"
        with open(outbox / filename, 'w') as f:
            json.dump(message, f, indent=2)
        
        return message['id']
    
    def read_messages(self, message_type=None):
        """Read messages from inbox"""
        inbox = self.messages_dir / "inbox" / self.agent_id
        if not inbox.exists():
            return []
        
        messages = []
        for msg_file in inbox.glob("*.json"):
            with open(msg_file) as f:
                msg = json.load(f)
                if message_type is None or msg['type'] == message_type:
                    messages.append(msg)
        
        return sorted(messages, key=lambda m: m['timestamp'])
    
    def share_artifact(self, category, filename, content):
        """Share a file in the shared context"""
        category_dir = self.shared_context / category
        category_dir.mkdir(parents=True, exist_ok=True)
        
        filepath = category_dir / filename
        with open(filepath, 'w') as f:
            f.write(content)
        
        # Announce the artifact
        self.send_message(
            "all",
            "announcement",
            f"New {category} artifact: {filename}",
            f"Created {filepath}",
            context={"file": str(filepath)}
        )
        
        return filepath
    
    def broadcast_to_role(self, role_name, message_type, subject, body, **kwargs):
        """Send a message to all agents of a specific role"""
        return self.send_message(f"all_{role_name}", message_type, subject, body, **kwargs)
    
    def request_from_role(self, role_name, subject, body, **kwargs):
        """Request help from any agent with a specific role"""
        return self.send_message(
            f"all_{role_name}", 
            "request", 
            subject, 
            body,
            requires_response=True,
            **kwargs
        )
    
    def get_agents_by_role(self):
        """Get a summary of agents by role (parsed from agent IDs in messages)"""
        role_counts = {}
        
        # Parse received messages to identify active agents
        messages = self.read_messages()
        for msg in messages:
            sender = msg.get('from', '')
            # Extract role from agent ID format: agent_{role}_{instance}_{id}
            parts = sender.split('_')
            if len(parts) >= 3 and parts[0] == 'agent':
                role = parts[1]
                role_counts[role] = role_counts.get(role, 0) + 1
        
        return role_counts
```

### 6. Status Updates via stdout

Agents report status through structured JSON output:

```json
{"type": "status", "message": "Starting API design", "progress": 0.1}
{"type": "artifact", "path": "shared/context/architecture/api-spec.yaml", "description": "REST API specification"}
{"type": "dependency", "waiting_for": "agent_researcher_001", "reason": "Need database recommendations"}
{"type": "complete", "result": "success", "outputs": ["api-spec.yaml", "system-design.md"]}
```

### 7. Dependency Coordination

```rust
pub struct DependencyManager {
    dependencies: HashMap<String, Vec<String>>, // agent -> [dependencies]
    completed: HashSet<String>,
}

impl DependencyManager {
    pub async fn check_dependencies(&self, agent_id: &str) -> Result<bool> {
        if let Some(deps) = self.dependencies.get(agent_id) {
            for dep in deps {
                if !self.completed.contains(dep) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
    
    pub async fn wait_for_dependencies(&self, agent_id: &str) -> Result<()> {
        while !self.check_dependencies(agent_id).await? {
            // Check for new completions
            self.update_completed_agents().await?;
            
            // Check for messages about blockers
            if let Some(blocker_msg) = self.check_for_blocker_resolution(agent_id).await? {
                info!("Blocker resolved: {}", blocker_msg);
                break;
            }
            
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        Ok(())
    }
}
```

### 8. Broadcast Patterns

For system-wide coordination:

```rust
impl MessageRouter {
    pub async fn broadcast_message(&self, mut message: Message) -> Result<()> {
        let agents = self.agents.read().await;
        
        for (agent_id, _) in agents.iter() {
            if agent_id != &message.from {
                let mut broadcast_msg = message.clone();
                broadcast_msg.to = agent_id.clone();
                self.route_message(broadcast_msg).await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn announce_milestone(&self, milestone: &str) -> Result<()> {
        let message = Message {
            id: generate_message_id(),
            timestamp: Utc::now(),
            from: "maos_orchestrator".to_string(),
            to: "all".to_string(),
            message_type: MessageType::Announcement,
            subject: "Milestone Reached".to_string(),
            body: milestone.to_string(),
            ..Default::default()
        };
        
        self.broadcast_message(message).await
    }
}
```

## File Locking Strategy

To prevent conflicts when multiple agents access shared files:

```rust
pub struct FileLockManager {
    locks: Arc<RwLock<HashMap<PathBuf, String>>>, // path -> agent_id
}

impl FileLockManager {
    pub async fn acquire_lock(&self, path: &Path, agent_id: &str) -> Result<FileLock> {
        let mut locks = self.locks.write().await;
        
        if let Some(owner) = locks.get(path) {
            if owner != agent_id {
                return Err(anyhow!("File locked by {}", owner));
            }
        }
        
        locks.insert(path.to_path_buf(), agent_id.to_string());
        
        Ok(FileLock {
            path: path.to_path_buf(),
            manager: self.clone(),
        })
    }
}
```

## Consequences

### Positive
- **Simple Integration**: File-based communication works with any tool
- **Persistence**: Messages and artifacts are automatically saved
- **Debugging**: Easy to inspect communication history
- **Flexibility**: Agents can use any language/framework
- **Resilience**: Survives agent crashes
- **Role-Based Routing**: Easy to message all agents of a specific role
- **Pattern Matching**: Flexible targeting with wildcards
- **Agent Discovery**: Agents can find others by role or capability

### Negative  
- **Latency**: File system operations slower than memory
- **Polling**: Agents must poll for new messages
- **Cleanup**: Old messages need periodic cleanup
- **Complexity**: Multiple communication channels
- **Role Parsing**: Must extract role from agent IDs

### Mitigation
- Implement file system watchers for real-time updates
- Add message expiration and archival
- Provide client libraries for common languages
- Clear documentation and examples

## References
- Unix philosophy: Everything is a file
- Actor model for message passing
- Enterprise Integration Patterns
- File system notification APIs (inotify, FSEvents)

---
*Date: 2025-07-09*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollettLaFollett Labs LLC)*