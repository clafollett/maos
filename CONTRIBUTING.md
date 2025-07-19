# Contributing to MAOS

Welcome to the Multi-Agent Orchestration System (MAOS) project! This guide will help you set up your development environment and contribute effectively.

## 🚀 Quick Start

### 1. Development Environment Setup

```bash
# Clone the repository
git clone https://github.com/clafollett/maos.git
cd maos

# Source the development stack environment
source stack.env

# Set up development environment (requires 'just' command runner)
just dev-setup
```

### 2. Install Just Command Runner

MAOS uses [`just`](https://github.com/casey/just) for development task automation:

```bash
# macOS
brew install just

# Linux
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/bin

# Windows (PowerShell)
winget install --id Casey.Just --exact

# Or via Cargo (all platforms)
cargo install just
```

### 3. Essential Commands

```bash
# See all available commands
just

# Run pre-commit checks
just pre-commit

# Development workflow
just format    # Format code
just lint      # Run clippy
just test      # Run tests
just build     # Build project
```

## 📋 Development Stack

Our development environment is defined in `stack.env` to ensure consistency across all contributors and CI environments. This prevents "works on my machine" issues.

### Required Tools

- **Rust**: Stable toolchain (defined in `rust-toolchain.toml`)
- **Just**: Task runner for development workflows
- **cargo-audit**: Security vulnerability scanner (auto-installed via `just dev-setup`)

### Stack Validation

```bash
# Load environment and validate versions
source stack.env
just validate-stack
```

## 🛠️ Development Workflow

### 1. Before You Start

1. Read the [Development Workflow](docs/DEVELOPMENT_WORKFLOW.md) document
2. Check the [GitHub Project Board](https://github.com/clafollett/maos/projects/1) for available issues
3. Pick an issue labeled `status:ready`

### 2. Branch Naming Convention

```
<type>/issue-<number>/<brief-description>

Examples:
- feature/issue-15/agent-registration
- fix/issue-23/session-cleanup  
- docs/issue-45/api-documentation
```

### 3. Development Process

```bash
# 1. Create and switch to feature branch
git checkout -b feature/issue-X/description

# 2. Make your changes following TDD principles
just test      # Write failing tests first
# ... implement code ...
just test      # Make tests pass
just format    # Format code
just lint      # Fix any linting issues

# 3. Run full pre-commit checks
just pre-commit

# 4. Commit your changes
git add .
git commit -m "feat: implement feature X (#issue-number)"

# 5. Push and create PR
git push -u origin feature/issue-X/description
gh pr create --title "Feature: Description" --body "Closes #X"
```

### 4. Code Quality Standards

All code must pass these quality gates:

- ✅ **Formatting**: `just format-check`
- ✅ **Linting**: `just lint` (zero warnings with `-D warnings`)
- ✅ **Tests**: `just test` (all tests pass, >90% coverage goal)
- ✅ **Security**: `just audit` (no known vulnerabilities)
- ✅ **Compilation**: `just build` (clean build)

### 5. Git Hooks

MAOS automatically sets up git hooks during `just dev-setup` to catch issues before committing:

```bash
# Git hooks are installed automatically with dev-setup
just dev-setup

# Or install hooks manually
just setup-git-hooks

# Test quality checks manually
just pre-commit
```

The git hooks will automatically run `just pre-commit` before each commit, ensuring all quality gates pass.

## 🏗️ Architecture Guidelines

MAOS follows Domain-Driven Design (DDD) principles with Clean Architecture:

### Project Structure

```
crates/
├── maos/           # Main binary (CLI + MCP server)
├── maos-domain/    # Domain models and business logic
├── maos-app/       # Use cases and application services  
└── maos-io/        # Technical implementations (I/O)
```

### Coding Standards

1. **Test-First Development**: Write failing tests before implementation
2. **Domain-Driven Design**: Keep business logic in the domain layer
3. **Clean Architecture**: Dependencies point inward to domain
4. **Modern Rust**: Use Rust 2024 edition with latest idioms
5. **Documentation**: Document public APIs with examples

### String Formatting (Rust 1.88.0+)

Use inline format strings (enforced by clippy):

```rust
// ✅ Good - Modern Rust style
let agent_id = 42;
let endpoint = "http://example.com";

println!("Processing agent: {agent_id}");
error!("Failed to connect to {endpoint}");

// ❌ Bad - Legacy style
println!("Processing agent: {}", agent_id);
error!("Failed to connect to {}", endpoint);
```

## 🧪 Testing

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_happy_path() {
        // Test the main success scenario
    }

    #[tokio::test]
    async fn test_error_cases() {
        // Test error conditions
    }

    #[tokio::test]
    async fn test_edge_cases() {
        // Test boundary conditions
    }
}
```

### Running Tests

```bash
just test           # All tests
just test-coverage  # With coverage report (requires cargo-tarpaulin)
cargo test specific_test  # Single test
```

## 📝 Commit Standards

Follow Conventional Commits with GitHub issue linking:

```
<type>: <description> (#<issue_number>)

Types:
- feat: New features (minor version bump)
- fix: Bug fixes (patch version bump)  
- chore: Maintenance (no version bump)
- docs: Documentation (no version bump)
- refactor: Code refactoring (no version bump)
- test: Adding/updating tests (no version bump)

Examples:
- feat: implement agent registration system (#15)
- fix: resolve task assignment race condition (#23)
- docs: add API documentation for orchestration (#42)
```

## 🔄 Pull Request Process

### 1. PR Requirements

- [ ] All quality gates pass (CI will check)
- [ ] Tests added/updated for new functionality
- [ ] Documentation updated if needed
- [ ] Code follows DDD and Clean Architecture principles
- [ ] Commit messages follow conventional format

### 2. Review Process

1. **Automated Checks**: CI runs all quality gates
2. **Code Review**: At least one approving review required
3. **Merge**: Squash and merge to main after approval

### 3. PR Template

Your PR should include:

```markdown
## Summary
Brief description of changes.

## Related Issue
Closes #[issue number]

## Type of Change
- [ ] Bug fix
- [ ] New feature  
- [ ] Refactoring
- [ ] Documentation

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Coverage maintained >90%

## Checklist
- [ ] Code follows DDD principles
- [ ] Documentation updated
- [ ] All CI checks pass
```

## 🆘 Getting Help

- **Questions**: Create a discussion in the GitHub repository
- **Bugs**: Create an issue with reproduction steps
- **Features**: Create an issue with detailed requirements
- **Development**: Check [DEVELOPMENT_WORKFLOW.md](docs/DEVELOPMENT_WORKFLOW.md)

## 🔧 IDE Setup

### VS Code (Recommended)

MAOS includes VS Code configuration for optimal development experience:

```bash
# Install recommended extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension skellock.just
code --install-extension tamasfe.even-better-toml
```

**Included configurations:**
- **settings.json**: Rust Analyzer with clippy integration, format on save
- **extensions.json**: Recommended extensions for Rust development
- **launch.json**: Debug configurations for MAOS binary and tests

### Other IDEs

**IntelliJ IDEA / CLion:**
- Install Rust plugin
- Configure rustfmt and clippy integration
- Set up run configurations for `just` commands

**Vim/Neovim:**
- Use rust-analyzer with your LSP client
- Configure rustfmt for format on save
- Set up just command integration

### Editor-agnostic Setup

All editors should be configured to:
1. Use rust-analyzer as the language server
2. Run rustfmt on save with our rustfmt.toml
3. Show clippy lints inline with `-D warnings`
4. Exclude target/ directory from file watching

## 📚 Additional Resources

- [Development Workflow](docs/DEVELOPMENT_WORKFLOW.md) - Detailed development process
- [Architecture Decisions](docs/architecture/decisions/) - ADR documents
- [Domain Model](docs/domain-model/) - Business logic documentation
- [API Specifications](docs/specifications/) - Technical specifications

---

Thank you for contributing to MAOS! 🚀 Together we're building the future of multi-agent orchestration.