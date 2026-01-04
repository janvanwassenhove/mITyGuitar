# Build Instructions

This guide covers building mITyGuitar from source for development and distribution.

## Prerequisites

### Required Tools
- **Rust 1.70+**: [Install via rustup](https://rustup.rs/)
- **Node.js 18+**: [Download from nodejs.org](https://nodejs.org/)
- **Git**: For cloning the repository

### Platform-Specific Requirements

**Windows:**
- Windows 10/11
- Visual Studio Build Tools or Visual Studio Community
- Windows SDK

**macOS:**
- macOS 10.15+
- Xcode Command Line Tools: `xcode-select --install`

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install build-essential libgtk-3-dev libwebkit2gtk-4.0-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Linux (Fedora):**
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install gtk3-devel webkit2gtk3-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel
```

**Linux (Arch):**
```bash
sudo pacman -S base-devel gtk3 webkit2gtk openssl libappindicator-gtk3 librsvg
```

## Quick Setup

### 1. Clone Repository
```powershell
git clone https://github.com/janvanwassenhove/mITyGuitar.git
cd mITyGuitar
```

### 2. Build & Run (Development)

**Windows - Quick start:**
```cmd
# Use included batch file
start.bat
```

**All platforms:**
```powershell
# Install frontend dependencies
cd apps/desktop
npm install

# Run in development mode (with hot-reload)
npm run tauri:dev
```

This will:
- Start Vite dev server for React frontend
- Compile Rust backend incrementally
- Launch desktop app with hot-reload enabled

## Development Workflow

### Backend Development (Rust)

**Build all crates:**
```powershell
# From project root
cargo build --workspace
```

**Run tests:**
```powershell
# All tests
cargo test --workspace

# Specific crate
cargo test -p controller
cargo test -p mapping  
cargo test -p audio
cargo test -p config
```

**Format code:**
```powershell
cargo fmt --all
```

**Lint code:**
```powershell
cargo clippy --workspace -- -D warnings
```

### Frontend Development (React)

**Start development server:**
```powershell
cd apps/desktop
npm run dev        # Vite dev server only
npm run tauri:dev  # Full Tauri app with backend
```

**Type checking:**
```powershell
npm run type-check
```

**Lint frontend:**
```powershell
npm run lint
```

### Full Development Setup

**With file watching and hot-reload:**
```powershell
# Terminal 1: Backend development
cargo watch -x "build --workspace"

# Terminal 2: Frontend development  
cd apps/desktop
npm run tauri:dev
```

## Production Builds

### Desktop Application

**Build optimized app bundle:**
```powershell
cd apps/desktop
npm run tauri:build
```

**Output locations:**
- **Windows**: `src-tauri/target/release/bundle/msi/`
- **macOS**: `src-tauri/target/release/bundle/dmg/`
- **Linux**: `src-tauri/target/release/bundle/deb/` and `bundle/appimage/`

### Rust Binaries Only

**Build optimized Rust workspace:**
```powershell
cargo build --release --workspace
```

Binaries will be in: `target/release/`

## Configuration

### Build Features

**SoundFont support (default):**
```powershell
cargo build --features "audio/soundfont"
```

**Without SoundFont (smaller binary):**
```powershell
cargo build --no-default-features
```

### Environment Variables

**Rust build settings:**
```powershell
# Release optimization
$env:CARGO_PROFILE_RELEASE_LTO = "fat"
$env:CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1"

# Development debugging
$env:RUST_LOG = "debug"
$env:RUST_BACKTRACE = "1"
```

**Tauri settings:**
```powershell
# Disable dev mode for testing production-like build
$env:TAURI_DEV = "false"
```

## Troubleshooting

### Common Build Errors

**"Rust compiler not found":**
```powershell
# Update Rust
rustup update

# Verify installation
rustc --version
cargo --version
```

**"Node.js modules missing":**
```powershell
cd apps/desktop
Remove-Item node_modules -Recurse -Force  # Windows
rm -rf node_modules                        # macOS/Linux
npm install
```

**"Tauri CLI not found":**
```powershell
cd apps/desktop
npm install @tauri-apps/cli --save-dev
```

**"Build tools missing" (Windows):**
- Install Visual Studio Build Tools
- Or install Visual Studio Community with C++ development tools

**"GTK development libraries missing" (Linux):**
```bash
# Ubuntu/Debian
sudo apt install libgtk-3-dev libwebkit2gtk-4.0-dev

# Fedora  
sudo dnf install gtk3-devel webkit2gtk3-devel

# Arch
sudo pacman -S gtk3 webkit2gtk
```

### Performance Issues

**Slow compilation:**
```powershell
# Enable parallel compilation
$env:CARGO_BUILD_JOBS = "4"  # Adjust to CPU cores

# Use faster linker (Linux)
sudo apt install lld
$env:RUSTFLAGS = "-C link-arg=-fuse-ld=lld"
```

**Large binary size:**
```toml
# Add to Cargo.toml [profile.release]
opt-level = "z"     # Optimize for size
lto = true         # Link-time optimization
codegen-units = 1  # Single codegen unit
panic = "abort"    # Smaller panic handling
strip = true       # Strip debug symbols
```

### Audio Issues

**No audio output in development:**
1. Check system audio settings
2. Try different sample rates in config
3. Verify audio device permissions

**Crackling/stuttering:**
1. Increase buffer size in config: `256` → `512`
2. Close other audio applications
3. Check CPU usage during development

## Performance Optimization

### Development Builds

**Faster incremental compilation:**
```toml
# Add to Cargo.toml
[profile.dev]
opt-level = 1      # Basic optimization
debug = true       # Keep debug info
incremental = true # Enable incremental compilation
```

### Release Builds

**Maximum optimization:**
```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = "fat"          # Full link-time optimization
codegen-units = 1    # Single codegen unit
panic = "abort"      # Smaller panic handling
strip = true         # Strip debug symbols
```

## Testing

### Unit Tests
```powershell
# All tests with output
cargo test --workspace --verbose

# Test specific functionality
cargo test chord_resolution
cargo test audio_rendering
cargo test config_serialization
```

### Integration Tests
```powershell
# Run app and test basic functionality
cd apps/desktop
npm run tauri:dev

# Test audio output
# Test controller simulation  
# Test song loading
# Test configuration persistence
```

### Cross-Platform Testing

**Windows:**
- Test on Windows 10 and 11
- Verify both x64 and ARM64 if applicable
- Test installer/MSI package

**macOS:**
- Test on Intel and Apple Silicon
- Verify code signing (for distribution)
- Test DMG installer

**Linux:**
- Test on Ubuntu, Fedora, Arch
- Verify both x64 and ARM64
- Test AppImage and DEB packages

## Continuous Integration

### GitHub Actions

Example workflow for automated builds:

```yaml
name: Build
on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
    
    - name: Install dependencies (Linux)
      if: matrix.os == 'ubuntu-latest'
      run: |
        sudo apt update
        sudo apt install -y libgtk-3-dev libwebkit2gtk-4.0-dev libssl-dev
    
    - name: Cache Rust dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ matrix.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Build Rust workspace
      run: cargo build --release --workspace
    
    - name: Run tests
      run: cargo test --workspace
    
    - name: Build Tauri app
      run: |
        cd apps/desktop
        npm install
        npm run tauri:build
```

## Distribution

### Code Signing

**Windows:**
- Obtain code signing certificate
- Configure in `tauri.conf.json`

**macOS:**
- Join Apple Developer Program
- Configure code signing identity
- Notarize app for distribution

### App Store Distribution

**Microsoft Store:**
- Package as MSIX
- Submit via Partner Center

**Mac App Store:**
- Follow Apple guidelines
- Submit via App Store Connect

### Direct Distribution

- Host releases on GitHub
- Provide checksums for security
- Document installation instructions
- Consider auto-updater implementation

## Next Steps

After successful build:
1. **Test thoroughly** on target platforms
2. **Verify audio performance** across different hardware
3. **Test with real controllers** if available
4. **Check memory usage** and optimize if needed
5. **Document any platform-specific quirks**

For help with specific build issues, check:
- [Tauri Documentation](https://tauri.app/)
- [Rust Build Guide](https://doc.rust-lang.org/cargo/)
- [Project Issues](https://github.com/janvanwassenhove/mITyGuitar/issues)

Happy building! 🦀🎸