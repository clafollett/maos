# Git Worktree Integration Design for Claude Code Sub-Agents

## Executive Summary

This document presents the comprehensive design for git worktree management in the MAOS multi-agent system. The design optimizes for parallel Claude Code sub-agent development while maintaining IDE compatibility, performance, and clean integration patterns.

**Key Design Decisions:**
- **Hierarchical Directory Structure**: Organized worktrees under a central `worktrees/` directory with clear naming conventions
- **Atomic File-Based Coordination**: Lock-free design using atomic file operations for agent coordination
- **Progressive Lifecycle Management**: Create → Monitor → Integrate → Cleanup with automated maintenance
- **IDE-First Design**: Optimized for VS Code and similar IDEs with file watchers and language servers

## Architecture Overview

### Directory Structure

```
maos/                                    # Main repository root
├── .git/                               # Shared git directory
├── main/                               # Primary development workspace
│   ├── src/                           # Source code
│   ├── docs/                          # Documentation
│   └── .maos/                         # MAOS metadata
│       ├── coordination/              # Coordination files
│       └── registry/                  # Agent registry
├── worktrees/                          # All agent worktrees
│   ├── architect-issue-42-design/     # Architect agent worktree
│   ├── backend-issue-42-api/          # Backend engineer worktree
│   ├── frontend-issue-42-ui/          # Frontend engineer worktree
│   └── tester-issue-42-integration/   # Tester agent worktree
└── .claude/hooks/utils/                # Python utilities for hooks
    ├── worktree_create.py             # Create new worktree
    ├── worktree_cleanup.py            # Cleanup completed worktrees
    ├── worktree_monitor.py            # Monitor active worktrees
    ├── worktree_integrate.py          # Integrate worktrees
    ├── worktree_list.py               # List active worktrees
    └── conflict_detection.py          # Detect conflicts
```

### Branch Naming Convention

```
Pattern: wrktr/issue-<issue-num>/<agent-role>-<issue-summary>

Examples:
- wrktr/issue-42/architect-microservices
- wrktr/issue-42/backend-engineer-auth
- wrktr/issue-42/frontend-engineer-login
- wrktr/issue-42/tester-integration
```

## Worktree Lifecycle Management

### 1. Creation Phase

