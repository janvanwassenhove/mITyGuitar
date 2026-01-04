# mITyGuitar Release Builder (PowerShell version)
param(
    [Parameter(Mandatory=$false)]
    [string]$Version,
    
    [Parameter(Mandatory=$false)]
    [switch]$SkipTests,
    
    [Parameter(Mandatory=$false)]
    [switch]$SkipPush,
    
    [Parameter(Mandatory=$false)]
    [ValidateSet("windows", "macos", "linux", "all")]
    [string]$Platform = "all",
    
    [Parameter(Mandatory=$false)]
    [switch]$Help
)

if ($Help) {
    Write-Host @"
mITyGuitar Release Builder

Usage:
    .\release.ps1 [-Version <version>] [-Platform <platform>] [-SkipTests] [-SkipPush] [-Help]

Parameters:
    -Version <version>  : Specify the version (e.g., 1.0.0, 0.2.1-alpha)
    -Platform <platform>: Target platform (windows, macos, linux, all). Default: all
    -SkipTests         : Skip cargo check (faster but less safe)
    -SkipPush          : Don't push to remote repository
    -Help              : Show this help message

Examples:
    .\release.ps1 -Version "1.0.0"
    .\release.ps1 -Version "0.2.0-beta" -Platform "windows"
    .\release.ps1 -Version "1.0.0" -Platform "all"
"@
    exit 0
}

Write-Host ""
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host "         mITyGuitar Release Builder" -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host ""

