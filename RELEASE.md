# Release Scripts

This repository contains multiple release scripts for building and deploying mITyGuitar desktop application releases across different platforms:

## Scripts

### 1. `release.bat` (Full Release - Windows)
**Recommended for production releases on Windows**

- ✅ Comprehensive validation and safety checks
- ✅ Automatic version updating across all files
- ✅ Full build with optimization
- ✅ Git tagging and commit management
- ✅ Backup and restore functionality
- ✅ Clean working directory validation
- ✅ Dynamic version display in Help > About

**Usage:**
```cmd
.\release.bat
```

### 2. `release.ps1` (Full Release - PowerShell)
**Cross-platform version with platform-specific builds**

- ✅ All features of release.bat
- ✅ Command-line parameters for automation
- ✅ Better error handling and colored output
- ✅ Platform-specific build support
- ✅ Cross-platform installer generation

**Usage:**
```powershell
# Interactive mode (all platforms)
.\release.ps1

# Automated mode
.\release.ps1 -Version "1.0.0"

# Platform-specific builds
.\release.ps1 -Version "1.0.0" -Platform "windows"
.\release.ps1 -Version "1.0.0" -Platform "macos"
.\release.ps1 -Version "1.0.0" -Platform "linux"

# Skip tests for faster build
.\release.ps1 -Version "0.2.0-beta" -SkipTests

# Don't push to remote
.\release.ps1 -Version "0.1.5" -SkipPush

# Show help
.\release.ps1 -Help
```

**Parameters:**
- `-Version <version>`: Specify version directly
- `-Platform <platform>`: Target platform (windows, macos, linux, all). Default: all
- `-SkipTests`: Skip cargo check (faster but less safe)  
- `-SkipPush`: Don't push to remote repository
- `-Help`: Show help message

### 3. `ci-release.ps1` (CI/CD Release)
**For automated builds in GitHub Actions or other CI systems**

- ⚡ Designed for CI environments
- ⚡ Platform-specific cross-compilation support
- ⚡ No interactive prompts

**Usage:**
```powershell
# CI builds
.\ci-release.ps1 -Version "1.0.0" -Platform "windows"
.\ci-release.ps1 -Version "1.0.0" -Platform "macos" 
.\ci-release.ps1 -Version "1.0.0" -Platform "linux"
```

### 4. `quick-release.bat` (Development Release)
**For quick development/alpha releases**

- ⚡ Faster build (debug mode)
- ⚡ Minimal validation
- ⚡ Good for development iterations

⚠️ **Warning:** This creates debug builds and should only be used for development/testing purposes.

## Platform Support

### Supported Installers

**Windows:**
- 📦 **MSI Installer**: Standard Windows installer package
- 📦 **NSIS Setup**: Executable installer with custom branding

**macOS:**
- 📦 **DMG**: Disk image for easy drag-and-drop installation
- 📦 **APP Bundle**: Native macOS application bundle

**Linux:**
- 📦 **DEB Package**: Debian/Ubuntu package manager compatible
- 📦 **AppImage**: Portable application format

### Cross-Platform Building

- **Native builds** are recommended for best compatibility
- **Cross-compilation** is supported but may have limitations
- **CI/CD environments** can build for multiple platforms using `ci-release.ps1`

## Version Management

### Dynamic Version Display
The application now dynamically displays the current version in **Help > About** using Tauri's API. No more hardcoded version strings!

### Files Updated
The scripts automatically update versions in:
- `Cargo.toml` (workspace)
- `apps/desktop/package.json`
- `apps/desktop/src-tauri/Cargo.toml`
- `apps/desktop/src-tauri/tauri.conf.json`

### Version Format

All scripts support semantic versioning:

- **Production releases**: `1.0.0`, `2.1.3`
- **Pre-releases**: `1.0.0-alpha`, `0.5.0-beta`, `2.0.0-rc.1`
- **Development**: `0.1.0-dev`, `1.0.0-nightly`

## Files Updated

The scripts automatically update versions in:
- `Cargo.toml` (workspace)
- `apps/desktop/package.json`
- `apps/desktop/src-tauri/Cargo.toml`
- `apps/desktop/src-tauri/tauri.conf.json`

## Build Artifacts

After successful build, platform-specific artifacts are located in:
- **All platforms**: `apps/desktop/src-tauri/target/release/bundle/`

### Platform-specific locations:
- **Windows**: `bundle/msi/` and `bundle/nsis/`
- **macOS**: `bundle/dmg/` and `bundle/macos/` 
- **Linux**: `bundle/deb/` and `bundle/appimage/`

## Prerequisites

- Git repository with clean working directory
- Node.js and npm installed
- Rust and Cargo installed  
- Tauri CLI installed (`npm install -g @tauri-apps/cli`)

### Platform-specific prerequisites:

**Windows:**
- Windows SDK (for MSI builds)
- NSIS (for NSIS builds) - usually installed automatically

**macOS:**
- Xcode Command Line Tools
- macOS deployment target compatible with Tauri

**Linux:**
- Build essentials (`build-essential` on Ubuntu/Debian)
- Additional system dependencies may be required

## Troubleshooting

### Common Issues:

1. **"Working directory is not clean"**
   - Commit or stash your changes before running the release script

2. **"Cargo check failed"**
   - Fix any compilation errors in your Rust code
   - Run `cargo check --workspace` manually to see detailed errors

3. **"Frontend build failed"**
   - Check your TypeScript/React code for errors
   - Run `npm run build` in `apps/desktop/` manually

4. **"Tauri build failed"**
   - Ensure all dependencies are installed
   - Check Tauri configuration files

### Recovery:

If a release fails, the scripts automatically restore backup files. You can also manually restore:

```cmd
copy Cargo.toml.backup Cargo.toml
copy apps\desktop\package.json.backup apps\desktop\package.json
copy apps\desktop\src-tauri\Cargo.toml.backup apps\desktop\src-tauri\Cargo.toml
copy apps\desktop\src-tauri\tauri.conf.json.backup apps\desktop\src-tauri\tauri.conf.json
```

## GitHub Releases

After creating a release tag, you can create a GitHub release at:
`https://github.com/janvanwassenhove/mITyGuitar/releases/new?tag=v<version>`

Upload the build artifacts from the bundle directory to the GitHub release for distribution.