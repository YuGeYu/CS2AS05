use tauri::AppHandle;

use crate::errors::AppError;
use crate::models::bot_difficulty::{
    BotProfileDocument, BotProfileList, BotProfileOperation, BotToolState, CreateBotProfileRequest,
    SaveBotProfileRequest, VpkEntry,
};
use crate::services::bot_difficulty;

#[tauri::command]
pub fn list_bot_profiles(app: AppHandle, root_path: String) -> Result<BotProfileList, String> {
    bot_difficulty::list(&app, &root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn open_bot_profile(
    app: AppHandle,
    root_path: String,
    profile_id: String,
) -> Result<BotProfileDocument, String> {
    bot_difficulty::open(&app, &root_path, &profile_id).map_err(AppError::into_string)
}

#[tauri::command]
pub fn get_bot_workshop_state(app: AppHandle) -> Result<BotToolState, String> {
    bot_difficulty::inspect_vpk_tool(&app).map_err(AppError::into_string)
}

#[tauri::command]
pub fn list_vpk_entries(app: AppHandle, input_path: String) -> Result<Vec<VpkEntry>, String> {
    bot_difficulty::list_vpk_entries(&app, &input_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn create_bot_profile(
    app: AppHandle,
    root_path: String,
    request: CreateBotProfileRequest,
) -> Result<BotProfileOperation, String> {
    bot_difficulty::create(&app, &root_path, request).map_err(AppError::into_string)
}

#[tauri::command]
pub fn save_bot_profile(
    app: AppHandle,
    root_path: String,
    request: SaveBotProfileRequest,
) -> Result<BotProfileOperation, String> {
    bot_difficulty::save(&app, &root_path, request).map_err(AppError::into_string)
}

#[tauri::command]
pub fn apply_bot_profile(
    app: AppHandle,
    root_path: String,
    profile_id: String,
) -> Result<BotProfileOperation, String> {
    bot_difficulty::apply(&app, &root_path, &profile_id).map_err(AppError::into_string)
}
