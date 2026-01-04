use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use gilrs::{Event, EventType, Button, Axis};
use serde::{Deserialize, Serialize};

/// Maximum number of raw events to keep in memory
const MAX_RAW_EVENTS: usize = 500;

/// Raw event captured from gilrs for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawInputEvent {
    /// Monotonic timestamp (milliseconds since diagnostics start)
    pub timestamp_ms: u64,
    /// Unix timestamp (milliseconds since epoch)
    pub unix_timestamp_ms: u64,
    /// Gamepad ID
    pub gamepad_id: usize,
    /// Gamepad name
    pub gamepad_name: String,
    /// Event type (ButtonPressed, ButtonReleased, AxisChanged, etc.)
    pub event_type: String,
    /// Logical button name (if applicable)
    pub button: Option<String>,
    /// Logical axis name (if applicable)
    pub axis: Option<String>,
    /// Axis value (for AxisChanged events)
    pub value: Option<f32>,
    /// Raw event code (Debug format of ev.event)
    pub raw_code: String,
}

impl RawInputEvent {
    /// Create a RawInputEvent from a gilrs Event
    pub fn from_gilrs_event(event: &Event, gamepad_name: &str) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let (event_type, button, axis, value, raw_code) = match event.event {
            EventType::ButtonPressed(btn, code) => (
                "ButtonPressed".to_string(),
                Some(format!("{:?}", btn)),
                None,
                None,
                format!("{:?} (code: {:?})", btn, code),
            ),
            EventType::ButtonRepeated(btn, code) => (
                "ButtonRepeated".to_string(),
                Some(format!("{:?}", btn)),
                None,
                None,
                format!("{:?} (code: {:?})", btn, code),
            ),
            EventType::ButtonReleased(btn, code) => (
                "ButtonReleased".to_string(),
                Some(format!("{:?}", btn)),
                None,
                None,
                format!("{:?} (code: {:?})", btn, code),
            ),
            EventType::ButtonChanged(btn, val, code) => (
                "ButtonChanged".to_string(),
                Some(format!("{:?}", btn)),
                None,
                Some(val),
                format!("{:?} = {} (code: {:?})", btn, val, code),
            ),
            EventType::AxisChanged(ax, val, code) => (
                "AxisChanged".to_string(),
                None,
                Some(format!("{:?}", ax)),
                Some(val),
                format!("{:?} = {:.4} (code: {:?})", ax, val, code),
            ),
            EventType::Connected => (
                "Connected".to_string(),
                None,
                None,
                None,
                "Controller connected".to_string(),
            ),
            EventType::Disconnected => (
                "Disconnected".to_string(),
                None,
                None,
                None,
                "Controller disconnected".to_string(),
            ),
            EventType::Dropped => (
                "Dropped".to_string(),
                None,
                None,
                None,
                "Event dropped".to_string(),
            ),
        };

        Self {
            timestamp_ms,
            unix_timestamp_ms: timestamp_ms,
            gamepad_id: event.id.into(),
            gamepad_name: gamepad_name.to_string(),
            event_type,
            button,
            axis,
            value,
            raw_code,
        }
    }
}

/// Raw diagnostics recorder for guitar controller
pub struct RawDiagnostics {
    enabled: Arc<Mutex<bool>>,
    events: Arc<Mutex<VecDeque<RawInputEvent>>>,
    start_time: Instant,
    max_events: usize,
}

