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
# Pattern: agent/issue-<issue-num>/<agent-role>-<issue-summary>
agent/issue-42/architect-microservices
agent/issue-42/engineer-api
agent/issue-42/tester-integration
agent/issue-43/researcher-performance

# For parallel sub-tasks
agent/issue-42/engineer-auth
agent/issue-42/engineer-data
```

### 3. Worktree Creation Pattern

```python
#!/usr/bin/env python3
# create_agent_worktree.py

import json
import subprocess
import sys
from datetime import datetime
from pathlib import Path
import re

# Constants
MAX_DESC_LENGTH = 20

def create_agent_worktree(agent_role, issue_id, task_description, base_branch='main'):
    """Create a git worktree for a Claude Code agent."""
    
    # Generate safe branch name
    safe_desc = re.sub(r'[^a-zA-Z0-9-]', '-', task_description)[:MAX_DESC_LENGTH]
    branch_name = f"agent/issue-{issue_id}/{agent_role}-{safe_desc}"
    worktree_path = f"./worktrees/{agent_role}-issue-{issue_id}-{safe_desc}"
    
    # Create worktree
    subprocess.run([
        "git", "worktree", "add", "-b", branch_name, worktree_path, base_branch
    ], check=True)
    
    # Lock worktree with reason
    subprocess.run([
        "git", "worktree", "lock", worktree_path,
        "--reason", f"Claude Code agent: {agent_role} working on issue {issue_id}"
    ], check=True)
    
    # Create tracking file
    metadata = {
        "agent": agent_role,
        "issue": issue_id,
        "task": task_desc,
        "created": datetime.utcnow().isoformat() + "Z",
        "status": "in_progress"
    }
    
    Path(worktree_path).joinpath(".agent-metadata.json").write_text(
        json.dumps(metadata, indent=2)
    )
    
    print(f"Worktree created at: {worktree_path}")
    print(f"Branch: {branch_name}")
    
if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python create_agent_worktree.py <agent_role> <issue_id> <task_desc> [base_branch]")
        sys.exit(1)
    
    create_agent_worktree(*sys.argv[1:])
```

### 4. Coordination Pattern: Task Registry

```bash
# .maos/active-agents.json
{
  "agents": [
    {
      "id": "architect-001",
      "worktree": "worktrees/architect-issue-42-microservices",
      "branch": "agent/issue-42/architect-microservices",
      "files_modified": ["src/architecture/", "docs/design/"],
      "status": "active",
      "started": "2024-01-27T10:00:00Z"
    },
    {
      "id": "engineer-001",
      "worktree": "worktrees/engineer-issue-42-api",
      "branch": "agent/issue-42/engineer-api",
      "files_modified": ["src/api/", "tests/api/"],
      "status": "active",
      "started": "2024-01-27T10:05:00Z"
    }
  ]
}
```

### 5. Conflict Prevention Pattern

```python
#!/usr/bin/env python3
# check_file_conflicts.py

import subprocess
import sys
from pathlib import Path

def check_conflicts(target_files):
    """Check if files are being modified in other worktrees."""
    conflicts = []
    
    # Get all worktrees
    result = subprocess.run(
        ["git", "worktree", "list", "--porcelain"],
        capture_output=True, text=True
    )
    
    worktrees = []
    for line in result.stdout.split('\n'):
        if line.startswith('worktree '):
            worktrees.append(line.split(' ', 1)[1])
    
    # Check each worktree for modifications
    for worktree in worktrees:
        for file in target_files:
            file_path = Path(worktree) / file
            if file_path.exists():
                # Check if file has uncommitted changes
                result = subprocess.run(
                    ["git", "-C", worktree, "diff", "--name-only"],
                    capture_output=True, text=True
                )
                if file in result.stdout:
                    conflicts.append(f"{file} is being modified in {worktree}")
    
    if conflicts:
        print("Potential conflicts detected:")
        for conflict in conflicts:
            print(f"  - {conflict}")
        return False
    
    return True

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python check_file_conflicts.py <file1> [file2] ...")
        sys.exit(1)
    
    if not check_conflicts(sys.argv[1:]):
        sys.exit(1)
