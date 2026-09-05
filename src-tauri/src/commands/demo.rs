use tauri::AppHandle;

use crate::{
    demo::playback,
    models::demo::*,
    services::{demo, panel},
};

#[tauri::command]
pub fn play_demo(
    app: AppHandle,
    state: tauri::State<'_, playback::DemoPlaybackState>,
    demo_id: i64,
    root_path: String,
) -> Result<DemoPlaybackResult, String> {
    playback::play_demo(&app, &state, demo_id, &root_path, None).map_err(|e| e.into_string())
}

#[tauri::command]
pub fn reveal_demo_file(app: AppHandle, demo_id: i64) -> Result<RevealDemoResult, String> {
    playback::reveal_demo_file(&app, demo_id).map_err(|e| e.into_string())
}

#[tauri::command]
pub fn list_demo_roots(app: AppHandle) -> Result<Vec<DemoRoot>, String> {
    demo::list_roots(&app).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn add_demo_root(app: AppHandle, path: String, scan_depth: i64) -> Result<DemoRoot, String> {
    demo::add_root(&app, &path, scan_depth).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn ensure_default_demo_root(app: AppHandle, root_path: String) -> Result<DemoRoot, String> {
    demo::ensure_default_root(&app, &root_path).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn update_demo_root(
    app: AppHandle,
    id: i64,
    enabled: bool,
    scan_depth: i64,
) -> Result<(), String> {
    demo::update_root(&app, id, enabled, scan_depth).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn remove_demo_root(app: AppHandle, id: i64) -> Result<(), String> {
    demo::remove_root(&app, id).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn delete_demo_file(app: AppHandle, demo_id: i64) -> Result<(), String> {
    demo::delete_file(&app, demo_id).map_err(|e| e.into_string())
}
#[tauri::command]
pub async fn scan_demo_roots(
    app: AppHandle,
    root_id: Option<i64>,
) -> Result<DemoScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || demo::scan(&app, root_id))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.into_string())
}
#[tauri::command]
pub async fn import_demo_file(app: AppHandle, path: String) -> Result<DemoImportResult, String> {
    tauri::async_runtime::spawn_blocking(move || demo::import_file(&app, &path, "manual", false))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.into_string())
}
#[tauri::command]
pub fn list_demos(
    app: AppHandle,
    query: String,
    status: String,
    page: i64,
    page_size: i64,
) -> Result<DemoListPage, String> {
    demo::list(&app, &query, &status, page, page_size).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn list_analysis_jobs(
    app: AppHandle,
    active_only: bool,
) -> Result<Vec<DemoAnalysisJob>, String> {
    demo::list_analysis_jobs(&app, active_only).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn cancel_analysis_job(app: AppHandle, job_id: i64) -> Result<(), String> {
    demo::cancel_analysis_job(&app, job_id).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn retry_analysis_job(app: AppHandle, job_id: i64) -> Result<(), String> {
    demo::retry_analysis_job(&app, job_id).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn delete_analysis_job(app: AppHandle, job_id: i64) -> Result<(), String> {
    demo::delete_analysis_job(&app, job_id).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_match_overview(app: AppHandle, demo_id: i64) -> Result<MatchOverview, String> {
    demo::match_overview(&app, demo_id).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_match_scoreboard(
    app: AppHandle,
    demo_id: i64,
) -> Result<Vec<MatchScoreboardPlayer>, String> {
    demo::match_scoreboard(&app, demo_id).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_match_performance_radar(
    app: AppHandle,
    demo_id: i64,
    player_keys: Option<Vec<String>>,
) -> Result<MatchPerformanceRadar, String> {
    demo::match_performance_radar(&app, demo_id, player_keys).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_match_rounds(app: AppHandle, demo_id: i64) -> Result<Vec<MatchRoundSummary>, String> {
    demo::match_rounds(&app, demo_id).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_match_economy(app: AppHandle, demo_id: i64) -> Result<Vec<MatchEconomyRow>, String> {
    demo::match_economy(&app, demo_id).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_match_duels(app: AppHandle, demo_id: i64) -> Result<MatchDuelMatrix, String> {
    demo::match_duels(&app, demo_id).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_match_utility(app: AppHandle, demo_id: i64) -> Result<Vec<MatchUtilityRow>, String> {
    demo::match_utility(&app, demo_id).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_match_events(
    app: AppHandle,
    demo_id: i64,
    filters: MatchEventFilters,
    page: i64,
    page_size: i64,
) -> Result<MatchEventPage, String> {
    demo::match_events(&app, demo_id, filters, page, page_size).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_player_match_detail(
    app: AppHandle,
    demo_id: i64,
    stable_key: String,
) -> Result<PlayerMatchDetail, String> {
    demo::player_match_detail(&app, demo_id, &stable_key).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn export_match(
    app: AppHandle,
    demo_id: i64,
    format: String,
    destination_path: String,
) -> Result<ExportMatchResult, String> {
    demo::export_match(&app, demo_id, &format, &destination_path).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn ensure_spatial_analysis(
    app: AppHandle,
    demo_id: i64,
    sampling_hz: i64,
) -> Result<i64, String> {
    demo::ensure_spatial_analysis(&app, demo_id, sampling_hz).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_round_positions(
    app: AppHandle,
    demo_id: i64,
    round_number: i64,
    sampling_hz: i64,
) -> Result<RoundPositions, String> {
    demo::round_positions(&app, demo_id, round_number, sampling_hz).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_heatmap_points(
    app: AppHandle,
    demo_id: i64,
    filters: HeatmapFilters,
) -> Result<Vec<HeatmapPoint>, String> {
    demo::heatmap_points(&app, demo_id, &filters).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn save_heatmap_png(
    app: AppHandle,
    demo_id: i64,
    filters: HeatmapFilters,
    destination_path: String,
) -> Result<HeatmapExportResult, String> {
    demo::save_heatmap_png(&app, demo_id, &filters, &destination_path).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn launch_demo_at_tick(
    app: AppHandle,
    demo_id: i64,
    tick: i64,
    player_key: Option<String>,
) -> Result<LaunchDemoResult, String> {
    demo::launch_demo_at_tick(&app, demo_id, tick, player_key.as_deref())
        .map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_demo_report(app: AppHandle, id: i64) -> Result<DemoReport, String> {
    demo::report(&app, id).map_err(|e| e.into_string())
}
#[tauri::command]
pub async fn retry_demo_parse(app: AppHandle, id: i64) -> Result<DemoImportResult, String> {
    let path = demo::path_for_id(&app, id).map_err(|e| e.into_string())?;
    tauri::async_runtime::spawn_blocking(move || demo::import_file(&app, &path, "retry", true))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.into_string())
}
#[tauri::command]
pub fn get_demo_settings(
    app: AppHandle,
    root_path: String,
) -> Result<DemoRecordingSettings, String> {
    let desired = demo::recording_desired(&app).map_err(|e| e.into_string())?;
    panel::demo_recording_state(&root_path, desired).map_err(|e| e.into_string())
}
#[tauri::command]
pub fn set_demo_recording_enabled(
    app: AppHandle,
    root_path: String,
    enabled: bool,
) -> Result<DemoRecordingSettings, String> {
    panel::apply_demo_recording(&root_path, enabled).map_err(|e| e.into_string())?;
    demo::set_recording_desired(&app, enabled).map_err(|e| e.into_string())?;
    panel::demo_recording_state(&root_path, enabled).map_err(|e| e.into_string())
}
