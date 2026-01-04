use std::time::Instant;

/// Transport clock for beat-based playback
#[derive(Debug, Clone)]
pub struct Transport {
    pub bpm: f64,
    pub time_sig: [u32; 2],
    pub count_in_bars: u32,
    pub speed_multiplier: f64,
    
    pub is_playing: bool,
    pub current_beat: f64,
    
    start_instant: Option<Instant>,
    paused_at_beat: f64,
}

impl Transport {
    pub fn new(bpm: f64, time_sig: [u32; 2], count_in_bars: u32) -> Self {
        Self {
            bpm,
            time_sig,
            count_in_bars,
            speed_multiplier: 1.0,
            is_playing: false,
            current_beat: 0.0,
            start_instant: None,
            paused_at_beat: 0.0,
        }
    }

    /// Start or resume playback
    pub fn play(&mut self) {
        if !self.is_playing {
            self.is_playing = true;
            self.start_instant = Some(Instant::now());
            self.paused_at_beat = self.current_beat;
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if self.is_playing {
            self.update_current_beat();
            self.is_playing = false;
            self.paused_at_beat = self.current_beat;
            self.start_instant = None;
        }
    }

    /// Stop and reset to beginning (including count-in)
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_beat = -(self.count_in_bars as f64 * self.time_sig[0] as f64);
        self.paused_at_beat = self.current_beat;
        self.start_instant = None;
    }

    /// Seek to a specific beat
    pub fn seek(&mut self, beat: f64) {
        let was_playing = self.is_playing;
        if was_playing {
            self.pause();
        }
        self.current_beat = beat;
        self.paused_at_beat = beat;
        if was_playing {
            self.play();
        }
    }

    /// Set speed multiplier (0.75x, 1.0x, 1.25x, etc.)
    pub fn set_speed(&mut self, multiplier: f64) {
        let was_playing = self.is_playing;
        if was_playing {
            self.update_current_beat();
            self.paused_at_beat = self.current_beat;
        }
        self.speed_multiplier = multiplier;
        if was_playing {
            self.start_instant = Some(Instant::now());
        }
    }

    /// Update current beat based on elapsed time
    pub fn update_current_beat(&mut self) {
        if let Some(start) = self.start_instant {
            let elapsed = start.elapsed().as_secs_f64();
            let beats_elapsed = self.seconds_to_beats(elapsed);
            self.current_beat = self.paused_at_beat + beats_elapsed;
        }
    }

    /// Get current beat (updates if playing)
    pub fn get_current_beat(&mut self) -> f64 {
        if self.is_playing {
            self.update_current_beat();
        }
        self.current_beat
    }

    /// Convert beats to seconds
    pub fn beats_to_seconds(&self, beats: f64) -> f64 {
        let seconds_per_beat = (60.0 / self.bpm) / self.speed_multiplier;
        beats * seconds_per_beat
    }

    /// Convert seconds to beats
    pub fn seconds_to_beats(&self, seconds: f64) -> f64 {
        let seconds_per_beat = (60.0 / self.bpm) / self.speed_multiplier;
        seconds / seconds_per_beat
    }

    /// Check if in count-in period
    pub fn is_in_count_in(&self) -> bool {
        self.current_beat < 0.0
    }

    /// Get current bar number
    pub fn get_current_bar(&self) -> i32 {
        let beats_per_bar = self.time_sig[0] as f64;
        (self.current_beat / beats_per_bar).floor() as i32
    }

    /// Get current beat within bar (0-indexed)
    pub fn get_beat_in_bar(&self) -> f64 {
        let beats_per_bar = self.time_sig[0] as f64;
        self.current_beat.rem_euclid(beats_per_bar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_transport_basic() {
        let mut transport = Transport::new(120.0, [4, 4], 2);
        
        // Should start at count-in
        assert_eq!(transport.current_beat, 0.0);
        assert!(!transport.is_playing);
        
        transport.stop();
        assert_eq!(transport.current_beat, -8.0); // -2 bars * 4 beats
    }

    #[test]
    fn test_transport_playback() {
        let mut transport = Transport::new(120.0, [4, 4], 0);
        transport.play();
        
        thread::sleep(Duration::from_millis(500));
        let beat = transport.get_current_beat();
        
        // At 120 BPM, 0.5 seconds = 1 beat
        assert!(beat >= 0.9 && beat <= 1.1);
    }

    #[test]
    fn test_transport_speed_multiplier() {
        let mut transport = Transport::new(120.0, [4, 4], 0);
        transport.set_speed(2.0); // 2x speed
        
        // At 2x speed, beats should advance twice as fast
        let seconds = 0.5;
        let beats = transport.seconds_to_beats(seconds);
        assert!((beats - 2.0).abs() < 0.01);
    }
}
