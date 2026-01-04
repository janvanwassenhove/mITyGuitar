import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import MappingWizardView from "./MappingWizardView";

interface MappingProfile {
  version: number;
  name: string;
  controller: {
    name: string;
    label: string | null;
    vendor_id: number | null;
    product_id: number | null;
  };
  mappings: Record<string, any>;
  created_at: number;
  modified_at: number;
}

export default function ProfileManagerView() {
  const [profiles, setProfiles] = useState<string[]>([]);
  const [selectedProfile, setSelectedProfile] = useState<string | null>(null);
  const [activeProfile, setActiveProfile] = useState<string | null>(null);
  const [profileData, setProfileData] = useState<MappingProfile | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [newProfileName, setNewProfileName] = useState("");
  const [newControllerName, setNewControllerName] = useState("");
  const [message, setMessage] = useState("");
  const [showMappingWizard, setShowMappingWizard] = useState(false);

  // Load profile list
  const loadProfiles = async () => {
    try {
      const list = await invoke<string[]>("list_mapping_profiles");
      setProfiles(list);
      
      // Get active profile
      const active = await invoke<string | null>("get_active_profile");
      setActiveProfile(active);
    } catch (error) {
      setMessage(`Error loading profiles: ${error}`);
    }
  };

  useEffect(() => {
    loadProfiles();
  }, []);

  // Load a specific profile
  const loadProfile = async (name: string) => {
    try {
      const profile = await invoke<MappingProfile>("load_mapping_profile", { name });
      setProfileData(profile);
      setSelectedProfile(name);
      setActiveProfile(name);
      setMessage(`Loaded profile: ${name}`);
    } catch (error) {
      setMessage(`Error loading profile: ${error}`);
    }
  };

  // Create a new profile
  const createProfile = async () => {
    if (!newProfileName.trim() || !newControllerName.trim()) {
      setMessage("Please enter both profile name and controller name");
      return;
    }

    try {
      const profile = await invoke<MappingProfile>("create_mapping_profile", {
        name: newProfileName,
        controllerName: newControllerName
      });
      setMessage(`Created profile: ${newProfileName}`);
      setIsCreating(false);
      setNewProfileName("");
      setNewControllerName("");
      await loadProfiles();
      setSelectedProfile(profile.name);
      setProfileData(profile);
    } catch (error) {
      setMessage(`Error creating profile: ${error}`);
    }
  };

  // Set active profile
  const setActive = async (name: string) => {
    try {
      await invoke("set_active_profile", { name });
      setActiveProfile(name);
      setMessage(`Set active profile: ${name}`);
    } catch (error) {
      setMessage(`Error setting active profile: ${error}`);
    }
  };

  // Delete profile
  const deleteProfile = async (name: string) => {
    try {
      await invoke("delete_mapping_profile", { name });
      setMessage(`Deleted profile: ${name}`);
      await loadProfiles();
      if (selectedProfile === name) {
        setSelectedProfile(null);
        setProfileData(null);
      }
      if (activeProfile === name) {
        setActiveProfile(null);
      }
    } catch (error) {
      setMessage(`Error deleting profile: ${error}`);
    }
  };

  return (
    <div className="live-view">
      {/* Header */}
      <div className="info-panel" style={{ marginBottom: "2rem" }}>
        <h2 style={{ margin: "0 0 10px 0", color: "var(--color-text-primary)" }}>
          📋 Device Manager
        </h2>
        <p style={{ margin: 0, color: "var(--color-text-secondary)" }}>
          Create, load, and manage controller mapping profiles
        </p>
      </div>

      {/* Message Bar */}
      {message && (
        <div className="info-panel" style={{
          marginBottom: "2rem",
          background: "var(--color-accent-muted)",
          border: "1px solid var(--color-accent)",
          color: "var(--color-text-primary)"
        }}>
          {message}
        </div>
      )}

      {/* Main Content */}
      <div className="control-panel" style={{ alignItems: "stretch", minHeight: "500px" }}>
        {/* Left: Profile List */}
        <div className="info-panel" style={{ width: "300px", minWidth: "300px" }}>
          {/* Create Button */}
          <button
            onClick={() => setIsCreating(!isCreating)}
            className={isCreating ? "button-secondary" : "button-primary"}
            style={{
              width: "100%",
              padding: "12px",
              fontSize: "16px",
              fontWeight: "600",
              marginBottom: "1rem",
              background: isCreating ? "#f87171" : undefined
            }}
          >
            {isCreating ? "Cancel" : "+ New Profile"}
          </button>

          {/* Create Form */}
          {isCreating && (
            <div className="info-panel" style={{
              background: "var(--color-bg-tertiary)",
              marginBottom: "1rem"
            }}>
              <h4 style={{ margin: "0 0 10px 0", color: "var(--color-text-primary)" }}>New Profile</h4>
              <input
                type="text"
                placeholder="Profile Name"
                value={newProfileName}
                onChange={(e) => setNewProfileName(e.target.value)}
                style={{
                  width: "100%",
                  padding: "8px",
                  marginBottom: "10px",
                  fontSize: "14px",
                  background: "rgba(255, 255, 255, 0.1)",
                  border: "1px solid rgba(255, 255, 255, 0.2)",
                  borderRadius: "6px",
                  color: "#fff",
                  boxSizing: "border-box"
                }}
              />
              <input
                type="text"
                placeholder="Controller Name"
                value={newControllerName}
                onChange={(e) => setNewControllerName(e.target.value)}
                style={{
                  width: "100%",
                  padding: "8px",
                  marginBottom: "10px",
                  fontSize: "14px",
                  background: "rgba(255, 255, 255, 0.1)",
                  border: "1px solid rgba(255, 255, 255, 0.2)",
                  borderRadius: "6px",
                  color: "#fff",
                  boxSizing: "border-box"
                }}
              />
              <button
                onClick={createProfile}
                className="button-primary"
                style={{
                  width: "100%",
                  padding: "10px",
                  fontSize: "14px",
                  fontWeight: "600"
                }}
              >
                Create
              </button>
            </div>
          )}

          {/* Profile List */}
          <div style={{ flex: 1, overflowY: "auto", marginTop: "1rem" }}>
            <h3 style={{ margin: "0 0 15px 0", color: "var(--color-text-primary)" }}>
              Profiles ({profiles.length})
            </h3>
            {profiles.length === 0 ? (
              <p style={{ color: "var(--color-text-muted)", textAlign: "center", marginTop: "40px" }}>
                No profiles yet.<br />Create one to get started!
              </p>
            ) : (
              profiles.map((name) => (
                <div
                  key={name}
                  style={{
                    padding: "12px",
                    marginBottom: "8px",
                    borderRadius: "8px",
                    background: selectedProfile === name 
                      ? "var(--color-accent-muted)"
                      : "var(--color-bg-tertiary)",
                    border: selectedProfile === name
                      ? "1px solid var(--color-accent)"
                      : "1px solid var(--color-border)",
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    gap: "10px",
                    transition: "all 0.2s ease"
                  }}
                  onClick={() => loadProfile(name)}
                >
                  <div style={{ flex: 1 }}>
                    <div style={{ color: "var(--color-text-primary)", fontWeight: "500" }}>
                      {name}
                      {activeProfile === name && (
                        <span style={{ 
                          marginLeft: "8px", 
                          fontSize: "12px", 
                          color: "#4ade80" 
                        }}>
                          ● Active
                        </span>
                      )}
                    </div>
                  </div>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      deleteProfile(name);
                    }}
                    style={{
                      padding: "6px 12px",
                      fontSize: "12px",
                      color: "#fff",
                      background: "#f44336",
                      border: "none",
                      borderRadius: "4px",
                      cursor: "pointer"
                    }}
                  >
                    Delete
                  </button>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Right: Profile Details */}
        <div className="info-panel" style={{
          flex: 1,
          overflowY: "auto"
        }}>
          {profileData ? (
            <>
              <div style={{ marginBottom: "20px" }}>
                <h2 style={{ margin: "0 0 10px 0", color: "var(--color-text-primary)" }}>
                  {profileData.name}
                </h2>
                <div style={{ color: "var(--color-text-secondary)", fontSize: "14px" }}>
                  Controller: {profileData.controller.name}
                </div>
                {profileData.controller.label && (
                  <div style={{ color: "var(--color-text-secondary)", fontSize: "14px" }}>
                    Label: {profileData.controller.label}
                  </div>
                )}
              </div>

              {activeProfile !== selectedProfile && (
                <button
                  onClick={() => selectedProfile && setActive(selectedProfile)}
                  className="button-primary"
                  style={{
                    marginBottom: "20px",
                    padding: "12px 24px",
                    fontSize: "14px",
                    fontWeight: "600",
                    background: "#4ade80"
                  }}
                >
                  Set as Active Profile
                </button>
              )}

              <div style={{ marginBottom: "20px" }}>
                <h3 style={{ margin: "0 0 15px 0", color: "var(--color-text-primary)" }}>
                  Mappings ({Object.keys(profileData.mappings).length})
                </h3>
                {Object.keys(profileData.mappings).length === 0 ? (
                  <p style={{ color: "var(--color-text-muted)" }}>
                    No mappings yet. Use the Mapping Wizard to create mappings.
                  </p>
                ) : (
                  <div style={{
                    display: "grid",
                    gridTemplateColumns: "repeat(auto-fill, minmax(250px, 1fr))",
                    gap: "12px"
                  }}>
                    {Object.entries(profileData.mappings).map(([action, binding]: [string, any]) => (
                      <div
                        key={action}
                        className="info-panel"
                        style={{
                          background: "var(--color-bg-tertiary)",
                          border: "1px solid var(--color-border)",
                          padding: "12px"
                        }}
                      >
                        <div style={{ 
                          color: "var(--color-text-primary)", 
                          fontWeight: "500",
                          marginBottom: "6px"
                        }}>
                          {action}
                        </div>
                        <div style={{ 
                          fontSize: "12px", 
                          color: "var(--color-text-muted)",
                          fontFamily: "monospace"
                        }}>
                          {binding.Button ? (
                            <>Button: {binding.Button.logical_button || binding.Button.code}</>
                          ) : binding.Axis ? (
                            <>Axis: {binding.Axis.logical_axis}</>
                          ) : (
                            "Unknown"
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              <div style={{
                paddingTop: "20px",
                borderTop: "1px solid var(--color-border)",
                color: "var(--color-text-muted)",
                fontSize: "12px"
              }}>
                <div>Created: {new Date(profileData.created_at).toLocaleString()}</div>
                <div>Modified: {new Date(profileData.modified_at).toLocaleString()}</div>
              </div>

              {/* Action Buttons */}
              <div className="control-panel" style={{ marginTop: "2rem", paddingTop: "1rem", borderTop: "1px solid var(--color-border)" }}>
                <button
                  onClick={() => setShowMappingWizard(true)}
                  className="button-primary"
                  style={{ marginRight: "10px" }}
                >
                  Configure Mapping
                </button>
                <button
                  onClick={() => {
                    if (window.confirm(`Are you sure you want to delete profile "${selectedProfile}"?`)) {
                      deleteProfile(selectedProfile!);
                    }
                  }}
                  className="button-secondary"
                  style={{
                    color: "#f87171",
                    borderColor: "#f87171",
                    marginRight: "10px"
                  }}
                >
                  Delete Profile
                </button>
                <button
                  onClick={() => window.location.reload()}
                  className="button-secondary"
                >
                  Refresh
                </button>
              </div>
            </>
          ) : (
            <div style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
              color: "var(--color-text-muted)"
            }}>
              <div style={{ fontSize: "48px", marginBottom: "20px" }}>📋</div>
              <p>Select a profile to view details</p>
            </div>
          )}
        </div>
      </div>

      {/* Mapping Wizard Overlay */}
      {showMappingWizard && (
        <div style={{
          position: "fixed",
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: "rgba(0, 0, 0, 0.95)",
          backdropFilter: "blur(4px)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          zIndex: 9999
        }}>
          <div style={{
            width: "90%",
            height: "90%",
            maxWidth: "1200px",
            position: "relative"
          }}>
            <button
              onClick={() => setShowMappingWizard(false)}
              style={{
                position: "absolute",
                top: "10px",
                right: "10px",
                background: "var(--color-bg-secondary)",
                border: "2px solid var(--color-border)",
                borderRadius: "6px",
                color: "var(--color-text-primary)",
                fontSize: "18px",
                fontWeight: "bold",
                padding: "8px 12px",
                cursor: "pointer",
                zIndex: 1001
              }}
            >
              ×
            </button>
            <MappingWizardView />
          </div>
        </div>
      )}
    </div>
  );
}
