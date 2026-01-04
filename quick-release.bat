@echo off
setlocal enabledelayedexpansion

echo.
echo =================================================
echo      mITyGuitar Quick Release Builder
echo =================================================
echo.

REM Check if we're in a git repository
git rev-parse --git-dir >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Not in a git repository!
    pause
    exit /b 1
)

REM Get current version
for /f "tokens=2 delims=:" %%a in ('type "Cargo.toml" ^| findstr /C:"version = "') do (
    set "current_version=%%a"
    goto :found_version
)
:found_version
set "current_version=%current_version: =%"
set "current_version=%current_version:"=%"

echo Current version: %current_version%
echo.

REM Prompt for new version
:prompt_version
set /p "new_version=Enter new version (e.g., 0.1.1-dev, 0.2.0-alpha): "
if "%new_version%"=="" (
    echo Please enter a version!
    goto :prompt_version
)

echo.
echo Preparing quick release v%new_version%...
echo.

REM Update version in all files
echo Updating version files...
powershell -Command "(Get-Content 'Cargo.toml') -replace 'version = \"%current_version%\"', 'version = \"%new_version%\"' | Set-Content 'Cargo.toml'"
powershell -Command "(Get-Content 'apps\desktop\package.json') -replace '\"%current_version%\"', '\"%new_version%\"' | Set-Content 'apps\desktop\package.json'"
powershell -Command "(Get-Content 'apps\desktop\src-tauri\Cargo.toml') -replace 'version = \"%current_version%\"', 'version = \"%new_version%\"' | Set-Content 'apps\desktop\src-tauri\Cargo.toml'"
powershell -Command "(Get-Content 'apps\desktop\src-tauri\tauri.conf.json') -replace '\"%current_version%\"', '\"%new_version%\"' | Set-Content 'apps\desktop\src-tauri\tauri.conf.json'"

echo.
echo Quick check...
cargo check --workspace
if %errorlevel% neq 0 (
    echo ERROR: Quick check failed!
    pause
    exit /b 1
)

echo.
echo Building debug release (faster build)...
cd apps\desktop
call npm run build
if %errorlevel% neq 0 (
    echo ERROR: Frontend build failed!
    pause
    exit /b 1
)

call npm run tauri build -- --debug
if %errorlevel% neq 0 (
    echo ERROR: Tauri build failed!
    pause
    exit /b 1
)
cd ..\..

echo.
echo Committing and tagging...
git add -A
git commit -m "Quick release v%new_version%"
git tag -a "v%new_version%" -m "Quick release v%new_version%"

echo.
echo =================================================
echo Quick release v%new_version% created!
echo =================================================
echo.
echo Debug build artifacts are located in:
echo - apps\desktop\src-tauri\target\debug\bundle\
echo.
echo To push: git push origin main && git push origin v%new_version%
echo.

pause