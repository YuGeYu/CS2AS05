use tauri::AppHandle;

use crate::errors::AppError;
use crate::models::cs2::OperationResult;
use crate::models::demotracer::{DemoTracerInstallResult, DemoTracerStatus};
use crate::services::demotracer;

#[tauri::command]
pub fn demotracer_get_status(
    app: AppHandle,
    root_path: Option<String>,
) -> Result<DemoTracerStatus, String> {
    demotracer::get_status(&app, root_path.as_deref()).map_err(AppError::into_string)
}

#[tauri::command]
pub fn demotracer_install_playback(
    app: AppHandle,
    root_path: String,
) -> Result<DemoTracerInstallResult, String> {
    demotracer::install_playback(&app, &root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn demotracer_uninstall_playback(root_path: String) -> Result<OperationResult, String> {
    demotracer::uninstall_playback(&root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn demotracer_open_gui(app: AppHandle) -> Result<OperationResult, String> {
    demotracer::open_gui(&app).map_err(AppError::into_string)
}
