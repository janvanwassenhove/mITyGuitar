use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result, Context};

use crate::harmonic::{
    Genre, GenrePreset, HarmonicRole, ChordQuality,
    WhammyDefaults, SustainDefaults
};
use crate::resolution::ChordResolver;

/// Preset loader for genre-based chord mappings
#[derive(Debug)]
pub struct PresetLoader {
    assets_path: std::path::PathBuf,
}

impl PresetLoader {
    pub fn new<P: AsRef<Path>>(assets_path: P) -> Self {
        Self {
            assets_path: assets_path.as_ref().to_path_buf(),
        }
    }

    /// Load all genre presets from JSON files
    pub async fn load_all_presets(&self) -> Result<ChordResolver> {
        let mut resolver = ChordResolver::new();
        
        for genre in Genre::all() {
            if let Ok(preset) = self.load_preset(*genre).await {
                resolver.load_preset(*genre, preset);
            } else {
                log::warn!("Failed to load preset for {}, using default", genre.name());
                resolver.load_preset(*genre, Self::create_default_preset(*genre));
            }
        }

        Ok(resolver)
    }

    /// Load a specific genre preset from JSON file
    async fn load_preset(&self, genre: Genre) -> Result<GenrePreset> {
        let filename = format!("{}.json", genre.name().to_lowercase());
        let preset_path = self.assets_path.join("chordmaps").join(filename);
        
        let content = tokio::fs::read_to_string(&preset_path)
            .await
            .with_context(|| format!("Failed to read preset file: {}", preset_path.display()))?;
            
        let preset: GenrePreset = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse preset JSON for {}", genre.name()))?;
            
        Ok(preset)
    }

    /// Save a preset to JSON file
    pub async fn save_preset(&self, genre: Genre, preset: &GenrePreset) -> Result<()> {
        let chordmaps_dir = self.assets_path.join("chordmaps");
        tokio::fs::create_dir_all(&chordmaps_dir).await?;
        
        let filename = format!("{}.json", genre.name().to_lowercase());
        let preset_path = chordmaps_dir.join(filename);
        
        let json = serde_json::to_string_pretty(preset)
            .with_context(|| format!("Failed to serialize preset for {}", genre.name()))?;
            
        tokio::fs::write(&preset_path, json)
            .await
            .with_context(|| format!("Failed to write preset file: {}", preset_path.display()))?;
            
        Ok(())
    }

