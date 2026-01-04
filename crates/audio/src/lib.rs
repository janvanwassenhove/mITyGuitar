pub mod synth;
pub mod engine;
pub mod instant_callback;

#[cfg(feature = "soundfont")]
pub mod soundfont;

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use mapping::MusicEvent;
use ringbuf::{HeapRb, traits::Split};
use ringbuf::traits::{Consumer, Producer};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Engine control commands
#[derive(Debug, Clone)]
enum EngineControl {
    UseFallbackSynth,
    SetVirtualInstrument(SynthInstrumentType),
    SetReleaseMultiplier(f32),
    SetSustainEnabled(bool),
    SetSustainReleaseTime(f32),
    #[cfg(feature = "soundfont")]
    LoadSoundFont(std::path::PathBuf),
}

pub use synth::{FallbackSynth, InstrumentType as SynthInstrumentType};
pub use engine::AudioEngine;
pub use instant_callback::InstantAudioCallback;

#[cfg(feature = "soundfont")]
pub use soundfont::{SoundFontInfo, InstrumentInfo, InstrumentType as SoundFontInstrumentType, SoundFontManager, SoundFontSynth};

/// Audio statistics for diagnostics
#[derive(Debug, Clone, serde::Serialize)]
pub struct AudioStats {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub underruns: u64,
    pub active_voices: usize,
    pub estimated_latency_ms: f32,
}

// Wrapper to make Stream Send+Sync
// SAFETY: Stream is only used for its lifetime (never accessed after creation)
// The audio callback has its own references and doesn't need the stream
struct StreamWrapper(Stream);
unsafe impl Send for StreamWrapper {}
unsafe impl Sync for StreamWrapper {}

/// Audio output manager
pub struct AudioOutput {
    _stream: StreamWrapper,
    event_producer: ringbuf::HeapProd<MusicEvent>,
    stats: Arc<AudioStatsInner>,
    engine_control_tx: std::sync::mpsc::Sender<EngineControl>,
    stream_error: Arc<std::sync::atomic::AtomicBool>,
    buffer_size: Option<u32>,
}

struct AudioStatsInner {
    sample_rate: u32,
    buffer_size: u32,
    underruns: AtomicU64,
    active_voices: AtomicUsize,
}

impl AudioOutput {
    /// Create a new audio output with specified buffer size
    pub fn new(buffer_size: Option<u32>) -> Result<Self> {
        Self::create_with_device(None, buffer_size)
    }

    /// Try to reconnect to an available audio device
    pub fn try_reconnect(&mut self) -> Result<()> {
        log::info!("Attempting to reconnect to audio device...");
        match Self::create_with_device(None, self.buffer_size) {
            Ok(new_output) => {
                // Replace the current output with the new one
                self._stream = new_output._stream;
                self.event_producer = new_output.event_producer;
                self.stats = new_output.stats;
                self.engine_control_tx = new_output.engine_control_tx;
                self.stream_error.store(false, std::sync::atomic::Ordering::Relaxed);
                log::info!("Successfully reconnected to audio device");
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to reconnect to audio device: {}", e);
                Err(e)
            }
        }
    }

    /// Check if there was a stream error
    pub fn has_stream_error(&self) -> bool {
        self.stream_error.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Create audio output with a specific device (or find available one)
    fn create_with_device(device_name: Option<&str>, buffer_size: Option<u32>) -> Result<Self> {
        let host = cpal::default_host();
        
        // Try to get the specified device or find an available one
        let device = if let Some(name) = device_name {
            host.output_devices()?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .context(format!("Audio device '{}' not found", name))?
        } else {
            // Try default device first
            match host.default_output_device() {
                Some(dev) => dev,
                None => {
                    // If default is not available, try to find any available device
                    log::warn!("Default audio device not available, searching for alternatives...");
                    host.output_devices()?
                        .next()
                        .context("No audio output device available")?
                }
            }
        };

        log::info!("Using audio device: {}", device.name()?);

        let config = Self::get_config(&device, buffer_size)?;
        let sample_rate = config.sample_rate.0;
        
        // Create ring buffer for events (lock-free, RT-safe)
        let ring_buffer = HeapRb::<MusicEvent>::new(1024);
        let (event_producer, mut event_consumer) = ring_buffer.split(); // mutable for Consumer trait

        let stats = Arc::new(AudioStatsInner {
            sample_rate,
            buffer_size: buffer_size.unwrap_or(256),
            underruns: AtomicU64::new(0),
            active_voices: AtomicUsize::new(0),
        });

        let stats_clone = Arc::clone(&stats);

        // Create audio engine
        let mut engine = AudioEngine::new(sample_rate);
        
        // Create channel for engine control
        let (engine_control_tx, engine_control_rx) = std::sync::mpsc::channel::<EngineControl>();
        let engine_control_rx = Arc::new(std::sync::Mutex::new(engine_control_rx));
        let engine_control_rx_clone = Arc::clone(&engine_control_rx);

        // Create error flag for stream monitoring
        let stream_error = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stream_error_clone = Arc::clone(&stream_error);

        // Build the audio stream
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Check for engine control commands
                if let Ok(rx) = engine_control_rx_clone.try_lock() {
                    while let Ok(command) = rx.try_recv() {
                        match command {
                            EngineControl::UseFallbackSynth => {
                                if let Err(e) = engine.use_fallback_synth() {
                                    log::error!("Failed to switch to fallback synth: {}", e);
                                }
                            }
                            EngineControl::SetVirtualInstrument(instrument) => {
                                if let Err(e) = engine.set_virtual_instrument(instrument) {
                                    log::error!("Failed to set virtual instrument: {}", e);
                                }
                            }
                            EngineControl::SetReleaseMultiplier(multiplier) => {
                                engine.set_release_multiplier(multiplier);
                            }
                            EngineControl::SetSustainEnabled(enabled) => {
                                engine.set_sustain_enabled(enabled);
                            }
                            EngineControl::SetSustainReleaseTime(time) => {
                                engine.set_sustain_release_time(time);
                            }
                            #[cfg(feature = "soundfont")]
                            EngineControl::LoadSoundFont(path) => {
                                if let Err(e) = engine.load_soundfont(&path) {
                                    log::error!("Failed to load soundfont: {}", e);
                                }
                            }
                        }
                    }
                }
                
                Self::audio_callback(data, &mut engine, &mut event_consumer, &stats_clone);
            },
            move |err| {
                log::error!("Audio stream error: {}", err);
                stream_error_clone.store(true, std::sync::atomic::Ordering::Relaxed);
            },
            None,
        )?;

