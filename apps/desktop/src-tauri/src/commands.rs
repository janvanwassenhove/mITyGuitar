use crate::state::AppState;
use audio::AudioStats;
use config::AppConfig;
use controller::{
    ControllerStateSnapshot, RawInputEvent, 
    AppAction, MappingProfile, CaptureResult, CaptureState, ControllerId,
};
use mapping::{LegacyGenre as Genre};
use song::{SongChart, InstrumentRef};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tauri::{State, Manager};
use hidapi::HidApi;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreInfo {
    pub name: String,
    pub patterns: Vec<String>,
    pub current_pattern_index: usize,
}

/// Get current controller state (INSTANT atomic read!)
#[tauri::command]
pub fn get_controller_state(state: State<AppState>) -> ControllerStateSnapshot {
    // Get the current state first (INSTANT!)
    let controller_state = state.get_controller_state();
    
    // Process input for audio using the conversion function
    let _ = state.process_controller_input();
    
    controller_state
}

/// Simulator: handle key down
#[cfg(feature = "simulator")]
#[tauri::command]
pub fn simulator_key_down(key: String, state: State<AppState>) -> Result<(), String> {
    let mut sim = state.simulator.lock().unwrap();
    sim.key_down(&key);
    drop(sim);
    
    // Process input
    state.process_controller_input().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(feature = "simulator"))]
#[tauri::command]
pub fn simulator_key_down(_key: String, _state: State<AppState>) -> Result<(), String> {
    Err("Simulator not enabled".to_string())
}

/// Simulator: handle key up
#[cfg(feature = "simulator")]
#[tauri::command]
pub fn simulator_key_up(key: String, state: State<AppState>) -> Result<(), String> {
    let mut sim = state.simulator.lock().unwrap();
    sim.key_up(&key);
    drop(sim);
    
    // Process input to update mapper state
    state.process_controller_input().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(feature = "simulator"))]
#[tauri::command]
pub fn simulator_key_up(_key: String, _state: State<AppState>) -> Result<(), String> {
    Err("Simulator not enabled".to_string())
}

