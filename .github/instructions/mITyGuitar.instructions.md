# mITyGuitar — Copilot Instructions

## Product summary
mITyGuitar is a cross-platform desktop app (Windows/macOS/Linux) that connects to a Rock Band guitar controller (USB dongle via HID) and turns it into a **low-latency** live instrument:
- Full controller support (all buttons/axes)
- Live UI visualization of every input
- Instrument selection (SoundFont-first) + fallback synth
- Chord patterns linked to genres (punk/rock/edm)
- Proper menu bar + clear text buttons for common actions

## Repo assets you MUST use
- The repo contains a `./soundfont/` folder with SF2/SF files.
- The app must be able to load and play these SoundFonts by default in dev.
- For production builds, ensure default SoundFonts remain available (bundle/copy into app resources on build, then load from app resource dir).

## Controller controls (must be supported)
Buttons:
- FRET_GREEN, FRET_RED, FRET_YELLOW, FRET_BLUE, FRET_ORANGE
- SOLO_GREEN, SOLO_RED, SOLO_YELLOW, SOLO_BLUE, SOLO_ORANGE
- STRUM_UP, STRUM_DOWN
- FX_SWITCH_UP, FX_SWITCH_CENTER, FX_SWITCH_DOWN
- DPAD_UP, DPAD_DOWN, DPAD_LEFT, DPAD_RIGHT
- BTN_START, BTN_SELECT, BTN_SYSTEM

Analog:
- WHAMMY_AXIS
- TILT_AXIS
- ACCEL_X, ACCEL_Y, ACCEL_Z

## Non-negotiables
- **Low latency**: audio should feel instant.
- **RT-safe audio callback**: no allocations, no locks in the callback.
- **Menu + Text buttons**: the app must be fully usable without hidden gestures.
- **Config-driven**: mapping/chords/instruments via versioned JSON.

## Suggested workspace layout
- `crates/controller`
  - HID discovery via `hidapi`
  - Parse HID reports into `ControllerState`
  - Feature flag: `controller_simulator` for keyboard/mouse simulation
- `crates/mapping`
  - Chord engine and genre presets
  - Convert controller state + strum -> musical events
- `crates/audio`
  - `cpal` output stream
  - RT-safe event queue (SPSC ring buffer)
  - Synth backends:
    - Backend A: SoundFont playback (use provided SoundFonts)
    - Backend B: Built-in polyphonic synth fallback
  - Minimal FX chain (RT-safe): distortion + cab-sim EQ (biquads)
- `crates/config`
  - JSON schemas, defaults, versioning, migrations
- `apps/desktop`
  - Tauri v2 backend + React/TS/Vite UI
  - Menus + screens + wiring

## Architecture rules
### Threads
- Input thread:
  - Reads HID reports and updates `ControllerState`
  - Pushes small events to audio + UI queues
  - Must not block on UI
- Audio thread (cpal callback):
  - Pulls events from lock-free queue
  - Renders audio; updates synth/FX state
  - **No malloc, no mutex**
- UI:
  - Receives throttled state snapshots (~60–120Hz max)
  - Sends commands/config updates via Tauri

### Internal data model
- `ControlId` enum for all buttons/axes
- `ControllerState` snapshot:
  - button bitset + axis map (normalized -1..1 or 0..1)
  - timestamp
- `MusicEvent` (from mapping -> audio):
  - NoteOn/NoteOff, PitchBend, CC, PresetNext/Prev, PanicAllNotesOff, etc.

## UI requirements (must implement)
### Menu bar (required)
- File:
  - Quit
- Instruments:
  - Previous Instrument
  - Next Instrument
  - Choose SoundFont…
  - Rescan SoundFonts
  - Choose Preset/Program (where available)
- Chords:
  - Previous Pattern
  - Next Pattern
  - Genre: Punk / Rock / EDM
  - Edit Patterns…
- View:
  - Live View
  - Instruments
  - Mapping / Calibration
  - Diagnostics
- Help:
  - About
  - Troubleshooting

### Text buttons (required)
On main screens, provide visible buttons for:
- Prev Instrument / Next Instrument
- Prev Pattern / Next Pattern
- Panic (All Notes Off)
- Rescan SoundFonts (in Instruments screen)

### Live View visualization (required)
Show:
- 5 frets + 5 solo frets (lit when pressed)
- Strum Up/Down indicator
- Whammy axis slider
- FX switch position
- D-pad + Start/Select/System
- Tilt + accelerometer XYZ meters with numeric values

## SoundFont handling rules
- Development:
  - Default scan `./soundfont/` and populate instrument list.
- Production:
  - Bundle/copy default SoundFonts into application resources.
  - Load from `AppHandle::path().resource_dir()` (or equivalent) and support user-added SoundFonts from config dir.
- Provide:
  - “Rescan SoundFonts” action
  - File chooser to load additional SF2/SF
  - Recent SoundFonts list

## Chord mapping behavior
- Held fret combo + STRUM_UP/DOWN triggers chord playback.
- Provide at least 3 genre presets:
  - Punk: power chords + aggressive voicing
  - Rock: triads + sus chords
  - EDM: minor-first, 7ths/9ths options, expressive modulation
- Patterns must be selectable via menu + buttons and bindable via FX switch or D-pad.

## Audio engine requirements
- RT-safe event transport: SPSC ring buffer preferred.
- Voice manager: polyphony, stealing strategy, note off handling.
- WHAMMY_AXIS controls pitch bend or vibrato depth (configurable).
- TILT/ACCEL modulate filter cutoff/tremolo/etc (configurable).
- Implement minimal RT-safe DSP:
  - Distortion: soft clip (tanh/atan) with gain staging
  - Cab-sim EQ: biquad filters (low/high shelf + optional peaking)

## Diagnostics view (required)
Expose:
- Sample rate, buffer size
- Estimated latency (approx)
- Underrun count
- Current device (controller) status
- Current SoundFont + preset/program

## Config (JSON, versioned)
Store in app config dir:
- `controller_mapping.json`
- `chord_mode.json`
- `instrument_mode.json`
Include:
- schema version
- defaults
- migration scaffolding for future versions

## Testing
- Unit tests for chord engine mapping: (held frets + strum) => expected chord events
- Config round-trip tests

## Definition of done for the first vertical slice
- App launches, shows Live View visualization (simulator OK)
- Menu bar exists and works
- Text buttons exist and work
- Strumming triggers chord playback (fallback synth OK)
- SoundFonts from `./soundfont/` show up in Instruments screen (even if SF2 playback is implemented in the next iteration)
- Config loads/saves

## Style & quality
- Keep modules small and explicit
- Prefer deterministic behavior; avoid magic heuristics
- Logs behind debug flags; do not spam stdout by default
- Use lined icons in the UI (not multi-color)

## Application
when making changes, always validate against the full application to ensure end-to-end functionality.Ensure the applications builds, compiles and runs correctly after applying the instructions.