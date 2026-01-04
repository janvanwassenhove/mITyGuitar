import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface CaptureState {
  target_action: string | null;
  started_at: number | null;
  duration_ms: number;
  captured_events: any[];
  auto_capture: boolean;
  is_active: boolean;
}

interface CaptureResult {
  success: boolean;
  binding: any | null;
  message: string;
  conflict: string | null;
}

const APP_ACTIONS = [
  // Main Frets
  { name: "FretGreen", display: "Fret Green", category: "Main Frets" },
  { name: "FretRed", display: "Fret Red", category: "Main Frets" },
  { name: "FretYellow", display: "Fret Yellow", category: "Main Frets" },
  { name: "FretBlue", display: "Fret Blue", category: "Main Frets" },
  { name: "FretOrange", display: "Fret Orange", category: "Main Frets" },
  
  // Solo Frets
  { name: "SoloGreen", display: "Solo Green", category: "Solo Frets" },
  { name: "SoloRed", display: "Solo Red", category: "Solo Frets" },
  { name: "SoloYellow", display: "Solo Yellow", category: "Solo Frets" },
  { name: "SoloBlue", display: "Solo Blue", category: "Solo Frets" },
  { name: "SoloOrange", display: "Solo Orange", category: "Solo Frets" },
  
  // Strum
  { name: "StrumUp", display: "Strum Up", category: "Strum" },
  { name: "StrumDown", display: "Strum Down", category: "Strum" },
  
  // D-Pad
  { name: "DPadUp", display: "D-Pad Up", category: "Navigation" },
  { name: "DPadDown", display: "D-Pad Down", category: "Navigation" },
  { name: "DPadLeft", display: "D-Pad Left", category: "Navigation" },
  { name: "DPadRight", display: "D-Pad Right", category: "Navigation" },
  
  // Menu
  { name: "Start", display: "Start", category: "Menu" },
  { name: "Select", display: "Select/Back", category: "Menu" },
  
  // Analog
  { name: "WhammyBar", display: "Whammy Bar", category: "Analog" },
  { name: "TouchStrip", display: "Touch Strip", category: "Analog" },
];

