use serde::{Deserialize, Serialize};
use controller::ControlId;
use crate::chord::{Chord, ChordPattern, ChordQuality};

/// Musical genre with associated chord patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Genre {
    Punk,
    Rock,
    Edm,
    Metal,
    Folk,
    Pop,
}

impl Genre {
    /// Get all available genres
    pub fn all() -> Vec<Genre> {
        vec![Genre::Punk, Genre::Rock, Genre::Edm, Genre::Metal, Genre::Folk, Genre::Pop]
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Genre::Punk => "Punk",
            Genre::Rock => "Rock",
            Genre::Edm => "EDM",
            Genre::Metal => "Metal",
            Genre::Folk => "Folk",
            Genre::Pop => "Pop",
        }
    }

    /// Get chord patterns for this genre
    pub fn get_patterns(&self) -> Vec<ChordPattern> {
        match self {
            Genre::Punk => Self::punk_patterns(),
            Genre::Rock => Self::rock_patterns(),
            Genre::Edm => Self::edm_patterns(),
            Genre::Metal => Self::metal_patterns(),
            Genre::Folk => Self::folk_patterns(),
            Genre::Pop => Self::pop_patterns(),
        }
    }

    /// Punk patterns: Power chords and aggressive voicings
    fn punk_patterns() -> Vec<ChordPattern> {
        let mut patterns = Vec::new();

        // Pattern 1: Basic power chords
        let mut p1 = ChordPattern::new("Punk Power Chords");
        p1.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Power));  // E
        p1.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Power));    // A
        p1.add_mapping(vec![ControlId::FretYellow], Chord::new(7, ChordQuality::Power)); // B
        p1.add_mapping(vec![ControlId::FretBlue], Chord::new(3, ChordQuality::Power));   // G
        p1.add_mapping(vec![ControlId::FretOrange], Chord::new(-2, ChordQuality::Power)); // D
        
        // Two fret combos
        p1.add_mapping(
            vec![ControlId::FretGreen, ControlId::FretRed],
            Chord::new(0, ChordQuality::Power)
        );
        p1.add_mapping(
            vec![ControlId::FretRed, ControlId::FretYellow],
            Chord::new(5, ChordQuality::Power)
        );
        patterns.push(p1);

        // Pattern 2: Aggressive sus chords
        let mut p2 = ChordPattern::new("Punk Sus");
        p2.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Sus4));
        p2.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Sus4));
        p2.add_mapping(vec![ControlId::FretYellow], Chord::new(7, ChordQuality::Power));
        patterns.push(p2);

        // Pattern 3: Low tuning power chords
        let mut p3 = ChordPattern::new("Punk Drop D");
        p3.add_mapping(vec![ControlId::FretGreen], Chord::new(-2, ChordQuality::Power)); // D
        p3.add_mapping(vec![ControlId::FretRed], Chord::new(0, ChordQuality::Power));    // E
        p3.add_mapping(vec![ControlId::FretYellow], Chord::new(3, ChordQuality::Power)); // G
        p3.add_mapping(vec![ControlId::FretBlue], Chord::new(5, ChordQuality::Power));   // A
        patterns.push(p3);

        patterns
    }

    /// Rock patterns: Triads and sus chords
    fn rock_patterns() -> Vec<ChordPattern> {
        let mut patterns = Vec::new();

        // Pattern 1: Major triads
        let mut p1 = ChordPattern::new("Rock Major");
        p1.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Major));  // E
        p1.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Major));    // A
        p1.add_mapping(vec![ControlId::FretYellow], Chord::new(-2, ChordQuality::Major)); // D
        p1.add_mapping(vec![ControlId::FretBlue], Chord::new(3, ChordQuality::Major));   // G
        p1.add_mapping(vec![ControlId::FretOrange], Chord::new(7, ChordQuality::Major)); // B
        patterns.push(p1);

        // Pattern 2: Power chords
        let mut p2 = ChordPattern::new("Rock Power");
        p2.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Power));
        p2.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Power));
        p2.add_mapping(vec![ControlId::FretYellow], Chord::new(7, ChordQuality::Power));
        p2.add_mapping(vec![ControlId::FretBlue], Chord::new(3, ChordQuality::Power));
        patterns.push(p2);

        // Pattern 3: Mixed major/minor
        let mut p3 = ChordPattern::new("Rock Mixed");
        p3.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Major));
        p3.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Minor));
        p3.add_mapping(vec![ControlId::FretYellow], Chord::new(-2, ChordQuality::Major));
        p3.add_mapping(vec![ControlId::FretBlue], Chord::new(3, ChordQuality::Major));
        patterns.push(p3);

        // Pattern 4: 7th chords
        let mut p4 = ChordPattern::new("Rock 7ths");
        p4.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Dominant7));
        p4.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Dominant7));
        p4.add_mapping(vec![ControlId::FretYellow], Chord::new(7, ChordQuality::Major7));
        patterns.push(p4);

        patterns
    }

    /// EDM patterns: Minor-first with 7ths/9ths
    fn edm_patterns() -> Vec<ChordPattern> {
        let mut patterns = Vec::new();

        // Pattern 1: Minor chords
        let mut p1 = ChordPattern::new("EDM Minor");
        p1.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Minor));
        p1.add_mapping(vec![ControlId::FretRed], Chord::new(3, ChordQuality::Minor));
        p1.add_mapping(vec![ControlId::FretYellow], Chord::new(5, ChordQuality::Minor));
        p1.add_mapping(vec![ControlId::FretBlue], Chord::new(7, ChordQuality::Major));
        p1.add_mapping(vec![ControlId::FretOrange], Chord::new(10, ChordQuality::Major));
        patterns.push(p1);

        // Pattern 2: Minor 7ths
        let mut p2 = ChordPattern::new("EDM Minor 7");
        p2.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Minor7));
        p2.add_mapping(vec![ControlId::FretRed], Chord::new(3, ChordQuality::Minor7));
        p2.add_mapping(vec![ControlId::FretYellow], Chord::new(5, ChordQuality::Minor7));
        p2.add_mapping(vec![ControlId::FretBlue], Chord::new(7, ChordQuality::Major7));
        patterns.push(p2);

        // Pattern 3: Sus chords for build-ups
        let mut p3 = ChordPattern::new("EDM Sus");
        p3.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Sus2));
        p3.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Sus2));
        p3.add_mapping(vec![ControlId::FretYellow], Chord::new(7, ChordQuality::Sus4));
        p3.add_mapping(vec![ControlId::FretBlue], Chord::new(3, ChordQuality::Sus4));
        patterns.push(p3);

        // Pattern 4: Augmented/diminished for tension
        let mut p4 = ChordPattern::new("EDM Tension");
        p4.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Minor));
        p4.add_mapping(vec![ControlId::FretRed], Chord::new(2, ChordQuality::Diminished));
        p4.add_mapping(vec![ControlId::FretYellow], Chord::new(5, ChordQuality::Minor));
        p4.add_mapping(vec![ControlId::FretBlue], Chord::new(8, ChordQuality::Augmented));
        patterns.push(p4);

        patterns
    }
    
    /// Metal patterns: Heavy power chords and palm mutes
    fn metal_patterns() -> Vec<ChordPattern> {
        let mut patterns = Vec::new();

        // Pattern 1: Drop-tuned power chords
        let mut p1 = ChordPattern::new("Metal Power");
        p1.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Power));  // I
        p1.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Power));    // IV
        p1.add_mapping(vec![ControlId::FretYellow], Chord::new(7, ChordQuality::Power)); // V
        p1.add_mapping(vec![ControlId::FretBlue], Chord::new(10, ChordQuality::Power));  // bVII
        p1.add_mapping(vec![ControlId::FretOrange], Chord::new(2, ChordQuality::Power)); // II
        patterns.push(p1);

        // Pattern 2: Diminished for dark tension
        let mut p2 = ChordPattern::new("Metal Dark");
        p2.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Minor));
        p2.add_mapping(vec![ControlId::FretRed], Chord::new(3, ChordQuality::Diminished));
        p2.add_mapping(vec![ControlId::FretYellow], Chord::new(5, ChordQuality::Power));
        p2.add_mapping(vec![ControlId::FretBlue], Chord::new(7, ChordQuality::Minor));
        p2.add_mapping(vec![ControlId::FretOrange], Chord::new(10, ChordQuality::Power));
        patterns.push(p2);

        // Pattern 3: Aggressive tritones
        let mut p3 = ChordPattern::new("Metal Aggro");
        p3.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Power));
        p3.add_mapping(vec![ControlId::FretRed], Chord::new(6, ChordQuality::Diminished)); // Tritone
        p3.add_mapping(vec![ControlId::FretYellow], Chord::new(5, ChordQuality::Power));
        p3.add_mapping(vec![ControlId::FretBlue], Chord::new(10, ChordQuality::Power));
        patterns.push(p3);

        patterns
    }
    
    /// Folk patterns: Open chords and suspended sounds
    fn folk_patterns() -> Vec<ChordPattern> {
        let mut patterns = Vec::new();

        // Pattern 1: Classic folk progression
        let mut p1 = ChordPattern::new("Folk Classic");
        p1.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Major));   // I
        p1.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Major));     // IV
        p1.add_mapping(vec![ControlId::FretYellow], Chord::new(7, ChordQuality::Major));  // V
        p1.add_mapping(vec![ControlId::FretBlue], Chord::new(-5, ChordQuality::Major));   // VI
        p1.add_mapping(vec![ControlId::FretOrange], Chord::new(2, ChordQuality::Minor));  // ii
        patterns.push(p1);

        // Pattern 2: Sus chords for texture
        let mut p2 = ChordPattern::new("Folk Texture");
        p2.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Sus2));
        p2.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Sus4));
        p2.add_mapping(vec![ControlId::FretYellow], Chord::new(7, ChordQuality::Sus2));
        p2.add_mapping(vec![ControlId::FretBlue], Chord::new(-5, ChordQuality::Major));
        patterns.push(p2);

        // Pattern 3: Minor folk
        let mut p3 = ChordPattern::new("Folk Minor");
        p3.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Minor));
        p3.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Minor));
        p3.add_mapping(vec![ControlId::FretYellow], Chord::new(7, ChordQuality::Major));
        p3.add_mapping(vec![ControlId::FretBlue], Chord::new(3, ChordQuality::Major));
        patterns.push(p3);

        patterns
    }
    
    /// Pop patterns: Major chords with 7ths
    fn pop_patterns() -> Vec<ChordPattern> {
        let mut patterns = Vec::new();

        // Pattern 1: Pop progression (I-V-vi-IV)
        let mut p1 = ChordPattern::new("Pop Classic");
        p1.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Major));   // I
        p1.add_mapping(vec![ControlId::FretRed], Chord::new(7, ChordQuality::Major));     // V
        p1.add_mapping(vec![ControlId::FretYellow], Chord::new(-3, ChordQuality::Minor)); // vi
        p1.add_mapping(vec![ControlId::FretBlue], Chord::new(5, ChordQuality::Major));    // IV
        p1.add_mapping(vec![ControlId::FretOrange], Chord::new(2, ChordQuality::Minor));  // ii
        patterns.push(p1);

        // Pattern 2: Pop with 7ths
        let mut p2 = ChordPattern::new("Pop 7ths");
        p2.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Major7));
        p2.add_mapping(vec![ControlId::FretRed], Chord::new(7, ChordQuality::Dominant7));
        p2.add_mapping(vec![ControlId::FretYellow], Chord::new(-3, ChordQuality::Minor7));
        p2.add_mapping(vec![ControlId::FretBlue], Chord::new(5, ChordQuality::Major7));
        patterns.push(p2);

        // Pattern 3: Bright pop
        let mut p3 = ChordPattern::new("Pop Bright");
        p3.add_mapping(vec![ControlId::FretGreen], Chord::new(0, ChordQuality::Major));
        p3.add_mapping(vec![ControlId::FretRed], Chord::new(5, ChordQuality::Major));
        p3.add_mapping(vec![ControlId::FretYellow], Chord::new(7, ChordQuality::Major));
        p3.add_mapping(vec![ControlId::FretBlue], Chord::new(2, ChordQuality::Sus2));
        patterns.push(p3);

        patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_genres() {
        let genres = Genre::all();
        assert_eq!(genres.len(), 6);
    }

    #[test]
    fn test_genre_patterns() {
        assert!(!Genre::Punk.get_patterns().is_empty());
        assert!(!Genre::Rock.get_patterns().is_empty());
        assert!(!Genre::Edm.get_patterns().is_empty());
        assert!(!Genre::Metal.get_patterns().is_empty());
        assert!(!Genre::Folk.get_patterns().is_empty());
        assert!(!Genre::Pop.get_patterns().is_empty());
    }

    #[test]
    fn test_punk_has_power_chords() {
        let patterns = Genre::Punk.get_patterns();
        assert!(patterns.iter().any(|p| p.name.contains("Power")));
    }

    #[test]
    fn test_edm_has_minor_chords() {
        let patterns = Genre::Edm.get_patterns();
        assert!(patterns.iter().any(|p| p.name.contains("Minor")));
    }
}
