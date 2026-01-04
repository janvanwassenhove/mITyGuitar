# mITyGuitar Documentation

Welcome to the mITyGuitar documentation! This directory contains comprehensive guides for users and developers.

## 📖 User Documentation

### Quick Start
- **[QUICKSTART.md](../QUICKSTART.md)** - Get running in 5 minutes
- **[README.md](../README.md)** - Main project overview and features

### Creating Content
- **[SONG_FORMAT.md](SONG_FORMAT.md)** - Complete guide to creating `.mitychart.json` songs
- **[Song Examples](../assets/songs/)** - Included songs and format reference

### Hardware Setup
- **[CONTROLLER_SETUP.md](CONTROLLER_SETUP.md)** - Controller mapping wizard and hardware configuration

### Visual Resources
- **[Documentation Images](images/README.md)** - Screenshots, diagrams, and visual guides

## 🔧 Technical Documentation

### Development
- **[BUILD.md](BUILD.md)** - Build instructions for all platforms
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design and implementation overview

### Audio System
- **[SOUNDFONT_INTEGRATION.md](SOUNDFONT_INTEGRATION.md)** - SoundFont system and audio engine details
- **[CHORD_MAPPING.md](CHORD_MAPPING.md)** - Genre-based chord mapping implementation

## 🎯 Quick Reference

### File Formats
- **Songs**: `.mitychart.json` format with timing, chords, and lyrics
- **Config**: JSON configuration in platform-specific location
- **SoundFonts**: `.sf2` files in `soundfont/` directory

### Key Directories
```
guitar/
├── docs/                   # 📖 This documentation
├── apps/desktop/          # 🖥️ Tauri app (React + Rust)
├── crates/                # 🦀 Rust libraries
│   ├── controller/        # 🎮 Input handling
│   ├── mapping/           # 🎵 Chord engine
│   ├── audio/             # 🔊 Audio synthesis
│   └── config/            # ⚙️ Configuration
├── assets/songs/          # 🎶 Song library
└── soundfont/             # 🎸 Audio samples
```

### Architecture Overview
```
Guitar Controller → Input Processing → Chord Mapping → Audio Synthesis → Output
       ↓                    ↓              ↓              ↓
   HID/Simulator    Controller Crate   Mapping Crate   Audio Crate
```

## 🚀 Common Tasks

### I want to...
- **Play songs**: Load [included songs](../assets/songs/README.md) or create your own
- **Add custom songs**: Follow the [Song Format Guide](SONG_FORMAT.md)
- **Change sounds**: Switch SoundFonts via menu or add your own `.sf2` files
- **Connect hardware**: Use the [Controller Setup Guide](CONTROLLER_SETUP.md)
- **Contribute code**: Check [Build Instructions](BUILD.md) and [Architecture](ARCHITECTURE.md)

### Troubleshooting
- **No sound**: Check [audio troubleshooting](BUILD.md#troubleshooting) section
- **Performance issues**: See [latency tips](../README.md#troubleshooting) in main README
- **Build problems**: Check platform-specific [build requirements](BUILD.md#prerequisites)

## 📚 Additional Resources

### External Links
- [Tauri Documentation](https://tauri.app/) - Desktop app framework
- [Rust Book](https://doc.rust-lang.org/book/) - Learning Rust
- [SoundFont Spec](https://www.synthfont.com/sf2_format.php) - SF2 file format

### Community
- [GitHub Repository](https://github.com/janvanwassenhove/mITyGuitar)
- [Issues & Bug Reports](https://github.com/janvanwassenhove/mITyGuitar/issues)
- [Discussions](https://github.com/janvanwassenhove/mITyGuitar/discussions)

---

**Happy jamming!** 🎸🎵