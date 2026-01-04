# Quick Start Guide

## Running the App

### Prerequisites
- Rust 1.70+: https://rustup.rs/
- Node.js 18+: https://nodejs.org/
- Windows: No additional setup needed
- macOS: Xcode Command Line Tools
- Linux: See Tauri prerequisites

### Development Mode

1. **Install dependencies:**
```powershell
cd apps/desktop
npm install
```

2. **Run the app:**
```powershell
npm run tauri:dev
```

This will:
- Start the Vite dev server
- Build the Rust backend
- Launch the desktop app with hot-reload

### First Launch

The app starts in **simulator mode** by default, so no hardware is needed!

## Using the App

### Simulator Controls

| Key | Action |
|-----|--------|
| 1-5 | Fret buttons (Green, Red, Yellow, Blue, Orange) |
| Q-T | Solo fret buttons |
| Space | Strum down |
| Arrow Up | Strum up |

### Playing Your First Chord

1. Hold fret button **1** (Green)
2. Press **Space** to strum
3. You should hear a chord!

### Changing Sounds

**Via Menu:**
- Chords → Genre → Punk / Rock / EDM
- Chords → Next Pattern

**Via Buttons:**
- Click "Next Pattern" / "Prev Pattern"

### If No Sound

1. Check Diagnostics view (View → Diagnostics)
2. Verify buffer underruns = 0
3. Check your OS audio settings
4. Try the Panic button

## Building for Production

```powershell
cd apps/desktop
npm run tauri:build
```

Output will be in `src-tauri/target/release/bundle/`

## Troubleshooting

### Build Errors

**Rust compilation failed:**
```powershell
# Update Rust
rustup update

# Clean and rebuild
cd apps/desktop/src-tauri
cargo clean
cd ..
npm run tauri:dev
```

**Node modules issues:**
```powershell
cd apps/desktop
Remove-Item node_modules -Recurse -Force
npm install
```

### Audio Issues

**Crackling/stuttering:**
- Increase buffer size in config to 512
- Close other audio apps

**No audio output:**
- Check Windows audio mixer
- Verify default playback device
- Restart the app

## Next Steps

- Read [README.md](../README.md) for full documentation
- Check `.github/instructions/mITyGuitar.instructions.md` for architecture details
- Explore the crates for implementation details

## Keyboard Shortcuts

| Action | Keys |
|--------|------|
| Strum Down | Space, Arrow Down |
| Strum Up | Arrow Up |
| Green Fret | 1, Q |
| Red Fret | 2, W |
| Yellow Fret | 3, E |
| Blue Fret | 4, R |
| Orange Fret | 5, T |

Have fun making music! 🎸🎵