export default function MappingWizardView() {
  const [currentStep, setCurrentStep] = useState(0);
  const [captureState, setCaptureState] = useState<CaptureState | null>(null);
  const [progress, setProgress] = useState(0);
  const [message, setMessage] = useState("");
  const [mappedActions, setMappedActions] = useState<Set<string>>(new Set());
  const [autoCapture, setAutoCapture] = useState(false);
  const [controllerConnected, setControllerConnected] = useState(false);
  const [currentMappings, setCurrentMappings] = useState<Record<string, any>>({});

  const currentAction = APP_ACTIONS[currentStep];

  // Default controller mappings (hardcoded in the controller)
  const getDefaultMapping = (actionName: string): string => {
    const defaultMappings = {
      "FretGreen": "Button: South",
      "FretRed": "Button: East", 
      "FretYellow": "Button: North",
      "FretBlue": "Button: West",
      "FretOrange": "Button: LeftTrigger/LeftTrigger2",
      "SoloGreen": "Button: South",
      "SoloRed": "Button: East",
      "SoloYellow": "Button: North", 
      "SoloBlue": "Button: West",
      "SoloOrange": "Button: LeftTrigger/LeftTrigger2",
      "StrumUp": "Button: DPadUp",
      "StrumDown": "Button: DPadDown",
      "DPadUp": "Button: DPadUp",
      "DPadDown": "Button: DPadDown",
      "DPadLeft": "Button: DPadLeft", 
      "DPadRight": "Button: DPadRight",
      "Start": "Button: Start",
      "Select": "Button: Select",
      "WhammyBar": "Axis: RightStickX",
      "TouchStrip": "Axis: LeftStickY"
    };
    return defaultMappings[actionName] || "No default mapping";
  };

  // Load current mappings for reference
  useEffect(() => {
    const loadCurrentMappings = async () => {
      try {
        const activeProfile = await invoke<string | null>("get_active_profile");
        if (activeProfile) {
          const profileData = await invoke<any>("load_mapping_profile", { name: activeProfile });
          setCurrentMappings(profileData.mappings || {});
        }
      } catch (error) {
        console.error("Failed to load current mappings:", error);
      }
    };

    loadCurrentMappings();
  }, []);

  // Check controller connection status
  useEffect(() => {
    const checkController = async () => {
      try {
        const state = await invoke<any>("get_controller_state");
        setControllerConnected(state.connected || false);
      } catch (error) {
        console.error("Failed to check controller state:", error);
        setControllerConnected(false);
      }
    };

    checkController();
    const interval = setInterval(checkController, 1000);
    return () => clearInterval(interval);
  }, []);

  // Fetch capture state
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const stateStr = await invoke<string>("wizard_get_state");
        const state = JSON.parse(stateStr) as CaptureState;
        setCaptureState(state);

        // Calculate progress
        if (state.is_active && state.started_at) {
          const elapsed = Date.now() - state.started_at;
          const percent = Math.min((elapsed / state.duration_ms) * 100, 100);
          setProgress(percent);
        } else {
          setProgress(0);
        }
      } catch (error) {
        console.error("Failed to get wizard state:", error);
      }
    }, 100);

    return () => clearInterval(interval);
  }, []);

  const startCapture = async () => {
    try {
      setMessage(`Press the ${currentAction.display} button/control...`);
      await invoke("wizard_start_capture", { action: currentAction.name });
    } catch (error) {
      setMessage(`Error: ${error}`);
    }
  };

  const stopCapture = async () => {
    try {
      await invoke("wizard_stop_capture");
      setProgress(0);
    } catch (error) {
      console.error("Failed to stop capture:", error);
    }
  };

  const finalizeCapture = async () => {
    try {
      const result = await invoke<CaptureResult>("wizard_finalize_capture");
      
      if (result.success) {
        setMessage(`✅ ${result.message}`);
        setMappedActions(new Set([...mappedActions, currentAction.name]));
        
        // Auto-advance to next action if auto-capture is enabled
        if (autoCapture && currentStep < APP_ACTIONS.length - 1) {
          setTimeout(() => {
            setCurrentStep(currentStep + 1);
            setTimeout(startCapture, 500);
          }, 1000);
        }
      } else {
        setMessage(`❌ ${result.message}`);
        if (result.conflict) {
          setMessage(`⚠️ Conflict with ${result.conflict}`);
        }
      }
    } catch (error) {
      setMessage(`Error: ${error}`);
    }
  };

  const skipAction = () => {
    if (currentStep < APP_ACTIONS.length - 1) {
      setCurrentStep(currentStep + 1);
      setMessage("");
      setProgress(0);
    }
  };

  const previousAction = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
      setMessage("");
      setProgress(0);
    }
  };

  const toggleAutoCapture = async () => {
    const newValue = !autoCapture;
    setAutoCapture(newValue);
    try {
      await invoke("wizard_set_auto_capture", { enabled: newValue });
    } catch (error) {
      console.error("Failed to set auto-capture:", error);
    }
  };

  const clearWizard = async () => {
    try {
      await invoke("wizard_clear");
      setMessage("");
      setProgress(0);
    } catch (error) {
      console.error("Failed to clear wizard:", error);
    }
  };

  // Group actions by category
  const actionsByCategory = APP_ACTIONS.reduce((acc, action) => {
    if (!acc[action.category]) {
      acc[action.category] = [];
    }
    acc[action.category].push(action);
    return acc;
  }, {} as Record<string, typeof APP_ACTIONS>);

  return (
    <div className="live-view">
      {/* Header */}
      <div className="info-panel" style={{ marginBottom: "2rem" }}>
        <h2 style={{ margin: "0 0 10px 0", color: "var(--color-text-primary)" }}>
          🎸 Controller Mapping Wizard
        </h2>
        <p style={{ margin: "0 0 10px 0", color: "var(--color-text-secondary)" }}>
          Map your guitar controller buttons to app actions step by step
        </p>
        <div style={{ 
          display: "flex", 
          alignItems: "center", 
          gap: "8px",
          fontSize: "14px"
        }}>
          <span style={{ 
            width: "10px", 
            height: "10px", 
            borderRadius: "50%", 
            background: controllerConnected ? "#4ade80" : "#f87171",
            flexShrink: 0
          }}></span>
          <span style={{ color: "var(--color-text-secondary)" }}>
            Controller: {controllerConnected ? "Connected" : "Not Connected"}
          </span>
          {!controllerConnected && (
            <span style={{ color: "#f87171", fontSize: "12px", fontStyle: "italic" }}>
              (Connect a controller to capture inputs)
            </span>
          )}
        </div>
      </div>

      {/* Progress Overview */}
      <div className="info-panel" style={{ marginBottom: "2rem" }}>
        <div style={{ 
          display: "flex", 
          justifyContent: "space-between", 
          marginBottom: "10px",
          color: "var(--color-text-primary)"
        }}>
          <span>Overall Progress</span>
          <span>{mappedActions.size} / {APP_ACTIONS.length} actions mapped</span>
        </div>
        <div style={{
          width: "100%",
          height: "8px",
          background: "var(--color-bg-tertiary)",
          borderRadius: "4px",
          overflow: "hidden"
        }}>
          <div style={{
            width: `${(mappedActions.size / APP_ACTIONS.length) * 100}%`,
            height: "100%",
            background: "var(--color-accent)",
            transition: "width 0.3s ease"
          }} />
        </div>
      </div>

      {/* Main Content */}
      <div className="control-panel" style={{ alignItems: "stretch", minHeight: "500px" }}>
        {/* Left: Action List */}
        <div className="info-panel" style={{ width: "300px", minWidth: "300px" }}>
          <h3 style={{ margin: "0 0 15px 0", color: "var(--color-text-primary)" }}>Actions</h3>
          <div style={{ overflowY: "auto", maxHeight: "500px" }}>
            {Object.entries(actionsByCategory).map(([category, actions]) => (
              <div key={category} style={{ marginBottom: "20px" }}>
                <div style={{ 
                  fontSize: "12px", 
                  color: "var(--color-text-muted)", 
                  marginBottom: "8px",
                  fontWeight: "600",
                  textTransform: "uppercase"
                }}>
                  {category}
                </div>
                {actions.map((action) => {
                  const globalIdx = APP_ACTIONS.indexOf(action);
                  const isCurrent = globalIdx === currentStep;
                  const isMapped = mappedActions.has(action.name);
                  
                  return (
                    <div
                      key={action.name}
                      onClick={() => setCurrentStep(globalIdx)}
                      style={{
                        padding: "10px",
                        marginBottom: "6px",
                        borderRadius: "8px",
                        background: isCurrent 
                          ? "var(--color-accent-muted)" 
                          : "var(--color-bg-tertiary)",
                        border: isCurrent 
                          ? "1px solid var(--color-accent)"
                          : "1px solid var(--color-border)",
                        color: "var(--color-text-primary)",
                        cursor: "pointer",
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "center",
                        transition: "all 0.2s ease"
                      }}
                    >
                      <span>{action.display}</span>
                      {isMapped && <span style={{ color: "#4ade80" }}>✓</span>}
                    </div>
                  );
                })}
              </div>
            ))}
          </div>
        </div>

        {/* Right: Capture Interface */}
        <div style={{
          flex: 1,
          display: "flex",
          flexDirection: "column",
          gap: "1rem"
        }}>
          {/* Current Action Card */}
          <div className="info-panel" style={{ textAlign: "center" }}>
            <div style={{ fontSize: "14px", color: "var(--color-text-muted)", marginBottom: "10px" }}>
              Step {currentStep + 1} of {APP_ACTIONS.length}
            </div>
            <h2 style={{ margin: "0 0 10px 0", color: "var(--color-text-primary)", fontSize: "32px" }}>
              {currentAction.display}
            </h2>
            <div style={{ fontSize: "14px", color: "var(--color-text-secondary)", marginBottom: "15px" }}>
              {currentAction.category}
            </div>
            
            {/* Current Mapping Reference */}
            <div style={{
              padding: "12px",
              background: "var(--color-bg-tertiary)",
              border: "1px solid var(--color-border)",
              borderRadius: "8px",
              textAlign: "left"
            }}>
              <div style={{ 
                fontSize: "12px", 
                color: "var(--color-text-muted)", 
                marginBottom: "6px",
                textTransform: "uppercase",
                letterSpacing: "0.5px"
              }}>
                Current Mapping:
              </div>
              <div style={{ 
                fontSize: "14px", 
                color: "var(--color-text-primary)",
                fontFamily: "monospace"
              }}>
                {(() => {
                  const profileMapping = currentMappings[currentAction.name];
                  if (profileMapping) {
                    // Show profile-specific mapping
                    if (profileMapping.Button) {
                      return (
                        <div>
                          <span style={{ color: "#4ade80" }}>Profile: </span>
                          Button: {profileMapping.Button.logical_button || profileMapping.Button.code || 'Unknown'}
                        </div>
                      );
                    } else if (profileMapping.Axis) {
                      return (
                        <div>
                          <span style={{ color: "#4ade80" }}>Profile: </span>
                          Axis: {profileMapping.Axis.logical_axis || 'Unknown'}
                        </div>
                      );
                    }
                  }
                  
                  // Show default mapping
                  const defaultMapping = getDefaultMapping(currentAction.name);
                  return (
                    <div>
                      <span style={{ color: "#60a5fa" }}>Default: </span>
                      {defaultMapping}
                    </div>
                  );
                })()}
              </div>
            </div>
          </div>

          {/* Capture Progress */}
          {captureState?.is_active && (
            <div className="info-panel" style={{
              background: "var(--color-accent-muted)",
              border: "1px solid var(--color-accent)"
            }}>
              <div style={{ 
                marginBottom: "10px", 
                color: "var(--color-text-primary)",
                display: "flex",
                justifyContent: "space-between"
              }}>
                <span>Capturing...</span>
                <span>{captureState.captured_events.length} events</span>
              </div>
              <div style={{
                width: "100%",
                height: "6px",
                background: "var(--color-bg-tertiary)",
                borderRadius: "3px",
                overflow: "hidden"
              }}>
                <div style={{
                  width: `${progress}%`,
                  height: "100%",
                  background: "linear-gradient(90deg, #4CAF50 0%, #8BC34A 100%)",
                  transition: "width 0.1s linear"
                }} />
              </div>
            </div>
          )}

          {/* Message */}
          {message && (
            <div style={{
              padding: "15px",
              background: "rgba(255, 255, 255, 0.05)",
              borderRadius: "12px",
              border: "1px solid rgba(255, 255, 255, 0.1)",
              color: "#fff"
            }}>
              {message}
            </div>
          )}

          {/* Captured Events Preview */}
          {!captureState?.is_active && captureState?.captured_events && captureState.captured_events.length > 0 && (
            <div className="info-panel" style={{ marginTop: "15px" }}>
              <h4 style={{ margin: "0 0 10px 0", color: "var(--color-text-primary)" }}>
                Captured {captureState.captured_events.length} events:
              </h4>
              <div style={{ 
                maxHeight: "120px", 
                overflowY: "auto", 
                marginBottom: "15px",
                background: "var(--color-bg-tertiary)",
                borderRadius: "6px",
                padding: "10px"
              }}>
                {captureState.captured_events.map((event, idx) => (
                  <div key={idx} style={{
                    fontSize: "12px",
                    fontFamily: "monospace",
                    color: "var(--color-text-muted)",
                    marginBottom: "4px",
                    borderBottom: idx < captureState.captured_events.length - 1 ? "1px solid var(--color-border)" : "none",
                    paddingBottom: "4px"
                  }}>
                    <span style={{ color: "#4ade80" }}>{event.event_type}</span>
                    {event.button && <span style={{ color: "#60a5fa" }}> {event.button}</span>}
                    {event.axis && <span style={{ color: "#f59e0b" }}> {event.axis}={event.value?.toFixed(3)}</span>}
                  </div>
                ))}
              </div>
              <div style={{
                padding: "10px",
                background: "var(--color-accent-muted)",
                border: "1px solid var(--color-accent)",
                borderRadius: "6px",
                color: "var(--color-text-primary)"
              }}>
                <strong>Will create binding:</strong><br />
                <span style={{ fontSize: "14px", fontFamily: "monospace" }}>
                  {(() => {
                    const buttonEvents = captureState.captured_events.filter(e => e.button);
                    const axisEvents = captureState.captured_events.filter(e => e.axis);
                    
                    if (buttonEvents.length > 0) {
                      const buttonCounts = {};
                      buttonEvents.forEach(e => {
                        buttonCounts[e.button] = (buttonCounts[e.button] || 0) + 1;
                      });
                      const mostCommon = Object.entries(buttonCounts).sort(([,a], [,b]) => b - a)[0];
                      return `Button: ${mostCommon[0]} (detected ${mostCommon[1]} times)`;
                    } else if (axisEvents.length > 0) {
                      const axis = axisEvents[0];
                      return `Axis: ${axis.axis} (range: ${Math.min(...axisEvents.map(e => e.value)).toFixed(3)} to ${Math.max(...axisEvents.map(e => e.value)).toFixed(3)})`;
                    }
                    return "Unknown binding type";
                  })()}
                </span>
              </div>
            </div>
          )}

          {/* Controls */}
          <div className="info-panel">
            <div style={{ display: "flex", gap: "10px", marginBottom: "15px" }}>
              <button
                onClick={startCapture}
                disabled={captureState?.is_active || !controllerConnected}
                className="button-primary"
                style={{
                  flex: 1,
                  padding: "15px",
                  fontSize: "16px",
                  fontWeight: "600",
                  opacity: captureState?.is_active || !controllerConnected ? 0.5 : 1,
                  cursor: captureState?.is_active || !controllerConnected ? "not-allowed" : "pointer"
                }}
              >
                {!controllerConnected ? "No Controller" : 
                 captureState?.is_active ? "Capturing..." : "Start Capture"}
              </button>

              {captureState?.is_active && (
                <button
                  onClick={stopCapture}
                  style={{
                    flex: 1,
                    padding: "12px 24px",
                    fontSize: "16px",
                    fontWeight: "600",
                    color: "#fff",
                    background: "#f87171",
                    border: "none",
                    borderRadius: "8px",
                    cursor: "pointer"
                  }}
                >
                  Stop
                </button>
              )}

              {!captureState?.is_active && captureState?.captured_events && captureState.captured_events.length > 0 && (
                <button
                  onClick={finalizeCapture}
                  className="button-primary"
                  style={{
                    flex: 1,
                    padding: "12px 24px",
                    fontSize: "16px",
                    fontWeight: "600",
                    background: "#4ade80"
                  }}
                >
                  Accept
                </button>
              )}
            </div>

            <div style={{ display: "flex", gap: "10px" }}>
              <button
                onClick={previousAction}
                disabled={currentStep === 0}
                className="button-secondary"
                style={{
                  flex: 1,
                  padding: "10px",
                  fontSize: "14px",
                  cursor: currentStep === 0 ? "not-allowed" : "pointer",
                  opacity: currentStep === 0 ? 0.5 : 1
                }}
              >
                ← Previous
              </button>

              <button
                onClick={skipAction}
                disabled={currentStep === APP_ACTIONS.length - 1}
                className="button-secondary"
                style={{
                  flex: 1,
                  padding: "10px",
                  fontSize: "14px",
                  cursor: currentStep === APP_ACTIONS.length - 1 ? "not-allowed" : "pointer",
                  opacity: currentStep === APP_ACTIONS.length - 1 ? 0.5 : 1
                }}
              >
                Skip →
              </button>
            </div>

            <div style={{ 
              display: "flex", 
              alignItems: "center", 
              gap: "10px",
              marginTop: "15px",
              paddingTop: "15px",
              borderTop: "1px solid var(--color-border)"
            }}>
              <label style={{ display: "flex", alignItems: "center", gap: "8px", color: "var(--color-text-primary)", cursor: "pointer" }}>
                <input
                  type="checkbox"
                  checked={autoCapture}
                  onChange={toggleAutoCapture}
                  style={{ width: "18px", height: "18px" }}
                />
                Auto-advance to next action
              </label>

              <button
                onClick={clearWizard}
                className="button-secondary"
                style={{
                  marginLeft: "auto",
                  padding: "8px 16px",
                  fontSize: "14px"
                }}
              >
                Clear
              </button>
            </div>
          </div>

          {/* Completion Status */}
          {mappedActions.size === APP_ACTIONS.length && (
            <div className="info-panel" style={{ 
              border: "1px solid #4ade80",
              background: "rgba(74, 222, 128, 0.1)"
            }}>
              <h3 style={{ color: "#4ade80", margin: "0 0 10px 0" }}>🎉 Mapping Complete!</h3>
              <p style={{ margin: "0 0 15px 0", color: "var(--color-text-secondary)" }}>
                All actions have been mapped. You can now use your custom controller configuration.
              </p>
              <div className="control-panel">
                <button
                  onClick={() => {
                    // TODO: Implement save functionality
                    setMessage("Mapping complete! Use the Device Manager to save.");
                  }}
                  className="button-primary"
                  style={{ background: "#4ade80" }}
                >
                  Complete Mapping
                </button>
                <button
                  onClick={() => window.location.reload()}
                  className="button-secondary"
                >
                  Return to Live View
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
