import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import FretBoard from "./FretBoard";
import ChordMappingControls from "./ChordMappingControls";

interface LiveViewProps {
  genreInfo: any;
  onAction: (action: string) => void;
}

interface ControllerState {
  fret_green: boolean;
  fret_red: boolean;
  fret_blue: boolean;
  fret_yellow: boolean;
  fret_orange: boolean;
  solo_green: boolean;
  solo_red: boolean;
  solo_blue: boolean;
  solo_yellow: boolean;
  solo_orange: boolean;
  strum_up: boolean;
  strum_down: boolean;
  dpad_up: boolean;
  dpad_down: boolean;
  dpad_left: boolean;
  dpad_right: boolean;
  start: boolean;
  select: boolean;
  whammy_bar: number;
  connected: boolean;
  timestamp: number;
}

interface AppConfig {
  soundfonts: {
    current: string | null;
  };
}

interface ChordMapState {
  green: string;
  red: string;
  yellow: string;
  blue: string;
  orange: string;
}

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

interface InstrumentInfo {
  name: string;
  path?: string;
  size_bytes?: number;
  instrument_type: 'SoundFont' | 'Virtual';
}

export default function LiveView({ genreInfo, onAction }: LiveViewProps) {
  const [controllerState, setControllerState] = useState<ControllerState | null>(null);
  const [currentSoundfont, setCurrentSoundfont] = useState<string | null>(null);
  const [simulatorEnabled, setSimulatorEnabled] = useState<boolean>(false);
  const [mainChords, setMainChords] = useState<ChordMapState>({
    green: 'E5', red: 'A5', yellow: 'B5', blue: 'D5', orange: 'C#5'
  });
  const [soloChords, setSoloChords] = useState<ChordMapState>({
    green: 'E5', red: 'A5', yellow: 'B5', blue: 'D5', orange: 'C#5'  
  });
  const [chordMappingSettings, setChordMappingSettings] = useState<ChordMappingSettings>({
    genre: 'Punk',
    key_root: 'E',
    mode: 'Major',
    sustain_enabled: true,
    sustain_release_time_ms: 500,
    whammy_enabled: true,
    whammy_pitch_bend_range: 1.0,
    whammy_vibrato_depth: 0.0,
    whammy_filter_cutoff_enabled: false
  });  const [availableInstruments, setAvailableInstruments] = useState<InstrumentInfo[]>([]);
  const [showInstrumentDropdown, setShowInstrumentDropdown] = useState<boolean>(false);
  const [previousDpadUp, setPreviousDpadUp] = useState<boolean>(false);
  const [previousDpadDown, setPreviousDpadDown] = useState<boolean>(false);
  const [keyboardShortcutsExpanded, setKeyboardShortcutsExpanded] = useState<boolean>(false);

  const availableGenres = ['EDM', 'Folk', 'Metal', 'Pop', 'Punk', 'Rock'];

  // Load sustain settings from config on mount
  useEffect(() => {
    const loadAudioConfig = async () => {
      try {
        const config = await invoke<any>("get_config");
        setChordMappingSettings(prev => ({
          ...prev,
          sustain_enabled: config.audio.sustain_enabled ?? true,
          sustain_release_time_ms: config.audio.sustain_release_time_ms ?? 500
        }));
      } catch (error) {
        console.error("Failed to load audio config:", error);
      }
    };
    loadAudioConfig();
  }, []);

  useEffect(() => {
    // Poll controller state for DISPLAY ONLY - audio triggers instantly via callbacks!
    const interval = setInterval(async () => {
      try {
        const state = await invoke<ControllerState>("get_controller_state");
        
        // Detect D-pad up press (edge detection)
        if (state.dpad_up && !previousDpadUp) {
          // Cycle to next genre
          const currentIndex = availableGenres.indexOf(chordMappingSettings.genre);
          const nextIndex = (currentIndex + 1) % availableGenres.length;
          const newSettings = { ...chordMappingSettings, genre: availableGenres[nextIndex] };
          handleSettingsChange(newSettings);
        }
        
        // Detect D-pad down press (edge detection)
        if (state.dpad_down && !previousDpadDown) {
          // Cycle to previous genre
          const currentIndex = availableGenres.indexOf(chordMappingSettings.genre);
          const prevIndex = (currentIndex - 1 + availableGenres.length) % availableGenres.length;
          const newSettings = { ...chordMappingSettings, genre: availableGenres[prevIndex] };
          handleSettingsChange(newSettings);
        }
        
        setPreviousDpadUp(state.dpad_up);
        setPreviousDpadDown(state.dpad_down);
        setControllerState(state);
        
        // Auto-disable simulator if hardware controller is connected
        if (state.connected && simulatorEnabled) {
          setSimulatorEnabled(false);
        }
        
        // Also update soundfont display when it changes
        loadCurrentSoundfont();
      } catch (error) {
        console.error("Failed to get controller state:", error);
      }
    }, 100); // Reduced to 10 Hz - display only, audio is instant!

    // Load initial chord mapping and soundfont
    loadChordMapping();
    loadCurrentSoundfont();
    loadAvailableInstruments();

    return () => clearInterval(interval);
  }, [simulatorEnabled, previousDpadUp, previousDpadDown, chordMappingSettings.genre]);

  // Load chord mapping when settings change
  useEffect(() => {
    loadChordMapping();
  }, [chordMappingSettings.genre, chordMappingSettings.key_root, chordMappingSettings.mode]);
  
  // Apply sustain settings when they change
  useEffect(() => {
    const applySustainSettings = async () => {
      try {
        await invoke("set_sustain_enabled", { enabled: chordMappingSettings.sustain_enabled });
        await invoke("set_sustain_release_time", { timeMs: chordMappingSettings.sustain_release_time_ms });
      } catch (error) {
        console.error("Failed to apply sustain settings:", error);
      }
    };
    
    applySustainSettings();
  }, [chordMappingSettings.sustain_enabled, chordMappingSettings.sustain_release_time_ms]);

  const loadChordMapping = async () => {
    try {
      const chordMap = await invoke<{main: ChordMapState, solo: ChordMapState}>("get_chord_mapping", {
        genre: chordMappingSettings.genre,
        keyRoot: chordMappingSettings.key_root,
        mode: chordMappingSettings.mode
      });
      setMainChords(chordMap.main);
      setSoloChords(chordMap.solo);
    } catch (error) {
      console.error("Failed to load chord mapping:", error);
    }
  };

  const loadCurrentSoundfont = async () => {
    try {
      const config = await invoke<AppConfig>("get_app_config");
      setCurrentSoundfont(config.soundfonts.current);
    } catch (error) {
      console.error("Failed to load app config:", error);
    }
  };

  const loadAvailableInstruments = async () => {
    try {
      const instruments = await invoke<InstrumentInfo[]>("get_available_instruments");
      setAvailableInstruments(instruments);
    } catch (error) {
      console.error("Failed to load available instruments:", error);
      setAvailableInstruments([]);
    }
  };

  const handleInstrumentSelect = async (instrumentName: string) => {
    try {
      await invoke("set_instrument", { name: instrumentName });
      setCurrentSoundfont(instrumentName);
      setShowInstrumentDropdown(false);
    } catch (error) {
      // Fallback to set_soundfont if set_instrument doesn't exist
      try {
        await invoke("set_soundfont", { name: instrumentName });
        setCurrentSoundfont(instrumentName);
        setShowInstrumentDropdown(false);
      } catch (fallbackError) {
        console.error("Failed to set instrument:", error, fallbackError);
      }
    }
  };

  const handleChordEdit = async (fret: keyof ChordMapState, row: 'main' | 'solo', newChord: string) => {
    try {
      await invoke("update_chord_override", {
        fretButton: fret,
        row: row,
        chordSpec: newChord
      });
      
      // Update local state
      if (row === 'main') {
        setMainChords(prev => ({ ...prev, [fret]: newChord }));
      } else {
        setSoloChords(prev => ({ ...prev, [fret]: newChord }));
      }
    } catch (error) {
      console.error("Failed to update chord override:", error);
    }
  };

  const handleSettingsChange = async (newSettings: ChordMappingSettings) => {
    try {
      await invoke("update_chord_mapping_settings", { settings: newSettings });
      setChordMappingSettings(newSettings);
    } catch (error) {
      console.error("Failed to update chord mapping settings:", error);
    }
  };

  useEffect(() => {
    // Handle keyboard input for simulator - only when enabled
    const handleKeyDown = (e: KeyboardEvent) => {
      if (simulatorEnabled) {
        invoke("simulator_key_down", { key: e.key }).catch(console.error);
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (simulatorEnabled) {
        invoke("simulator_key_up", { key: e.key }).catch(console.error);
      }
    };

    // Close instrument dropdown when clicking outside
    const handleClickOutside = (e: MouseEvent) => {
      if (showInstrumentDropdown) {
        const target = e.target as Element;
        if (!target.closest('.info-panel')) {
          setShowInstrumentDropdown(false);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    window.addEventListener("click", handleClickOutside);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      window.removeEventListener("click", handleClickOutside);
    };
  }, [simulatorEnabled, showInstrumentDropdown]);

  const isButtonPressed = (button: string): boolean => {
    if (!controllerState) return false;
    
    switch (button) {
      case "FretGreen": return controllerState.fret_green;
      case "FretRed": return controllerState.fret_red;
      case "FretYellow": return controllerState.fret_yellow;
      case "FretBlue": return controllerState.fret_blue;
      case "FretOrange": return controllerState.fret_orange;
      case "StrumUp": return controllerState.strum_up;
      case "StrumDown": return controllerState.strum_down;
      case "DPadLeft": return controllerState.dpad_left;
      case "DPadRight": return controllerState.dpad_right;
      case "DPadUp": return controllerState.dpad_up;
      case "DPadDown": return controllerState.dpad_down;
      case "Start": return controllerState.start;
      case "Select": return controllerState.select;
      // Solo buttons might not be supported yet
      case "SoloGreen":
      case "SoloRed":
      case "SoloYellow":
      case "SoloBlue":
      case "SoloOrange":
        return false; // Not implemented in new controller yet
      default: return false;
    }
  };

  const getAxisValue = (axis: string): number => {
    if (!controllerState) return 0;
    
    switch (axis) {
      case "WhammyBar": return controllerState.whammy_bar;
      default: return 0;
    }
  };

  return (
    <div className="live-view">

      {/* Chord Mapping Controls */}
      <ChordMappingControls 
        settings={chordMappingSettings}
        onSettingsChange={handleSettingsChange}
      />

      {/* Chord Mapping and Controller Layout */}
      <div style={{ display: 'flex', gap: '24px', alignItems: 'flex-start' }}>
        {/* Chord Mapping Column */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Current Instrument - above Chord Mapping */}
          <div className="info-panel" style={{ position: 'relative' }}>
            <div className="info-row">
              <span className="info-label">Instrument:</span>
              <span 
                style={{ 
                  fontWeight: 600, 
                  cursor: 'pointer', 
                  padding: '4px 8px',
                  borderRadius: '4px',
                  background: 'rgba(255, 255, 255, 0.1)',
                  border: '1px solid rgba(255, 255, 255, 0.2)',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  userSelect: 'none'
                }}
                onClick={() => setShowInstrumentDropdown(!showInstrumentDropdown)}
                onMouseEnter={(e) => e.currentTarget.style.background = 'rgba(255, 255, 255, 0.15)'}
                onMouseLeave={(e) => e.currentTarget.style.background = 'rgba(255, 255, 255, 0.1)'}
              >
                {currentSoundfont || "Fallback Synth"}
                <span style={{ fontSize: '12px', color: 'rgba(255, 255, 255, 0.7)' }}>▼</span>
              </span>
            </div>
            
            {showInstrumentDropdown && (
              <div style={{
                position: 'absolute',
                top: '100%',
                left: 0,
                right: 0,
                zIndex: 1000,
                background: '#2a2a2a',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                borderRadius: '4px',
                maxHeight: '300px',
                overflowY: 'auto',
                boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)'
              }}>
                {availableInstruments.map((instrument) => (
                  <div
                    key={instrument.name}
                    style={{
                      padding: '8px 12px',
                      cursor: 'pointer',
                      borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center'
                    }}
                    onClick={() => handleInstrumentSelect(instrument.name)}
                    onMouseEnter={(e) => e.currentTarget.style.background = 'rgba(255, 255, 255, 0.1)'}
                    onMouseLeave={(e) => e.currentTarget.style.background = 'transparent'}
                  >
                    <span style={{ color: '#fff' }}>{instrument.name}</span>
                    <span style={{ 
                      fontSize: '11px', 
                      color: 'rgba(255, 255, 255, 0.5)',
                      background: instrument.instrument_type === 'Virtual' ? 'rgba(0, 150, 255, 0.3)' : 'rgba(255, 150, 0, 0.3)',
                      padding: '2px 6px',
                      borderRadius: '3px'
                    }}>
                      {instrument.instrument_type === 'Virtual' ? 'Virtual' : 'SoundFont'}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* New Fret Board */}
          <FretBoard
            mainChords={mainChords}
            soloChords={soloChords}
            controllerState={controllerState}
            isEditable={true}
            onChordEdit={handleChordEdit}
          />
        </div>

        {/* Controller Inputs */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0', flex: '1', background: '#1a1a1a', borderRadius: '12px', padding: '0', overflow: 'hidden', height: 'fit-content' }}>
          {/* Strum Bar */}
          <div className="strum-bar" style={{ margin: '0', borderRadius: '0', borderBottom: '1px solid #2a2a2a' }}>
            <label style={{ fontWeight: 600, width: 100 }}>Strum Bar:</label>
            <div className={`strum-indicator ${isButtonPressed("StrumUp") ? "active" : ""}`}>
              ⬆️ UP
            </div>
            <div className={`strum-indicator ${isButtonPressed("StrumDown") ? "active" : ""}`}>
              ⬇️ DOWN
            </div>
          </div>

          {/* D-Pad */}
          <div className="strum-bar" style={{ margin: '0', borderRadius: '0', borderBottom: '1px solid #2a2a2a' }}>
            <label style={{ fontWeight: 600, width: 100 }}>D-Pad:</label>
            <div className={`strum-indicator ${isButtonPressed("DPadLeft") ? "active" : ""}`}>
              ⬅️ LEFT
            </div>
            <div className={`strum-indicator ${isButtonPressed("DPadRight") ? "active" : ""}`}>
              ➡️ RIGHT
            </div>
          </div>

          {/* Whammy Bar */}
          <div className="axis-control" style={{ margin: '0', borderRadius: '0', borderBottom: '1px solid #2a2a2a' }}>
            <h3>Whammy Bar</h3>
            <div className="axis-meter">
              <div className="axis-bar">
                <div
                  className="axis-fill"
                  style={{ width: `${getAxisValue("WhammyBar") * 100}%` }}
                />
              </div>
              <span className="axis-value">{getAxisValue("WhammyBar").toFixed(2)}</span>
            </div>
          </div>

          {/* Tilt */}
          <div className="axis-control" style={{ margin: '0', borderRadius: '0' }}>
            <h3>Tilt</h3>
            <div className="axis-meter">
              <div className="axis-bar">
                <div
                  className="axis-fill"
                  style={{ width: `${getAxisValue("TiltSensor") * 100}%` }}
                />
              </div>
              <span className="axis-value">{getAxisValue("TiltSensor").toFixed(2)}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Keyboard Shortcuts Help */}
      <div className="info-panel" style={{ marginTop: "2rem" }}>
        <div className="settings-header" onClick={() => setKeyboardShortcutsExpanded(!keyboardShortcutsExpanded)}>
          <h3>Keyboard Shortcuts (Simulator)</h3>
          <span className={`expand-icon ${keyboardShortcutsExpanded ? 'expanded' : ''}`}>
            ▼
          </span>
        </div>
        {keyboardShortcutsExpanded && (
          <>
            <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer', marginBottom: '1rem' }}>
              <span style={{ fontSize: '0.9rem' }}>Enable Simulator</span>
              <input 
                type="checkbox" 
                checked={simulatorEnabled}
                onChange={(e) => setSimulatorEnabled(e.target.checked)}
                disabled={controllerState?.connected}
                style={{ width: '18px', height: '18px', cursor: controllerState?.connected ? 'not-allowed' : 'pointer' }}
              />
            </label>
            {controllerState?.connected && (
              <div style={{ marginBottom: '0.5rem', padding: '0.5rem', background: '#2a2a2a', borderRadius: '4px', fontSize: '0.85rem' }}>
                ⚠️ Simulator disabled - Hardware guitar controller detected
              </div>
            )}
            <div className="info-row">
              <span className="info-label">Frets:</span>
              <span style={{ opacity: simulatorEnabled ? 1 : 0.5 }}>1-5 (main), Q-T (solo)</span>
            </div>
            <div className="info-row">
              <span className="info-label">Strum:</span>
              <span style={{ opacity: simulatorEnabled ? 1 : 0.5 }}>Space (down), Arrow Up (up)</span>
            </div>
            <div className="info-row">
              <span className="info-label">Genre:</span>
              <span>D-Pad Up/Down (cycle genres)</span>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
