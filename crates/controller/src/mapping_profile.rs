use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};

/// Version for mapping profile schema
const MAPPING_PROFILE_VERSION: u32 = 1;

/// App-level action that can be triggered by controller input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppAction {
    // Main fret buttons
    FretGreen,
    FretRed,
    FretYellow,
    FretBlue,
    FretOrange,
    
    // Solo fret buttons
    SoloGreen,
    SoloRed,
    SoloYellow,
    SoloBlue,
    SoloOrange,
    
    // Strum
    StrumUp,
    StrumDown,
    
    // D-pad
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    
    // Menu
    Start,
    Select,
    System,
    
    // Analog axes
    WhammyAxis,
    TiltAxis,
    GenericAxis1,
    GenericAxis2,
}

impl AppAction {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::FretGreen => "Green Fret",
            Self::FretRed => "Red Fret",
            Self::FretYellow => "Yellow Fret",
            Self::FretBlue => "Blue Fret",
            Self::FretOrange => "Orange Fret",
            Self::SoloGreen => "Solo Green",
            Self::SoloRed => "Solo Red",
            Self::SoloYellow => "Solo Yellow",
            Self::SoloBlue => "Solo Blue",
            Self::SoloOrange => "Solo Orange",
            Self::StrumUp => "Strum Up",
            Self::StrumDown => "Strum Down",
            Self::DPadUp => "D-Pad Up",
            Self::DPadDown => "D-Pad Down",
            Self::DPadLeft => "D-Pad Left",
            Self::DPadRight => "D-Pad Right",
            Self::Start => "Start",
            Self::Select => "Select",
            Self::System => "System",
            Self::WhammyAxis => "Whammy Bar",
            Self::TiltAxis => "Tilt Sensor",
            Self::GenericAxis1 => "Generic Axis 1",
            Self::GenericAxis2 => "Generic Axis 2",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Self::FretGreen | Self::FretRed | Self::FretYellow | Self::FretBlue | Self::FretOrange => "Main Frets",
            Self::SoloGreen | Self::SoloRed | Self::SoloYellow | Self::SoloBlue | Self::SoloOrange => "Solo Frets",
            Self::StrumUp | Self::StrumDown => "Strum",
            Self::DPadUp | Self::DPadDown | Self::DPadLeft | Self::DPadRight => "D-Pad",
            Self::Start | Self::Select | Self::System => "Menu",
            Self::WhammyAxis | Self::TiltAxis | Self::GenericAxis1 | Self::GenericAxis2 => "Analog",
        }
    }

    pub fn all_actions() -> Vec<Self> {
        vec![
            // Main frets
            Self::FretGreen, Self::FretRed, Self::FretYellow, Self::FretBlue, Self::FretOrange,
            // Solo frets
            Self::SoloGreen, Self::SoloRed, Self::SoloYellow, Self::SoloBlue, Self::SoloOrange,
            // Strum
            Self::StrumUp, Self::StrumDown,
            // D-pad
            Self::DPadUp, Self::DPadDown, Self::DPadLeft, Self::DPadRight,
            // Menu
            Self::Start, Self::Select, Self::System,
            // Analog
            Self::WhammyAxis, Self::TiltAxis, Self::GenericAxis1, Self::GenericAxis2,
        ]
    }
}

/// Raw binding signature for a button
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonBinding {
    /// Raw event code as string (Debug format)
    pub code: String,
    /// Logical button name (optional, for reference)
    pub logical_button: Option<String>,
}

/// Raw binding signature for an axis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisBinding {
    /// Raw event code as string (Debug format, optional)
    pub code: Option<String>,
    /// Logical axis name
    pub logical_axis: String,
    /// Minimum observed value during capture
    pub min: f32,
    /// Maximum observed value during capture
    pub max: f32,
    /// Deadzone threshold (0.0-1.0)
    pub deadzone: f32,
    /// Invert axis direction
    pub invert: bool,
}

/// Raw event binding (button or axis)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RawBinding {
    #[serde(rename = "button")]
    Button(ButtonBinding),
    #[serde(rename = "axis")]
    Axis(AxisBinding),
}

/// Controller identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerId {
    /// Gamepad name from gilrs
    pub name: String,
    /// Optional user-defined label
    pub label: Option<String>,
    /// Vendor ID if available
    pub vendor_id: Option<u16>,
    /// Product ID if available
    pub product_id: Option<u16>,
}

/// Complete mapping profile for a controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProfile {
    /// Schema version
    pub version: u32,
    /// Profile name
    pub name: String,
    /// Controller identifier
    pub controller: ControllerId,
    /// Mappings from AppAction to RawBinding
    pub mappings: HashMap<AppAction, RawBinding>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
}

