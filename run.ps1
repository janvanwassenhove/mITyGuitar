# mITyGuitar Build Script
# Quick setup and run for development

Write-Host "🎸 mITyGuitar Setup & Run Script" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Check prerequisites
Write-Host "Checking prerequisites..." -ForegroundColor Yellow

# Check Rust
try {
    $rustVersion = rustc --version
    Write-Host "✓ Rust installed: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Rust not found! Please install from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check Node.js
try {
    $nodeVersion = node --version
    Write-Host "✓ Node.js installed: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Node.js not found! Please install from https://nodejs.org/" -ForegroundColor Red
    exit 1
}

# Check npm
try {
    $npmVersion = npm --version
    Write-Host "✓ npm installed: v$npmVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ npm not found!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "All prerequisites met!" -ForegroundColor Green
Write-Host ""

# Navigate to desktop app
Set-Location -Path "apps\desktop"

# Check if node_modules exists
if (-Not (Test-Path "node_modules")) {
    Write-Host "Installing npm dependencies (this may take a few minutes)..." -ForegroundColor Yellow
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ npm install failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ npm dependencies installed" -ForegroundColor Green
    Write-Host ""
}

# Run the app
Write-Host "Starting mITyGuitar..." -ForegroundColor Cyan
Write-Host ""
Write-Host "This will:" -ForegroundColor White
Write-Host "  1. Start Vite dev server" -ForegroundColor White
Write-Host "  2. Build Rust backend" -ForegroundColor White
Write-Host "  3. Launch the desktop app" -ForegroundColor White
Write-Host ""
Write-Host "First build may take 2-5 minutes..." -ForegroundColor Yellow
Write-Host ""
Write-Host "Once launched, try:" -ForegroundColor Cyan
Write-Host "  - Press '1' key (Green fret)" -ForegroundColor White
Write-Host "  - Press Space (Strum down)" -ForegroundColor White
Write-Host "  - You should hear a chord!" -ForegroundColor White
Write-Host ""
Write-Host "Press Ctrl+C to stop the app." -ForegroundColor Yellow
Write-Host ""

npm run tauri:dev
