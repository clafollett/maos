---
allowed-tools: Bash, Read
description: Show current Claude Code session info and associated MAOS sessions
argument-hint: none
---

# Current Session Info

Shows information about the current Claude Code session and any associated MAOS sessions.

## Get Session Info

```bash
# Show current Claude session
if [ -f ".maos/.current_claude_session" ]; then
    echo "=== Current Claude Session ==="
    cat .maos/.current_claude_session | python3 -m json.tool
    echo ""
fi

# Check for current orchestration
if [ -f ".maos/.current_orchestration" ]; then
    echo "=== Current Orchestration ==="
    cat .maos/.current_orchestration | python3 -m json.tool
    echo ""
fi

# Show the MAOS session for this Claude session if it exists
if [ -f ".maos/.current_claude_session" ]; then
    # Safer JSON parsing using Python module
    CLAUDE_SESSION=$(python3 -c "
import json
import sys
try:
    with open('.maos/.current_claude_session', 'r') as f:
        data = json.load(f)
        print(data.get('session_id', ''))
except:
    sys.exit(1)
" 2>/dev/null)
    
    if [ ! -z "$CLAUDE_SESSION" ]; then
        echo "=== MAOS Session for Current Claude Session ==="
        python3 .claude/hooks/maos/session_manager.py show "$CLAUDE_SESSION" 2>/dev/null || echo "No MAOS session created yet for this Claude session"
        echo ""
    fi
fi

# Show all active MAOS sessions
echo "=== All Active MAOS Sessions ==="
python3 .claude/hooks/maos/session_manager.py list active

echo -e "\n=== Session Details ==="
echo "MAOS now uses Claude session IDs directly!"
echo "Each Claude conversation has its own MAOS session."
```

## Note

The UserPromptSubmit hook captures the Claude session ID for every prompt, making it available to slash commands!