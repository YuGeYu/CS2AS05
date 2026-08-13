use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

use crate::errors::AppError;
use crate::services::cs2;

const UPSTREAM_COMMIT: &str = "75f52fbd5fd0616dbbdd09a65c3a1981593400d1";
const PLUGIN_VERSION: &str = "1.8.2";
const MAX_LOADOUT_BYTES: usize = 1_048_576;
const MAX_IMAGE_BYTES: u64 = 2 * 1_048_576;
const MAX_IMAGE_CACHE_BYTES: u64 = 256 * 1_048_576;
const MAX_IMAGE_FETCHES: usize = 6;
static ACTIVE_IMAGE_FETCHES: AtomicUsize = AtomicUsize::new(0);
const PLUGIN_RELATIVE: &str = "addons/counterstrikesharp/plugins/PlayerSkinMod";
const REQUIRED_RESOURCES: &[(&str, &str)] = &[
    (
        "PlayerSkinMod.dll",
        "0DAC2B47275EC6AD308F8F5F4797140D1AA8BA63DA93F2840DA90BA9A3838388",
    ),
    (
        "PlayerSkinMod.json",
        "D9D686289899663C92795C04738B563E065DF8B7170DF02B8518F2C666A19706",
    ),
    (
        "skins_en.json",
        "DFD0A2CB407065FC0B567E67891FC7E2DFA3AB1C794D1708FF1DDBCBAD29F98B",
    ),
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    loadout_path: String,
    sha256: String,
    slot_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCheckResult {
    all_present: bool,
    missing_files: Vec<String>,
    hash_mismatches: Vec<String>,
    version_mismatch: bool,
    deployed_version: Option<String>,
    panel_version: String,
    counterstrikesharp_installed: bool,
    counterstrikesharp_version: Option<String>,
    player_skin_mod_present: bool,
    manifest_version: Option<String>,
    resource_version: String,
    resource_hashes_match: bool,
    loadout_readable: bool,
    can_deploy: bool,
    blocked_code: Option<String>,
    blocked_message: Option<String>,
    selected_root: String,
    csgo_root: String,
    target_dir: String,
    loadout_path: String,
    hashes: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployResult {
    target_dir: String,
    files: Vec<String>,
    plugin_version: String,
    hashes: BTreeMap<String, String>,
    loadout_path: String,
    counterstrikesharp_installed: bool,
    backup_dir: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedImage {
    path: String,
    sha256: String,
    mime: String,
    bytes: usize,
    cache_hit: bool,
}

#[tauri::command]
pub fn skin_forge_get_config(app: AppHandle) -> Result<serde_json::Value, String> {
    let draft = draft_file(&app).map_err(AppError::into_string)?;
    Ok(serde_json::json!({ "language": "zh-CN", "cs2Path": null, "draftPath": draft }))
}

#[tauri::command]
pub async fn skin_forge_cache_image(app: AppHandle, url: String) -> Result<CachedImage, String> {
    cache_image(&app, &url).await.map_err(AppError::into_string)
}

#[tauri::command]
pub fn skin_forge_load_loadout(root_path: String, _slot: u8) -> Result<serde_json::Value, String> {
    let path = loadout_file(&root_path).map_err(AppError::into_string)?;
    if !path.is_file() {
        return Ok(serde_json::Value::Null);
    }
    let bytes = fs::read(&path).map_err(|error| format!("读取皮肤配置失败：{error}"))?;
    if bytes.len() > MAX_LOADOUT_BYTES {
        return Err("[LOADOUT_TOO_LARGE] 皮肤配置超过 1 MiB。".into());
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("[LOADOUT_INVALID] 皮肤配置 JSON 损坏：{error}"))
}

#[tauri::command]
pub fn skin_forge_save_loadout(
    app: AppHandle,
    root_path: String,
    slot: u8,
    loadout: serde_json::Value,
) -> Result<SaveResult, String> {
    require_plugin_ready(&root_path).map_err(AppError::into_string)?;
    validate_loadout(&loadout, slot).map_err(AppError::into_string)?;
    let bytes = serde_json::to_vec_pretty(&loadout).map_err(|error| error.to_string())?;
    if bytes.len() > MAX_LOADOUT_BYTES {
        return Err("[LOADOUT_TOO_LARGE] 皮肤配置超过 1 MiB。".into());
    }
    let target = loadout_file(&root_path).map_err(AppError::into_string)?;
    let draft = draft_file(&app).map_err(AppError::into_string)?;
    write_atomically(&draft, &bytes).map_err(AppError::into_string)?;
    write_atomically(&target, &bytes).map_err(AppError::into_string)?;
    let readback = fs::read(&target).map_err(|error| error.to_string())?;
    let parsed: serde_json::Value =
        serde_json::from_slice(&readback).map_err(|error| error.to_string())?;
    Ok(SaveResult {
        loadout_path: target.display().to_string(),
        sha256: sha256_bytes(&readback),
        slot_count: parsed.as_object().map_or(0, serde_json::Map::len),
    })
}

#[tauri::command]
pub fn skin_forge_reset_loadout(root_path: String, slot: u8) -> Result<SaveResult, String> {
    require_plugin_ready(&root_path).map_err(AppError::into_string)?;
    let target = loadout_file(&root_path).map_err(AppError::into_string)?;
    let mut file = if target.is_file() {
        serde_json::from_slice::<serde_json::Value>(&fs::read(&target).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        serde_json::json!({})
    };
    let slot_count = {
        let object = file.as_object_mut().ok_or("皮肤配置根节点必须是对象。")?;
        object.remove(&slot.to_string());
        object.len()
    };
    let bytes = serde_json::to_vec_pretty(&file).map_err(|e| e.to_string())?;
    write_atomically(&target, &bytes).map_err(AppError::into_string)?;
    Ok(SaveResult {
        loadout_path: target.display().to_string(),
        sha256: sha256_bytes(&bytes),
        slot_count,
    })
}

#[tauri::command]
pub fn skin_forge_check_plugin(root_path: String) -> Result<PluginCheckResult, String> {
    check_plugin(&root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn skin_forge_deploy_plugin(app: AppHandle, root_path: String) -> Result<DeployResult, String> {
    if cs2::check_cs2_process().map_err(AppError::into_string)? {
        return Err("[CS2_RUNNING] 检测到 cs2.exe 正在运行。请退出 CS2 后再部署插件。".into());
    }
    let csgo = resolve_csgo_root(&root_path).map_err(AppError::into_string)?;
    let api = csgo.join("addons/counterstrikesharp/api/CounterStrikeSharp.API.dll");
    if !api.is_file() {
        return Err("[COUNTERSTRIKESHARP_MISSING] 当前目录没有 CounterStrikeSharp.API.dll；请先安装现有 Bot Improver 基础环境。".into());
    }
    let source = resolve_resource_dir(&app).map_err(AppError::into_string)?;
    verify_resources(&source).map_err(AppError::into_string)?;
    let target = plugin_dir(&csgo).map_err(AppError::into_string)?;
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;
    let backup = backup_existing(&app, &target).map_err(AppError::into_string)?;
    let deploy = deploy_resource_files(&source, &target);
    if let Err(error) = deploy {
        restore_backup(&target, backup.as_deref()).map_err(AppError::into_string)?;
        return Err(format!(
            "[DEPLOY_ROLLED_BACK] 部署失败并已恢复备份：{}",
            error.into_string()
        ));
    }
    let check = check_plugin(&root_path).map_err(AppError::into_string)?;
    if !check.all_present || !check.hash_mismatches.is_empty() {
        restore_backup(&target, backup.as_deref()).map_err(AppError::into_string)?;
        return Err("[DEPLOY_VERIFY_FAILED] 部署后哈希校验失败，已恢复备份。".into());
    }
    Ok(DeployResult {
        target_dir: target.display().to_string(),
        files: REQUIRED_RESOURCES
            .iter()
            .map(|(name, _)| (*name).to_string())
            .collect(),
        plugin_version: PLUGIN_VERSION.into(),
        hashes: check.hashes,
        loadout_path: target.join("player_loadout.json").display().to_string(),
        counterstrikesharp_installed: true,
        backup_dir: backup.map(|path| path.display().to_string()),
    })
}

fn check_plugin(root_path: &str) -> Result<PluginCheckResult, AppError> {
    let csgo = resolve_csgo_root(root_path)?;
    let target = plugin_dir(&csgo)?;
    let mut missing = Vec::new();
    let mut mismatches = Vec::new();
    let mut hashes = BTreeMap::new();
    for (name, expected) in REQUIRED_RESOURCES {
        let path = target.join(name);
        if !path.is_file() {
            missing.push((*name).to_string());
            continue;
        }
        let actual = sha256_file(&path)?;
        hashes.insert((*name).to_string(), actual.clone());
        if actual != *expected {
            mismatches.push((*name).to_string());
        }
    }
    let loadout = target.join("player_loadout.json");
    let loadout_readable = !loadout.is_file()
        || fs::read(&loadout)
            .ok()
            .filter(|bytes| bytes.len() <= MAX_LOADOUT_BYTES)
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .is_some();
    let version = read_plugin_version(&target.join("PlayerSkinMod.json"));
    let version_mismatch = version
        .as_deref()
        .is_some_and(|value| value != PLUGIN_VERSION);
    let api = csgo
        .join("addons/counterstrikesharp/api/CounterStrikeSharp.API.dll")
        .is_file();
    let resources_match = missing.is_empty() && mismatches.is_empty();
    let (blocked_code, blocked_message) = if !api {
        (
            Some("COUNTERSTRIKESHARP_MISSING".into()),
            Some("未检测到 CounterStrikeSharp，不能部署 PlayerSkinMod。".into()),
        )
    } else if !loadout_readable {
        (
            Some("LOADOUT_INVALID".into()),
            Some("现有 player_loadout.json 无法读取；请先备份或修复配置。".into()),
        )
    } else {
        (None, None)
    };
    Ok(PluginCheckResult {
        all_present: resources_match && !version_mismatch && api,
        missing_files: missing,
        hash_mismatches: mismatches,
        version_mismatch,
        deployed_version: version.clone(),
        panel_version: UPSTREAM_COMMIT.into(),
        counterstrikesharp_installed: api,
        counterstrikesharp_version: api
            .then(|| "已检测 API DLL（运行时版本需由插件日志确认）".into()),
        player_skin_mod_present: target.join("PlayerSkinMod.dll").is_file(),
        manifest_version: version.clone(),
        resource_version: PLUGIN_VERSION.into(),
        resource_hashes_match: resources_match,
        loadout_readable,
        can_deploy: api && loadout_readable,
        blocked_code,
        blocked_message,
        selected_root: root_path.into(),
        csgo_root: csgo.display().to_string(),
        target_dir: target.display().to_string(),
        loadout_path: target.join("player_loadout.json").display().to_string(),
        hashes,
    })
}

fn require_plugin_ready(root_path: &str) -> Result<PluginCheckResult, AppError> {
    let check = check_plugin(root_path).map_err(|error| {
        AppError::runtime(format!(
            "[PLAYER_SKIN_MOD_REQUIRED] 无法确认 PlayerSkinMod 状态：{}。请先使用“部署 / 更新插件”。",
            error.into_string()
        ))
    })?;
    if !check.all_present || !check.hash_mismatches.is_empty() {
        return Err(AppError::runtime(
            "[PLAYER_SKIN_MOD_REQUIRED] PlayerSkinMod 尚未完整部署或文件校验未通过。请先使用“部署 / 更新插件”。",
        ));
    }
    Ok(check)
}

fn resolve_csgo_root(selected: &str) -> Result<PathBuf, AppError> {
    let selected = dunce::canonicalize(selected)
        .map_err(|e| AppError::runtime(format!("无法规范化 CS2 目录：{e}")))?;
    let candidate = if selected
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("csgo"))
        && selected
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|name| name.eq_ignore_ascii_case("game"))
    {
        selected
    } else {
        selected.join("game/csgo")
    };
    let csgo = dunce::canonicalize(&candidate)
        .map_err(|e| AppError::runtime(format!("未找到 game/csgo：{e}")))?;
    if !csgo.join("gameinfo.gi").is_file() {
        return Err(AppError::runtime(
            "[CS2_ROOT_INVALID] 目标目录缺少 gameinfo.gi。",
        ));
    }
    Ok(csgo)
}

fn plugin_dir(csgo: &Path) -> Result<PathBuf, AppError> {
    let target = csgo.join(PLUGIN_RELATIVE);
    if !target.starts_with(csgo) {
        return Err(AppError::runtime("[PATH_ESCAPE] 插件路径越过 CS2 根目录。"));
    }
    Ok(target)
}

fn loadout_file(root: &str) -> Result<PathBuf, AppError> {
    let csgo = resolve_csgo_root(root)?;
    Ok(plugin_dir(&csgo)?.join("player_loadout.json"))
}
fn draft_file(app: &AppHandle) -> Result<PathBuf, AppError> {
    Ok(app
        .path()
        .app_local_data_dir()
        .map_err(|e| AppError::runtime(e.to_string()))?
        .join("skin-forge/draft-player_loadout.json"))
}
fn resolve_resource_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let resource = app
        .path()
        .resource_dir()
        .map_err(|e| AppError::runtime(e.to_string()))?;
    let candidates = [
        resource.join("skin-forge/PlayerSkinMod"),
        resource.join("resources/skin-forge/PlayerSkinMod"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/skin-forge/PlayerSkinMod"),
    ];
    candidates
        .into_iter()
        .find(|path| path.join("PlayerSkinMod.dll").is_file())
        .ok_or_else(|| {
            AppError::runtime("[PLUGIN_RESOURCE_MISSING] 未找到内置 PlayerSkinMod 资源。")
        })
}
fn verify_resources(source: &Path) -> Result<(), AppError> {
    for (name, expected) in REQUIRED_RESOURCES {
        let path = source.join(name);
        if !path.is_file() {
            return Err(AppError::runtime(format!(
                "[PLUGIN_RESOURCE_MISSING] 缺少 {name}"
            )));
        }
        let actual = sha256_file(&path)?;
        if actual != *expected {
            return Err(AppError::runtime(format!(
                "[PLUGIN_RESOURCE_HASH] {name} 哈希不匹配。"
            )));
        }
    }
    Ok(())
}
fn deploy_resource_files(source: &Path, target: &Path) -> Result<(), AppError> {
    for (name, _) in REQUIRED_RESOURCES {
        write_atomically(
            &target.join(name),
            &fs::read(source.join(name)).map_err(io_error)?,
        )?;
    }
    Ok(())
}
fn validate_loadout(value: &serde_json::Value, slot: u8) -> Result<(), AppError> {
    let root = value
        .as_object()
        .ok_or_else(|| AppError::runtime("配置根节点必须是对象。"))?;
    if root.keys().any(|key| key.parse::<u32>().is_err()) {
        return Err(AppError::runtime("配置 slot key 必须是非负整数。"));
    }
    let item = root
        .get(&slot.to_string())
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| AppError::runtime(format!("配置缺少 slot {slot} 对象。")))?;
    for (name, value) in item {
        if name.to_ascii_lowercase().contains("wear") {
            validate_wear_value(value)?;
        }
        if name.to_ascii_lowercase().contains("seed") {
            validate_non_negative(value, "种子")?;
        }
        if name.starts_with("weapon") && value.is_object() {
            for key in value.as_object().into_iter().flat_map(|map| map.keys()) {
                if matches!(key.parse::<u32>(), Ok(1..=65_535)) {
                    continue;
                }
                return Err(AppError::runtime(format!("非法武器 defindex：{key}")));
            }
        }
        if name == "weaponStickers" {
            for stickers in value.as_object().into_iter().flat_map(|map| map.values()) {
                if stickers.as_array().is_some_and(|array| array.len() > 5) {
                    return Err(AppError::runtime("每把武器最多允许 5 个贴纸。"));
                }
            }
        }
    }
    Ok(())
}
fn validate_non_negative(value: &serde_json::Value, label: &str) -> Result<(), AppError> {
    if let Some(number) = value.as_i64() {
        if number < 0 {
            return Err(AppError::runtime(format!("{label}不能为负数。")));
        }
    } else if let Some(map) = value.as_object() {
        for item in map.values() {
            validate_non_negative(item, label)?;
        }
    }
    Ok(())
}
fn validate_wear_value(value: &serde_json::Value) -> Result<(), AppError> {
    if let Some(number) = value.as_f64() {
        if !(0.0..=1.0).contains(&number) {
            return Err(AppError::runtime("磨损值必须在 0..1。"));
        }
    } else if let Some(map) = value.as_object() {
        for item in map.values() {
            validate_wear_value(item)?;
        }
    }
    Ok(())
}
fn write_atomically(target: &Path, bytes: &[u8]) -> Result<(), AppError> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    let temp = target.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = File::create(&temp).map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    drop(file);
    if target.exists() {
        let old = target.with_extension("replace-backup");
        if old.exists() {
            fs::remove_file(&old).map_err(io_error)?;
        }
        fs::rename(target, &old).map_err(io_error)?;
        if let Err(error) = fs::rename(&temp, target) {
            let _ = fs::rename(&old, target);
            return Err(io_error(error));
        }
        fs::remove_file(old).map_err(io_error)?;
    } else {
        fs::rename(temp, target).map_err(io_error)?;
    }
    Ok(())
}
fn backup_existing(app: &AppHandle, target: &Path) -> Result<Option<PathBuf>, AppError> {
    if !target.is_dir() {
        return Ok(None);
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::runtime(e.to_string()))?
        .as_secs();
    let backup = app
        .path()
        .app_local_data_dir()
        .map_err(|e| AppError::runtime(e.to_string()))?
        .join(format!("skin-forge/backups/{stamp}"));
    copy_dir_recursive(target, &backup)?;
    Ok(Some(backup))
}
fn restore_backup(target: &Path, backup: Option<&Path>) -> Result<(), AppError> {
    if target.is_dir() {
        fs::remove_dir_all(target).map_err(io_error)?;
    }
    if let Some(source) = backup.filter(|path| path.is_dir()) {
        copy_dir_recursive(source, target)?;
    }
    Ok(())
}
fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), AppError> {
    fs::create_dir_all(target).map_err(io_error)?;
    for entry in fs::read_dir(source).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let destination = target.join(entry.file_name());
        if entry.file_type().map_err(io_error)?.is_dir() {
            copy_dir_recursive(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), destination).map_err(io_error)?;
        }
    }
    Ok(())
}
fn read_plugin_version(path: &Path) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    value
        .pointer("/PluginProfile/Version")?
        .as_str()
        .map(str::to_string)
}
fn sha256_file(path: &Path) -> Result<String, AppError> {
    let mut file = File::open(path).map_err(io_error)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65_536];
    loop {
        let count = file.read(&mut buffer).map_err(io_error)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:X}", hasher.finalize()))
}
fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:X}", hasher.finalize())
}
async fn cache_image(app: &AppHandle, input: &str) -> Result<CachedImage, AppError> {
    let url = validate_image_url(input)?;
    let key = sha256_bytes(input.as_bytes());
    let cache_root = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(error.to_string()))?
        .join("skin-forge/cache/images");
    fs::create_dir_all(&cache_root).map_err(io_error)?;
    if let Some(hit) = read_cached_image(&cache_root, &key)? {
        return Ok(hit);
    }
    let _fetch_guard = ImageFetchGuard::acquire()?;
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 3 {
                return attempt.error("too many redirects");
            }
            if validate_image_url(attempt.url().as_str()).is_err() {
                return attempt.error("redirect target is outside the image allowlist");
            }
            attempt.follow()
        }))
        .build()
        .map_err(|error| AppError::runtime(error.to_string()))?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| AppError::runtime(format!("[IMAGE_FETCH_FAILED] {error}")))?
        .error_for_status()
        .map_err(|error| AppError::runtime(format!("[IMAGE_HTTP_FAILED] {error}")))?;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_IMAGE_BYTES)
    {
        return Err(AppError::runtime("[IMAGE_TOO_LARGE] 图片超过 2 MiB。"));
    }
    let mime = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .split(';')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let extension = extension_for_mime(&mime)?;
    let bytes = response
        .bytes()
        .await
        .map_err(|error| AppError::runtime(format!("[IMAGE_READ_FAILED] {error}")))?;
    if bytes.len() as u64 > MAX_IMAGE_BYTES {
        return Err(AppError::runtime("[IMAGE_TOO_LARGE] 图片超过 2 MiB。"));
    }
    let target = cache_root.join(format!("{key}.{extension}"));
    if !target.starts_with(&cache_root) {
        return Err(AppError::runtime("[IMAGE_PATH_ESCAPE] 缓存路径越界。"));
    }
    prune_image_cache(
        &cache_root,
        MAX_IMAGE_CACHE_BYTES.saturating_sub(bytes.len() as u64),
    )?;
    write_atomically(&target, &bytes)?;
    Ok(CachedImage {
        path: target.display().to_string(),
        sha256: sha256_bytes(&bytes),
        mime,
        bytes: bytes.len(),
        cache_hit: false,
    })
}

