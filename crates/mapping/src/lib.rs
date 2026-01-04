use serde::{Deserialize, Serialize};

pub mod chord;
pub mod genre;
pub mod harmonic;
pub mod resolution;
pub mod performance;
pub mod presets;

// Re-export legacy types for compatibility
pub use chord::{Chord, ChordQuality, ChordPattern};
pub use genre::Genre as LegacyGenre;

// New genre-based chord mapping API
pub use harmonic::{
    FretButton, HarmonicRole, Genre, Mode, Note, ChordQuality as NewChordQuality, 
    ChordSpec, GenrePreset, PatternChordOverride, FretRow, WhammyDefaults, SustainDefaults
};
pub use resolution::ChordResolver;
pub use performance::{PerformanceEngine, PerformanceEvent, PerformanceState};
pub use presets::PresetLoader;

use controller::{ControlId, ControllerState};

/// Musical event generated from controller input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MusicEvent {
    /// Start playing a note
    NoteOn { note: u8, velocity: u8 },
    
    /// Stop playing a note
    NoteOff { note: u8 },
    
    /// Pitch bend (-8192 to 8191, 0 = center)
    PitchBend(i16),
    
    /// Control change (CC number, value 0-127)
    ControlChange { cc: u8, value: u8 },
    
    /// Change instrument preset
    PresetChange(usize),
    
    /// Stop all notes immediately
    PanicAllNotesOff,
}

/// Maps controller state to musical events (Legacy - for compatibility)
pub struct Mapper {
    genre: LegacyGenre,
    pattern_index: usize,
    last_strum_state: bool,
    last_frets: Vec<ControlId>,
    active_notes: Vec<u8>,
    /// Current key root (0-11 for C-B)
    key_root: u8,
    /// Current mode (true = major, false = minor)
    is_major: bool,
}

impl Mapper {
    pub fn new(genre: LegacyGenre) -> Self {
        Self {
            genre,
            pattern_index: 0,
            last_strum_state: false,
            last_frets: Vec::new(),
            active_notes: Vec::new(),
            key_root: 4, // Default to E
            is_major: true, // Default to Major
        }
    }
    
    /// Create a new mapper with specific key and mode
    pub fn new_with_key_mode(genre: LegacyGenre, key_root: u8, is_major: bool) -> Self {
        Self {
            genre,
            pattern_index: 0,
            last_strum_state: false,
            last_frets: Vec::new(),
            active_notes: Vec::new(),
            key_root: key_root % 12,
            is_major,
        }
    }

    /// Process controller state and generate musical events
    pub fn process(&mut self, state: &ControllerState) -> Vec<MusicEvent> {
        let mut events = Vec::new();

        // Get current fret combination
        let frets = state.pressed_frets();
        
        // Check for strum trigger (edge detection)
        let strum_active = state.is_strumming();
        let strum_triggered = strum_active && !self.last_strum_state;
        let strum_released = !strum_active && self.last_strum_state;
        self.last_strum_state = strum_active;

        // Check if frets changed while notes are playing
        let frets_changed = frets != self.last_frets && !self.active_notes.is_empty();

        if strum_triggered {
            // Release previous notes (let them fade out naturally)
            for note in &self.active_notes {
                events.push(MusicEvent::NoteOff { note: *note });
            }
            self.active_notes.clear();

            // Map to chord
            if let Some(chord) = self.fret_combo_to_chord(&frets) {
                // Play chord notes - transpose based on current key
                // The chord.root is an offset from E (which is 0 in the chord system)
                // We need to add our key_root to transpose it
                let base_note = 40 + self.key_root; // E2 (40) + key_root offset
                let notes = chord.to_midi_notes(base_note);
                let velocity = 100; // TODO: Calculate from strum velocity
                
                for note in &notes {
                    events.push(MusicEvent::NoteOn {
                        note: *note,
                        velocity,
                    });
                    self.active_notes.push(*note);
                }
            } else {
                // No frets pressed or invalid combo - play single note
                let note = 40 + self.key_root;
                events.push(MusicEvent::NoteOn { note, velocity: 100 });
                self.active_notes.push(note);
            }
            
            self.last_frets = frets.clone();
        } else if strum_released {
            // Release all active notes when strum is released (let them fade out)
            for note in &self.active_notes {
                events.push(MusicEvent::NoteOff { note: *note });
            }
            self.active_notes.clear();
            self.last_frets = frets;
        } else if frets_changed {
            // When frets change while strumming, release old notes and play new ones
            // This allows natural fade-out while new notes start
            for note in &self.active_notes {
                events.push(MusicEvent::NoteOff { note: *note });
            }
            self.active_notes.clear();
            
            // Play new chord immediately
            if let Some(chord) = self.fret_combo_to_chord(&frets) {
                let base_note = 40 + self.key_root;
                let notes = chord.to_midi_notes(base_note);
                let velocity = 100;
                
                for note in &notes {
                    events.push(MusicEvent::NoteOn {
                        note: *note,
                        velocity,
                    });
                    self.active_notes.push(*note);
                }
            } else {
                let note = 40 + self.key_root;
                events.push(MusicEvent::NoteOn { note, velocity: 100 });
                self.active_notes.push(note);
            }
            
            self.last_frets = frets;
        }

        // Handle whammy bar for pitch bend
        let whammy = state.axis(ControlId::WhammyBar);
        if whammy.abs() > 0.01 {
            let bend_amount = (whammy * 8191.0) as i16;
            events.push(MusicEvent::PitchBend(bend_amount));
        }

        events
    }

