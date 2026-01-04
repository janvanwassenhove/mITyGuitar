use serde::{Deserialize, Serialize};

/// Musical fret button mappings (constant across all genres)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FretButton {
    Green,
    Red,
    Yellow,
    Blue,
    Orange,
}

/// Harmonic roles that map to specific chords based on genre
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum HarmonicRole {
    I,        // Root/home
    IV,       // Movement
    V,        // Drive/tension
    bVII,     // Anthem/punk color
    II,       // Minor ii
    VI,       // Relative minor vi
}

/// Musical genres with different chord mapping approaches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Genre {
    Punk,
    Edm,
    Rock,
    Pop,
    Folk,
    Metal,
}

impl Genre {
    /// Get all available genres
    pub fn all() -> &'static [Genre] {
        &[Genre::Punk, Genre::Edm, Genre::Rock, Genre::Pop, Genre::Folk, Genre::Metal]
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Genre::Punk => "Punk",
            Genre::Edm => "EDM", 
            Genre::Rock => "Rock",
            Genre::Pop => "Pop",
            Genre::Folk => "Folk",
            Genre::Metal => "Metal",
        }
    }

    /// Get default key root for this genre
    pub fn default_key_root(&self) -> Note {
        match self {
            Genre::Punk => Note::E,
            Genre::Edm => Note::A,
            Genre::Rock => Note::A,
            Genre::Pop => Note::C,
            Genre::Folk => Note::G,
            Genre::Metal => Note::E,
        }
    }

    /// Get default mode for this genre
    pub fn default_mode(&self) -> Mode {
        match self {
            Genre::Punk | Genre::Rock | Genre::Pop | Genre::Folk => Mode::Major,
            Genre::Edm | Genre::Metal => Mode::Minor,
        }
    }
}

/// Musical modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    Major,
    Minor,
}

/// Chord qualities/types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChordQuality {
    #[serde(rename = "power5")]
    Power5,      // Power chord (root + fifth)
    #[serde(rename = "major")]
    Major,       // Major triad
    #[serde(rename = "minor")]
    Minor,       // Minor triad
    #[serde(rename = "sus2")]
    Sus2,        // Suspended 2nd
    #[serde(rename = "sus4")]
    Sus4,        // Suspended 4th
    #[serde(rename = "add9")]
    Add9,        // Add 9th
}

impl ChordQuality {
    /// Get intervals for this chord quality (in semitones from root)
    pub fn intervals(&self) -> Vec<u8> {
        match self {
            ChordQuality::Power5 => vec![0, 7],                    // Root, fifth
            ChordQuality::Major => vec![0, 4, 7],                  // Root, major third, fifth
            ChordQuality::Minor => vec![0, 3, 7],                  // Root, minor third, fifth
            ChordQuality::Sus2 => vec![0, 2, 7],                   // Root, second, fifth
            ChordQuality::Sus4 => vec![0, 5, 7],                   // Root, fourth, fifth
            ChordQuality::Add9 => vec![0, 4, 7, 14],               // Root, major third, fifth, ninth
        }
    }
}

/// Musical notes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Note {
    C, Cs, D, Ds, E, F, Fs, G, Gs, A, As, B,
}