struct ImageFetchGuard;

impl ImageFetchGuard {
    fn acquire() -> Result<Self, AppError> {
        ACTIVE_IMAGE_FETCHES
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < MAX_IMAGE_FETCHES).then_some(active + 1)
            })
            .map_err(|_| AppError::runtime("[IMAGE_BUSY] 图片缓存任务繁忙，请稍后重试。"))?;
        Ok(Self)
    }
}

impl Drop for ImageFetchGuard {
    fn drop(&mut self) {
        ACTIVE_IMAGE_FETCHES.fetch_sub(1, Ordering::AcqRel);
    }
}

fn validate_image_url(input: &str) -> Result<reqwest::Url, AppError> {
    let url = reqwest::Url::parse(input)
        .map_err(|_| AppError::runtime("[IMAGE_URL_INVALID] 图片地址无效。"))?;
    if url.scheme() != "https" {
        return Err(AppError::runtime(
            "[IMAGE_SCHEME_BLOCKED] 图片只允许 HTTPS。",
        ));
    }
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    let allowed = host == "cdn.steamstatic.com"
        || host == "community.akamai.steamstatic.com"
        || host.ends_with(".steamstatic.com");
    if !allowed {
        return Err(AppError::runtime(
            "[IMAGE_HOST_BLOCKED] 图片域名不在允许列表。",
        ));
    }
    Ok(url)
}

