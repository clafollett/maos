# MAOS - Multi-Agent Orchestration System

**An intelligent MCP server that orchestrates specialized AI agents for complex software development workflows.**

## Overview

MAOS (Multi-Agent Orchestration System) is a sophisticated orchestration platform that coordinates multiple specialized AI agents to tackle complex software development projects. Instead of relying on a single generalist AI, MAOS spawns and coordinates teams of specialist agents, each optimized for specific domains like architecture, engineering, security, and data modeling.

## Quick Start

### 🚀 **Automated Development Environment**

MAOS includes a fully automated development environment that ensures consistency across all contributors and CI environments:

```bash
# 1. Clone the repository
git clone https://github.com/clafollett/maos.git
cd maos

# 2. Install just task runner (one-time setup)
cargo install just

# 3. One-command setup (installs everything automatically)
source stack.env && just dev-setup
```

**What this does automatically:**
- ✅ Validates required Rust toolchain versions
- ✅ Installs all development dependencies  
- ✅ Sets up git hooks for quality enforcement
- ✅ Runs full quality pipeline (format, lint, test, security audit)
- ✅ Configures VS Code with optimal Rust settings

**Enforced Standards:**
- 🔧 **Rust Toolchain**: Automatically pinned via `rust-toolchain.toml`
- 📝 **Code Formatting**: `rustfmt` with MAOS configuration  
- 🔍 **Code Quality**: `clippy` with strict warnings-as-errors
- 🧪 **Testing**: Full test suite with coverage tracking
- 🔒 **Security**: Automated vulnerability scanning
- 🪝 **Git Hooks**: Pre-commit validation of all quality gates

### 🛠️ **Development Workflow**

```bash
# All development commands via 'just' task runner
just                # List all available commands
just ci             # Run full CI pipeline locally  
just pre-commit     # Run all quality checks
just test           # Run tests
just format         # Format code
just lint           # Run clippy lints
just audit          # Security audit
```

**CI/CD Integration**: Our GitHub Actions automatically enforce the same environment standards, ensuring "no surprises" between local development and CI.

## Architecture Philosophy

### Specialized Agent Teams
MAOS embraces the "best of breed" approach with **20 specialized agent roles**:

- **1 Meta-Role**: Orchestrator agent for intelligent workflow coordination
- **5 Architect Specialists**: Solution, Application, Data, API, and Security architects
- **3 Engineer Specialists**: Backend, Frontend, and Mobile engineers  
- **11 Domain Specialists**: Researchers, QA, DevOps, Data Scientists, UX Designers, and more

### Adaptive Phase-Based Orchestration
Rather than attempting to plan entire projects upfront, MAOS uses adaptive phase-based orchestration:

- **Discovery-First**: Always begins with research and analysis phases
- **Incremental Planning**: Plans 1-2 phases ahead based on current knowledge
- **Adaptive Evolution**: Modifies plans based on actual phase outputs and discoveries
- **Intelligent Coordination**: The Orchestrator agent continuously refines strategy

### Claude 4 Optimized
MAOS leverages the latest Claude 4 models strategically:

- **Claude 4 Opus**: Ultimate reasoning for Orchestrator and complex architecture decisions
- **Claude 4 Sonnet**: Advanced technical work for implementation and design

## Key Features

### 🎯 **Domain Expertise**
Each agent role is optimized for specific expertise areas with tailored prompts, capabilities, and resource allocations.

### 🔄 **Adaptive Planning**
The Orchestrator learns from each phase completion, adapting subsequent phases based on discovered complexity and requirements.

### 🏗️ **File-Based Communication**
Agents communicate through structured file systems, enabling persistence, debugging, and integration with any CLI tool.

### 📊 **Session Management**
Comprehensive state tracking with recovery capabilities, allowing orchestrations to survive interruptions and resume seamlessly.

### 🔧 **MCP Integration**
Built as an MCP (Model Context Protocol) server, MAOS integrates natively with Claude Code and other MCP-compatible tools.

## Architecture Decisions

MAOS is built on a foundation of carefully considered architectural decisions:

- **[ADR-01](docs/architecture/decisions/01-use-ddd-architecture.md)**: Domain-Driven Design Architecture
- **[ADR-02](docs/architecture/decisions/02-hybrid-storage-strategy.md)**: Hybrid Storage Strategy (SQLite + File System)
- **[ADR-03](docs/architecture/decisions/03-session-management.md)**: Session Orchestration and State Management
- **[ADR-10](docs/architecture/decisions/10-mcp-server-architecture.md)**: MCP Server Architecture
- **[ADR-11](docs/architecture/decisions/11-adaptive-phase-based-orchestration.md)**: Adaptive Phase-Based Orchestration

[View all architectural decisions →](docs/architecture/decisions/)

## Documentation

### 📚 **Comprehensive Architecture**
- **[Agent Roles Reference](docs/architecture/references/agent-roles.md)**: Complete guide to all 20 specialist roles
- **[Model Selection Guide](docs/architecture/references/model-selection-guide.md)**: Strategic Claude model selection for optimal performance
- **[Phase Patterns](docs/architecture/references/phase-patterns.md)**: Proven orchestration patterns for different project types
- **[Orchestrator Specification](docs/architecture/references/orchestrator-specification.md)**: Deep dive into the meta-orchestration agent

### 🛠️ **Implementation Guides**
- **[Role Templates](assets/agent-roles/)**: Detailed prompt templates for each specialist role
- **[MCP Tools](docs/architecture/references/mcp-tools.md)**: MCP server tool definitions and usage
- **[Agent Integration Strategy](docs/AGENT_INTEGRATION_STRATEGY.md)**: Practical implementation approaches

## Development Status

MAOS is currently in the **architecture and design phase**. The comprehensive documentation and architectural decisions have been finalized, establishing the foundation for implementation.

### ✅ **Completed**
- Complete architectural design and ADR documentation
- 20 specialized agent role definitions with detailed templates
- Model selection optimization for Claude 4 generation
- Phase-based orchestration patterns and strategies
- MCP server architecture specification

### 🚧 **In Progress**
- Core MCP server implementation
- Agent lifecycle management system
- Session orchestration engine
- File-based communication infrastructure

## Why MAOS?

### **Beyond Single-Agent Limitations**
Traditional AI development tools rely on a single generalist agent. MAOS recognizes that complex software projects benefit from specialized expertise, just like human development teams.

### **Proven Orchestration Patterns**
MAOS codifies proven software development workflows into reusable orchestration patterns, from simple sequential tasks to complex parallel architectures.

### **Enterprise-Ready Architecture**
Built with enterprise requirements in mind: session persistence, state recovery, audit trails, and comprehensive error handling.

### **Tool Integration**
Designed to work with existing development tools and workflows through file-based communication and MCP protocol integration.

## Project Structure

```
maos/
├── docs/                         # Comprehensive architecture documentation
│   ├── architecture/
│   │   ├── decisions/            # Architectural Decision Records (ADRs)
│   │   └── references/           # Reference documentation
│   │       ├── role-templates/   # Detailed agent role templates
│   │       └── *.md              # Architecture guides and patterns
│   └── *.md                      # Integration and strategy documentation
├── maos-poc/                     # Proof of concept implementations
└── README.md                     # This file
```

## Contributing

MAOS is being developed as an open architecture for multi-agent orchestration. The current focus is on completing the core implementation based on the established architectural foundation.

## License

[License to be determined]

---

*MAOS represents a new paradigm in AI-assisted software development, moving from single-agent limitations to coordinated specialist teams that mirror how human development teams operate most effectively.*