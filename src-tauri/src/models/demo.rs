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
    pub cache_hit: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoPlaybackResult {
    pub demo_file_id: i64,
    pub session_id: String,
    pub source_path: String,
    pub prepared_path: String,
    pub started: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevealDemoResult {
    pub demo_file_id: i64,
    pub path: String,
    pub revealed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMatchReportReady {
    pub session_id: String,
    pub report_id: i64,
    pub file_name: String,
    pub completed_at: i64,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMatchReportFailed {
    pub session_id: String,
    pub demo_id: Option<i64>,
    pub error_code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoScanResult {
    pub root_paths: Vec<String>,
    pub scanned_directories: u64,
    pub discovered_dem_files: u64,
    pub imported: u64,
    pub cache_hits: u64,
    pub parsed: u64,
    pub failed: u64,
    pub permission_errors: u64,
    pub last_scan_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoAnalysisJob {
    pub id: i64,
    pub demo_file_id: i64,
    pub file_name: String,
    pub kind: String,
    pub stage: String,
    pub progress: i64,
    pub attempts: i64,
    pub error_code: Option<String>,
    pub error_detail: Option<String>,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchOverview {
    pub demo_file_id: i64,
    pub file_name: String,
    pub map_name: Option<String>,
    pub server_name: Option<String>,
    pub team_a_score: Option<i64>,
    pub team_b_score: Option<i64>,
    pub total_rounds: i64,
    pub total_kills: i64,
    pub analyzed_at: Option<i64>,
    pub quality_json: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchScoreboardPlayer {
    pub stable_key: String,
    pub name: Option<String>,
    pub steam_id: Option<String>,
    pub is_bot: bool,
    pub team_name: Option<String>,
    pub team_number: Option<i64>,
    pub kills: Option<i64>,
    pub deaths: Option<i64>,
    pub assists: Option<i64>,
    pub damage_health: Option<i64>,
    pub headshots: Option<i64>,
    pub adr: Option<f64>,
    pub kast_percent: Option<f64>,
    pub rating: Option<f64>,
    pub rating_model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchRoundSummary {
    pub round_number: i64,
    pub start_tick: Option<i64>,
    pub freeze_end_tick: Option<i64>,
    pub end_tick: Option<i64>,
    pub official_end_tick: Option<i64>,
    pub winner_side: Option<String>,
    pub reason: Option<String>,
    pub event_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchEconomyRow {
    pub round_number: i64,
    pub team_number: i64,
    pub team: Option<String>,
    pub equipment_value: Option<i64>,
    pub money_start: Option<i64>,
    pub money_spent: Option<i64>,
    pub economy_type: Option<String>,
    pub quality: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchDuelPlayer {
    pub key: String,
    pub name: Option<String>,
    pub is_bot: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchDuelTeam {
    pub number: Option<i64>,
    pub label: String,
    pub players: Vec<MatchDuelPlayer>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchDuelCell {
    pub x_player_key: String,
    pub y_player_key: String,
    pub x_kills_y: i64,
    pub y_kills_x: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchDuelMatrix {
    pub team_x: MatchDuelTeam,
    pub team_y: MatchDuelTeam,
    pub cells: Vec<MatchDuelCell>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchUtilityRow {
    pub stable_key: String,
    pub player_name: Option<String>,
    pub he_damage: Option<i64>,
    pub flash_victims: i64,
    pub throws: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MatchEventFilters {
    pub kind: Option<String>,
    pub round_number: Option<i64>,
    pub player_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchEventRow {
    pub id: i64,
    pub round_number: Option<i64>,
    pub tick: i64,
    pub kind: String,
    pub actor_key: Option<String>,
    pub target_key: Option<String>,
    pub payload_json: String,
    pub quality: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchEventPage {
    pub items: Vec<MatchEventRow>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRoundDetail {
    pub round_number: i64,
    pub side: Option<String>,
    pub kills: Option<i64>,
    pub deaths: Option<i64>,
    pub assists: Option<i64>,
    pub damage_health: Option<i64>,
    pub survived: Option<bool>,
    pub traded: Option<bool>,
    pub kast: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMatchDetail {
    pub player: MatchScoreboardPlayer,
    pub rounds: Vec<PlayerRoundDetail>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMatchResult {
    pub path: String,
    pub bytes_written: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapFilters {
    pub kind: String,
    #[serde(default)]
    pub round_numbers: Vec<i64>,
    #[serde(default)]
    pub player_keys: Vec<String>,
    #[serde(default)]
    pub team_numbers: Vec<i64>,
    #[serde(default = "default_heatmap_layer")]
    pub layer: String,
    #[serde(default = "default_heatmap_radius")]
    pub radius: u32,
    #[serde(default = "default_heatmap_opacity")]
    pub opacity: f64,
}

fn default_heatmap_layer() -> String {
    "all".into()
}
fn default_heatmap_radius() -> u32 {
    18
}
fn default_heatmap_opacity() -> f64 {
    0.72
}

impl Default for HeatmapFilters {
    fn default() -> Self {
        Self {
            kind: "player_death".into(),
            round_numbers: Vec::new(),
            player_keys: Vec::new(),
            team_numbers: Vec::new(),
            layer: default_heatmap_layer(),
            radius: default_heatmap_radius(),
            opacity: default_heatmap_opacity(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapExportResult {
    pub path: String,
    pub bytes_written: u64,
    pub width: u32,
    pub height: u32,
    pub map_name: String,
    pub layer: String,
    pub input_points: usize,
    pub rendered_points: usize,
    pub discarded_points: usize,
    pub asset_sha256: String,
    pub output_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchDemoResult {
    pub demo_file_id: i64,
    pub tick: i64,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionPoint {
    pub tick: i32,
    pub stable_key: String,
    pub name: Option<String>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: Option<f32>,
    pub health: Option<i32>,
    pub armor: Option<i32>,
    pub team_number: Option<i32>,
    #[serde(default)]
    pub participant_role: String,
    pub alive: Option<bool>,
    pub weapon: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundPositions {
    pub demo_file_id: i64,
    pub round_number: i64,
    pub sampling_hz: i64,
    pub points: Vec<PositionPoint>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapPoint {
    pub tick: i64,
    pub x: f64,
    pub y: f64,
    pub weight: f64,
    pub kind: String,
    pub round_number: Option<i64>,
    pub player_key: Option<String>,
    pub z: Option<f64>,
    pub team_number: Option<i64>,
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
    pub participant_role: String,
    #[serde(default)]
    pub identity_source: String,
    #[serde(default)]
    pub team_number: Option<i32>,
    pub team: Option<String>,
    pub kills: Option<u32>,
    pub deaths: Option<u32>,
    pub assists: Option<u32>,
    pub damage: Option<u32>,
    pub headshots: Option<u32>,
    #[serde(default)]
    pub stats_source: String,
    #[serde(default)]
    pub rounds_played: Option<u32>,
    #[serde(default)]
    pub rounds_survived: Option<u32>,
    #[serde(default)]
    pub kast_rounds: Option<u32>,
    #[serde(default)]
    pub multi_kills: Option<u32>,
    #[serde(default)]
    pub first_kills: Option<u32>,
    #[serde(default)]
    pub first_deaths: Option<u32>,
    #[serde(default)]
    pub trade_kills: Option<u32>,
    #[serde(default)]
    pub trade_denials: Option<u32>,
    #[serde(default)]
    pub adr: Option<f64>,
    #[serde(default)]
    pub kast_percent: Option<f64>,
    #[serde(default)]
    pub headshot_percent: Option<f64>,
    #[serde(default)]
    pub round_swing: Option<f64>,
    #[serde(default)]
    pub economy_adjustment: Option<f64>,
    #[serde(default)]
    pub rating_status: String,
    #[serde(default)]
    pub rating: Option<DemoRating>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoRating {
    pub model_version: String,
    pub kill_component: f64,
    pub damage_component: f64,
    pub survival_component: f64,
    pub assist_component: f64,
    pub rating: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoDataQuality {
    pub scoreboard_status: String,
    pub warnings: Vec<String>,
    pub entity_parse_status: String,
    #[serde(default)]
    pub rating_status: String,
    #[serde(default)]
    pub rating_warnings: Vec<String>,
    #[serde(default)]
    pub score_source: String,
    #[serde(default)]
    pub score_quality: String,
    #[serde(default)]
    pub score_warnings: Vec<String>,
    #[serde(default)]
    pub canonical_ct_score: Option<u32>,
    #[serde(default)]
    pub canonical_t_score: Option<u32>,
    #[serde(default)]
    pub props_ct_score: Option<u32>,
    #[serde(default)]
    pub props_t_score: Option<u32>,
    #[serde(default)]
    pub logical_rounds: u32,
    #[serde(default)]
    pub completed_rounds: u32,
    #[serde(default)]
    pub unfinished_rounds: u32,
}

impl Default for DemoDataQuality {
    fn default() -> Self {
        Self {
            scoreboard_status: "unavailable".into(),
            warnings: vec!["报告由旧解析器生成，需要重新解析。".into()],
            entity_parse_status: "failed".into(),
            rating_status: "unavailable".into(),
            rating_warnings: vec![
                "当前 Demo 缺少 K/D/A、伤害或已完成回合，无法计算简易 Rating。".into(),
            ],
            score_source: "unavailable".into(),
            score_quality: "unavailable".into(),
            score_warnings: vec![],
            canonical_ct_score: None,
            canonical_t_score: None,
            props_ct_score: None,
            props_t_score: None,
            logical_rounds: 0,
            completed_rounds: 0,
            unfinished_rounds: 0,
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
    #[serde(default)]
    pub events: Vec<DemoEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoEvent {
    pub tick: i32,
    pub kind: String,
    pub actor: Option<String>,
    pub target: Option<String>,
    #[serde(default)]
    pub actor_key: Option<String>,
    #[serde(default)]
    pub target_key: Option<String>,
    pub weapon: Option<String>,
    pub headshot: Option<bool>,
    pub detail: Option<String>,
    #[serde(default)]
    pub payload: serde_json::Value,
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
