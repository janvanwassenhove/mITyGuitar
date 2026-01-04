//! Fallback polyphonic synthesizer
//! Simple but musical synth that works without external dependencies

const MAX_VOICES: usize = 16;
const ATTACK_TIME: f32 = 0.01;  // 10ms attack
const RELEASE_TIME: f32 = 0.3;  // 300ms release

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstrumentType {
    CleanElectricGuitar,
    DistortedGuitar,
    AcousticGuitar,
    ClassicalGuitar,
    ElectricBass,
    AcousticBass,
    Piano,
    Organ,
    Strings,
    SynthLead,
    SynthPad,
    BrassSection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WaveType {
    Sine,
    Saw,
    Square,
    Triangle,
    Noise,
}

#[derive(Debug, Clone, Copy)]
struct InstrumentSettings {
    wave_type: WaveType,
    attack_time: f32,
    release_time: f32,
    filter_cutoff: f32,
    resonance: f32,
    distortion: f32,
    volume: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EnvelopeStage {
    Off,
    Attack,
    Sustain,
    Release,
}

struct Voice {
    note: u8,
    frequency: f32,
    phase: f32,
    envelope_stage: EnvelopeStage,
    envelope_value: f32,
    velocity: f32,
    settings: InstrumentSettings,
    filter_state: f32,
    sustain_enabled: bool,
    sustain_release_time: f32,
}

impl Voice {
    fn new() -> Self {
        Self {
            note: 0,
            frequency: 0.0,
            phase: 0.0,
            envelope_stage: EnvelopeStage::Off,
            envelope_value: 0.0,
            velocity: 0.0,
            settings: get_instrument_settings(InstrumentType::CleanElectricGuitar),
            filter_state: 0.0,
            sustain_enabled: false,
            sustain_release_time: 0.5,
        }
    }

    fn is_active(&self) -> bool {
        self.envelope_stage != EnvelopeStage::Off
    }

    fn trigger(&mut self, note: u8, velocity: u8, _sample_rate: u32, settings: InstrumentSettings, sustain_enabled: bool, sustain_release_time: f32) {
        self.note = note;
        self.velocity = velocity as f32 / 127.0;
        self.frequency = midi_to_frequency(note);
        self.phase = 0.0;
        self.envelope_stage = EnvelopeStage::Attack;
        self.envelope_value = 0.0;
        self.settings = settings;
        self.filter_state = 0.0;
        self.sustain_enabled = sustain_enabled;
        self.sustain_release_time = sustain_release_time;
    }

    fn release(&mut self) {
        if self.envelope_stage == EnvelopeStage::Attack || self.envelope_stage == EnvelopeStage::Sustain {
            self.envelope_stage = EnvelopeStage::Release;
        }
    }

    fn render_sample(&mut self, sample_rate: u32, pitch_bend: f32) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        // Update envelope with instrument-specific timing
        let envelope_delta = 1.0 / sample_rate as f32;
        match self.envelope_stage {
            EnvelopeStage::Attack => {
                self.envelope_value += envelope_delta / self.settings.attack_time;
                if self.envelope_value >= 1.0 {
                    self.envelope_value = 1.0;
                    self.envelope_stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => {
                // Hold at 1.0
            }
            EnvelopeStage::Release => {
                // Use sustain release time if sustain is enabled, otherwise use instrument release
                let release_time = if self.sustain_enabled {
                    self.sustain_release_time
                } else {
                    self.settings.release_time
                };
                self.envelope_value -= envelope_delta / release_time;
                if self.envelope_value <= 0.0 {
                    self.envelope_value = 0.0;
                    self.envelope_stage = EnvelopeStage::Off;
                }
            }
            EnvelopeStage::Off => return 0.0,
        }

        // Apply pitch bend (in semitones)
        let bent_frequency = self.frequency * 2.0_f32.powf(pitch_bend / 12.0);

        // Generate waveform based on instrument type
        let phase_increment = bent_frequency / sample_rate as f32;
        self.phase += phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        let mut sample = match self.settings.wave_type {
            WaveType::Sine => (self.phase * 2.0 * std::f32::consts::PI).sin(),
            WaveType::Saw => (self.phase * 2.0) - 1.0,
            WaveType::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },
            WaveType::Triangle => {
                if self.phase < 0.5 {
                    (self.phase * 4.0) - 1.0
                } else {
                    3.0 - (self.phase * 4.0)
                }
            }
            WaveType::Noise => (fastrand::f32() * 2.0) - 1.0,
        };

        // Apply simple low-pass filter
        let cutoff = self.settings.filter_cutoff;
        self.filter_state += (sample - self.filter_state) * cutoff;
        sample = self.filter_state;

        // Apply distortion if specified
        if self.settings.distortion > 0.0 {
            let gain = 1.0 + self.settings.distortion * 10.0;
            sample = (sample * gain).tanh() / gain.tanh();
        }

        // Apply envelope, velocity, and volume
        sample * self.envelope_value * self.velocity * self.settings.volume
    }
}

/// Simple polyphonic synthesizer
pub struct FallbackSynth {
    voices: [Voice; MAX_VOICES],
    sample_rate: u32,
    pitch_bend: f32, // In semitones (-2 to +2)
    current_instrument: InstrumentType,
    release_multiplier: f32, // Multiplier for all release times
    sustain_enabled: bool, // Whether sustain mode is enabled
    sustain_release_time: f32, // Custom release time for sustain mode (in seconds)
}

impl FallbackSynth {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            voices: std::array::from_fn(|_| Voice::new()),
            sample_rate,
            pitch_bend: 0.0,
            current_instrument: InstrumentType::CleanElectricGuitar,
            release_multiplier: 1.0,
            sustain_enabled: false,
            sustain_release_time: 0.5,
        }
    }

    pub fn set_instrument(&mut self, instrument: InstrumentType) {
        self.current_instrument = instrument;
        // Stop all currently playing voices when switching instruments
        self.all_notes_off();
    }
    
    /// Set the release time multiplier
    pub fn set_release_multiplier(&mut self, multiplier: f32) {
        self.release_multiplier = multiplier.clamp(0.1, 10.0); // Limit to reasonable range
    }
    
    /// Enable or disable sustain mode
    pub fn set_sustain_enabled(&mut self, enabled: bool) {
        self.sustain_enabled = enabled;
    }
    
    /// Set the sustain release time in seconds
    pub fn set_sustain_release_time(&mut self, time_seconds: f32) {
        self.sustain_release_time = time_seconds.clamp(0.05, 10.0); // 50ms to 10s
    }

    pub fn note_on(&mut self, note: u8, velocity: u8) {
        // Find a free voice or steal the oldest
        let sample_rate = self.sample_rate;
        let mut settings = get_instrument_settings(self.current_instrument);
        // Apply release multiplier (only when sustain is disabled)
        if !self.sustain_enabled {
            settings.release_time *= self.release_multiplier;
        }
        
        // Store sustain settings to avoid borrowing issues
        let sustain_enabled = self.sustain_enabled;
        let sustain_release_time = self.sustain_release_time;
        
        if let Some(voice) = self.find_free_voice() {
            voice.trigger(note, velocity, sample_rate, settings, sustain_enabled, sustain_release_time);
        } else if let Some(voice) = self.voices.first_mut() {
            // Voice stealing: take the first voice
            voice.trigger(note, velocity, sample_rate, settings, sustain_enabled, sustain_release_time);
        }
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.note == note && voice.is_active() {
                voice.release();
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.release();
        }
    }

    pub fn set_pitch_bend(&mut self, amount: i16) {
        // Convert -8192 to +8191 to -2 to +2 semitones
        self.pitch_bend = (amount as f32 / 8192.0) * 2.0;
    }

    pub fn render(&mut self, buffer: &mut [f32]) {
        // Clear buffer first
        for sample in buffer.iter_mut() {
            *sample = 0.0;
        }

        // Render each active voice
        for voice in &mut self.voices {
            if voice.is_active() {
                for i in (0..buffer.len()).step_by(2) {
                    let sample = voice.render_sample(self.sample_rate, self.pitch_bend);
                    // Stereo output (same signal to both channels)
                    buffer[i] += sample;
                    if i + 1 < buffer.len() {
                        buffer[i + 1] += sample;
                    }
                }
            }
        }

        // Soft limiter to prevent clipping
        for sample in buffer.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    fn find_free_voice(&mut self) -> Option<&mut Voice> {
        self.voices.iter_mut().find(|v| !v.is_active())
    }
}

/// Convert MIDI note number to frequency in Hz
fn midi_to_frequency(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

/// Get settings for different instrument types
fn get_instrument_settings(instrument: InstrumentType) -> InstrumentSettings {
    match instrument {
        InstrumentType::CleanElectricGuitar => InstrumentSettings {
            wave_type: WaveType::Saw,
            attack_time: 0.005,
            release_time: 1.0,
            filter_cutoff: 0.8,
            resonance: 0.2,
            distortion: 0.0,
            volume: 0.4,
        },
        InstrumentType::DistortedGuitar => InstrumentSettings {
            wave_type: WaveType::Saw,
            attack_time: 0.01,
            release_time: 0.8,
            filter_cutoff: 0.6,
            resonance: 0.4,
            distortion: 0.7,
            volume: 0.35,
        },
        InstrumentType::AcousticGuitar => InstrumentSettings {
            wave_type: WaveType::Triangle,
            attack_time: 0.02,
            release_time: 2.0,
            filter_cutoff: 0.7,
            resonance: 0.1,
            distortion: 0.0,
            volume: 0.45,
        },
        InstrumentType::ClassicalGuitar => InstrumentSettings {
            wave_type: WaveType::Triangle,
            attack_time: 0.03,
            release_time: 2.5,
            filter_cutoff: 0.6,
            resonance: 0.15,
            distortion: 0.0,
            volume: 0.4,
        },
        InstrumentType::ElectricBass => InstrumentSettings {
            wave_type: WaveType::Sine,
            attack_time: 0.01,
            release_time: 1.2,
            filter_cutoff: 0.4,
            resonance: 0.3,
            distortion: 0.1,
            volume: 0.6,
        },
        InstrumentType::AcousticBass => InstrumentSettings {
            wave_type: WaveType::Triangle,
            attack_time: 0.02,
            release_time: 1.8,
            filter_cutoff: 0.3,
            resonance: 0.2,
            distortion: 0.0,
            volume: 0.55,
        },
        InstrumentType::Piano => InstrumentSettings {
            wave_type: WaveType::Triangle,
            attack_time: 0.001,
            release_time: 3.0,
            filter_cutoff: 0.9,
            resonance: 0.1,
            distortion: 0.0,
            volume: 0.5,
        },
        InstrumentType::Organ => InstrumentSettings {
            wave_type: WaveType::Sine,
            attack_time: 0.1,
            release_time: 0.1,
            filter_cutoff: 0.8,
            resonance: 0.0,
            distortion: 0.0,
            volume: 0.4,
        },
        InstrumentType::Strings => InstrumentSettings {
            wave_type: WaveType::Saw,
            attack_time: 0.2,
            release_time: 1.5,
            filter_cutoff: 0.7,
            resonance: 0.3,
            distortion: 0.0,
            volume: 0.35,
        },
        InstrumentType::SynthLead => InstrumentSettings {
            wave_type: WaveType::Square,
            attack_time: 0.01,
            release_time: 0.5,
            filter_cutoff: 0.9,
            resonance: 0.5,
            distortion: 0.2,
            volume: 0.4,
        },
        InstrumentType::SynthPad => InstrumentSettings {
            wave_type: WaveType::Saw,
            attack_time: 0.5,
            release_time: 2.0,
            filter_cutoff: 0.5,
            resonance: 0.4,
            distortion: 0.0,
            volume: 0.3,
        },
        InstrumentType::BrassSection => InstrumentSettings {
            wave_type: WaveType::Saw,
            attack_time: 0.05,
            release_time: 0.3,
            filter_cutoff: 0.8,
            resonance: 0.2,
            distortion: 0.1,
            volume: 0.45,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_to_frequency() {
        let a440 = midi_to_frequency(69);
        assert!((a440 - 440.0).abs() < 0.1);
        
        let middle_c = midi_to_frequency(60);
        assert!((middle_c - 261.63).abs() < 1.0);
    }

    #[test]
    fn test_synth_note_on() {
        let mut synth = FallbackSynth::new(48000);
        assert_eq!(synth.active_voice_count(), 0);
        
        synth.note_on(60, 100);
        assert_eq!(synth.active_voice_count(), 1);
    }

    #[test]
    fn test_synth_render() {
        let mut synth = FallbackSynth::new(48000);
        synth.note_on(60, 100);
        
        let mut buffer = vec![0.0; 256];
        synth.render(&mut buffer);
        
        // Check that some audio was generated
        let has_signal = buffer.iter().any(|&s| s.abs() > 0.001);
        assert!(has_signal);
    }
}
