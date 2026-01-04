# Controller Mapping Wizard Implementation Status

![Controller Mapping Workflow](images/controller-flow.png)
*Controller input processing and mapping workflow*

## ✅ Completed Backend (Rust)

1. **Data Structures** (`mapping_profile.rs`)
   - `AppAction` enum with all possible controller actions
   - `ButtonBinding` and `AxisBinding` structs
   - `RawBinding` enum supporting both types
   - `MappingProfile` with versioning and persistence
   - `MappingProfileManager` for loading/saving/managing profiles

2. **Capture Wizard** (`mapping_wizard.rs`)
   - `MappingWizard` for guided capture flow
   - `CaptureState` tracking current capture session
   - `CaptureResult` with conflict detection
   - Auto-capture mode support
   - Event analysis and binding generation

3. **Integration**
   - Added to `PerformanceController`
   - Event recording in polling loop
   - Both diagnostics and wizard record events

## 🚧 Remaining Work

### Backend (Rust) - Tauri Commands
Add to `apps/desktop/src-tauri/src/commands.rs`:

```rust
// Mapping Wizard Commands
#[tauri::command]
pub fn wizard_start_capture(action: String, state: State<AppState>) -> Result<(), String>

#[tauri::command]
pub fn wizard_stop_capture(state: State<AppState>) -> Result<(), String>

#[tauri::command]
pub fn wizard_finalize_capture(state: State<AppState>) -> Result<CaptureResult, String>

#[tauri::command]
pub fn wizard_get_state(state: State<AppState>) -> Result<CaptureState, String>

#[tauri::command]
pub fn wizard_set_auto_capture(enabled: bool, state: State<AppState>) -> Result<(), String>

// Profile Management Commands
#[tauri::command]
pub fn list_mapping_profiles(state: State<AppState>) -> Result<Vec<String>, String>

#[tauri::command]
pub fn load_mapping_profile(name: String, state: State<AppState>) -> Result<(), String>

#[tauri::command]
pub fn save_mapping_profile(profile: MappingProfile, state: State<AppState>) -> Result<(), String>

#[tauri::command]
pub fn create_mapping_profile(name: String, controller_name: String, state: State<AppState>) -> Result<MappingProfile, String>

#[tauri::command]
pub fn delete_mapping_profile(name: String, state: State<AppState>) -> Result<(), String>

#[tauri::command]
pub fn get_active_profile(state: State<AppState>) -> Result<Option<MappingProfile>, String>

#[tauri::command]
pub fn update_profile_mapping(action: AppAction, binding: RawBinding, state: State<AppState>) -> Result<(), String>
```

### Frontend (React) - Components

1. **MappingWizardView.tsx** - Main wizard component
2. **CaptureStep.tsx** - Individual capture step
3. **ProfileManager.tsx** - Profile list/create/delete
4. **TestMappingView.tsx** - Test the mappings

### AppState Integration
Add to `apps/desktop/src-tauri/src/state.rs`:
- MappingProfileManager field
- Load/save active profile
- Apply mappings to input processing

## 📋 Next Steps Priority

1. Add Tauri commands to commands.rs
2. Register commands in main.rs  
3. Add MappingProfileManager to AppState
4. Create React components
5. Add wizard to navigation menu

## 🎯 Key Features Implemented

✅ Complete data model with versioning
✅ Profile persistence (JSON)
✅ Capture wizard logic with auto-capture
✅ Event analysis and binding generation
✅ Conflict detection framework
✅ Integration with existing diagnostics

## 📝 Notes

- The wizard records events through the same pipeline as raw diagnostics
- Profiles stored in JSON for easy editing/sharing
- Supports button AND axis mapping
- Deadzone and invert support for axes
- Collision detection for duplicate bindings