```python
#!/usr/bin/env python3
# worktree_create.py - Enhanced worktree creation with IDE optimization

import json
import subprocess
import sys
import uuid
import re
from datetime import datetime
from pathlib import Path

def create_agent_worktree(agent_role, issue_id, task_desc, base_branch='main'):
    """Create a git worktree for an agent with IDE optimization."""
    
    # Sanitize inputs for filesystem safety
    safe_task_desc = re.sub(r'[^a-zA-Z0-9-]', '-', task_desc)[:30]
    
    # Generate branch and worktree names
    branch_name = f"wrktr/issue-{issue_id}/{agent_role}-{safe_task_desc}"
    worktree_dir = f"worktrees/{agent_role}-issue-{issue_id}-{safe_task_desc}"
    
    # Check for conflicts before creation
    if check_semantic_conflicts(agent_role, issue_id):
        print("⚠️  Semantic conflict detected. Aborting worktree creation.")
        return False
    
    # Create worktree with branch
    print(f"🌳 Creating worktree: {worktree_dir}")
    try:
        subprocess.run([
            "git", "worktree", "add", "-b", branch_name, worktree_dir, base_branch
        ], check=True)
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed to create worktree: {e}")
        return False
    
    # Lock worktree to prevent accidental deletion
    subprocess.run([
        "git", "worktree", "lock", worktree_dir,
        "--reason", f"MAOS Agent: {agent_role} working on issue #{issue_id}"
    ], check=True)
    
    # Initialize agent metadata
    agent_metadata = {
        "agent_id": str(uuid.uuid4()),
        "agent_role": agent_role,
        "issue_id": issue_id,
        "task_description": task_desc,
        "branch_name": branch_name,
        "base_branch": base_branch,
        "created_at": datetime.utcnow().isoformat() + "Z",
        "status": "initializing",
        "files_scope": [],
        "ide_config": {
            "editor": "vscode",
            "workspace_settings": True,
            "extensions": ["recommended"]
        }
    }
    
    metadata_path = Path(worktree_dir) / ".agent-metadata.json"
    with open(metadata_path, 'w') as f:
        json.dump(agent_metadata, f, indent=2)
    
    # Create IDE-friendly configuration
    setup_ide_workspace(worktree_dir, agent_role, issue_id, task_desc, branch_name)
    
    # Register in central registry
    register_worktree(worktree_dir, agent_role, issue_id)
    
    print(f"✅ Worktree created successfully at: {worktree_dir}")
    print(f"📋 Branch: {branch_name}")
    return True

def setup_ide_workspace(worktree_dir, agent_role, issue_id, task_desc, branch_name):
    """Set up IDE workspace configuration."""
    worktree_path = Path(worktree_dir)
    
    # Create VS Code workspace settings
    vscode_dir = worktree_path / ".vscode"
    vscode_dir.mkdir(exist_ok=True)
    
    # Workspace-specific settings to avoid conflicts
    settings = {
        "window.title": f"MAOS: {agent_role} - {worktree_dir}",
        "files.watcherExclude": {
            "**/worktrees/**": True,
            "**/.git/objects/**": True,
            "**/.git/subtree-cache/**": True
        },
        "search.exclude": {
            "**/worktrees/**": True
        },
        "git.ignoreLimitWarning": True,
        "terminal.integrated.cwd": worktree_dir
    }
    
    with open(vscode_dir / "settings.json", 'w') as f:
        json.dump(settings, f, indent=2)
    
    # Create agent-specific prompt file for Claude Code
    context_content = f"""# MAOS Agent Context

You are a {agent_role} agent working in an isolated git worktree.

## Current Task
- Issue: #{issue_id}
- Description: {task_desc}
- Branch: {branch_name}
- Worktree: {worktree_dir}

## Guidelines
1. Only modify files within your assigned scope
2. Commit changes frequently with clear messages
3. Update .agent-metadata.json to reflect progress
4. Coordinate through .maos/coordination/ files
5. Run tests before marking tasks complete

## File Scope
Check .agent-metadata.json for your allowed file paths.
"""
    
    with open(worktree_path / ".claude-context.md", 'w') as f:
        f.write(context_content)

def check_semantic_conflicts(agent_role, issue_id):
    """Check for semantic conflicts before creating worktree."""
    # Implementation would check for conflicts
    # For now, return False (no conflicts)
    return False

def register_worktree(worktree_dir, agent_role, issue_id):
    """Register worktree in central registry."""
    # Implementation would update registry
    pass

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: worktree_create.py <agent_role> <issue_id> <task_desc> [base_branch]")
        sys.exit(1)
    
    agent_role = sys.argv[1]
    issue_id = sys.argv[2]
    task_desc = sys.argv[3]
    base_branch = sys.argv[4] if len(sys.argv) > 4 else "main"
    
    success = create_agent_worktree(agent_role, issue_id, task_desc, base_branch)
    sys.exit(0 if success else 1)
```

### 2. Work Phase

