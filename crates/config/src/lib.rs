use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = "mityguitar_config.json";
const CONFIG_VERSION: u32 = 1;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    pub controller: ControllerConfig,
    pub audio: AudioConfig,
    pub soundfonts: SoundFontConfig,
    pub mapping: MappingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    pub device_id: String,
    pub simulator_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub backend: String,
    #[serde(default = "default_release_multiplier")]
    pub release_time_multiplier: f32,
    #[serde(default)]
    pub sustain_enabled: bool,
    #[serde(default = "default_sustain_release_time")]
    pub sustain_release_time_ms: f32,
}

fn default_release_multiplier() -> f32 {
    1.0
}

fn default_sustain_release_time() -> f32 {
    500.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundFontConfig {
    pub current: Option<String>,
    pub preset: PresetInfo,
    pub recent: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetInfo {
    pub bank: u32,
    pub program: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfig {
    pub genre: String,
    pub pattern_index: usize,
    pub whammy_mode: String,
    pub fx_switch_mode: String,
    pub tilt_mode: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            controller: ControllerConfig {
                device_id: "auto".to_string(),
                simulator_mode: true, // Default to simulator for development
            },
            audio: AudioConfig {
                sample_rate: 48000,
                buffer_size: 256,
                backend: "fallback".to_string(),
                release_time_multiplier: 1.0,
                sustain_enabled: false,
                sustain_release_time_ms: 500.0,
            },
            soundfonts: SoundFontConfig {
                current: Some("Electric_guitar.sf2".to_string()),
                preset: PresetInfo {
                    bank: 0,
                    program: 0,
                },
                recent: Vec::new(),
            },
            mapping: MappingConfig {
                genre: "rock".to_string(),
                pattern_index: 0,
                whammy_mode: "pitch_bend".to_string(),
                fx_switch_mode: "effects".to_string(),
                tilt_mode: "filter_cutoff".to_string(),
            },
        }
    }
}

impl AppConfig {
    /// Load config from disk, or create default if not found
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        
        if path.exists() {
            let data = fs::read_to_string(&path)
                .context("Failed to read config file")?;
            let mut config: AppConfig = serde_json::from_str(&data)
                .context("Failed to parse config file")?;
            
            // Migrate if needed
            if config.version < CONFIG_VERSION {
                config = Self::migrate(config)?;
            }
            
            Ok(config)
        } else {
            // Create default config
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let data = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(&path, data)
            .context("Failed to write config file")?;
        
        Ok(())
    }

    /// Get the path to the config file
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?;
        
        Ok(config_dir.join("mityguitar").join(CONFIG_FILE_NAME))
    }

    /// Migrate from older config versions
    fn migrate(mut config: AppConfig) -> Result<Self> {
        // Handle version migrations here
        config.version = CONFIG_VERSION;
        Ok(config)
    }

    /// Add a SoundFont to recent list
    pub fn add_recent_soundfont(&mut self, path: String) {
        // Remove if already present
        self.soundfonts.recent.retain(|p| p != &path);
        
        // Add to front
        self.soundfonts.recent.insert(0, path);
        
        // Keep only last 10
        self.soundfonts.recent.truncate(10);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.audio.sample_rate, 48000);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, config.version);
    }

    #[test]
    fn test_recent_soundfonts() {
        let mut config = AppConfig::default();
        config.add_recent_soundfont("test1.sf2".to_string());
        config.add_recent_soundfont("test2.sf2".to_string());
        
        assert_eq!(config.soundfonts.recent.len(), 2);
        assert_eq!(config.soundfonts.recent[0], "test2.sf2");
    }
}
