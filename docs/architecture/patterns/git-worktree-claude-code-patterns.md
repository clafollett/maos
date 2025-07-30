# Git Worktree Patterns for Claude Code Multi-Agent Development

## Overview

This document provides concrete implementation patterns for using Git worktrees to enable parallel development with multiple Claude Code instances. These patterns are based on research of best practices, real-world usage, and known limitations.

## Core Concepts

Git worktrees allow multiple working directories to be attached to a single repository, enabling parallel development without the overhead of multiple clones or constant branch switching.

## Implementation Patterns

### 1. Directory Structure Pattern

```
maos/                           # Root directory
├── .git/                       # Main git directory (shared)
├── main/                       # Main worktree
├── worktrees/                  # Container for all worktrees
│   ├── feature-auth/          # Feature worktree
│   ├── bugfix-api/            # Bugfix worktree
│   └── experiment-ml/         # Experimental worktree
└── scripts/                   # Automation scripts
```

### 2. Branch Naming Convention for Multi-Agent Work

```
# Pattern: <agent-role>/<issue-id>/<descriptive-name>
architect/issue-42/microservices-design
engineer/issue-42/api-implementation
tester/issue-42/integration-tests
researcher/issue-43/performance-analysis

# For parallel sub-tasks
engineer/issue-42/api-implementation-auth
engineer/issue-42/api-implementation-data
```

### 3. Worktree Creation Script

```bash
#!/bin/bash
# create-agent-worktree.sh

AGENT_ROLE=$1
ISSUE_ID=$2
TASK_DESC=$3
BASE_BRANCH=${4:-main}

# Generate branch name
BRANCH_NAME="${AGENT_ROLE}/issue-${ISSUE_ID}/${TASK_DESC}"
WORKTREE_PATH="./worktrees/${BRANCH_NAME//\//-}"

# Create worktree
git worktree add -b "$BRANCH_NAME" "$WORKTREE_PATH" "$BASE_BRANCH"

# Lock worktree with reason
git worktree lock "$WORKTREE_PATH" --reason "Claude Code agent: $AGENT_ROLE working on issue $ISSUE_ID"

# Create tracking file
echo "{
  \"agent\": \"$AGENT_ROLE\",
  \"issue\": \"$ISSUE_ID\",
  \"task\": \"$TASK_DESC\",
  \"created\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
  \"status\": \"in_progress\"
}" > "$WORKTREE_PATH/.agent-metadata.json"

echo "Worktree created at: $WORKTREE_PATH"
echo "Branch: $BRANCH_NAME"
```

### 4. Coordination Pattern: Task Registry

```bash
# .maos/active-agents.json
{
  "agents": [
    {
      "id": "architect-001",
      "worktree": "worktrees/architect-issue-42-microservices",
      "branch": "architect/issue-42/microservices-design",
      "files_modified": ["src/architecture/", "docs/design/"],
      "status": "active",
      "started": "2024-01-27T10:00:00Z"
    },
    {
      "id": "engineer-001",
      "worktree": "worktrees/engineer-issue-42-api",
      "branch": "engineer/issue-42/api-implementation",
      "files_modified": ["src/api/", "tests/api/"],
      "status": "active",
      "started": "2024-01-27T10:05:00Z"
    }
  ]
}
```

### 5. Conflict Prevention Pattern

```bash
#!/bin/bash
# check-file-conflicts.sh

# Check if files are being modified in other worktrees
check_conflicts() {
  local target_files=("$@")
  local conflicts=()
  
  # Read active agents registry
  for worktree in $(git worktree list --porcelain | grep "worktree" | cut -d' ' -f2); do
    for file in "${target_files[@]}"; do
      if [[ -f "$worktree/$file" ]] && git -C "$worktree" diff --name-only | grep -q "$file"; then
        conflicts+=("$file is being modified in $worktree")
      fi
    done
  done
  
  if [ ${#conflicts[@]} -gt 0 ]; then
    echo "Potential conflicts detected:"
    printf '%s\n' "${conflicts[@]}"
    return 1
  fi
  
  return 0
}
```

### 6. Merge Strategy Pattern

```bash
#!/bin/bash
# parallel-merge-strategy.sh

# Sequential merge for complex features
sequential_merge() {
  local base_branch="main"
  local feature_branches=("$@")
  
  for branch in "${feature_branches[@]}"; do
    echo "Merging $branch..."
    git checkout "$base_branch"
    git merge --no-ff "$branch" -m "Merge $branch into $base_branch"
    
    # Run tests after each merge
    if ! npm test; then
      echo "Tests failed after merging $branch"
      git reset --hard HEAD~1
      return 1
    fi
  done
}

# Octopus merge for simple, non-conflicting branches
octopus_merge() {
  local base_branch="main"
  local feature_branches=("$@")
  
  git checkout "$base_branch"
  git merge --no-ff "${feature_branches[@]}" -m "Octopus merge: ${feature_branches[*]}"
}
```

