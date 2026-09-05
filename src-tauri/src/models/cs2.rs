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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cs2ProcessInfo {
    pub pid: u32,
    pub exe_name: String,
    pub exe_path: Option<String>,
    pub parent_pid: Option<u32>,
    pub start_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cs2ProcessSnapshot {
    pub observed_at: i64,
    pub processes: Vec<Cs2ProcessInfo>,
    pub confidence: String,
    pub sample_count: u8,
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
    pub idea_section_id: String,
    pub account_username: String,
    pub auto_registered: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantAccount {
    pub username: Option<String>,
    pub logged_in: bool,
}
