use anyhow::Result;
use audio::{AudioOutput, AudioStats};
#[cfg(feature = "soundfont")]
use audio::{SoundFontInfo, InstrumentInfo, SoundFontInstrumentType as InstrumentType, SoundFontManager};
use audio::synth::InstrumentType as SynthInstrumentType;
use config::AppConfig;
use controller::{PerformanceController, ControllerStateSnapshot, ControllerState, ControlId, MappingProfileManager};
use mapping::{LegacyGenre as Genre, Mapper, MusicEvent};
use std::sync::{Arc, Mutex};
use once_cell::sync::OnceCell;
#[cfg(feature = "soundfont")]
use std::path::PathBuf;

#[cfg(feature = "simulator")]
use controller::simulator::ControllerSimulator;

use crate::song_player::SongPlayer;

// Global audio output - initialized once at startup
static AUDIO: OnceCell<Mutex<AudioOutput>> = OnceCell::new();

/// Initialize the global audio output
pub fn init_audio(buffer_size: Option<u32>) -> Result<()> {
    let audio = AudioOutput::new(buffer_size)?;
    AUDIO.set(Mutex::new(audio))
        .map_err(|_| anyhow::anyhow!("Audio already initialized"))?;
    Ok(())
}

/// Get reference to the global audio output
fn with_audio<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut AudioOutput) -> Result<R>,
{
    let audio_mutex = AUDIO.get()
        .ok_or_else(|| anyhow::anyhow!("Audio not initialized"))?;
    let mut audio = audio_mutex.lock().unwrap();
    f(&mut *audio)
}

/// Shared application state
pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub mapper: Arc<Mutex<Mapper>>,
    pub controller: Arc<Mutex<PerformanceController>>, // New high-performance controller
    pub profile_manager: Arc<Mutex<MappingProfileManager>>,
    pub song_player: Arc<Mutex<SongPlayer>>,
    
    #[cfg(feature = "soundfont")]
    pub soundfont_manager: Arc<Mutex<SoundFontManager>>,
    
    #[cfg(feature = "simulator")]
    pub simulator: Arc<Mutex<ControllerSimulator>>,
    
    // Flag to track if hardware controller is responsive
    hw_controller_enabled: Arc<Mutex<bool>>,
    
    // Track previous button states for detecting button presses
    prev_dpad_left: Arc<Mutex<bool>>,
    prev_dpad_right: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        // Load configuration
        let config = AppConfig::load()?;
        log::info!("Config loaded: sample_rate={}, buffer_size={}", 
            config.audio.sample_rate, config.audio.buffer_size);
        
        // Initialize audio (global, not in state)
        init_audio(Some(config.audio.buffer_size))?;
        log::info!("Audio output initialized");
        
        // Initialize SoundFont manager
        #[cfg(feature = "soundfont")]
        let soundfont_manager = {
            // Current dir when running with cargo/tauri is the src-tauri folder
            // so we need to go up to the workspace root to find soundfont/
            let current_dir = std::env::current_dir().unwrap_or_default();
            log::info!("Current working directory: {:?}", current_dir);
            
            // Try multiple possible locations for soundfont directory
            let possible_paths = vec![
                PathBuf::from("soundfont"),                    // If CWD is workspace root
                PathBuf::from("../soundfont"),                 // If CWD is apps/desktop
                PathBuf::from("../../soundfont"),              // If CWD is apps/desktop/src-tauri
                PathBuf::from("../../../soundfont"),           // If CWD is somewhere deeper
            ];
            
            let soundfont_dir = possible_paths.iter()
                .find(|path| path.exists())
                .cloned()
                .unwrap_or_else(|| PathBuf::from("soundfont")); // Fallback to default
            
            log::info!("Looking for soundfont directory at: {:?}", soundfont_dir);
            
            let manager = SoundFontManager::new(&soundfont_dir)
                .unwrap_or_else(|e| {
                    log::warn!("Failed to initialize SoundFont manager: {}. Continuing without SoundFonts.", e);
                    // Create an empty manager by using a non-existent directory
                    SoundFontManager::new(&PathBuf::from("___nonexistent___")).unwrap()
                });
            
            log::info!("SoundFont manager initialized with {} soundfonts", manager.list().len());
            Arc::new(Mutex::new(manager))
        };
        
        // Create mapper with configured genre
        let genre = match config.mapping.genre.as_str() {
            "punk" => Genre::Punk,
            "rock" => Genre::Rock,
            "edm" => Genre::Edm,
            "metal" => Genre::Metal,
            "folk" => Genre::Folk,
            "pop" => Genre::Pop,
            _ => Genre::Rock,
        };
        let mut mapper = Mapper::new(genre);
        
        // Set pattern index from config
        for _ in 0..config.mapping.pattern_index {
            mapper.next_pattern();
        }
        
        #[cfg(feature = "simulator")]
        let simulator = ControllerSimulator::new();
        