        stream.play()?;

        log::info!(
            "Audio stream started: {}Hz, buffer: {} samples",
            sample_rate,
            buffer_size.unwrap_or(256)
        );

        Ok(Self {
            _stream: StreamWrapper(stream),
            event_producer,
            stats,
            engine_control_tx,
            stream_error,
            buffer_size,
        })
    }
    
    #[cfg(feature = "soundfont")]
    pub fn load_soundfont(&self, path: std::path::PathBuf) -> Result<()> {
        self.engine_control_tx.send(EngineControl::LoadSoundFont(path))
            .context("Failed to send soundfont load message")?;
        Ok(())
    }

    /// Switch to using fallback synth for virtual instruments
    pub fn use_fallback_synth(&self) -> Result<()> {
        self.engine_control_tx.send(EngineControl::UseFallbackSynth)
            .context("Failed to send fallback synth message")?;
        Ok(())
    }

    /// Set virtual instrument type
    pub fn set_virtual_instrument(&self, instrument: SynthInstrumentType) -> Result<()> {
        self.engine_control_tx.send(EngineControl::SetVirtualInstrument(instrument))
            .context("Failed to send virtual instrument message")?;
        Ok(())
    }
    
    /// Set release time multiplier (affects how long notes fade out)
    pub fn set_release_multiplier(&self, multiplier: f32) -> Result<()> {
        self.engine_control_tx.send(EngineControl::SetReleaseMultiplier(multiplier))
            .context("Failed to send release multiplier message")?;
        Ok(())
    }
    
    /// Enable or disable sustain mode
    pub fn set_sustain_enabled(&self, enabled: bool) -> Result<()> {
        self.engine_control_tx.send(EngineControl::SetSustainEnabled(enabled))
            .context("Failed to send sustain enabled message")?;
        Ok(())
    }
    
    /// Set sustain release time in seconds
    pub fn set_sustain_release_time(&self, time_seconds: f32) -> Result<()> {
        self.engine_control_tx.send(EngineControl::SetSustainReleaseTime(time_seconds))
            .context("Failed to send sustain release time message")?;
        Ok(())
    }

    fn get_config(device: &Device, buffer_size: Option<u32>) -> Result<StreamConfig> {
        let default_config = device.default_output_config()?;
        
        let mut config = StreamConfig {
            channels: 2,
            sample_rate: default_config.sample_rate(),
            buffer_size: if let Some(size) = buffer_size {
                cpal::BufferSize::Fixed(size)
            } else {
                cpal::BufferSize::Default
            },
        };

        // Prefer 48kHz if available
        if config.sample_rate.0 != 48000 {
            config.sample_rate = cpal::SampleRate(48000);
        }

        Ok(config)
    }

    /// RT-safe audio callback - NO ALLOCATIONS, NO LOCKS
    fn audio_callback(
        data: &mut [f32],
        engine: &mut AudioEngine,
        event_consumer: &mut ringbuf::HeapCons<MusicEvent>,
        stats: &AudioStatsInner,
    ) {
        // Process all pending events
        while let Some(event) = Consumer::try_pop(event_consumer) {
            engine.handle_event(event);
        }

        // Generate audio
        engine.render(data);

        // Update stats (atomic operations are RT-safe)
        stats.active_voices.store(engine.active_voice_count(), Ordering::Relaxed);
    }

    /// Send a music event to the audio thread (RT-safe, lock-free)
    pub fn send_event(&mut self, event: MusicEvent) -> Result<()> {
        Producer::try_push(&mut self.event_producer, event)
            .map_err(|_| anyhow::anyhow!("Audio event queue full"))?;
        Ok(())
    }

    /// Get current audio statistics
    pub fn get_stats(&self) -> AudioStats {
        let buffer_size = self.stats.buffer_size;
        let sample_rate = self.stats.sample_rate;
        
        AudioStats {
            sample_rate,
            buffer_size,
            underruns: self.stats.underruns.load(Ordering::Relaxed),
            active_voices: self.stats.active_voices.load(Ordering::Relaxed),
            estimated_latency_ms: (buffer_size as f32 / sample_rate as f32) * 1000.0,
        }
    }

    /// Send panic/all notes off
    pub fn panic(&mut self) -> Result<()> {
        self.send_event(MusicEvent::PanicAllNotesOff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_stats() {
        let stats = AudioStats {
            sample_rate: 48000,
            buffer_size: 256,
            underruns: 0,
            active_voices: 0,
            estimated_latency_ms: 5.33,
        };
        
        assert_eq!(stats.sample_rate, 48000);
        assert!(stats.estimated_latency_ms < 10.0);
    }
}
