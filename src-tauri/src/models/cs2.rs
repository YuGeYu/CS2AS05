use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cs2RootCandidate {
    pub path: String,
    pub source: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cs2EnvironmentStatus {
    pub root_path: String,
    pub game_dir_exists: bool,
    pub csgo_dir_exists: bool,
    pub metamod_exists: bool,
    pub counterstrike_sharp_exists: bool,
    pub gameinfo_exists: bool,
    pub backup_online_gameinfo_exists: bool,
    pub backup_withbots_gameinfo_exists: bool,
    pub base_environment_ready: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsPayload {
    pub summary: String,
    pub full_log: String,
    pub log_path: String,
}