impl MappingProfile {
    pub fn new(name: String, controller: ControllerId) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            version: MAPPING_PROFILE_VERSION,
            name,
            controller,
            mappings: HashMap::new(),
            created_at: now,
            modified_at: now,
        }
    }

    pub fn add_mapping(&mut self, action: AppAction, binding: RawBinding) {
        self.mappings.insert(action, binding);
        self.update_modified_time();
    }

    pub fn remove_mapping(&mut self, action: &AppAction) {
        self.mappings.remove(action);
        self.update_modified_time();
    }

    pub fn get_binding(&self, action: &AppAction) -> Option<&RawBinding> {
        self.mappings.get(action)
    }

    fn update_modified_time(&mut self) {
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Find which action is bound to a given raw signature
    pub fn find_action_for_signature(&self, signature: &str) -> Option<AppAction> {
        for (action, binding) in &self.mappings {
            let matches = match binding {
                RawBinding::Button(btn) => btn.code == signature,
                RawBinding::Axis(ax) => {
                    ax.logical_axis == signature || ax.code.as_ref() == Some(&signature.to_string())
                }
            };
            if matches {
                return Some(*action);
            }
        }
        None
    }
}

/// Manager for mapping profiles
pub struct MappingProfileManager {
    profiles_dir: PathBuf,
    active_profile: Option<MappingProfile>,
}

impl MappingProfileManager {
    pub fn new(profiles_dir: PathBuf) -> Result<Self> {
        // Create profiles directory if it doesn't exist
        if !profiles_dir.exists() {
            fs::create_dir_all(&profiles_dir)
                .context("Failed to create profiles directory")?;
        }

        Ok(Self {
            profiles_dir,
            active_profile: None,
        })
    }

    /// List all available profile files
    pub fn list_profiles(&self) -> Result<Vec<String>> {
        let mut profiles = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.profiles_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Some(name) = entry.path().file_stem() {
                            profiles.push(name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        
        profiles.sort();
        Ok(profiles)
    }

    /// Load a profile by name
    pub fn load_profile(&mut self, name: &str) -> Result<()> {
        let path = self.get_profile_path(name);
        let content = fs::read_to_string(&path)
            .context(format!("Failed to read profile: {}", name))?;
        
        let profile: MappingProfile = serde_json::from_str(&content)
            .context("Failed to parse profile JSON")?;
        
        log::info!("📋 Loaded mapping profile: {}", name);
        self.active_profile = Some(profile);
        Ok(())
    }

    /// Save the active profile
    pub fn save_active_profile(&self) -> Result<()> {
        let profile = self.active_profile.as_ref()
            .context("No active profile to save")?;
        
        self.save_profile(profile)
    }

    /// Save a profile
    pub fn save_profile(&self, profile: &MappingProfile) -> Result<()> {
        let path = self.get_profile_path(&profile.name);
        let json = serde_json::to_string_pretty(profile)
            .context("Failed to serialize profile")?;
        
        fs::write(&path, json)
            .context("Failed to write profile file")?;
        
        log::info!("💾 Saved mapping profile: {}", profile.name);
        Ok(())
    }

    /// Delete a profile
    pub fn delete_profile(&self, name: &str) -> Result<()> {
        let path = self.get_profile_path(name);
        fs::remove_file(&path)
            .context(format!("Failed to delete profile: {}", name))?;
        
        log::info!("🗑️ Deleted mapping profile: {}", name);
        Ok(())
    }

    /// Get the active profile
    pub fn active_profile(&self) -> Option<&MappingProfile> {
        self.active_profile.as_ref()
    }

    /// Get mutable reference to active profile
    pub fn active_profile_mut(&mut self) -> Option<&mut MappingProfile> {
        self.active_profile.as_mut()
    }

    /// Set active profile
    pub fn set_active_profile(&mut self, profile: MappingProfile) {
        self.active_profile = Some(profile);
    }

    fn get_profile_path(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{}.json", name))
    }

    /// Create a default profile for a controller
    pub fn create_default_profile(&self, controller: ControllerId) -> MappingProfile {
        MappingProfile::new("Default".to_string(), controller)
    }
}

/// Generate a unique signature string for matching raw events
pub fn generate_button_signature(code: &str, logical_button: Option<&str>) -> String {
    format!("btn:{}:{}", code, logical_button.unwrap_or("?"))
}

/// Generate a unique signature string for axis events
pub fn generate_axis_signature(logical_axis: &str, code: Option<&str>) -> String {
    if let Some(c) = code {
        format!("axis:{}:{}", logical_axis, c)
    } else {
        format!("axis:{}", logical_axis)
    }
}
