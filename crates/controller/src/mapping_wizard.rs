use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use gilrs::Event;
use serde::{Deserialize, Serialize};
use crate::raw_diagnostics::RawInputEvent;
use crate::mapping_profile::{AppAction, RawBinding, ButtonBinding, AxisBinding};

/// Capture state for the mapping wizard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureState {
    /// Currently capturing for this action
    pub target_action: Option<AppAction>,
    /// Capture start time (milliseconds since epoch)
    pub started_at: Option<u64>,
    /// Duration of capture window (milliseconds)
    pub duration_ms: u64,
    /// Events captured during this session
    pub captured_events: Vec<CapturedEventSummary>,
    /// Auto-capture mode enabled
    pub auto_capture: bool,
    /// Whether capture is currently active
    pub is_active: bool,
}

/// Summary of a captured event for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedEventSummary {
    pub timestamp_ms: u64,
    pub event_type: String,
    pub button: Option<String>,
    pub axis: Option<String>,
    pub value: Option<f32>,
    pub raw_code: String,
    pub signature: String,
}

impl CapturedEventSummary {
    pub fn from_raw_event(event: &RawInputEvent) -> Self {
        let signature = if let Some(btn) = &event.button {
            format!("btn:{}:{}", event.raw_code, btn)
        } else if let Some(ax) = &event.axis {
            format!("axis:{}", ax)
        } else {
            "unknown".to_string()
        };

        Self {
            timestamp_ms: event.timestamp_ms,
            event_type: event.event_type.clone(),
            button: event.button.clone(),
            axis: event.axis.clone(),
            value: event.value,
            raw_code: event.raw_code.clone(),
            signature,
        }
    }
}

/// Result of a capture attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureResult {
    pub success: bool,
    pub binding: Option<RawBinding>,
    pub message: String,
    pub conflict: Option<AppAction>,
}

/// Capture wizard for mapping controller inputs
pub struct MappingWizard {
    state: Arc<Mutex<CaptureState>>,
}

impl MappingWizard {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CaptureState {
                target_action: None,
                started_at: None,
                duration_ms: 2000, // 2 second capture window
                captured_events: Vec::new(),
                auto_capture: false,
                is_active: false,
            })),
        }
    }

    /// Start capturing for a specific action
    pub fn start_capture(&self, action: AppAction) {
        let mut state = self.state.lock().unwrap();
        state.target_action = Some(action);
        state.started_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
        );
        state.captured_events.clear();
        state.is_active = true;
        log::info!("🎯 Started capture for: {:?}", action);
    }

    /// Stop capturing
    pub fn stop_capture(&self) {
        let mut state = self.state.lock().unwrap();
        state.is_active = false;
        log::info!("⏹️ Stopped capture");
    }

    /// Record a raw event during capture
    pub fn record_event(&self, event: &RawInputEvent) {
        let mut state = self.state.lock().unwrap();
        
        if !state.is_active {
            return;
        }

        // Check if capture window has expired
        if let Some(started_at) = state.started_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            
            if now - started_at > state.duration_ms {
                state.is_active = false;
                log::info!("⏱️ Capture window expired");
                return;
            }
        }

        // Add event to captured list
        state.captured_events.push(CapturedEventSummary::from_raw_event(event));
    }

    /// Analyze captured events and generate a binding
    pub fn finalize_capture(&self) -> CaptureResult {
        let mut state = self.state.lock().unwrap();
        state.is_active = false;

        if state.captured_events.is_empty() {
            return CaptureResult {
                success: false,
                binding: None,
                message: "No events captured".to_string(),
                conflict: None,
            };
        }

        // Analyze events to determine best binding
        let button_events: Vec<_> = state.captured_events.iter()
            .filter(|e| e.button.is_some())
            .collect();
        
        let axis_events: Vec<_> = state.captured_events.iter()
            .filter(|e| e.axis.is_some())
            .collect();

        // Prefer button bindings for button-like controls
        if !button_events.is_empty() {
            // Use the most common button event
            let mut button_counts = std::collections::HashMap::new();
            for event in &button_events {
                *button_counts.entry(event.signature.clone()).or_insert(0) += 1;
            }

            if let Some((signature, _)) = button_counts.iter().max_by_key(|(_, count)| *count) {
                let event = button_events.iter()
                    .find(|e| &e.signature == signature)
                    .unwrap();

                let binding = RawBinding::Button(ButtonBinding {
                    code: event.raw_code.clone(),
                    logical_button: event.button.clone(),
                });

                let message = format!("Captured button: {}", event.button.as_ref().unwrap_or(&"unknown".to_string()));
                
                state.captured_events.clear();

                return CaptureResult {
                    success: true,
                    binding: Some(binding),
                    message,
                    conflict: None,
                };
            }
        }

        // Handle axis bindings
        if !axis_events.is_empty() {
            let axis_name = axis_events[0].axis.clone().unwrap();
            let mut min = f32::MAX;
            let mut max = f32::MIN;

            for event in &axis_events {
                if let Some(val) = event.value {
                    min = min.min(val);
                    max = max.max(val);
                }
            }

            // Only consider it a valid axis if there was movement
            if (max - min).abs() > 0.1 {
                let binding = RawBinding::Axis(AxisBinding {
                    code: None,
                    logical_axis: axis_name.clone(),
                    min,
                    max,
                    deadzone: 0.05,
                    invert: false,
                });

                state.captured_events.clear();

                return CaptureResult {
                    success: true,
                    binding: Some(binding),
                    message: format!("Captured axis: {} (range: {:.2} to {:.2})", axis_name, min, max),
                    conflict: None,
                };
            }
        }

        state.captured_events.clear();

        CaptureResult {
            success: false,
            binding: None,
            message: "Could not determine valid binding from captured events".to_string(),
            conflict: None,
        }
    }

    /// Get current capture state
    pub fn get_state(&self) -> CaptureState {
        self.state.lock().unwrap().clone()
    }

    /// Set auto-capture mode
    pub fn set_auto_capture(&self, enabled: bool) {
        let mut state = self.state.lock().unwrap();
        state.auto_capture = enabled;
    }

    /// Set capture duration
    pub fn set_capture_duration(&self, duration_ms: u64) {
        let mut state = self.state.lock().unwrap();
        state.duration_ms = duration_ms;
    }

    /// Clear captured events
    pub fn clear_events(&self) {
        let mut state = self.state.lock().unwrap();
        state.captured_events.clear();
    }
}

impl Default for MappingWizard {
    fn default() -> Self {
        Self::new()
    }
}
