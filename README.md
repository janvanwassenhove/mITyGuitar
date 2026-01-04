# mITyGuitar 🎸

Turn your Rock Band guitar controller into a low-latency live musical instrument!

## Overview

mITyGuitar is a cross-platform desktop application (Windows/macOS/Linux) that transforms Rock Band guitar controllers into expressive musical instruments. Play chords, trigger soundfonts, and perform with ultra-low latency audio synthesis.

**🎵 Play along to songs, create chord progressions, or jam with genre-based patterns**

## Features

### 🎸 **Musical Performance**
- 🎵 **Song Playback** - Play along to custom `.mitychart.json` songs with lyrics and chord changes
- 🎹 **Chord Mapping** - Genre-based chord patterns (Punk, Rock, EDM, Pop, Folk, Metal)
- 🎼 **SoundFont Integration** - High-quality guitar sounds from included SF2 files
- 🎚️ **Live Effects** - Whammy bar, sustain, and genre-specific FX

### 🎮 **Controller Support**
- 🎯 **Full Hardware Support** - Rock Band guitars via USB HID
- 🧪 **Simulator Mode** - Keyboard controls for development/testing
- 🎛️ **Live Visualization** - Real-time input display and diagnostics

### ⚡ **Performance**
- ⚡ **Ultra-low Latency** - Sub-10ms audio engine
- 💾 **Smart Configuration** - Persistent settings with hot-reload
- 🔧 **Cross-Platform** - Windows, macOS, Linux support

## Quick Start

### Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Node.js 18+ ([Install Node](https://nodejs.org/))
- Rock Band guitar controller with USB dongle (or use simulator mode)

### Building & Running

**Option 1: Using start.bat (Windows)**
```cmd
# Clone repository
git clone <repository-url>
cd guitar

# Start with included batch file
start.bat
```

**Option 2: Manual setup (All platforms)**
```powershell
# Clone repository
git clone <repository-url>
cd guitar

# Install dependencies and run
cd apps/desktop
npm install
npm run tauri:dev
```

### First Launch

1. **No hardware needed** - App starts in simulator mode
2. **Play your first chord**: Hold `1` key + press `Space`
3. **Browse songs**: Check [Song Library](assets/songs/README.md) for included songs
4. **Change genres**: Try different chord patterns via menu or buttons

> 📖 **New to mITyGuitar?** See [QUICKSTART.md](QUICKSTART.md) for a 5-minute guide

## Song Library

mITyGuitar includes a growing library of songs in `.mitychart.json` format:

![Song Library Interface](docs/images/song-library.png)
*Song library with included songs and upload functionality*

### Playing Songs
- **Upload**: Click "⬆ Upload Song" in Song Play view
- **Browse**: Click "📁 Song Library" to see available songs
- **Load**: Click "▶ Load" to start playing any song

### Included Songs
- **Greensleeves** - Traditional folk song
- **Simple Blues** - 12-bar blues progression
- **Djo - End Of Beginning** - Modern pop track

### Song Format Example
```json
{
  "meta": {
    "title": "Song Title",
    "artist": "Artist Name",
    "youtube": "https://youtube.com/watch?v=...",
    "spotify": "https://open.spotify.com/track/..."
  },
  "clock": {
    "bpm": 120,
    "timeSig": [4, 4],
    "subdivision": "8n"
  },
  "mapping": {
    "chords": {
      "C": { "frets": ["GREEN", "RED"] },
      "G": { "frets": ["YELLOW", "BLUE"] }
    }
  },
  "lanes": [
    {
      "name": "Main",
      "events": [
        { "startBeat": 0, "dur": 1, "chord": "C" },
        { "startBeat": 2, "dur": 1, "chord": "G" }
      ]
    }
  ],
  "lyrics": [
    { "startBeat": 0, "annotations": [
      { "word": "First", "timeBeat": "0.0" },
      { "word": "line", "timeBeat": "0.5" }
    ]}
  ],
  "sections": [
    { "name": "Intro", "fromBeat": 0, "toBeat": 8 }
  ]
}
```

> 📝 **Creating Songs**: See [Song Format Guide](docs/SONG_FORMAT.md) for detailed documentation

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

### Playing Songs

1. **Load a Song**: Song Play → 📁 Song Library → ▶ Load
2. **Follow the Chart**: Green dots show when to strum
3. **Watch Lyrics**: Follow word-by-word timing
4. **Hit the Chords**: Fret combinations light up before each chord

### Free Play Mode

1. **Hold Frets**: Press 1-5 (Green to Orange) or Q-T (Solo)
2. **Strum**: Space (down) or Arrow Up
3. **Change Genre**: Menu → Chords → Genre → Rock/Punk/EDM
4. **Switch Patterns**: Click "Next Pattern" button

### Controls

| Action | Keyboard | Controller |
|--------|----------|------------|
| Green Fret | 1, Q | Green button |
| Red Fret | 2, W | Red button |
| Yellow Fret | 3, E | Yellow button |
| Blue Fret | 4, R | Blue button |
| Orange Fret | 5, T | Orange button |
| Strum Down | Space, ↓ | Strum down |
| Strum Up | ↑ | Strum up |
| Whammy | - | Whammy bar |
| FX Switch | F1-F3 | FX toggle |

### Panic (Emergency Stop)

- **Menu**: File → Panic
- **Button**: Click "🛑 Panic" in interface

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

### ✅ Completed (Phase 1)
- [x] Core Rust crates (controller, mapping, audio, config)
- [x] Tauri v2 desktop application with React UI
- [x] SoundFont integration with 9 included SF2 files
- [x] 6 genre system (Punk, Rock, EDM, Pop, Folk, Metal)
- [x] Live visualization and diagnostics
- [x] Song playback system with `.mitychart.json` format
- [x] Chord mapping with sustain and whammy effects
- [x] Cross-platform builds (Windows/macOS/Linux)

### 🚧 In Progress (Phase 2)
- [ ] Hardware controller integration (HID)
- [ ] Controller mapping wizard
- [ ] Song editor/creator tool
- [ ] More SoundFont presets and effects

### 🔮 Planned (Phase 3)
- [ ] Online song sharing/library
- [ ] Recording and playback
- [ ] MIDI export
- [ ] Plugin architecture for custom effects

## Documentation

📖 **User Guides**
- [Quick Start](QUICKSTART.md) - Get running in 5 minutes
- [Song Format](docs/SONG_FORMAT.md) - Create custom songs
- [Controller Setup](docs/CONTROLLER_SETUP.md) - Hardware configuration

🔧 **Technical Documentation**
- [Architecture](docs/ARCHITECTURE.md) - System design overview
- [SoundFont Integration](docs/SOUNDFONT_INTEGRATION.md) - Audio system details
- [Chord Mapping](docs/CHORD_MAPPING.md) - Genre and pattern system
- [Build Instructions](docs/BUILD.md) - Development setup

## Contributing

Contributions welcome! Please:

1. Follow Rust conventions and add tests
2. Keep audio code RT-safe (no locks in audio callback)
3. Update documentation for user-facing changes
4. Test on multiple platforms when possible

## License

MIT License - see [LICENSE](LICENSE) for details

## Credits

- **SoundFonts**: Various open-source SF2 files
- **Built with**: Rust 🦀, Tauri, React, TypeScript
- **Audio**: cpal, oxisynth for cross-platform low-latency audio

---

**Made with ❤️ for guitar gamers who want to make real music!** 🎸🎮🎵
