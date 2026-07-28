// Compatibility behavior follows the released ed0ard/CS2-Bot-Improver v1.4.3 Panel.
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use chrono::Local;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tauri::AppHandle;

use crate::errors::AppError;
use crate::models::demo::DemoRecordingSettings;
use crate::models::panel::{
    BotItemsState, DifficultyState, DropKnivesState, LaunchResult, ModeState,
    PanelInitializationResult, PanelSnapshot, PresetsState,
};
use crate::services::cs2;
use crate::services::cs2_discovery;
use crate::services::demo;

const KNIVES: [u16; 20] = [
    500, 503, 505, 506, 507, 508, 509, 512, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 525,
    526,
];
const DEFAULT_KNIVES: [u16; 5] = [507, 508, 515, 519, 525];
const CFG_FILES: [&str; 2] = ["cfg/my_bot_normal_config.cfg", "cfg/my_bot_ffa_config.cfg"];
const PANEL_STATE_FILE: &str = "cfg/cs2as05-panel-state.json";
const CORE_CONFIG_FILE: &str = "addons/counterstrikesharp/configs/core.json";
const BOT_ITEM_KEYS: [(&str, &str); 8] = [
    ("profiles", "bot_hider github.com/XBribo all"),
    ("agents", "bot_randomizer github.com/ed0ard agents"),
    ("music", "bot_randomizer github.com/ed0ard music"),
    ("weapons", "bot_randomizer github.com/ed0ard weapons"),
    ("knives", "bot_randomizer github.com/ed0ard knives"),
    ("gloves", "bot_randomizer github.com/ed0ard gloves"),
    ("stickers", "bot_randomizer github.com/ed0ard stickers"),
    ("charms", "bot_randomizer github.com/ed0ard charms"),
];

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PanelPreferences {
    #[serde(default = "state_schema")]
    schema: u32,
    #[serde(default = "initialized_version")]
    initialized_by: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mode: Option<PreferenceValue<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    difficulty: Option<PreferenceValue<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    aim: Option<PreferenceValue<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nades: Option<PreferenceValue<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bot_items: Option<BotItemsPreference>,
    #[serde(skip)]
    bot_items_unknown: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    drop_knives: Option<DropKnivesPreference>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PreferenceValue<T> {
    initialized: bool,
    value: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BotItemsPreference {
    initialized: bool,
    #[serde(default)]
    profiles: bool,
    #[serde(default)]
    agents: bool,
    #[serde(default)]
    music: bool,
    #[serde(default)]
    weapons: bool,
    #[serde(default)]
    knives: bool,
    #[serde(default)]
    gloves: bool,
    #[serde(default)]
    stickers: bool,
    #[serde(default)]
    charms: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DropKnivesPreference {
    initialized: bool,
    bind_key: String,
    selected: Vec<u16>,
}

fn state_schema() -> u32 {
    1
}

fn initialized_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

impl PanelPreferences {
    fn empty() -> Self {
        Self {
            schema: state_schema(),
            initialized_by: initialized_version(),
            ..Self::default()
        }
    }

    fn normalize(mut self) -> Self {
        if self.schema != state_schema() {
            return Self::empty();
        }
        self.mode = self.mode.filter(|field| {
            field.initialized && ["online", "bots"].contains(&field.value.as_str())
        });
        self.difficulty = self.difficulty.filter(|field| {
            field.initialized && ["Low", "Medium", "High"].contains(&field.value.as_str())
        });
        self.aim = self.aim.filter(|field| {
            field.initialized && ["head", "mixed", "body"].contains(&field.value.as_str())
        });
        self.nades = self.nades.filter(|field| {
            field.initialized
                && ["max", "more", "normal", "less", "off"].contains(&field.value.as_str())
        });
        self.bot_items = self.bot_items.filter(|field| field.initialized);
        self.drop_knives = self.drop_knives.filter(|field| {
            field.initialized
                && validate_bind_key(&field.bind_key).is_ok()
                && field.selected.iter().all(|id| KNIVES.contains(id))
        });
        self
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.mode.is_none()
            && self.difficulty.is_none()
            && self.aim.is_none()
            && self.nades.is_none()
            && self.bot_items.is_none()
            && self.drop_knives.is_none()
    }
}

pub fn initialize_panel_defaults(root_path: &str) -> Result<PanelInitializationResult, AppError> {
    let root = cs2::normalize_root(root_path)?;
    initialize_panel_defaults_at(&root, false)
}

fn initialize_panel_defaults_at(
    root: &Path,
    new_install: bool,
) -> Result<PanelInitializationResult, AppError> {
    initialize_panel_defaults_at_with_running(root, new_install, cs2::check_cs2_process()?)
}

fn initialize_panel_defaults_at_with_running(
    root: &Path,
    new_install: bool,
    cs2_running: bool,
) -> Result<PanelInitializationResult, AppError> {
    if cs2_running {
        return Ok(PanelInitializationResult {
            status: "deferred".into(),
            initialized_fields: Vec::new(),
        });
    }
    let csgo = root.join("game/csgo");
    let ready = panel_files_ready(&csgo);
    if !ready {
        return Ok(PanelInitializationResult {
            status: "unchanged".into(),
            initialized_fields: Vec::new(),
        });
    }
    let mut preferences = load_preferences(&csgo)?;
    let mut initialized_fields = Vec::new();
    panel_transaction(&csgo, || {
        if preferences.mode.is_none() {
            let value = disk_mode(&csgo).unwrap_or_else(|| "bots".into());
            if disk_mode(&csgo).is_none() {
                write_mode_at(&csgo, &value)?;
            }
            preferences.mode = Some(preference(value));
            initialized_fields.push("mode".into());
        }
        if preferences.difficulty.is_none() {
            let value = disk_difficulty(&csgo).unwrap_or_else(|| "Low".into());
            if disk_difficulty(&csgo).is_none() {
                write_difficulty_at(&csgo, &value)?;
            }
            preferences.difficulty = Some(preference(value));
            initialized_fields.push("difficulty".into());
        }
        let normal_cfg = read_text(&csgo.join(CFG_FILES[0]))
            .map_err(|error| io_context("读取 cfg", &csgo.join(CFG_FILES[0]), error))?;
        if preferences.aim.is_none() {
            let value = managed_value(&normal_cfg, "bot_aim", &["head", "mixed", "body"])
                .unwrap_or_else(|| "mixed".into());
            if managed_value(&normal_cfg, "bot_aim", &["head", "mixed", "body"]).is_none() {
                write_preset_at(&csgo, "bot_aim", &value)?;
            }
            preferences.aim = Some(preference(value));
            initialized_fields.push("aim".into());
        }
        if preferences.nades.is_none() {
            let value = managed_value(
                &normal_cfg,
                "bot_nades",
                &["max", "more", "normal", "less", "off"],
            )
            .unwrap_or_else(|| "less".into());
            if managed_value(
                &normal_cfg,
                "bot_nades",
                &["max", "more", "normal", "less", "off"],
            )
            .is_none()
            {
                write_preset_at(&csgo, "bot_nades", &value)?;
            }
            preferences.nades = Some(preference(value));
            initialized_fields.push("nades".into());
        }
        if preferences.bot_items.is_none() {
            let path = csgo.join(CORE_CONFIG_FILE);
            let items = read_bot_items_strict(&path)?;
            preferences.bot_items = Some(bot_items_preference_at(&path, &items));
            initialized_fields.push("botItems".into());
        }
        if preferences.drop_knives.is_none() {
            let parsed = parse_drop_knives_optional(&normal_cfg);
            let had_managed_bind = parsed.is_some();
            let (bind_key, selected) = parsed.unwrap_or_else(|| {
                if new_install {
                    ("\\".into(), DEFAULT_KNIVES.to_vec())
                } else {
                    ("\\".into(), Vec::new())
                }
            });
            if new_install && !had_managed_bind {
                write_drop_knives_at(&csgo, &bind_key, &selected)?;
            }
            preferences.drop_knives = Some(DropKnivesPreference {
                initialized: true,
                bind_key,
                selected,
            });
            initialized_fields.push("dropKnives".into());
        }
        if initialized_fields.is_empty() {
            Ok(())
        } else {
            save_preferences(&csgo, &preferences)
        }
    })?;
    Ok(PanelInitializationResult {
        status: if initialized_fields.is_empty() {
            "unchanged".into()
        } else {
            "initialized".into()
        },
        initialized_fields,
    })
}

pub(crate) fn capture_panel_preferences(csgo: &Path) -> Result<PanelPreferences, AppError> {
    let state = load_preferences(csgo)?;
    let normal_cfg = read_text(&csgo.join(CFG_FILES[0])).ok();
    let mut captured = state;
    captured.mode = captured.mode.or_else(|| disk_mode(csgo).map(preference));
    captured.difficulty = captured
        .difficulty
        .or_else(|| disk_difficulty(csgo).map(preference));
    captured.aim = captured.aim.or_else(|| {
        normal_cfg
            .as_deref()
            .and_then(|cfg| managed_value(cfg, "bot_aim", &["head", "mixed", "body"]))
            .map(preference)
    });
    captured.nades = captured.nades.or_else(|| {
        normal_cfg
            .as_deref()
            .and_then(|cfg| {
                managed_value(cfg, "bot_nades", &["max", "more", "normal", "less", "off"])
            })
            .map(preference)
    });
    if captured.bot_items.is_none() {
        let path = csgo.join(CORE_CONFIG_FILE);
        if path.is_file() {
            let items = read_bot_items_strict(&path)?;
            captured.bot_items = Some(bot_items_preference_at(&path, &items));
        }
    }
    let core_path = csgo.join(CORE_CONFIG_FILE);
    if core_path.is_file() {
        captured.bot_items_unknown = read_bot_item_unknown_fields(&core_path)?;
    }
    if captured.drop_knives.is_none() {
        captured.drop_knives = normal_cfg.as_deref().map(|cfg| {
            let (bind_key, selected) =
                parse_drop_knives_optional(cfg).unwrap_or_else(|| ("\\".into(), Vec::new()));
            DropKnivesPreference {
                initialized: true,
                bind_key,
                selected,
            }
        });
    }
    Ok(captured)
}

pub(crate) fn restore_panel_preferences(
    csgo: &Path,
    preferences: &PanelPreferences,
    new_install: bool,
) -> Result<(), AppError> {
    if let Some(field) = &preferences.mode {
        write_mode_at(csgo, &field.value)?;
    }
    if let Some(field) = &preferences.difficulty {
        write_difficulty_at(csgo, &field.value)?;
    }
    if let Some(field) = &preferences.aim {
        write_preset_at(csgo, "bot_aim", &field.value)?;
    }
    if let Some(field) = &preferences.nades {
        write_preset_at(csgo, "bot_nades", &field.value)?;
    }
    if let Some(field) = &preferences.bot_items {
        write_bot_items_preference_at(
            &csgo.join(CORE_CONFIG_FILE),
            &preferences.bot_items_unknown,
            &BotItemsState {
                profiles: field.profiles,
                agents: field.agents,
                music: field.music,
                weapons: field.weapons,
                knives: field.knives,
                gloves: field.gloves,
                stickers: field.stickers,
                charms: field.charms,
                writable: false,
            },
        )?;
    }
    if let Some(field) = &preferences.drop_knives {
        write_drop_knives_at(csgo, &field.bind_key, &field.selected)?;
    }
    if !preferences.is_empty() {
        save_preferences(csgo, preferences)?;
    }
    let root = csgo
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| invalid("[PANEL_PATH_INVALID] CS2 目录层级无效。"))?;
    initialize_panel_defaults_at(root, new_install)?;
    Ok(())
}

fn preference(value: String) -> PreferenceValue<String> {
    PreferenceValue {
        initialized: true,
        value,
    }
}

fn panel_files_ready(csgo: &Path) -> bool {
    [
        "gameinfo.gi",
        "backup/Online/gameinfo.gi",
        "backup/WithBots/gameinfo.gi",
        "overrides/Low/botprofile.vpk",
        "overrides/Medium/botprofile.vpk",
        "overrides/High/botprofile.vpk",
        CFG_FILES[0],
        CFG_FILES[1],
    ]
    .into_iter()
    .all(|relative| csgo.join(relative).is_file())
}

fn disk_mode(csgo: &Path) -> Option<String> {
    let active = fs::read(csgo.join("gameinfo.gi")).ok()?;
    let online = fs::read(csgo.join("backup/Online/gameinfo.gi")).ok();
    let bots = fs::read(csgo.join("backup/WithBots/gameinfo.gi")).ok();
    if online.as_deref() == Some(&active) {
        Some("online".into())
    } else if bots.as_deref() == Some(&active)
        || String::from_utf8_lossy(&active).contains("csgo/addons/metamod")
    {
        Some("bots".into())
    } else {
        Some("online".into())
    }
}

fn disk_difficulty(csgo: &Path) -> Option<String> {
    let active = fs::read(csgo.join("overrides/botprofile.vpk")).ok()?;
    ["Low", "Medium", "High"]
        .into_iter()
        .find(|level| {
            fs::read(csgo.join(format!("overrides/{level}/botprofile.vpk")))
                .ok()
                .as_deref()
                == Some(active.as_slice())
        })
        .map(str::to_string)
}

fn load_preferences(csgo: &Path) -> Result<PanelPreferences, AppError> {
    let path = csgo.join(PANEL_STATE_FILE);
    if !path.is_file() {
        return Ok(PanelPreferences::empty());
    }
    let bytes = fs::read(&path).map_err(|error| io_context("读取 Panel 状态", &path, error))?;
    match serde_json::from_slice::<PanelPreferences>(&bytes) {
        Ok(state) if state.schema == state_schema() => Ok(state.normalize()),
        Ok(_) | Err(_) => {
            archive_corrupt(&path)?;
            Ok(PanelPreferences::empty())
        }
    }
}

fn save_preferences(csgo: &Path, preferences: &PanelPreferences) -> Result<(), AppError> {
    let mut state = preferences.clone().normalize();
    state.schema = state_schema();
    state.initialized_by = initialized_version();
    let bytes = serde_json::to_vec_pretty(&state).map_err(|error| {
        invalid(format!(
            "[PANEL_STATE_WRITE] 无法序列化 Panel 状态：{error}"
        ))
    })?;
    atomic_write(&csgo.join(PANEL_STATE_FILE), &bytes)
}

fn archive_corrupt(path: &Path) -> Result<PathBuf, AppError> {
    let archived = path.with_extension(format!(
        "{}.corrupt-{}",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("file"),
        Local::now().format("%Y%m%d-%H%M%S%3f")
    ));
    fs::rename(path, &archived).map_err(|error| io_context("归档损坏配置", path, error))?;
    Ok(archived)
}

fn bot_items_preference_at(path: &Path, items: &BotItemsState) -> BotItemsPreference {
    let _ = path;
    BotItemsPreference {
        initialized: true,
        profiles: items.profiles,
        agents: items.agents,
        music: items.music,
        weapons: items.weapons,
        knives: items.knives,
        gloves: items.gloves,
        stickers: items.stickers,
        charms: items.charms,
    }
}

fn read_bot_items_strict(path: &Path) -> Result<BotItemsState, AppError> {
    let bytes = fs::read(path)
        .map_err(|error| io_context("读取 CounterStrikeSharp core.json", path, error))?;
    let value = serde_json::from_slice::<Value>(&bytes).map_err(|error| {
        invalid(format!(
            "[PANEL_BOT_ITEM_CORRUPT] CounterStrikeSharp core.json 损坏：{}\n{error}",
            path.display()
        ))
    })?;
    let object = value.as_object().ok_or_else(|| {
        invalid("[PANEL_BOT_ITEM_SCHEMA] CounterStrikeSharp core.json 不是 JSON 对象。")
    })?;
    let read = |item: &str| -> Result<bool, AppError> {
        let key = bot_item_core_key(item).expect("managed Bot Item key");
        match object.get(key) {
            None => Ok(true),
            Some(value) => value.as_bool().ok_or_else(|| {
                invalid(format!(
                    "[PANEL_BOT_ITEM_SCHEMA] core.json 字段 {key} 必须为布尔值。"
                ))
            }),
        }
    };
    Ok(BotItemsState {
        profiles: read("profiles")?,
        agents: read("agents")?,
        music: read("music")?,
        weapons: read("weapons")?,
        knives: read("knives")?,
        gloves: read("gloves")?,
        stickers: read("stickers")?,
        charms: read("charms")?,
        writable: false,
    })
}

fn write_bot_items_at(path: &Path, items: &BotItemsState) -> Result<(), AppError> {
    let mut object = fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    set_bot_item_values(&mut object, items);
    let bytes = serde_json::to_vec_pretty(&object)
        .map_err(|error| invalid(format!("[PANEL_JSON_WRITE] 无法序列化 Bot 配置：{error}")))?;
    atomic_write(path, &bytes)
}

fn write_bot_items_preference_at(
    path: &Path,
    unknown: &Map<String, Value>,
    items: &BotItemsState,
) -> Result<(), AppError> {
    let mut object = fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    object.extend(unknown.clone());
    set_bot_item_values(&mut object, items);
    let bytes = serde_json::to_vec_pretty(&object)
        .map_err(|error| invalid(format!("[PANEL_JSON_WRITE] 无法序列化 Bot 配置：{error}")))?;
    atomic_write(path, &bytes)
}

fn read_bot_item_unknown_fields(path: &Path) -> Result<Map<String, Value>, AppError> {
    let bytes = fs::read(path)
        .map_err(|error| io_context("读取 CounterStrikeSharp core.json", path, error))?;
    let value = serde_json::from_slice::<Value>(&bytes).map_err(|error| {
        invalid(format!(
            "[PANEL_BOT_ITEM_CORRUPT] CounterStrikeSharp core.json 损坏：{}\n{error}",
            path.display()
        ))
    })?;
    let mut object = value.as_object().cloned().ok_or_else(|| {
        invalid("[PANEL_BOT_ITEM_SCHEMA] CounterStrikeSharp core.json 不是 JSON 对象。")
    })?;
    for (_, key) in BOT_ITEM_KEYS {
        object.remove(key);
    }
    Ok(object)
}

fn set_bot_item_values(object: &mut Map<String, Value>, items: &BotItemsState) {
    for (item, key) in BOT_ITEM_KEYS {
        let enabled = match item {
            "profiles" => items.profiles,
            "agents" => items.agents,
            "music" => items.music,
            "weapons" => items.weapons,
            "knives" => items.knives,
            "gloves" => items.gloves,
            "stickers" => items.stickers,
            "charms" => items.charms,
            _ => unreachable!(),
        };
        object.insert(key.into(), Value::Bool(enabled));
    }
}

fn bot_item_core_key(item: &str) -> Option<&'static str> {
    BOT_ITEM_KEYS
        .iter()
        .find_map(|(managed, key)| (*managed == item).then_some(*key))
}

fn write_mode_at(csgo: &Path, mode: &str) -> Result<(), AppError> {
    let source = csgo.join(if mode == "online" {
        "backup/Online/gameinfo.gi"
    } else {
        "backup/WithBots/gameinfo.gi"
    });
    let bytes = fs::read(&source).map_err(|error| io_context("读取模式源文件", &source, error))?;
    atomic_write(&csgo.join("gameinfo.gi"), &bytes)
}

fn write_difficulty_at(csgo: &Path, level: &str) -> Result<(), AppError> {
    let source = csgo.join(format!("overrides/{level}/botprofile.vpk"));
    let bytes = fs::read(&source).map_err(|error| io_context("读取难度文件", &source, error))?;
    atomic_write(&csgo.join("overrides/botprofile.vpk"), &bytes)
}

fn write_preset_at(csgo: &Path, command: &str, value: &str) -> Result<(), AppError> {
    for relative in CFG_FILES {
        replace_cfg_line(&csgo.join(relative), command, &format!("{command} {value}"))?;
    }
    Ok(())
}

fn write_drop_knives_at(csgo: &Path, bind_key: &str, selected: &[u16]) -> Result<(), AppError> {
    let commands = selected
        .iter()
        .map(|id| format!("subclass_create {id}"))
        .collect::<Vec<_>>()
        .join(";");
    let replacement = (!selected.is_empty()).then(|| format!("bind {bind_key} \"{commands}\""));
    for relative in CFG_FILES {
        replace_drop_bind(&csgo.join(relative), replacement.as_deref())?;
    }
    Ok(())
}

const DEMO_RECORDING_BEGIN: &str = "// CS2AS05 DEMO RECORDING BEGIN";
const DEMO_RECORDING_END: &str = "// CS2AS05 DEMO RECORDING END";

fn replace_demo_recording_block(path: &Path, enabled: bool) -> Result<(), AppError> {
    let text = read_text(path).map_err(|e| io_context("读取 cfg", path, e))?;
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let has_bom = text.starts_with('\u{feff}');
    let mut lines = Vec::new();
    let mut in_block = false;
    for original in text.trim_start_matches('\u{feff}').lines() {
        let line = original.trim();
        if line == DEMO_RECORDING_BEGIN {
            in_block = true;
            continue;
        }
        if line == DEMO_RECORDING_END {
            in_block = false;
            continue;
        }
        if !in_block {
            lines.push(original.to_string());
        }
    }
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    let value = if enabled { 1 } else { 0 };
    lines.extend([
        DEMO_RECORDING_BEGIN.to_string(),
        format!("tv_enable {value}"),
        format!("tv_autorecord {value}"),
        DEMO_RECORDING_END.to_string(),
    ]);
    let mut output = format!("{}{newline}", lines.join(newline));
    if has_bom {
        output.insert(0, '\u{feff}');
    }
    atomic_write(path, output.as_bytes())
}

fn cfg_demo_recording_applied(path: &Path, enabled: bool) -> bool {
    let Ok(text) = read_text(path) else {
        return false;
    };
    let expected = if enabled { "1" } else { "0" };
    let mut in_block = false;
    let mut tv_enable = false;
    let mut tv_autorecord = false;
    for line in text.lines().map(str::trim) {
        if line == DEMO_RECORDING_BEGIN {
            in_block = true;
            continue;
        }
        if line == DEMO_RECORDING_END {
            in_block = false;
            continue;
        }
        if in_block {
            tv_enable |= line == format!("tv_enable {expected}");
            tv_autorecord |= line == format!("tv_autorecord {expected}");
        }
    }
    tv_enable && tv_autorecord
}

pub fn apply_demo_recording(root_path: &str, enabled: bool) -> Result<(), AppError> {
    if cs2::check_cs2_process()? {
        return Err(invalid(
            "[DEMO_RECORDING_CS2_RUNNING] CS2 运行中不能修改自动录制设置。",
        ));
    }
    let root = cs2::normalize_root(root_path)?;
    let csgo = root.join("game/csgo");
    panel_transaction(&csgo, || {
        for relative in CFG_FILES {
            replace_demo_recording_block(&csgo.join(relative), enabled)?;
        }
        Ok(())
    })
}

pub fn demo_recording_state(
    root_path: &str,
    desired: bool,
) -> Result<DemoRecordingSettings, AppError> {
    let root = cs2::normalize_root(root_path)?;
    let csgo = root.join("game/csgo");
    let normal = cfg_demo_recording_applied(&csgo.join(CFG_FILES[0]), desired);
    let ffa = cfg_demo_recording_applied(&csgo.join(CFG_FILES[1]), desired);
    Ok(DemoRecordingSettings {
        desired_enabled: desired,
        normal_cfg_applied: normal,
        ffa_cfg_applied: ffa,
        drifted: !normal || !ffa,
        writable: !cs2::check_cs2_process()?,
        scope: "bots-only",
    })
}

fn panel_transaction<T>(
    csgo: &Path,
    operation: impl FnOnce() -> Result<T, AppError>,
) -> Result<T, AppError> {
    let paths = [
        "gameinfo.gi",
        "overrides/botprofile.vpk",
        CFG_FILES[0],
        CFG_FILES[1],
        CORE_CONFIG_FILE,
        PANEL_STATE_FILE,
    ];
    let original = paths
        .iter()
        .map(|relative| {
            let path = csgo.join(relative);
            (path.clone(), fs::read(&path).ok())
        })
        .collect::<Vec<_>>();
    match operation() {
        Ok(value) => Ok(value),
        Err(error) => {
            let mut rollback_errors = Vec::new();
            for (path, bytes) in original {
                let result = match bytes {
                    Some(bytes) => {
                        if let Some(parent) = path.parent() {
                            fs::create_dir_all(parent).and_then(|_| fs::write(&path, bytes))
                        } else {
                            fs::write(&path, bytes)
                        }
                    }
                    None if path.exists() => fs::remove_file(&path),
                    None => Ok(()),
                };
                if let Err(rollback_error) = result {
                    rollback_errors.push(format!("{}: {rollback_error}", path.display()));
                }
            }
            if rollback_errors.is_empty() {
                Err(error)
            } else {
                Err(invalid(format!(
                    "{}\n[PANEL_ROLLBACK_FAILED] {}",
                    error.into_string(),
                    rollback_errors.join("; ")
                )))
            }
        }
    }
}

pub fn snapshot(root_path: &str) -> Result<PanelSnapshot, AppError> {
    let root = cs2::normalize_root(root_path)?;
    snapshot_at(&root)
}

fn snapshot_at(root: &Path) -> Result<PanelSnapshot, AppError> {
    let csgo = root.join("game/csgo");
    let required = [
        "gameinfo.gi",
        "backup/Online/gameinfo.gi",
        "backup/WithBots/gameinfo.gi",
        "addons/metamod",
        "addons/counterstrikesharp",
        "cfg/my_bot_normal_config.cfg",
        "cfg/my_bot_ffa_config.cfg",
        "overrides/Low/botprofile.vpk",
        "overrides/Medium/botprofile.vpk",
        "overrides/High/botprofile.vpk",
    ];
    let missing_files = required
        .iter()
        .filter(|relative| {
            let target = csgo.join(relative);
            if relative.ends_with("/metamod") || relative.ends_with("/counterstrikesharp") {
                !target.is_dir()
            } else {
                !target.is_file()
            }
        })
        .map(|relative| relative.to_string())
        .collect::<Vec<_>>();
    let ready = missing_files.is_empty();
    let cs2_running = cs2::check_cs2_process()?;

    let active_gameinfo = fs::read(csgo.join("gameinfo.gi")).ok();
    let online = fs::read(csgo.join("backup/Online/gameinfo.gi")).ok();
    let bots = fs::read(csgo.join("backup/WithBots/gameinfo.gi")).ok();
    let mode = if active_gameinfo == online && online.is_some() {
        Some("online".to_string())
    } else if active_gameinfo == bots && bots.is_some() {
        Some("bots".to_string())
    } else {
        active_gameinfo.as_deref().map(|bytes| {
            if String::from_utf8_lossy(bytes).contains("csgo/addons/metamod") {
                "bots".to_string()
            } else {
                "online".to_string()
            }
        })
    };

    let active_profile = fs::read(csgo.join("overrides/botprofile.vpk")).ok();
    let difficulty = ["Low", "Medium", "High"]
        .into_iter()
        .find(|level| {
            active_profile == fs::read(csgo.join(format!("overrides/{level}/botprofile.vpk"))).ok()
        })
        .map(str::to_string);

    let normal_cfg = read_text(&csgo.join(CFG_FILES[0])).unwrap_or_default();
    let aim = managed_value(&normal_cfg, "bot_aim", &["head", "mixed", "body"]);
    let nades = managed_value(
        &normal_cfg,
        "bot_nades",
        &["max", "more", "normal", "less", "off"],
    );
    let (bind_key, selected) = parse_drop_knives_optional(&normal_cfg)
        .or_else(|| {
            load_preferences(&csgo)
                .ok()
                .and_then(|preferences| preferences.drop_knives)
                .filter(|field| field.selected.is_empty())
                .map(|field| (field.bind_key, field.selected))
        })
        .unwrap_or_else(|| ("\\".into(), Vec::new()));
    let bot_items_path = csgo.join(CORE_CONFIG_FILE);
    let bot_items = read_bot_items(&bot_items_path);

    Ok(PanelSnapshot {
        root_path: root.display().to_string(),
        ready,
        missing_files,
        cs2_running,
        mode: ModeState {
            insecure: mode.as_deref() == Some("bots"),
            current: mode,
            writable: ready && !cs2_running,
        },
        difficulty: DifficultyState {
            current: difficulty,
            available: vec!["Low".into(), "Medium".into(), "High".into()],
        },
        presets: PresetsState {
            aim,
            nades,
            writable: ready,
        },
        bot_items: BotItemsState {
            writable: ready,
            ..bot_items
        },
        drop_knives: DropKnivesState {
            bind_key,
            selected,
            writable: ready,
        },
    })
}

pub fn set_mode(root_path: &str, mode: &str) -> Result<PanelSnapshot, AppError> {
    if !["online", "bots"].contains(&mode) {
        return Err(invalid("[PANEL_MODE_INVALID] 未知运行模式。"));
    }
    if cs2::check_cs2_process()? {
        return Err(invalid("[CS2_RUNNING] 请先退出 CS2，再切换模式。"));
    }
    let root = cs2::normalize_root(root_path)?;
    let csgo = root.join("game/csgo");
    let mut preferences = load_preferences(&csgo)?;
    panel_transaction(&csgo, || {
        write_mode_at(&csgo, mode)?;
        preferences.mode = Some(preference(mode.to_string()));
        save_preferences(&csgo, &preferences)
    })?;
    snapshot_at(&root)
}

pub fn set_difficulty(root_path: &str, level: &str) -> Result<PanelSnapshot, AppError> {
    if !["Low", "Medium", "High"].contains(&level) {
        return Err(invalid("[PANEL_DIFFICULTY_INVALID] 未知难度。"));
    }
    let root = cs2::normalize_root(root_path)?;
    let csgo = root.join("game/csgo");
    let mut preferences = load_preferences(&csgo)?;
    panel_transaction(&csgo, || {
        write_difficulty_at(&csgo, level)?;
        preferences.difficulty = Some(preference(level.to_string()));
        save_preferences(&csgo, &preferences)
    })?;
    snapshot_at(&root)
}

pub fn set_preset(root_path: &str, command: &str, value: &str) -> Result<PanelSnapshot, AppError> {
    let allowed = match command {
        "bot_aim" => &["head", "mixed", "body"][..],
        "bot_nades" => &["max", "more", "normal", "less", "off"][..],
        _ => return Err(invalid("[PANEL_PRESET_INVALID] 未知预设类型。")),
    };
    if !allowed.contains(&value) {
        return Err(invalid("[PANEL_PRESET_INVALID] 预设值不在允许范围。"));
    }
    let root = cs2::normalize_root(root_path)?;
    let csgo = root.join("game/csgo");
    let mut preferences = load_preferences(&csgo)?;
    panel_transaction(&csgo, || {
        write_preset_at(&csgo, command, value)?;
        match command {
            "bot_aim" => preferences.aim = Some(preference(value.to_string())),
            "bot_nades" => preferences.nades = Some(preference(value.to_string())),
            _ => unreachable!(),
        }
        save_preferences(&csgo, &preferences)
    })?;
    snapshot_at(&root)
}

pub fn set_bot_item(root_path: &str, item: &str, enabled: bool) -> Result<PanelSnapshot, AppError> {
    if bot_item_core_key(item).is_none() {
        return Err(invalid("[PANEL_BOT_ITEM_INVALID] 未知 Bot 物品开关。"));
    }
    let root = cs2::normalize_root(root_path)?;
    let csgo = root.join("game/csgo");
    let path = csgo.join(CORE_CONFIG_FILE);
    let mut preferences = load_preferences(&csgo)?;
    panel_transaction(&csgo, || {
        let mut items = if path.is_file() {
            read_bot_items_strict(&path)?
        } else {
            BotItemsState::default()
        };
        match item {
            "profiles" => items.profiles = enabled,
            "agents" => items.agents = enabled,
            "music" => items.music = enabled,
            "weapons" => items.weapons = enabled,
            "knives" => items.knives = enabled,
            "gloves" => items.gloves = enabled,
            "stickers" => items.stickers = enabled,
            "charms" => items.charms = enabled,
            _ => unreachable!(),
        }
        write_bot_items_at(&path, &items)?;
        preferences.bot_items = Some(bot_items_preference_at(&path, &items));
        save_preferences(&csgo, &preferences)
    })?;
    snapshot_at(&root)
}

pub fn set_drop_knives(
    root_path: &str,
    bind_key: &str,
    selected: &[u16],
) -> Result<PanelSnapshot, AppError> {
    validate_bind_key(bind_key)?;
    let selected_set = selected.iter().copied().collect::<BTreeSet<_>>();
    if selected_set.iter().any(|id| !KNIVES.contains(id)) {
        return Err(invalid(
            "[PANEL_KNIFE_INVALID] 刀具 subclass 不在允许范围。",
        ));
    }
    let ordered = KNIVES
        .into_iter()
        .filter(|id| selected_set.contains(id))
        .collect::<Vec<_>>();
    let root = cs2::normalize_root(root_path)?;
    let csgo = root.join("game/csgo");
    let mut preferences = load_preferences(&csgo)?;
    panel_transaction(&csgo, || {
        write_drop_knives_at(&csgo, bind_key, &ordered)?;
        preferences.drop_knives = Some(DropKnivesPreference {
            initialized: true,
            bind_key: bind_key.to_string(),
            selected: ordered.clone(),
        });
        save_preferences(&csgo, &preferences)
    })?;
    snapshot_at(&root)
}

pub fn launch_cs2(app: &AppHandle, root_path: &str, mode: &str) -> Result<LaunchResult, AppError> {
    let result = launch_cs2_inner(app, root_path, mode);
    if let Err(AppError::Runtime(message)) = &result {
        cs2::write_runtime_log("ERROR", message);
    }
    result
}

fn launch_cs2_inner(
    app: &AppHandle,
    root_path: &str,
    mode: &str,
) -> Result<LaunchResult, AppError> {
    if !["online", "bots"].contains(&mode) {
        return Err(invalid("[PANEL_MODE_INVALID] 未知运行模式。"));
    }
    let _guard = if mode == "bots" {
        Some(
            launch_coordinator()
                .try_lock()
                .map_err(|_| invalid("[BOT_PLUGIN_LAUNCH_BUSY] 已有 BOT 启动请求正在处理。"))?,
        )
    } else {
        None
    };
    let (root, steam) = prepare_launch_paths(root_path, cs2::check_cs2_process, |root| {
        cs2_discovery::find_steam_executable(Some(root))
    })?;
    if mode == "bots" && demo::recording_desired(app)? {
        apply_demo_recording(root_path, true)?;
    }
    if mode == "bots" {
        initialize_panel_defaults_at(&root, false)?;
    }
    let (plugin_action, plugin_version) = if mode == "bots" {
        ensure_current_bot_plugin(app, root_path)?
    } else {
        ("unchanged".into(), String::new())
    };
    if cs2::check_cs2_process()? {
        return Err(invalid("[CS2_RUNNING] CS2 已在运行。"));
    }
    set_mode(root_path, mode)?;
    let insecure = mode == "bots";
    let mut options = vec!["-applaunch", "730"];
    if insecure {
        options.extend(["-insecure", "-console", "-condebug"]);
    }
    cs2::write_runtime_log("INFO", &format!("使用 Steam 客户端：{}", steam.display()));
    Command::new(&steam)
        .args(&options)
        .spawn()
        .map_err(|e| io_context("启动 Steam", &steam, e))?;
    demo::observe_assistant_launch(app.clone(), chrono::Utc::now().timestamp_millis());
    Ok(LaunchResult {
        options: options[2..].join(" "),
        insecure,
        plugin_action,
        plugin_version,
    })
}

fn prepare_launch_paths(
    root_path: &str,
    check_cs2_running: impl FnOnce() -> Result<bool, AppError>,
    resolve_steam: impl FnOnce(&Path) -> Option<PathBuf>,
) -> Result<(PathBuf, PathBuf), AppError> {
    if check_cs2_running()? {
        return Err(invalid("[CS2_RUNNING] CS2 已在运行。"));
    }
    let root = cs2::normalize_root(root_path)?;
    let steam = resolve_steam(&root).ok_or_else(|| {
        invalid(
            "[STEAM_NOT_FOUND] 未找到 Steam 客户端。请确认 Steam 已安装，或先启动一次 Steam 后重试。",
        )
    })?;
    Ok((root, steam))
}

fn launch_coordinator() -> &'static Mutex<()> {
    static COORDINATOR: OnceLock<Mutex<()>> = OnceLock::new();
    COORDINATOR.get_or_init(|| Mutex::new(()))
}