impl Note {
    /// Get MIDI note number for this note in octave 4 (middle C = 60)
    pub fn to_midi(self, octave: i8) -> u8 {
        let base = match self {
            Note::C => 0,
            Note::Cs => 1,
            Note::D => 2,
            Note::Ds => 3,
            Note::E => 4,
            Note::F => 5,
            Note::Fs => 6,
            Note::G => 7,
            Note::Gs => 8,
            Note::A => 9,
            Note::As => 10,
            Note::B => 11,
        };
        ((octave + 4) * 12 + base as i8) as u8
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Note::C => "C",
            Note::Cs => "C#",
            Note::D => "D",
            Note::Ds => "D#",
            Note::E => "E",
            Note::F => "F",
            Note::Fs => "F#",
            Note::G => "G",
            Note::Gs => "G#",
            Note::A => "A",
            Note::As => "A#",
            Note::B => "B",
        }
    }

    /// Parse note from string
    pub fn from_str(s: &str) -> Option<Note> {
        match s.to_uppercase().as_str() {
            "C" => Some(Note::C),
            "C#" | "CS" | "DB" => Some(Note::Cs),
            "D" => Some(Note::D),
            "D#" | "DS" | "EB" => Some(Note::Ds),
            "E" => Some(Note::E),
            "F" => Some(Note::F),
            "F#" | "FS" | "GB" => Some(Note::Fs),
            "G" => Some(Note::G),
            "G#" | "GS" | "AB" => Some(Note::Gs),
            "A" => Some(Note::A),
            "A#" | "AS" | "BB" => Some(Note::As),
            "B" => Some(Note::B),
            _ => None,
        }
    }
}

/// Complete chord specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordSpec {
    pub root: Note,
    pub quality: ChordQuality,
    #[serde(default)]
    pub octave_offset: i8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voicing_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx_profile: Option<String>,
}

impl ChordSpec {
    pub fn new(root: Note, quality: ChordQuality) -> Self {
        Self {
            root,
            quality,
            octave_offset: 0,
            voicing_tag: None,
            fx_profile: None,
        }
    }

    /// Get MIDI notes for this chord
    pub fn to_midi_notes(&self, base_octave: i8) -> Vec<u8> {
        let root_note = self.root.to_midi(base_octave + self.octave_offset);
        self.quality.intervals()
            .into_iter()
            .map(|interval| root_note + interval)
            .collect()
    }

    /// Get display name for this chord
    pub fn display_name(&self) -> String {
        let quality_suffix = match self.quality {
            ChordQuality::Power5 => "5",
            ChordQuality::Major => "",
            ChordQuality::Minor => "m",
            ChordQuality::Sus2 => "sus2",
            ChordQuality::Sus4 => "sus4",
            ChordQuality::Add9 => "add9",
        };
        format!("{}{}", self.root.name(), quality_suffix)
    }
}

/// Global fret button to harmonic role mapping (constant across app)
pub const FRET_HARMONIC_MAPPING: &[(FretButton, HarmonicRole)] = &[
    (FretButton::Green, HarmonicRole::I),      // Home/root
    (FretButton::Red, HarmonicRole::IV),       // Movement
    (FretButton::Yellow, HarmonicRole::V),     // Drive/tension
    (FretButton::Blue, HarmonicRole::bVII),    // Anthem/punk color
    (FretButton::Orange, HarmonicRole::II),    // Tension/color (can be ii or vi based on genre)
];

/// Genre preset defining chord mappings and defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenrePreset {
    pub name: String,
    pub default_mode: Mode,
    pub default_key: Note,
    pub role_to_chord_quality: std::collections::HashMap<HarmonicRole, ChordQuality>,
    pub whammy_defaults: WhammyDefaults,
    pub sustain_defaults: SustainDefaults,
}

/// Whammy bar effect configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhammyDefaults {
    pub enabled: bool,
    pub pitch_bend_range_semitones: f32,
    pub vibrato_depth: f32,
    pub filter_cutoff_enabled: bool,
    pub smoothing_factor: f32,
}

impl Default for WhammyDefaults {
    fn default() -> Self {
        Self {
            enabled: true,
            pitch_bend_range_semitones: 1.0,
            vibrato_depth: 0.0,
            filter_cutoff_enabled: false,
            smoothing_factor: 0.8,
        }
    }
}

/// Sustain behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SustainDefaults {
    pub enabled: bool,
    pub release_time_ms: f32,
}

impl Default for SustainDefaults {
    fn default() -> Self {
        Self {
            enabled: true,
            release_time_ms: 500.0,
        }
    }
}

/// Pattern-level chord overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternChordOverride {
    pub fret_button: FretButton,
    pub row: FretRow,
    pub chord_spec: ChordSpec,
}

/// Fret row designation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FretRow {
    Main,
    Solo,
}