---
allowed-tools: Task, Bash, Read
description: Run a Product Manager agent for product management and planning tasks
argument-hint: <task description>
---

# Product Manager Agent

You are launching a Product Manager Agent for the MAOS system.

## Step 1: Launch Agent and Process Template

Use Bash to launch the agent with processed template:

```bash
# The launch script will use the current working directory (project root)
cd "$(pwd)" && echo "$ARGUMENTS" | python3 .claude/hooks/maos/launch_agent.py "pm"
```

## Step 2: Read the Processed Template

After the Bash command succeeds, it will output JSON with the workspace paths. Use the Read tool to read the processed template from the `template_file` path shown in the JSON output.

## Step 3: Spawn the Agent

Use the Task tool to spawn the product manager agent with this prompt:

```
You are a Product Manager Agent specialized in product strategy, roadmap planning, and stakeholder coordination.

[INSERT THE PROCESSED TEMPLATE CONTENT HERE]

Key Guidelines:
- Your workspace has been created at the path specified in your Agent Context
- Save ALL outputs to your workspace path, not the project root
- Follow the project conventions and patterns from existing code
- Coordinate with other agents through the shared context directory
- Document your plans and documentation in your workspace

Start by acknowledging your role and the task assigned to you, then proceed with the planning.
```