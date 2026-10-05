use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoTracerStatus {
    pub resource_version: String,
    pub playback_version: String,
    pub gui_path: Option<String>,
    pub playback_package_path: Option<String>,
    pub gui_resource_ready: bool,
    pub playback_resource_ready: bool,
    pub playback_installed: bool,
    pub installed_file_count: usize,
    pub missing_files: Vec<String>,
    pub hash_mismatches: Vec<String>,
    pub counter_strike_sharp_ready: bool,
    pub metamod_ready: bool,
    pub cs2_running: bool,
    pub selected_root: Option<String>,
    pub ready: bool,
    pub blocked_code: Option<String>,
    pub blocked_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoTracerInstallResult {
    pub status: DemoTracerStatus,
    pub installed_files: Vec<String>,
    pub backup_path: Option<String>,
}
