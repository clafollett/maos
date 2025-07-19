# MAOS Development Task Runner
# Install just: https://github.com/casey/just#installation
# Usage: just <recipe>

# Load environment variables from stack.env
set dotenv-load := true
set dotenv-filename := "stack.env"

# Default recipe (runs when you just type 'just')
default:
    @just --list

# Check if environment is properly configured
check-env:
    @echo "🔧 Checking environment configuration..."
    @test -f stack.env || (echo "❌ stack.env file not found. Ensure it exists and is properly sourced." && exit 1)
    @test -n "${RUST_TOOLCHAIN:-}" || (echo "❌ RUST_TOOLCHAIN not set. Run: source stack.env" && exit 1)
    @test -n "${BUILD_FLAGS:-}" || (echo "❌ BUILD_FLAGS not set. Run: source stack.env" && exit 1)
    @echo "✅ Environment properly configured"

# Development setup and validation
dev-setup:
    @echo "🚀 Setting up MAOS development environment..."
    @just check-env
    @just validate-stack
    @just install-deps
    @just setup-git-hooks
    @just format
    @just lint
    @just test
    @echo "✅ Development environment ready!"

# Validate stack versions match stack.env
validate-stack:
    @echo "🔍 Validating development stack..."
    @echo "Required files check:"
    @test -f rust-toolchain.toml || (echo "❌ rust-toolchain.toml missing" && exit 1)
    @test -f clippy.toml || (echo "❌ clippy.toml missing" && exit 1)
    @test -f rustfmt.toml || (echo "❌ rustfmt.toml missing" && exit 1)
    @echo "✅ All required files present"
    @echo "Toolchain versions:"
    @rustc --version
    @cargo --version
    @just --version
    @echo "Environment variables:"
    @echo "RUST_TOOLCHAIN: ${RUST_TOOLCHAIN}"
    @echo "BUILD_FLAGS: ${BUILD_FLAGS}"
    @echo "📋 Stack validation complete"

# Install development dependencies
install-deps:
    @echo "📦 Installing Rust components..."
    rustup component add rustfmt clippy rust-src
    @echo "📦 Installing cargo tools..."
    cargo install cargo-audit --quiet || echo "⚠️  cargo-audit already installed"
    @echo "✅ Dependencies installed"

# Code formatting
format:
    @echo "🎨 Formatting code..."
    cargo fmt --all

# Check formatting without applying changes
format-check:
    @echo "🔍 Checking code formatting..."
    cargo fmt --all -- --check

# Linting with clippy
lint:
    @echo "🔍 Running clippy lints..."
    cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
test:
    @echo "🧪 Running tests..."
    cargo test --all-features

# Run tests with coverage (requires cargo-tarpaulin)
test-coverage:
    @echo "📊 Running tests with coverage..."
    cargo tarpaulin --all-features --out Html

# Security audit
audit:
    @echo "🔒 Running security audit..."
    cargo audit

# Build debug version
build:
    @echo "🔨 Building debug version..."
    cargo build --all-targets

# Build release version
build-release:
    @echo "🚀 Building release version..."
    cargo build --release --all-targets

# Check compilation without building
check:
    @echo "✅ Checking compilation..."
    cargo check --all-targets

# Pre-commit checks (all quality gates)
pre-commit: check-env format-check lint test audit
    @echo "✅ All pre-commit checks passed!"

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean

# Update dependencies
update:
    @echo "📦 Updating dependencies..."
    cargo update

# Run the MAOS CLI
run *args:
    @echo "🤖 Running MAOS..."
    cargo run -- {{args}}

# Development watch mode (requires cargo-watch)
watch:
    @echo "👀 Watching for changes..."
    cargo watch -x check -x test

# Generate documentation
docs:
    @echo "📚 Generating documentation..."
    cargo doc --all-features --open

# Full CI pipeline locally
ci: format-check lint test audit build
    @echo "🎉 Full CI pipeline completed successfully!"

# Set up git hooks (pure Rust alternative to pre-commit)
setup-git-hooks:
    #!/usr/bin/env bash
    echo "🪝 Setting up git hooks..."
    mkdir -p .git/hooks
    cat > .git/hooks/pre-commit << 'HOOK_EOF'
    #!/bin/sh
    # MAOS Pre-commit Hook - Validates environment and runs quality checks
    
    set -e  # Exit on any error
    
    echo "🪝 MAOS Pre-commit validation starting..."
    
    # Validate development environment
    echo "📋 Sourcing stack.env..."
    source stack.env || {
        echo "❌ Failed to source stack.env"
        echo "💡 Make sure you're in the project root directory"
        exit 1
    }
    
    # Validate stack configuration
    echo "🔍 Validating development stack..."
    just validate-stack || {
        echo "❌ Stack validation failed"
        echo "💡 Run 'just dev-setup' to fix your environment"
        exit 1
    }
    
    # Run all quality checks
    echo "✅ Running pre-commit quality checks..."
    just pre-commit || {
        echo "❌ Pre-commit checks failed"
        echo "💡 Fix the issues above and try committing again"
        exit 1
    }
    
    echo "🎉 All pre-commit checks passed!"
    HOOK_EOF
    chmod +x .git/hooks/pre-commit
    echo "✅ Git hooks installed! All commits will validate environment and run quality checks"