use controller::AudioCallback;
use mapping::MusicEvent;
use std::sync::Arc;
use ringbuf::traits::{Producer, Split};

/// Instant audio callback that triggers sounds directly from controller events
pub struct InstantAudioCallback {
    event_producer: Arc<std::sync::Mutex<ringbuf::HeapProd<MusicEvent>>>,
}

impl InstantAudioCallback {
    pub fn new() -> Self {
        // Create a high-capacity ring buffer for instant event delivery
        let (producer, _consumer) = ringbuf::HeapRb::<MusicEvent>::new(4096).split();
        
        Self {
            event_producer: Arc::new(std::sync::Mutex::new(producer)),
        }
    }
    
    /// Get the consumer end for the audio system to read events
    pub fn get_consumer(&self) -> Option<ringbuf::HeapCons<MusicEvent>> {
        // This would need to be redesigned to properly integrate with the audio system
        // For now, returning None as this needs architectural changes
        None
    }
    
    /// Map fret number to MIDI note (standard guitar tuning)
    fn fret_to_note(&self, fret: u8) -> u8 {
        match fret {
            0 => 64, // E (green)
            1 => 67, // G (red) 
            2 => 71, // B (yellow)
            3 => 74, // D (blue)
            4 => 77, // F (orange)
            _ => 60, // Default to middle C
        }
    }
}

impl AudioCallback for InstantAudioCallback {
    fn on_fret_press(&self, fret: u8, velocity: f32) {
        let note = self.fret_to_note(fret);
        let velocity_midi = (velocity * 127.0) as u8;
        
        let event = MusicEvent::NoteOn {
            note,
            velocity: velocity_midi,
        };
        
        // Send event instantly to audio thread (lock-free push)
        if let Ok(mut producer) = self.event_producer.try_lock() {
            let _ = producer.try_push(event);
        }
        
        log::debug!("🎸 INSTANT fret press: fret={} note={} vel={}", fret, note, velocity_midi);
    }
    
    fn on_fret_release(&self, fret: u8) {
        let note = self.fret_to_note(fret);
        
        let event = MusicEvent::NoteOff { note };
        
        // Send event instantly to audio thread
        if let Ok(mut producer) = self.event_producer.try_lock() {
            let _ = producer.try_push(event);
        }
        
        log::debug!("🎸 INSTANT fret release: fret={} note={}", fret, note);
    }
    
    fn on_strum(&self, up: bool, velocity: f32) {
        log::debug!("🎸 INSTANT strum: up={} velocity={}", up, velocity);
        
        // Strum could trigger chord progression or strum effect
        // Implementation depends on whether we want individual notes or chord strumming
    }
    
    fn on_whammy_change(&self, value: f32) {
        // Convert to pitch bend value (-1..1 -> -8192..8191)
        let pitch_bend = (value * 8192.0) as i16;
        
        let event = MusicEvent::PitchBend(pitch_bend);
        
        // Send event instantly to audio thread
        if let Ok(mut producer) = self.event_producer.try_lock() {
            let _ = producer.try_push(event);
        }
        
        log::debug!("🎸 INSTANT whammy: value={} bend={}", value, pitch_bend);
    }
}