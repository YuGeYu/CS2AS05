use crate::errors::AppError;
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
pub fn open_update_download(url: String) -> Result<(), String> {
    support::open_update_download(&url).map_err(AppError::into_string)
}
