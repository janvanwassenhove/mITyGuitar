use mapping::MusicEvent;
use crate::synth::{FallbackSynth, InstrumentType as SynthInstrumentType};

#[cfg(feature = "soundfont")]
use crate::soundfont::SoundFontSynth;

enum SynthEngine {
    Fallback(FallbackSynth),
    #[cfg(feature = "soundfont")]
    SoundFont(SoundFontSynth),
}

/// Main audio engine that processes events and renders audio
pub struct AudioEngine {
    synth: SynthEngine,
    sample_rate: u32,
    release_multiplier: f32,
}

impl AudioEngine {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            synth: SynthEngine::Fallback(FallbackSynth::new(sample_rate)),
            sample_rate,
            release_multiplier: 1.0,
        }
    }
    
    /// Set the release time multiplier for all instruments
    pub fn set_release_multiplier(&mut self, multiplier: f32) {
        self.release_multiplier = multiplier;
        match &mut self.synth {
            SynthEngine::Fallback(synth) => synth.set_release_multiplier(multiplier),
            #[cfg(feature = "soundfont")]
            SynthEngine::SoundFont(_) => {
                // SoundFont uses its own envelope, can't modify easily
            }
        }
    }
    
    /// Enable or disable sustain mode
    pub fn set_sustain_enabled(&mut self, enabled: bool) {
        match &mut self.synth {
            SynthEngine::Fallback(synth) => synth.set_sustain_enabled(enabled),
            #[cfg(feature = "soundfont")]
            SynthEngine::SoundFont(_) => {
                // SoundFont doesn't support this yet
            }
        }
    }
    
    /// Set sustain release time in seconds
    pub fn set_sustain_release_time(&mut self, time_seconds: f32) {
        match &mut self.synth {
            SynthEngine::Fallback(synth) => synth.set_sustain_release_time(time_seconds),
            #[cfg(feature = "soundfont")]
            SynthEngine::SoundFont(_) => {
                // SoundFont doesn't support this yet
            }
        }
    }
    
    #[cfg(feature = "soundfont")]
    pub fn load_soundfont(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        log::info!("Loading soundfont: {:?}", path);
        let mut sf_synth = SoundFontSynth::new(self.sample_rate as f32)?;
        sf_synth.load_soundfont(path)?;
        self.synth = SynthEngine::SoundFont(sf_synth);
        log::info!("Soundfont loaded successfully");
        Ok(())
    }

    /// Switch to using the fallback synthesizer (for virtual instruments)
    pub fn use_fallback_synth(&mut self) -> anyhow::Result<()> {
        log::info!("Switching to fallback synth for virtual instrument");
        self.synth = SynthEngine::Fallback(FallbackSynth::new(self.sample_rate));
        log::info!("Switched to fallback synth successfully");
        Ok(())
    }

    /// Set virtual instrument type (when using fallback synth)
    pub fn set_virtual_instrument(&mut self, instrument: SynthInstrumentType) -> anyhow::Result<()> {
        log::info!("Setting virtual instrument: {:?}", instrument);
        match &mut self.synth {
            SynthEngine::Fallback(synth) => {
                synth.set_instrument(instrument);
                log::info!("Virtual instrument set successfully");
                Ok(())
            }
            #[cfg(feature = "soundfont")]
            SynthEngine::SoundFont(_) => {
                // Switch to fallback first, then set instrument
                self.use_fallback_synth()?;
                if let SynthEngine::Fallback(synth) = &mut self.synth {
                    synth.set_instrument(instrument);
                }
                Ok(())
            }
        }
    }

    /// Handle a music event (called in audio thread, must be RT-safe)
    pub fn handle_event(&mut self, event: MusicEvent) {
        match &mut self.synth {
            SynthEngine::Fallback(synth) => {
                match event {
                    MusicEvent::NoteOn { note, velocity } => synth.note_on(note, velocity),
                    MusicEvent::NoteOff { note } => synth.note_off(note),
                    MusicEvent::PitchBend(amount) => synth.set_pitch_bend(amount),
                    MusicEvent::PanicAllNotesOff => synth.all_notes_off(),
                    _ => {}
                }
            }
            #[cfg(feature = "soundfont")]
            SynthEngine::SoundFont(synth) => {
                match event {
                    MusicEvent::NoteOn { note, velocity } => synth.note_on(0, note, velocity),
                    MusicEvent::NoteOff { note } => synth.note_off(0, note),
                    MusicEvent::PitchBend(amount) => {
                        // Convert i16 (-8192..8191) to f32 (-1.0..1.0)
                        let normalized = (amount as f32) / 8192.0;
                        synth.set_pitch_bend(normalized);
                    },
                    MusicEvent::PanicAllNotesOff => synth.all_notes_off(),
                    _ => {}
                }
            }
        }
    }

    /// Render audio into the output buffer (RT-safe)
    pub fn render(&mut self, buffer: &mut [f32]) {
        match &mut self.synth {
            SynthEngine::Fallback(synth) => synth.render(buffer),
            #[cfg(feature = "soundfont")]
            SynthEngine::SoundFont(synth) => synth.render(buffer),
        }
    }

    /// Get count of active voices
    pub fn active_voice_count(&self) -> usize {
        match &self.synth {
            SynthEngine::Fallback(synth) => synth.active_voice_count(),
            #[cfg(feature = "soundfont")]
            SynthEngine::SoundFont(_) => 0, // TODO: implement for soundfont
        }
    }
}
