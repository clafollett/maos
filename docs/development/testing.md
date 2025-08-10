# MAOS Testing Guide

## Overview

MAOS follows Test-Driven Development (TDD) principles with comprehensive test coverage across unit, integration, and performance tests. All code must have associated tests before merging.

## Test Organization

```
maos/
├── crates/
│   ├── maos-cli/
│   │   ├── src/
│   │   │   └── lib.rs         # Unit tests in #[cfg(test)] modules
│   │   └── tests/             # Integration tests
│   │       └── cli_test.rs
│   ├── maos-security/
│   │   ├── src/
│   │   │   └── validators.rs  # Unit tests alongside code
│   │   └── tests/
│   │       └── security_integration.rs
│   └── maos-testing/          # Shared test utilities
│       └── src/
│           ├── fixtures.rs    # Test data generators
│           └── mocks.rs       # Mock implementations
└── tests/                     # Workspace-level integration tests
    └── end_to_end.rs
```

## Running Tests

### All Tests

```bash
# Run all tests across workspace
just test

# With output for passing tests
just test -- --nocapture

# Run specific test
cargo test test_security_validation

# Run tests for specific crate
just test-crate maos-security
```

### Test Categories

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Doc tests only
cargo test --doc

# Specific test file
cargo test --test cli_test
```

### Test Coverage

```bash
# Generate coverage report
just coverage

# With HTML output
just coverage-html
# Open coverage/index.html

# Coverage for specific crate
cd crates/maos-security
cargo tarpaulin --out Html
```

## Writing Tests

### Unit Tests

Place unit tests in the same file as the code:

```rust
// src/security/validators.rs
pub fn validate_command(cmd: &str) -> Result<(), SecurityError> {
    if cmd.contains("rm -rf /") {
        return Err(SecurityError::DangerousCommand);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_dangerous_rm() {
        let result = validate_command("rm -rf /");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SecurityError::DangerousCommand
        ));
    }

    #[test]
    fn test_allows_safe_commands() {
        assert!(validate_command("ls -la").is_ok());
        assert!(validate_command("git status").is_ok());
    }
}
```

### Integration Tests

Create separate test files in `tests/` directory:

```rust
// tests/cli_integration.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_pre_tool_use_blocks_dangerous_command() {
    let mut cmd = Command::cargo_bin("maos").unwrap();
    
    cmd.arg("pre-tool-use")
       .env("TOOL_NAME", "Bash")
       .env("COMMAND", "rm -rf /")
       .assert()
       .failure()
       .code(2)
       .stderr(predicate::str::contains("Security violation"));
}

#[test]
fn test_worktree_creation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // Initialize git repo
    init_test_repo(repo_path);
    
    let mut cmd = Command::cargo_bin("maos").unwrap();
    cmd.arg("pre-tool-use")
       .env("TOOL_NAME", "Task")
       .env("AGENT_TYPE", "backend-engineer")
       .current_dir(repo_path)
       .assert()
       .success();
       
    // Verify worktree was created
    assert!(repo_path.join("worktrees").exists());
}
```

### Test Utilities

Use the `maos-testing` crate for shared utilities:

```rust
// crates/maos-testing/src/fixtures.rs
use tempfile::TempDir;
use std::process::Command;

