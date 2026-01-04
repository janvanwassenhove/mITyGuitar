use serde::{Deserialize, Serialize};
use controller::ControlId;

/// Chord quality/type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChordQuality {
    Major,
    Minor,
    Power,      // Root + fifth (no third)
    Major7,
    Minor7,
    Dominant7,
    Sus2,
    Sus4,
    Diminished,
    Augmented,
}

/// A musical chord
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chord {
    pub root: i8,           // Semitones from reference (0 = reference note)
    pub quality: ChordQuality,
    pub inversion: u8,      // 0 = root position, 1 = first inversion, etc.
}

impl Chord {
    pub fn new(root: i8, quality: ChordQuality) -> Self {
        Self {
            root,
            quality,
            inversion: 0,
        }
    }

    /// Convert chord to MIDI note numbers relative to a root note
    pub fn to_midi_notes(&self, base_note: u8) -> Vec<u8> {
        let root = (base_note as i8 + self.root) as u8;
        let mut intervals = self.get_intervals();
        
        // Apply inversion
        for _ in 0..self.inversion {
            if let Some(first) = intervals.first().copied() {
                intervals.remove(0);
                intervals.push(first + 12);
            }
        }
        
        // Convert intervals to absolute notes
        intervals.iter().map(|&interval| root + interval).collect()
    }

    /// Get the intervals for this chord quality
    fn get_intervals(&self) -> Vec<u8> {
        match self.quality {
            ChordQuality::Major => vec![0, 4, 7],
            ChordQuality::Minor => vec![0, 3, 7],
            ChordQuality::Power => vec![0, 7],
            ChordQuality::Major7 => vec![0, 4, 7, 11],
            ChordQuality::Minor7 => vec![0, 3, 7, 10],
            ChordQuality::Dominant7 => vec![0, 4, 7, 10],
            ChordQuality::Sus2 => vec![0, 2, 7],
            ChordQuality::Sus4 => vec![0, 5, 7],
            ChordQuality::Diminished => vec![0, 3, 6],
            ChordQuality::Augmented => vec![0, 4, 8],
        }
    }
}

/// Pattern that maps fret combinations to chords
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordPattern {
    pub name: String,
    pub mappings: Vec<(Vec<ControlId>, Chord)>,
}

impl ChordPattern {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mappings: Vec::new(),
        }
    }

    /// Add a mapping from fret combination to chord
    pub fn add_mapping(&mut self, frets: Vec<ControlId>, chord: Chord) {
        self.mappings.push((frets, chord));
    }

    /// Map a fret combination to a chord
    pub fn map_frets(&self, frets: &[ControlId]) -> Option<Chord> {
        // Try exact match first
        for (pattern_frets, chord) in &self.mappings {
            if frets.len() == pattern_frets.len() && frets.iter().all(|f| pattern_frets.contains(f)) {
                return Some(chord.clone());
            }
        }
        
        // If no exact match, return a default based on fret count
        if !frets.is_empty() {
            // Simple fallback: single fret = power chord
            Some(Chord::new(0, ChordQuality::Power))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chord_intervals() {
        let chord = Chord::new(0, ChordQuality::Major);
        let notes = chord.to_midi_notes(60);
        assert_eq!(notes, vec![60, 64, 67]); // C, E, G
    }

    #[test]
    fn test_minor_chord() {
        let chord = Chord::new(0, ChordQuality::Minor);
        let notes = chord.to_midi_notes(60);
        assert_eq!(notes, vec![60, 63, 67]); // C, Eb, G
    }

    #[test]
    fn test_power_chord() {
        let chord = Chord::new(0, ChordQuality::Power);
        let notes = chord.to_midi_notes(60);
        assert_eq!(notes, vec![60, 67]); // C, G
    }

    #[test]
    fn test_chord_pattern() {
        let mut pattern = ChordPattern::new("Test");
        pattern.add_mapping(
            vec![ControlId::FretGreen],
            Chord::new(0, ChordQuality::Major),
        );
        
        let result = pattern.map_frets(&[ControlId::FretGreen]);
        assert!(result.is_some());
    }
}
