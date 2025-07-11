# MAOS Environment Variables Reference

## Overview

This document consolidates all MAOS environment variables used for agent configuration and communication. These variables are automatically set by MAOS when spawning agent processes.

## Environment Variables

### Core Agent Identity

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `MAOS_AGENT_ID` | Unique agent identifier | `agent_engineer_2_xyz789` | Yes |
| `MAOS_AGENT_ROLE` | Agent's role name | `engineer`, `custom_analyst` | Yes |
| `MAOS_AGENT_INSTANCE` | Instance number for this role | `1`, `2`, `3` | Yes |
| `MAOS_SESSION_ID` | Current orchestration session | `sess_abc123` | Yes |

### Role Configuration

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `MAOS_AGENT_ROLE_DESC` | Role description for custom roles | `Analyze API performance and suggest optimizations` | For custom roles only |
| `MAOS_AGENT_ROLE_RESPONSIBILITIES` | Detailed responsibilities | `Design APIs, Review security, Document endpoints` | For custom roles only |

### File System Paths

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `MAOS_WORKSPACE` | Agent's isolated workspace | `~/.maos/projects/{hash}/sessions/{id}/agents/{agent-id}/workspace` | Yes |
| `MAOS_SHARED_CONTEXT` | Shared specifications directory | `~/.maos/projects/{hash}/sessions/{id}/shared/context` | Yes |
| `MAOS_MESSAGE_DIR` | Inter-agent message queue | `~/.maos/projects/{hash}/sessions/{id}/shared/messages` | Yes |
| `MAOS_PROJECT_ROOT` | Original project directory | `/Users/me/myproject` | Yes |

### Session Information

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `MAOS_SESSION_OBJECTIVE` | Session's main objective | `Build a REST API for user management` | No |
| `MAOS_SESSION_STRATEGY` | Execution strategy | `parallel`, `sequential`, `adaptive` | No |

### Resource Limits

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `MAOS_TIMEOUT_MINUTES` | Agent timeout in minutes | `45` | No (uses role default) |
| `MAOS_MAX_MEMORY_MB` | Memory limit in MB | `2048` | No (uses role default) |

## Usage Examples

### Reading Environment Variables in Agents

#### Python
```python
import os

# Get agent identity
agent_id = os.environ['MAOS_AGENT_ID']
role = os.environ['MAOS_AGENT_ROLE']
session_id = os.environ['MAOS_SESSION_ID']

# Get paths
workspace = os.environ['MAOS_WORKSPACE']
shared_context = os.environ['MAOS_SHARED_CONTEXT']
message_dir = os.environ['MAOS_MESSAGE_DIR']

# Check for custom role
if role.startswith('custom_'):
    role_desc = os.environ.get('MAOS_AGENT_ROLE_DESC', '')
```

#### Rust
```rust
use std::env;

// Get agent identity
let agent_id = env::var("MAOS_AGENT_ID").expect("MAOS_AGENT_ID not set");
let role = env::var("MAOS_AGENT_ROLE").expect("MAOS_AGENT_ROLE not set");
let session_id = env::var("MAOS_SESSION_ID").expect("MAOS_SESSION_ID not set");

// Get paths
let workspace = env::var("MAOS_WORKSPACE").expect("MAOS_WORKSPACE not set");
let shared_context = env::var("MAOS_SHARED_CONTEXT").expect("MAOS_SHARED_CONTEXT not set");
let message_dir = env::var("MAOS_MESSAGE_DIR").expect("MAOS_MESSAGE_DIR not set");
```

#### Bash
```bash
#!/bin/bash

# Agent identity
echo "Agent ID: $MAOS_AGENT_ID"
echo "Role: $MAOS_AGENT_ROLE"
echo "Session: $MAOS_SESSION_ID"

# Work in isolated workspace
cd "$MAOS_WORKSPACE"

# Read shared specifications
cat "$MAOS_SHARED_CONTEXT/api-spec.yaml"

# Send message to another agent
echo '{"type": "request", "content": "Please review"}' > \
  "$MAOS_MESSAGE_DIR/agent_engineer_1-to-agent_reviewer_1/$(date +%s).json"
```