fn extension_for_mime(mime: &str) -> Result<&'static str, AppError> {
    match mime {
        "image/png" => Ok("png"),
        "image/jpeg" => Ok("jpg"),
        "image/webp" => Ok("webp"),
        "image/avif" => Ok("avif"),
        _ => Err(AppError::runtime(
            "[IMAGE_MIME_BLOCKED] 远端内容不是允许的图片格式。",
        )),
    }
}

fn mime_for_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "png" => Some("image/png"),
        "jpg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "avif" => Some("image/avif"),
        _ => None,
    }
}

fn read_cached_image(cache_root: &Path, key: &str) -> Result<Option<CachedImage>, AppError> {
    for extension in ["png", "jpg", "webp", "avif"] {
        let existing = cache_root.join(format!("{key}.{extension}"));
        if !existing.is_file() {
            continue;
        }
        let metadata = fs::metadata(&existing).map_err(io_error)?;
        if metadata.len() > MAX_IMAGE_BYTES {
            fs::remove_file(&existing).map_err(io_error)?;
            continue;
        }
        let bytes = fs::read(&existing).map_err(io_error)?;
        return Ok(Some(CachedImage {
            path: existing.display().to_string(),
            sha256: sha256_bytes(&bytes),
            mime: mime_for_extension(extension).unwrap_or_default().into(),
            bytes: bytes.len(),
            cache_hit: true,
        }));
    }
    Ok(None)
}

