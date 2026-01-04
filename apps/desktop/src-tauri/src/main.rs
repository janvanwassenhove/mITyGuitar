// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;
mod commands;
mod song_player;

use state::AppState;
use tauri::{Manager, menu::{Menu, MenuItem}, tray::{TrayIconBuilder, TrayIconEvent}};

fn main() {
    env_logger::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize application state
            let state = AppState::new()?;
            app.manage(state);
            
            // Create system tray menu
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            
            // Build system tray - Tauri will use the icon specified in tauri.conf.json
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("mITyGuitar")
                .menu(&menu)
                .tooltip("mITyGuitar")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            if let Some(app) = tray.app_handle().get_webview_window("main") {
                                let _ = app.show();
                                let _ = app.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;
            
            log::info!("mITyGuitar initialized with system tray");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_controller_state,
            commands::simulator_key_down,
            commands::simulator_key_up,
            commands::set_genre,
            commands::next_pattern,
            commands::prev_pattern,
            commands::next_instrument,
            commands::prev_instrument,
            commands::panic_all_notes_off,
            commands::quit_app,
            commands::get_audio_stats,
            commands::get_config,
            commands::save_config,
            commands::get_genres,
            commands::get_current_genre_info,
            commands::get_available_instruments,
            commands::get_available_soundfonts,
            commands::set_soundfont,
            commands::rescan_soundfonts,
            commands::upload_soundfont,
            commands::check_hardware_controller,
            commands::get_controller_debug_info,
            commands::check_audio_health,
            commands::set_release_multiplier,
            commands::set_sustain_enabled,
            commands::set_sustain_release_time,
            // New chord mapping commands
            commands::get_chord_mapping,
            commands::update_chord_override,
            commands::update_chord_mapping_settings,
            commands::get_app_config,
            // Raw diagnostics commands
            commands::set_raw_diagnostics_enabled,
            commands::get_raw_diagnostics,
            commands::clear_raw_diagnostics,
            commands::get_raw_diagnostics_status,
            // Mapping wizard commands
            commands::wizard_start_capture,
            commands::wizard_stop_capture,
            commands::wizard_finalize_capture,
            commands::wizard_get_state,
            commands::wizard_set_auto_capture,
            commands::wizard_clear,
            // Mapping profile commands
            commands::list_mapping_profiles,
            commands::load_mapping_profile,
            commands::save_mapping_profile,
            commands::create_mapping_profile,
            commands::delete_mapping_profile,
            commands::set_active_profile,
            commands::get_active_profile,
            commands::update_profile_mapping,
            // Song play commands
            commands::song_load_chart,
            commands::song_load_default_chart,
            commands::song_load_chart_from_path,
            commands::song_get_chart,
            commands::song_play,
            commands::song_pause,
            commands::song_stop,
            commands::song_seek,
            commands::song_set_speed,
            commands::song_get_transport_state,
            commands::song_check_strum,
            commands::song_update_sustain,
            commands::song_get_score,
            commands::song_set_instrument,
            commands::song_clear_instrument_override,
            // Song library commands
            commands::song_save_to_library,
            commands::song_list_library,
            commands::song_load_from_library,
            commands::song_delete_from_library,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
