# MAOS Release Process

## Overview

MAOS releases are distributed as compiled binaries through multiple channels: NPM (via npx), Homebrew, and direct downloads. This guide covers the build, test, and release process.

## Release Channels

### 1. NPM Package (@maos/cli)
- Primary distribution method
- Enables `npx @maos/cli` usage
- Auto-detects platform and downloads appropriate binary

### 2. Homebrew
- macOS and Linux users
- Simple `brew install maos`
- Automatic updates via `brew upgrade`

### 3. Direct Downloads
- GitHub Releases
- Pre-compiled binaries for all platforms
- Installation scripts for automation

## Version Management

MAOS follows Semantic Versioning (SemVer):

```
MAJOR.MINOR.PATCH

1.0.0 - First stable release
1.1.0 - New features (backward compatible)
1.1.1 - Bug fixes
2.0.0 - Breaking changes
```

### Version Locations

Update version in:
1. `Cargo.toml` (workspace root)
2. All crate `Cargo.toml` files
3. `package.json` (NPM distribution)
4. `CHANGELOG.md`

## Build Process

### Local Release Build

```bash
# Build optimized release binary
just build-release

# Output: target/release/maos
```

### Cross-Platform Builds

```bash
# Install cross-compilation tool
cargo install cross

# Build for all targets
just build-all-targets

# This builds:
# - Linux x86_64 (gnu and musl)
# - Linux ARM64
# - macOS x86_64
# - macOS ARM64 (Apple Silicon)
# - Windows x86_64
```

### Build Matrix

```yaml
targets:
  - x86_64-unknown-linux-gnu
  - x86_64-unknown-linux-musl
  - aarch64-unknown-linux-gnu
  - x86_64-apple-darwin
  - aarch64-apple-darwin
  - x86_64-pc-windows-msvc
```

## Release Checklist

### 1. Pre-Release

```bash
# Update version numbers
just bump-version 1.2.0

# Update CHANGELOG.md
# Document all changes since last release

# Run full test suite
just test-all

# Check for security vulnerabilities
just audit

# Verify binary size
just check-binary-size

# Test on all platforms
just test-cross-platform
```

### 2. Create Release

```bash
# Create and push tag
git tag -a v1.2.0 -m "Release v1.2.0"
git push origin v1.2.0

# GitHub Actions will:
# 1. Build binaries for all platforms
# 2. Run tests on each platform
# 3. Create draft release
# 4. Upload artifacts
```

### 3. Release Verification

Test each distribution channel:

```bash
# Test NPM package
npm pack
npx ./maos-cli-1.2.0.tgz --version

# Test direct download
curl -sSL https://github.com/clafollett/maos/releases/download/v1.2.0/maos-linux-x64 -o maos
chmod +x maos
./maos --version

# Test installation script
curl -sSL https://raw.githubusercontent.com/clafollett/maos/main/scripts/install.sh | sh
```

## NPM Distribution

### Package Structure

```
npm-package/
├── package.json
├── README.md
├── LICENSE
├── install.js        # Platform detection and download
└── bin/
    └── maos         # Wrapper script
```

### package.json

```json
{
  "name": "@maos/cli",
  "version": "1.2.0",
  "description": "Multi-Agent Orchestration System CLI",
  "bin": {
    "maos": "./bin/maos"
  },
  "scripts": {
    "postinstall": "node install.js"
  },
  "os": ["darwin", "linux", "win32"],
  "cpu": ["x64", "arm64"],
  "engines": {
    "node": ">=14.0.0"
  }
}
```

### install.js

```javascript
const { platform, arch } = process;
const { createWriteStream, chmodSync } = require('fs');
const { pipeline } = require('stream/promises');
const fetch = require('node-fetch');

async function install() {
  const binaryName = getBinaryName(platform, arch);
  const downloadUrl = `https://github.com/clafollett/maos/releases/download/v${version}/${binaryName}`;
  
  const response = await fetch(downloadUrl);
  const fileStream = createWriteStream('./bin/maos');
  
  await pipeline(response.body, fileStream);
  chmodSync('./bin/maos', 0o755);
}

function getBinaryName(platform, arch) {
  const mapping = {
    'darwin-x64': 'maos-macos-x64',
    'darwin-arm64': 'maos-macos-arm64',
    'linux-x64': 'maos-linux-x64',
    'linux-arm64': 'maos-linux-arm64',
    'win32-x64': 'maos-windows-x64.exe'
  };
  
  return mapping[`${platform}-${arch}`];
}
```

### Publishing to NPM

```bash
# Login to NPM
npm login

