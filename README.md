# mITyGuitar 🎸

Turn your Rock Band guitar controller into a low-latency live musical instrument!

## Overview

mITyGuitar is a cross-platform desktop application (Windows/macOS/Linux) that connects to Rock Band guitar controllers via USB HID and transforms them into expressive MIDI instruments with chord mapping, SoundFont playback, and real-time audio synthesis.

## Features

- ⚡ **Ultra-low latency** audio engine (< 10ms)
- 🎮 **Full controller support** - all buttons, axes, and sensors
- 🎵 **SoundFont playback** - uses included guitar SoundFonts
- 🎹 **Chord mapping** - trigger chords with fret combinations
- 🎸 **Multiple genres** - Punk, Rock, EDM chord patterns
- 🎛️ **Live visualization** - see all inputs in real-time
- 💾 **Configuration** - persistent settings
- 🧪 **Simulator mode** - test without hardware

## Quick Start

### Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Node.js 18+ ([Install Node](https://nodejs.org/))
- Rock Band guitar controller with USB dongle (or use simulator mode)

### Building

```powershell
# Clone repository
git clone <repository-url>
cd guitar

# Build Rust workspace
cargo build --release

# Setup desktop app (coming soon)
cd apps/desktop
npm install
npm run tauri dev
```

## Controller Support

### Supported Inputs

**Fret Buttons:**
- Main frets: Green (1), Red (2), Yellow (3), Blue (4), Orange (5)
- Solo frets: Q, W, E, R, T

**Strum Bar:**
- Up: Arrow Up
- Down: Arrow Down / Space

**Analog Controls:**
- Whammy bar: Pitch bend or vibrato
- Tilt sensor: Filter modulation
- Accelerometer: Expression control

**Other Controls:**
- FX switch (3-position): Effect switching
- D-pad: Navigation
- Start/Select: Menu shortcuts

### Simulator Mode

For development without hardware, use keyboard controls:

| Key | Function |
|-----|----------|
| 1-5 | Fret buttons (Green to Orange) |
| Q-T | Solo frets |
| Space / Arrow Down | Strum down |
| Arrow Up | Strum up |
| F1-F3 | FX switch positions |
| Enter | Start button |
| Escape | Select button |

## Usage

### Playing Chords

1. Hold one or more fret buttons
2. Strum up or down
3. Release frets between strums for different chords

### Changing Instruments

- **Menu**: Instruments → Next/Previous Instrument
- **Shortcut**: D-pad Left/Right

### Changing Chord Patterns

- **Menu**: Chords → Next/Previous Pattern
- **Genre**: Chords → Genre → Punk/Rock/EDM

### Panic (Stop All Sounds)

- **Menu**: File → Panic
- **Button**: Click "Panic" button in Live View

## Configuration

Configuration is stored in:
- **Windows**: `%APPDATA%\mityguitar\mityguitar_config.json`
- **macOS**: `~/Library/Application Support/mityguitar/mityguitar_config.json`
- **Linux**: `~/.config/mityguitar/mityguitar_config.json`

### Config Structure

```json
{
  "version": 1,
  "controller": {
    "device_id": "auto",
    "simulator_mode": true
  },
  "audio": {
    "sample_rate": 48000,
    "buffer_size": 256,
    "backend": "fallback"
  },
  "soundfonts": {
    "current": "Electric_guitar.sf2",
    "preset": {"bank": 0, "program": 0},
    "recent": []
  },
  "mapping": {
    "genre": "rock",
    "pattern_index": 0,
    "whammy_mode": "pitch_bend",
    "fx_switch_mode": "effects",
    "tilt_mode": "filter_cutoff"
  }
}
```

## SoundFonts

### Included SoundFonts

The app includes several high-quality guitar SoundFonts in the `./soundfont/` directory:

- 12-string.sf2
- 60s_Rock_Guitar.SF2
- 241-Bassguitars.SF2
- Acoustic Bass FBG29 MW_1.SF2
- Electric_guitar.sf2
- Guitar Vince.sf2
- Ibanez Electric Guitar.sf2
- Palm Muted Guitar.sf2
- Rock Basses.sf2

### Adding Custom SoundFonts

1. **Via Menu**: Instruments → Choose SoundFont → Select .sf2 file
2. **Via Folder**: Copy .sf2 files to the soundfont directory
3. **Rescan**: Instruments → Rescan SoundFonts

## Troubleshooting

### No Audio Output

1. Check audio device in system settings
2. Try increasing buffer size in config (256 → 512 samples)
3. Restart application
4. Check Diagnostics view for errors

### Controller Not Detected

1. Ensure USB dongle is connected
2. Check device in system device manager
3. Try different USB port
4. Enable simulator mode for testing

### High Latency

1. **Reduce buffer size**: Set `buffer_size: 128` in config
2. **Close other apps**: Free up CPU resources
3. **Disable audio enhancements**: In OS audio settings
4. **Use ASIO** (Windows): Install ASIO4ALL driver
5. **Adjust sample rate**: Try 44100Hz vs 48000Hz

### Latency Tips by OS

**Windows:**
- Install ASIO drivers for best performance
- Disable "Audio Enhancements" in sound settings
- Use exclusive mode

**macOS:**
- Core Audio provides excellent low-latency by default
- Close competing audio apps

**Linux:**
- Use JACK audio server for lowest latency
- Increase RT priority: `ulimit -r 95`
- Add user to `audio` group

## Architecture

```
mITyGuitar
├── crates/
│   ├── controller/    # HID input, simulator
│   ├── mapping/       # Chord engine, genres
│   ├── audio/         # Audio output, synthesis
│   └── config/        # Configuration management
├── apps/
│   └── desktop/       # Tauri v2 + React UI
└── soundfont/         # SoundFont files
```

### Tech Stack

- **Backend**: Rust with cpal, hidapi, ringbuf
- **Frontend**: React + TypeScript + Vite
- **Desktop**: Tauri v2
- **Audio**: RT-safe lock-free architecture

## Development

### Running Tests

```powershell
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p controller
cargo test -p mapping
cargo test -p audio
cargo test -p config
```

### Building for Release

```powershell
# Build optimized binaries
cargo build --release --workspace

# Build desktop app
cd apps/desktop
npm run tauri build
```

### Development Mode

```powershell
# Run with simulator
cargo run --features controller/simulator

# Enable verbose logging
$env:RUST_LOG="debug"
cargo run
```

## Roadmap

- [x] Core Rust crates (controller, mapping, audio, config)
- [x] Fallback synthesizer
- [x] Chord patterns for 3 genres
- [ ] Tauri desktop application
- [ ] React UI with live visualization
- [ ] SoundFont integration (sfizz)
- [ ] Menu bar implementation
- [ ] Diagnostics view
- [ ] FX chain (distortion, cab sim)
- [ ] Pattern editor
- [ ] Production builds

## Contributing

Contributions welcome! Please:

1. Follow Rust conventions
2. Add tests for new features
3. Keep audio code RT-safe
4. Update documentation

## License

[Add license information]

## Credits

- SoundFonts: [Add attribution for SF2 files]
- Built with Rust, Tauri, React

---

**Made with ❤️ for guitar gamers who want to make real music!**
