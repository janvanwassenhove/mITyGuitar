use crate::chart::{ChordEvent, ChordMapping};
use std::collections::HashMap;

/// Hit window tolerance in beats
pub const HIT_WINDOW: f64 = 0.5;

/// Result of a strum attempt
#[derive(Debug, Clone, PartialEq)]
pub enum HitResult {
    Hit { event: ChordEventHit, accuracy: f64 },
    Miss { reason: MissReason },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChordEventHit {
    pub beat: f64,
    pub chord: String,
    pub is_sustain: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MissReason {
    NoEventInWindow,
    WrongFrets,
    AlreadyHit,
}

/// Hit detection state
pub struct HitDetector {
    chord_mappings: HashMap<String, Vec<String>>,
    hit_events: Vec<HitEvent>,
    sustaining_event: Option<SustainingEvent>,
}

#[derive(Debug, Clone)]
struct HitEvent {
    beat: f64,
    chord: String,
    hit_at_beat: f64,
}

#[derive(Debug, Clone)]
struct SustainingEvent {
    chord: String,
    start_beat: f64,
    end_beat: f64,
    required_frets: Vec<String>,
}

impl HitDetector {
    pub fn new(chord_mappings: &HashMap<String, ChordMapping>) -> Self {
        let mappings = chord_mappings
            .iter()
            .map(|(name, mapping)| (name.clone(), mapping.frets.clone()))
            .collect();

        Self {
            chord_mappings: mappings,
            hit_events: Vec::new(),
            sustaining_event: None,
        }
    }

    /// Reset hit detection state
    pub fn reset(&mut self) {
        self.hit_events.clear();
        self.sustaining_event = None;
    }

    /// Check if a strum at the current beat with given frets results in a hit
    pub fn check_strum(
        &mut self,
        current_beat: f64,
        pressed_frets: &[String],
        events: &[&ChordEvent],
    ) -> HitResult {
        // Find events within hit window
        let candidates: Vec<&ChordEvent> = events
            .iter()
            .filter(|e| {
                let diff = (e.beat - current_beat).abs();
                diff <= HIT_WINDOW && !self.is_already_hit(e.beat, &e.chord)
            })
            .copied()
            .collect();

        if candidates.is_empty() {
            return HitResult::Miss {
                reason: MissReason::NoEventInWindow,
            };
        }

        // Find closest event that matches frets
        for event in candidates {
            if let Some(required_frets) = self.chord_mappings.get(&event.chord) {
                if self.frets_match(pressed_frets, required_frets) {
                    let accuracy = 1.0 - ((event.beat - current_beat).abs() / HIT_WINDOW);
                    
                    // Register hit
                    self.hit_events.push(HitEvent {
                        beat: event.beat,
                        chord: event.chord.clone(),
                        hit_at_beat: current_beat,
                    });

                    // Start sustain if duration >= 2 beats
                    let is_sustain = event.dur >= 2.0;
                    if is_sustain {
                        self.sustaining_event = Some(SustainingEvent {
                            chord: event.chord.clone(),
                            start_beat: event.beat,
                            end_beat: event.beat + event.dur,
                            required_frets: required_frets.clone(),
                        });
                    }

                    return HitResult::Hit {
                        event: ChordEventHit {
                            beat: event.beat,
                            chord: event.chord.clone(),
                            is_sustain,
                        },
                        accuracy,
                    };
                }
            }
        }

        HitResult::Miss {
            reason: MissReason::WrongFrets,
        }
    }

    /// Update sustain state based on current frets
    pub fn update_sustain(&mut self, current_beat: f64, pressed_frets: &[String]) -> bool {
        if let Some(sustain) = &self.sustaining_event {
            // Check if still in sustain window
            if current_beat < sustain.start_beat || current_beat > sustain.end_beat {
                self.sustaining_event = None;
                return false;
            }

            // Check if frets are still held
            let frets_held = self.frets_match(pressed_frets, &sustain.required_frets);
            if !frets_held {
                self.sustaining_event = None;
                return false;
            }

            return true;
        }

        false
    }

    /// Check if an event was already hit
    fn is_already_hit(&self, beat: f64, chord: &str) -> bool {
        self.hit_events
            .iter()
            .any(|h| (h.beat - beat).abs() < 0.01 && h.chord == chord)
    }

    /// Check if pressed frets match required frets
    fn frets_match(&self, pressed: &[String], required: &[String]) -> bool {
        if pressed.len() != required.len() {
            return false;
        }

        let mut pressed_sorted = pressed.to_vec();
        let mut required_sorted = required.to_vec();
        pressed_sorted.sort();
        required_sorted.sort();

        pressed_sorted == required_sorted
    }

    /// Get hit statistics
    pub fn get_stats(&self) -> HitStats {
        HitStats {
            total_hits: self.hit_events.len(),
        }
    }

    /// Get currently sustaining chord if any
    pub fn get_sustaining_chord(&self) -> Option<String> {
        self.sustaining_event.as_ref().map(|s| s.chord.clone())
    }
}

#[derive(Debug, Clone)]
pub struct HitStats {
    pub total_hits: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::ChordEvent;

    fn create_test_mappings() -> HashMap<String, ChordMapping> {
        let mut mappings = HashMap::new();
        mappings.insert(
            "C".to_string(),
            ChordMapping {
                frets: vec!["GREEN".to_string()],
            },
        );
        mappings.insert(
            "G".to_string(),
            ChordMapping {
                frets: vec!["RED".to_string()],
            },
        );
        mappings
    }

    #[test]
    fn test_hit_detection_success() {
        let mappings = create_test_mappings();
        let mut detector = HitDetector::new(&mappings);

        let event = ChordEvent {
            beat: 10.0,
            dur: 2.0,
            chord: "C".to_string(),
            section: None,
        };

        let result = detector.check_strum(10.1, &["GREEN".to_string()], &[&event]);

        match result {
            HitResult::Hit { event: hit, accuracy } => {
                assert_eq!(hit.chord, "C");
                assert!(accuracy > 0.9);
            }
            _ => panic!("Expected hit"),
        }
    }

    #[test]
    fn test_hit_detection_wrong_frets() {
        let mappings = create_test_mappings();
        let mut detector = HitDetector::new(&mappings);

        let event = ChordEvent {
            beat: 10.0,
            dur: 2.0,
            chord: "C".to_string(),
            section: None,
        };

        let result = detector.check_strum(10.1, &["RED".to_string()], &[&event]);

        match result {
            HitResult::Miss { reason } => {
                assert_eq!(reason, MissReason::WrongFrets);
            }
            _ => panic!("Expected miss"),
        }
    }

    #[test]
    fn test_hit_detection_out_of_window() {
        let mappings = create_test_mappings();
        let mut detector = HitDetector::new(&mappings);

        let event = ChordEvent {
            beat: 10.0,
            dur: 2.0,
            chord: "C".to_string(),
            section: None,
        };

        let result = detector.check_strum(11.0, &["GREEN".to_string()], &[&event]);

        match result {
            HitResult::Miss { reason } => {
                assert_eq!(reason, MissReason::NoEventInWindow);
            }
            _ => panic!("Expected miss"),
        }
    }
}
