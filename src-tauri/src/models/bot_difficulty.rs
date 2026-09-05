use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BotProfileSummary {
    pub id: String,
    pub name: String,
    pub source: String,
    pub base_difficulty: Option<String>,
    pub active: bool,
    pub read_only: bool,
    pub db_sha256: Option<String>,
    pub vpk_sha256: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BotProfileList {
    pub profiles: Vec<BotProfileSummary>,
    pub tool_version: Option<String>,
    pub tool_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BotProfileDocument {
    pub profile: BotProfileSummary,
    pub text: Option<String>,
    pub entry_path: String,
    pub validation: String,
    pub dirty: bool,
    pub tool_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBotProfileRequest {
    pub base_profile_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBotProfileRequest {
    pub profile_id: String,
    pub text: String,
    pub expected_db_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BotProfileOperation {
    pub profile: BotProfileSummary,
    pub backup_path: Option<String>,
    pub applied: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BotToolState {
    pub status: String,
    pub version: String,
    pub sha256: Option<String>,
    pub path_hint: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpkEntry {
    pub path: String,
    pub size: u64,
    pub sha256: Option<String>,
}