/// Set the current genre
#[tauri::command]
pub fn set_genre(genre_name: String, state: State<AppState>) -> Result<(), String> {
    let genre = match genre_name.to_lowercase().as_str() {
        "punk" => Genre::Punk,
        "rock" => Genre::Rock,
        "edm" => Genre::Edm,
        "metal" => Genre::Metal,
        "folk" => Genre::Folk,
        "pop" => Genre::Pop,
        _ => return Err("Invalid genre".to_string()),
    };
    
    let mut mapper = state.mapper.lock().unwrap();
    mapper.set_genre(genre);
    
    // Update config
    let mut config = state.config.lock().unwrap();
    config.mapping.genre = genre_name;
    config.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Next chord pattern
#[tauri::command]
pub fn next_pattern(state: State<AppState>) -> Result<(), String> {
    let mut mapper = state.mapper.lock().unwrap();
    mapper.next_pattern();
    
    // Update config
    let pattern_index = mapper.pattern_index();
    drop(mapper);
    
    let mut config = state.config.lock().unwrap();
    config.mapping.pattern_index = pattern_index;
    config.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Previous chord pattern
#[tauri::command]
pub fn prev_pattern(state: State<AppState>) -> Result<(), String> {
    let mut mapper = state.mapper.lock().unwrap();
    mapper.prev_pattern();
    
    // Update config
    let pattern_index = mapper.pattern_index();
    drop(mapper);
    
    let mut config = state.config.lock().unwrap();
    config.mapping.pattern_index = pattern_index;
    config.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Next instrument
#[cfg(feature = "soundfont")]
#[tauri::command]
pub fn next_instrument(state: State<AppState>) -> Result<(), String> {
    state.next_instrument_internal()
}

#[cfg(not(feature = "soundfont"))]
#[tauri::command]
pub fn next_instrument(_state: State<AppState>) -> Result<(), String> {
    Err("SoundFont feature not enabled".to_string())
}

/// Previous instrument
#[cfg(feature = "soundfont")]
#[tauri::command]
pub fn prev_instrument(state: State<AppState>) -> Result<(), String> {
    state.prev_instrument_internal()
}

#[cfg(not(feature = "soundfont"))]
#[tauri::command]
pub fn prev_instrument(_state: State<AppState>) -> Result<(), String> {
    Err("SoundFont feature not enabled".to_string())
}

/// Panic - stop all notes
#[tauri::command]
pub fn panic_all_notes_off(state: State<AppState>) -> Result<(), String> {
    let mut mapper = state.mapper.lock().unwrap();
    let events = mapper.panic();
    drop(mapper);
    
    for event in events {
        crate::state::send_audio_event(event).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// Quit the application
#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

/// Get audio statistics
#[tauri::command]
pub fn get_audio_stats(state: State<AppState>) -> AudioStats {
    state.get_audio_stats()
}

/// Get current configuration
#[tauri::command]
pub fn get_config(state: State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

/// Save configuration
#[tauri::command]
pub fn save_config(config: AppConfig, state: State<AppState>) -> Result<(), String> {
    let mut current_config = state.config.lock().unwrap();
    *current_config = config;
    current_config.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Get all available genres
#[tauri::command]
pub fn get_genres() -> Vec<String> {
    Genre::all().iter().map(|g| g.name().to_string()).collect()
}

/// Get current genre info with patterns
#[tauri::command]
pub fn get_current_genre_info(state: State<AppState>) -> GenreInfo {
    let mapper = state.mapper.lock().unwrap();
    let genre = mapper.genre();
    let patterns = genre.get_patterns();
    
    GenreInfo {
        name: genre.name().to_string(),
        patterns: patterns.iter().map(|p| p.name.clone()).collect(),
        current_pattern_index: mapper.pattern_index(),
    }
}

/// Get available instruments (both SoundFonts and Virtual)
#[cfg(feature = "soundfont")]
#[tauri::command]
pub fn get_available_instruments(state: State<AppState>) -> Result<JsonValue, String> {
    let instruments = state.get_available_instruments()?;
    log::info!("get_available_instruments: returning {} instruments", instruments.len());
    for inst in &instruments {
        log::debug!("  - {} ({:?})", inst.name, inst.instrument_type);
    }
    serde_json::to_value(instruments).map_err(|e| {
        log::error!("Failed to serialize instruments: {}", e);
        e.to_string()
    })
}

#[cfg(not(feature = "soundfont"))]
#[tauri::command]
pub fn get_available_instruments(_state: State<AppState>) -> Result<JsonValue, String> {
    Err("SoundFont feature not enabled".to_string())
}

/// Get available soundfonts (legacy compatibility)
#[cfg(feature = "soundfont")]
#[tauri::command]
pub fn get_available_soundfonts(state: State<AppState>) -> Result<JsonValue, String> {
    let soundfonts = state.get_available_soundfonts()?;
    log::info!("get_available_soundfonts: returning {} soundfonts", soundfonts.len());
    for sf in &soundfonts {
        log::debug!("  - {} ({})", sf.name, sf.path.display());
    }
    serde_json::to_value(soundfonts).map_err(|e| {
        log::error!("Failed to serialize soundfonts: {}", e);
        e.to_string()
    })
}

#[cfg(not(feature = "soundfont"))]
#[tauri::command]
pub fn get_available_soundfonts(_state: State<AppState>) -> Result<JsonValue, String> {
    Err("SoundFont feature not enabled".to_string())
}

/// Set the active soundfont
#[cfg(feature = "soundfont")]
#[tauri::command]
pub fn set_soundfont(name: String, state: State<AppState>) -> Result<(), String> {
    state.set_soundfont(name)
}

#[cfg(not(feature = "soundfont"))]
#[tauri::command]
pub fn set_soundfont(_name: String, _state: State<AppState>) -> Result<(), String> {
    Err("SoundFont feature not enabled".to_string())
}

/// Rescan the soundfont directory
#[cfg(feature = "soundfont")]
#[tauri::command]
pub fn rescan_soundfonts(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<(), String> {
    log::info!("rescan_soundfonts command called");
    
    // Get user's app data directory for persistent uploaded soundfonts
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    let user_soundfonts_dir = app_data_dir.join("soundfonts");
    
    let result = state.rescan_soundfonts(Some(user_soundfonts_dir));
    if let Err(ref e) = result {
        log::error!("rescan_soundfonts failed: {}", e);
    }
    result
}

#[cfg(not(feature = "soundfont"))]
#[tauri::command]
pub fn rescan_soundfonts(_state: State<AppState>) -> Result<(), String> {
    Err("SoundFont feature not enabled".to_string())
}

/// Upload and save a soundfont file to the app data directory
#[cfg(feature = "soundfont")]
#[tauri::command]
pub fn upload_soundfont(file_path: String, file_name: String, app_handle: tauri::AppHandle, state: State<AppState>) -> Result<String, String> {
    use std::fs;
    use std::path::PathBuf;
    
    log::info!("upload_soundfont called with file: {}, name: {}", file_path, file_name);
    
    // Validate file extension
    if !file_name.to_lowercase().ends_with(".sf2") {
        return Err("Only .sf2 files are supported".to_string());
    }
    
    // Get app data directory
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    // Create soundfonts subdirectory if it doesn't exist
    let soundfonts_dir = app_data_dir.join("soundfonts");
    fs::create_dir_all(&soundfonts_dir)
        .map_err(|e| format!("Failed to create soundfonts directory: {}", e))?;
    
    // Destination path
    let dest_path = soundfonts_dir.join(&file_name);
    
    // Check if file already exists
    if dest_path.exists() {
        return Err(format!("A soundfont with the name '{}' already exists", file_name));
    }
    
    // Copy the file
    fs::copy(&file_path, &dest_path)
        .map_err(|e| format!("Failed to copy soundfont file: {}", e))?;
    
    log::info!("Soundfont saved to: {:?}", dest_path);
    
    // Rescan soundfonts to include the new file
    state.rescan_soundfonts(Some(soundfonts_dir))?;
    
    Ok(format!("Soundfont '{}' uploaded successfully", file_name))
}

#[cfg(not(feature = "soundfont"))]
#[tauri::command]
pub fn upload_soundfont(_file_path: String, _file_name: String, _app_handle: tauri::AppHandle, _state: State<AppState>) -> Result<String, String> {
    Err("SoundFont feature not enabled".to_string())
}

/// Check if a Rock Band guitar controller is detected
#[tauri::command]
pub fn check_hardware_controller(state: State<AppState>) -> Result<String, String> {
    // First, process gilrs events to detect any newly connected controllers
    {
        let controller = state.controller.lock().unwrap();
        let _ = controller.process_events();
        drop(controller);
    }
    
    let mut devices = Vec::new();
    devices.push("=== Scanning for Guitar Controllers ===".to_string());
    devices.push("".to_string());
    
    // Check gilrs gamepads
    {
        let controller = state.controller.lock().unwrap();
        if controller.find_device().unwrap_or(false) {
            devices.push("✅ Gilrs detected gamepad(s):".to_string());
            // The find_device logs will show details
        } else {
            devices.push("⚠️ No gamepads detected by gilrs".to_string());
        }
        devices.push("".to_string());
        drop(controller);
    }
    
    // Known guitar VID/PIDs
    let guitar_devices: &[(u16, u16)] = &[
        (0x1bad, 0x0004), // Harmonix Guitar for Nintendo Wii
        (0x1bad, 0x3010), // Harmonix Rock Band Guitar
        (0x1bad, 0x0002), // Harmonix Rock Band Guitar
        (0x1bad, 0x3110), // Harmonix Rock Band 3 Mustang Guitar
        (0x1430, 0x4734), // RedOctane Guitar Hero 4
        (0x1430, 0x474b), // RedOctane Guitar Hero World Tour
        (0x12ba, 0x0100), // RedOctane Guitar Hero
    ];
    
    match HidApi::new() {
        Ok(api) => {
            let mut device_count = 0;
            let mut found_guitars = 0;
            
            for device_info in api.device_list() {
                device_count += 1;
                let vid = device_info.vendor_id();
                let pid = device_info.product_id();
                let name = device_info.product_string().unwrap_or("Unknown");
                let manufacturer = device_info.manufacturer_string().unwrap_or("Unknown");
                
                // Check if it's a known guitar
                let is_guitar = guitar_devices.iter().any(|&(known_vid, known_pid)| vid == known_vid && pid == known_pid);
                let marker = if is_guitar { " *** GUITAR ***" } else { "" };
                
                devices.push(format!("[HID] VID:{:04x} PID:{:04x} {} ({}){}",
                    vid, pid, name, manufacturer, marker));
                
                if is_guitar {
                    found_guitars += 1;
                }
            }
            
            if device_count == 0 {
                devices.push("  (No HID devices found)".to_string());
            } else {
                devices.push("".to_string());
                devices.push(format!("=== Summary: {} HID devices, {} guitars detected ===", device_count, found_guitars));
            }
        }
        Err(e) => {
            devices.push(format!("Error accessing HID devices: {}", e));
        }
    }
    
    Ok(devices.join("\n"))
}

/// Get controller debug information
#[tauri::command]
pub fn get_controller_debug_info(state: State<AppState>) -> Result<String, String> {
    let controller = state.controller.lock().unwrap();
    Ok(controller.get_debug_info())
}

// ========== NEW CHORD MAPPING COMMANDS ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordMapResponse {
    pub main: HashMap<String, String>,
    pub solo: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordMappingSettings {
    pub genre: String,
    pub key_root: String,
    pub mode: String,
    pub sustain_enabled: bool,
    pub sustain_release_time_ms: f32,
    pub whammy_enabled: bool,
    pub whammy_pitch_bend_range: f32,
    pub whammy_vibrato_depth: f32,
    pub whammy_filter_cutoff_enabled: bool,
}

/// Get current chord mapping for main and solo frets
#[tauri::command]
pub fn get_chord_mapping(
    genre: String, 
    key_root: String, 
    mode: String, 
    _state: State<AppState>
) -> Result<ChordMapResponse, String> {
    // TODO: Implement using new chord resolver
    // For now, return default mappings based on genre
    
    let (main, solo) = match genre.to_lowercase().as_str() {
        "punk" => (
            generate_punk_chords(&key_root),
            generate_punk_chords(&key_root) // Solo same as main but octave up
        ),
        "edm" => (
            generate_edm_chords(&key_root, &mode),
            generate_edm_chords(&key_root, &mode)
        ),
        "rock" => (
            generate_rock_chords(&key_root),
            generate_rock_chords(&key_root)
        ),
        "pop" => (
            generate_pop_chords(&key_root),
            generate_pop_chords(&key_root)
        ),
        "folk" => (
            generate_folk_chords(&key_root),
            generate_folk_chords(&key_root)
        ),
        "metal" => (
            generate_metal_chords(&key_root),
            generate_metal_chords(&key_root)
        ),
        _ => (
            generate_punk_chords(&key_root), // Default to punk
            generate_punk_chords(&key_root)
        ),
    };
    
    Ok(ChordMapResponse { main, solo })
}

/// Update chord override for a specific fret button
#[tauri::command]
pub fn update_chord_override(
    fret_button: String,
    row: String,
    chord_spec: String,
    _state: State<AppState>
) -> Result<(), String> {
    // TODO: Implement chord override storage
    log::info!("Chord override: {} {} -> {}", row, fret_button, chord_spec);
    Ok(())
}

/// Update chord mapping settings
#[tauri::command]
pub fn update_chord_mapping_settings(
    settings: ChordMappingSettings,
    state: State<AppState>
) -> Result<(), String> {
    log::info!("Updating chord mapping settings: {:?}", settings);
    
    // Update the mapper with new genre, key, and mode
    let mut mapper = state.mapper.lock().unwrap();
    
    // Update genre
    let genre = match settings.genre.to_lowercase().as_str() {
        "punk" => Genre::Punk,
        "rock" => Genre::Rock,
        "edm" => Genre::Edm,
        "metal" => Genre::Metal,
        "folk" => Genre::Folk,
        "pop" => Genre::Pop,
        _ => {
            log::warn!("Unknown genre '{}', keeping current", settings.genre);
            return Err(format!("Invalid genre: {}", settings.genre));
        }
    };
    mapper.set_genre(genre);
    
    // Update key root
    if let Some(key_note) = parse_note(&settings.key_root) {
        mapper.set_key_root(key_note as u8);
    } else {
        log::warn!("Invalid key root '{}', keeping current", settings.key_root);
        return Err(format!("Invalid key: {}", settings.key_root));
    }
    
    // Update mode
    let is_major = settings.mode.to_lowercase() == "major";
    mapper.set_mode(is_major);
    
    // Update config
    let mut config = state.config.lock().unwrap();
    config.mapping.genre = settings.genre.clone();
    
    log::info!("Chord mapping settings updated successfully");
    Ok(())
}

/// Get current app config including soundfont info
#[tauri::command]
pub fn get_app_config(state: State<AppState>) -> Result<JsonValue, String> {
    let config = state.config.lock().unwrap();
    let soundfont_current = config.soundfonts.current.clone();
    
    Ok(serde_json::json!({
        "soundfonts": {
            "current": soundfont_current
        }
    }))
}

// Helper functions for generating chord mappings

fn generate_punk_chords(key_root: &str) -> HashMap<String, String> {
    // Simple power chord mapping in key
    let root_note = parse_note(key_root).unwrap_or(0);
    let chords = [
        (0, "5"),   // I
        (5, "5"),   // IV  
        (7, "5"),   // V
        (10, "5"),  // bVII
        (9, "5"),   // VI
    ];
    
    let mut result = HashMap::new();
    let frets = ["green", "red", "yellow", "blue", "orange"];
    
    for (i, fret) in frets.iter().enumerate() {
        if i < chords.len() {
            let (interval, quality) = chords[i];
            let chord_root = (root_note + interval) % 12;
            result.insert(fret.to_string(), format!("{}{}", note_name(chord_root), quality));
        }
    }
    
    result
}

fn generate_edm_chords(key_root: &str, mode: &str) -> HashMap<String, String> {
    let root_note = parse_note(key_root).unwrap_or(0);
    let is_minor = mode.to_lowercase() == "minor";
    
    let chords = if is_minor {
        [
            (0, "m"),   // Im
            (5, "m"),   // IVm  
            (7, "sus2"), // V
            (10, ""),   // bVII
            (8, ""),    // VI
        ]
    } else {
        [
            (0, ""),    // I
            (5, ""),    // IV  
            (7, "sus2"), // V
            (10, ""),   // bVII
            (9, "m"),   // VIm
        ]
    };
    
    let mut result = HashMap::new();
    let frets = ["green", "red", "yellow", "blue", "orange"];
    
    for (i, fret) in frets.iter().enumerate() {
        if i < chords.len() {
            let (interval, quality) = chords[i];
            let chord_root = (root_note + interval) % 12;
            result.insert(fret.to_string(), format!("{}{}", note_name(chord_root), quality));
        }
    }
    
    result
}

fn generate_rock_chords(key_root: &str) -> HashMap<String, String> {
    let root_note = parse_note(key_root).unwrap_or(0);
    let chords = [
        (0, ""),    // I
        (5, ""),    // IV  
        (7, ""),    // V
        (10, ""),   // bVII
        (2, "m"),   // IIm
    ];
    
    let mut result = HashMap::new();
    let frets = ["green", "red", "yellow", "blue", "orange"];
    
    for (i, fret) in frets.iter().enumerate() {
        if i < chords.len() {
            let (interval, quality) = chords[i];
            let chord_root = (root_note + interval) % 12;
            result.insert(fret.to_string(), format!("{}{}", note_name(chord_root), quality));
        }
    }
    
    result
}

fn generate_pop_chords(key_root: &str) -> HashMap<String, String> {
    let root_note = parse_note(key_root).unwrap_or(0);
    let chords = [
        (0, "add9"),  // I
        (5, "add9"),  // IV  
        (7, "sus4"),  // V
        (10, ""),     // bVII
        (9, "m"),     // VIm
    ];
    
    let mut result = HashMap::new();
    let frets = ["green", "red", "yellow", "blue", "orange"];
    
    for (i, fret) in frets.iter().enumerate() {
        if i < chords.len() {
            let (interval, quality) = chords[i];
            let chord_root = (root_note + interval) % 12;
            result.insert(fret.to_string(), format!("{}{}", note_name(chord_root), quality));
        }
    }
    
    result
}

fn generate_folk_chords(key_root: &str) -> HashMap<String, String> {
    let root_note = parse_note(key_root).unwrap_or(0);
    let chords = [
        (0, ""),      // I
        (5, ""),      // IV  
        (7, "sus4"),  // V
        (10, ""),     // bVII
        (9, "m"),     // VIm
    ];
    
    let mut result = HashMap::new();
    let frets = ["green", "red", "yellow", "blue", "orange"];
    
    for (i, fret) in frets.iter().enumerate() {
        if i < chords.len() {
            let (interval, quality) = chords[i];
            let chord_root = (root_note + interval) % 12;
            result.insert(fret.to_string(), format!("{}{}", note_name(chord_root), quality));
        }
    }
    
    result
}

fn generate_metal_chords(key_root: &str) -> HashMap<String, String> {
    let root_note = parse_note(key_root).unwrap_or(0);
    let chords = [
        (0, "5"),   // I
        (5, "5"),   // IV  
        (7, "5"),   // V
        (10, "5"),  // bVII
        (2, "5"),   // II
    ];
    
    let mut result = HashMap::new();
    let frets = ["green", "red", "yellow", "blue", "orange"];
    
    for (i, fret) in frets.iter().enumerate() {
        if i < chords.len() {
            let (interval, quality) = chords[i];
            let chord_root = (root_note + interval) % 12;
            result.insert(fret.to_string(), format!("{}{}", note_name(chord_root), quality));
        }
    }
    
    result
}

fn parse_note(note: &str) -> Option<usize> {
    match note.to_uppercase().as_str() {
        "C" => Some(0),
        "C#" | "DB" => Some(1),
        "D" => Some(2), 
        "D#" | "EB" => Some(3),
        "E" => Some(4),
        "F" => Some(5),
        "F#" | "GB" => Some(6),
        "G" => Some(7),
        "G#" | "AB" => Some(8),
        "A" => Some(9),
        "A#" | "BB" => Some(10),
        "B" => Some(11),
        _ => None,
    }
}

fn note_name(note: usize) -> &'static str {
    match note % 12 {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "D#", 
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "G#",
        9 => "A",
        10 => "A#",
        11 => "B",
        _ => "C",
    }
}

/// Check for audio stream errors and attempt reconnection
#[tauri::command]
pub fn check_audio_health(state: State<AppState>) -> Result<bool, String> {
    state.check_and_reconnect_audio()
        .map_err(|e| e.to_string())
}

/// Set the release time multiplier for note fade-out
#[tauri::command]
pub fn set_release_multiplier(multiplier: f32, state: State<AppState>) -> Result<(), String> {
    state.set_release_multiplier(multiplier)
        .map_err(|e| e.to_string())
}

/// Enable or disable sustain mode
#[tauri::command]
pub fn set_sustain_enabled(enabled: bool, state: State<AppState>) -> Result<(), String> {
    state.set_sustain_enabled(enabled)
        .map_err(|e| e.to_string())
}

/// Set sustain release time in milliseconds
#[tauri::command]
pub fn set_sustain_release_time(time_ms: f32, state: State<AppState>) -> Result<(), String> {
    let time_seconds = time_ms / 1000.0;
    state.set_sustain_release_time(time_seconds)
        .map_err(|e| e.to_string())
}

// ============================================================================
// Raw Input Diagnostics Commands
// ============================================================================

/// Enable or disable raw input diagnostics
#[tauri::command]
pub fn set_raw_diagnostics_enabled(enabled: bool, state: State<AppState>) -> Result<(), String> {
    let controller = state.controller.lock().unwrap();
    let diagnostics = controller.raw_diagnostics();
    diagnostics.set_enabled(enabled);
    Ok(())
}

/// Get raw input diagnostics events
#[tauri::command]
pub fn get_raw_diagnostics(state: State<AppState>) -> Result<Vec<RawInputEvent>, String> {
    let controller = state.controller.lock().unwrap();
    let diagnostics = controller.raw_diagnostics();
    Ok(diagnostics.get_events())
}

/// Clear raw input diagnostics
#[tauri::command]
pub fn clear_raw_diagnostics(state: State<AppState>) -> Result<(), String> {
    let controller = state.controller.lock().unwrap();
    let diagnostics = controller.raw_diagnostics();
    diagnostics.clear();
    Ok(())
}

/// Get raw diagnostics status
#[tauri::command]
pub fn get_raw_diagnostics_status(state: State<AppState>) -> Result<(bool, usize), String> {
    let controller = state.controller.lock().unwrap();
    let diagnostics = controller.raw_diagnostics();
    Ok((diagnostics.is_enabled(), diagnostics.event_count()))
}

// Mapping Wizard Commands
// ============================================================================

/// Start capturing for a specific app action
#[tauri::command]
pub fn wizard_start_capture(action: String, state: State<AppState>) -> Result<(), String> {
    let controller = state.controller.lock().unwrap();
    let wizard = controller.mapping_wizard();
    
    // Parse action string to AppAction enum
    let app_action = serde_json::from_str::<AppAction>(&format!("\"{}\"", action))
        .map_err(|e| format!("Invalid action: {}", e))?;
    
    wizard.start_capture(app_action);
    Ok(())
}

/// Stop current capture
#[tauri::command]
pub fn wizard_stop_capture(state: State<AppState>) -> Result<(), String> {
    let controller = state.controller.lock().unwrap();
    let wizard = controller.mapping_wizard();
    wizard.stop_capture();
    Ok(())
}

/// Finalize capture and get result
#[tauri::command]
pub fn wizard_finalize_capture(state: State<AppState>) -> Result<CaptureResult, String> {
    let controller = state.controller.lock().unwrap();
    let wizard = controller.mapping_wizard();
    Ok(wizard.finalize_capture())
}

/// Get current wizard state
#[tauri::command]
pub fn wizard_get_state(state: State<AppState>) -> Result<String, String> {
    let controller = state.controller.lock().unwrap();
    let wizard = controller.mapping_wizard();
    let capture_state = wizard.get_state();
    // Serialize to JSON for frontend
    serde_json::to_string(&capture_state)
        .map_err(|e| format!("Failed to serialize state: {}", e))
}

/// Set auto-capture mode
#[tauri::command]
pub fn wizard_set_auto_capture(enabled: bool, state: State<AppState>) -> Result<(), String> {
    let controller = state.controller.lock().unwrap();
    let wizard = controller.mapping_wizard();
    wizard.set_auto_capture(enabled);
    Ok(())
}

/// Clear wizard state
#[tauri::command]
pub fn wizard_clear(state: State<AppState>) -> Result<(), String> {
    let controller = state.controller.lock().unwrap();
    let wizard = controller.mapping_wizard();
    wizard.clear_events();
    Ok(())
}

// Mapping Profile Commands
// ============================================================================

/// List all available mapping profiles
#[tauri::command]
pub fn list_mapping_profiles(state: State<AppState>) -> Result<Vec<String>, String> {
    let manager = state.profile_manager.lock().unwrap();
    manager.list_profiles()
        .map_err(|e| e.to_string())
}

/// Load a mapping profile by name
#[tauri::command]
pub fn load_mapping_profile(name: String, state: State<AppState>) -> Result<MappingProfile, String> {
    let mut manager = state.profile_manager.lock().unwrap();
    manager.load_profile(&name)
        .map_err(|e| e.to_string())?;
    // Return the loaded profile
    manager.active_profile()
        .cloned()
        .ok_or_else(|| "Profile loaded but not found".to_string())
}

/// Save a mapping profile
#[tauri::command]
pub fn save_mapping_profile(profile: MappingProfile, state: State<AppState>) -> Result<(), String> {
    let mut manager = state.profile_manager.lock().unwrap();
    manager.set_active_profile(profile);
    manager.save_active_profile()
        .map_err(|e| e.to_string())
}

/// Create a new mapping profile
#[tauri::command]
pub fn create_mapping_profile(name: String, controller_name: String, state: State<AppState>) -> Result<MappingProfile, String> {
    let manager = state.profile_manager.lock().unwrap();
    let controller_id = ControllerId {
        name: controller_name.clone(),
        label: Some(controller_name),
        vendor_id: None,
        product_id: None,
    };
    let mut profile = manager.create_default_profile(controller_id);
    profile.name = name;
    drop(manager);
    
    // Save the profile
    let mut manager = state.profile_manager.lock().unwrap();
    manager.set_active_profile(profile.clone());
    manager.save_active_profile()
        .map_err(|e| e.to_string())?;
    Ok(profile)
}

/// Delete a mapping profile
#[tauri::command]
pub fn delete_mapping_profile(name: String, state: State<AppState>) -> Result<(), String> {
    let manager = state.profile_manager.lock().unwrap();
    manager.delete_profile(&name)
        .map_err(|e| e.to_string())
}

/// Set the active mapping profile
#[tauri::command]
pub fn set_active_profile(name: String, state: State<AppState>) -> Result<(), String> {
    let mut manager = state.profile_manager.lock().unwrap();
    manager.load_profile(&name)
        .map_err(|e| e.to_string())
}

/// Get the currently active profile name
#[tauri::command]
pub fn get_active_profile(state: State<AppState>) -> Result<Option<String>, String> {
    let manager = state.profile_manager.lock().unwrap();
    Ok(manager.active_profile().map(|p| p.name.clone()))
}

/// Update a specific mapping in the active profile
#[tauri::command]
pub fn update_profile_mapping(action: String, binding: String, state: State<AppState>) -> Result<(), String> {
    let mut manager = state.profile_manager.lock().unwrap();
    
    // Parse action and binding
    let app_action = serde_json::from_str::<AppAction>(&format!("\"{}\"", action))
        .map_err(|e| format!("Invalid action: {}", e))?;
    let raw_binding = serde_json::from_str(&binding)
        .map_err(|e| format!("Invalid binding: {}", e))?;
    
    // Get active profile, modify it, and save
    if let Some(profile) = manager.active_profile_mut() {
        profile.mappings.insert(app_action, raw_binding);
        manager.save_active_profile()
            .map_err(|e| e.to_string())
    } else {
        Err("No active profile".to_string())
    }
}

// ============================================================================
// Song Play Commands
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongChartData {
    pub meta: serde_json::Value,
    pub clock: serde_json::Value,
    pub playback: serde_json::Value,
    pub mapping: serde_json::Value,
    pub lanes: serde_json::Value,
    pub lyrics: serde_json::Value,
    pub sections: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportState {
    pub is_playing: bool,
    pub current_beat: f64,
    pub bpm: f64,
    pub time_sig: [u32; 2],
    pub speed_multiplier: f64,
    pub is_in_count_in: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreData {
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub hits: u32,
    pub misses: u32,
    pub accuracy: f64,
    pub grade: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitResultData {
    pub is_hit: bool,
    pub chord: Option<String>,
    pub accuracy: Option<f64>,
    pub miss_reason: Option<String>,
}

/// Load a song chart from JSON string
#[tauri::command]
pub fn song_load_chart(json: String, state: State<AppState>) -> Result<(), String> {
    let mut player = state.song_player.lock().unwrap();
    player.load_chart(&json).map_err(|e| e.to_string())
}

/// Load the default Greensleeves chart
#[tauri::command]
pub fn song_load_default_chart(state: State<AppState>) -> Result<(), String> {
    let json = include_str!("../../../../assets/songs/greensleeves.mitychart.json");
    let mut player = state.song_player.lock().unwrap();
    player.load_chart(json).map_err(|e| e.to_string())
}

/// Load a song chart from a path in the assets directory
#[tauri::command]
pub fn song_load_chart_from_path(path: String, state: State<AppState>) -> Result<(), String> {
    // Map common asset paths
    let json = match path.as_str() {
        "assets/songs/simple-blues.mitychart.json" => {
            include_str!("../../../../assets/songs/simple-blues.mitychart.json")
        }
        "assets/songs/greensleeves.mitychart.json" => {
            include_str!("../../../../assets/songs/greensleeves.mitychart.json")
        }
        _ => return Err(format!("Unknown asset path: {}", path)),
    };
    
    let mut player = state.song_player.lock().unwrap();
    player.load_chart(json).map_err(|e| e.to_string())
}

/// Get current chart data
#[tauri::command]
pub fn song_get_chart(state: State<AppState>) -> Result<Option<String>, String> {
    let player = state.song_player.lock().unwrap();
    if let Some(chart) = player.get_chart() {
        serde_json::to_string(chart).map(Some).map_err(|e| e.to_string())
    } else {
        Ok(None)
    }
}

/// Play/resume song
#[tauri::command]
pub fn song_play(state: State<AppState>) -> Result<(), String> {
    let mut player = state.song_player.lock().unwrap();
    player.play();
    Ok(())
}

/// Pause song
#[tauri::command]
pub fn song_pause(state: State<AppState>) -> Result<(), String> {
    let mut player = state.song_player.lock().unwrap();
    player.pause();
    Ok(())
}

/// Stop song and reset
#[tauri::command]
pub fn song_stop(state: State<AppState>) -> Result<(), String> {
    let mut player = state.song_player.lock().unwrap();
    player.stop();
    Ok(())
}

/// Seek to beat
#[tauri::command]
pub fn song_seek(beat: f64, state: State<AppState>) -> Result<(), String> {
    let mut player = state.song_player.lock().unwrap();
    player.seek(beat);
    Ok(())
}

/// Set playback speed
#[tauri::command]
pub fn song_set_speed(multiplier: f64, state: State<AppState>) -> Result<(), String> {
    let mut player = state.song_player.lock().unwrap();
    player.set_speed(multiplier);
    Ok(())
}

/// Get transport state
#[tauri::command]
pub fn song_get_transport_state(state: State<AppState>) -> Result<TransportState, String> {
    let mut player = state.song_player.lock().unwrap();
    let current_beat = player.get_current_beat();
    let transport = player.get_transport_state();
    Ok(TransportState {
        is_playing: transport.is_playing,
        current_beat,
        bpm: transport.bpm,
        time_sig: transport.time_sig,
        speed_multiplier: transport.speed_multiplier,
        is_in_count_in: transport.is_in_count_in(),
    })
}

/// Check strum for hit detection
#[tauri::command]
pub fn song_check_strum(pressed_frets: Vec<String>, state: State<AppState>) -> Result<HitResultData, String> {
    let mut player = state.song_player.lock().unwrap();
    
    if let Some(result) = player.check_strum(pressed_frets) {
        match result {
            song::HitResult::Hit { event, accuracy } => {
                Ok(HitResultData {
                    is_hit: true,
                    chord: Some(event.chord),
                    accuracy: Some(accuracy),
                    miss_reason: None,
                })
            }
            song::HitResult::Miss { reason } => {
                let reason_str = match reason {
                    song::MissReason::NoEventInWindow => "no_event",
                    song::MissReason::WrongFrets => "wrong_frets",
                    song::MissReason::AlreadyHit => "already_hit",
                };
                Ok(HitResultData {
                    is_hit: false,
                    chord: None,
                    accuracy: None,
                    miss_reason: Some(reason_str.to_string()),
                })
            }
        }
    } else {
        Err("No chart loaded".to_string())
    }
}

/// Update sustain state
#[tauri::command]
pub fn song_update_sustain(pressed_frets: Vec<String>, state: State<AppState>) -> Result<bool, String> {
    let mut player = state.song_player.lock().unwrap();
    Ok(player.update_sustain(pressed_frets))
}

/// Get current score
#[tauri::command]
pub fn song_get_score(state: State<AppState>) -> Result<ScoreData, String> {
    let player = state.song_player.lock().unwrap();
    let scorer = player.get_score();
    Ok(ScoreData {
        score: scorer.score,
        combo: scorer.combo,
        max_combo: scorer.max_combo,
        hits: scorer.hits,
        misses: scorer.misses,
        accuracy: scorer.get_accuracy(),
        grade: scorer.get_grade().to_string(),
    })
}

/// Set user override instrument
#[tauri::command]
pub fn song_set_instrument(instrument_type: String, label: String, state: State<AppState>) -> Result<(), String> {
    let mut player = state.song_player.lock().unwrap();
    player.set_user_instrument(Some(InstrumentRef {
        instrument_type,
        label,
    }));
    Ok(())
}

/// Clear user override instrument
#[tauri::command]
pub fn song_clear_instrument_override(state: State<AppState>) -> Result<(), String> {
    let mut player = state.song_player.lock().unwrap();
    player.set_user_instrument(None);
    Ok(())
}

// ============================================================================
// Song Library Management
// ============================================================================

use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongLibraryEntry {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub filename: String,
}

fn get_songs_directory() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe_path.parent().ok_or("Failed to get exe directory")?;
    
    // In development: use workspace root
    // In production: use app data directory
    let songs_dir = if cfg!(debug_assertions) {
        // Development: use workspace assets/songs
        exe_dir.join("../../../../assets/songs")
    } else {
        // Production: use app data
        let app_data = dirs::data_dir().ok_or("Failed to get app data directory")?;
        app_data.join("mityguitar").join("songs")
    };
    
    // Create directory if it doesn't exist
    fs::create_dir_all(&songs_dir).map_err(|e| e.to_string())?;
    
    // Canonicalize to get absolute path
    songs_dir.canonicalize().map_err(|e| e.to_string())
}

/// Save a song to the library
#[tauri::command]
pub fn song_save_to_library(json: String, filename: String) -> Result<String, String> {
    // Validate JSON first
    let chart: SongChart = serde_json::from_str(&json).map_err(|e| format!("Invalid song JSON: {}", e))?;
    
    let songs_dir = get_songs_directory()?;
    
    // Ensure filename has .mitychart.json extension
    let filename = if filename.ends_with(".mitychart.json") {
        filename
    } else if filename.ends_with(".json") {
        filename.replace(".json", ".mitychart.json")
    } else {
        format!("{}.mitychart.json", filename)
    };
    
    let file_path = songs_dir.join(&filename);
    
    // Pretty print the JSON
    let pretty_json = serde_json::to_string_pretty(&chart).map_err(|e| e.to_string())?;
    fs::write(&file_path, pretty_json).map_err(|e| format!("Failed to save song: {}", e))?;
    
    Ok(filename)
}

/// List all songs in the library
#[tauri::command]
pub fn song_list_library() -> Result<Vec<SongLibraryEntry>, String> {
    let songs_dir = get_songs_directory()?;
    
    let mut entries = Vec::new();
    
    // Read directory
    let dir_entries = fs::read_dir(&songs_dir).map_err(|e| format!("Failed to read songs directory: {}", e))?;
    
    for entry in dir_entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        // Only process .mitychart.json files
        if !path.is_file() || !path.to_string_lossy().ends_with(".mitychart.json") {
            continue;
        }
        
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        // Try to read and parse the file to get metadata
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(chart) = serde_json::from_str::<SongChart>(&content) {
                entries.push(SongLibraryEntry {
                    id: filename.clone(),
                    title: chart.meta.title.clone(),
                    artist: chart.meta.artist.clone(),
                    filename,
                });
            }
        }
    }
    
    // Sort by title
    entries.sort_by(|a, b| a.title.cmp(&b.title));
    
    Ok(entries)
}

/// Load a song from the library
#[tauri::command]
pub fn song_load_from_library(filename: String, state: State<AppState>) -> Result<(), String> {
    let songs_dir = get_songs_directory()?;
    let file_path = songs_dir.join(&filename);
    
    if !file_path.exists() {
        return Err(format!("Song file not found: {}", filename));
    }
    
    let json = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read song file: {}", e))?;
    
    let mut player = state.song_player.lock().unwrap();
    player.load_chart(&json).map_err(|e| e.to_string())
}

/// Delete a song from the library
#[tauri::command]
pub fn song_delete_from_library(filename: String) -> Result<(), String> {
    let songs_dir = get_songs_directory()?;
    let file_path = songs_dir.join(&filename);
    
    if !file_path.exists() {
        return Err(format!("Song file not found: {}", filename));
    }
    
    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete song: {}", e))
}
