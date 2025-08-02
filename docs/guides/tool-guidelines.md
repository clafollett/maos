# Claude Code Sub-Agent Tool Guidelines

This document provides guidance on which tools should be available to different types of sub-agents. Remember: tools enable capabilities, and overly restricting them limits agent effectiveness.

## Core Principle
**Start generous, restrict only when necessary.** It's better to have a tool and not need it than to need a tool and not have it.

## Essential Tools by Category

### 🔍 Discovery & Analysis Tools
- **Read**: Essential for ALL agents - needed to understand code, documentation, configurations
- **Grep**: Critical for searching codebases, finding patterns, understanding implementations  
- **Glob**: Important for finding files by pattern, understanding project structure
- **LS**: Necessary for exploring directories, understanding project layout
- **WebSearch**: CRUCIAL for looking up documentation, best practices, solutions, and API references

### 📝 Creation & Modification Tools
- **Write**: For creating new files (documentation, code, configs)
- **Edit**: For making single changes to existing files
- **MultiEdit**: For complex refactoring or multiple changes

### 🔧 Execution & System Tools
- **Bash**: For running commands, tests, builds, installations
- **Task**: For delegating specialized work to other agents
- **TodoWrite**: For tracking complex multi-step work

### 🔬 Specialized Tools
- **WebFetch**: For analyzing specific web pages or documentation
- **NotebookRead/NotebookEdit**: For Jupyter notebook work

## Tool Recommendations by Agent Type

### 🏗️ Architects (API, Application, Data, Solution)
**Must Have**: Read, Write, Edit, MultiEdit, Grep, Glob, LS, WebSearch
**Strongly Recommended**: Task (for delegating implementation), TodoWrite (for complex designs)
**Optional**: Bash (for prototyping), WebFetch (for specific research)

### 💻 Engineers (Backend, Frontend, Mobile, DevOps)
**Must Have**: Read, Write, Edit, MultiEdit, Bash, Grep, Glob, LS, WebSearch
**Strongly Recommended**: Task (for delegating specialized work), TodoWrite (for tracking implementation)
**Optional**: WebFetch (for specific documentation)

### 📊 Analysts (Business, Data Scientist, Researcher)
**Must Have**: Read, Write, Grep, Glob, LS, WebSearch, WebFetch
**Strongly Recommended**: Edit, MultiEdit (for reports), TodoWrite (for research tracking)
**Context Dependent**: Bash (for data scientists running analyses)

### ✅ Quality & Security (QA, Tester, Code Reviewer, Security)
**Must Have**: Read, Grep, Glob, LS, WebSearch
**Strongly Recommended**: Write (for reports), Edit (for fixes), Bash (for running tests)
**Important**: Task (for delegating fixes), TodoWrite (for tracking issues)

### 📚 Documentation & Design (Tech Writer, UX Designer, PRD Specialist)
**Must Have**: Read, Write, Edit, MultiEdit, Grep, Glob, LS, WebSearch
**Strongly Recommended**: WebFetch (for research), TodoWrite (for content planning)
**Optional**: Task (for delegating technical reviews)

### 🔧 Operations & MAOS Specialists
**Must Have**: Read, Write, Edit, MultiEdit, Bash, Grep, Glob, LS
**Strongly Recommended**: Task (for orchestration), TodoWrite (for complex workflows)
**Context Dependent**: WebSearch (for troubleshooting)

## Why These Tools Matter

### WebSearch is ESSENTIAL
- Engineers need to look up framework documentation
- Architects need to research best practices and patterns
- Security specialists need to check CVE databases
- Everyone needs Stack Overflow sometimes!

### Task Enables Collaboration
- Architects can delegate implementation details
- Engineers can spawn specialized testers
- Complex problems need multiple perspectives

### TodoWrite Prevents Dropped Balls
- Multi-step implementations need tracking
- Complex analyses have many phases
- Architectural decisions have many considerations

### LS is Fundamental
- Can't navigate without seeing what's there
- Project structure understanding is critical
- Finding the right files requires exploration

## Guidelines for Meta-Agent

When creating or editing agents:

1. **Start with the full recommended set** for that agent type
2. **Only remove tools if there's a specific reason** (e.g., security constraints)
3. **Always include WebSearch** unless the agent is specifically offline-only
4. **Never remove Read, Grep, or Glob** - these are fundamental
5. **Include Task for any agent that might need specialized help**
6. **Include TodoWrite for any agent doing complex multi-step work**

Remember: Sub-agents have ZERO context from the user conversation. They need tools to discover, understand, and complete their work effectively. Don't handcuff them by being overly restrictive!