pub fn create_test_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    
    Command::new("git")
        .args(&["init"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to init git repo");
        
    // Add initial commit
    std::fs::write(dir.path().join("README.md"), "Test repo").unwrap();
    
    Command::new("git")
        .args(&["add", "."])
        .current_dir(dir.path())
        .output()
        .unwrap();
        
    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(dir.path())
        .output()
        .unwrap();
        
    dir
}

// Usage in tests
#[test]
fn test_with_repo() {
    let repo = create_test_repo();
    // Test code here
    // TempDir automatically cleans up
}
```

## Test Patterns

### 1. Table-Driven Tests

```rust
#[test]
fn test_dangerous_commands() {
    let dangerous_commands = vec![
        ("rm -rf /", true),
        ("rm -rf /*", true),
        ("sudo rm -rf /", true),
        ("rm -rf ~", true),
        ("rm file.txt", false),
        ("rm -f specific.txt", false),
    ];
    
    for (cmd, should_block) in dangerous_commands {
        let result = validate_command(cmd);
        assert_eq!(
            result.is_err(),
            should_block,
            "Command '{}' blocking mismatch",
            cmd
        );
    }
}
```

### 2. Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_path_validation_never_panics(path in any::<String>()) {
        // Should handle any input without panicking
        let _ = validate_path(&path);
    }
    
    #[test]
    fn test_session_id_format(
        agent_type in "[a-z-]+",
        timestamp in 1000000000u64..2000000000u64
    ) {
        let session_id = format!("{}-{}", agent_type, timestamp);
        assert!(is_valid_session_id(&session_id));
    }
}
```

### 3. Async Tests

```rust
#[tokio::test]
async fn test_async_tts_provider() {
    let provider = ElevenLabsProvider::new("test-key");
    
    let result = provider.speak("Hello world").await;
    
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_file_locks() {
    let lock_manager = FileLockManager::new();
    
    let handles = (0..10).map(|i| {
        let manager = lock_manager.clone();
        tokio::spawn(async move {
            manager.acquire_lock(&format!("file{}.txt", i)).await
        })
    });
    
    let results = futures::future::join_all(handles).await;
    assert!(results.iter().all(|r| r.is_ok()));
}
```

## Performance Tests

### Benchmarks

```rust
// benches/startup_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_command_validation(c: &mut Criterion) {
    c.bench_function("validate safe command", |b| {
        b.iter(|| {
            validate_command(black_box("git status"))
        })
    });
    
    c.bench_function("validate dangerous command", |b| {
        b.iter(|| {
            validate_command(black_box("rm -rf /"))
        })
    });
}

criterion_group!(benches, benchmark_command_validation);
criterion_main!(benches);
```

Run benchmarks:
```bash
just bench
# Or specific benchmark
cargo bench --bench startup_bench
```

### Performance Regression Tests

```rust
#[test]
fn test_hook_performance_target() {
    let start = std::time::Instant::now();
    
    // Simulate hook execution
    validate_command("git status").unwrap();
    create_session("test-session").unwrap();
    
    let duration = start.elapsed();
    
    assert!(
        duration.as_millis() < 10,
        "Hook execution took {}ms, target is <10ms",
        duration.as_millis()
    );
}
```

## Mocking

### Mock Implementations

```rust
// crates/maos-testing/src/mocks.rs
use mockall::automock;

#[automock]
pub trait TtsProvider {
    fn speak(&self, text: &str) -> Result<(), TtsError>;
    fn is_available(&self) -> bool;
}

// Usage in tests
#[test]
fn test_tts_fallback() {
    let mut mock = MockTtsProvider::new();
    
    mock.expect_is_available()
        .times(1)
        .returning(|| false);
        
    let manager = TtsManager::new(vec![Box::new(mock)]);
    
    // Test fallback behavior
    assert!(manager.speak("test").is_err());
}
```

### Filesystem Mocking

```rust
use std::fs;
use tempfile::TempDir;

fn setup_test_env() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join(".claude/hooks/maos/config.json");
    
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::write(&config_path, r#"{"maos": {"tts": {"enabled": true}}}"#).unwrap();
    
    (temp, config_path)
}

#[test]
fn test_config_loading() {
    let (_temp, config_path) = setup_test_env();
    
    let config = load_config(&config_path).unwrap();
    assert!(config.maos.tts.enabled);
}
```

## Test Data

### Fixtures

```rust
// crates/maos-testing/src/fixtures.rs
pub mod sessions {
    pub const VALID_SESSION: &str = r#"{
        "session_id": "sess-123",
        "started_at": "2024-01-20T10:30:00Z",
        "agents": []
    }"#;
    
    pub fn create_test_session() -> Session {
        serde_json::from_str(VALID_SESSION).unwrap()
    }
}

// Usage
#[test]
fn test_session_operations() {
    let session = fixtures::sessions::create_test_session();
    assert_eq!(session.id, "sess-123");
}
```

### Snapshot Testing

```rust
use insta::assert_snapshot;

#[test]
fn test_error_message_format() {
    let error = SecurityError::DangerousCommand("rm -rf /".into());
    
    assert_snapshot!(error.to_string(), @r###"
    Security violation: Blocked dangerous command: rm -rf /
    
    This command could delete critical system files.
    Use a more specific path or remove the -f flag.
    "###);
}
```

## CI/CD Testing

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    
    steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    
    - name: Run tests
      run: |
        cargo test --all-features
        cargo test --no-default-features
    
    - name: Check coverage
      if: matrix.os == 'ubuntu-latest'
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out Xml
        bash <(curl -s https://codecov.io/bash)
```

## Best Practices

### Test Naming

```rust
// Good test names
#[test]
fn test_validate_command_blocks_rm_rf_root() { }

#[test]
fn test_worktree_creation_with_custom_branch_name() { }

// Bad test names
#[test]
fn test1() { }

#[test]
fn test_validation() { }
```

### Test Independence

```rust
// Each test should be independent
#[test]
fn test_independent_1() {
    let temp = TempDir::new().unwrap();
    // Test logic
    // Cleanup happens automatically
}

#[test]
fn test_independent_2() {
    let temp = TempDir::new().unwrap();
    // Different test, different temp dir
}
```

### Clear Assertions

```rust
// Good - clear error messages
assert_eq!(
    result.agents.len(),
    2,
    "Expected 2 agents, but found {}",
    result.agents.len()
);

// Better - use custom matchers
assert!(
    result.contains_agent("backend-engineer"),
    "Backend engineer agent not found in session"
);
```

## Debugging Tests

### Running Single Test

```bash
# Run with output
cargo test test_name -- --nocapture

# With backtrace
RUST_BACKTRACE=1 cargo test test_name

# With logging
RUST_LOG=debug cargo test test_name
```

### Test Timeout

```rust
#[test]
#[timeout(1000)] // 1 second timeout
fn test_should_complete_quickly() {
    // Test that might hang
}
```

### Conditional Tests

```rust
#[test]
#[cfg(target_os = "macos")]
fn test_macos_say_command() {
    // macOS specific test
}

#[test]
#[ignore] // Ignored by default
fn test_requires_real_api_key() {
    // Run with: cargo test -- --ignored
}
```

## Related Documentation

- [Development Setup](./setup.md) - Environment configuration
- [Contributing](../../CONTRIBUTING.md) - Development workflow
- [Architecture](../architecture/rust-cli-architecture.md) - System design