## Environment Variable Flow

### 1. Setting Variables (MAOS Server)

```rust
pub fn prepare_agent_environment(
    agent: &Agent,
    session: &Session,
    paths: &SessionPaths,
) -> HashMap<String, String> {
    let mut env = HashMap::new();
    
    // Core identity
    env.insert("MAOS_AGENT_ID".to_string(), agent.id.clone());
    env.insert("MAOS_AGENT_ROLE".to_string(), agent.role.name.clone());
    env.insert("MAOS_AGENT_INSTANCE".to_string(), agent.instance_number.to_string());
    env.insert("MAOS_SESSION_ID".to_string(), session.id.clone());
    
    // Role configuration for custom roles
    if !agent.role.is_predefined {
        env.insert("MAOS_AGENT_ROLE_DESC".to_string(), 
                   agent.role.description.clone());
        env.insert("MAOS_AGENT_ROLE_RESPONSIBILITIES".to_string(), 
                   agent.role.responsibilities.clone());
    }
    
    // Paths
    env.insert("MAOS_WORKSPACE".to_string(), 
               paths.agent_workspace(&agent.id).to_string_lossy().to_string());
    env.insert("MAOS_SHARED_CONTEXT".to_string(), 
               paths.shared_context().to_string_lossy().to_string());
    env.insert("MAOS_MESSAGE_DIR".to_string(), 
               paths.message_dir().to_string_lossy().to_string());
    env.insert("MAOS_PROJECT_ROOT".to_string(), 
               session.project_root.to_string_lossy().to_string());
    
    // Optional session info
    env.insert("MAOS_SESSION_OBJECTIVE".to_string(), session.objective.clone());
    env.insert("MAOS_SESSION_STRATEGY".to_string(), session.strategy.to_string());
    
    env
}
```

### 2. Spawning Process with Environment

```rust
let mut cmd = Command::new("claude");
cmd.args(["-p", &project_root.to_string_lossy()])
   .arg(prompt_file.to_string_lossy().as_ref())
   .envs(&environment_vars)  // All MAOS_* variables
   .stdin(Stdio::null())
   .stdout(Stdio::piped())
   .stderr(Stdio::piped());

let child = cmd.spawn()?;
```

## Variable Validation

### Required Variables
The following variables MUST be set for all agents:
- `MAOS_AGENT_ID`
- `MAOS_AGENT_ROLE`
- `MAOS_AGENT_INSTANCE`
- `MAOS_SESSION_ID`
- `MAOS_WORKSPACE`
- `MAOS_SHARED_CONTEXT`
- `MAOS_MESSAGE_DIR`
- `MAOS_PROJECT_ROOT`

### Conditional Variables
These are set only when applicable:
- `MAOS_AGENT_ROLE_DESC` - Only for custom roles
- `MAOS_AGENT_ROLE_RESPONSIBILITIES` - Only for custom roles
- `MAOS_SESSION_OBJECTIVE` - Optional session context
- `MAOS_SESSION_STRATEGY` - Optional execution hint

## Security Considerations

1. **Path Validation**: All path variables should be validated to prevent directory traversal
2. **Value Sanitization**: Role descriptions and custom values should be sanitized
3. **Environment Isolation**: Each agent process should only see its own MAOS variables
4. **No Secrets**: Never pass sensitive data through environment variables

## Debugging

To debug agent environment setup:

```bash
# List all MAOS variables
env | grep ^MAOS_

# Verify paths exist
ls -la "$MAOS_WORKSPACE"
ls -la "$MAOS_SHARED_CONTEXT"
ls -la "$MAOS_MESSAGE_DIR"
```

## References

This document consolidates environment variable definitions from:
- ADR-004: CLI Integration and Process Spawning (primary definition)
- ADR-008: Agent Communication Patterns (usage examples)