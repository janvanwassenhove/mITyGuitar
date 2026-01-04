use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use gilrs::{Gilrs, Button, Axis, GamepadId};
use serde::{Deserialize, Serialize};
use hidapi::HidApi;

#[cfg(feature = "simulator")]
pub mod simulator;

// New high-performance controller module
pub mod high_performance;
pub use high_performance::{PerformanceController, ControllerStateSnapshot, AtomicControllerState, AudioCallback};

// Raw diagnostics module
pub mod raw_diagnostics;
pub use raw_diagnostics::{RawDiagnostics, RawInputEvent};

// Mapping profile and wizard modules
pub mod mapping_profile;
pub use mapping_profile::{AppAction, RawBinding, ButtonBinding, AxisBinding, MappingProfile, MappingProfileManager, ControllerId};

pub mod mapping_wizard;
pub use mapping_wizard::{MappingWizard, CaptureState, CaptureResult, CapturedEventSummary};

// Known Rock Band / Guitar Hero controller VID/PID combinations
const GUITAR_DEVICES: &[(u16, u16)] = &[
    // Harmonix devices
    (0x1bad, 0x0004), // Harmonix Guitar for Nintendo Wii
    (0x1bad, 0x3010), // Harmonix Rock Band Guitar
    (0x1bad, 0x0002), // Harmonix Rock Band Guitar
    (0x1bad, 0x3110), // Harmonix Rock Band 3 Mustang Guitar
    // RedOctane / Activision devices  
    (0x1430, 0x4734), // RedOctane Guitar Hero 4
    (0x1430, 0x474b), // RedOctane Guitar Hero World Tour
    (0x12ba, 0x0100), // RedOctane Guitar Hero
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlId {
    FretGreen,
    FretRed,
    FretYellow,
    FretBlue,
    FretOrange,
    SoloGreen,
    SoloRed,
    SoloYellow,
    SoloBlue,
    SoloOrange,
    StrumUp,
    StrumDown,
    Start,
    Select,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    WhammyBar,
    TiltSensor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerState {
    pub buttons: HashMap<ControlId, bool>,
    pub axes: HashMap<ControlId, f32>,
    pub timestamp: f64,
}

impl Default for ControllerState {
    fn default() -> Self {
        let mut buttons = HashMap::new();
        // Main fret buttons
        buttons.insert(ControlId::FretGreen, false);
        buttons.insert(ControlId::FretRed, false);
        buttons.insert(ControlId::FretYellow, false);
        buttons.insert(ControlId::FretBlue, false);
        buttons.insert(ControlId::FretOrange, false);
        // Solo fret buttons
        buttons.insert(ControlId::SoloGreen, false);
        buttons.insert(ControlId::SoloRed, false);
        buttons.insert(ControlId::SoloYellow, false);
        buttons.insert(ControlId::SoloBlue, false);
        buttons.insert(ControlId::SoloOrange, false);
        // Other buttons
        buttons.insert(ControlId::StrumUp, false);
        buttons.insert(ControlId::StrumDown, false);
        buttons.insert(ControlId::Start, false);
        buttons.insert(ControlId::Select, false);
        buttons.insert(ControlId::DPadUp, false);
        buttons.insert(ControlId::DPadDown, false);
        buttons.insert(ControlId::DPadLeft, false);
        buttons.insert(ControlId::DPadRight, false);

        let mut axes = HashMap::new();
        axes.insert(ControlId::WhammyBar, 0.0);
        axes.insert(ControlId::TiltSensor, 0.0);

        Self {
            buttons,
            axes,
            timestamp: 0.0,
        }
    }
}

impl ControllerState {
    pub fn set_button(&mut self, control: ControlId, pressed: bool) {
        self.buttons.insert(control, pressed);
    }

    pub fn set_axis(&mut self, control: ControlId, value: f32) {
        self.axes.insert(control, value);
    }

    /// Get list of currently pressed fret buttons
    pub fn pressed_frets(&self) -> Vec<ControlId> {
        let fret_buttons = [
            ControlId::FretGreen,
            ControlId::FretRed,
            ControlId::FretYellow,
            ControlId::FretBlue,
            ControlId::FretOrange,
        ];
        
        fret_buttons
            .iter()
            .filter(|&&fret| self.buttons.get(&fret).copied().unwrap_or(false))
            .copied()
            .collect()
    }

    /// Check if strum bar is currently active (up or down)
    pub fn is_strumming(&self) -> bool {
        self.buttons.get(&ControlId::StrumUp).copied().unwrap_or(false)
            || self.buttons.get(&ControlId::StrumDown).copied().unwrap_or(false)
    }

    /// Get axis value (0.0 if not found)
    pub fn axis(&self, control: ControlId) -> f32 {
        self.axes.get(&control).copied().unwrap_or(0.0)
    }
}

/// Main controller manager using gilrs
#[derive(Clone)]
pub struct Controller {
    gilrs: Arc<Mutex<Gilrs>>,
    state: Arc<Mutex<ControllerState>>,
    start_time: Instant,
    active_gamepad: Arc<Mutex<Option<GamepadId>>>,
}

impl Controller {
    pub fn new() -> Result<Self> {
        let gilrs = Gilrs::new().map_err(|e| anyhow::anyhow!("Failed to initialize gilrs: {:?}", e))?;
        
        log::info!("🎮 Gilrs initialized, scanning for controllers...");
        
        Ok(Self {
            gilrs: Arc::new(Mutex::new(gilrs)),
            state: Arc::new(Mutex::new(ControllerState::default())),
            start_time: Instant::now(),
            active_gamepad: Arc::new(Mutex::new(None)),
        })
    }

    /// Process gilrs events (connection/disconnection) in a non-blocking way
    /// This should be called periodically but NOT every frame
    pub fn process_events(&self) -> Result<()> {
        let mut gilrs = self.gilrs.lock().unwrap();
        
        // Process all pending events (non-blocking)
        while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
            use gilrs::EventType;
            match event {
                EventType::Connected => {
                    let gamepad = gilrs.gamepad(id);
                    log::info!("🎮 Gamepad connected: {} (ID: {:?})", gamepad.name(), id);
                    
                    // Auto-connect to first guitar controller
                    if self.active_gamepad.lock().unwrap().is_none() {
                        let name = gamepad.name().to_lowercase();
                        if name.contains("guitar") || name.contains("rock band") || name.contains("hero") {
                            log::info!("✅ Auto-connecting to guitar controller");
                            *self.active_gamepad.lock().unwrap() = Some(id);
                        } else {
                            log::info!("💡 Auto-connecting to first gamepad (will map as guitar)");
                            *self.active_gamepad.lock().unwrap() = Some(id);
                        }
                    }
                }
                EventType::Disconnected => {
                    log::info!("🎮 Gamepad disconnected (ID: {:?})", id);
                    let mut active = self.active_gamepad.lock().unwrap();
                    if *active == Some(id) {
                        log::warn!("⚠️ Active guitar controller disconnected");
                        *active = None;
                    }
                }
                EventType::ButtonPressed(button, _) => {
                    log::debug!("🎮 Button pressed: {:?}", button);
                }
                EventType::ButtonReleased(button, _) => {
                    log::debug!("🎮 Button released: {:?}", button);
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    /// Check if a guitar controller is detected
    pub fn find_device(&self) -> Result<bool> {
        let gilrs = self.gilrs.lock().unwrap();
        
        // Check for gamepads (non-blocking)
        let mut found_any_gamepad = false;
        for (_id, gamepad) in gilrs.gamepads() {
            let name = gamepad.name();
            log::info!("🎮 Gilrs found gamepad: {}", name);
            found_any_gamepad = true;
            
            // Check if it's a guitar controller
            if name.to_lowercase().contains("guitar") 
                || name.to_lowercase().contains("rock band")
                || name.to_lowercase().contains("hero") {
                log::info!("✅ Guitar controller detected via gilrs: {}", name);
                return Ok(true);
            }
        }
        
        if found_any_gamepad {
            log::info!("⚠️ Found gamepad(s) but none identified as guitar controller");
            log::info!("💡 Will use first gamepad and map buttons as guitar");
            return Ok(true);
        }
        
        // Fallback: Check HID devices for known guitar VID/PIDs
        log::info!("🔍 No gamepads found via gilrs, checking HID devices...");
        match HidApi::new() {
            Ok(api) => {
                log::info!("📋 Scanning all HID devices:");
                for device_info in api.device_list() {
                    let vid = device_info.vendor_id();
                    let pid = device_info.product_id();
                    let name = device_info.product_string().unwrap_or("Unknown");
                    let manufacturer = device_info.manufacturer_string().unwrap_or("Unknown");
                    
                    // Log all HID devices for debugging
                    log::info!("  - VID:{:04x} PID:{:04x} {} ({})", vid, pid, name, manufacturer);
                    
                    // Check if it matches known guitar VID/PIDs
                    for &(known_vid, known_pid) in GUITAR_DEVICES {
                        if vid == known_vid && pid == known_pid {
                            log::info!("✅ Guitar controller detected via HID: {} (VID:{:04x} PID:{:04x})", name, vid, pid);
                            log::warn!("⚠️ Guitar detected but not recognized as gamepad by OS");
                            log::warn!("💡 You may need to install drivers or configure the device");
                            return Ok(true);
                        }
                    }
                }
                log::info!("📋 End of HID device scan");
            }
            Err(e) => {
                log::warn!("Failed to initialize HID API for fallback detection: {}", e);
            }
        }
        
        log::info!("ℹ️ No guitar controller detected");
        Ok(false)
    }

    /// Connect to the first available guitar controller
    pub fn connect(&self) -> Result<bool> {
        let gilrs = self.gilrs.lock().unwrap();
        
        // Try to find a guitar controller first
        for (id, gamepad) in gilrs.gamepads() {
            let name = gamepad.name();
            if name.to_lowercase().contains("guitar") 
                || name.to_lowercase().contains("rock band")
                || name.to_lowercase().contains("hero") {
                *self.active_gamepad.lock().unwrap() = Some(id);
                log::info!("✅ Connected to guitar controller: {}", name);
                return Ok(true);
            }
        }
        
        // If no guitar found, use first available gamepad
        if let Some((id, gamepad)) = gilrs.gamepads().next() {
            *self.active_gamepad.lock().unwrap() = Some(id);
            log::info!("✅ Connected to gamepad: {} (treating as guitar)", gamepad.name());
            return Ok(true);
        }
        
        log::warn!("⚠️ No gamepad found to connect to");
        Ok(false)
    }

    /// Poll for controller events and update state
    pub fn poll(&self) -> Result<()> {
        // Use regular lock() - this was never the problem, next_event() was!
        let gilrs = self.gilrs.lock().unwrap();
        
        let active_id = *self.active_gamepad.lock().unwrap();
        
        if active_id.is_none() {
            return Ok(());
        }
        
        let active_id = active_id.unwrap();
        
        // Read gamepad state and copy values (don't borrow)
        let gamepad = gilrs.gamepad(active_id);
        
        use gilrs::Button;
        use gilrs::Axis;
        
        // Copy all button states
        let green = gamepad.is_pressed(Button::South);
        let red = gamepad.is_pressed(Button::East);
        let blue = gamepad.is_pressed(Button::West);
        let yellow = gamepad.is_pressed(Button::North);
        let orange = gamepad.is_pressed(Button::LeftTrigger) || gamepad.is_pressed(Button::LeftTrigger2);
        
        // Check if we have a real strum bar (RightTrigger buttons)
        let has_strum_bar = gamepad.is_pressed(Button::RightTrigger) || gamepad.is_pressed(Button::RightTrigger2);
        
        // Strum bar - use RightTrigger if available, otherwise fall back to D-pad
        let (strum_up, strum_down, dpad_up, dpad_down) = if has_strum_bar {
            // Use RightTrigger for strum, D-pad for d-pad
            (
                gamepad.is_pressed(Button::RightTrigger),
                gamepad.is_pressed(Button::RightTrigger2),
                gamepad.is_pressed(Button::DPadUp),
                gamepad.is_pressed(Button::DPadDown),
            )
        } else {
            // Use D-pad for strum, no separate d-pad
            (
                gamepad.is_pressed(Button::DPadUp),
                gamepad.is_pressed(Button::DPadDown),
                false,
                false,
            )
        };
        
        let dpad_left = gamepad.is_pressed(Button::DPadLeft);
        let dpad_right = gamepad.is_pressed(Button::DPadRight);
        let start = gamepad.is_pressed(Button::Start);
        let select = gamepad.is_pressed(Button::Select);
        
        // Copy axis value
        let whammy = gamepad.value(Axis::RightStickX);
        
        // Release gilrs lock before acquiring state lock
        drop(gilrs);
        
        // Use lock() for critical state updates to ensure they always succeed
        let mut state = self.state.lock().unwrap();
        state.set_button(ControlId::FretGreen, green);
        state.set_button(ControlId::FretRed, red);
        state.set_button(ControlId::FretBlue, blue);
        state.set_button(ControlId::FretYellow, yellow);
        state.set_button(ControlId::FretOrange, orange);
        state.set_button(ControlId::StrumUp, strum_up);
        state.set_button(ControlId::StrumDown, strum_down);
        state.set_button(ControlId::DPadUp, dpad_up);
        state.set_button(ControlId::DPadDown, dpad_down);
        state.set_button(ControlId::DPadLeft, dpad_left);
        state.set_button(ControlId::DPadRight, dpad_right);
        state.set_button(ControlId::Start, start);
        state.set_button(ControlId::Select, select);
        
        // Update axis directly while we have the state lock
        state.set_axis(ControlId::WhammyBar, whammy);
        drop(state);
        self.update_timestamp();
        
        Ok(())
    }

    fn handle_button_press(&self, button: Button) {
        // Log ALL button presses for debugging
        log::info!("🔘 Button pressed: {:?}", button);
        
        let mut state = self.state.lock().unwrap();
        
        match button {
            // Fret buttons - standard mapping for most Guitar Hero controllers
            Button::South => {
                state.set_button(ControlId::FretGreen, true);
                log::debug!("🟢 Green fret pressed");
            }
            Button::East => {
                state.set_button(ControlId::FretRed, true);
                log::debug!("🔴 Red fret pressed");
            }
            Button::West => {
                state.set_button(ControlId::FretBlue, true);
                log::debug!("🔵 Blue fret pressed");
            }
            Button::North => {
                state.set_button(ControlId::FretYellow, true);
                log::debug!("🟡 Yellow fret pressed");
            }
            Button::LeftTrigger | Button::LeftTrigger2 => {
                state.set_button(ControlId::FretOrange, true);
                log::debug!("🟠 Orange fret pressed");
            }
            
            // Strum bar - RightTrigger buttons (separate from D-pad)
            Button::RightTrigger => {
                state.set_button(ControlId::StrumUp, true);
                log::debug!("⬆️ Strum up (RightTrigger)");
            }
            Button::RightTrigger2 => {
                state.set_button(ControlId::StrumDown, true);
                log::debug!("⬇️ Strum down (RightTrigger2)");
            }
            
            // D-pad buttons - separate, or used for strum if no RightTrigger
            Button::DPadUp => {
                // Check if we should use this for strum or d-pad
                // (handled in poll method, here we just set strum for compatibility)
                state.set_button(ControlId::StrumUp, true);
                log::debug!("⬆️ D-pad up / Strum up");
            }
            Button::DPadDown => {
                state.set_button(ControlId::StrumDown, true);
                log::debug!("⬇️ D-pad down / Strum down");
            }
            Button::DPadLeft => {
                state.set_button(ControlId::DPadLeft, true);
                log::debug!("⬅️ D-pad left");
            }
            Button::DPadRight => {
                state.set_button(ControlId::DPadRight, true);
                log::debug!("➡️ D-pad right");
            }
            
            // Start/Select buttons
            Button::Start => {
                state.set_button(ControlId::Start, true);
                log::debug!("▶️ Start pressed");
            }
            Button::Select => {
                state.set_button(ControlId::Select, true);
                log::debug!("⏸️ Select pressed");
            }
            
            _ => {
                log::trace!("Unknown button pressed: {:?}", button);
            }
        }
    }

    fn handle_button_release(&self, button: Button) {
        let mut state = self.state.lock().unwrap();
        
        match button {
            Button::South => state.set_button(ControlId::FretGreen, false),
            Button::East => state.set_button(ControlId::FretRed, false),
            Button::West => state.set_button(ControlId::FretBlue, false),
            Button::North => state.set_button(ControlId::FretYellow, false),
            Button::LeftTrigger | Button::LeftTrigger2 => state.set_button(ControlId::FretOrange, false),
            Button::RightTrigger => state.set_button(ControlId::StrumUp, false),
            Button::RightTrigger2 => state.set_button(ControlId::StrumDown, false),
            Button::DPadUp => {
                state.set_button(ControlId::DPadUp, false);
                state.set_button(ControlId::StrumUp, false);
            }
            Button::DPadDown => {
                state.set_button(ControlId::DPadDown, false);
                state.set_button(ControlId::StrumDown, false);
            }
            Button::DPadLeft => state.set_button(ControlId::DPadLeft, false),
            Button::DPadRight => state.set_button(ControlId::DPadRight, false),
            Button::Start => state.set_button(ControlId::Start, false),
            Button::Select => state.set_button(ControlId::Select, false),
            _ => {}
        }
    }

    fn handle_axis_change(&self, axis: Axis, value: f32) {
        let mut state = self.state.lock().unwrap();
        
        match axis {
            // Whammy bar on RightStickX (user confirmed this is working)
            Axis::RightStickX => {
                let normalized = (value + 1.0) / 2.0;
                state.set_axis(ControlId::WhammyBar, normalized);
            }
            // Tilt sensor - try multiple axes (Y-axis stick movement or Z triggers)
            Axis::LeftZ | Axis::RightZ | Axis::RightStickY | Axis::LeftStickY | Axis::LeftStickX => {
                let normalized = (value + 1.0) / 2.0;
                state.set_axis(ControlId::TiltSensor, normalized);
            }
            // Ignore other axes silently to reduce log spam
            _ => {}
        }
    }

    fn update_timestamp(&self) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        self.state.lock().unwrap().timestamp = elapsed;
    }

    pub fn get_state(&self) -> ControllerState {
        // Use lock() to ensure reliable state access for frontend
        self.state.lock().unwrap().clone()
    }

    /// List all connected gamepads (for debugging)
    pub fn list_all_devices(&self) -> Vec<String> {
        let mut devices = Vec::new();
        
        // List gilrs gamepads (non-blocking)
        let Ok(gilrs) = self.gilrs.try_lock() else {
            devices.push("⚠️ Cannot scan gamepads: controller is busy".to_string());
            devices.push("Please try again in a moment.".to_string());
            devices.push("".to_string());
            
            // Still try to scan HID devices
            devices.push("=== HID Devices (Scanning anyway) ===".to_string());
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
                        
                        let is_guitar = GUITAR_DEVICES.iter().any(|&(known_vid, known_pid)| vid == known_vid && pid == known_pid);
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
                        devices.push(format!("=== Summary: {} HID devices, {} guitars ===", device_count, found_guitars));
                    }
                }
                Err(e) => {
                    devices.push(format!("  Error accessing HID: {}", e));
                }
            }
            return devices;
        };
        
        let gamepad_count = gilrs.gamepads().count();
        devices.push(format!("=== Gilrs Gamepads ({}) ===", gamepad_count));
        
        for (id, gamepad) in gilrs.gamepads() {
            devices.push(format!("[Gilrs {}] {} ({})", 
                id, 
                gamepad.name(),
                if gamepad.is_connected() { "connected" } else { "disconnected" }
            ));
        }
        
        if gamepad_count == 0 {
            devices.push("  (No gamepads detected by gilrs)".to_string());
        }
        
        drop(gilrs); // Release lock before HID API
        
        // List ALL HID devices for debugging
        devices.push("".to_string());
        devices.push("=== All HID Devices ===".to_string());
        
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
                    let is_guitar = GUITAR_DEVICES.iter().any(|&(known_vid, known_pid)| vid == known_vid && pid == known_pid);
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
                    devices.push(format!("=== Summary: {} HID devices total, {} guitars detected ===", device_count, found_guitars));
                }
            }
            Err(e) => {
                devices.push(format!("  Error accessing HID: {}", e));
            }
        }
        
        devices
    }

    /// Get debug info about controller state
    pub fn get_debug_info(&self) -> String {
        let state = self.state.lock().unwrap();
        let active = self.active_gamepad.lock().unwrap();
        
        let mut info = String::new();
        info.push_str(&format!("Active gamepad: {:?}\n", active));
        info.push_str(&format!("Timestamp: {:.2}\n", state.timestamp));
        info.push_str("Buttons:\n");
        for (control, pressed) in &state.buttons {
            if *pressed {
                info.push_str(&format!("  {:?}: pressed\n", control));
            }
        }
        info.push_str("Axes:\n");
        for (control, value) in &state.axes {
            if value.abs() > 0.01 {
                info.push_str(&format!("  {:?}: {:.2}\n", control, value));
            }
        }
        
        info
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            log::error!("Failed to create Controller: {}", e);
            // Create a minimal fallback controller
            Self {
                gilrs: Arc::new(Mutex::new(Gilrs::new().unwrap())),
                state: Arc::new(Mutex::new(ControllerState::default())),
                start_time: Instant::now(),
                active_gamepad: Arc::new(Mutex::new(None)),
            }
        })
    }
}