# Check if we're in a git repository
try {
    git rev-parse --git-dir 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw }
} catch {
    Write-Host "ERROR: Not in a git repository!" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

# Check if working directory is clean
$status = git status --porcelain 2>$null
if ($status) {
    Write-Host "ERROR: Working directory is not clean! Please commit or stash changes first." -ForegroundColor Red
    Write-Host ""
    git status --short
    Read-Host "Press Enter to exit"
    exit 1
}

# Get current version
$currentVersion = (Get-Content "Cargo.toml" | Select-String 'version = "(.+)"').Matches.Groups[1].Value

Write-Host "Current version: $currentVersion" -ForegroundColor Yellow
Write-Host ""

# Get new version
if (-not $Version) {
    do {
        $Version = Read-Host "Enter new version (e.g., 0.1.1, 1.0.0, 0.2.0-alpha)"
    } while (-not $Version)
}

# Validate version format
if ($Version -notmatch '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$') {
    Write-Host "ERROR: Invalid version format! Use semantic versioning (e.g., 1.0.0 or 1.0.0-alpha)" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host ""
Write-Host "Preparing release v$Version..." -ForegroundColor Green
Write-Host "Target platform: $Platform" -ForegroundColor Green
Write-Host ""

# Function to restore backup files
function Restore-BackupFiles {
    Write-Host ""
    Write-Host "=================================================" -ForegroundColor Red
    Write-Host "ERROR: Build failed! Restoring backup files..." -ForegroundColor Red
    Write-Host "=================================================" -ForegroundColor Red
    Write-Host ""
    
    if (Test-Path "Cargo.toml.backup") {
        Copy-Item "Cargo.toml.backup" "Cargo.toml" -Force
        Remove-Item "Cargo.toml.backup" -Force
    }
    if (Test-Path "apps\desktop\package.json.backup") {
        Copy-Item "apps\desktop\package.json.backup" "apps\desktop\package.json" -Force
        Remove-Item "apps\desktop\package.json.backup" -Force
    }
    if (Test-Path "apps\desktop\src-tauri\Cargo.toml.backup") {
        Copy-Item "apps\desktop\src-tauri\Cargo.toml.backup" "apps\desktop\src-tauri\Cargo.toml" -Force
        Remove-Item "apps\desktop\src-tauri\Cargo.toml.backup" -Force
    }
    if (Test-Path "apps\desktop\src-tauri\tauri.conf.json.backup") {
        Copy-Item "apps\desktop\src-tauri\tauri.conf.json.backup" "apps\desktop\src-tauri\tauri.conf.json" -Force
        Remove-Item "apps\desktop\src-tauri\tauri.conf.json.backup" -Force
    }
    Write-Host "Backup files restored. Please fix the issues and try again." -ForegroundColor Yellow
    Read-Host "Press Enter to exit"
    exit 1
}

# Create backup of version files
Write-Host "Creating backup of version files..." -ForegroundColor Yellow
Copy-Item "Cargo.toml" "Cargo.toml.backup" -Force
Copy-Item "apps\desktop\package.json" "apps\desktop\package.json.backup" -Force
Copy-Item "apps\desktop\src-tauri\Cargo.toml" "apps\desktop\src-tauri\Cargo.toml.backup" -Force
Copy-Item "apps\desktop\src-tauri\tauri.conf.json" "apps\desktop\src-tauri\tauri.conf.json.backup" -Force

try {
    # Update version in workspace Cargo.toml
    Write-Host "Updating workspace Cargo.toml..." -ForegroundColor Yellow
    (Get-Content 'Cargo.toml') -replace "version = `"$currentVersion`"", "version = `"$Version`"" | Set-Content 'Cargo.toml'

    # Update version in desktop package.json
    Write-Host "Updating desktop package.json..." -ForegroundColor Yellow
    (Get-Content 'apps\desktop\package.json') -replace "`"$currentVersion`"", "`"$Version`"" | Set-Content 'apps\desktop\package.json'

    # Update version in desktop Cargo.toml
    Write-Host "Updating desktop Cargo.toml..." -ForegroundColor Yellow
    (Get-Content 'apps\desktop\src-tauri\Cargo.toml') -replace "version = `"$currentVersion`"", "version = `"$Version`"" | Set-Content 'apps\desktop\src-tauri\Cargo.toml'

    # Update version in tauri.conf.json
    Write-Host "Updating tauri.conf.json..." -ForegroundColor Yellow
    (Get-Content 'apps\desktop\src-tauri\tauri.conf.json') -replace "`"$currentVersion`"", "`"$Version`"" | Set-Content 'apps\desktop\src-tauri\tauri.conf.json'

    Write-Host ""
    Write-Host "=================================================" -ForegroundColor Cyan
    Write-Host "Building and testing the application..." -ForegroundColor Cyan
    Write-Host "=================================================" -ForegroundColor Cyan
    Write-Host ""

    # Clean previous builds
    Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
    Push-Location "apps\desktop"
    if (Test-Path "dist") { Remove-Item "dist" -Recurse -Force }
    if (Test-Path "src-tauri\target\release") { Remove-Item "src-tauri\target\release" -Recurse -Force }
    Pop-Location

    # Run tests
    if (-not $SkipTests) {
        Write-Host "Running cargo check..." -ForegroundColor Yellow
        cargo check --workspace
        if ($LASTEXITCODE -ne 0) {
            Write-Host "ERROR: Cargo check failed!" -ForegroundColor Red
            Restore-BackupFiles
        }
    }

    # Build the frontend
    Write-Host "Building frontend..." -ForegroundColor Yellow
    Push-Location "apps\desktop"
    npm run build
    if ($LASTEXITCODE -ne 0) {
        Pop-Location
        Write-Host "ERROR: Frontend build failed!" -ForegroundColor Red
        Restore-BackupFiles
    }
    Pop-Location

    # Build the Tauri application
    Write-Host "Building Tauri application for $Platform (this may take a while)..." -ForegroundColor Yellow
    Push-Location "apps\desktop"
    
    # Platform-specific build messages
    if ($Platform -ne "all") {
        switch ($Platform) {
            "windows" { 
                Write-Host "Building Windows installers (MSI, NSIS)..." -ForegroundColor Cyan
            }
            "macos" { 
                Write-Host "Building macOS installers (DMG, APP)..." -ForegroundColor Cyan
                if ($env:OS -eq "Windows_NT") {
                    Write-Host "Warning: Building for macOS from Windows may have limitations" -ForegroundColor Yellow
                }
            }
            "linux" { 
                Write-Host "Building Linux installers (DEB, AppImage)..." -ForegroundColor Cyan
                if ($env:OS -eq "Windows_NT") {
                    Write-Host "Warning: Building for Linux from Windows may have limitations" -ForegroundColor Yellow
                }
            }
        }
    } else {
        Write-Host "Building for all supported platforms..." -ForegroundColor Cyan
    }
    
    # Execute build
    npm run tauri build
    if ($LASTEXITCODE -ne 0) {
        Pop-Location
        Write-Host "ERROR: Tauri build failed!" -ForegroundColor Red
        Restore-BackupFiles
    }
    Pop-Location

    Write-Host ""
    Write-Host "=================================================" -ForegroundColor Green
    Write-Host "Build successful! Creating git release..." -ForegroundColor Green
    Write-Host "=================================================" -ForegroundColor Green
    Write-Host ""

    # Commit version changes
    Write-Host "Committing version changes..." -ForegroundColor Yellow
    git add Cargo.toml apps\desktop\package.json apps\desktop\src-tauri\Cargo.toml apps\desktop\src-tauri\tauri.conf.json
    git commit -m "Release v$Version"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Git commit failed!" -ForegroundColor Red
        Restore-BackupFiles
    }

    # Create git tag
    Write-Host "Creating git tag v$Version..." -ForegroundColor Yellow
    git tag -a "v$Version" -m "Release v$Version"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Git tag creation failed!" -ForegroundColor Red
        Restore-BackupFiles
    }

    # Push changes and tag
    if (-not $SkipPush) {
        Write-Host ""
        $pushChoice = Read-Host "Push changes and tag to remote? (y/n)"
        if ($pushChoice -eq "y" -or $pushChoice -eq "Y") {
            Write-Host "Pushing changes..." -ForegroundColor Yellow
            git push origin main
            if ($LASTEXITCODE -ne 0) {
                Write-Host "WARNING: Failed to push commits, but release was built successfully" -ForegroundColor Yellow
            }
            
            Write-Host "Pushing tag..." -ForegroundColor Yellow
            git push origin "v$Version"
            if ($LASTEXITCODE -ne 0) {
                Write-Host "WARNING: Failed to push tag, but release was built successfully" -ForegroundColor Yellow
            }
        }
    }

    # Clean up backup files
    Write-Host "Cleaning up backup files..." -ForegroundColor Yellow
    Remove-Item "Cargo.toml.backup" -Force -ErrorAction SilentlyContinue
    Remove-Item "apps\desktop\package.json.backup" -Force -ErrorAction SilentlyContinue
    Remove-Item "apps\desktop\src-tauri\Cargo.toml.backup" -Force -ErrorAction SilentlyContinue
    Remove-Item "apps\desktop\src-tauri\tauri.conf.json.backup" -Force -ErrorAction SilentlyContinue

    Write-Host ""
    Write-Host "=================================================" -ForegroundColor Green
    Write-Host "SUCCESS! Release v$Version created!" -ForegroundColor Green
    Write-Host "=================================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Build artifacts are located in:" -ForegroundColor Yellow
    Write-Host "- apps\desktop\src-tauri\target\release\bundle\" -ForegroundColor White
    Write-Host ""
    Write-Host "Platform-specific installers:" -ForegroundColor Yellow
    if ($Platform -eq "all" -or $Platform -eq "windows") {
        Write-Host "  🪟 Windows: .msi, .exe (NSIS installer)" -ForegroundColor Cyan
    }
    if ($Platform -eq "all" -or $Platform -eq "macos") {
        Write-Host "  🍎 macOS: .dmg, .app bundle" -ForegroundColor Cyan
    }
    if ($Platform -eq "all" -or $Platform -eq "linux") {
        Write-Host "  🐧 Linux: .deb, .AppImage" -ForegroundColor Cyan
    }
    Write-Host ""
    Write-Host "Git tag created: v$Version" -ForegroundColor Yellow
    Write-Host "App version updated in Help > About dialog" -ForegroundColor Yellow
    Write-Host ""
    if (-not $SkipPush -and ($pushChoice -eq "y" -or $pushChoice -eq "Y")) {
        Write-Host "You can now create a GitHub release at:" -ForegroundColor Yellow
        Write-Host "https://github.com/janvanwassenhove/mITyGuitar/releases/new?tag=v$Version" -ForegroundColor Cyan
        Write-Host ""
    }

} catch {
    Write-Host "ERROR: An unexpected error occurred: $_" -ForegroundColor Red
    Restore-BackupFiles
}

Read-Host "Press Enter to exit"