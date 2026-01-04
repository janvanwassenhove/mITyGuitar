// SoundFont scanning and management
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};
use crate::synth::InstrumentType as SynthInstrumentType;

#[cfg(feature = "soundfont")]
use oxisynth::{SoundFont, Synth, SynthDescriptor};
use std::io::BufReader;
use std::fs::File;

/// Information about an instrument (SoundFont or Virtual)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentInfo {
    pub name: String,
    pub path: Option<PathBuf>,  // None for virtual instruments
    pub size_bytes: Option<u64>, // None for virtual instruments
    pub instrument_type: InstrumentType,
}

impl InstrumentInfo {
    /// Get the synthesizer instrument type for virtual instruments
    pub fn get_synth_instrument_type(&self) -> Option<SynthInstrumentType> {
        if self.instrument_type != InstrumentType::Virtual {
            return None;
        }
        
        match self.name.as_str() {
            "Clean Electric Guitar" => Some(SynthInstrumentType::CleanElectricGuitar),
            "Distorted Guitar" => Some(SynthInstrumentType::DistortedGuitar),
            "Acoustic Guitar" => Some(SynthInstrumentType::AcousticGuitar),
            "Classical Guitar" => Some(SynthInstrumentType::ClassicalGuitar),
            "Electric Bass" => Some(SynthInstrumentType::ElectricBass),
            "Acoustic Bass" => Some(SynthInstrumentType::AcousticBass),
            "Piano" => Some(SynthInstrumentType::Piano),
            "Organ" => Some(SynthInstrumentType::Organ),
            "Strings" => Some(SynthInstrumentType::Strings),
            "Synth Lead" => Some(SynthInstrumentType::SynthLead),
            "Synth Pad" => Some(SynthInstrumentType::SynthPad),
            "Brass Section" => Some(SynthInstrumentType::BrassSection),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstrumentType {
    SoundFont,
    Virtual,
}

/// Legacy SoundFont-only structure for backwards compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundFontInfo {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
}

/// SoundFont manager (now supports virtual instruments too)
#[derive(Debug)]
pub struct SoundFontManager {
    soundfonts: Vec<SoundFontInfo>,
    instruments: Vec<InstrumentInfo>, // Combined list
    soundfont_dir: PathBuf,
    additional_dirs: Vec<PathBuf>, // Additional directories to scan
}

impl SoundFontManager {
    /// Create a new SoundFont manager for the given directory
    pub fn new<P: AsRef<Path>>(soundfont_dir: P) -> Result<Self> {
        let soundfont_dir = soundfont_dir.as_ref().to_path_buf();
        let mut manager = Self {
            soundfonts: Vec::new(),
            instruments: Vec::new(),
            soundfont_dir,
            additional_dirs: Vec::new(),
        };
        manager.scan()?;
        manager.add_virtual_instruments();
        Ok(manager)
    }

    /// Create a new SoundFont manager with multiple directories
    pub fn with_additional_dirs<P: AsRef<Path>>(soundfont_dir: P, additional_dirs: Vec<PathBuf>) -> Result<Self> {
        let soundfont_dir = soundfont_dir.as_ref().to_path_buf();
        let mut manager = Self {
            soundfonts: Vec::new(),
            instruments: Vec::new(),
            soundfont_dir,
            additional_dirs,
        };
        manager.scan()?;
        manager.add_virtual_instruments();
        Ok(manager)
    }

    /// Add predefined virtual instruments
    fn add_virtual_instruments(&mut self) {
        let virtual_instruments = vec![
            "Clean Electric Guitar",
            "Distorted Guitar", 
            "Acoustic Guitar",
            "Classical Guitar",
            "Electric Bass",
            "Acoustic Bass",
            "Piano",
            "Organ",
            "Strings",
            "Synth Lead",
            "Synth Pad",
            "Brass Section",
        ];

        for name in virtual_instruments {
            self.instruments.push(InstrumentInfo {
                name: name.to_string(),
                path: None,
                size_bytes: None,
                instrument_type: InstrumentType::Virtual,
            });
        }
    }

    /// Scan the soundfont directory for .sf2 files
    pub fn scan(&mut self) -> Result<()> {
        self.soundfonts.clear();
        // Clear only soundfont instruments, keep virtual ones
        self.instruments.retain(|i| i.instrument_type == InstrumentType::Virtual);

        // Collect all directories to scan
        let mut dirs_to_scan = vec![self.soundfont_dir.clone()];
        dirs_to_scan.extend(self.additional_dirs.iter().cloned());

        // Scan each directory
        for dir in dirs_to_scan {
            if !dir.exists() {
                log::warn!("SoundFont directory does not exist: {:?}", dir);
                continue;
            }

            log::info!("Scanning SoundFont directory: {:?}", dir);

            for entry in fs::read_dir(&dir)
                .context(format!("Failed to read soundfont directory: {:?}", dir))?
            {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() && path.extension().and_then(|e| e.to_str()).map(|s| s.eq_ignore_ascii_case("sf2")).unwrap_or(false) {
                    let metadata = fs::metadata(&path)?;
                    let name = path.file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    // Add to both lists for compatibility
                    self.soundfonts.push(SoundFontInfo {
                        name: name.clone(),
                        path: path.clone(),
                        size_bytes: metadata.len(),
                    });

                    self.instruments.push(InstrumentInfo {
                        name: name.clone(),
                        path: Some(path.clone()),
                        size_bytes: Some(metadata.len()),
                        instrument_type: InstrumentType::SoundFont,
                    });

                    log::info!("Found SoundFont: {:?}", path);
                }
            }
        }

        log::info!("Found {} SoundFont files across all directories", self.soundfonts.len());
        Ok(())
    }

    /// Get list of all instruments (SoundFonts + Virtual)
    pub fn list_instruments(&self) -> &[InstrumentInfo] {
        &self.instruments
    }

    /// Get list of available soundfonts
    pub fn list(&self) -> &[SoundFontInfo] {
        &self.soundfonts
    }

    /// Get a soundfont by name (legacy compatibility)
    pub fn get_by_name(&self, name: &str) -> Option<&SoundFontInfo> {
        self.soundfonts.iter().find(|sf| sf.name == name)
    }

    /// Get an instrument by name (SoundFont or Virtual)
    pub fn get_instrument_by_name(&self, name: &str) -> Option<&InstrumentInfo> {
        self.instruments.iter().find(|inst| inst.name == name)
    }

    /// Get the first guitar-like soundfont (contains "guitar" in name)
    pub fn get_default_guitar(&self) -> Option<&SoundFontInfo> {
        self.soundfonts.iter()
            .find(|sf| sf.name.to_lowercase().contains("guitar"))
            .or_else(|| self.soundfonts.first())
    }

    /// Get the first guitar-like instrument (SoundFont or Virtual)
    pub fn get_default_guitar_instrument(&self) -> Option<&InstrumentInfo> {
        self.instruments.iter()
            .find(|inst| inst.name.to_lowercase().contains("guitar"))
            .or_else(|| self.instruments.first())
    }
}

#[cfg(feature = "soundfont")]
/// SoundFont-based synthesizer
pub struct SoundFontSynth {
    synth: Synth,
    active_soundfont: Option<String>,
}

#[cfg(feature = "soundfont")]
impl SoundFontSynth {
    /// Create a new SoundFont synthesizer
    pub fn new(sample_rate: f32) -> Result<Self> {
        let settings = SynthDescriptor {
            sample_rate: sample_rate,
            gain: 0.5,
            ..Default::default()
        };

        let synth = Synth::new(settings)
            .context("Failed to create synthesizer")?;

        Ok(Self {
            synth,
            active_soundfont: None,
        })
    }

    /// Load a SoundFont file
    pub fn load_soundfont<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        log::info!("Loading SoundFont: {:?}", path);

        let file = File::open(path)
            .context("Failed to open SoundFont file")?;
        let mut reader = BufReader::new(file);

        let soundfont = SoundFont::load(&mut reader)
            .context("Failed to load SoundFont")?;

        self.synth.add_font(soundfont, true);
        
        self.active_soundfont = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        log::info!("SoundFont loaded successfully");
        Ok(())
    }

    /// Get the currently active soundfont name
    pub fn active_soundfont(&self) -> Option<&str> {
        self.active_soundfont.as_deref()
    }

    /// Send note on event
    pub fn note_on(&mut self, channel: u8, key: u8, velocity: u8) {
        if let Err(e) = self.synth.send_event(oxisynth::MidiEvent::NoteOn {
            channel,
            key,
            vel: velocity,
        }) {
            log::error!("Failed to send note on: {}", e);
        }
    }

    /// Send note off event
    pub fn note_off(&mut self, channel: u8, key: u8) {
        if let Err(e) = self.synth.send_event(oxisynth::MidiEvent::NoteOff {
            channel,
            key,
        }) {
            log::error!("Failed to send note off: {}", e);
        }
    }

    /// Send program change (instrument selection)
    pub fn program_change(&mut self, channel: u8, program: u8) {
        if let Err(e) = self.synth.send_event(oxisynth::MidiEvent::ProgramChange {
            channel,
            program_id: program,
        }) {
            log::error!("Failed to send program change: {}", e);
        }
    }

    /// Send control change
    pub fn control_change(&mut self, channel: u8, control: u8, value: u8) {
        if let Err(e) = self.synth.send_event(oxisynth::MidiEvent::ControlChange {
            channel,
            ctrl: control,
            value,
        }) {
            log::error!("Failed to send control change: {}", e);
        }
    }

    /// Set pitch bend
    pub fn set_pitch_bend(&mut self, amount: f32) {
        // Convert -1.0..1.0 to 0..16383 (14-bit MIDI pitch bend)
        let value = ((amount + 1.0) * 8191.5) as u16;
        if let Err(e) = self.synth.send_event(oxisynth::MidiEvent::PitchBend {
            channel: 0,
            value,
        }) {
            log::error!("Failed to send pitch bend: {}", e);
        }
    }

    /// Stop all notes
    pub fn all_notes_off(&mut self) {
        // Send note off for all possible notes on channel 0
        for note in 0..128 {
            self.note_off(0, note);
        }
    }

    /// Render audio samples
    pub fn render(&mut self, buffer: &mut [f32]) {
        // oxisynth renders into a tuple of slices (left, right)
        let frames = buffer.len() / 2;
        
        // Split buffer into two halves for left/right channels
        let (left_buf, right_buf) = buffer.split_at_mut(frames);

        // Create slices for rendering
        self.synth.write((left_buf, right_buf));

        // The audio is now written, but we need to interleave it
        // We'll use a temporary buffer to avoid borrowing issues
        let mut temp = vec![0.0f32; buffer.len()];
        for i in 0..frames {
            temp[i * 2] = buffer[i];
            temp[i * 2 + 1] = buffer[frames + i];
        }
        buffer.copy_from_slice(&temp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soundfont_manager() {
        // This test requires the soundfont directory to exist
        if let Ok(manager) = SoundFontManager::new("../../soundfont") {
            assert!(manager.list().len() > 0, "Expected to find some soundfonts");
            
            let default = manager.get_default_guitar();
            assert!(default.is_some(), "Expected to find a default guitar soundfont");
        }
    }
}
