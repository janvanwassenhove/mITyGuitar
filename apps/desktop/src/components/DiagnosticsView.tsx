import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AudioStats {
  sample_rate: number;
  buffer_size: number;
  underruns: number;
  active_voices: number;
  estimated_latency_ms: number;
}

interface AppConfig {
  soundfonts: {
    current: string | null;
  };
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

interface RawInputEvent {
  timestamp_ms: number;
  unix_timestamp_ms: number;
  gamepad_id: number;
  gamepad_name: string;
  event_type: string;
  button: string | null;
  axis: string | null;
  value: number | null;
  raw_code: string;
}

export default function DiagnosticsView() {
  const [stats, setStats] = useState<AudioStats | null>(null);
  const [currentSoundfont, setCurrentSoundfont] = useState<string | null>(null);
  const [controllerDebug, setControllerDebug] = useState<string>("");
  const [controllerState, setControllerState] = useState<ControllerState | null>(null);
  const [rawDiagnosticsEnabled, setRawDiagnosticsEnabled] = useState<boolean>(false);
  const [rawEvents, setRawEvents] = useState<RawInputEvent[]>([]);
  const [eventCount, setEventCount] = useState<number>(0);
  const [filterEventType, setFilterEventType] = useState<string>("All");
  const [filterText, setFilterText] = useState<string>("");
  const [showOnlyChanges, setShowOnlyChanges] = useState<boolean>(false);

  useEffect(() => {
    const loadStats = async () => {
      try {
        const audioStats = await invoke<AudioStats>("get_audio_stats");
        setStats(audioStats);
      } catch (error) {
        console.error("Failed to load stats:", error);
      }
    };

    loadStats();
    const interval = setInterval(loadStats, 1000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const loadConfig = async () => {
      try {
        const config = await invoke<AppConfig>("get_config");
        setCurrentSoundfont(config.soundfonts.current);
      } catch (error) {
        console.error("Failed to load config:", error);
      }
    };

    loadConfig();
    const interval = setInterval(loadConfig, 2000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const loadDebugInfo = async () => {
      try {
        const info = await invoke<string>("get_controller_debug_info");
        setControllerDebug(info);
      } catch (error) {
        console.error("Failed to load controller debug info:", error);
      }
    };

    loadDebugInfo();
    const interval = setInterval(loadDebugInfo, 100);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const loadControllerState = async () => {
      try {
        const state = await invoke<ControllerState>("get_controller_state");
        setControllerState(state);
      } catch (error) {
        console.error("Failed to load controller state:", error);
      }
    };

    loadControllerState();
    const interval = setInterval(loadControllerState, 50);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const loadRawDiagnostics = async () => {
      if (rawDiagnosticsEnabled) {
        try {
          const events = await invoke<RawInputEvent[]>("get_raw_diagnostics");
          setRawEvents(events);
          const [enabled, count] = await invoke<[boolean, number]>("get_raw_diagnostics_status");
          setEventCount(count);
        } catch (error) {
          console.error("Failed to load raw diagnostics:", error);
        }
      }
    };

    if (rawDiagnosticsEnabled) {
      loadRawDiagnostics();
      const interval = setInterval(loadRawDiagnostics, 100);
      return () => clearInterval(interval);
    }
  }, [rawDiagnosticsEnabled]);

  const toggleRawDiagnostics = async () => {
    try {
      const newState = !rawDiagnosticsEnabled;
      await invoke("set_raw_diagnostics_enabled", { enabled: newState });
      setRawDiagnosticsEnabled(newState);
      if (!newState) {
        setRawEvents([]);
      }
    } catch (error) {
      console.error("Failed to toggle raw diagnostics:", error);
    }
  };

  const clearRawDiagnostics = async () => {
    try {
      await invoke("clear_raw_diagnostics");
      setRawEvents([]);
      setEventCount(0);
    } catch (error) {
      console.error("Failed to clear raw diagnostics:", error);
    }
  };

  const copyToClipboard = () => {
    const text = filteredEvents.map(e => 
      `[${e.timestamp_ms}ms] ${e.event_type} | ${e.button || e.axis || 'N/A'} | ${e.raw_code}`
    ).join('\n');
    navigator.clipboard.writeText(text);
  };

  const filteredEvents = rawEvents.filter(event => {
    if (filterEventType !== "All") {
      if (filterEventType === "Buttons" && !event.event_type.includes("Button")) return false;
      if (filterEventType === "Axes" && event.event_type !== "AxisChanged") return false;
      if (filterEventType === "System" && !["Connected", "Disconnected", "Dropped"].includes(event.event_type)) return false;
    }
    
    if (filterText) {
      const searchText = filterText.toLowerCase();
      const matchesButton = event.button?.toLowerCase().includes(searchText);
      const matchesAxis = event.axis?.toLowerCase().includes(searchText);
      const matchesRawCode = event.raw_code.toLowerCase().includes(searchText);
      if (!matchesButton && !matchesAxis && !matchesRawCode) return false;
    }
    
    return true;
  });

  if (!stats) {
    return <div className="diagnostics-view">Loading...</div>;
  }

  const renderButtonState = (label: string, isPressed: boolean, isMapped: boolean = true) => (
    <div style={{
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      padding: '8px 12px',
      background: isPressed ? 'rgba(74, 222, 128, 0.2)' : 'rgba(255, 255, 255, 0.05)',
      border: `1px solid ${isPressed ? 'rgba(74, 222, 128, 0.5)' : 'rgba(255, 255, 255, 0.1)'}`,
      borderRadius: '6px',
      transition: 'all 0.1s ease'
    }}>
      <span style={{ fontWeight: 500 }}>{label}</span>
      <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
        <span style={{
          fontSize: '11px',
          padding: '2px 6px',
          borderRadius: '3px',
          background: isMapped ? 'rgba(59, 130, 246, 0.3)' : 'rgba(156, 163, 175, 0.3)',
          color: isMapped ? '#93c5fd' : '#9ca3af'
        }}>
          {isMapped ? 'MAPPED' : 'UNMAPPED'}
        </span>
        <span style={{
          width: '60px',
          textAlign: 'center',
          fontWeight: 600,
          color: isPressed ? '#4ade80' : '#6b7280'
        }}>
          {isPressed ? 'PRESSED' : 'OFF'}
        </span>
      </div>
    </div>
  );

  const renderAxisState = (label: string, value: number, isMapped: boolean = true) => (
    <div style={{
      padding: '12px',
      background: 'rgba(255, 255, 255, 0.05)',
      border: '1px solid rgba(255, 255, 255, 0.1)',
      borderRadius: '6px'
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
        <span style={{ fontWeight: 500 }}>{label}</span>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <span style={{
            fontSize: '11px',
            padding: '2px 6px',
            borderRadius: '3px',
            background: isMapped ? 'rgba(59, 130, 246, 0.3)' : 'rgba(156, 163, 175, 0.3)',
            color: isMapped ? '#93c5fd' : '#9ca3af'
          }}>
            {isMapped ? 'MAPPED' : 'UNMAPPED'}
          </span>
          <span style={{ fontWeight: 600, minWidth: '50px', textAlign: 'right' }}>
            {value.toFixed(2)}
          </span>
        </div>
      </div>
      <div style={{
        width: '100%',
        height: '8px',
        background: 'rgba(0, 0, 0, 0.3)',
        borderRadius: '4px',
        overflow: 'hidden'
      }}>
        <div style={{
          width: `${value * 100}%`,
          height: '100%',
          background: value > 0 ? 'linear-gradient(90deg, #3b82f6, #60a5fa)' : 'transparent',
          transition: 'width 0.1s ease'
        }} />
      </div>
    </div>
  );

  return (
    <div className="diagnostics-view" style={{ width: '100%', maxWidth: '1400px', margin: '0 auto' }}>
      {/* Top Row: Audio & Performance */}
      <div style={{ display: 'flex', gap: '24px', marginBottom: '24px' }}>
        <div className="info-panel" style={{ flex: 1 }}>
          <h3 style={{ marginTop: 0 }}>Audio Configuration</h3>
          <div className="info-row">
            <span className="info-label">Sample Rate:</span>
            <span style={{ fontWeight: 600 }}>{stats.sample_rate} Hz</span>
          </div>
          <div className="info-row">
            <span className="info-label">Buffer Size:</span>
            <span style={{ fontWeight: 600 }}>{stats.buffer_size} samples</span>
          </div>
          <div className="info-row">
            <span className="info-label">Estimated Latency:</span>
            <span style={{ fontWeight: 600, color: stats.estimated_latency_ms < 10 ? "#4ade80" : stats.estimated_latency_ms < 20 ? "#fbbf24" : "#f87171" }}>
              {stats.estimated_latency_ms.toFixed(2)} ms
            </span>
          </div>
        </div>

        <div className="info-panel" style={{ flex: 1 }}>
          <h3 style={{ marginTop: 0 }}>Performance</h3>
          <div className="info-row">
            <span className="info-label">Active Voices:</span>
            <span style={{ fontWeight: 600 }}>{stats.active_voices}</span>
          </div>
          <div className="info-row">
            <span className="info-label">Buffer Underruns:</span>
            <span style={{ fontWeight: 600, color: stats.underruns > 0 ? "#f87171" : "#4ade80" }}>
              {stats.underruns}
            </span>
          </div>
          <div className="info-row">
            <span className="info-label">Status:</span>
            <span style={{ fontWeight: 600, color: stats.underruns === 0 ? "#4ade80" : "#f87171" }}>
              {stats.underruns === 0 ? "Healthy" : "Degraded"}
            </span>
          </div>
        </div>
      </div>

      {/* Configuration Row */}
      <div className="info-panel" style={{ marginBottom: '24px' }}>
        <h3 style={{ marginTop: 0 }}>Current Configuration</h3>
        <div style={{ display: 'flex', gap: '48px', flexWrap: 'wrap' }}>
          <div className="info-row" style={{ minWidth: '250px' }}>
            <span className="info-label">Audio Backend:</span>
            <span style={{ fontWeight: 600 }}>{currentSoundfont ? "SoundFont Synth" : "Fallback Synthesizer"}</span>
          </div>
          <div className="info-row" style={{ minWidth: '250px' }}>
            <span className="info-label">SoundFont:</span>
            <span style={{ fontWeight: 600 }}>{currentSoundfont || "None (using fallback synth)"}</span>
          </div>
          <div className="info-row" style={{ minWidth: '250px' }}>
            <span className="info-label">Controller:</span>
            <span style={{ fontWeight: 600, color: controllerState?.connected ? "#4ade80" : "#f87171" }}>
              {controllerState?.connected ? "Connected" : "Disconnected"}
            </span>
          </div>
        </div>
      </div>

      {/* Controller Input Monitor */}
      <div className="info-panel" style={{ marginBottom: '24px' }}>
        <h3 style={{ marginTop: 0 }}>Controller Input Monitor</h3>
        
        {/* Fret Buttons */}
        <div style={{ marginBottom: '20px' }}>
          <h4 style={{ margin: '0 0 12px 0', fontSize: '14px', color: 'rgba(255, 255, 255, 0.7)' }}>Main Fret Buttons</h4>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '8px' }}>
            {renderButtonState('Green', controllerState?.fret_green || false)}
            {renderButtonState('Red', controllerState?.fret_red || false)}
            {renderButtonState('Yellow', controllerState?.fret_yellow || false)}
            {renderButtonState('Blue', controllerState?.fret_blue || false)}
            {renderButtonState('Orange', controllerState?.fret_orange || false)}
          </div>
        </div>

        {/* Solo Buttons */}
        <div style={{ marginBottom: '20px' }}>
          <h4 style={{ margin: '0 0 12px 0', fontSize: '14px', color: 'rgba(255, 255, 255, 0.7)' }}>Solo Fret Buttons</h4>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '8px' }}>
            {renderButtonState('Solo Green', controllerState?.solo_green || false, false)}
            {renderButtonState('Solo Red', controllerState?.solo_red || false, false)}
            {renderButtonState('Solo Yellow', controllerState?.solo_yellow || false, false)}
            {renderButtonState('Solo Blue', controllerState?.solo_blue || false, false)}
            {renderButtonState('Solo Orange', controllerState?.solo_orange || false, false)}
          </div>
        </div>

        {/* Control Buttons */}
        <div style={{ marginBottom: '20px' }}>
          <h4 style={{ margin: '0 0 12px 0', fontSize: '14px', color: 'rgba(255, 255, 255, 0.7)' }}>Control Buttons</h4>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '8px' }}>
            {renderButtonState('Strum Up', controllerState?.strum_up || false)}
            {renderButtonState('Strum Down', controllerState?.strum_down || false)}
            {renderButtonState('D-Pad Up (Genre Next)', controllerState?.dpad_up || false)}
            {renderButtonState('D-Pad Down (Genre Prev)', controllerState?.dpad_down || false)}
            {renderButtonState('D-Pad Left', controllerState?.dpad_left || false)}
            {renderButtonState('D-Pad Right', controllerState?.dpad_right || false)}
            {renderButtonState('Start', controllerState?.start || false, false)}
            {renderButtonState('Select', controllerState?.select || false, false)}
          </div>
        </div>

