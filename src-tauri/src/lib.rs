mod commands;
mod errors;
mod models;
mod services;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            services::demo::initialize(app.handle()).map_err(|error| error.into_string())?;
            services::demo::refresh_watcher(app.handle()).map_err(|error| error.into_string())?;
            Ok(())
        })
        .manage(services::cs2_discovery::ScanCoordinator::default())
        .manage(services::demo::DemoWatcherState::default())
        .invoke_handler(tauri::generate_handler![
            commands::intro::get_intro_public_data,
            commands::demo::list_demo_roots,
            commands::demo::add_demo_root,
            commands::demo::ensure_default_demo_root,
            commands::demo::update_demo_root,
            commands::demo::remove_demo_root,
            commands::demo::scan_demo_roots,
            commands::demo::import_demo_file,
            commands::demo::list_demos,
            commands::demo::get_demo_report,
            commands::demo::retry_demo_parse,
            commands::demo::get_demo_settings,
            commands::demo::set_demo_recording_enabled,
            commands::cs2::discover_cs2_roots,
            commands::cs2::inspect_cs2_root,
            commands::cs2::install_bot_package,
            commands::cs2::open_upstream_panel,
            commands::cs2::uninstall_bot_package,
            commands::cs2::check_cs2_process,
            commands::cs2::get_diagnostics_payload,
            commands::cs2::guess_cs2_roots,
            commands::cs2::stop_guess_cs2_roots,
            commands::panel::get_panel_snapshot,
            commands::panel::initialize_panel_defaults,
            commands::panel::set_panel_mode,
            commands::panel::set_panel_difficulty,
            commands::panel::set_panel_aim,
            commands::panel::set_panel_nades,
            commands::panel::set_panel_bot_item,
            commands::panel::set_panel_drop_knives,
            commands::panel::launch_panel_cs2,
            commands::support::open_official_site,
            commands::support::open_idea_page,
            commands::support::open_release_page,
            commands::support::open_upstream_project,
            commands::support::open_update_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
