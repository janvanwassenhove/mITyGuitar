use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicI32, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::Result;
use gilrs::{Gilrs, GamepadId, Button, Axis};
use crate::raw_diagnostics::RawDiagnostics;
use crate::mapping_wizard::MappingWizard;

/// High-performance atomic controller state for zero-latency access
/// All fields are atomic for lock-free access from multiple threads
#[derive(Debug, Default)]
pub struct AtomicControllerState {
    // Fret buttons (atomic booleans for instant access)
    pub fret_green: AtomicBool,
    pub fret_red: AtomicBool,  
    pub fret_blue: AtomicBool,
    pub fret_yellow: AtomicBool,
    pub fret_orange: AtomicBool,
    
    // Strum (atomic booleans)
    pub strum_up: AtomicBool,
    pub strum_down: AtomicBool,
    
    // D-pad (atomic booleans)  
    pub dpad_up: AtomicBool,
    pub dpad_down: AtomicBool,
    pub dpad_left: AtomicBool,
    pub dpad_right: AtomicBool,
    
    // Face buttons (atomic booleans)
    pub start: AtomicBool,
    pub select: AtomicBool,
    
    // Whammy bar (atomic i32 storing f32 bits)
    pub whammy_bar: AtomicI32,
    
    // Connection state
    pub connected: AtomicBool,
    
    // Last update timestamp (nanoseconds since epoch)
    pub last_update: AtomicU64,
}

impl AtomicControllerState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get whammy bar value as f32
    pub fn get_whammy(&self) -> f32 {
        f32::from_bits(self.whammy_bar.load(Ordering::Relaxed) as u32)
    }
    
    /// Set whammy bar value from f32
    pub fn set_whammy(&self, value: f32) {
        self.whammy_bar.store(value.to_bits() as i32, Ordering::Relaxed);
    }
    
    /// Update timestamp to current time
    pub fn update_timestamp(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        self.last_update.store(now, Ordering::Relaxed);
    }
}

/// Audio callback trait for instant sound triggering
pub trait AudioCallback: Send + Sync {
    /// Called immediately when a fret button is pressed
    fn on_fret_press(&self, fret: u8, velocity: f32);
    /// Called immediately when a fret button is released  
    fn on_fret_release(&self, fret: u8);
    /// Called immediately when strum occurs
    fn on_strum(&self, up: bool, velocity: f32);
    /// Called when whammy bar changes
    fn on_whammy_change(&self, value: f32);
}

/// High-performance controller with 1000Hz polling and direct audio callbacks
pub struct PerformanceController {
    state: Arc<AtomicControllerState>,
    audio_callback: Option<Arc<dyn AudioCallback>>,
    polling_thread: Option<thread::JoinHandle<()>>,
    should_stop: Arc<AtomicBool>,
    gilrs: Arc<std::sync::Mutex<Gilrs>>,
    active_gamepad: Arc<std::sync::Mutex<Option<GamepadId>>>, // Store GamepadId directly
    raw_diagnostics: Arc<RawDiagnostics>,
    mapping_wizard: Arc<MappingWizard>,
}

impl PerformanceController {
    /// Create new high-performance controller
    pub fn new() -> Result<Self> {
        log::info!("🎮 Initializing high-performance controller...");
        let gilrs = Gilrs::new().map_err(|e| anyhow::anyhow!("Failed to initialize gilrs: {}", e))?;
        log::info!("🎮 Gilrs initialized successfully");
        
        Ok(Self {
            state: Arc::new(AtomicControllerState::new()),
            audio_callback: None,
            polling_thread: None,
            should_stop: Arc::new(AtomicBool::new(false)),
            gilrs: Arc::new(std::sync::Mutex::new(gilrs)),
            active_gamepad: Arc::new(std::sync::Mutex::new(None)), // None = no gamepad
            raw_diagnostics: Arc::new(RawDiagnostics::new()),
            mapping_wizard: Arc::new(MappingWizard::new()),
        })
    }
    
    /// Get reference to raw diagnostics
    pub fn raw_diagnostics(&self) -> Arc<RawDiagnostics> {
        Arc::clone(&self.raw_diagnostics)
    }

    /// Get reference to mapping wizard
    pub fn mapping_wizard(&self) -> Arc<MappingWizard> {
        Arc::clone(&self.mapping_wizard)
    }
    