        {/* Analog Inputs */}
        <div>
          <h4 style={{ margin: '0 0 12px 0', fontSize: '14px', color: 'rgba(255, 255, 255, 0.7)' }}>Analog Inputs</h4>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '12px' }}>
            {renderAxisState('Whammy Bar', controllerState?.whammy_bar || 0)}
            {renderAxisState('Tilt Sensor', 0, false)}
          </div>
        </div>
      </div>

      {/* Hardware Debug Info */}
      <div className="info-panel" style={{ marginBottom: '24px' }}>
        <h3 style={{ marginTop: 0 }}>Hardware Controller Debug</h3>
        <pre style={{ 
          fontFamily: "monospace", 
          fontSize: "0.85em",
          whiteSpace: "pre-wrap",
          margin: 0,
          padding: '12px',
          background: 'rgba(0, 0, 0, 0.3)',
          borderRadius: '6px',
          maxHeight: '200px',
          overflowY: 'auto'
        }}>
          {controllerDebug || 'No controller info available'}
        </pre>
      </div>

      {/* Latency Tips */}
      <div className="info-panel">
        <h3 style={{ marginTop: 0 }}>Performance Tips</h3>
        <ul style={{ paddingLeft: "1.5rem", lineHeight: "1.8", margin: 0 }}>
          <li>
            Current latency is <strong>{stats.estimated_latency_ms < 10 ? "excellent" : stats.estimated_latency_ms < 20 ? "good" : "high"}</strong> ({stats.estimated_latency_ms.toFixed(1)}ms)
          </li>
          <li>For lower latency, reduce buffer size in config (128-256 samples)</li>
          <li>Close other audio applications to free resources</li>
          <li>Use ASIO drivers on Windows for best performance</li>
          {stats.underruns > 0 && (
            <li style={{ color: "#f87171" }}>
              ⚠️ Buffer underruns detected! Try increasing buffer size.
            </li>
          )}
        </ul>
      </div>

      {/* Raw Input Diagnostics */}
      <div className="info-panel" style={{ marginTop: '24px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
          <h3 style={{ margin: 0 }}>Raw Input Diagnostics</h3>
          <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
            <span style={{ fontSize: '14px', color: 'rgba(255, 255, 255, 0.7)' }}>
              {eventCount} events captured
            </span>
            <button
              onClick={toggleRawDiagnostics}
              style={{
                padding: '6px 12px',
                borderRadius: '4px',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                background: rawDiagnosticsEnabled ? 'rgba(74, 222, 128, 0.2)' : 'rgba(255, 255, 255, 0.1)',
                color: rawDiagnosticsEnabled ? '#4ade80' : '#fff',
                cursor: 'pointer',
                fontWeight: 600,
                fontSize: '13px'
              }}
            >
              {rawDiagnosticsEnabled ? '⏸ Disable' : '▶️ Enable'}
            </button>
            <button
              onClick={clearRawDiagnostics}
              style={{
                padding: '6px 12px',
                borderRadius: '4px',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                background: 'rgba(255, 255, 255, 0.1)',
                color: '#fff',
                cursor: 'pointer',
                fontSize: '13px'
              }}
            >
              🗑️ Clear
            </button>
            <button
              onClick={copyToClipboard}
              disabled={filteredEvents.length === 0}
              style={{
                padding: '6px 12px',
                borderRadius: '4px',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                background: 'rgba(255, 255, 255, 0.1)',
                color: filteredEvents.length === 0 ? '#666' : '#fff',
                cursor: filteredEvents.length === 0 ? 'not-allowed' : 'pointer',
                fontSize: '13px'
              }}
            >
              📋 Copy
            </button>
          </div>
        </div>

        {/* Filters */}
        <div style={{ display: 'flex', gap: '12px', marginBottom: '16px', flexWrap: 'wrap' }}>
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
            <label style={{ fontSize: '13px', color: 'rgba(255, 255, 255, 0.7)' }}>Filter:</label>
            <select
              value={filterEventType}
              onChange={(e) => setFilterEventType(e.target.value)}
              style={{
                padding: '4px 8px',
                borderRadius: '4px',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                background: 'rgba(0, 0, 0, 0.3)',
                color: '#fff',
                fontSize: '13px'
              }}
            >
              <option value="All">All Events</option>
              <option value="Buttons">Buttons Only</option>
              <option value="Axes">Axes Only</option>
              <option value="System">System Only</option>
            </select>
          </div>
          
          <input
            type="text"
            placeholder="Search button/axis/code..."
            value={filterText}
            onChange={(e) => setFilterText(e.target.value)}
            style={{
              padding: '4px 12px',
              borderRadius: '4px',
              border: '1px solid rgba(255, 255, 255, 0.2)',
              background: 'rgba(0, 0, 0, 0.3)',
              color: '#fff',
              fontSize: '13px',
              flex: '1',
              minWidth: '200px'
            }}
          />

          <label style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '13px', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={showOnlyChanges}
              onChange={(e) => setShowOnlyChanges(e.target.checked)}
              style={{ width: '14px', height: '14px' }}
            />
            <span>Only show changes</span>
          </label>
        </div>

        {/* Event List */}
        <div style={{
          maxHeight: '400px',
          overflowY: 'auto',
          background: 'rgba(0, 0, 0, 0.3)',
          borderRadius: '6px',
          border: '1px solid rgba(255, 255, 255, 0.1)'
        }}>
          {!rawDiagnosticsEnabled && (
            <div style={{ padding: '24px', textAlign: 'center', color: 'rgba(255, 255, 255, 0.5)' }}>
              Enable raw diagnostics to capture controller events
            </div>
          )}
          
          {rawDiagnosticsEnabled && filteredEvents.length === 0 && (
            <div style={{ padding: '24px', textAlign: 'center', color: 'rgba(255, 255, 255, 0.5)' }}>
              No events captured yet. Press buttons on your controller...
            </div>
          )}

          {rawDiagnosticsEnabled && filteredEvents.length > 0 && (
            <table style={{ width: '100%', fontSize: '12px', fontFamily: 'monospace' }}>
              <thead style={{ position: 'sticky', top: 0, background: '#1a1a1a', borderBottom: '1px solid rgba(255, 255, 255, 0.2)' }}>
                <tr>
                  <th style={{ padding: '8px', textAlign: 'left', fontWeight: 600 }}>Time (ms)</th>
                  <th style={{ padding: '8px', textAlign: 'left', fontWeight: 600 }}>Event Type</th>
                  <th style={{ padding: '8px', textAlign: 'left', fontWeight: 600 }}>Logical</th>
                  <th style={{ padding: '8px', textAlign: 'left', fontWeight: 600 }}>Value</th>
                  <th style={{ padding: '8px', textAlign: 'left', fontWeight: 600 }}>Raw Code</th>
                </tr>
              </thead>
              <tbody>
                {filteredEvents.map((event, idx) => (
                  <tr key={`${event.timestamp_ms}-${idx}`} style={{
                    borderBottom: '1px solid rgba(255, 255, 255, 0.05)',
                    background: idx % 2 === 0 ? 'rgba(255, 255, 255, 0.02)' : 'transparent'
                  }}>
                    <td style={{ padding: '6px 8px', color: 'rgba(255, 255, 255, 0.6)' }}>
                      {event.timestamp_ms}
                    </td>
                    <td style={{ padding: '6px 8px' }}>
                      <span style={{
                        padding: '2px 6px',
                        borderRadius: '3px',
                        fontSize: '11px',
                        background: 
                          event.event_type.includes('Pressed') ? 'rgba(74, 222, 128, 0.2)' :
                          event.event_type.includes('Released') ? 'rgba(239, 68, 68, 0.2)' :
                          event.event_type === 'AxisChanged' ? 'rgba(59, 130, 246, 0.2)' :
                          'rgba(156, 163, 175, 0.2)'
                      }}>
                        {event.event_type}
                      </span>
                    </td>
                    <td style={{ padding: '6px 8px', color: '#60a5fa' }}>
                      {event.button || event.axis || '-'}
                    </td>
                    <td style={{ padding: '6px 8px', color: 'rgba(255, 255, 255, 0.8)' }}>
                      {event.value !== null ? event.value.toFixed(3) : '-'}
                    </td>
                    <td style={{ padding: '6px 8px', color: 'rgba(255, 255, 255, 0.7)', fontSize: '11px' }}>
                      {event.raw_code}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        <div style={{ marginTop: '12px', fontSize: '12px', color: 'rgba(255, 255, 255, 0.6)', lineHeight: '1.6' }}>
          <strong>💡 Tip:</strong> Raw diagnostics show both the logical button mapping (e.g., "DPadUp") and the raw event code. 
          Use this to identify which physical controls map to the same logical inputs.
        </div>
      </div>
    </div>
  );
}
