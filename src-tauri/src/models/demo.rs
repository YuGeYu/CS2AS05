use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoRoot {
    pub id: i64,
    pub path: String,
    pub enabled: bool,
    pub scan_depth: i64,
    pub last_scan_at: Option<i64>,
    pub last_error: Option<String>,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoListItem {
    pub id: i64,
    pub file_name: String,
    pub path: String,
    pub size_bytes: i64,
    pub mtime_ms: i64,
    pub status: String,
    pub error_code: Option<String>,
    pub map_name: Option<String>,
    pub total_rounds: Option<i64>,
    pub kills: Option<i64>,
    pub parsed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoListPage {
    pub items: Vec<DemoListItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoImportResult {
    pub demo_file_id: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoScanResult {
    pub discovered: u64,
    pub parsed: u64,
    pub failed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoReport {
    pub schema_version: u32,
    pub parser_name: String,
    pub parser_commit: String,
    pub parser_adapter_version: String,
    pub metrics_version: String,
    pub app_version: String,
    #[serde(default)]
    pub data_quality: DemoDataQuality,
    pub summary: DemoSummary,
    pub players: Vec<DemoPlayer>,
    pub rounds: Vec<DemoRound>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoSummary {
    pub demo_file_id: i64,
    pub file_name: String,
    pub path: String,
    pub map_name: Option<String>,
    pub server_name: Option<String>,
    pub file_time_ms: i64,
    pub size_bytes: i64,
    pub total_rounds: u32,
    pub total_kills: u32,
    pub team_a_score: Option<u32>,
    pub team_b_score: Option<u32>,
    pub parsed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoPlayer {
    #[serde(default)]
    pub key: String,
    pub steam_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<i32>,
    pub name: Option<String>,
    #[serde(default)]
    pub is_bot: bool,
    #[serde(default)]
    pub team_number: Option<i32>,
    pub team: Option<String>,
    pub kills: Option<u32>,
    pub deaths: Option<u32>,
    pub assists: Option<u32>,
    pub damage: Option<u32>,
    pub headshots: Option<u32>,
    #[serde(default)]
    pub identity_source: String,
    #[serde(default)]
    pub stats_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoDataQuality {
    pub scoreboard_status: String,
    pub warnings: Vec<String>,
    pub entity_parse_status: String,
}

impl Default for DemoDataQuality {
    fn default() -> Self {
        Self {
            scoreboard_status: "unavailable".into(),
            warnings: vec!["报告由旧解析器生成，需要重新解析。".into()],
            entity_parse_status: "failed".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoRound {
    pub number: u32,
    pub start_tick: Option<i32>,
    pub end_tick: Option<i32>,
    pub winner: Option<String>,
    pub reason: Option<String>,
    pub kills: Vec<DemoEvent>,
    pub bomb_events: Vec<DemoEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoEvent {
    pub tick: i32,
    pub kind: String,
    pub actor: Option<String>,
    pub target: Option<String>,
    pub weapon: Option<String>,
    pub headshot: Option<bool>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoRecordingSettings {
    pub desired_enabled: bool,
    pub normal_cfg_applied: bool,
    pub ffa_cfg_applied: bool,
    pub drifted: bool,
    pub writable: bool,
    pub scope: &'static str,
}