    /// Create default preset for a genre (fallback)
    fn create_default_preset(genre: Genre) -> GenrePreset {
        let mut role_to_chord_quality = HashMap::new();
        
        match genre {
            Genre::Punk => {
                // All power chords
                role_to_chord_quality.insert(HarmonicRole::I, ChordQuality::Power5);
                role_to_chord_quality.insert(HarmonicRole::IV, ChordQuality::Power5);
                role_to_chord_quality.insert(HarmonicRole::V, ChordQuality::Power5);
                role_to_chord_quality.insert(HarmonicRole::bVII, ChordQuality::Power5);
                role_to_chord_quality.insert(HarmonicRole::VI, ChordQuality::Power5);
            },
            Genre::Edm => {
                // Minor with sus colors
                role_to_chord_quality.insert(HarmonicRole::I, ChordQuality::Minor);
                role_to_chord_quality.insert(HarmonicRole::IV, ChordQuality::Minor);
                role_to_chord_quality.insert(HarmonicRole::V, ChordQuality::Sus2);
                role_to_chord_quality.insert(HarmonicRole::bVII, ChordQuality::Major);
                role_to_chord_quality.insert(HarmonicRole::VI, ChordQuality::Major);
            },
            Genre::Rock => {
                // Classic rock chords
                role_to_chord_quality.insert(HarmonicRole::I, ChordQuality::Major);
                role_to_chord_quality.insert(HarmonicRole::IV, ChordQuality::Major);
                role_to_chord_quality.insert(HarmonicRole::V, ChordQuality::Major);
                role_to_chord_quality.insert(HarmonicRole::bVII, ChordQuality::Major);
                role_to_chord_quality.insert(HarmonicRole::II, ChordQuality::Minor);
            },
            Genre::Pop => {
                // Bright voicings with color
                role_to_chord_quality.insert(HarmonicRole::I, ChordQuality::Add9);
                role_to_chord_quality.insert(HarmonicRole::IV, ChordQuality::Add9);
                role_to_chord_quality.insert(HarmonicRole::V, ChordQuality::Sus4);
                role_to_chord_quality.insert(HarmonicRole::bVII, ChordQuality::Major);
                role_to_chord_quality.insert(HarmonicRole::VI, ChordQuality::Minor);
            },
            Genre::Folk => {
                // Open, sus flavor
                role_to_chord_quality.insert(HarmonicRole::I, ChordQuality::Major);
                role_to_chord_quality.insert(HarmonicRole::IV, ChordQuality::Major);
                role_to_chord_quality.insert(HarmonicRole::V, ChordQuality::Sus4);
                role_to_chord_quality.insert(HarmonicRole::bVII, ChordQuality::Major);
                role_to_chord_quality.insert(HarmonicRole::VI, ChordQuality::Minor);
            },
            Genre::Metal => {
                // Dark power chords
                role_to_chord_quality.insert(HarmonicRole::I, ChordQuality::Power5);
                role_to_chord_quality.insert(HarmonicRole::IV, ChordQuality::Power5);
                role_to_chord_quality.insert(HarmonicRole::V, ChordQuality::Power5);
                role_to_chord_quality.insert(HarmonicRole::bVII, ChordQuality::Power5);
                role_to_chord_quality.insert(HarmonicRole::II, ChordQuality::Power5);
            },
        }

        let whammy_defaults = match genre {
            Genre::Punk => WhammyDefaults {
                enabled: true,
                pitch_bend_range_semitones: 1.0,
                vibrato_depth: 0.0,
                filter_cutoff_enabled: false,
                smoothing_factor: 0.8,
            },
            Genre::Rock => WhammyDefaults {
                enabled: true,
                pitch_bend_range_semitones: 2.0,
                vibrato_depth: 0.1,
                filter_cutoff_enabled: true,
                smoothing_factor: 0.7,
            },
            Genre::Pop => WhammyDefaults {
                enabled: true,
                pitch_bend_range_semitones: 0.5,
                vibrato_depth: 0.05,
                filter_cutoff_enabled: false,
                smoothing_factor: 0.9,
            },
            Genre::Folk => WhammyDefaults {
                enabled: true,
                pitch_bend_range_semitones: 0.3,
                vibrato_depth: 0.2,
                filter_cutoff_enabled: false,
                smoothing_factor: 0.85,
            },
            Genre::Edm => WhammyDefaults {
                enabled: true,
                pitch_bend_range_semitones: 3.0,
                vibrato_depth: 0.0,
                filter_cutoff_enabled: true,
                smoothing_factor: 0.6,
            },
            Genre::Metal => WhammyDefaults {
                enabled: true,
                pitch_bend_range_semitones: 1.5,
                vibrato_depth: 0.0,
                filter_cutoff_enabled: false,
                smoothing_factor: 0.75,
            },
        };

        GenrePreset {
            name: genre.name().to_string(),
            default_mode: genre.default_mode(),
            default_key: genre.default_key_root(),
            role_to_chord_quality,
            whammy_defaults,
            sustain_defaults: SustainDefaults::default(),
        }
    }

    /// Initialize default preset files if they don't exist
    pub async fn init_default_presets(&self) -> Result<()> {
        let chordmaps_dir = self.assets_path.join("chordmaps");
        tokio::fs::create_dir_all(&chordmaps_dir).await?;
        
        for genre in Genre::all() {
            let filename = format!("{}.json", genre.name().to_lowercase());
            let preset_path = chordmaps_dir.join(filename);
            
            if !preset_path.exists() {
                let preset = Self::create_default_preset(*genre);
                self.save_preset(*genre, &preset).await?;
                log::info!("Created default preset for {}", genre.name());
            }
        }
        
        Ok(())
    }
}

impl Default for PresetLoader {
    fn default() -> Self {
        Self::new("./assets")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_preset_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let loader = PresetLoader::new(temp_dir.path());
        
        let preset = PresetLoader::create_default_preset(Genre::Punk);
        loader.save_preset(Genre::Punk, &preset).await.unwrap();
        
        let loaded_preset = loader.load_preset(Genre::Punk).await.unwrap();
        assert_eq!(preset.name, loaded_preset.name);
        assert_eq!(preset.default_key, loaded_preset.default_key);
    }

    #[tokio::test]
    async fn test_init_default_presets() {
        let temp_dir = TempDir::new().unwrap();
        let loader = PresetLoader::new(temp_dir.path());
        
        loader.init_default_presets().await.unwrap();
        
        // Check that all preset files exist
        for genre in Genre::all() {
            let filename = format!("{}.json", genre.name().to_lowercase());
            let preset_path = temp_dir.path().join("chordmaps").join(filename);
            assert!(preset_path.exists());
        }
    }
}