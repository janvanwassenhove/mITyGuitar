@echo off
setlocal enabledelayedexpansion

echo.
echo =================================================
echo          mITyGuitar Release Builder
echo =================================================
echo.

REM Check if we're in a git repository
git rev-parse --git-dir >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Not in a git repository!
    pause
    exit /b 1
)

REM Check if working directory is clean
for /f %%i in ('git status --porcelain 2^>nul') do (
    echo ERROR: Working directory is not clean! Please commit or stash changes first.
    echo.
    git status --short
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
set /p "new_version=Enter new version (e.g., 0.1.1, 1.0.0, 0.2.0-alpha): "
if "%new_version%"=="" (
    echo Please enter a version!
    goto :prompt_version
)

REM Validate version format (basic check)
echo %new_version% | findstr /R "^[0-9]\+\.[0-9]\+\.[0-9]\+\(-[a-zA-Z0-9]\+\)\?$" >nul
if %errorlevel% neq 0 (
    echo ERROR: Invalid version format! Use semantic versioning (e.g., 1.0.0 or 1.0.0-alpha)
    goto :prompt_version
)

echo.
echo Preparing release v%new_version%...
echo.

REM Create backup of version files
echo Creating backup of version files...
copy "Cargo.toml" "Cargo.toml.backup" >nul
copy "apps\desktop\package.json" "apps\desktop\package.json.backup" >nul
copy "apps\desktop\src-tauri\Cargo.toml" "apps\desktop\src-tauri\Cargo.toml.backup" >nul
copy "apps\desktop\src-tauri\tauri.conf.json" "apps\desktop\src-tauri\tauri.conf.json.backup" >nul

REM Update version in workspace Cargo.toml
echo Updating workspace Cargo.toml...
powershell -Command "(Get-Content 'Cargo.toml') -replace 'version = \"%current_version%\"', 'version = \"%new_version%\"' | Set-Content 'Cargo.toml'"

REM Update version in desktop package.json
echo Updating desktop package.json...
powershell -Command "(Get-Content 'apps\desktop\package.json') -replace '\"%current_version%\"', '\"%new_version%\"' | Set-Content 'apps\desktop\package.json'"

REM Update version in desktop Cargo.toml
echo Updating desktop Cargo.toml...
powershell -Command "(Get-Content 'apps\desktop\src-tauri\Cargo.toml') -replace 'version = \"%current_version%\"', 'version = \"%new_version%\"' | Set-Content 'apps\desktop\src-tauri\Cargo.toml'"

REM Update version in tauri.conf.json
echo Updating tauri.conf.json...
powershell -Command "(Get-Content 'apps\desktop\src-tauri\tauri.conf.json') -replace '\"%current_version%\"', '\"%new_version%\"' | Set-Content 'apps\desktop\src-tauri\tauri.conf.json'"

echo.
echo =================================================
echo Building and testing the application...
echo =================================================
echo.

REM Clean previous builds
echo Cleaning previous builds...
cd apps\desktop
if exist "dist" rmdir /s /q "dist" >nul 2>&1
if exist "src-tauri\target\release" rmdir /s /q "src-tauri\target\release" >nul 2>&1
cd ..\..

REM Run tests (if any)
echo Running cargo check...
cargo check --workspace
if %errorlevel% neq 0 (
    echo ERROR: Cargo check failed!
    goto :restore_backup
)

REM Build the frontend
echo Building frontend...
cd apps\desktop
call npm run build
if %errorlevel% neq 0 (
    echo ERROR: Frontend build failed!
    cd ..\..
    goto :restore_backup
)
cd ..\..

REM Build the Tauri application in release mode
echo Building Tauri application (this may take a while)...
cd apps\desktop
call npm run tauri build
if %errorlevel% neq 0 (
    echo ERROR: Tauri build failed!
    cd ..\..
    goto :restore_backup
)
cd ..\..

echo.
echo =================================================
echo Build successful! Creating git release...
echo =================================================
echo.

REM Commit version changes
echo Committing version changes...
git add Cargo.toml apps\desktop\package.json apps\desktop\src-tauri\Cargo.toml apps\desktop\src-tauri\tauri.conf.json
git commit -m "Release v%new_version%"
if %errorlevel% neq 0 (
    echo ERROR: Git commit failed!
    goto :restore_backup
)

REM Create git tag
echo Creating git tag v%new_version%...
git tag -a "v%new_version%" -m "Release v%new_version%"
if %errorlevel% neq 0 (
    echo ERROR: Git tag creation failed!
    goto :restore_backup
)

REM Push changes and tag
echo.
set /p "push_choice=Push changes and tag to remote? (y/n): "
if /i "%push_choice%"=="y" (
    echo Pushing changes...
    git push origin main
    if %errorlevel% neq 0 (
        echo WARNING: Failed to push commits, but release was built successfully
    )
    
    echo Pushing tag...
    git push origin "v%new_version%"
    if %errorlevel% neq 0 (
        echo WARNING: Failed to push tag, but release was built successfully
    )
)

REM Clean up backup files
echo Cleaning up backup files...
del "Cargo.toml.backup" >nul 2>&1
del "apps\desktop\package.json.backup" >nul 2>&1
del "apps\desktop\src-tauri\Cargo.toml.backup" >nul 2>&1
del "apps\desktop\src-tauri\tauri.conf.json.backup" >nul 2>&1

echo.
echo =================================================
echo SUCCESS! Release v%new_version% created!
echo =================================================
echo.
echo Build artifacts are located in:
echo - apps\desktop\src-tauri\target\release\bundle\
echo.
echo Platform-specific installers:
echo   Windows: .msi, .exe (NSIS installer)
echo   macOS: .dmg, .app bundle (if built on macOS)
echo   Linux: .deb, .AppImage (if built on Linux)
echo.
echo Git tag created: v%new_version%
echo App version updated in Help ^> About dialog
echo.
if /i "%push_choice%"=="y" (
    echo You can now create a GitHub release at:
    echo https://github.com/janvanwassenhove/mITyGuitar/releases/new?tag=v%new_version%
    echo.
)

pause
goto :end

:restore_backup
echo.
echo =================================================
echo ERROR: Build failed! Restoring backup files...
echo =================================================
echo.
if exist "Cargo.toml.backup" (
    copy "Cargo.toml.backup" "Cargo.toml" >nul
    del "Cargo.toml.backup" >nul
)
if exist "apps\desktop\package.json.backup" (
    copy "apps\desktop\package.json.backup" "apps\desktop\package.json" >nul
    del "apps\desktop\package.json.backup" >nul
)
if exist "apps\desktop\src-tauri\Cargo.toml.backup" (
    copy "apps\desktop\src-tauri\Cargo.toml.backup" "apps\desktop\src-tauri\Cargo.toml" >nul
    del "apps\desktop\src-tauri\Cargo.toml.backup" >nul
)
if exist "apps\desktop\src-tauri\tauri.conf.json.backup" (
    copy "apps\desktop\src-tauri\tauri.conf.json.backup" "apps\desktop\src-tauri\tauri.conf.json" >nul
    del "apps\desktop\src-tauri\tauri.conf.json.backup" >nul
)
echo Backup files restored. Please fix the issues and try again.
pause
exit /b 1

:end
endlocal