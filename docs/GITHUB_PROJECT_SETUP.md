# GitHub Project Setup Guide

This guide provides step-by-step instructions for setting up the MAOS project on GitHub with proper issue templates, project board, and automation.

## Repository Configuration

### 1. Repository Settings

```bash
# Repository settings to configure manually:
```

**General Settings:**
- Repository name: `maos`
- Description: "Multi-Agent Orchestration System - Production-grade agent coordination in Rust"
- Visibility: Private
- Features to enable:
  - [x] Issues
  - [x] Projects
  - [x] Wiki (for architecture diagrams)
  - [x] Discussions (for design discussions)
  - [x] Security and analysis

**Branch Protection Rules:**
- Branch: `main`
- Settings:
  - [x] Require pull request reviews before merging
  - [x] Require status checks to pass before merging
  - [x] Require branches to be up to date before merging
  - [x] Require conversation resolution before merging
  - [x] Restrict pushes that create files larger than 100MB

**Required Status Checks:**
- `test-suite-ubuntu`
- `test-suite-macos`
- `test-suite-windows`
- `clippy-check`
- `fmt-check`
- `security-audit`

### 2. Project Board Setup

**Project Name:** MAOS Development
**Template:** Automated kanban with reviews

**Custom Columns:**
1. **📋 Backlog** - All planned work
2. **📝 Planning** - Needs specification
3. **🎯 Ready** - Ready for development
4. **🚧 In Progress** - Currently being worked on
5. **👀 Review** - In code review
6. **🔧 Testing** - In testing phase
7. **✅ Done** - Completed

**Automation Rules:**
- Issues → Auto-add to Backlog
- PR opened → Move to Review
- PR merged → Move to Done
- Issue closed → Move to Done

### 3. Labels Configuration

**Type Labels:**
```yaml
- name: "type:epic"
  color: "8B5CF6"
  description: "Large feature spanning multiple issues"
  
- name: "type:feature"
  color: "22C55E"
  description: "New functionality"
  
- name: "type:bug"
  color: "EF4444"
  description: "Something isn't working"
  
- name: "type:refactor"
  color: "3B82F6"
  description: "Code improvement without feature changes"
  
- name: "type:docs"
  color: "8B5CF6"
  description: "Documentation updates"
  
- name: "type:test"
  color: "F59E0B"
  description: "Adding or updating tests"
  
- name: "type:chore"
  color: "6B7280"
  description: "Maintenance tasks"
```

**Priority Labels:**
```yaml
- name: "priority:critical"
  color: "DC2626"
  description: "Must be fixed immediately"
  
- name: "priority:high"
  color: "EA580C"
  description: "Should be completed soon"
  
- name: "priority:medium"
  color: "CA8A04"
  description: "Can be scheduled normally"
  
- name: "priority:low"
  color: "65A30D"
  description: "Nice to have"
```

**Component Labels:**
```yaml
- name: "component:domain"
  color: "1E40AF"
  description: "Domain layer changes"
  
- name: "component:application"
  color: "1E40AF"
  description: "Application layer changes"
  
- name: "component:infrastructure"
  color: "1E40AF"
  description: "Infrastructure layer changes"
  
- name: "component:cli"
  color: "1E40AF"
  description: "CLI/Presentation layer changes"
  
- name: "component:docs"
  color: "7C3AED"
  description: "Documentation changes"
```

**Status Labels:**
```yaml
- name: "status:blocked"
  color: "DC2626"
  description: "Blocked by dependency"
  
- name: "status:needs-spec"
  color: "F59E0B"
  description: "Needs detailed specification"
  
- name: "status:ready"
  color: "22C55E"
  description: "Ready for development"
  
- name: "status:in-review"
  color: "8B5CF6"
  description: "Undergoing code review"
```

### 4. Milestones

```markdown
**v0.0.1 - Project Foundation**
- Due: Week 1
- Description: Complete project setup and planning
- Issues: 10-15 setup and planning issues

**v0.1.0 - Domain Model**
- Due: Week 3
- Description: Core domain model with full test coverage
- Issues: 20-30 domain implementation issues

**v0.2.0 - Application Layer**
- Due: Week 5
- Description: Use cases and application services
- Issues: 15-25 application layer issues

**v0.3.0 - Infrastructure**
- Due: Week 7
- Description: Persistence, messaging, and adapters
- Issues: 25-35 infrastructure issues

**v0.4.0 - CLI Interface**
- Due: Week 9
- Description: Complete CLI with all commands
- Issues: 15-20 CLI issues

**v0.5.0 - Release Candidate**
- Due: Week 10
- Description: Integration testing and documentation
- Issues: 10-15 testing and polish issues
```

## Issue Templates

### 1. Epic Template

```markdown
---
name: Epic
about: Large feature spanning multiple issues
title: 'EPIC: '
labels: 'type:epic'
assignees: ''
---

## Epic Overview
[Brief description of the epic]

## Business Value
[Why is this epic important?]

## Acceptance Criteria
- [ ] [High-level acceptance criteria]
- [ ] [Another criteria]

## Technical Approach
[High-level technical approach]

## Child Issues
- [ ] #[issue] - [Brief description]
- [ ] #[issue] - [Brief description]
- [ ] #[issue] - [Brief description]

## Definition of Done
- [ ] All child issues completed
- [ ] Integration tests pass
- [ ] Documentation updated
- [ ] Code reviewed and approved
```

### 2. Feature Template

