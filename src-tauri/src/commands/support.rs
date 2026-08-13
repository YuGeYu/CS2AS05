use crate::errors::AppError;
use tauri::AppHandle;

use crate::models::cs2::{AssistantPreferences, FaultSubmissionResult, OperationResult};
use crate::services::support;

#[tauri::command]
pub fn open_official_site() -> Result<(), String> {
    support::open_official_site().map_err(AppError::into_string)
}

#[tauri::command]
pub fn open_idea_page() -> Result<(), String> {
    support::open_idea_page().map_err(AppError::into_string)
}

#[tauri::command]
pub fn open_release_page() -> Result<(), String> {
    support::open_release_page().map_err(AppError::into_string)
}

#[tauri::command]
pub fn open_upstream_project() -> Result<(), String> {
    support::open_upstream_project().map_err(AppError::into_string)
}

#[tauri::command]
pub fn open_reference_project(project: String) -> Result<(), String> {
    support::open_reference_project(&project).map_err(AppError::into_string)
}

#[tauri::command]
pub fn open_update_download(url: String) -> Result<(), String> {
    support::open_update_download(&url).map_err(AppError::into_string)
}

#[tauri::command]
pub fn get_assistant_preferences() -> Result<AssistantPreferences, String> {
    support::get_assistant_preferences().map_err(AppError::into_string)
}

#[tauri::command]
pub fn set_assistant_autostart(enabled: bool) -> Result<AssistantPreferences, String> {
    support::set_assistant_autostart(enabled).map_err(AppError::into_string)
}

#[tauri::command]
pub fn clear_assistant_data(app: AppHandle) -> Result<OperationResult, String> {
    support::clear_assistant_data(&app).map_err(AppError::into_string)
}

#[tauri::command]
pub async fn submit_fault_report(
    app: AppHandle,
    details: String,
    root_path: Option<String>,
) -> Result<FaultSubmissionResult, String> {
    support::submit_fault_report(&app, &details, root_path.as_deref())
        .await
        .map_err(AppError::into_string)
}
