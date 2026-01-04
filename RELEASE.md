# Release Scripts

This repository contains three release scripts for building and deploying mITyGuitar desktop application releases:

## Scripts

### 1. `release.bat` (Full Release - Windows)
**Recommended for production releases**

- ✅ Comprehensive validation and safety checks
- ✅ Automatic version updating across all files
- ✅ Full build with optimization
- ✅ Git tagging and commit management
- ✅ Backup and restore functionality
- ✅ Clean working directory validation

**Usage:**
```cmd
.\release.bat
```

The script will:
1. Check git repository status
2. Prompt for version input (e.g., `1.0.0`, `0.2.1-alpha`)
3. Update versions in all relevant files
4. Build and test the application
5. Create git commit and tag
6. Optionally push to remote repository

### 2. `release.ps1` (Full Release - PowerShell)
**Cross-platform version with additional features**

- ✅ All features of release.bat
- ✅ Command-line parameters for automation
- ✅ Better error handling
- ✅ Colored output

**Usage:**
```powershell
# Interactive mode
.\release.ps1

# Automated mode
.\release.ps1 -Version "1.0.0"

# Skip tests for faster build
.\release.ps1 -Version "0.2.0-beta" -SkipTests

# Don't push to remote
.\release.ps1 -Version "0.1.5" -SkipPush

# Show help
.\release.ps1 -Help
```

**Parameters:**
- `-Version <version>`: Specify version directly
- `-SkipTests`: Skip cargo check (faster but less safe)  
- `-SkipPush`: Don't push to remote repository
- `-Help`: Show help message

### 3. `quick-release.bat` (Development Release)
**For quick development/alpha releases**

- ⚡ Faster build (debug mode)
- ⚡ Minimal validation
- ⚡ Good for development iterations

**Usage:**
```cmd
.\quick-release.bat
```

⚠️ **Warning:** This creates debug builds and should only be used for development/testing purposes.

## Version Format

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

After successful build, artifacts are located in:
- **Release builds**: `apps/desktop/src-tauri/target/release/bundle/`
- **Debug builds**: `apps/desktop/src-tauri/target/debug/bundle/`

## Prerequisites

- Git repository with clean working directory
- Node.js and npm installed
- Rust and Cargo installed
- Tauri CLI installed (`npm install -g @tauri-apps/cli`)

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