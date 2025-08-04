# MAOS Linting and Code Quality

## Overview

MAOS enforces strict code quality standards through automated linting and formatting. All code must pass formatting checks and have zero clippy warnings before merging.

## Tools

### rustfmt - Code Formatting

Automatically formats Rust code according to project standards:

```bash
# Format all code
cargo fmt

# Check formatting without changes
cargo fmt -- --check

# Format specific file
cargo fmt -- src/main.rs
```

**Configuration**: `.rustfmt.toml`
```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Max"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

### clippy - Rust Linter

Catches common mistakes and enforces best practices:

```bash
# Run clippy with warnings as errors
cargo clippy -- -D warnings

# Auto-fix clippy warnings (unstable)
cargo clippy --fix -Z unstable-options

# Run clippy on all targets
cargo clippy --all-targets --all-features -- -D warnings
```

**Key Lints Enforced**:
- No unused code
- No missing documentation for public items
- Proper error handling
- Idiomatic Rust patterns
- Performance optimizations

### cargo check - Fast Compilation Check

Quick syntax and type checking without full compilation:

```bash
# Check current package
cargo check

# Check all workspace members
cargo check --workspace

# Check with all features
cargo check --all-features
```

## CI Enforcement

All pull requests must pass these checks:

### Required Status Checks

1. **Format Check**
   ```yaml
   - name: Check formatting
     run: cargo fmt -- --check
   ```

2. **Clippy Lints**
   ```yaml
   - name: Run clippy
     run: cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Compilation Check**
   ```yaml
   - name: Check compilation
     run: cargo check --all-features
   ```

See [Development Workflow](../DEVELOPMENT_WORKFLOW.md#branch-protection-rules) for branch protection settings.

## Local Development

### Pre-commit Hooks

Run all checks before committing:

```bash
# Manual pre-commit
just pre-commit

# Includes:
# - cargo fmt
# - cargo clippy -- -D warnings
# - cargo test
# - cargo doc --no-deps
```

### IDE Integration

#### VS Code

`.vscode/settings.json`:
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

#### IntelliJ/CLion

1. Settings → Languages & Frameworks → Rust
2. Enable "Run rustfmt on Save"
3. Set Clippy as external linter

## Common Issues and Fixes

### Unused Code Warnings

```rust
// Allow for work in progress
#[allow(dead_code)]
fn future_feature() { }

// Or remove if truly unused
```

### Missing Documentation

```rust
// Bad
pub fn process_data(input: &str) -> Result<String> { }

// Good
/// Processes input data and returns formatted result.
/// 
/// # Errors
/// Returns error if input is invalid.
pub fn process_data(input: &str) -> Result<String> { }
```

### Clippy Auto-fixes

Many clippy warnings can be auto-fixed:

```bash
# Auto-fix safe warnings
cargo clippy --fix

# Auto-fix with unstable features
cargo clippy --fix -Z unstable-options
```

### Format Conflicts

If rustfmt and your code disagree:

```rust
// Disable formatting for specific section
#[rustfmt::skip]
let matrix = [
    1, 0, 0,
    0, 1, 0,
    0, 0, 1,
];
```

## Performance Linting

### Clippy Performance Lints

Enable additional performance checks:

```toml
# clippy.toml
avoid-breaking-exported-api = false
cognitive-complexity-threshold = 30

# In code
#![warn(clippy::perf)]
#![warn(clippy::pedantic)]
```

### Common Performance Issues

1. **Unnecessary Allocations**
   ```rust
   // Bad
   fn get_name() -> String {
       "MAOS".to_string()
   }
   
   // Good
   fn get_name() -> &'static str {
       "MAOS"
   }
   ```

2. **Inefficient Iterations**
   ```rust
   // Bad
   for i in 0..vec.len() {
       println!("{}", vec[i]);
   }
   
   // Good
   for item in &vec {
       println!("{}", item);
   }
   ```

## Best Practices

1. **Run linting frequently** - Don't wait for CI
2. **Fix warnings immediately** - Don't let them accumulate
3. **Understand the warnings** - Learn from clippy's suggestions
4. **Configure appropriately** - Adjust lints for your project needs
5. **Document exceptions** - Use `#[allow()]` with explanations

## Custom Lints

Add project-specific lints in `src/lib.rs` or `src/main.rs`:

```rust
#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
```

## Related Documentation

- [Contributing Guide](../../CONTRIBUTING.md) - Development standards
- [Development Workflow](../DEVELOPMENT_WORKFLOW.md) - CI/CD requirements
- [Testing Guide](../development/testing.md) - Test standards