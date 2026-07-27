use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelSnapshot {
    pub root_path: String,
    pub ready: bool,
    pub missing_files: Vec<String>,
    pub cs2_running: bool,
    pub mode: ModeState,
    pub difficulty: DifficultyState,
    pub presets: PresetsState,
    pub bot_items: BotItemsState,
    pub drop_knives: DropKnivesState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeState {
    pub current: Option<String>,
    pub insecure: bool,
    pub writable: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DifficultyState {
    pub current: Option<String>,
    pub available: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetsState {
    pub aim: Option<String>,
    pub nades: Option<String>,
    pub writable: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BotItemsState {
    #[serde(default)]
    pub profiles: bool,
    #[serde(default)]
    pub agents: bool,
    #[serde(default)]
    pub music: bool,
    #[serde(default)]
    pub weapons: bool,
    #[serde(default)]
    pub knives: bool,
    #[serde(default)]
    pub gloves: bool,
    #[serde(default)]
    pub stickers: bool,
    #[serde(default)]
    pub charms: bool,
    #[serde(skip_deserializing)]
    pub writable: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DropKnivesState {
    pub bind_key: String,
    pub selected: Vec<u16>,
    pub writable: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub options: String,
    pub insecure: bool,
    pub plugin_action: String,
    pub plugin_version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelInitializationResult {
    pub status: String,
    pub initialized_fields: Vec<String>,
}