// Initialize high-performance controller with instant audio callbacks
        let mut controller = PerformanceController::new()
            .unwrap_or_else(|e| {
                log::warn!("Failed to initialize high-performance controller: {}", e);
                // Return a default controller if needed - for now, let's panic to catch issues
                panic!("High-performance controller required for instant response");
            });

        // Set up instant audio callback for zero-latency sound triggering
        // TODO: Re-enable audio callbacks after fixing integration issues
        // let audio_callback = Arc::new(InstantAudioCallback::new());
        // controller.set_audio_callback(audio_callback);
        
        // Start high-frequency polling (1000Hz) for instant response
        controller.start_polling()
            .unwrap_or_else(|e| {
                log::error!("Failed to start high-performance polling: {}", e);
            });

        // Scan for existing controllers
        let _ = controller.scan_for_controllers();
        
        log::info!("✅ High-performance controller initialized (1000Hz polling)");
        
        #[cfg(feature = "simulator")]
        log::info!("⌨️  Keyboard input enabled (works alongside hardware guitar)");
        
        // Load soundfont from config on startup
        #[cfg(feature = "soundfont")]
        if let Some(ref soundfont_name) = config.soundfonts.current {
            log::info!("Loading configured soundfont: {}", soundfont_name);
            let manager = soundfont_manager.lock().unwrap();
            if let Some(soundfont) = manager.get_by_name(soundfont_name) {
                let path = soundfont.path.clone();
                drop(manager); // Release lock before calling into audio
                
                if let Err(e) = with_audio(|audio| audio.load_soundfont(path)) {
                    log::error!("Failed to load soundfont on startup: {}", e);
                } else {
                    log::info!("✅ Soundfont loaded on startup: {}", soundfont_name);
                }
            } else {
                log::warn!("Configured soundfont '{}' not found in directory", soundfont_name);
            }
        }
        
        // Apply release time multiplier from config
        let release_multiplier = config.audio.release_time_multiplier;
        if let Err(e) = with_audio(|audio| audio.set_release_multiplier(release_multiplier)) {
            log::error!("Failed to set release multiplier: {}", e);
        } else {
            log::info!("✅ Release time multiplier set to: {}", release_multiplier);
        }
        
        // Initialize profile manager
        let profiles_dir = std::env::current_dir()
            .unwrap_or_default()
            .join("mapping_profiles");
        let profile_manager = MappingProfileManager::new(profiles_dir.clone())
            .unwrap_or_else(|e| {
                log::warn!("Failed to initialize profile manager: {}. Using temp directory.", e);
                // Fallback to temp directory
                let temp_dir = std::env::temp_dir().join("mityguitar_profiles");
                MappingProfileManager::new(temp_dir).unwrap()
            });
        
        // Initialize song player with available instruments
        let available_instruments = vec![
            ("virtual".to_string(), "Basic Guitar".to_string()),
            #[cfg(feature = "soundfont")]
            ("soundfont".to_string(), "Clean Guitar".to_string()),
        ];
        let song_player = SongPlayer::new(available_instruments);
        
        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            mapper: Arc::new(Mutex::new(mapper)),
            controller: Arc::new(Mutex::new(controller)),
            profile_manager: Arc::new(Mutex::new(profile_manager)),
            song_player: Arc::new(Mutex::new(song_player)),
            #[cfg(feature = "soundfont")]
            soundfont_manager,
            #[cfg(feature = "simulator")]
            simulator: Arc::new(Mutex::new(simulator)),
            hw_controller_enabled: Arc::new(Mutex::new(true)), // Enabled by default, will work if available
            prev_dpad_left: Arc::new(Mutex::new(false)),
            prev_dpad_right: Arc::new(Mutex::new(false)),
        })
    }
    
    /// Get current controller state (INSTANT - just atomic reads!)
    pub fn get_controller_state(&self) -> ControllerStateSnapshot {
        // Hardware enabled check
        let hw_enabled = *self.hw_controller_enabled.lock().unwrap();

        if hw_enabled {
            // Get atomic state snapshot - this is INSTANT! No polling overhead.
            let controller = self.controller.lock().unwrap();
            controller.get_state() // This just reads atomics - microsecond access!
        } else {
            // Hardware disabled, return empty state  
            ControllerStateSnapshot {
                fret_green: false,
                fret_red: false,
                fret_blue: false,
                fret_yellow: false,
                fret_orange: false,
                strum_up: false,
                strum_down: false,
                dpad_up: false,
                dpad_down: false,
                dpad_left: false,
                dpad_right: false,
                start: false,
                select: false,
                whammy_bar: 0.0,
                connected: false,
                timestamp: 0,
            }
        }
    }
    
    pub fn process_controller_input(&self) -> Result<()> {
        let state = self.get_controller_state();
        
        // Check for d-pad button presses to switch instruments
        #[cfg(feature = "soundfont")]
        {
            let mut prev_left = self.prev_dpad_left.lock().unwrap();
            let mut prev_right = self.prev_dpad_right.lock().unwrap();
            
            // Detect d-pad left press (transition from false to true)
            if state.dpad_left && !*prev_left {
                log::info!("🎸 D-Pad Left: switching to previous instrument");
                if let Err(e) = self.prev_instrument_internal() {
                    log::warn!("Failed to switch to previous instrument: {}", e);
                }
            }
            
            // Detect d-pad right press (transition from false to true)
            if state.dpad_right && !*prev_right {
                log::info!("🎸 D-Pad Right: switching to next instrument");
                if let Err(e) = self.next_instrument_internal() {
                    log::warn!("Failed to switch to next instrument: {}", e);
                }
            }
            
            // Update previous states
            *prev_left = state.dpad_left;
            *prev_right = state.dpad_right;
        }
        
        // Convert ControllerStateSnapshot to old ControllerState format for mapper
        let old_state = controller_snapshot_to_state(&state);
        
        // Process through mapper
        let events = {
            let mut mapper = self.mapper.lock().unwrap();
            mapper.process(&old_state)
        };
        
        // Send events to audio (global)
        for event in events {
            send_audio_event(event)?;
        }
        
        Ok(())
    }
    
    pub fn get_audio_stats(&self) -> AudioStats {
        with_audio(|audio| Ok(audio.get_stats())).unwrap()
    }
    
    /// Check audio health and attempt reconnection if needed
    pub fn check_and_reconnect_audio(&self) -> Result<bool> {
        check_audio_health()
    }
    
    /// Set the release time multiplier
    pub fn set_release_multiplier(&self, multiplier: f32) -> Result<()> {
        // Update config
        let mut config = self.config.lock().unwrap();
        config.audio.release_time_multiplier = multiplier;
        let config_clone = config.clone();
        drop(config);
        
        // Save config
        if let Err(e) = config_clone.save() {
            log::warn!("Failed to save config after setting release multiplier: {}", e);
        }
        
        // Apply to audio engine
        with_audio(|audio| audio.set_release_multiplier(multiplier))
    }
    
    /// Enable or disable sustain mode
    pub fn set_sustain_enabled(&self, enabled: bool) -> Result<()> {
        with_audio(|audio| audio.set_sustain_enabled(enabled))
    }
    
    /// Set sustain release time in seconds
    pub fn set_sustain_release_time(&self, time_seconds: f32) -> Result<()> {
        with_audio(|audio| audio.set_sustain_release_time(time_seconds))
    }
    
    #[cfg(feature = "soundfont")]
    pub fn get_available_instruments(&self) -> Result<Vec<InstrumentInfo>, String> {
        let manager = self.soundfont_manager.lock().unwrap();
        Ok(manager.list_instruments().to_vec())
    }

    #[cfg(feature = "soundfont")]
    pub fn get_available_soundfonts(&self) -> Result<Vec<SoundFontInfo>, String> {
        let manager = self.soundfont_manager.lock().unwrap();
        Ok(manager.list().to_vec())
    }
    
    #[cfg(feature = "soundfont")]
    pub fn next_instrument_internal(&self) -> Result<(), String> {
        let manager = self.soundfont_manager.lock().unwrap();
        let instruments = manager.list_instruments();
        
        if instruments.is_empty() {
            return Err("No instruments available".to_string());
        }
        
        // Get current instrument name from config
        let config = self.config.lock().unwrap();
        let current_name = config.soundfonts.current.clone();
        drop(config);
        
        // Find next instrument
        let next_name = if let Some(current) = current_name {
            let current_idx = instruments.iter().position(|inst| inst.name == current);
            if let Some(idx) = current_idx {
                let next_idx = (idx + 1) % instruments.len();
                instruments[next_idx].name.clone()
            } else {
                instruments[0].name.clone()
            }
        } else {
            instruments[0].name.clone()
        };
        
        drop(manager);
        
        // Load the next instrument
        self.set_instrument(next_name)
    }
    
    #[cfg(feature = "soundfont")]
    pub fn prev_instrument_internal(&self) -> Result<(), String> {
        let manager = self.soundfont_manager.lock().unwrap();
        let instruments = manager.list_instruments();
        
        if instruments.is_empty() {
            return Err("No instruments available".to_string());
        }
        
        // Get current instrument name from config
        let config = self.config.lock().unwrap();
        let current_name = config.soundfonts.current.clone();
        drop(config);
        
        // Find previous instrument
        let prev_name = if let Some(current) = current_name {
            let current_idx = instruments.iter().position(|inst| inst.name == current);
            if let Some(idx) = current_idx {
                let prev_idx = if idx == 0 { instruments.len() - 1 } else { idx - 1 };
                instruments[prev_idx].name.clone()
            } else {
                instruments[0].name.clone()
            }
        } else {
            instruments[0].name.clone()
        };
        
        drop(manager);
        
        // Load the previous instrument
        self.set_instrument(prev_name)
    }

    /// Set instrument (handles both SoundFonts and Virtual instruments)
    #[cfg(feature = "soundfont")]
    pub fn set_instrument(&self, name: String) -> Result<(), String> {
        let (instrument_type, instrument_path, instrument_info) = {
            let manager = self.soundfont_manager.lock().unwrap();
            let instrument = manager.get_instrument_by_name(&name)
                .ok_or_else(|| format!("Instrument '{}' not found", name))?;
            
            (instrument.instrument_type.clone(), instrument.path.clone(), instrument.clone())
        }; // manager is dropped here automatically
        
        match instrument_type {
            InstrumentType::SoundFont => {
                // Handle SoundFont loading
                if let Some(path) = instrument_path {
                    // Update config
                    {
                        let mut config = self.config.lock().unwrap();
                        config.soundfonts.current = Some(name.clone());
                        let _ = config.save(); // Don't fail on save errors
                    }
                    
                    log::info!("Set soundfont to: {}", name);
                    
                    // Load the SoundFont
                    with_audio(|audio| audio.load_soundfont(path))
                        .map_err(|e| format!("Failed to load soundfont: {}", e))?;
                        
                    Ok(())
                } else {
                    Err("SoundFont instrument missing path".to_string())
                }
            },
            InstrumentType::Virtual => {
                // Update config to mark as virtual instrument
                {
                    let mut config = self.config.lock().unwrap();
                    config.soundfonts.current = Some(name.clone());
                    let _ = config.save(); // Don't fail on save errors
                }
                
                log::info!("Set virtual instrument to: {}", name);
                
                // Get the specific synth instrument type
                if let Some(synth_instrument) = instrument_info.get_synth_instrument_type() {
                    // Set the virtual instrument with specific type
                    with_audio(|audio| audio.set_virtual_instrument(synth_instrument))
                        .map_err(|e| format!("Failed to set virtual instrument: {}", e))?;
                } else {
                    // Fallback to generic synth
                    with_audio(|audio| audio.use_fallback_synth())
                        .map_err(|e| format!("Failed to switch to virtual instrument: {}", e))?;
                }
                    
                Ok(())
            }
        }
    }
    
    #[cfg(feature = "soundfont")]
    pub fn set_soundfont(&self, name: String) -> Result<(), String> {
        let manager = self.soundfont_manager.lock().unwrap();
        let soundfont = manager.get_by_name(&name)
            .ok_or_else(|| format!("SoundFont not found: {}", name))?;
        
        // Load the soundfont into the audio engine
        let path = soundfont.path.clone();
        drop(manager); // Release lock before calling into audio
        
        with_audio(|audio| audio.load_soundfont(path))
            .map_err(|e| format!("Failed to load soundfont: {}", e))?;
        
        // Update config
        let mut config = self.config.lock().unwrap();
        config.soundfonts.current = Some(name.clone());
        config.save().map_err(|e| e.to_string())?;
        
        log::info!("Set soundfont to: {}", name);
        Ok(())
    }
    
    pub fn rescan_soundfonts(&self, user_soundfonts_dir: Option<PathBuf>) -> Result<(), String> {
        // Try multiple possible locations for soundfont directory
        let possible_paths = vec![
            PathBuf::from("soundfont"),                    // If CWD is workspace root
            PathBuf::from("../soundfont"),                 // If CWD is apps/desktop
            PathBuf::from("../../soundfont"),              // If CWD is apps/desktop/src-tauri
            PathBuf::from("../../../soundfont"),           // If CWD is somewhere deeper
        ];
        
        let soundfont_dir = possible_paths.iter()
            .find(|path| path.exists())
            .cloned()
            .unwrap_or_else(|| PathBuf::from("soundfont")); // Fallback to default
        
        log::info!("Rescanning soundfonts in: {:?}", soundfont_dir);
        
        // Build additional directories list
        let additional_dirs = if let Some(user_dir) = user_soundfonts_dir {
            log::info!("Also scanning user directory: {:?}", user_dir);
            vec![user_dir]
        } else {
            Vec::new()
        };
        
        let new_manager = if additional_dirs.is_empty() {
            audio::SoundFontManager::new(&soundfont_dir)
        } else {
            audio::SoundFontManager::with_additional_dirs(&soundfont_dir, additional_dirs)
        }
        .map_err(|e| format!("Failed to scan soundfonts: {}", e))?;
        
        let count = new_manager.list().len();
        *self.soundfont_manager.lock().unwrap() = new_manager;
        
        log::info!("Rescanned soundfonts: found {} files", count);
        Ok(())
    }
}