### 7. Cleanup and Maintenance Pattern

```bash
#!/bin/bash
# cleanup-worktrees.sh

# Clean up completed worktrees
cleanup_completed() {
  for worktree_info in $(git worktree list --porcelain); do
    if [[ $worktree_info == worktree* ]]; then
      worktree_path=${worktree_info#worktree }
      
      # Check if metadata exists and task is completed
      if [[ -f "$worktree_path/.agent-metadata.json" ]]; then
        status=$(jq -r '.status' "$worktree_path/.agent-metadata.json")
        if [[ "$status" == "completed" ]]; then
          echo "Removing completed worktree: $worktree_path"
          git worktree remove "$worktree_path"
        fi
      fi
    fi
  done
  
  # Prune any stale worktree references
  git worktree prune
}

# Weekly maintenance
weekly_maintenance() {
  echo "Running weekly worktree maintenance..."
  
  # List all worktrees with age
  echo "Active worktrees:"
  git worktree list --porcelain | grep -E "^(worktree|branch)" | paste - - | \
    while read -r worktree branch; do
      path=${worktree#worktree }
      branch_name=${branch#branch refs/heads/}
      if [[ -f "$path/.agent-metadata.json" ]]; then
        created=$(jq -r '.created' "$path/.agent-metadata.json")
        echo "  $branch_name (created: $created)"
      fi
    done
  
  # Clean up old worktrees (older than 2 weeks)
  cleanup_old_worktrees 14
}
```

### 8. Claude Code Integration Pattern

```bash
#!/bin/bash
# claude-agent-launcher.sh

launch_claude_agent() {
  local agent_role=$1
  local issue_id=$2
  local task_desc=$3
  
  # Create worktree
  ./scripts/create-agent-worktree.sh "$agent_role" "$issue_id" "$task_desc"
  
  # Get worktree path
  local branch_name="${agent_role}/issue-${issue_id}/${task_desc}"
  local worktree_path="./worktrees/${branch_name//\//-}"
  
  # Create agent prompt file
  cat > "$worktree_path/.claude-prompt.md" << EOF
You are a $agent_role agent working on issue #$issue_id.

Task: $task_desc

Working directory: $worktree_path
Branch: $branch_name

Please follow these guidelines:
1. Only modify files within your assigned scope
2. Run tests before committing
3. Update .agent-metadata.json when task is complete
4. Create clear, atomic commits

Current task status can be found in .agent-metadata.json
EOF

  # Launch Claude Code in the worktree
  cd "$worktree_path" && code .
}
```

### 9. Status Monitoring Pattern

```bash
#!/bin/bash
# monitor-agents.sh

# Real-time agent status dashboard
monitor_agents() {
  while true; do
    clear
    echo "=== MAOS Agent Status Dashboard ==="
    echo "Time: $(date)"
    echo ""
    
    # Show active worktrees and their status
    for worktree_info in $(git worktree list --porcelain); do
      if [[ $worktree_info == worktree* ]]; then
        worktree_path=${worktree_info#worktree }
        
        if [[ -f "$worktree_path/.agent-metadata.json" ]]; then
          agent=$(jq -r '.agent' "$worktree_path/.agent-metadata.json")
          issue=$(jq -r '.issue' "$worktree_path/.agent-metadata.json")
          status=$(jq -r '.status' "$worktree_path/.agent-metadata.json")
          
          # Get current activity
          changes=$(git -C "$worktree_path" status --porcelain | wc -l)
          
          echo "[$status] Agent: $agent | Issue: #$issue | Changes: $changes"
        fi
      fi
    done
    
    sleep 5
  done
}
```

### 10. Best Practices Summary

1. **Limit Active Worktrees**: Keep no more than 5-6 active worktrees per developer
2. **Use Descriptive Names**: Follow the agent/issue/task naming pattern
3. **Lock Active Worktrees**: Prevent accidental deletion during active development
4. **Regular Cleanup**: Run maintenance scripts weekly
5. **Monitor Conflicts**: Check for file conflicts before starting work
6. **Atomic Commits**: Make small, focused commits in each worktree
7. **Test Isolation**: Each worktree should have its own test environment
8. **Document Status**: Keep .agent-metadata.json updated

## Known Limitations

1. **Submodules**: Worktrees have incomplete support for submodules
2. **Disk Space**: Each worktree is a full checkout, consuming significant space
3. **Tool Compatibility**: Not all Git GUIs support worktrees well
4. **Branch Restrictions**: Cannot checkout same branch in multiple worktrees
5. **Path Dependencies**: Moving repositories can break worktree links

## Conclusion

These patterns enable efficient parallel development with Claude Code agents by providing structure, automation, and clear coordination mechanisms. The key is maintaining discipline around naming conventions, regular cleanup, and proactive conflict prevention.