fn ensure_current_bot_plugin(
    app: &AppHandle,
    root_path: &str,
) -> Result<(String, String), AppError> {
    let current = Version::parse(env!("CARGO_PKG_VERSION")).map_err(|error| {
        invalid(format!(
            "[BOT_PLUGIN_VERSION_INVALID] 当前程序版本无效：{error}"
        ))
    })?;
    match plugin_gate_decision(cs2::inspect_bot_plugin_version(root_path)?, &current)? {
        PluginGateDecision::Unchanged(version) => Ok(("unchanged".into(), version)),
        PluginGateDecision::Install => {
            let version = cs2::ensure_bot_plugin_current(app, root_path).map_err(|error| {
                invalid(format!(
                    "[BOT_PLUGIN_AUTO_INSTALL_FAILED] {}",
                    error.into_string()
                ))
            })?;
            Ok(("installed".into(), version))
        }
    }
}

#[derive(Debug, PartialEq)]
enum PluginGateDecision {
    Unchanged(String),
    Install,
}

fn plugin_gate_decision(
    status: cs2::PluginVersionStatus,
    current: &Version,
) -> Result<PluginGateDecision, AppError> {
    match status {
        cs2::PluginVersionStatus::Valid { version }
            if compare_version_core(&version, current) != Ordering::Less =>
        {
            Ok(PluginGateDecision::Unchanged(version.to_string()))
        }
        cs2::PluginVersionStatus::Invalid { version: Some(version), .. }
            if compare_version_core(&version, current) == Ordering::Greater => Err(invalid(
            "[BOT_PLUGIN_HIGHER_VERSION_UNTRUSTED] 检测到更高版本插件但无法验证完整性，请在安装与诊断页明确覆盖安装。",
        )),
        _ => Ok(PluginGateDecision::Install),
    }
}