/// Send an event to the global audio output
pub fn send_audio_event(event: MusicEvent) -> Result<()> {
    with_audio(|audio| audio.send_event(event))
}

/// Convert new ControllerStateSnapshot to old ControllerState format for mapper compatibility
fn controller_snapshot_to_state(snapshot: &ControllerStateSnapshot) -> ControllerState {
    let mut state = ControllerState::default();
    
    // Map buttons
    state.buttons.insert(ControlId::FretGreen, snapshot.fret_green);
    state.buttons.insert(ControlId::FretRed, snapshot.fret_red);
    state.buttons.insert(ControlId::FretBlue, snapshot.fret_blue);
    state.buttons.insert(ControlId::FretYellow, snapshot.fret_yellow);
    state.buttons.insert(ControlId::FretOrange, snapshot.fret_orange);
    state.buttons.insert(ControlId::StrumUp, snapshot.strum_up);
    state.buttons.insert(ControlId::StrumDown, snapshot.strum_down);
    // Skip dpad_up and dpad_down as they're not in the original ControlId enum
    state.buttons.insert(ControlId::DPadLeft, snapshot.dpad_left);
    state.buttons.insert(ControlId::DPadRight, snapshot.dpad_right);
    state.buttons.insert(ControlId::Start, snapshot.start);
    state.buttons.insert(ControlId::Select, snapshot.select);
    
    // Map axes
    state.axes.insert(ControlId::WhammyBar, snapshot.whammy_bar);
    
    state
}

/// Check audio health and reconnect if needed
pub fn check_audio_health() -> Result<bool> {
    with_audio(|audio| {
        if audio.has_stream_error() {
            log::warn!("Audio stream error detected, attempting reconnection...");
            audio.try_reconnect()?;
            Ok(true) // Reconnection was needed and successful
        } else {
            Ok(false) // No reconnection needed
        }
    })
}
