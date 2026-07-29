use tauri::AppHandle;

use crate::{
    models::demo::*,
    services::{demo, panel},
};

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
