use crate::errors::AppError;
use crate::models::panel::{LaunchResult, PanelInitializationResult, PanelSnapshot};
use crate::services::panel;
use tauri::AppHandle;

#[tauri::command]
pub fn get_panel_snapshot(root_path: String) -> Result<PanelSnapshot, String> {
    panel::snapshot(&root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn initialize_panel_defaults(root_path: String) -> Result<PanelInitializationResult, String> {
    panel::initialize_panel_defaults(&root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn set_panel_mode(
    app: AppHandle,
    root_path: String,
    mode: String,
) -> Result<PanelSnapshot, String> {
    panel::set_mode_with_app(&app, &root_path, &mode).map_err(AppError::into_string)
}

#[tauri::command]
pub fn set_panel_difficulty(root_path: String, level: String) -> Result<PanelSnapshot, String> {
    panel::set_difficulty(&root_path, &level).map_err(AppError::into_string)
}

#[tauri::command]
pub fn set_panel_aim(root_path: String, value: String) -> Result<PanelSnapshot, String> {
    panel::set_preset(&root_path, "bot_aim", &value).map_err(AppError::into_string)
}

#[tauri::command]
pub fn set_panel_nades(root_path: String, value: String) -> Result<PanelSnapshot, String> {
    panel::set_preset(&root_path, "bot_nades", &value).map_err(AppError::into_string)
}

#[tauri::command]
pub fn set_panel_bot_item(
    root_path: String,
    item: String,
    enabled: bool,
) -> Result<PanelSnapshot, String> {
    panel::set_bot_item(&root_path, &item, enabled).map_err(AppError::into_string)
}

#[tauri::command]
pub fn set_panel_drop_knives(
    root_path: String,
    bind_key: String,
    selected: Vec<u16>,
) -> Result<PanelSnapshot, String> {
    panel::set_drop_knives(&root_path, &bind_key, &selected).map_err(AppError::into_string)
}

#[tauri::command]
pub fn launch_panel_cs2(
    app: AppHandle,
    root_path: String,
    mode: String,
) -> Result<LaunchResult, String> {
    panel::launch_cs2(&app, &root_path, &mode).map_err(AppError::into_string)
}