fn compare_version_core(left: &Version, right: &Version) -> Ordering {
    (left.major, left.minor, left.patch).cmp(&(right.major, right.minor, right.patch))
}

fn replace_cfg_line(path: &Path, command: &str, replacement: &str) -> Result<(), AppError> {
    let text = read_text(path).map_err(|e| io_context("读取 cfg", path, e))?;
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut found = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with(&format!("{command} ")) {
            if !found {
                lines.push(replacement.to_string());
                found = true;
            }
        } else {
            lines.push(line.to_string());
        }
    }
    if !found {
        lines.push(replacement.to_string());
    }
    atomic_write(path, format!("{}{newline}", lines.join(newline)).as_bytes())
}

fn replace_drop_bind(path: &Path, replacement: Option<&str>) -> Result<(), AppError> {
    let text = read_text(path).map_err(|e| io_context("读取 cfg", path, e))?;
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines = text
        .lines()
        .filter(|line| !is_managed_drop_bind(line))
        .map(str::to_string)
        .collect::<Vec<_>>();
    if let Some(line) = replacement {
        lines.push(line.to_string());
    }
    atomic_write(path, format!("{}{newline}", lines.join(newline)).as_bytes())
}

fn is_managed_drop_bind(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with("bind ") || !trimmed.contains("subclass_create ") {
        return false;
    }
    let Some((_, quoted)) = trimmed.split_once('"') else {
        return false;
    };
    let commands = quoted.trim_end_matches('"').split(';');
    commands.into_iter().all(|command| {
        command
            .trim()
            .strip_prefix("subclass_create ")
            .and_then(|value| value.parse::<u16>().ok())
            .is_some_and(|id| KNIVES.contains(&id))
    })
}