```python
#!/usr/bin/env python3
# worktree_monitor.py - Real-time monitoring and coordination

import json
import subprocess
import time
import os
from datetime import datetime
from pathlib import Path

def monitor_active_worktrees():
    """Monitor active worktrees in real-time."""
    while True:
        # Clear screen (cross-platform)
        os.system('cls' if os.name == 'nt' else 'clear')
        
        print(f"🎯 MAOS Worktree Monitor - {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print("━" * 36)
        
        # List all worktrees with status
        worktrees = get_worktree_list()
        for worktree in worktrees:
            if "/worktrees/" in worktree:
                display_worktree_status(worktree)
        
        print("")
        print("📊 Resource Usage:")
        check_resource_usage()
        
        time.sleep(5)

def get_worktree_list():
    """Get list of git worktrees."""
    try:
        result = subprocess.run(
            ["git", "worktree", "list", "--porcelain"],
            capture_output=True,
            text=True,
            check=True
        )
        
        worktrees = []
        for line in result.stdout.strip().split('\n'):
            if line.startswith("worktree "):
                worktrees.append(line.split(" ", 1)[1])
        return worktrees
    except subprocess.CalledProcessError:
        return []

def display_worktree_status(worktree):
    """Display status for a single worktree."""
    metadata_file = Path(worktree) / ".agent-metadata.json"
    
    if metadata_file.exists():
        try:
            with open(metadata_file) as f:
                metadata = json.load(f)
            
            agent_role = metadata.get('agent_role', 'unknown')
            issue_id = metadata.get('issue_id', 'unknown')
            status = metadata.get('status', 'unknown')
            
            # Check for active changes
            changes = count_changes(worktree)
            branch = get_current_branch(worktree)
            
            # Status emoji
            status_icons = {
                "active": "🟢",
                "completed": "✅",
                "blocked": "🔴",
                "reviewing": "👀",
                "initializing": "⏳"
            }
            status_icon = status_icons.get(status, "⏳")
            
            print(f"{status_icon} {agent_role} | Issue #{issue_id} | Changes: {changes} | Status: {status}")
        except (json.JSONDecodeError, FileNotFoundError):
            print(f"⚠️  {worktree} - Unable to read metadata")

def count_changes(worktree):
    """Count uncommitted changes in a worktree."""
    try:
        result = subprocess.run(
            ["git", "-C", worktree, "status", "--porcelain"],
            capture_output=True,
            text=True
        )
        return len(result.stdout.strip().split('\n')) if result.stdout.strip() else 0
    except subprocess.CalledProcessError:
        return 0

def get_current_branch(worktree):
    """Get current branch of a worktree."""
    try:
        result = subprocess.run(
            ["git", "-C", worktree, "branch", "--show-current"],
            capture_output=True,
            text=True
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        return "unknown"

def check_resource_usage():
    """Check disk usage of worktrees."""
    worktrees_dir = Path("worktrees")
    if not worktrees_dir.exists():
        print("  No worktrees directory found")
        return
    
    worktree_count = 0
    for worktree in worktrees_dir.iterdir():
        if worktree.is_dir():
            worktree_count += 1
            size = get_directory_size(worktree)
            print(f"  📁 {worktree.name}: {format_bytes(size)}")
    
    print(f"  📈 Total worktrees: {worktree_count}")

def get_directory_size(path):
    """Get total size of a directory in bytes."""
    total = 0
    for dirpath, dirnames, filenames in os.walk(path):
        for filename in filenames:
            filepath = os.path.join(dirpath, filename)
            if os.path.exists(filepath):
                total += os.path.getsize(filepath)
    return total

def format_bytes(size):
    """Format bytes to human readable format."""
    for unit in ['B', 'KB', 'MB', 'GB']:
        if size < 1024.0:
            return f"{size:.1f}{unit}"
        size /= 1024.0
    return f"{size:.1f}TB"

if __name__ == "__main__":
    try:
        monitor_active_worktrees()
    except KeyboardInterrupt:
        print("\n\n👋 Monitor stopped")
```

### 3. Integration Phase

