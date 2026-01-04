//! Controller simulator for development and testing without hardware

use crate::{ControlId, ControllerState};
use std::collections::HashMap;

/// Maps keyboard keys to controller inputs for simulation
pub struct ControllerSimulator {
    state: ControllerState,
    key_bindings: HashMap<String, ControlId>,
}

impl ControllerSimulator {
    pub fn new() -> Self {
        let mut key_bindings = HashMap::new();
        
        // Fret buttons (1-5)
        key_bindings.insert("1".to_string(), ControlId::FretGreen);
        key_bindings.insert("2".to_string(), ControlId::FretRed);
        key_bindings.insert("3".to_string(), ControlId::FretYellow);
        key_bindings.insert("4".to_string(), ControlId::FretBlue);
        key_bindings.insert("5".to_string(), ControlId::FretOrange);
        
        // Solo buttons (Q, W, E, R, T)
        key_bindings.insert("q".to_string(), ControlId::SoloGreen);
        key_bindings.insert("Q".to_string(), ControlId::SoloGreen);
        key_bindings.insert("w".to_string(), ControlId::SoloRed);
        key_bindings.insert("W".to_string(), ControlId::SoloRed);
        key_bindings.insert("e".to_string(), ControlId::SoloYellow);
        key_bindings.insert("E".to_string(), ControlId::SoloYellow);
        key_bindings.insert("r".to_string(), ControlId::SoloBlue);
        key_bindings.insert("R".to_string(), ControlId::SoloBlue);
        key_bindings.insert("t".to_string(), ControlId::SoloOrange);
        key_bindings.insert("T".to_string(), ControlId::SoloOrange);
        
        // Strum (Arrow Up/Down or Space)
        key_bindings.insert("ArrowUp".to_string(), ControlId::StrumUp);
        key_bindings.insert("ArrowDown".to_string(), ControlId::StrumDown);
        key_bindings.insert(" ".to_string(), ControlId::StrumDown); // Space key
        
        // Standard buttons
        key_bindings.insert("Enter".to_string(), ControlId::Start);
        key_bindings.insert("Escape".to_string(), ControlId::Select);
        
        Self {
            state: ControllerState::default(),
            key_bindings,
        }
    }

    /// Handle a keyboard key press
    pub fn key_down(&mut self, key: &str) {
        log::debug!("🎹 Key down: {:?}", key);
        if let Some(control) = self.key_bindings.get(key) {
            log::debug!("  ➜ Mapped to: {:?}", control);
            self.state.set_button(*control, true);
        } else {
            log::trace!("  ➜ No mapping found for key: {:?}", key);
        }
    }

    /// Handle a keyboard key release
    pub fn key_up(&mut self, key: &str) {
        log::debug!("🎹 Key up: {:?}", key);
        if let Some(control) = self.key_bindings.get(key) {
            log::debug!("  ➜ Unmapped from: {:?}", control);
            self.state.set_button(*control, false);
        }
    }

    /// Update an axis value (for UI sliders during testing)
    pub fn set_axis(&mut self, control: ControlId, value: f32) {
        self.state.set_axis(control, value.clamp(-1.0, 1.0));
    }

    /// Get current state
    pub fn get_state(&self) -> &ControllerState {
        &self.state
    }

    /// Get a mutable reference to state for direct manipulation
    pub fn get_state_mut(&mut self) -> &mut ControllerState {
        &mut self.state
    }

    /// Get keyboard bindings for UI display
    pub fn get_bindings(&self) -> &HashMap<String, ControlId> {
        &self.key_bindings
    }
}

impl Default for ControllerSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulator_key_press() {
        let mut sim = ControllerSimulator::new();
        
        sim.key_down("1");
        assert!(sim.get_state().button(ControlId::FretGreen));
        
        sim.key_up("1");
        assert!(!sim.get_state().button(ControlId::FretGreen));
    }

    #[test]
    fn test_simulator_strum() {
        let mut sim = ControllerSimulator::new();
        
        sim.key_down("Space");
        assert!(sim.get_state().button(ControlId::StrumDown));
    }
}