    /// Set audio callback for instant sound triggering
    pub fn set_audio_callback(&mut self, callback: Arc<dyn AudioCallback>) {
        self.audio_callback = Some(callback);
    }
    
    /// Start high-frequency polling thread (1000Hz = 1ms intervals)
    pub fn start_polling(&mut self) -> Result<()> {
        log::info!("🚀 Starting high-performance polling thread...");
        
        if self.polling_thread.is_some() {
            log::warn!("Polling thread already running");
            return Ok(()); // Already running
        }
        
        let state = Arc::clone(&self.state);
        let audio_callback = self.audio_callback.clone();
        let should_stop = Arc::clone(&self.should_stop);
        let gilrs = Arc::clone(&self.gilrs);
        let active_gamepad = Arc::clone(&self.active_gamepad);
        let raw_diagnostics = Arc::clone(&self.raw_diagnostics);
        let mapping_wizard = Arc::clone(&self.mapping_wizard);
        
        self.should_stop.store(false, Ordering::Relaxed);
        
        let thread = thread::spawn(move || {
            log::info!("🚀 High-performance polling thread started (1000Hz)");
            
            // Previous state for edge detection
            let mut prev_frets = [false; 5];  // green, red, yellow, blue, orange
            let mut prev_strum = [false; 2];  // up, down
            
            while !should_stop.load(Ordering::Relaxed) {
                let start_time = Instant::now();
                
                // Lock gilrs briefly to process events and poll
                {
                    let Ok(mut gilrs) = gilrs.try_lock() else {
                        thread::sleep(Duration::from_micros(100)); // 0.1ms if locked
                        continue;
                    };
                    
                    // Process connection events AND record for raw diagnostics
                    while let Some(event) = gilrs.next_event() {
                        // Get gamepad name for diagnostics
                        let gamepad_name = if let Some(gp) = gilrs.connected_gamepad(event.id) {
                            gp.name().to_string()
                        } else {
                            "Unknown".to_string()
                        };
                        
                        // Record raw event for diagnostics
                        raw_diagnostics.record_event(&event, &gamepad_name);
                        
                        // Create RawInputEvent for mapping wizard directly
                        let raw_event = crate::raw_diagnostics::RawInputEvent::from_gilrs_event(&event, &gamepad_name);
                        mapping_wizard.record_event(&raw_event);
                        
                        match event.event {
                            gilrs::EventType::Connected => {
                                let gamepad = gilrs.gamepad(event.id);
                                log::info!("🎮 Guitar connected: {} (ID: {:?})", gamepad.name(), event.id);
                                *active_gamepad.lock().unwrap() = Some(event.id);
                                state.connected.store(true, Ordering::Relaxed);
                            }
                            gilrs::EventType::Disconnected => {
                                log::info!("🎮 Guitar disconnected (ID: {:?})", event.id);
                                let mut guard = active_gamepad.lock().unwrap();
                                if *guard == Some(event.id) {
                                    *guard = None;
                                    state.connected.store(false, Ordering::Relaxed);
                                }
                            }
                            _ => {}
                        }
                    }
                    
                    // Poll active gamepad if connected (get gamepad_id once to avoid double lock)
                    let current_gamepad_id = {
                        let guard = active_gamepad.lock().unwrap();
                        *guard
                    };
                    
                    if let Some(gamepad_id) = current_gamepad_id {
                        let gamepad = gilrs.gamepad(gamepad_id);
                        
                        // Read all button states (fastest possible)
                        let frets = [
                            gamepad.is_pressed(Button::South),  // Green
                            gamepad.is_pressed(Button::East),   // Red  
                            gamepad.is_pressed(Button::North),  // Yellow (was West - swapped)
                            gamepad.is_pressed(Button::West),   // Blue (was North - swapped)
                            gamepad.is_pressed(Button::LeftTrigger) || gamepad.is_pressed(Button::LeftTrigger2), // Orange
                        ];
                        
                        // Check if we have a real strum bar (RightTrigger buttons)
                        let has_strum_bar = gamepad.is_pressed(Button::RightTrigger) || gamepad.is_pressed(Button::RightTrigger2);
                        
                        let strum = if has_strum_bar {
                            // Use RightTrigger buttons for strum if available
                            [
                                gamepad.is_pressed(Button::RightTrigger),
                                gamepad.is_pressed(Button::RightTrigger2),
                            ]
                        } else {
                            // Fall back to D-pad for strum if no RightTrigger
                            [
                                gamepad.is_pressed(Button::DPadUp),
                                gamepad.is_pressed(Button::DPadDown),
                            ]
                        };
                        
                        // D-pad is ONLY read if we're NOT using it for strum
                        let dpad = if has_strum_bar {
                            [
                                gamepad.is_pressed(Button::DPadUp),
                                gamepad.is_pressed(Button::DPadDown),
                            ]
                        } else {
                            [false, false] // Don't report d-pad if it's being used for strum
                        };
                        
                        // Update atomic state (lock-free)
                        state.fret_green.store(frets[0], Ordering::Relaxed);
                        state.fret_red.store(frets[1], Ordering::Relaxed);
                        state.fret_yellow.store(frets[2], Ordering::Relaxed);
                        state.fret_blue.store(frets[3], Ordering::Relaxed);
                        state.fret_orange.store(frets[4], Ordering::Relaxed);
                        
                        state.strum_up.store(strum[0], Ordering::Relaxed);
                        state.strum_down.store(strum[1], Ordering::Relaxed);
                        
                        state.dpad_up.store(dpad[0], Ordering::Relaxed);
                        state.dpad_down.store(dpad[1], Ordering::Relaxed);
                        
                        // D-pad and other controls
                        state.dpad_left.store(gamepad.is_pressed(Button::DPadLeft), Ordering::Relaxed);
                        state.dpad_right.store(gamepad.is_pressed(Button::DPadRight), Ordering::Relaxed);
                        state.start.store(gamepad.is_pressed(Button::Start), Ordering::Relaxed);
                        state.select.store(gamepad.is_pressed(Button::Select), Ordering::Relaxed);
                        
                        // Whammy bar
                        let whammy = gamepad.value(Axis::RightStickX);
                        state.set_whammy(whammy);
                        
                        // Update timestamp
                        state.update_timestamp();
                        
                        // Instant audio callbacks on button press edges (non-blocking)
                        if let Some(ref callback) = audio_callback {
                            // Detect fret button press/release edges
                            for (i, (&current, &previous)) in frets.iter().zip(prev_frets.iter()).enumerate() {
                                if current && !previous {
                                    // Button pressed - instant audio trigger (non-blocking)
                                    callback.on_fret_press(i as u8, 1.0);
                                } else if !current && previous {
                                    // Button released (non-blocking)
                                    callback.on_fret_release(i as u8);
                                }
                            }
                            
                            // Detect strum edges (non-blocking)
                            for (i, (&current, &previous)) in strum.iter().zip(prev_strum.iter()).enumerate() {
                                if current && !previous {
                                    // Strum - instant audio trigger (non-blocking)
                                    callback.on_strum(i == 0, 1.0); // true = up, false = down
                                }
                            }
                        }
                        
                        // Update previous state for next edge detection
                        prev_frets.copy_from_slice(&frets);
                        prev_strum.copy_from_slice(&strum);
                    }
                } // Release gilrs lock
                
                // Maintain 1000Hz (1ms) timing - sleep for remaining time
                let elapsed = start_time.elapsed();
                if elapsed < Duration::from_millis(1) {
                    thread::sleep(Duration::from_millis(1) - elapsed);
                }
            }
            
            log::info!("🛑 High-performance polling thread stopped");
        });
        
        self.polling_thread = Some(thread);
        Ok(())
    }
    
