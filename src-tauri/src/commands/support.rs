use crate::errors::AppError;
use tauri::{AppHandle, Manager};

use crate::models::cs2::{
    AssistantAccount, AssistantPreferences, FaultSubmissionResult, OperationResult,
};
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
pub fn open_api_purchase() -> Result<(), String> {
    support::open_api_purchase().map_err(AppError::into_string)
}

#[tauri::command]
pub fn get_ai_connection(app: AppHandle) -> Result<support::AiConnectionSummary, String> {
    support::get_ai_connection(&app).map_err(AppError::into_string)
}

#[tauri::command]
pub fn save_ai_connection(
    app: AppHandle,
    url: String,
    key: String,
    model: String,
) -> Result<support::AiConnectionSummary, String> {
    support::save_ai_connection(&app, &url, &key, &model).map_err(AppError::into_string)
}

#[tauri::command]
pub fn get_ai_chat_sessions(app: AppHandle) -> Result<Vec<support::AiChatSession>, String> {
    support::get_ai_chat_sessions(&app).map_err(AppError::into_string)
}

#[tauri::command]
pub fn save_ai_chat_sessions(
    app: AppHandle,
    sessions: Vec<support::AiChatSession>,
) -> Result<(), String> {
    support::save_ai_chat_sessions(&app, sessions).map_err(AppError::into_string)
}

#[tauri::command]
pub async fn get_ai_models(
    app: AppHandle,
    url: String,
    key: String,
) -> Result<Vec<String>, String> {
    support::get_ai_models(&app, &url, &key)
        .await
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn run_ai_powershell(app: AppHandle, command: String) -> Result<String, String> {
    let working_dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| format!("无法定位 PowerShell 工作目录：{error}"))?
        .join("quick-support");
    tauri::async_runtime::spawn_blocking(move || support::run_ai_powershell(&command, &working_dir))
        .await
        .map_err(|error| format!("PowerShell 操作中断：{error}"))?
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn chat_ai(
    app: AppHandle,
    messages: Vec<support::AiChatMessage>,
    context: Option<String>,
) -> Result<String, String> {
    support::chat_ai(&app, messages, context)
        .await
        .map_err(AppError::into_string)
}

#[tauri::command]
pub fn open_fault_idea_page(ticket_id: String) -> Result<(), String> {
    support::open_fault_idea_page(&ticket_id).map_err(AppError::into_string)
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
pub fn open_resource_link(url: String) -> Result<(), String> {
    support::open_resource_link(&url).map_err(AppError::into_string)
}

#[tauri::command]
pub fn open_community_download(url: String) -> Result<(), String> {
    support::open_community_download(&url).map_err(AppError::into_string)
}
#[tauri::command]
pub fn open_account_register() -> Result<(), String> {
    support::open_account_register().map_err(AppError::into_string)
}

#[tauri::command]
pub fn launch_community_connect(
    app: AppHandle,
    root_path: String,
    connection: String,
) -> Result<crate::models::panel::PanelSnapshot, String> {
    support::launch_community_connect(&app, &root_path, &connection).map_err(AppError::into_string)
}

#[tauri::command]
pub fn should_show_volume_smoke_guide(app: AppHandle) -> Result<bool, String> {
    support::should_show_volume_smoke_guide(&app).map_err(AppError::into_string)
}

#[tauri::command]
pub fn dismiss_volume_smoke_guide(app: AppHandle) -> Result<(), String> {
    support::dismiss_volume_smoke_guide(&app).map_err(AppError::into_string)
}

#[tauri::command]
pub fn is_bot_profile_guide_seen(app: AppHandle) -> Result<bool, String> {
    support::is_bot_profile_guide_seen(&app).map_err(AppError::into_string)
}

#[tauri::command]
pub fn dismiss_bot_profile_guide(app: AppHandle) -> Result<(), String> {
    support::dismiss_bot_profile_guide(&app).map_err(AppError::into_string)
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

#[tauri::command]
pub fn get_assistant_account(app: AppHandle) -> Result<AssistantAccount, String> {
    support::get_assistant_account(&app).map_err(AppError::into_string)
}

#[tauri::command]
pub async fn login_assistant(
    app: AppHandle,
    username: String,
    password: String,
) -> Result<AssistantAccount, String> {
    support::login_assistant(&app, &username, &password)
        .await
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn get_community_auth(app: AppHandle) -> Result<support::CommunityAuth, String> {
    support::get_community_auth(&app)
        .await
        .map_err(AppError::into_string)
}

#[tauri::command]
pub async fn download_community_file(
    app: AppHandle,
    url: String,
    filename: String,
    token: String,
) -> Result<String, String> {
    support::download_community_file(&app, &url, &filename, &token)
        .await
        .map_err(AppError::into_string)
}

#[tauri::command]
pub fn logout_assistant(app: AppHandle) -> Result<AssistantAccount, String> {
    support::logout_assistant(&app).map_err(AppError::into_string)
}