    /// Map fret combination to a chord
    fn fret_combo_to_chord(&self, frets: &[ControlId]) -> Option<Chord> {
        if frets.is_empty() {
            return None;
        }

        let patterns = self.genre.get_patterns();
        if patterns.is_empty() {
            return None;
        }

        let pattern = &patterns[self.pattern_index % patterns.len()];
        let mut chord = pattern.map_frets(frets)?;
        
        // Adjust chord quality based on mode
        // In minor keys, convert major chords to minor (except V and bVII)
        if !self.is_major {
            chord.quality = match chord.quality {
                ChordQuality::Major => ChordQuality::Minor,
                ChordQuality::Major7 => ChordQuality::Minor7,
                // Keep power chords, sus chords, and dominant as-is
                other => other,
            };
        }
        
        Some(chord)
    }

    /// Change genre
    pub fn set_genre(&mut self, genre: LegacyGenre) {
        self.genre = genre;
        self.pattern_index = 0;
    }

    /// Get current genre
    pub fn genre(&self) -> &LegacyGenre {
        &self.genre
    }
    
    /// Set the key root (0-11 for C-B)
    pub fn set_key_root(&mut self, key_root: u8) {
        self.key_root = key_root % 12;
    }
    
    /// Get the current key root
    pub fn key_root(&self) -> u8 {
        self.key_root
    }
    
    /// Set the mode (true = major, false = minor)
    pub fn set_mode(&mut self, is_major: bool) {
        self.is_major = is_major;
    }
    
    /// Get the current mode
    pub fn is_major(&self) -> bool {
        self.is_major
    }

    /// Cycle to next pattern
    pub fn next_pattern(&mut self) {
        let patterns = self.genre.get_patterns();
        if !patterns.is_empty() {
            self.pattern_index = (self.pattern_index + 1) % patterns.len();
        }
    }

    /// Cycle to previous pattern
    pub fn prev_pattern(&mut self) {
        let patterns = self.genre.get_patterns();
        if !patterns.is_empty() {
            if self.pattern_index == 0 {
                self.pattern_index = patterns.len() - 1;
            } else {
                self.pattern_index -= 1;
            }
        }
    }

    /// Get current pattern index
    pub fn pattern_index(&self) -> usize {
        self.pattern_index
    }

    /// Send panic/all notes off
    pub fn panic(&mut self) -> Vec<MusicEvent> {
        let mut events = Vec::new();
        
        for note in &self.active_notes {
            events.push(MusicEvent::NoteOff { note: *note });
        }
        self.active_notes.clear();
        
        events.push(MusicEvent::PanicAllNotesOff);
        events
    }
}

impl Default for Mapper {
    fn default() -> Self {
        Self::new(LegacyGenre::Rock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapper_creation() {
        let mapper = Mapper::new(Genre::Rock);
        assert_eq!(mapper.pattern_index(), 0);
    }

    #[test]
    fn test_pattern_navigation() {
        let mut mapper = Mapper::new(LegacyGenre::Punk);
        let initial = mapper.pattern_index();
        
        mapper.next_pattern();
        assert_ne!(mapper.pattern_index(), initial);
        
        mapper.prev_pattern();
        assert_eq!(mapper.pattern_index(), initial);
    }
}
