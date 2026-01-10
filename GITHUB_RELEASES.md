# Release Guide

## Overview

Simple, one-script solution for building and releasing the mITyGuitar desktop application with automatic GitHub releases.

## Quick Start

### Standard Release (Recommended)
```powershell
# Complete automated release with GitHub release
.\release.ps1 -Version "1.0.0" -CreateGitHubRelease

# Windows only with GitHub release  
.\release.ps1 -Version "1.0.1" -Platform "windows" -CreateGitHubRelease
```

### Development Release
```powershell
# Build without pushing to GitHub
.\release.ps1 -Version "0.2.0-dev" -SkipPush

# Quick build (skip tests)
.\release.ps1 -Version "0.2.0-dev" -SkipTests -SkipPush
```

## The One Script: `release.ps1`

**Purpose**: Complete build and release automation - the only script you need!

**Parameters**:
- `-Version`: Version to build (e.g., "1.0.0", "0.2.0-alpha")
- `-Platform`: Target platform ("windows", "macos", "linux", "all") - Default: "all"
- `-CreateGitHubRelease`: Automatically create GitHub release with installers
- `-SkipTests`: Skip cargo checks (faster builds)
- `-SkipPush`: Don't push to remote repository

## What It Does

1. ✅ **Updates all version files** (Cargo.toml, package.json, tauri.conf.json)
2. ✅ **Builds the application** with frontend and Tauri backend
3. ✅ **Creates Windows installers** (MSI + NSIS)
4. ✅ **Creates git tag** with version
5. ✅ **Pushes to GitHub** (optional)
6. ✅ **Creates GitHub release** with downloadable installers (optional)

## Prerequisites

### Required Software
1. **Tauri development environment** (Node.js, Rust)
2. **GitHub CLI** (optional, for automatic GitHub releases):
   ```powershell
   winget install --id GitHub.cli
   gh auth login
   ```

### Setup Requirements
- Clean git working directory
- Valid semantic version format (e.g., 1.0.0, 0.2.0-alpha)

## Usage Examples

### Full Production Release
```powershell
# Build, tag, push, and create GitHub release
.\release.ps1 -Version "1.0.0" -CreateGitHubRelease
```

### Development Build
```powershell
# Local build only (no git operations)
.\release.ps1 -Version "1.1.0-dev" -SkipPush
```

### Quick Test Build
```powershell
# Fast build for testing (skip validation)
.\release.ps1 -Version "test-1.0.0" -SkipTests -SkipPush
```

## Troubleshooting

### GitHub CLI Issues
```powershell
# Install GitHub CLI
winget install --id GitHub.cli

# Authenticate
gh auth login

# Check status
gh auth status
```

### Build Failures
- Ensure git working directory is clean: `git status`
- Check Tauri environment: `npm run tauri --version`
- Verify dependencies: `cargo check --workspace`

## Release Assets

Each release automatically includes:
- **MSI Package**: `mITyGuitar_{version}_x64_en-US.msi`
- **NSIS Installer**: `mITyGuitar_{version}_x64-setup.exe`
- **Professional release notes** with installation instructions

## Best Practices

1. **Use semantic versioning**: 1.0.0, 1.1.0, 2.0.0-alpha
2. **Test locally first**: Use `-SkipPush` for testing
3. **Create GitHub releases**: Use `-CreateGitHubRelease` for distribution
4. **Keep git clean**: Commit changes before releasing

## GitHub URLs

- **Latest**: https://github.com/janvanwassenhove/mITyGuitar/releases/latest
- **All Releases**: https://github.com/janvanwassenhove/mITyGuitar/releases