fn parse_drop_knives_optional(text: &str) -> Option<(String, Vec<u16>)> {
    if let Some(line) = text.lines().find(|line| is_managed_drop_bind(line)) {
        let before_quote = line.split('"').next().unwrap_or_default().trim();
        let bind_key = before_quote
            .strip_prefix("bind ")
            .unwrap_or_default()
            .trim()
            .to_string();
        let selected = KNIVES
            .into_iter()
            .filter(|id| line.contains(&format!("subclass_create {id}")))
            .collect();
        return Some((bind_key, selected));
    }
    None
}

fn managed_value(text: &str, command: &str, allowed: &[&str]) -> Option<String> {
    text.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        (parts.next() == Some(command))
            .then(|| parts.next())
            .flatten()
            .filter(|value| allowed.contains(value))
            .map(str::to_string)
    })
}

fn read_bot_items(path: &Path) -> BotItemsState {
    if path.is_file() {
        read_bot_items_strict(path).unwrap_or_default()
    } else {
        BotItemsState::default()
    }
}

fn validate_bind_key(key: &str) -> Result<(), AppError> {
    const NAMED_KEYS: [&str; 39] = [
        "\\",
        "space",
        "enter",
        "tab",
        "escape",
        "backspace",
        "semicolon",
        "'",
        ",",
        ".",
        "/",
        "-",
        "=",
        "[",
        "]",
        "`",
        "capslock",
        "uparrow",
        "downarrow",
        "leftarrow",
        "rightarrow",
        "ins",
        "del",
        "home",
        "end",
        "pgup",
        "pgdn",
        "shift",
        "rshift",
        "ctrl",
        "rctrl",
        "alt",
        "ralt",
        "kp_0",
        "kp_1",
        "kp_2",
        "kp_3",
        "kp_4",
        "kp_5",
    ];
    let valid = matches!(key.as_bytes(), [b'a'..=b'z'] | [b'0'..=b'9'])
        || (key.len() >= 2
            && key.len() <= 3
            && key.starts_with('f')
            && key[1..]
                .parse::<u8>()
                .is_ok_and(|number| (1..=12).contains(&number)))
        || NAMED_KEYS.contains(&key)
        || ["kp_6", "kp_7", "kp_8", "kp_9"].contains(&key);
    valid
        .then_some(())
        .ok_or_else(|| invalid("[PANEL_BIND_INVALID] 按键不在允许范围。"))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid("[PANEL_PATH_INVALID] 目标文件没有父目录。"))?;
    fs::create_dir_all(parent).map_err(|e| io_context("创建目标目录", parent, e))?;
    let backup = if path.is_file() {
        let backup = path.with_extension(format!(
            "{}.backup-{}",
            path.extension().and_then(|v| v.to_str()).unwrap_or("file"),
            Local::now().format("%Y%m%d-%H%M%S%3f")
        ));
        fs::copy(path, &backup).map_err(|e| io_context("创建写前备份", path, e))?;
        Some(backup)
    } else {
        None
    };
    let temporary = path.with_extension(format!(
        "{}.tmp-{}",
        path.extension().and_then(|v| v.to_str()).unwrap_or("file"),
        std::process::id()
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)
            .map_err(|e| io_context("创建临时文件", &temporary, e))?;
        file.write_all(bytes)
            .map_err(|e| io_context("写入临时文件", &temporary, e))?;
        file.sync_all()
            .map_err(|e| io_context("同步临时文件", &temporary, e))?;
        drop(file);
        #[cfg(windows)]
        if path.exists() {
            fs::remove_file(path).map_err(|e| io_context("替换目标文件", path, e))?;
        }
        fs::rename(&temporary, path).map_err(|e| io_context("提交目标文件", path, e))?;
        let actual = fs::read(path).map_err(|e| io_context("回读目标文件", path, e))?;
        if actual != bytes {
            return Err(invalid("[PANEL_WRITE_VERIFY] 写后校验失败。"));
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
        if !path.exists() {
            if let Some(backup) = backup {
                let _ = fs::copy(backup, path);
            }
        }
    }
    result
}