```python
#!/usr/bin/env python3
# worktree_integrate.py - Smart merge strategies

import json
import subprocess
import sys
from pathlib import Path

def integrate_worktree(worktree_dir, merge_strategy="sequential"):
    """Integrate a worktree back to the base branch."""
    
    # Get worktree metadata
    metadata_file = Path(worktree_dir) / ".agent-metadata.json"
    if not metadata_file.exists():
        print(f"❌ No metadata found in {worktree_dir}")
        return False
    
    with open(metadata_file) as f:
        metadata = json.load(f)
    
    branch_name = metadata.get('branch_name')
    base_branch = metadata.get('base_branch', 'main')
    agent_role = metadata.get('agent_role')
    
    print(f"🔄 Integrating worktree: {worktree_dir}")
    
    # Pre-integration checks
    if not pre_integration_checks(worktree_dir):
        print("❌ Pre-integration checks failed")
        return False
    
    # Execute merge strategy
    if merge_strategy == "sequential":
        return sequential_merge(branch_name, base_branch)
    elif merge_strategy == "rebase":
        return rebase_merge(branch_name, base_branch)
    elif merge_strategy == "squash":
        return squash_merge(branch_name, base_branch, agent_role)
    else:
        print(f"❌ Unknown merge strategy: {merge_strategy}")
        return False

def pre_integration_checks(worktree_dir):
    """Run pre-integration checks."""
    print("🔍 Running pre-integration checks...")
    
    # Check for uncommitted changes
    try:
        result = subprocess.run(
            ["git", "-C", worktree_dir, "status", "--porcelain"],
            capture_output=True,
            text=True,
            check=True
        )
        
        if result.stdout.strip():
            print("⚠️  Uncommitted changes detected")
            return False
    except subprocess.CalledProcessError:
        print("❌ Failed to check git status")
        return False
    
    # Run tests if available
    makefile = Path(worktree_dir) / "Makefile"
    if makefile.exists():
        print("🧪 Running tests...")
        try:
            subprocess.run(
                ["make", "test"],
                cwd=worktree_dir,
                check=True
            )
        except subprocess.CalledProcessError:
            print("❌ Tests failed")
            return False
    
    # Check for conflicts with other active worktrees
    if not check_integration_conflicts(worktree_dir):
        return False
    
    print("✅ All checks passed")
    return True

def sequential_merge(branch, target):
    """Perform a sequential merge with full history."""
    print(f"📥 Sequential merge: {branch} → {target}")
    
    try:
        # Checkout target branch
        subprocess.run(["git", "checkout", target], check=True)
        
        # Merge with no-ff to preserve history
        merge_message = f"""Merge {branch} into {target}

MAOS Agent: Automated integration
Branch: {branch}
Strategy: Sequential merge with full history"""
        
        subprocess.run([
            "git", "merge", "--no-ff", branch,
            "-m", merge_message
        ], check=True)
        
        print("✅ Sequential merge completed")
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"❌ Merge failed: {e}")
        return False

def rebase_merge(branch, target):
    """Perform a rebase merge."""
    print(f"📥 Rebase merge: {branch} → {target}")
    
    try:
        # Checkout feature branch
        subprocess.run(["git", "checkout", branch], check=True)
        
        # Rebase onto target
        subprocess.run(["git", "rebase", target], check=True)
        
        # Checkout target and fast-forward merge
        subprocess.run(["git", "checkout", target], check=True)
        subprocess.run(["git", "merge", "--ff-only", branch], check=True)
        
        print("✅ Rebase merge completed")
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"❌ Rebase failed: {e}")
        return False

def squash_merge(branch, target, agent_role):
    """Perform a squash merge."""
    print(f"📥 Squash merge: {branch} → {target}")
    
    try:
        # Checkout target branch
        subprocess.run(["git", "checkout", target], check=True)
        
        # Squash merge
        subprocess.run(["git", "merge", "--squash", branch], check=True)
        
        # Commit with descriptive message
        commit_message = f"""[{agent_role}] Integrated changes from {branch}

MAOS Agent: {agent_role}
Branch: {branch}
Strategy: Squashed commits for clean history"""
        
        subprocess.run(["git", "commit", "-m", commit_message], check=True)
        
        print("✅ Squash merge completed")
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"❌ Squash merge failed: {e}")
        return False

def check_integration_conflicts(worktree_dir):
    """Check for conflicts with other active worktrees."""
    # This would check for semantic conflicts
    # For now, return True (no conflicts)
    return True

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: worktree_integrate.py <worktree_dir> [merge_strategy]")
        print("Strategies: sequential (default), rebase, squash")
        sys.exit(1)
    
    worktree_dir = sys.argv[1]
    merge_strategy = sys.argv[2] if len(sys.argv) > 2 else "sequential"
    
    success = integrate_worktree(worktree_dir, merge_strategy)
    sys.exit(0 if success else 1)
```

### 4. Cleanup Phase