    /// Stop the polling thread
    pub fn stop_polling(&mut self) {
        self.should_stop.store(true, Ordering::Relaxed);
        
        if let Some(thread) = self.polling_thread.take() {
            let _ = thread.join();
        }
    }
    
    /// Get current controller state (lock-free read)
    pub fn get_state(&self) -> ControllerStateSnapshot {
        let state = &self.state;
        
        ControllerStateSnapshot {
            fret_green: state.fret_green.load(Ordering::Relaxed),
            fret_red: state.fret_red.load(Ordering::Relaxed),
            fret_blue: state.fret_blue.load(Ordering::Relaxed),
            fret_yellow: state.fret_yellow.load(Ordering::Relaxed),
            fret_orange: state.fret_orange.load(Ordering::Relaxed),
            strum_up: state.strum_up.load(Ordering::Relaxed),
            strum_down: state.strum_down.load(Ordering::Relaxed),
            dpad_up: state.dpad_up.load(Ordering::Relaxed),
            dpad_down: state.dpad_down.load(Ordering::Relaxed),
            dpad_left: state.dpad_left.load(Ordering::Relaxed),
            dpad_right: state.dpad_right.load(Ordering::Relaxed),
            start: state.start.load(Ordering::Relaxed),
            select: state.select.load(Ordering::Relaxed),
            whammy_bar: state.get_whammy(),
            connected: state.connected.load(Ordering::Relaxed),
            timestamp: state.last_update.load(Ordering::Relaxed),
        }
    }
    
