use crate::errors::AppError;
use crate::services::map_rotation;

#[tauri::command]
pub fn get_map_rotation_default(
    root_path: String,
) -> Result<map_rotation::MapRotationDefault, String> {
    map_rotation::get(&root_path).map_err(AppError::into_string)
}
#[tauri::command]
pub fn set_map_rotation_default(
    root_path: String,
    enabled: bool,
) -> Result<map_rotation::MapRotationDefault, String> {
    map_rotation::set(&root_path, enabled).map_err(AppError::into_string)
}
#[tauri::command]
pub fn reset_map_rotation_default(
    root_path: String,
) -> Result<map_rotation::MapRotationDefault, String> {
    map_rotation::reset(&root_path).map_err(AppError::into_string)
}
