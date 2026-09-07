use tauri::async_runtime::spawn_blocking;
use tauri::AppHandle;

use crate::errors::AppError;
use crate::models::bot_difficulty::{
    BotProfileDocument, BotProfileList, BotProfileOperation, BotToolState, CreateBotProfileRequest,
    RenameBotProfileRequest, SaveBotProfileRequest, VpkEntry,
};
use crate::services::bot_difficulty;

#[tauri::command]
pub async fn list_bot_profiles(
    app: AppHandle,
    root_path: String,
) -> Result<BotProfileList, String> {
    spawn_blocking(move || bot_difficulty::list(&app, &root_path))
        .await
        .map_err(|error| error.to_string())?
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn open_bot_profile(
    app: AppHandle,
    root_path: String,
    profile_id: String,
) -> Result<BotProfileDocument, String> {
    spawn_blocking(move || bot_difficulty::open(&app, &root_path, &profile_id))
        .await
        .map_err(|error| error.to_string())?
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn get_bot_workshop_state(app: AppHandle) -> Result<BotToolState, String> {
    spawn_blocking(move || bot_difficulty::inspect_vpk_tool(&app))
        .await
        .map_err(|error| error.to_string())?
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn list_vpk_entries(app: AppHandle, input_path: String) -> Result<Vec<VpkEntry>, String> {
    spawn_blocking(move || bot_difficulty::list_vpk_entries(&app, &input_path))
        .await
        .map_err(|error| error.to_string())?
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn create_bot_profile(
    app: AppHandle,
    root_path: String,
    request: CreateBotProfileRequest,
) -> Result<BotProfileOperation, String> {
    spawn_blocking(move || bot_difficulty::create(&app, &root_path, request))
        .await
        .map_err(|error| error.to_string())?
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn save_bot_profile(
    app: AppHandle,
    root_path: String,
    request: SaveBotProfileRequest,
) -> Result<BotProfileOperation, String> {
    spawn_blocking(move || bot_difficulty::save(&app, &root_path, request))
        .await
        .map_err(|error| error.to_string())?
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn rename_bot_profile(
    app: AppHandle,
    request: RenameBotProfileRequest,
) -> Result<BotProfileOperation, String> {
    spawn_blocking(move || bot_difficulty::rename(&app, request))
        .await
        .map_err(|error| error.to_string())?
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn delete_bot_profile(
    app: AppHandle,
    root_path: String,
    profile_id: String,
) -> Result<BotProfileOperation, String> {
    spawn_blocking(move || bot_difficulty::delete(&app, &root_path, &profile_id))
        .await
        .map_err(|error| error.to_string())?
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn apply_bot_profile(
    app: AppHandle,
    root_path: String,
    profile_id: String,
) -> Result<BotProfileOperation, String> {
    spawn_blocking(move || bot_difficulty::apply(&app, &root_path, &profile_id))
        .await
        .map_err(|error| error.to_string())?
        .map_err(AppError::into_string)
}
