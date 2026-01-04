# Cross-Platform Release Script for GitHub Actions/CI
# This script is designed to be run in CI environments with cross-compilation support

param(
    [Parameter(Mandatory=$true)]
    [string]$Version,
    
    [Parameter(Mandatory=$false)]
    [ValidateSet("windows", "macos", "linux")]
    [string]$Platform = "windows",
    
    [Parameter(Mandatory=$false)]
    [string]$Architecture = "x64"
)

Write-Host ""
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host "    mITyGuitar Cross-Platform Release Builder" -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "Building for: $Platform ($Architecture)" -ForegroundColor Yellow
Write-Host "Version: $Version" -ForegroundColor Yellow
Write-Host ""

# Function to update version in files
function Update-Version {
    param([string]$NewVersion)
    
    Write-Host "Updating version to $NewVersion..." -ForegroundColor Yellow
    
    # Update workspace Cargo.toml
    (Get-Content 'Cargo.toml') -replace 'version = "[^"]*"', "version = `"$NewVersion`"" | Set-Content 'Cargo.toml'
    
    # Update desktop package.json
    (Get-Content 'apps\desktop\package.json') -replace '"version": "[^"]*"', "`"version`": `"$NewVersion`"" | Set-Content 'apps\desktop\package.json'
    
    # Update desktop Cargo.toml
    (Get-Content 'apps\desktop\src-tauri\Cargo.toml') -replace 'version = "[^"]*"', "version = `"$NewVersion`"" | Set-Content 'apps\desktop\src-tauri\Cargo.toml'
    
    # Update tauri.conf.json
    (Get-Content 'apps\desktop\src-tauri\tauri.conf.json') -replace '"version": "[^"]*"', "`"version`": `"$NewVersion`"" | Set-Content 'apps\desktop\src-tauri\tauri.conf.json'
}

# Update versions
Update-Version -NewVersion $Version

# Build frontend
Write-Host "Building frontend..." -ForegroundColor Yellow
Push-Location "apps\desktop"
npm run build
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    Write-Host "ERROR: Frontend build failed!" -ForegroundColor Red
    exit 1
}
Pop-Location

# Build for specific platform
Write-Host "Building Tauri application for $Platform..." -ForegroundColor Yellow
Push-Location "apps\desktop"

switch ($Platform) {
    "windows" {
        Write-Host "Building Windows installers..." -ForegroundColor Cyan
        npm run tauri build
        
        # List Windows artifacts
        Write-Host ""
        Write-Host "Windows artifacts created:" -ForegroundColor Green
        $bundlePath = "src-tauri\target\release\bundle"
        if (Test-Path "$bundlePath\msi") {
            Get-ChildItem "$bundlePath\msi\*.msi" | ForEach-Object { Write-Host "  📦 MSI: $($_.Name)" -ForegroundColor White }
        }
        if (Test-Path "$bundlePath\nsis") {
            Get-ChildItem "$bundlePath\nsis\*.exe" | ForEach-Object { Write-Host "  📦 NSIS: $($_.Name)" -ForegroundColor White }
        }
    }
    
    "macos" {
        Write-Host "Building macOS installers..." -ForegroundColor Cyan
        # For macOS, we need to ensure we're on macOS or have cross-compilation set up
        if ($env:RUNNER_OS -eq "macOS" -or $env:OS -ne "Windows_NT") {
            npm run tauri build
            
            # List macOS artifacts
            Write-Host ""
            Write-Host "macOS artifacts created:" -ForegroundColor Green
            $bundlePath = "src-tauri\target\release\bundle"
            if (Test-Path "$bundlePath\dmg") {
                Get-ChildItem "$bundlePath\dmg\*.dmg" | ForEach-Object { Write-Host "  📦 DMG: $($_.Name)" -ForegroundColor White }
            }
            if (Test-Path "$bundlePath\macos") {
                Get-ChildItem "$bundlePath\macos\*.app" -Directory | ForEach-Object { Write-Host "  📦 APP: $($_.Name)" -ForegroundColor White }
            }
        } else {
            Write-Host "ERROR: macOS builds require macOS environment or cross-compilation setup" -ForegroundColor Red
            Pop-Location
            exit 1
        }
    }
    
    "linux" {
        Write-Host "Building Linux installers..." -ForegroundColor Cyan
        # For Linux, we need to ensure we're on Linux or have cross-compilation set up
        if ($env:RUNNER_OS -eq "Linux" -or ($env:OS -ne "Windows_NT" -and $IsLinux)) {
            npm run tauri build
            
            # List Linux artifacts
            Write-Host ""
            Write-Host "Linux artifacts created:" -ForegroundColor Green
            $bundlePath = "src-tauri\target\release\bundle"
            if (Test-Path "$bundlePath\deb") {
                Get-ChildItem "$bundlePath\deb\*.deb" | ForEach-Object { Write-Host "  📦 DEB: $($_.Name)" -ForegroundColor White }
            }
            if (Test-Path "$bundlePath\appimage") {
                Get-ChildItem "$bundlePath\appimage\*.AppImage" | ForEach-Object { Write-Host "  📦 AppImage: $($_.Name)" -ForegroundColor White }
            }
        } else {
            Write-Host "ERROR: Linux builds require Linux environment or cross-compilation setup" -ForegroundColor Red
            Pop-Location
            exit 1
        }
    }
}

Pop-Location

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "=================================================" -ForegroundColor Green
    Write-Host "SUCCESS! $Platform release v$Version created!" -ForegroundColor Green
    Write-Host "=================================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Build artifacts are located in:" -ForegroundColor Yellow
    Write-Host "- apps\desktop\src-tauri\target\release\bundle\" -ForegroundColor White
    Write-Host ""
} else {
    Write-Host ""
    Write-Host "ERROR: Build failed for $Platform" -ForegroundColor Red
    exit 1
}