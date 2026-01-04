use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use gilrs::{Gilrs, Event, EventType, Button, Axis, GamepadId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlId {
    FretGreen,
    FretRed,
    FretYellow,
    FretBlue,
    FretOrange,
    StrumUp,
    StrumDown,
    Start,
    Select,
    WhammyBar,
    TiltSensor,
}

#[derive(Debug, Clone)]
pub struct ControllerState {
    pub buttons: HashMap<ControlId, bool>,
    pub axes: HashMap<ControlId, f32>,
    pub timestamp: f64,
}

impl Default for ControllerState {
    fn default() -> Self {
        let mut buttons = HashMap::new();
        buttons.insert(ControlId::FretGreen, false);
        buttons.insert(ControlId::FretRed, false);
        buttons.insert(ControlId::FretYellow, false);
        buttons.insert(ControlId::FretBlue, false);
        buttons.insert(ControlId::FretOrange, false);
        buttons.insert(ControlId::StrumUp, false);
        buttons.insert(ControlId::StrumDown, false);
        buttons.insert(ControlId::Start, false);
        buttons.insert(ControlId::Select, false);

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
}

/// Main controller manager using gilrs
pub struct Controller {
    gilrs: Arc<Mutex<Gilrs>>,
    state: Arc<Mutex<ControllerState>>,
    start_time: Instant,
    active_gamepad: Arc<Mutex<Option<GamepadId>>>,
}

impl Controller {
    pub fn new() -> Result<Self> {
        let gilrs = Gilrs::new().context("Failed to initialize gilrs")?;
        
        log::info!("🎮 Gilrs initialized, scanning for controllers...");
        
        Ok(Self {
            gilrs: Arc::new(Mutex::new(gilrs)),
            state: Arc::new(Mutex::new(ControllerState::default())),
            start_time: Instant::now(),
            active_gamepad: Arc::new(Mutex::new(None)),
        })
    }

    /// Check if a guitar controller is detected
    pub fn find_device(&self) -> Result<bool> {
        let gilrs = self.gilrs.lock().unwrap();
        
        for (_id, gamepad) in gilrs.gamepads() {
            let name = gamepad.name();
            log::info!("🎮 Found gamepad: {}", name);
            
            // Check if it's a guitar controller
            if name.to_lowercase().contains("guitar") 
                || name.to_lowercase().contains("rock band")
                || name.to_lowercase().contains("hero") {
                log::info!("✅ Guitar controller detected: {}", name);
                return Ok(true);
            }
        }
        
        // If no guitar found, check if any gamepad exists
        if gilrs.gamepads().next().is_some() {
            log::info!("⚠️ Found gamepad(s) but none identified as guitar controller");
            log::info!("💡 Will use first gamepad and map buttons as guitar");
            return Ok(true);
        }
        
        log::info!("ℹ️ No gamepad detected");
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
        let mut gilrs = self.gilrs.lock().unwrap();
        let active_id = *self.active_gamepad.lock().unwrap();
        
        if active_id.is_none() {
            return Ok(());
        }
        
        let active_id = active_id.unwrap();
        
        // Process all pending events
        while let Some(Event { id, event, .. }) = gilrs.next_event() {
            if id != active_id {
                continue; // Ignore events from other gamepads
            }
            
            match event {
                EventType::ButtonPressed(button, _) => {
                    self.handle_button_press(button);
                }
                EventType::ButtonReleased(button, _) => {
                    self.handle_button_release(button);
                }
                EventType::AxisChanged(axis, value, _) => {
                    self.handle_axis_change(axis, value);
                }
                EventType::Connected => {
                    log::info!("🎮 Controller connected");
                }
                EventType::Disconnected => {
                    log::warn!("⚠️ Controller disconnected");
                    *self.active_gamepad.lock().unwrap() = None;
                }
                _ => {}
            }
        }
        
        self.update_timestamp();
        Ok(())
    }

    fn handle_button_press(&self, button: Button) {
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
                state.set_button(ControlId::FretYellow, true);
                log::debug!("🟡 Yellow fret pressed");
            }
            Button::North => {
                state.set_button(ControlId::FretBlue, true);
                log::debug!("🔵 Blue fret pressed");
            }
            Button::LeftTrigger | Button::LeftTrigger2 => {
                state.set_button(ControlId::FretOrange, true);
                log::debug!("🟠 Orange fret pressed");
            }
            
            // Strum bar - usually mapped to D-pad
            Button::DPadUp => {
                state.set_button(ControlId::StrumUp, true);
                log::debug!("⬆️ Strum up");
            }
            Button::DPadDown => {
                state.set_button(ControlId::StrumDown, true);
                log::debug!("⬇️ Strum down");
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
            Button::West => state.set_button(ControlId::FretYellow, false),
            Button::North => state.set_button(ControlId::FretBlue, false),
            Button::LeftTrigger | Button::LeftTrigger2 => state.set_button(ControlId::FretOrange, false),
            Button::DPadUp => state.set_button(ControlId::StrumUp, false),
            Button::DPadDown => state.set_button(ControlId::StrumDown, false),
            Button::Start => state.set_button(ControlId::Start, false),
            Button::Select => state.set_button(ControlId::Select, false),
            _ => {}
        }
    }

    fn handle_axis_change(&self, axis: Axis, value: f32) {
        let mut state = self.state.lock().unwrap();
        
        match axis {
            // Whammy bar - typically on LeftZ axis
            Axis::LeftZ => {
                let normalized = (value + 1.0) / 2.0; // Convert -1..1 to 0..1
                state.set_axis(ControlId::WhammyBar, normalized);
                log::trace!("🎸 Whammy: {:.2}", normalized);
            }
            // Tilt sensor - typically on RightZ or other axis
            Axis::RightZ => {
                let normalized = (value + 1.0) / 2.0;
                state.set_axis(ControlId::TiltSensor, normalized);
                log::trace!("📐 Tilt: {:.2}", normalized);
            }
            _ => {}
        }
    }

    fn update_timestamp(&self) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        self.state.lock().unwrap().timestamp = elapsed;
    }

    pub fn get_state(&self) -> ControllerState {
        self.state.lock().unwrap().clone()
    }

    /// List all connected gamepads (for debugging)
    pub fn list_all_devices(&self) -> Vec<String> {
        let gilrs = self.gilrs.lock().unwrap();
        gilrs.gamepads()
            .map(|(id, gamepad)| {
                format!("[{}] {} ({})", 
                    id, 
                    gamepad.name(),
                    if gamepad.is_connected() { "connected" } else { "disconnected" }
                )
            })
            .collect()
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