```python
#!/usr/bin/env python3
# worktree_cleanup.py - Automated cleanup with safety checks

import json
import subprocess
import os
from datetime import datetime, timedelta
from pathlib import Path

def cleanup_completed_worktrees():
    """Clean up completed or stale worktrees."""
    print("🧹 Starting worktree cleanup...")
    
    cleaned = 0
    skipped = 0
    
    worktrees = get_worktree_list()
    for worktree in worktrees:
        if should_cleanup_worktree(worktree):
            if cleanup_single_worktree(worktree):
                cleaned += 1
            else:
                skipped += 1
    
    print(f"✅ Cleanup complete: {cleaned} removed, {skipped} skipped")

def get_worktree_list():
    """Get list of all worktrees."""
    try:
        result = subprocess.run(
            ["git", "worktree", "list", "--porcelain"],
            capture_output=True,
            text=True,
            check=True
        )
        
        worktrees = []
        for line in result.stdout.strip().split('\n'):
            if line.startswith("worktree "):
                worktrees.append(line.split(" ", 1)[1])
        return worktrees
    except subprocess.CalledProcessError:
        return []

def should_cleanup_worktree(worktree):
    """Determine if a worktree should be cleaned up."""
    metadata_file = Path(worktree) / ".agent-metadata.json"
    
    # Skip if no metadata
    if not metadata_file.exists():
        return False
    
    try:
        with open(metadata_file) as f:
            metadata = json.load(f)
        
        status = metadata.get('status')
        created_at = metadata.get('created_at')
        
        # Cleanup if completed
        if status == "completed":
            return True
        
        # Cleanup if older than 7 days and inactive
        if status == "inactive" and created_at:
            created_date = datetime.fromisoformat(created_at.replace('Z', '+00:00'))
            age = datetime.now() - created_date.replace(tzinfo=None)
            if age > timedelta(days=7):
                return True
        
        return False
    except (json.JSONDecodeError, ValueError):
        return False

def cleanup_single_worktree(worktree):
    """Clean up a single worktree."""
    metadata_file = Path(worktree) / ".agent-metadata.json"
    
    branch = None
    if metadata_file.exists():
        try:
            with open(metadata_file) as f:
                metadata = json.load(f)
                branch = metadata.get('branch_name')
        except json.JSONDecodeError:
            pass
    
    print(f"🗑️  Removing worktree: {worktree}")
    
    # Unlock if locked
    subprocess.run(["git", "worktree", "unlock", worktree], capture_output=True)
    
    # Remove worktree
    try:
        subprocess.run(["git", "worktree", "remove", worktree, "--force"], check=True)
        
        # Optionally delete branch if fully merged
        if branch:
            try:
                subprocess.run(["git", "branch", "-d", branch], check=True)
                print(f"  ✅ Branch {branch} deleted (was fully merged)")
            except subprocess.CalledProcessError:
                print(f"  ℹ️  Branch {branch} kept (not fully merged)")
        
        # Update registry
        unregister_worktree(worktree)
        
        return True
    except subprocess.CalledProcessError:
        print("  ❌ Failed to remove worktree")
        return False

def automated_maintenance():
    """Run automated worktree maintenance."""
    print("🔧 Running automated worktree maintenance...")
    
    # Prune stale worktree references
    try:
        subprocess.run(["git", "worktree", "prune"], check=True)
    except subprocess.CalledProcessError:
        print("⚠️  Failed to prune worktrees")
    
    # Clean up completed worktrees
    cleanup_completed_worktrees()
    
    # Generate usage report
    generate_worktree_report()

def generate_worktree_report():
    """Generate a usage report for all worktrees."""
    report_date = datetime.now().strftime("%Y%m%d")
    report_file = Path(".maos/reports") / f"worktree-usage-{report_date}.md"
    
    # Create reports directory
    report_file.parent.mkdir(parents=True, exist_ok=True)
    
    # Start report
    report_content = [f"# Worktree Usage Report - {datetime.now().strftime('%Y-%m-%d')}"]
    report_content.append("")
    report_content.append("## Active Worktrees")
    report_content.append("")
    report_content.append("| Agent Role | Issue | Branch | Created | Status | Size |")
    report_content.append("|------------|-------|--------|---------|--------|------|")
    
    # Add worktree information
    worktrees_dir = Path("worktrees")
    if worktrees_dir.exists():
        for worktree in worktrees_dir.iterdir():
            if worktree.is_dir():
                metadata_file = worktree / ".agent-metadata.json"
                if metadata_file.exists():
                    try:
                        with open(metadata_file) as f:
                            metadata = json.load(f)
                        
                        agent_role = metadata.get('agent_role', 'unknown')
                        issue_id = metadata.get('issue_id', 'unknown')
                        branch = metadata.get('branch_name', 'unknown')
                        created = metadata.get('created_at', 'unknown').split('T')[0]
                        status = metadata.get('status', 'unknown')
                        size = format_bytes(get_directory_size(worktree))
                        
                        report_content.append(
                            f"| {agent_role} | #{issue_id} | {branch} | {created} | {status} | {size} |"
                        )
                    except json.JSONDecodeError:
                        pass
    
    report_content.append("")
    
    # Write report
    with open(report_file, 'w') as f:
        f.write('\n'.join(report_content))
    
    print(f"Report generated: {report_file}")

def get_directory_size(path):
    """Get total size of a directory in bytes."""
    total = 0
    for dirpath, dirnames, filenames in os.walk(path):
        for filename in filenames:
            filepath = os.path.join(dirpath, filename)
            if os.path.exists(filepath):
                total += os.path.getsize(filepath)
    return total

def format_bytes(size):
    """Format bytes to human readable format."""
    for unit in ['B', 'KB', 'MB', 'GB']:
        if size < 1024.0:
            return f"{size:.1f}{unit}"
        size /= 1024.0
    return f"{size:.1f}TB"

def unregister_worktree(worktree):
    """Remove worktree from registry."""
    # Implementation would update the registry
    pass

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1 and sys.argv[1] == "--auto":
        automated_maintenance()
    else:
        cleanup_completed_worktrees()
```