fn read_text(path: &Path) -> std::io::Result<String> {
    fs::read_to_string(path)
}

fn invalid(message: impl Into<String>) -> AppError {
    AppError::runtime(message)
}

fn io_context(action: &str, path: &Path, error: std::io::Error) -> AppError {
    AppError::runtime(format!(
        "[PANEL_IO] {action}失败：{}\n{error}",
        path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_root(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("panel-{name}-{nonce}"))
    }

    fn create_panel_environment(
        root: &Path,
        mode: &str,
        difficulty: &str,
        cfg: &str,
        bot_items: Option<&str>,
    ) {
        let csgo = root.join("game/csgo");
        for relative in [
            "backup/Online",
            "backup/WithBots",
            "cfg",
            "overrides/Low",
            "overrides/Medium",
            "overrides/High",
            "addons/metamod",
            "addons/counterstrikesharp",
        ] {
            fs::create_dir_all(csgo.join(relative)).unwrap();
        }
        fs::write(csgo.join("backup/Online/gameinfo.gi"), b"online").unwrap();
        fs::write(
            csgo.join("backup/WithBots/gameinfo.gi"),
            b"Game csgo/addons/metamod",
        )
        .unwrap();
        fs::copy(
            csgo.join(format!(
                "backup/{}/gameinfo.gi",
                if mode == "online" {
                    "Online"
                } else {
                    "WithBots"
                }
            )),
            csgo.join("gameinfo.gi"),
        )
        .unwrap();
        for level in ["Low", "Medium", "High"] {
            fs::write(
                csgo.join(format!("overrides/{level}/botprofile.vpk")),
                level.as_bytes(),
            )
            .unwrap();
        }
        fs::copy(
            csgo.join(format!("overrides/{difficulty}/botprofile.vpk")),
            csgo.join("overrides/botprofile.vpk"),
        )
        .unwrap();
        for relative in CFG_FILES {
            fs::write(csgo.join(relative), cfg).unwrap();
        }
        let path = csgo.join(CORE_CONFIG_FILE);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bot_items.unwrap_or("{}")).unwrap();
    }

    #[test]
    fn detects_only_allowlisted_drop_bind() {
        assert!(is_managed_drop_bind(
            r#"bind \\ "subclass_create 500;subclass_create 526""#
        ));
        assert!(!is_managed_drop_bind(r#"bind f1 "say hello""#));
        assert!(!is_managed_drop_bind(r#"bind f1 "subclass_create 999""#));
    }

    #[test]
    fn bind_validation_rejects_cfg_injection() {
        assert!(validate_bind_key("f8").is_ok());
        assert!(validate_bind_key("kp_7").is_ok());
        assert!(validate_bind_key("\\").is_ok());
        assert!(validate_bind_key("x;quit").is_err());
        assert!(validate_bind_key("x\nquit").is_err());
        assert!(validate_bind_key("f8 quit").is_err());
        assert!(validate_bind_key("MediaPlayPause").is_err());
    }

    #[test]
    fn bot_plugin_gate_installs_old_keeps_valid_new_and_blocks_untrusted_new() {
        let current = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        assert_eq!(
            plugin_gate_decision(
                cs2::PluginVersionStatus::Valid {
                    version: Version::parse("0.5.5-test.1").unwrap()
                },
                &current
            )
            .unwrap(),
            PluginGateDecision::Unchanged("0.5.5-test.1".into())
        );
        assert_eq!(
            plugin_gate_decision(cs2::PluginVersionStatus::Missing, &current).unwrap(),
            PluginGateDecision::Install
        );
        assert_eq!(
            plugin_gate_decision(
                cs2::PluginVersionStatus::Valid {
                    version: Version::parse("0.5.4").unwrap()
                },
                &current
            )
            .unwrap(),
            PluginGateDecision::Install
        );
        assert_eq!(
            plugin_gate_decision(
                cs2::PluginVersionStatus::Valid {
                    version: Version::parse("0.6.0-test").unwrap()
                },
                &current
            )
            .unwrap(),
            PluginGateDecision::Unchanged("0.6.0-test".into())
        );
        let error = plugin_gate_decision(
            cs2::PluginVersionStatus::Invalid {
                reason: "tampered".into(),
                version: Some(Version::parse("0.6.0-test").unwrap()),
            },
            &current,
        )
        .unwrap_err()
        .into_string();
        assert!(error.contains("BOT_PLUGIN_HIGHER_VERSION_UNTRUSTED"));
    }

    #[test]
    fn steam_resolution_precedes_launch_mutations() {
        let source = include_str!("panel.rs");
        let launch = &source[source.find("fn launch_cs2_inner").unwrap()
            ..source.find("fn launch_coordinator").unwrap()];
        let steam = launch.find("find_steam_executable").unwrap();
        for mutation in [
            "initialize_panel_defaults_at",
            "ensure_current_bot_plugin",
            "set_mode",
        ] {
            assert!(
                steam < launch.find(mutation).unwrap(),
                "Steam must be resolved before {mutation}"
            );
        }
    }

    #[test]
    fn missing_steam_does_not_change_gameinfo_panel_state_or_plugin_marker() {
        let root = test_root("missing-steam-no-writes");
        let csgo = root.join("game/csgo");
        let protected = [
            (csgo.join("gameinfo.gi"), b"gameinfo-before".as_slice()),
            (
                csgo.join(PANEL_STATE_FILE),
                b"panel-state-before".as_slice(),
            ),
            (
                csgo.join("addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json"),
                b"plugin-marker-before".as_slice(),
            ),
        ];
        for (path, bytes) in &protected {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
        let before: Vec<Vec<u8>> = protected
            .iter()
            .map(|(path, _)| fs::read(path).unwrap())
            .collect();

        let error = prepare_launch_paths(root.to_str().unwrap(), || Ok(false), |_| None)
            .unwrap_err()
            .into_string();

        assert!(error.contains("STEAM_NOT_FOUND"));
        for ((path, _), expected) in protected.iter().zip(before) {
            assert_eq!(fs::read(path).unwrap(), expected);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn mutations_converge_to_the_aggregated_disk_snapshot() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("panel-contract-{nonce}"));
        let csgo = root.join("game/csgo");
        for relative in [
            "backup/Online",
            "backup/WithBots",
            "cfg",
            "overrides/Low",
            "overrides/Medium",
            "overrides/High",
        ] {
            fs::create_dir_all(csgo.join(relative)).unwrap();
        }
        fs::write(csgo.join("gameinfo.gi"), b"online").unwrap();
        fs::write(csgo.join("backup/Online/gameinfo.gi"), b"online").unwrap();
        fs::write(
            csgo.join("backup/WithBots/gameinfo.gi"),
            b"Game csgo/addons/metamod",
        )
        .unwrap();
        for level in ["Low", "Medium", "High"] {
            fs::write(
                csgo.join(format!("overrides/{level}/botprofile.vpk")),
                level.as_bytes(),
            )
            .unwrap();
        }
        let initial_cfg = "echo user\r\nbind f1 \"say hello\"\r\nbind \\ \"subclass_create 500;subclass_create 526\"\r\n";
        for relative in CFG_FILES {
            fs::write(csgo.join(relative), initial_cfg).unwrap();
        }
        fs::create_dir_all(csgo.join("addons/counterstrikesharp/configs")).unwrap();
        fs::write(csgo.join(CORE_CONFIG_FILE), b"{}").unwrap();

        let root_text = root.to_string_lossy();
        set_mode(&root_text, "bots").unwrap();
        set_difficulty(&root_text, "High").unwrap();
        set_preset(&root_text, "bot_aim", "head").unwrap();
        set_preset(&root_text, "bot_nades", "off").unwrap();
        set_bot_item(&root_text, "music", true).unwrap();
        let result = set_drop_knives(&root_text, "f8", &[526, 500, 500]).unwrap();

        assert_eq!(result.mode.current.as_deref(), Some("bots"));
        assert_eq!(result.difficulty.current.as_deref(), Some("High"));
        assert_eq!(result.presets.aim.as_deref(), Some("head"));
        assert_eq!(result.presets.nades.as_deref(), Some("off"));
        assert!(result.bot_items.music);
        assert_eq!(result.drop_knives.bind_key, "f8");
        assert_eq!(result.drop_knives.selected, vec![500, 526]);
        let cfg = fs::read_to_string(csgo.join(CFG_FILES[0])).unwrap();
        assert!(cfg.contains("bind f1 \"say hello\""));
        assert!(cfg.contains("bind f8 \"subclass_create 500;subclass_create 526\""));
        assert_eq!(cfg.matches("bot_aim head").count(), 1);
        assert_eq!(cfg.matches("bot_nades off").count(), 1);
        assert!(fs::read_dir(&csgo).unwrap().flatten().any(|entry| entry
            .file_name()
            .to_string_lossy()
            .starts_with("gameinfo.gi.backup-")));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn fresh_defaults_are_written_once_and_are_idempotent() {
        let root = test_root("fresh-defaults");
        create_panel_environment(&root, "bots", "Low", "echo template\n", None);

        let result = initialize_panel_defaults_at_with_running(&root, true, false).unwrap();
        assert_eq!(result.status, "initialized");
        assert_eq!(result.initialized_fields.len(), 6);
        let snapshot = snapshot_at(&root).unwrap();
        assert_eq!(snapshot.mode.current.as_deref(), Some("bots"));
        assert_eq!(snapshot.difficulty.current.as_deref(), Some("Low"));
        assert_eq!(snapshot.presets.aim.as_deref(), Some("mixed"));
        assert_eq!(snapshot.presets.nades.as_deref(), Some("less"));
        assert!(
            snapshot.bot_items.profiles
                && snapshot.bot_items.agents
                && snapshot.bot_items.music
                && snapshot.bot_items.weapons
                && snapshot.bot_items.knives
                && snapshot.bot_items.gloves
                && snapshot.bot_items.stickers
                && snapshot.bot_items.charms
        );
        assert_eq!(snapshot.drop_knives.selected, DEFAULT_KNIVES);
        for relative in CFG_FILES {
            let cfg = fs::read_to_string(root.join("game/csgo").join(relative)).unwrap();
            assert!(cfg.contains("subclass_create 507;subclass_create 508;subclass_create 515;subclass_create 519;subclass_create 525"));
            assert_eq!(cfg.matches("bot_nades less").count(), 1);
        }
        let state_path = root.join("game/csgo").join(PANEL_STATE_FILE);
        let state_before = fs::read(&state_path).unwrap();
        let backups_before = fs::read_dir(root.join("game/csgo/cfg"))
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().contains("backup-"))
            .count();

        let second = initialize_panel_defaults_at_with_running(&root, false, false).unwrap();
        assert_eq!(second.status, "unchanged");
        assert!(second.initialized_fields.is_empty());
        assert_eq!(fs::read(state_path).unwrap(), state_before);
        let backups_after = fs::read_dir(root.join("game/csgo/cfg"))
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().contains("backup-"))
            .count();
        assert_eq!(backups_after, backups_before);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn existing_explicit_values_including_all_off_and_empty_knives_are_preserved() {
        let root = test_root("existing-values");
        create_panel_environment(
            &root,
            "online",
            "High",
            "echo user\nbot_aim head\nbot_nades normal\n",
            Some(
                r#"{"bot_hider github.com/XBribo all":false,"bot_randomizer github.com/ed0ard agents":false,"bot_randomizer github.com/ed0ard music":false,"bot_randomizer github.com/ed0ard weapons":false,"bot_randomizer github.com/ed0ard knives":false,"bot_randomizer github.com/ed0ard gloves":false,"bot_randomizer github.com/ed0ard stickers":false,"bot_randomizer github.com/ed0ard charms":false}"#,
            ),
        );
        initialize_panel_defaults_at_with_running(&root, false, false).unwrap();
        let snapshot = snapshot_at(&root).unwrap();
        assert_eq!(snapshot.mode.current.as_deref(), Some("online"));
        assert_eq!(snapshot.difficulty.current.as_deref(), Some("High"));
        assert_eq!(snapshot.presets.aim.as_deref(), Some("head"));
        assert_eq!(snapshot.presets.nades.as_deref(), Some("normal"));
        assert!(
            !snapshot.bot_items.profiles
                && !snapshot.bot_items.agents
                && !snapshot.bot_items.music
                && !snapshot.bot_items.weapons
                && !snapshot.bot_items.knives
                && !snapshot.bot_items.gloves
                && !snapshot.bot_items.stickers
                && !snapshot.bot_items.charms
        );
        assert!(snapshot.drop_knives.selected.is_empty());
        let preferences = load_preferences(&root.join("game/csgo")).unwrap();
        assert!(preferences
            .drop_knives
            .is_some_and(|field| field.selected.is_empty()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn less_roundtrips_to_both_cfg_files_and_rejects_untrusted_values() {
        let root = test_root("less-roundtrip");
        create_panel_environment(
            &root,
            "bots",
            "Low",
            "bot_aim mixed\nbot_nades normal\nbot_nades max\n",
            Some("{}"),
        );
        let root_text = root.to_string_lossy();
        let snapshot = set_preset(&root_text, "bot_nades", "less").unwrap();
        assert_eq!(snapshot.presets.nades.as_deref(), Some("less"));
        for relative in CFG_FILES {
            let cfg = fs::read_to_string(root.join("game/csgo").join(relative)).unwrap();
            assert_eq!(cfg.matches("bot_nades less").count(), 1);
            assert_eq!(
                cfg.lines()
                    .filter(|line| line.starts_with("bot_nades "))
                    .count(),
                1
            );
        }
        assert_eq!(
            snapshot_at(&root).unwrap().presets.nades.as_deref(),
            Some("less")
        );
        let before = fs::read(root.join("game/csgo").join(CFG_FILES[0])).unwrap();
        let error = set_preset(&root_text, "bot_nades", "less;quit")
            .unwrap_err()
            .into_string();
        assert!(error.contains("PANEL_PRESET_INVALID"));
        assert_eq!(
            fs::read(root.join("game/csgo").join(CFG_FILES[0])).unwrap(),
            before
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_four_item_state_with_skins_loads_without_startup_failure() {
        let root = test_root("legacy-state");
        create_panel_environment(
            &root,
            "bots",
            "Low",
            "bot_aim mixed\nbot_nades normal\n",
            Some("{}"),
        );
        let state = serde_json::json!({
            "schema": 1,
            "initializedBy": "0.5.3",
            "botItems": {
                "initialized": true,
                "skins": true,
                "profiles": false,
                "agents": true,
                "music": false
            }
        });
        fs::write(
            root.join("game/csgo").join(PANEL_STATE_FILE),
            serde_json::to_vec(&state).unwrap(),
        )
        .unwrap();
        let preferences = load_preferences(&root.join("game/csgo")).unwrap();
        let items = preferences
            .bot_items
            .expect("legacy Bot Items must deserialize");
        assert!(!items.profiles && items.agents && !items.music);
        assert!(
            !items.weapons && !items.knives && !items.gloves && !items.stickers && !items.charms
        );
        assert!(!fs::read_dir(root.join("game/csgo/cfg"))
            .unwrap()
            .flatten()
            .any(|entry| entry.file_name().to_string_lossy().contains("corrupt-")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupt_state_is_archived_and_recovered_from_disk() {
        let root = test_root("corrupt-state");
        create_panel_environment(
            &root,
            "online",
            "High",
            "bot_aim body\nbot_nades max\n",
            Some(
                r#"{"futureOption":42,"bot_hider github.com/XBribo all":true,"bot_randomizer github.com/ed0ard agents":false,"bot_randomizer github.com/ed0ard music":true}"#,
            ),
        );
        let csgo = root.join("game/csgo");
        fs::write(csgo.join(PANEL_STATE_FILE), b"{broken").unwrap();
        initialize_panel_defaults_at_with_running(&root, false, false).unwrap();
        let snapshot = snapshot_at(&root).unwrap();
        assert_eq!(snapshot.presets.aim.as_deref(), Some("body"));
        assert_eq!(snapshot.presets.nades.as_deref(), Some("max"));
        assert!(fs::read_dir(csgo.join("cfg"))
            .unwrap()
            .flatten()
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .contains("panel-state.json.corrupt-")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupt_core_json_is_reported_and_not_overwritten() {
        let root = test_root("corrupt-items");
        create_panel_environment(
            &root,
            "bots",
            "Low",
            "bot_aim mixed\nbot_nades normal\n",
            Some("{broken"),
        );
        let csgo = root.join("game/csgo");
        let error = initialize_panel_defaults_at_with_running(&root, false, false)
            .unwrap_err()
            .into_string();
        assert!(error.contains("PANEL_BOT_ITEM_CORRUPT"));
        assert_eq!(fs::read(csgo.join(CORE_CONFIG_FILE)).unwrap(), b"{broken");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn initialization_is_deferred_without_writes_while_cs2_runs() {
        let root = test_root("deferred");
        create_panel_environment(&root, "bots", "Low", "echo template\n", None);
        let result = initialize_panel_defaults_at_with_running(&root, false, true).unwrap();
        assert_eq!(result.status, "deferred");
        assert!(!root.join("game/csgo").join(PANEL_STATE_FILE).exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn demo_recording_block_preserves_unknown_lines_and_converges_duplicates() {
        let root = test_root("demo-recording-block");
        fs::create_dir_all(&root).unwrap();
        let cfg = root.join("managed.cfg");
        fs::write(&cfg, "echo before\r\n// CS2AS05 DEMO RECORDING BEGIN\r\ntv_enable 0\r\n// CS2AS05 DEMO RECORDING END\r\necho after\r\n// CS2AS05 DEMO RECORDING BEGIN\r\ntv_autorecord 0\r\n// CS2AS05 DEMO RECORDING END\r\n").unwrap();
        replace_demo_recording_block(&cfg, true).unwrap();
        let text = fs::read_to_string(&cfg).unwrap();
        assert!(text.contains("echo before\r\n"));
        assert!(text.contains("echo after\r\n"));
        assert_eq!(text.matches(DEMO_RECORDING_BEGIN).count(), 1);
        assert_eq!(text.matches("tv_enable 1").count(), 1);
        assert_eq!(text.matches("tv_autorecord 1").count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn demo_recording_block_preserves_utf8_bom_and_supports_disabled_state() {
        let root = test_root("demo-recording-bom");
        fs::create_dir_all(&root).unwrap();
        let cfg = root.join("managed.cfg");
        fs::write(&cfg, "\u{feff}echo custom\n").unwrap();
        replace_demo_recording_block(&cfg, false).unwrap();
        let bytes = fs::read(&cfg).unwrap();
        assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]));
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("echo custom\n"));
        assert!(text.contains("tv_enable 0\n"));
        assert!(text.contains("tv_autorecord 0\n"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "requires CS2_PANEL_REAL_ROOT and mutates then restores the real gameinfo.gi"]
    fn real_cs2_online_bots_roundtrip_restores_initial_bytes() {
        let root = std::env::var("CS2_PANEL_REAL_ROOT")
            .expect("set CS2_PANEL_REAL_ROOT to a backed-up CS2 installation");
        let normalized = cs2::normalize_root(&root).unwrap();
        let active = normalized.join("game/csgo/gameinfo.gi");
        let initial = fs::read(&active).unwrap();

        let result = (|| {
            let online = set_mode(&root, "online")?;
            assert_eq!(online.mode.current.as_deref(), Some("online"));
            let bots = set_mode(&root, "bots")?;
            assert_eq!(bots.mode.current.as_deref(), Some("bots"));
            Ok::<(), AppError>(())
        })();

        atomic_write(&active, &initial).expect("the initial gameinfo.gi must be restored");
        assert_eq!(fs::read(&active).unwrap(), initial);
        result.unwrap();
    }
}