impl RawDiagnostics {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(Mutex::new(false)),
            events: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_RAW_EVENTS))),
            start_time: Instant::now(),
            max_events: MAX_RAW_EVENTS,
        }
    }

    /// Enable or disable raw diagnostics recording
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.lock().unwrap() = enabled;
        log::info!("🔍 Raw diagnostics {}", if enabled { "ENABLED" } else { "DISABLED" });
    }

    /// Check if diagnostics are enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    /// Record a raw gilrs event
    pub fn record_event(&self, event: &Event, gamepad_name: &str) {
        if !self.is_enabled() {
            return;
        }

        let timestamp_ms = self.start_time.elapsed().as_millis() as u64;
        let unix_timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let (event_type, button, axis, value, raw_code) = match event.event {
            EventType::ButtonPressed(btn, code) => (
                "ButtonPressed".to_string(),
                Some(format!("{:?}", btn)),
                None,
                None,
                format!("{:?} (code: {:?})", btn, code),
            ),
            EventType::ButtonRepeated(btn, code) => (
                "ButtonRepeated".to_string(),
                Some(format!("{:?}", btn)),
                None,
                None,
                format!("{:?} (code: {:?})", btn, code),
            ),
            EventType::ButtonReleased(btn, code) => (
                "ButtonReleased".to_string(),
                Some(format!("{:?}", btn)),
                None,
                None,
                format!("{:?} (code: {:?})", btn, code),
            ),
            EventType::ButtonChanged(btn, val, code) => (
                "ButtonChanged".to_string(),
                Some(format!("{:?}", btn)),
                None,
                Some(val),
                format!("{:?} = {} (code: {:?})", btn, val, code),
            ),
            EventType::AxisChanged(ax, val, code) => (
                "AxisChanged".to_string(),
                None,
                Some(format!("{:?}", ax)),
                Some(val),
                format!("{:?} = {:.4} (code: {:?})", ax, val, code),
            ),
            EventType::Connected => (
                "Connected".to_string(),
                None,
                None,
                None,
                "Controller connected".to_string(),
            ),
            EventType::Disconnected => (
                "Disconnected".to_string(),
                None,
                None,
                None,
                "Controller disconnected".to_string(),
            ),
            EventType::Dropped => (
                "Dropped".to_string(),
                None,
                None,
                None,
                "Event dropped".to_string(),
            ),
        };

        let raw_event = RawInputEvent {
            timestamp_ms,
            unix_timestamp_ms,
            gamepad_id: event.id.into(),
            gamepad_name: gamepad_name.to_string(),
            event_type,
            button,
            axis,
            value,
            raw_code,
        };

        let mut events = self.events.lock().unwrap();
        if events.len() >= self.max_events {
            events.pop_front();
        }
        events.push_back(raw_event);
    }

    /// Get all recorded events (newest first)
    pub fn get_events(&self) -> Vec<RawInputEvent> {
        let events = self.events.lock().unwrap();
        events.iter().rev().cloned().collect()
    }

    /// Clear all recorded events
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
        log::info!("🔍 Raw diagnostics cleared");
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    /// Set maximum number of events to keep
    pub fn set_max_events(&self, max: usize) {
        let mut events = self.events.lock().unwrap();
        while events.len() > max {
            events.pop_front();
        }
        drop(events);
        log::info!("🔍 Raw diagnostics max events set to {}", max);
    }
}

impl Default for RawDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to format button enum for display
pub fn format_button(btn: Button) -> &'static str {
    match btn {
        Button::South => "South (A/X/Green)",
        Button::East => "East (B/Circle/Red)",
        Button::North => "North (Y/Triangle/Yellow)",
        Button::West => "West (X/Square/Blue)",
        Button::LeftTrigger => "LeftTrigger (L1/LB/Orange)",
        Button::LeftTrigger2 => "LeftTrigger2 (L2/LT)",
        Button::RightTrigger => "RightTrigger (R1/RB)",
        Button::RightTrigger2 => "RightTrigger2 (R2/RT)",
        Button::Select => "Select/Back",
        Button::Start => "Start",
        Button::DPadUp => "DPad-Up",
        Button::DPadDown => "DPad-Down",
        Button::DPadLeft => "DPad-Left",
        Button::DPadRight => "DPad-Right",
        _ => "Unknown",
    }
}

/// Helper to format axis enum for display
pub fn format_axis(ax: Axis) -> &'static str {
    match ax {
        Axis::LeftStickX => "LeftStick-X",
        Axis::LeftStickY => "LeftStick-Y",
        Axis::RightStickX => "RightStick-X (Whammy)",
        Axis::RightStickY => "RightStick-Y",
        Axis::LeftZ => "LeftZ (L2 Analog)",
        Axis::RightZ => "RightZ (R2 Analog)",
        _ => "Unknown Axis",
    }
}