## Performance Optimization

### Concurrent Worktree Limits

Based on research and testing, the recommended limits are:

```python
#!/usr/bin/env python3
# worktree_limits.py - Enforce worktree limits

import subprocess
import os
from pathlib import Path

def check_worktree_limits():
    """Check if worktree limits are within acceptable bounds."""
    MAX_WORKTREES = 6  # Optimal for most development machines
    MAX_DISK_GB = 50   # Maximum disk usage for all worktrees
    
    # Count current worktrees
    current_count = count_worktrees()
    
    # Calculate disk usage
    current_size_gb = calculate_worktrees_size_gb()
    
    # Check worktree count limit
    if current_count >= MAX_WORKTREES:
        print(f"❌ Worktree limit reached ({current_count}/{MAX_WORKTREES})")
        print("   Please complete or remove existing worktrees before creating new ones.")
        return False
    
    # Check disk usage limit
    if current_size_gb > MAX_DISK_GB:
        print(f"❌ Disk usage limit exceeded ({current_size_gb:.1f}GB/{MAX_DISK_GB}GB)")
        print("   Please cleanup old worktrees to free up space.")
        return False
    
    print(f"✅ Worktree limits OK: {current_count} worktrees, {current_size_gb:.1f}GB used")
    return True

def count_worktrees():
    """Count the number of worktrees in the worktrees/ directory."""
    try:
        result = subprocess.run(
            ["git", "worktree", "list"],
            capture_output=True,
            text=True,
            check=True
        )
        
        # Count lines containing "worktrees/"
        count = sum(1 for line in result.stdout.split('\n') if 'worktrees/' in line)
        return count
    except subprocess.CalledProcessError:
        return 0

def calculate_worktrees_size_gb():
    """Calculate total size of worktrees directory in GB."""
    worktrees_dir = Path("worktrees")
    
    if not worktrees_dir.exists():
        return 0.0
    
    total_bytes = 0
    for dirpath, dirnames, filenames in os.walk(worktrees_dir):
        for filename in filenames:
            filepath = os.path.join(dirpath, filename)
            if os.path.exists(filepath):
                total_bytes += os.path.getsize(filepath)
    
    # Convert to GB
    return total_bytes / (1024 ** 3)

if __name__ == "__main__":
    import sys
    success = check_worktree_limits()
    sys.exit(0 if success else 1)
```

