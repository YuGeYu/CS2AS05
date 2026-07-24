mod commands;
mod errors;
mod models;
mod services;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::cs2::discover_cs2_roots,
            commands::cs2::inspect_cs2_root,
            commands::cs2::install_bot_package,
            commands::cs2::open_upstream_panel,
            commands::cs2::uninstall_bot_package,
            commands::cs2::check_cs2_process,
            commands::cs2::get_diagnostics_payload,
            commands::support::open_official_site,
            commands::support::open_idea_page,
            commands::support::open_release_page,
            commands::support::open_upstream_project,
            commands::support::open_update_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
