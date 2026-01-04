use song::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Song playback state manager
pub struct SongPlayer {
    chart: Option<SongChart>,
    transport: Transport,
    hit_detector: HitDetector,
    scorer: Scorer,
    instrument_resolver: InstrumentResolver,
    user_override_instrument: Option<InstrumentRef>,
}

impl SongPlayer {
    pub fn new(available_instruments: Vec<(String, String)>) -> Self {
        let global_default = ("virtual".to_string(), "Basic Guitar".to_string());
        Self {
            chart: None,
            transport: Transport::new(120.0, [4, 4], 2),
            hit_detector: HitDetector::new(&std::collections::HashMap::new()),
            scorer: Scorer::new(),
            instrument_resolver: InstrumentResolver::new(available_instruments, global_default),
            user_override_instrument: None,
        }
    }

    /// Load a song chart
    pub fn load_chart(&mut self, json: &str) -> anyhow::Result<()> {
        let chart = SongChart::from_json(json)?;
        
        // Initialize transport from chart
        self.transport = Transport::new(
            chart.clock.bpm,
            chart.clock.time_sig,
            chart.clock.count_in_bars,
        );

        // Initialize hit detector with chart mappings
        self.hit_detector = HitDetector::new(&chart.mapping.chords);

        // Reset scoring
        self.scorer.reset();

        self.chart = Some(chart);
        Ok(())
    }

    /// Get current chart
    pub fn get_chart(&self) -> Option<&SongChart> {
        self.chart.as_ref()
    }

    /// Play
    pub fn play(&mut self) {
        self.transport.play();
    }

    /// Pause
    pub fn pause(&mut self) {
        self.transport.pause();
    }

    /// Stop
    pub fn stop(&mut self) {
        self.transport.stop();
        self.hit_detector.reset();
        self.scorer.reset();
    }

    /// Seek to beat
    pub fn seek(&mut self, beat: f64) {
        self.transport.seek(beat);
    }

    /// Set speed
    pub fn set_speed(&mut self, multiplier: f64) {
        self.transport.set_speed(multiplier);
    }

    /// Get current beat
    pub fn get_current_beat(&mut self) -> f64 {
        self.transport.get_current_beat()
    }

    /// Check strum
    pub fn check_strum(&mut self, pressed_frets: Vec<String>) -> Option<HitResult> {
        let chart = self.chart.as_ref()?;
        let current_beat = self.transport.get_current_beat();
        
        // Get events in window
        let window_start = current_beat - HIT_WINDOW;
        let window_end = current_beat + HIT_WINDOW;
        let events = chart.get_chord_events_in_range(window_start, window_end);

        let result = self.hit_detector.check_strum(
            current_beat,
            &pressed_frets,
            &events,
        );

        // Update scoring
        self.scorer.register_hit(&result);

        Some(result)
    }

    /// Update sustain
    pub fn update_sustain(&mut self, pressed_frets: Vec<String>) -> bool {
        let current_beat = self.transport.get_current_beat();
        self.hit_detector.update_sustain(current_beat, &pressed_frets)
    }

    /// Get score
    pub fn get_score(&self) -> &Scorer {
        &self.scorer
    }

    /// Get transport state
    pub fn get_transport_state(&self) -> &Transport {
        &self.transport
    }

    /// Set user override instrument
    pub fn set_user_instrument(&mut self, instrument: Option<InstrumentRef>) {
        self.user_override_instrument = instrument;
    }

    /// Get resolved instrument
    pub fn get_resolved_instrument(&self) -> Option<ResolvedInstrument> {
        let chart = self.chart.as_ref()?;
        Some(self.instrument_resolver.resolve(
            &chart.playback.default_instrument,
            &chart.playback.fallback_instrument,
            self.user_override_instrument.as_ref(),
        ))
    }

    /// Get available instruments
    pub fn get_available_instruments(&self) -> &[(String, String)] {
        self.instrument_resolver.get_available_instruments()
    }
}
