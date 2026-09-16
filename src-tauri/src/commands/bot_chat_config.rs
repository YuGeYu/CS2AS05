use crate::errors::AppError;
use crate::services::bot_chat_config;

#[tauri::command]
pub fn get_bot_chat_config(root_path: String) -> Result<String, String> {
    bot_chat_config::read(&root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn set_bot_chat_config(root_path: String, content: String) -> Result<String, String> {
    bot_chat_config::write(&root_path, &content).map_err(AppError::into_string)
}
