mod commands;
mod demo;
mod errors;
mod models;
#[cfg(debug_assertions)]
mod preflight;
mod services;

use tauri::Manager;

macro_rules! app_invoke_handler {
    ($($extra:path),* $(,)?) => {
        tauri::generate_handler![
            commands::intro::get_intro_public_data,
            commands::demo::list_demo_roots,
            commands::demo::play_demo,
            commands::demo::reveal_demo_file,
            commands::demo::add_demo_root,
            commands::demo::ensure_default_demo_root,
            commands::demo::update_demo_root,
            commands::demo::remove_demo_root,
            commands::demo::delete_demo_file,
            commands::demo::scan_demo_roots,
            commands::demo::import_demo_file,
            commands::demo::list_demos,
            commands::demo::list_analysis_jobs,
            commands::demo::cancel_analysis_job,
            commands::demo::retry_analysis_job,
            commands::demo::delete_analysis_job,
            commands::demo::get_match_overview,
            commands::demo::get_match_scoreboard,
            commands::demo::get_match_performance_radar,
            commands::demo::get_match_rounds,
            commands::demo::get_match_economy,
            commands::demo::get_match_duels,
            commands::demo::get_match_utility,
            commands::demo::get_match_events,
            commands::demo::get_player_match_detail,
            commands::demo::export_match,
            commands::demo::ensure_spatial_analysis,
            commands::demo::get_round_positions,
            commands::demo::get_heatmap_points,
            commands::demo::save_heatmap_png,
            commands::demo::launch_demo_at_tick,
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
            commands::cs2::get_cs2_process_snapshot,
            commands::cs2::close_cs2,
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
            commands::bot_difficulty::list_bot_profiles,
            commands::bot_difficulty::open_bot_profile,
            commands::bot_difficulty::get_bot_workshop_state,
            commands::bot_difficulty::list_vpk_entries,
            commands::bot_difficulty::create_bot_profile,
            commands::bot_difficulty::save_bot_profile,
            commands::bot_difficulty::rename_bot_profile,
            commands::bot_difficulty::delete_bot_profile,
            commands::bot_difficulty::apply_bot_profile,
            commands::map_rotation::get_map_rotation_default,
            commands::map_rotation::set_map_rotation_default,
            commands::map_rotation::reset_map_rotation_default,
            commands::support::open_official_site,
            commands::support::open_idea_page,
            commands::support::open_api_purchase,
            commands::support::get_ai_connection,
            commands::support::save_ai_connection,
            commands::support::get_ai_chat_sessions,
            commands::support::save_ai_chat_sessions,
            commands::support::get_ai_models,
            commands::support::run_ai_powershell,
            commands::support::chat_ai,
            commands::support::open_fault_idea_page,
            commands::support::open_release_page,
            commands::support::open_upstream_project,
            commands::support::open_reference_project,
            commands::support::open_update_download,
            commands::support::get_assistant_preferences,
            commands::support::set_assistant_autostart,
            commands::support::clear_assistant_data,
            commands::support::submit_fault_report,
            commands::support::get_assistant_account,
            commands::support::login_assistant,
            commands::support::logout_assistant,
            commands::inventory_simulator::inventory_simulator_get_status,
            commands::inventory_simulator::inventory_simulator_install,
            commands::inventory_simulator::inventory_simulator_remove,
            commands::inventory_simulator::inventory_simulator_open_workshop,
            commands::inventory_simulator::inventory_simulator_check_service,
            commands::scoreboard::open_scoreboard,
            commands::scoreboard::scoreboard_frontend_ready,
            commands::scoreboard::scoreboard_present,
            commands::scoreboard::hide_scoreboard,
            commands::scoreboard::destroy_scoreboard,
            commands::scoreboard::report_scoreboard_boot_error,
            $($extra),*
        ]
    };
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    preflight::start().expect("failed to initialize demo preflight diagnostics");

    let builder = tauri::Builder::default()
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
        }));
    let builder = builder
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
            demo::coordinator::start(app.handle());
            demo::post_match::start(app.handle());
            Ok(())
        })
        .manage(services::cs2_discovery::ScanCoordinator::default())
        .manage(services::demo::DemoWatcherState::default())
        .manage(demo::playback::DemoPlaybackState::default())
        .manage(demo::post_match::GameSessionCoordinator::default())
        .manage(commands::scoreboard::ScoreboardState::default());

    #[cfg(debug_assertions)]
    let builder = builder.invoke_handler(app_invoke_handler![
        preflight::preflight_status,
        preflight::preflight_record_event,
        preflight::preflight_exit_app,
    ]);
    #[cfg(not(debug_assertions))]
    let builder = builder.invoke_handler(app_invoke_handler![]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
