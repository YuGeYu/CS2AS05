use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cs2RootCandidate {
    pub path: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cs2SuggestedRoot {
    pub path: String,
    pub source: String,
    pub confidence: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum Cs2RootScanEvent {
    Progress {
        elapsed_ms: u64,
        checked_locations: u32,
        current_location: Option<String>,
    },
    Candidate {
        elapsed_ms: u64,
        checked_locations: u32,
        candidate: Cs2SuggestedRoot,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cs2RootScanSummary {
    pub candidates: Vec<Cs2SuggestedRoot>,
    pub elapsed_ms: u64,
    pub checked_locations: u32,
    pub stop_reason: String,
    pub warnings: Vec<String>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantPreferences {
    pub autostart_enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FaultSubmissionResult {
    pub success: bool,
    pub ticket_id: String,
    pub message: String,
}