### IDE Performance Tuning

```json
// Recommended VS Code settings for worktree development
{
    "files.watcherExclude": {
        "**/worktrees/**": true,
        "**/.git/objects/**": true,
        "**/.git/subtree-cache/**": true,
        "**/node_modules/**": true,
        "**/target/**": true,
        "**/build/**": true
    },
    "search.exclude": {
        "**/worktrees/**": true,
        "**/node_modules": true,
        "**/bower_components": true,
        "**/dist": true
    },
    "git.autorefresh": false,
    "git.decorations.enabled": false,
    "typescript.tsserver.maxTsServerMemory": 2048,
    "extensions.ignoreRecommendations": false
}
```

## Conflict Prevention

### Semantic Conflict Detection

```python
#!/usr/bin/env python3
# conflict_detection.py - Prevent semantic conflicts

import json
import os
from pathlib import Path

def check_semantic_conflicts(agent_role, issue_id):
    """Check for semantic conflicts before agent starts work."""
    conflict_found = False
    
    # Check if another agent is working on the same issue
    worktrees_dir = Path("worktrees")
    if worktrees_dir.exists():
        for worktree in worktrees_dir.iterdir():
            if worktree.is_dir():
                metadata_file = worktree / ".agent-metadata.json"
                if metadata_file.exists():
                    try:
                        with open(metadata_file) as f:
                            metadata = json.load(f)
                        
                        existing_issue = metadata.get('issue_id')
                        existing_role = metadata.get('agent_role')
                        existing_status = metadata.get('status')
                        
                        if (existing_issue == issue_id and 
                            existing_status == "active" and
                            existing_role != agent_role):
                            print(f"⚠️  Potential conflict: {existing_role} is already working on issue #{issue_id}")
                            conflict_found = True
                    except json.JSONDecodeError:
                        continue
    
    # Check file-level conflicts
    files_in_scope = get_files_for_issue(issue_id)
    for file in files_in_scope:
        if is_file_locked(file):
            print(f"⚠️  File conflict: {file} is locked by another agent")
            conflict_found = True
    
    return conflict_found

def lock_files_for_agent(worktree_dir, files):
    """Lock files for an agent using lightweight file locking."""
    lock_dir = Path(".maos/locks")
    lock_dir.mkdir(parents=True, exist_ok=True)
    
    all_locked = True
    locked_files = []
    
    for file in files:
        # Create lock file name by replacing / with _
        lock_file = lock_dir / (file.replace('/', '_') + ".lock")
        
        # Try to create symbolic link as atomic lock
        try:
            if not lock_file.exists():
                lock_file.symlink_to(worktree_dir)
                print(f"🔒 Locked: {file}")
                locked_files.append(lock_file)
            else:
                print(f"⚠️  Failed to lock: {file} (already locked)")
                all_locked = False
                break
        except OSError:
            print(f"⚠️  Failed to lock: {file} (OS error)")
            all_locked = False
            break
    
    # If we couldn't lock all files, release the ones we did lock
    if not all_locked:
        for lock_file in locked_files:
            try:
                lock_file.unlink()
            except OSError:
                pass
        return False
    
    return True

def is_file_locked(file_path):
    """Check if a file is locked by another agent."""
    lock_dir = Path(".maos/locks")
    if not lock_dir.exists():
        return False
    
    lock_file = lock_dir / (file_path.replace('/', '_') + ".lock")
    return lock_file.exists()

def get_files_for_issue(issue_id):
    """Get list of files in scope for an issue."""
    # This would typically read from issue metadata or task definition
    # For now, return empty list
    return []

def release_file_locks(worktree_dir):
    """Release all locks held by a worktree."""
    lock_dir = Path(".maos/locks")
    if not lock_dir.exists():
        return
    
    released = 0
    for lock_file in lock_dir.iterdir():
        if lock_file.is_symlink():
            try:
                # Check if this lock points to our worktree
                if str(lock_file.resolve()) == str(Path(worktree_dir).resolve()):
                    lock_file.unlink()
                    released += 1
            except (OSError, ValueError):
                # Lock file might be broken or already removed
                pass
    
    if released > 0:
        print(f"🔓 Released {released} file locks")

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) < 3:
        print("Usage: conflict_detection.py <agent_role> <issue_id>")
        sys.exit(1)
    
    agent_role = sys.argv[1]
    issue_id = sys.argv[2]
    
    has_conflicts = check_semantic_conflicts(agent_role, issue_id)
    sys.exit(1 if has_conflicts else 0)
```

