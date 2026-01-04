use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All possible controller inputs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlId {
    // Main frets
    FretGreen,
    FretRed,
    FretYellow,
    FretBlue,
    FretOrange,
    
    // Solo frets
    SoloGreen,
    SoloRed,
    SoloYellow,
    SoloBlue,
    SoloOrange,
    
    // Strum bar
    StrumUp,
    StrumDown,
    
    // Analog controls
    WhammyAxis,
    TiltAxis,
    
    // FX switch (3-position)
    FxSwitchUp,
    FxSwitchCenter,
    FxSwitchDown,
    
    // D-pad
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    
    // Standard buttons
    BtnStart,
    BtnSelect,
    BtnSystem,
    
    // Accelerometer
    AccelX,
    AccelY,
    AccelZ,
}

impl ControlId {
    /// Returns true if this is a button (not an axis)
    pub fn is_button(&self) -> bool {
        !matches!(
            self,
            ControlId::WhammyAxis
                | ControlId::TiltAxis
                | ControlId::AccelX
                | ControlId::AccelY
                | ControlId::AccelZ
        )
    }

    /// Returns true if this is an axis (not a button)
    pub fn is_axis(&self) -> bool {
        !self.is_button()
    }

    /// Get a human-readable name for display
    pub fn display_name(&self) -> &'static str {
        match self {
            ControlId::FretGreen => "Green Fret",
            ControlId::FretRed => "Red Fret",
            ControlId::FretYellow => "Yellow Fret",
            ControlId::FretBlue => "Blue Fret",
            ControlId::FretOrange => "Orange Fret",
            ControlId::SoloGreen => "Solo Green",
            ControlId::SoloRed => "Solo Red",
            ControlId::SoloYellow => "Solo Yellow",
            ControlId::SoloBlue => "Solo Blue",
            ControlId::SoloOrange => "Solo Orange",
            ControlId::StrumUp => "Strum Up",
            ControlId::StrumDown => "Strum Down",
            ControlId::WhammyAxis => "Whammy Bar",
            ControlId::TiltAxis => "Tilt",
            ControlId::FxSwitchUp => "FX Up",
            ControlId::FxSwitchCenter => "FX Center",
            ControlId::FxSwitchDown => "FX Down",
            ControlId::DpadUp => "D-Pad Up",
            ControlId::DpadDown => "D-Pad Down",
            ControlId::DpadLeft => "D-Pad Left",
            ControlId::DpadRight => "D-Pad Right",
            ControlId::BtnStart => "Start",
            ControlId::BtnSelect => "Select",
            ControlId::BtnSystem => "System",
            ControlId::AccelX => "Accel X",
            ControlId::AccelY => "Accel Y",
            ControlId::AccelZ => "Accel Z",
        }
    }
}

/// Current state of all controller inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerState {
    /// Button states (true = pressed)
    pub buttons: HashMap<ControlId, bool>,
    
    /// Axis values (normalized -1.0 to 1.0 or 0.0 to 1.0)
    pub axes: HashMap<ControlId, f32>,
    
    /// Timestamp (milliseconds since controller connection)
    pub timestamp_ms: u64,
}

impl Default for ControllerState {
    fn default() -> Self {
        let mut buttons = HashMap::new();
        let mut axes = HashMap::new();
        
        // Initialize all buttons to unpressed
        for control in Self::all_buttons() {
            buttons.insert(control, false);
        }
        
        // Initialize all axes to neutral
        for control in Self::all_axes() {
            axes.insert(control, 0.0);
        }
        
        Self {
            buttons,
            axes,
            timestamp_ms: 0,
        }
    }
}

impl ControllerState {
    fn all_buttons() -> Vec<ControlId> {
        vec![
            ControlId::FretGreen,
            ControlId::FretRed,
            ControlId::FretYellow,
            ControlId::FretBlue,
            ControlId::FretOrange,
            ControlId::SoloGreen,
            ControlId::SoloRed,
            ControlId::SoloYellow,
            ControlId::SoloBlue,
            ControlId::SoloOrange,
            ControlId::StrumUp,
            ControlId::StrumDown,
            ControlId::FxSwitchUp,
            ControlId::FxSwitchCenter,
            ControlId::FxSwitchDown,
            ControlId::DpadUp,
            ControlId::DpadDown,
            ControlId::DpadLeft,
            ControlId::DpadRight,
            ControlId::BtnStart,
            ControlId::BtnSelect,
            ControlId::BtnSystem,
        ]
    }

    fn all_axes() -> Vec<ControlId> {
        vec![
            ControlId::WhammyAxis,
            ControlId::TiltAxis,
            ControlId::AccelX,
            ControlId::AccelY,
            ControlId::AccelZ,
        ]
    }

    /// Get a button state (returns false if not found)
    pub fn button(&self, id: ControlId) -> bool {
        self.buttons.get(&id).copied().unwrap_or(false)
    }

    /// Get an axis value (returns 0.0 if not found)
    pub fn axis(&self, id: ControlId) -> f32 {
        self.axes.get(&id).copied().unwrap_or(0.0)
    }

    /// Set a button state
    pub fn set_button(&mut self, id: ControlId, pressed: bool) {
        self.buttons.insert(id, pressed);
    }

    /// Set an axis value
    pub fn set_axis(&mut self, id: ControlId, value: f32) {
        self.axes.insert(id, value);
    }

    /// Get all currently pressed fret buttons (including solo)
    pub fn pressed_frets(&self) -> Vec<ControlId> {
        let mut frets = Vec::new();
        
        let all_frets = [
            ControlId::FretGreen,
            ControlId::FretRed,
            ControlId::FretYellow,
            ControlId::FretBlue,
            ControlId::FretOrange,
            ControlId::SoloGreen,
            ControlId::SoloRed,
            ControlId::SoloYellow,
            ControlId::SoloBlue,
            ControlId::SoloOrange,
        ];
        
        for fret in all_frets {
            if self.button(fret) {
                frets.push(fret);
            }
        }
        
        frets
    }

    /// Check if any strum action is active
    pub fn is_strumming(&self) -> bool {
        self.button(ControlId::StrumUp) || self.button(ControlId::StrumDown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = ControllerState::default();
        assert!(!state.button(ControlId::FretGreen));
        assert_eq!(state.axis(ControlId::WhammyAxis), 0.0);
    }

    #[test]
    fn test_button_operations() {
        let mut state = ControllerState::default();
        assert!(!state.button(ControlId::FretGreen));
        
        state.set_button(ControlId::FretGreen, true);
        assert!(state.button(ControlId::FretGreen));
    }

    #[test]
    fn test_pressed_frets() {
        let mut state = ControllerState::default();
        state.set_button(ControlId::FretGreen, true);
        state.set_button(ControlId::FretRed, true);
        
        let frets = state.pressed_frets();
        assert_eq!(frets.len(), 2);
        assert!(frets.contains(&ControlId::FretGreen));
        assert!(frets.contains(&ControlId::FretRed));
    }
}
