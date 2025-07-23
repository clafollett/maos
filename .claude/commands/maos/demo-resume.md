---
allowed-tools: Bash, Read, Write
description: Demo showing session resume workflow
argument-hint: none
---

# Demo: Session Resume Workflow

This demonstrates how MAOS session resume works.

## Step 1: Simulate an Interrupted Session

```bash
# Create a mock session that was interrupted
SESSION_ID="demo-$(date +%s)"
mkdir -p ".maos/sessions/$SESSION_ID/agents/backend-engineer"
mkdir -p ".maos/sessions/$SESSION_ID/shared"
mkdir -p ".maos/sessions/$SESSION_ID/checkpoints"

# Simulate some completed work
cat > ".maos/sessions/$SESSION_ID/shared/architecture.md" << 'EOF'
# API Architecture

Completed by architect agent:
- REST API design
- Database schema (PostgreSQL)  
- Authentication: JWT
- Endpoints: /users, /auth, /products
EOF

# Simulate a checkpoint from backend engineer
cat > ".maos/sessions/$SESSION_ID/checkpoints/backend_progress.json" << EOF
{
  "name": "backend_progress",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "data": {
    "completed": [
      "Database models created",
      "User authentication implemented", 
      "/users endpoint complete",
      "/auth endpoint complete"
    ],
    "in_progress": "Working on /products endpoint",
    "remaining": [
      "/products endpoint",
      "Unit tests",
      "Integration tests"
    ],
    "workspace_files": [
      "src/models/user.py",
      "src/models/product.py", 
      "src/api/users.py",
      "src/api/auth.py",
      "src/api/products.py (partial)"
    ]
  }
}
EOF

# Create session registry entry
mkdir -p ".maos/registry"
cat > ".maos/registry/$SESSION_ID.json" << EOF
{
  "session_id": "$SESSION_ID",
  "type": "orchestration",
  "task": "Build a complete REST API for e-commerce with user management and product catalog",
  "status": "timeout",
  "created": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "metadata": {
    "agents": ["architect", "backend-engineer"],
    "workspace": ".maos/sessions/$SESSION_ID",
    "checkpoints": ["backend_progress"],
    "outputs": ["architecture.md"]
  }
}
EOF

echo "✅ Created demo session: $SESSION_ID"
echo ""
echo "This simulates a session where:"
echo "1. Architect completed the design"
echo "2. Backend engineer implemented auth and users"  
echo "3. Session timed out during products implementation"
echo ""
echo "To see the session status:"
echo "  /maos:session show $SESSION_ID"
echo ""
echo "To resume this session:"
echo "  /maos:session resume $SESSION_ID"
```

## Step 2: Show How Resume Works

The resume command would:

1. Load the session context
2. Read checkpoints to understand progress
3. Provide continuation context to agents
4. Agents pick up where they left off

Example resume prompt for backend engineer:
```
You are resuming a previous session. Here's your progress:

Completed:
- Database models created
- User authentication implemented
- /users endpoint complete  
- /auth endpoint complete

In Progress:
- Working on /products endpoint

Remaining:
- Complete /products endpoint
- Unit tests
- Integration tests

Your workspace has these files:
- src/models/user.py
- src/models/product.py
- src/api/users.py
- src/api/auth.py
- src/api/products.py (partial)

Continue from where you left off, starting with completing the /products endpoint.
```

## The Magic of Explicit Resume

- **No guessing**: System knows exactly what was done
- **Clear handoff**: Agent knows what to continue
- **Preserved context**: All decisions maintained
- **Workspace intact**: Previous work available

This is much better than trying to automatically match tasks!