    /// Force connection scan (non-blocking)
    pub fn scan_for_controllers(&self) -> Result<bool> {
        self.process_events()
    }
    
    /// Process events (compatibility method - same as scan_for_controllers)
    pub fn process_events(&self) -> Result<bool> {
        let Ok(mut gilrs) = self.gilrs.try_lock() else {
            return Ok(false); // Locked, try later
        };
        
        // Process any pending connection events
        while let Some(event) = gilrs.next_event() {
            match event.event {
                gilrs::EventType::Connected => {
                    let gamepad = gilrs.gamepad(event.id);
                    log::info!("🎮 Guitar found during scan: {} (ID: {:?})", gamepad.name(), event.id);
                    *self.active_gamepad.lock().unwrap() = Some(event.id);
                    self.state.connected.store(true, Ordering::Relaxed);
                    return Ok(true);
                }
                _ => {}
            }
        }
        
        // Check for existing gamepads
        for (id, gamepad) in gilrs.gamepads() {
            log::info!("🎮 Existing gamepad found: {} (ID: {:?})", gamepad.name(), id);
            *self.active_gamepad.lock().unwrap() = Some(id);
            self.state.connected.store(true, Ordering::Relaxed);
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Check if any device is connected (compatibility with old interface)
    pub fn find_device(&self) -> Result<bool> {
        Ok(self.state.connected.load(Ordering::Relaxed))
    }
    
    /// Get debug information (compatibility with old interface)
    pub fn get_debug_info(&self) -> String {
        let mut info = String::new();
        info.push_str("Type: PerformanceController\n");
        info.push_str(&format!("Connected: {}\n", self.state.connected.load(Ordering::Relaxed)));
        info.push_str(&format!("Polling active: {}\n", self.polling_thread.is_some()));
        info.push_str(&format!("Last update: {}\n", self.state.last_update.load(Ordering::Relaxed)));
        
        let gamepad_lock = self.active_gamepad.lock().unwrap();
        info.push_str(&format!("Active gamepad: {:?}\n", 
            gamepad_lock.as_ref().map_or("None".to_string(), |id| format!("{:?}", id))));
        info.push_str(&format!("Fret Green: {}\n", 
            self.state.fret_green.load(Ordering::Relaxed)));
        info.push_str(&format!("Fret Red: {}\n", 
            self.state.fret_red.load(Ordering::Relaxed)));
        info.push_str(&format!("Fret Yellow: {}\n", 
            self.state.fret_yellow.load(Ordering::Relaxed)));
        info.push_str(&format!("Fret Blue: {}\n", 
            self.state.fret_blue.load(Ordering::Relaxed)));
        info.push_str(&format!("Fret Orange: {}\n", 
            self.state.fret_orange.load(Ordering::Relaxed)));
        info.push_str(&format!("Strum Up: {}\n", 
            self.state.strum_up.load(Ordering::Relaxed)));
        info.push_str(&format!("Strum Down: {}\n", 
            self.state.strum_down.load(Ordering::Relaxed)));
        info.push_str(&format!("Start: {}\n", 
            self.state.start.load(Ordering::Relaxed)));
        info.push_str(&format!("Select: {}\n", 
            self.state.select.load(Ordering::Relaxed)));
        info.push_str(&format!("Whammy Bar: {}\n", 
            self.state.get_whammy()));
        info
    }
}

impl Drop for PerformanceController {
    fn drop(&mut self) {
        self.stop_polling();
    }
}

/// Snapshot of controller state for display/logic
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ControllerStateSnapshot {
    pub fret_green: bool,
    pub fret_red: bool,
    pub fret_blue: bool,
    pub fret_yellow: bool,
    pub fret_orange: bool,
    pub strum_up: bool,
    pub strum_down: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub start: bool,
    pub select: bool,
    pub whammy_bar: f32,
    pub connected: bool,
    pub timestamp: u64,
}