# Publish package
cd npm-package
npm publish --access public

# Verify publication
npm view @maos/cli
```

## Homebrew Formula

### Formula Template

```ruby
class Maos < Formula
  desc "Multi-Agent Orchestration System for Claude Code"
  homepage "https://github.com/clafollett/maos"
  version "1.2.0"
  
  if OS.mac? && Hardware::CPU.intel?
    url "https://github.com/clafollett/maos/releases/download/v1.2.0/maos-macos-x64.tar.gz"
    sha256 "SHA256_HASH_HERE"
  elsif OS.mac? && Hardware::CPU.arm?
    url "https://github.com/clafollett/maos/releases/download/v1.2.0/maos-macos-arm64.tar.gz"
    sha256 "SHA256_HASH_HERE"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/clafollett/maos/releases/download/v1.2.0/maos-linux-x64.tar.gz"
    sha256 "SHA256_HASH_HERE"
  end

  def install
    bin.install "maos"
  end

  test do
    assert_match "maos #{version}", shell_output("#{bin}/maos --version")
  end
end
```

### Homebrew Tap

```bash
# Create tap repository
# github.com/clafollett/homebrew-maos

# Add formula
mkdir Formula
cp maos.rb Formula/

# Test locally
brew install --build-from-source Formula/maos.rb

# Push to tap
git add Formula/maos.rb
git commit -m "Update MAOS to v1.2.0"
git push
```

## GitHub Actions Release

### Release Workflow

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: maos-linux-x64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            artifact: maos-linux-arm64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: maos-macos-x64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: maos-macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: maos-windows-x64.exe
    
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        targets: ${{ matrix.target }}
    
    - name: Build Release
      run: cargo build --release --target ${{ matrix.target }}
    
    - name: Compress Binary
      run: |
        cd target/${{ matrix.target }}/release
        tar -czf ${{ matrix.artifact }}.tar.gz maos*
    
    - name: Upload Artifact
      uses: actions/upload-artifact@v3
      with:
        name: ${{ matrix.artifact }}
        path: target/${{ matrix.target }}/release/${{ matrix.artifact }}.tar.gz
  
  release:
    needs: build
    runs-on: ubuntu-latest
    
    steps:
    - name: Download Artifacts
      uses: actions/download-artifact@v3
    
    - name: Create Release
      uses: softprops/action-gh-release@v1
      with:
        draft: true
        files: |
          **/*.tar.gz
          **/*.exe
        body: |
          ## Changes
          
          See [CHANGELOG.md](https://github.com/clafollett/maos/blob/main/CHANGELOG.md)
          
          ## Installation
          
          ### NPX
          ```bash
          npx @maos/cli setup
          ```
          
          ### Homebrew
          ```bash
          brew install maos
          ```
          
          ### Direct Download
          Download the appropriate binary for your platform below.
```

## Binary Optimization

### Size Optimization

```toml
# Cargo.toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
strip = true        # Strip symbols
panic = "abort"     # Smaller panic handler
```

### Compression

```bash
# Use UPX for additional compression
upx --best --lzma target/release/maos

# Verify functionality after compression
./target/release/maos --version
```

## Testing Releases

### Automated Release Tests

```bash
# Test installation methods
just test-release-npm
just test-release-homebrew
just test-release-direct

# Test binary functionality
just test-release-binary

# Performance benchmarks
just bench-release
```

### Manual Testing Checklist

- [ ] Binary runs without dependencies
- [ ] All commands work correctly
- [ ] Performance meets <10ms target
- [ ] File size is reasonable
- [ ] Works on clean system
- [ ] Upgrade path works

## Post-Release

### 1. Update Documentation

- Update README.md with new version
- Update installation guides
- Update migration guide if needed

### 2. Announcements

- GitHub Release notes
- Discord announcement
- Twitter/Social media

### 3. Monitor

- Check NPM download stats
- Monitor GitHub issues
- Track Homebrew analytics

## Rollback Process

If issues are discovered:

```bash
# Yank NPM package
npm unpublish @maos/cli@1.2.0

# Update Homebrew formula to previous version
# Revert Formula/maos.rb

# Mark GitHub release as pre-release
# Or delete if critical
```

## Security Releases

For security fixes:

1. Follow responsible disclosure
2. Prepare fixes in private
3. Release simultaneously across all channels
4. Include security advisory

## Related Documentation

- [Development Setup](./setup.md) - Build environment
- [Testing Guide](./testing.md) - Release testing
- [Architecture](../architecture/rust-cli-architecture.md) - Binary structure