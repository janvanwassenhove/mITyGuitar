use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::harmonic::{
    FretButton, HarmonicRole, Genre, Mode, Note, ChordSpec, GenrePreset, 
    PatternChordOverride, FretRow, FRET_HARMONIC_MAPPING
};

/// Cached chord resolution result
type ChordMap = HashMap<FretButton, ChordSpec>;

/// Chord resolution with caching and pattern overrides
#[derive(Debug)]
pub struct ChordResolver {
    presets: HashMap<Genre, GenrePreset>,
    cache: Arc<RwLock<HashMap<ResolutionKey, ChordMap>>>,
}

/// Cache key for resolved chord maps
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ResolutionKey {
    genre: Genre,
    key_root: Note,
    mode: Mode,
    row: FretRow,
}

impl ChordResolver {
    pub fn new() -> Self {
        Self {
            presets: HashMap::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load preset for a genre
    pub fn load_preset(&mut self, genre: Genre, preset: GenrePreset) {
        self.presets.insert(genre, preset);
    }

    /// Resolve chord map for given parameters
    pub fn resolve_chord_map(
        &self, 
        genre: Genre, 
        key_root: Option<Note>, 
        mode: Option<Mode>,
        row: FretRow,
        overrides: &[PatternChordOverride]
    ) -> Result<ChordMap, String> {
        let preset = self.presets.get(&genre)
            .ok_or_else(|| format!("No preset found for genre: {}", genre.name()))?;

        let key_root = key_root.unwrap_or(preset.default_key);
        let mode = mode.unwrap_or(preset.default_mode);
        
        let cache_key = ResolutionKey {
            genre,
            key_root,
            mode,
            row,
        };

        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(self.apply_overrides(cached.clone(), overrides, row));
            }
        }

        // Resolve chords
        let mut chord_map = HashMap::new();
        
        for &(fret_button, harmonic_role) in FRET_HARMONIC_MAPPING {
            if let Some(chord_spec) = self.resolve_chord_for_role(
                harmonic_role, 
                key_root, 
                mode, 
                preset,
                row
            ) {
                chord_map.insert(fret_button, chord_spec);
            }
        }

        // Cache the result
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(cache_key, chord_map.clone());
        }

        Ok(self.apply_overrides(chord_map, overrides, row))
    }

    /// Resolve a chord for a specific harmonic role
    fn resolve_chord_for_role(
        &self,
        role: HarmonicRole,
        key_root: Note,
        mode: Mode,
        preset: &GenrePreset,
        row: FretRow,
    ) -> Option<ChordSpec> {
        let quality = preset.role_to_chord_quality.get(&role)?;
        let chord_root = self.get_chord_root_for_role(role, key_root, mode);
        
        let mut chord_spec = ChordSpec::new(chord_root, *quality);
        
        // Adjust octave for solo row
        if row == FretRow::Solo {
            chord_spec.octave_offset = 1;
        }
        
        Some(chord_spec)
    }

    /// Get the chord root note for a harmonic role in a given key
    fn get_chord_root_for_role(&self, role: HarmonicRole, key_root: Note, mode: Mode) -> Note {
        let root_semitones = key_root.to_midi(0);
        
        let interval = match (role, mode) {
            (HarmonicRole::I, _) => 0,           // Root
            (HarmonicRole::IV, Mode::Major) => 5,   // Perfect fourth
            (HarmonicRole::IV, Mode::Minor) => 5,   // Perfect fourth
            (HarmonicRole::V, _) => 7,           // Perfect fifth
            (HarmonicRole::bVII, _) => 10,       // Minor seventh
            (HarmonicRole::II, Mode::Major) => 2,   // Major second (ii in major)
            (HarmonicRole::II, Mode::Minor) => 2,   // Major second (ii in minor)
            (HarmonicRole::VI, Mode::Major) => 9,   // Major sixth (vi in major)
            (HarmonicRole::VI, Mode::Minor) => 8,   // Minor sixth (VI in minor)
        };
        
        let target_note = (root_semitones + interval) % 12;
        match target_note {
            0 => Note::C,
            1 => Note::Cs,
            2 => Note::D,
            3 => Note::Ds,
            4 => Note::E,
            5 => Note::F,
            6 => Note::Fs,
            7 => Note::G,
            8 => Note::Gs,
            9 => Note::A,
            10 => Note::As,
            11 => Note::B,
            _ => unreachable!(),
        }
    }

    /// Apply pattern overrides to the resolved chord map
    fn apply_overrides(
        &self,
        mut chord_map: ChordMap,
        overrides: &[PatternChordOverride],
        row: FretRow,
    ) -> ChordMap {
        for override_spec in overrides {
            if override_spec.row == row {
                chord_map.insert(override_spec.fret_button, override_spec.chord_spec.clone());
            }
        }
        chord_map
    }

    /// Clear resolution cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Get preset for a genre
    pub fn get_preset(&self, genre: Genre) -> Option<&GenrePreset> {
        self.presets.get(&genre)
    }
}

impl Default for ChordResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harmonic::*;

    fn create_test_preset() -> GenrePreset {
        let mut role_to_chord_quality = HashMap::new();
        role_to_chord_quality.insert(HarmonicRole::I, ChordQuality::Power5);
        role_to_chord_quality.insert(HarmonicRole::IV, ChordQuality::Power5);
        role_to_chord_quality.insert(HarmonicRole::V, ChordQuality::Power5);
        role_to_chord_quality.insert(HarmonicRole::bVII, ChordQuality::Power5);
        role_to_chord_quality.insert(HarmonicRole::II, ChordQuality::Power5);
        
        GenrePreset {
            name: "Test".to_string(),
            default_mode: Mode::Major,
            default_key: Note::E,
            role_to_chord_quality,
            whammy_defaults: WhammyDefaults::default(),
            sustain_defaults: SustainDefaults::default(),
        }
    }

    #[test]
    fn test_chord_resolution() {
        let mut resolver = ChordResolver::new();
        let preset = create_test_preset();
        resolver.load_preset(Genre::Punk, preset);

        let chord_map = resolver.resolve_chord_map(
            Genre::Punk,
            Some(Note::E),
            Some(Mode::Major),
            FretRow::Main,
            &[]
        ).unwrap();

        assert_eq!(chord_map.len(), 5);
        
        // Verify GREEN maps to I (E)
        let green_chord = chord_map.get(&FretButton::Green).unwrap();
        assert_eq!(green_chord.root, Note::E);
        assert_eq!(green_chord.quality, ChordQuality::Power5);
    }

    #[test]
    fn test_pattern_overrides() {
        let mut resolver = ChordResolver::new();
        let preset = create_test_preset();
        resolver.load_preset(Genre::Punk, preset);

        let override_spec = PatternChordOverride {
            fret_button: FretButton::Green,
            row: FretRow::Main,
            chord_spec: ChordSpec::new(Note::A, ChordQuality::Minor),
        };

        let chord_map = resolver.resolve_chord_map(
            Genre::Punk,
            Some(Note::E),
            Some(Mode::Major),
            FretRow::Main,
            &[override_spec]
        ).unwrap();

        // Verify override is applied
        let green_chord = chord_map.get(&FretButton::Green).unwrap();
        assert_eq!(green_chord.root, Note::A);
        assert_eq!(green_chord.quality, ChordQuality::Minor);
    }
}