fn prune_image_cache(cache_root: &Path, target_bytes: u64) -> Result<(), AppError> {
    let mut files = fs::read_dir(cache_root)
        .map_err(io_error)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            metadata.is_file().then(|| {
                (
                    entry.path(),
                    metadata.len(),
                    metadata.modified().unwrap_or(UNIX_EPOCH),
                )
            })
        })
        .collect::<Vec<_>>();
    let mut total = files.iter().map(|(_, size, _)| size).sum::<u64>();
    files.sort_by_key(|(_, _, modified)| *modified);
    for (path, size, _) in files {
        if total <= target_bytes {
            break;
        }
        fs::remove_file(path).map_err(io_error)?;
        total = total.saturating_sub(size);
    }
    Ok(())
}
fn io_error(error: std::io::Error) -> AppError {
    AppError::runtime(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_DIR_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

    fn test_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "skin-forge-{label}-{}-{}",
            std::process::id(),
            TEST_DIR_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn fake_cs2_root(label: &str) -> (PathBuf, PathBuf) {
        let root = test_dir(label);
        let csgo = root.join("game/csgo");
        fs::create_dir_all(csgo.join("addons/counterstrikesharp/api")).unwrap();
        fs::write(csgo.join("gameinfo.gi"), b"test").unwrap();
        fs::write(
            csgo.join("addons/counterstrikesharp/api/CounterStrikeSharp.API.dll"),
            b"test",
        )
        .unwrap();
        (root, csgo)
    }

    fn bundled_resources() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/skin-forge/PlayerSkinMod")
    }
    #[test]
    fn rejects_missing_slot_and_excess_stickers() {
        assert!(validate_loadout(&serde_json::json!({}), 0).is_err());
        assert!(validate_loadout(
            &serde_json::json!({"0":{"weaponStickers":{"7":[1,2,3,4,5,6]}}}),
            0
        )
        .is_err());
    }
    #[test]
    fn accepts_player_skin_mod_slot_contract() {
        assert!(validate_loadout(&serde_json::json!({"0":{"weaponWearsCt":{"7":0.12},"knifeWearCt":0.2,"weaponStickers":{"7":[{"id":1}]}}}), 0).is_ok());
    }
    #[test]
    fn fixed_resource_hashes_match_bundle() {
        let root = bundled_resources();
        verify_resources(&root).unwrap();
    }
    #[test]
    fn resolves_game_root_and_direct_csgo_path() {
        let root = test_dir("root");
        let csgo = root.join("game/csgo");
        fs::create_dir_all(&csgo).unwrap();
        fs::write(csgo.join("gameinfo.gi"), b"test").unwrap();
        assert_eq!(
            resolve_csgo_root(root.to_str().unwrap()).unwrap(),
            dunce::canonicalize(&csgo).unwrap()
        );
        assert_eq!(
            resolve_csgo_root(csgo.to_str().unwrap()).unwrap(),
            dunce::canonicalize(&csgo).unwrap()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn image_cache_rejects_non_https_and_unlisted_hosts() {
        assert!(validate_image_url("http://cdn.steamstatic.com/a.png").is_err());
        assert!(validate_image_url("https://steamstatic.com.evil.example/a.png").is_err());
        assert!(validate_image_url("https://cdn.steamstatic.com/a.png").is_ok());
        assert!(validate_image_url("https://community.akamai.steamstatic.com/a.png").is_ok());
    }

    #[test]
    fn image_cache_accepts_only_supported_mime_types() {
        assert_eq!(extension_for_mime("image/png").unwrap(), "png");
        assert_eq!(extension_for_mime("image/jpeg").unwrap(), "jpg");
        assert!(extension_for_mime("image/svg+xml").is_err());
        assert!(extension_for_mime("text/html").is_err());
    }

    #[test]
    fn image_cache_hit_is_bounded_and_reports_canonical_mime() {
        let root = test_dir("cache");
        fs::create_dir_all(&root).unwrap();
        let key = "ABC";
        fs::write(root.join(format!("{key}.jpg")), b"jpeg-test").unwrap();
        let hit = read_cached_image(&root, key).unwrap().unwrap();
        assert!(hit.cache_hit);
        assert_eq!(hit.mime, "image/jpeg");
        assert!(Path::new(&hit.path).starts_with(&root));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn image_cache_prunes_oldest_files_to_size_limit() {
        let root = test_dir("prune");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("a.png"), [0; 8]).unwrap();
        fs::write(root.join("b.png"), [0; 8]).unwrap();
        prune_image_cache(&root, 0).unwrap();
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn plugin_check_accepts_current_resources_without_loadout() {
        let (root, csgo) = fake_cs2_root("no-loadout");
        let target = plugin_dir(&csgo).unwrap();
        fs::create_dir_all(&target).unwrap();
        deploy_resource_files(&bundled_resources(), &target).unwrap();
        let check = check_plugin(root.to_str().unwrap()).unwrap();
        assert!(check.all_present);
        assert!(check.loadout_readable);
        assert!(check.can_deploy);
        assert_eq!(check.deployed_version.as_deref(), Some(PLUGIN_VERSION));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn plugin_check_detects_old_version_and_corrupted_hash() {
        let (root, csgo) = fake_cs2_root("old-version");
        let target = plugin_dir(&csgo).unwrap();
        fs::create_dir_all(&target).unwrap();
        deploy_resource_files(&bundled_resources(), &target).unwrap();
        let manifest = target.join("PlayerSkinMod.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
        value["PluginProfile"]["Version"] = serde_json::json!("1.8.0");
        fs::write(&manifest, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        fs::write(target.join("PlayerSkinMod.dll"), b"corrupted").unwrap();
        let check = check_plugin(root.to_str().unwrap()).unwrap();
        assert!(!check.all_present);
        assert!(check.version_mismatch);
        assert!(check.hash_mismatches.contains(&"PlayerSkinMod.dll".into()));
        assert!(check.hash_mismatches.contains(&"PlayerSkinMod.json".into()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn require_plugin_ready_accepts_complete_resources_and_rejects_hash_mismatch() {
        let (root, csgo) = fake_cs2_root("require-ready");
        let target = plugin_dir(&csgo).unwrap();
        fs::create_dir_all(&target).unwrap();
        deploy_resource_files(&bundled_resources(), &target).unwrap();
        assert!(require_plugin_ready(root.to_str().unwrap()).is_ok());
        fs::write(target.join("PlayerSkinMod.dll"), b"corrupted").unwrap();
        let error = match require_plugin_ready(root.to_str().unwrap()) {
            Ok(_) => panic!("corrupted plugin must be rejected"),
            Err(error) => error.into_string(),
        };
        assert!(error.starts_with("[PLAYER_SKIN_MOD_REQUIRED]"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn full_directory_restore_preserves_player_loadout() {
        let root = test_dir("restore");
        let target = root.join("target");
        let backup = root.join("backup");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("PlayerSkinMod.dll"), b"old").unwrap();
        fs::write(
            target.join("player_loadout.json"),
            br#"{"0":{"custom":"keep"}}"#,
        )
        .unwrap();
        copy_dir_recursive(&target, &backup).unwrap();
        fs::write(target.join("PlayerSkinMod.dll"), b"partial-new").unwrap();
        fs::remove_file(target.join("player_loadout.json")).unwrap();
        restore_backup(&target, Some(&backup)).unwrap();
        assert_eq!(fs::read(target.join("PlayerSkinMod.dll")).unwrap(), b"old");
        assert_eq!(
            fs::read(target.join("player_loadout.json")).unwrap(),
            br#"{"0":{"custom":"keep"}}"#
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_resource_copy_can_be_rolled_back_as_a_directory() {
        let root = test_dir("rollback");
        let source = root.join("source");
        let target = root.join("target");
        let backup = root.join("backup");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(source.join(REQUIRED_RESOURCES[0].0), b"new-first-file").unwrap();
        fs::write(target.join("PlayerSkinMod.dll"), b"old-dll").unwrap();
        fs::write(target.join("player_loadout.json"), b"loadout").unwrap();
        copy_dir_recursive(&target, &backup).unwrap();
        assert!(deploy_resource_files(&source, &target).is_err());
        restore_backup(&target, Some(&backup)).unwrap();
        assert_eq!(
            fs::read(target.join("PlayerSkinMod.dll")).unwrap(),
            b"old-dll"
        );
        assert_eq!(
            fs::read(target.join("player_loadout.json")).unwrap(),
            b"loadout"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