```markdown
---
name: Feature
about: New functionality
title: '[Component] '
labels: 'type:feature'
assignees: ''
---

## Overview
[Brief description of the feature]

## Acceptance Criteria
- [ ] [Specific, testable criteria]
- [ ] [Another criteria]
- [ ] All tests pass with >90% coverage
- [ ] Documentation updated

## Technical Specification

### Domain Model (if applicable)
```rust
// Example domain structure
```

### API Design (if applicable)
```rust
// Public API interfaces
```

### Database Schema (if applicable)
```sql
-- Schema changes
```

## Test Scenarios

### Happy Path
1. **Test:** [Description]
   - **Input:** [Input data]
   - **Expected:** [Expected outcome]

### Error Cases
1. **Test:** [Error scenario]
   - **Input:** [Invalid input]
   - **Expected:** [Error response]

### Edge Cases
1. **Test:** [Edge case]
   - **Input:** [Edge case input]
   - **Expected:** [Expected behavior]

## Implementation Notes
- Follow DDD principles
- Use TDD Red/Green/Refactor cycle
- Consider performance implications
- Add appropriate logging

## Dependencies
- **Blocked by:** [List blocking issues]
- **Blocks:** [List issues this blocks]

## Definition of Done
- [ ] Implementation complete
- [ ] Unit tests written and passing
- [ ] Integration tests (if applicable)
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] Code reviewed and approved
- [ ] PR merged to main
```

### 3. Bug Template

```markdown
---
name: Bug Report
about: Report a bug or issue
title: '[BUG] '
labels: 'type:bug'
assignees: ''
---

## Bug Description
[Clear description of the bug]

## Steps to Reproduce
1. [Step 1]
2. [Step 2]
3. [Step 3]

## Expected Behavior
[What should happen]

## Actual Behavior
[What actually happens]

## Environment
- OS: [e.g., macOS 12.0]
- Rust version: [e.g., 1.70.0]
- MAOS version: [e.g., 0.1.0]

## Logs/Screenshots
[Add any relevant logs or screenshots]

## Additional Context
[Any other relevant information]
```

## Initial Issues to Create

### Phase 1: Setup (Week 1)

1. **EPIC: Project Foundation**
   - Labels: `type:epic`, `priority:critical`
   - Milestone: `v0.0.1`

2. **Setup GitHub Project Board**
   - Labels: `type:chore`, `priority:high`, `component:docs`
   - Milestone: `v0.0.1`

3. **Configure CI/CD Pipeline**
   - Labels: `type:chore`, `priority:high`
   - Milestone: `v0.0.1`

4. **Create Architecture Documentation**
   - Labels: `type:docs`, `priority:high`, `component:docs`
   - Milestone: `v0.0.1`

5. **Design Domain Model**
   - Labels: `type:docs`, `priority:high`, `component:domain`
   - Milestone: `v0.0.1`

6. **Create API Specification**
   - Labels: `type:docs`, `priority:high`, `component:cli`
   - Milestone: `v0.0.1`

7. **Setup Rust Workspace**
   - Labels: `type:chore`, `priority:high`
   - Milestone: `v0.0.1`

8. **Write ADRs for Key Decisions**
   - Labels: `type:docs`, `priority:medium`, `component:docs`
   - Milestone: `v0.0.1`

### Phase 2: Domain Model (Weeks 2-3)

9. **EPIC: Domain Model Implementation**
   - Labels: `type:epic`, `priority:critical`
   - Milestone: `v0.1.0`

10. **Implement Agent Aggregate**
    - Labels: `type:feature`, `priority:high`, `component:domain`
    - Milestone: `v0.1.0`

11. **Implement Task Aggregate**
    - Labels: `type:feature`, `priority:high`, `component:domain`
    - Milestone: `v0.1.0`

12. **Create Value Objects**
    - Labels: `type:feature`, `priority:high`, `component:domain`
    - Milestone: `v0.1.0`

13. **Implement Domain Events**
    - Labels: `type:feature`, `priority:high`, `component:domain`
    - Milestone: `v0.1.0`

14. **Create Domain Services**
    - Labels: `type:feature`, `priority:medium`, `component:domain`
    - Milestone: `v0.1.0`

## Automation Setup

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: CI/CD Pipeline

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable]
    
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run tests
      run: cargo test --all-features
    
    - name: Generate coverage
      run: cargo tarpaulin --out Xml
    
    - name: Upload coverage
      uses: codecov/codecov-action@v3

  lint:
    name: Linting
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy, rustfmt
    
    - name: Check formatting
      run: cargo fmt --all -- --check
    
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
    - name: Install cargo-audit
      run: cargo install cargo-audit
    - name: Run security audit
      run: cargo audit
```

### Project Board Automation

```yaml
# .github/workflows/project-board.yml
name: Project Board Automation

on:
  issues:
    types: [opened, closed, reopened]
  pull_request:
    types: [opened, closed, reopened]

jobs:
  project-board:
    runs-on: ubuntu-latest
    steps:
    - name: Add issue to project
      uses: actions/add-to-project@v0.3.0
      with:
        project-url: https://github.com/users/clafollett/projects/1
        github-token: ${{ secrets.GITHUB_TOKEN }}
```

## Next Steps

1. **Create GitHub Repository** (if not already done)
2. **Configure Labels** using the provided YAML
3. **Set up Milestones** with dates
4. **Create Issue Templates** in `.github/ISSUE_TEMPLATE/`
5. **Set up Project Board** with automation
6. **Configure Branch Protection** rules
7. **Add CI/CD Workflow** files
8. **Create Initial Issues** for Phase 1

This setup ensures systematic development with proper tracking and automation! 🚀