```

### 6. Merge Strategy Pattern

```python
#!/usr/bin/env python3
# parallel_merge_strategy.py

import subprocess
import sys

def sequential_merge(base_branch='main', feature_branches=None):
    """Sequential merge for complex features."""
    if not feature_branches:
        return
    
    for branch in feature_branches:
        print(f"Merging {branch}...")
        
        # Checkout base branch
        subprocess.run(["git", "checkout", base_branch], check=True)
        
        # Merge feature branch
        subprocess.run([
            "git", "merge", "--no-ff", branch,
            "-m", f"Merge {branch} into {base_branch}"
        ], check=True)
        
        # Run tests after each merge
        result = subprocess.run(["npm", "test"], capture_output=True)
        if result.returncode != 0:
            print(f"Tests failed after merging {branch}")
            subprocess.run(["git", "reset", "--hard", "HEAD~1"], check=True)
            return False
    
    return True

def octopus_merge(base_branch='main', feature_branches=None):
    """Octopus merge for simple, non-conflicting branches."""
    if not feature_branches:
        return
    
    subprocess.run(["git", "checkout", base_branch], check=True)
    
    merge_cmd = ["git", "merge", "--no-ff"] + list(feature_branches)
    merge_cmd.extend(["-m", f"Octopus merge: {' '.join(feature_branches)}"])
    
    subprocess.run(merge_cmd, check=True)

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python parallel_merge_strategy.py <sequential|octopus> <branch1> [branch2] ...")
        sys.exit(1)
    
    strategy = sys.argv[1]
    branches = sys.argv[2:]
    
    if strategy == "sequential":
        sequential_merge(feature_branches=branches)
    elif strategy == "octopus":
        octopus_merge(feature_branches=branches)
    else:
        print(f"Unknown strategy: {strategy}")
        sys.exit(1)
```

### 7. Cleanup and Maintenance Pattern

```python
#!/usr/bin/env python3
# cleanup_worktrees.py

import json
import subprocess
from pathlib import Path
from datetime import datetime, timedelta

def cleanup_completed():
    """Clean up completed worktrees."""
    # Get all worktrees
    result = subprocess.run(
        ["git", "worktree", "list", "--porcelain"],
        capture_output=True, text=True
    )
    
    worktrees = []
    for line in result.stdout.split('\n'):
        if line.startswith('worktree '):
            worktrees.append(line.split(' ', 1)[1])
    
    # Check each worktree
    for worktree_path in worktrees:
        metadata_path = Path(worktree_path) / ".agent-metadata.json"
        if metadata_path.exists():
            with open(metadata_path) as f:
                metadata = json.load(f)
            
            if metadata.get('status') == 'completed':
                print(f"Removing completed worktree: {worktree_path}")
                subprocess.run(["git", "worktree", "remove", worktree_path])
    
    # Prune any stale worktree references
    subprocess.run(["git", "worktree", "prune"])

def weekly_maintenance(days_old=14):
    """Run weekly worktree maintenance."""
    print("Running weekly worktree maintenance...")
    
    # Get all worktrees
    result = subprocess.run(
        ["git", "worktree", "list", "--porcelain"],
        capture_output=True, text=True
    )
    
    print("\nActive worktrees:")
    worktrees = {}
    current_key = None
    
    for line in result.stdout.split('\n'):
        if line.startswith('worktree '):
            current_key = line.split(' ', 1)[1]
            worktrees[current_key] = {}
        elif line.startswith('branch ') and current_key:
            worktrees[current_key]['branch'] = line.split(' ', 1)[1].replace('refs/heads/', '')
    
    # Check age and display info
    cutoff_date = datetime.now() - timedelta(days=days_old)
    
    for worktree_path, info in worktrees.items():
        metadata_path = Path(worktree_path) / ".agent-metadata.json"
        if metadata_path.exists():
            with open(metadata_path) as f:
                metadata = json.load(f)
            
            created = metadata.get('created', '')
            branch = info.get('branch', 'unknown')
            print(f"  {branch} (created: {created})")
            
            # Remove old worktrees
            if created:
                created_date = datetime.fromisoformat(created.replace('Z', '+00:00'))
                if created_date < cutoff_date:
                    print(f"    → Removing old worktree (older than {days_old} days)")
                    subprocess.run(["git", "worktree", "remove", worktree_path])

