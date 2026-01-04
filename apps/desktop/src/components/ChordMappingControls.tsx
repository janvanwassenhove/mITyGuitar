import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './ChordMappingControls.css';

interface ChordMappingSettings {
  genre: string;
  key_root: string;
  mode: 'Major' | 'Minor';
  sustain_enabled: boolean;
  sustain_release_time_ms: number;
  whammy_enabled: boolean;
  whammy_pitch_bend_range: number;
  whammy_vibrato_depth: number;
  whammy_filter_cutoff_enabled: boolean;
}

interface ChordMappingControlsProps {
  settings: ChordMappingSettings;
  onSettingsChange: (newSettings: ChordMappingSettings) => void;
}

const GENRES = ['Punk', 'EDM', 'Rock', 'Pop', 'Folk', 'Metal'];
const NOTES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
const MODES = ['Major', 'Minor'] as const;

export default function ChordMappingControls({ settings, onSettingsChange }: ChordMappingControlsProps) {
  const [isExpanded, setIsExpanded] = useState<boolean>(false);

  const updateSetting = <K extends keyof ChordMappingSettings>(
    key: K,
    value: ChordMappingSettings[K]
  ) => {
    const newSettings = { ...settings, [key]: value };
    onSettingsChange(newSettings);
    
    // Save sustain settings to config when changed
    if (key === 'sustain_enabled' || key === 'sustain_release_time_ms') {
      saveAudioConfig(newSettings);
    }
  };

  const saveAudioConfig = async (newSettings: ChordMappingSettings) => {
    try {
      const config = await invoke<any>("get_config");
      config.audio.sustain_enabled = newSettings.sustain_enabled;
      config.audio.sustain_release_time_ms = newSettings.sustain_release_time_ms;
      await invoke("save_config", { config });
    } catch (error) {
      console.error("Failed to save audio config:", error);
    }
  };

  const toggleExpanded = () => {
    setIsExpanded(!isExpanded);
  };

  return (
    <div className="chord-mapping-controls">
      <div className="settings-header" onClick={toggleExpanded}>
        <h3>Guitar Settings</h3>
        <span className={`expand-icon ${isExpanded ? 'expanded' : ''}`}>
          ▼
        </span>
      </div>
      
      {isExpanded && (
        <div className="compact-layout">
          {/* Column 1: General Settings */}
          <div className="settings-column">
            <div className="column-header">General</div>
            <div className="control-group">
              <label htmlFor="genre-select">Genre</label>
              <select
                id="genre-select"
                value={settings.genre}
                onChange={(e) => updateSetting('genre', e.target.value)}
                className="control-select"
              >
                {GENRES.map(genre => (
                  <option key={genre} value={genre}>{genre}</option>
                ))}
              </select>
            </div>
            
            <div className="control-group">
              <label htmlFor="key-select">Key</label>
              <select
                id="key-select"
                value={settings.key_root}
                onChange={(e) => updateSetting('key_root', e.target.value)}
                className="control-select"
              >
                {NOTES.map(note => (
                  <option key={note} value={note}>{note}</option>
                ))}
              </select>
            </div>
            
            <div className="control-group">
              <label htmlFor="mode-select">Mode</label>
              <select
                id="mode-select"
                value={settings.mode}
                onChange={(e) => updateSetting('mode', e.target.value as 'Major' | 'Minor')}
                className="control-select"
              >
                {MODES.map(mode => (
                  <option key={mode} value={mode}>{mode}</option>
                ))}
              </select>
            </div>
          </div>

          {/* Column 2: Sustain Settings */}
          <div className="settings-column">
            <div className="column-header">Sustain</div>
            <div className="control-group">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={settings.sustain_enabled}
                  onChange={(e) => updateSetting('sustain_enabled', e.target.checked)}
                  className="control-checkbox"
                />
                <span className="checkbox-text">Enable Sustain</span>
              </label>
            </div>
            
            {settings.sustain_enabled && (
              <div className="control-group">
                <label htmlFor="sustain-release">
                  Release: {settings.sustain_release_time_ms.toFixed(0)}ms
                </label>
                <input
                  id="sustain-release"
                  type="range"
                  min="50"
                  max="2000"
                  step="50"
                  value={settings.sustain_release_time_ms}
                  onChange={(e) => updateSetting('sustain_release_time_ms', parseFloat(e.target.value))}
                  className="control-slider"
                  style={{
                    '--value': `${((settings.sustain_release_time_ms - 50) / (2000 - 50)) * 100}%`
                  } as React.CSSProperties}
                />
              </div>
            )}
          </div>

          {/* Column 3: Whammy Settings */}
          <div className="settings-column">
            <div className="column-header">Whammy Bar</div>
            <div className="control-group">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={settings.whammy_enabled}
                  onChange={(e) => updateSetting('whammy_enabled', e.target.checked)}
                  className="control-checkbox"
                />
                <span className="checkbox-text">Enable Whammy</span>
              </label>
            </div>
            
            {settings.whammy_enabled && (
              <>
                <div className="control-group">
                  <label htmlFor="whammy-pitch">
                    Pitch: {settings.whammy_pitch_bend_range.toFixed(1)}st
                  </label>
                  <input
                    id="whammy-pitch"
                    type="range"
                    min="0.1"
                    max="5.0"
                    step="0.1"
                    value={settings.whammy_pitch_bend_range}
                    onChange={(e) => updateSetting('whammy_pitch_bend_range', parseFloat(e.target.value))}
                    className="control-slider"
                    style={{
                      '--value': `${((settings.whammy_pitch_bend_range - 0.1) / (5.0 - 0.1)) * 100}%`
                    } as React.CSSProperties}
                  />
                </div>
                
                <div className="control-group">
                  <label htmlFor="whammy-vibrato">
                    Vibrato: {(settings.whammy_vibrato_depth * 100).toFixed(0)}%
                  </label>
                  <input
                    id="whammy-vibrato"
                    type="range"
                    min="0.0"
                    max="1.0"
                    step="0.05"
                    value={settings.whammy_vibrato_depth}
                    onChange={(e) => updateSetting('whammy_vibrato_depth', parseFloat(e.target.value))}
                    className="control-slider"
                    style={{
                      '--value': `${(settings.whammy_vibrato_depth / 1.0) * 100}%`
                    } as React.CSSProperties}
                  />
                </div>
                
                <div className="control-group">
                  <label className="checkbox-label">
                    <input
                      type="checkbox"
                      checked={settings.whammy_filter_cutoff_enabled}
                      onChange={(e) => updateSetting('whammy_filter_cutoff_enabled', e.target.checked)}
                      className="control-checkbox"
                    />
                    <span className="checkbox-text">Filter Sweep</span>
                  </label>
                </div>
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
}