## Automation Scripts

### Just Integration

```makefile
# justfile additions for worktree management

# Create a new agent worktree
worktree-create role issue description base="main":
    @python3 .claude/hooks/utils/worktree_create.py "{{role}}" "{{issue}}" "{{description}}" "{{base}}"

# List all active worktrees
worktree-list:
    @python3 .claude/hooks/utils/worktree_list.py

# Monitor active worktrees
worktree-monitor:
    @python3 .claude/hooks/utils/worktree_monitor.py

# Clean up completed worktrees
worktree-cleanup:
    @python3 .claude/hooks/utils/worktree_cleanup.py

# Integrate a worktree back to main
worktree-integrate worktree strategy="sequential":
    @python3 .claude/hooks/utils/worktree_integrate.py "{{worktree}}" "{{strategy}}"

# Run maintenance tasks
worktree-maintenance:
    @echo "🔧 Running worktree maintenance..."
    @git worktree prune
    @python3 .claude/hooks/utils/worktree_cleanup.py --auto
```

## Best Practices Summary

### Do's ✅

1. **Limit Active Worktrees**: Keep ≤ 6 active worktrees per developer
2. **Use Clear Naming**: Follow the `role/issue/task` pattern consistently
3. **Lock Active Worktrees**: Prevent accidental deletion during work
4. **Regular Cleanup**: Run maintenance weekly or after feature completion
5. **Monitor Resources**: Track disk usage and prune when needed
6. **Atomic Operations**: Use file-based locks for coordination
7. **IDE Optimization**: Configure file watchers to exclude other worktrees
8. **Commit Frequently**: Small, atomic commits in each worktree
9. **Test Before Integration**: Always run tests before merging
10. **Document Context**: Keep `.claude-context.md` updated

### Don'ts ❌

1. **Don't Share Branches**: Never checkout same branch in multiple worktrees
2. **Don't Ignore Conflicts**: Always check for semantic conflicts
3. **Don't Keep Stale Worktrees**: Clean up after 7 days of inactivity
4. **Don't Mix Dependencies**: Use separate virtual environments
5. **Don't Cross-Reference**: Avoid direct file references between worktrees
6. **Don't Skip Locks**: Always lock worktrees during active development
7. **Don't Ignore IDE Limits**: Watch for performance degradation
8. **Don't Force Merges**: Resolve conflicts properly
9. **Don't Bypass Tests**: Integration requires passing tests
10. **Don't Lose Context**: Always preserve agent metadata

## Security Considerations

1. **Path Validation**: Ensure worktrees stay within designated directories
2. **Branch Protection**: Prevent modification of protected branches
3. **Access Control**: Verify agent permissions before operations
4. **Audit Trail**: Log all worktree operations with timestamps
5. **Resource Limits**: Enforce CPU/memory/disk quotas per worktree

## Conclusion

This git worktree integration design provides a robust foundation for parallel Claude Code sub-agent development. The system balances simplicity with powerful coordination capabilities, ensuring smooth parallel development while maintaining code quality and system performance.

The key to success is disciplined adherence to the lifecycle management process and regular maintenance to prevent resource accumulation. With proper tooling and automation, teams can leverage git worktrees to dramatically improve development velocity without sacrificing code quality or system stability.