if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1 and sys.argv[1] == "weekly":
        weekly_maintenance()
    else:
        cleanup_completed()
```

### 8. Claude Code Integration Pattern

```python
#!/usr/bin/env python3
# claude_agent_launcher.py

import subprocess
import sys
from pathlib import Path
import re

def launch_claude_agent(agent_role, issue_id, task_desc):
    """Launch a Claude Code agent in a dedicated worktree."""
    
    # Create worktree
    safe_desc = re.sub(r'[^a-zA-Z0-9-]', '-', task_description)[:MAX_DESC_LENGTH]
    branch_name = f"agent/issue-{issue_id}/{agent_role}-{safe_desc}"
    worktree_path = f"./worktrees/{agent_role}-issue-{issue_id}-{safe_desc}"
    
    # Create the worktree
    subprocess.run([
        sys.executable, "create_agent_worktree.py",
        agent_role, issue_id, task_desc
    ])
    
    # Create agent prompt file
    prompt_content = f"""You are a {agent_role} agent working on issue #{issue_id}.

Task: {task_desc}

Working directory: {worktree_path}
Branch: {branch_name}

Please follow these guidelines:
1. Only modify files within your assigned scope
2. Run tests before committing
3. Update .agent-metadata.json when task is complete
4. Create clear, atomic commits

Current task status can be found in .agent-metadata.json
"""
    
    prompt_path = Path(worktree_path) / ".claude-prompt.md"
    prompt_path.write_text(prompt_content)
    
    # Launch Claude Code in the worktree
    subprocess.run(["code", worktree_path])
    print(f"Launched Claude Code for {agent_role} in {worktree_path}")

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python claude_agent_launcher.py <agent_role> <issue_id> <task_desc>")
        sys.exit(1)
    
    launch_claude_agent(*sys.argv[1:])
```

### 9. Status Monitoring Pattern

```python
#!/usr/bin/env python3
# monitor_agents.py

import json
import subprocess
import time
from pathlib import Path
from datetime import datetime

def monitor_agents():
    """Real-time agent status dashboard."""
    while True:
        # Clear screen (cross-platform)
        print('\033[2J\033[H', end='')
        
        print("=== MAOS Agent Status Dashboard ===")
        print(f"Time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print()
        
        # Get all worktrees
        result = subprocess.run(
            ["git", "worktree", "list", "--porcelain"],
            capture_output=True, text=True
        )
        
        worktrees = []
        for line in result.stdout.split('\n'):
            if line.startswith('worktree '):
                worktrees.append(line.split(' ', 1)[1])
        
        # Show status for each worktree
        for worktree_path in worktrees:
            metadata_path = Path(worktree_path) / ".agent-metadata.json"
            if metadata_path.exists():
                with open(metadata_path) as f:
                    metadata = json.load(f)
                
                agent = metadata.get('agent', 'unknown')
                issue = metadata.get('issue', 'unknown')
                status = metadata.get('status', 'unknown')
                
                # Get current activity
                result = subprocess.run(
                    ["git", "-C", worktree_path, "status", "--porcelain"],
                    capture_output=True, text=True
                )
                changes = len(result.stdout.strip().split('\n')) if result.stdout.strip() else 0
                
                print(f"[{status}] Agent: {agent} | Issue: #{issue} | Changes: {changes}")
        
        # Wait before refresh
        time.sleep(5)

if __name__ == "__main__":
    try:
        monitor_agents()
    except KeyboardInterrupt:
        print("\nMonitoring stopped.")
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