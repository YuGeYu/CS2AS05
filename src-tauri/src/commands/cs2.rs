use tauri::ipc::Channel;
use tauri::{AppHandle, State};

use crate::errors::AppError;
use crate::models::cs2::{
    Cs2EnvironmentStatus, Cs2RootCandidate, DiagnosticsPayload, OperationResult,
};
use crate::services::cs2;
use crate::services::cs2_discovery::{self, ScanCoordinator};

#[tauri::command]
pub fn discover_cs2_roots() -> Result<Vec<Cs2RootCandidate>, String> {
    cs2::discover_cs2_roots().map_err(AppError::into_string)
}

#[tauri::command]
pub fn inspect_cs2_root(root_path: String) -> Result<Cs2EnvironmentStatus, String> {
    cs2::inspect_cs2_root(&root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn check_cs2_process() -> Result<bool, String> {
    cs2::check_cs2_process().map_err(AppError::into_string)
}

#[tauri::command]
pub fn install_bot_package(app: AppHandle, root_path: String) -> Result<OperationResult, String> {
    cs2::install_bot_package(&app, &root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn open_upstream_panel(app: AppHandle) -> Result<OperationResult, String> {
    cs2::open_upstream_panel(&app).map_err(AppError::into_string)
}

#[tauri::command]
pub fn uninstall_bot_package(root_path: String) -> Result<OperationResult, String> {
    cs2::uninstall_bot_package(&root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn get_diagnostics_payload(root_path: Option<String>) -> Result<DiagnosticsPayload, String> {
    cs2::get_diagnostics_payload(root_path.as_deref()).map_err(AppError::into_string)
}

#[tauri::command]
pub async fn guess_cs2_roots(
    state: State<'_, ScanCoordinator>,
    on_event: Channel<crate::models::cs2::Cs2RootScanEvent>,
) -> Result<crate::models::cs2::Cs2RootScanSummary, String> {
    let coordinator = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || cs2_discovery::scan(&coordinator, on_event))
        .await
        .map_err(|error| format!("[CS2_SCAN_TASK] 扫描任务失败：{error}"))?
        .map_err(AppError::into_string)
}

#[tauri::command]
pub fn stop_guess_cs2_roots(state: State<'_, ScanCoordinator>) -> bool {
    state.stop()
}
