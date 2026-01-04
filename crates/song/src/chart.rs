use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Custom deserializer for timeBeat that accepts both string and number
fn deserialize_time_beat<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(f64),
    }

    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => Ok(s),
        StringOrNumber::Number(n) => Ok(n.to_string()),
    }
}

/// Main song chart structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongChart {
    pub meta: SongMeta,
    pub clock: ClockSettings,
    pub playback: PlaybackSettings,
    pub mapping: MappingSettings,
    pub lanes: Vec<Lane>,
    pub lyrics: Vec<LyricEvent>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongMeta {
    pub title: String,
    pub artist: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub youtube: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spotify: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockSettings {
    pub bpm: f64,
    #[serde(rename = "timeSig")]
    pub time_sig: [u32; 2],
    #[serde(rename = "countInBars")]
    pub count_in_bars: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackSettings {
    #[serde(rename = "defaultInstrument")]
    pub default_instrument: InstrumentRef,
    #[serde(rename = "fallbackInstrument")]
    pub fallback_instrument: InstrumentRef,
    #[serde(rename = "allowUserOverrideInstrument")]
    pub allow_user_override_instrument: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentRef {
    #[serde(rename = "type")]
    pub instrument_type: String, // "soundfont" | "virtual"
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    pub chords: HashMap<String, ChordMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordMapping {
    pub frets: Vec<String>, // ["GREEN"], ["RED", "YELLOW"], etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lane {
    pub name: String,
    pub events: Vec<ChordEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordEvent {
    #[serde(alias = "startBeat")]
    pub beat: f64,
    pub dur: f64,
    pub chord: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordAnnotation {
    pub word: String,
    #[serde(rename = "timeBeat", deserialize_with = "deserialize_time_beat")]
    pub time_beat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricEvent {
    #[serde(alias = "startBeat")]
    pub beat: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<WordAnnotation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub name: String,
    #[serde(rename = "fromBeat")]
    pub from_beat: f64,
    #[serde(rename = "toBeat")]
    pub to_beat: f64,
}

impl SongChart {
    /// Load a chart from JSON string
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let chart: SongChart = serde_json::from_str(json)?;
        chart.validate()?;
        Ok(chart)
    }

    /// Validate the chart structure
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.clock.bpm <= 0.0 {
            anyhow::bail!("BPM must be positive");
        }
        if self.clock.time_sig[1] == 0 {
            anyhow::bail!("Time signature denominator cannot be zero");
        }
        
        // Validate chord events reference valid chords
        for lane in &self.lanes {
            for event in &lane.events {
                if !self.mapping.chords.contains_key(&event.chord) {
                    anyhow::bail!("Chord '{}' not found in mapping", event.chord);
                }
                if event.dur <= 0.0 {
                    anyhow::bail!("Chord duration must be positive");
                }
            }
        }
        
        Ok(())
    }

    /// Get all chord events sorted by beat
    pub fn get_all_chord_events(&self) -> Vec<&ChordEvent> {
        let mut events: Vec<&ChordEvent> = self.lanes
            .iter()
            .flat_map(|lane| &lane.events)
            .collect();
        events.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap());
        events
    }

    /// Get chord events within a beat range
    pub fn get_chord_events_in_range(&self, start_beat: f64, end_beat: f64) -> Vec<&ChordEvent> {
        self.lanes
            .iter()
            .flat_map(|lane| &lane.events)
            .filter(|e| e.beat >= start_beat && e.beat < end_beat)
            .collect()
    }

    /// Get lyrics within a beat range
    pub fn get_lyrics_in_range(&self, start_beat: f64, end_beat: f64) -> Vec<&LyricEvent> {
        self.lyrics
            .iter()
            .filter(|l| l.beat >= start_beat && l.beat < end_beat)
            .collect()
    }

    /// Get current section at beat
    pub fn get_section_at_beat(&self, beat: f64) -> Option<&Section> {
        self.sections
            .iter()
            .find(|s| beat >= s.from_beat && beat < s.to_beat)
    }

    /// Calculate total song duration in beats
    pub fn total_beats(&self) -> f64 {
        let max_chord_beat = self.lanes
            .iter()
            .flat_map(|lane| &lane.events)
            .map(|e| e.beat + e.dur)
            .fold(0.0, f64::max);
        
        let max_section_beat = self.sections
            .iter()
            .map(|s| s.to_beat)
            .fold(0.0, f64::max);
        
        max_chord_beat.max(max_section_beat)
    }

    /// Convert beat to seconds
    pub fn beat_to_seconds(&self, beat: f64, speed_multiplier: f64) -> f64 {
        let seconds_per_beat = (60.0 / self.clock.bpm) / speed_multiplier;
        beat * seconds_per_beat
    }

    /// Convert seconds to beat
    pub fn seconds_to_beat(&self, seconds: f64, speed_multiplier: f64) -> f64 {
        let seconds_per_beat = (60.0 / self.clock.bpm) / speed_multiplier;
        seconds / seconds_per_beat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_chart() {
        let json = r#"{
            "meta": {
                "title": "Test Song",
                "artist": "Test Artist"
            },
            "clock": {
                "bpm": 120,
                "timeSig": [4, 4],
                "countInBars": 2
            },
            "playback": {
                "defaultInstrument": {
                    "type": "soundfont",
                    "label": "Clean Guitar"
                },
                "fallbackInstrument": {
                    "type": "virtual",
                    "label": "Basic Guitar"
                },
                "allowUserOverrideInstrument": true
            },
            "mapping": {
                "chords": {
                    "C": { "frets": ["GREEN"] }
                }
            },
            "lanes": [
                {
                    "name": "chords",
                    "events": [
                        { "beat": 0, "dur": 4, "chord": "C" }
                    ]
                }
            ],
            "lyrics": [],
            "sections": []
        }"#;

        let chart = SongChart::from_json(json).unwrap();
        assert_eq!(chart.meta.title, "Test Song");
        assert_eq!(chart.clock.bpm, 120.0);
    }
}
