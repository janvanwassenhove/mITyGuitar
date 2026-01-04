import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./AudioSettings.css";

interface AudioStats {
  sample_rate: number;
  buffer_size: number;
  underruns: number;
  active_voices: number;
  estimated_latency_ms: number;
}

export default function AudioSettings() {
  const [releaseMultiplier, setReleaseMultiplier] = useState<number>(1.0);
  const [sustainEnabled, setSustainEnabled] = useState<boolean>(false);
  const [sustainReleaseTime, setSustainReleaseTime] = useState<number>(500);
  const [audioStats, setAudioStats] = useState<AudioStats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    loadSettings();
    loadAudioStats();
    
    // Update audio stats periodically
    const interval = setInterval(loadAudioStats, 1000);
    return () => clearInterval(interval);
  }, []);

  const loadSettings = async () => {
    try {
      const config = await invoke<any>("get_config");
      setReleaseMultiplier(config.audio.release_time_multiplier || 1.0);
      setSustainEnabled(config.audio.sustain_enabled || false);
      setSustainReleaseTime(config.audio.sustain_release_time_ms || 500);
    } catch (err: any) {
      console.error("Failed to load audio settings:", err);
      setError("Failed to load settings");
    }
  };

  const loadAudioStats = async () => {
    try {
      const stats = await invoke<AudioStats>("get_audio_stats");
      setAudioStats(stats);
    } catch (err: any) {
      console.error("Failed to load audio stats:", err);
    }
  };

  const handleReleaseMultiplierChange = async (value: number) => {
    setReleaseMultiplier(value);
    try {
      await invoke("set_release_multiplier", { multiplier: value });
    } catch (err: any) {
      console.error("Failed to set release multiplier:", err);
      setError("Failed to update release time");
    }
  };

  const handleSustainEnabledChange = async (enabled: boolean) => {
    setSustainEnabled(enabled);
    try {
      await invoke("set_sustain_enabled", { enabled });
    } catch (err: any) {
      console.error("Failed to set sustain enabled:", err);
      setError("Failed to update sustain mode");
    }
  };

  const handleSustainReleaseTimeChange = async (timeMs: number) => {
    setSustainReleaseTime(timeMs);
    try {
      await invoke("set_sustain_release_time", { timeMs });
    } catch (err: any) {
      console.error("Failed to set sustain release time:", err);
      setError("Failed to update sustain release time");
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    setError(null);
    
    try {
      // Get current config and update audio settings
      const config = await invoke<any>("get_config");
      config.audio.release_time_multiplier = releaseMultiplier;
      config.audio.sustain_enabled = sustainEnabled;
      config.audio.sustain_release_time_ms = sustainReleaseTime;
      await invoke("save_config", { config });
      
      // Show success message briefly
      setTimeout(() => setIsSaving(false), 500);
    } catch (err: any) {
      console.error("Failed to save settings:", err);
      setError("Failed to save settings");
      setIsSaving(false);
    }
  };

  const formatLatency = (ms: number): string => {
    return ms.toFixed(1);
  };

  return (
    <div className="view-container">
      <div className="view-header">
        <h1>Audio Settings</h1>
        <p className="view-subtitle">Configure audio engine parameters and monitor performance</p>
      </div>

      <div className="view-content">
        {error && (
          <div className="error-message" style={{ marginBottom: "20px" }}>
            {error}
          </div>
        )}

        <div className="settings-container">
          {/* Audio Statistics */}
          {audioStats && (
            <div className="setting-group">
              <label>Audio Engine Status</label>
              <div className="audio-stats">
                <div className="stat-row">
                  <span className="stat-label">Sample Rate:</span>
                  <span className="stat-value">{audioStats.sample_rate} Hz</span>
                </div>
                <div className="stat-row">
                  <span className="stat-label">Buffer Size:</span>
                  <span className="stat-value">{audioStats.buffer_size} samples</span>
                </div>
                <div className="stat-row">
                  <span className="stat-label">Latency:</span>
                  <span className="stat-value">{formatLatency(audioStats.estimated_latency_ms)} ms</span>
                </div>
                <div className="stat-row">
                  <span className="stat-label">Active Voices:</span>
                  <span className="stat-value">{audioStats.active_voices}</span>
                </div>
                <div className="stat-row">
                  <span className="stat-label">Buffer Underruns:</span>
                  <span className={`stat-value ${audioStats.underruns > 0 ? "warning" : ""}`}>
                    {audioStats.underruns}
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* Release Time Multiplier */}
          <div className="setting-group">
            <label>
              Release Time Multiplier
              <span className="setting-description">
                Controls how long notes take to fade out after release
              </span>
            </label>
            <div className="slider-container">
              <input
                type="range"
                min="0.1"
                max="3.0"
                step="0.1"
                value={releaseMultiplier}
                onChange={(e) => handleReleaseMultiplierChange(parseFloat(e.target.value))}
                className="slider"
              />
              <div className="slider-value">{releaseMultiplier.toFixed(1)}x</div>
            </div>
            <div className="hint">
              <strong>Tip:</strong> Lower values (0.5-0.8x) create tighter, punchier sounds. 
              Higher values (1.5-3.0x) create longer, more sustained tones.
            </div>
          </div>

          {/* Sustain Mode */}
          <div className="setting-group">
            <label>
              Sustain Mode
              <span className="setting-description">
                Hold notes indefinitely until released (like a sustain pedal)
              </span>
            </label>
            <div className="toggle-container">
              <label className="toggle-switch">
                <input
                  type="checkbox"
                  checked={sustainEnabled}
                  onChange={(e) => handleSustainEnabledChange(e.target.checked)}
                />
                <span className="toggle-slider"></span>
              </label>
              <span className="toggle-label">
                {sustainEnabled ? "Enabled" : "Disabled"}
              </span>
            </div>
          </div>

          {/* Sustain Release Time */}
          {sustainEnabled && (
            <div className="setting-group">
              <label>
                Sustain Release Time
                <span className="setting-description">
                  How quickly sustained notes fade after release
                </span>
              </label>
              <div className="slider-container">
                <input
                  type="range"
                  min="100"
                  max="2000"
                  step="50"
                  value={sustainReleaseTime}
                  onChange={(e) => handleSustainReleaseTimeChange(parseFloat(e.target.value))}
                  className="slider"
                />
                <div className="slider-value">{sustainReleaseTime} ms</div>
              </div>
              <div className="hint">
                <strong>Tip:</strong> Shorter times (100-300ms) work well for fast passages. 
                Longer times (500-2000ms) create smooth, organ-like releases.
              </div>
            </div>
          )}

        </div>

        <div className="view-actions">
          <button 
            className="button-primary" 
            onClick={handleSave}
            disabled={isSaving}
          >
            {isSaving ? "Saved ✓" : "Save Settings"}
          </button>
        </div>
      </div>
    </div>
  );
}
