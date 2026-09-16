use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

use crate::errors::AppError;
use crate::services::{cs2, support};

const RESOURCE_VERSION: &str = "3.1.1-cs2as.1";
const UPSTREAM_TAG: &str = "3.1.1";
const UPSTREAM_COMMIT: &str = "6d2cb8a99e021b88523b77079b4a4ce765b44807";
const PLUGIN_PARENT_RELATIVE: &str = "addons/counterstrikesharp/plugins";
const PLUGIN_NAME: &str = "InventorySimulator";
const LEGACY_PLUGIN_NAME: &str = "PlayerSkinMod";
const GAMEDATA_RELATIVE: &str = "addons/counterstrikesharp/gamedata/inventory-simulator.json";
const CONFIG_RELATIVE: &str = "addons/counterstrikesharp/configs/plugins/InventorySimulator";
const CORE_CONFIG_RELATIVE: &str = "addons/counterstrikesharp/configs/core.json";
const API_RELATIVE: &str = "addons/counterstrikesharp/api/CounterStrikeSharp.API.dll";
const SERVICE_URL: &str = "https://inventory.cstrike.app";

const PLUGIN_RESOURCES: &[(&str, &str, &str)] = &[
    (
        "InventorySimulator.dll",
        "InventorySimulator.dll",
        "8138F3357E0893866C1A4DB55B4165A5BD060670A22080C721C283C90EAFF8A6",
    ),
    (
        "InventorySimulator.deps.json",
        "InventorySimulator.deps.json",
        "47F5066E468D33A99D550CD229085DB2F9B2B36DCFBD320737EB6ACB4204C23C",
    ),
    (
        "InventorySimulator.pdb",
        "InventorySimulator.pdb",
        "800F4B5D037A66D7B0FD6E0F16F7B2F06D626ECE58ADAA2F203DB41A757ABF84",
    ),
    (
        "lang/en.json",
        "lang/en.json",
        "2A97299D5E627339A4E88CC7214E1AE0011DCEBA9296944A89768A1F84CD433B",
    ),
    (
        "lang/pt-BR.json",
        "lang/pt-BR.json",
        "F04273F5C7E94C31C778590332949700633A51DAF8C07F81DF4931014DBB293D",
    ),
    (
        "lang/zh-Hans.json",
        "lang/zh-Hans.json",
        "C177F2C235504C6F6CA2A4C1322DFC7278A3707EB7D1EE3BAAE46A05999C20A3",
    ),
    (
        "manifest.json",
        "CS2AS05.inventory-simulator.json",
        "6A966144EEFDA8BF702A9C3688A3FC3EBEA78F04409357B31233600571A47038",
    ),
];
const GAMEDATA_HASH: &str = "5C12600DAC1BA131F0EE98723F2A842EDD048BA0EE836AC2B9D8644AC80578CB";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventorySimulatorStatus {
    selected_root: String,
    csgo_root: String,
    cs2_running: bool,
    counter_strike_sharp_installed: bool,
    counter_strike_sharp_version: Option<String>,
    legacy_player_skin_mod_present: bool,
    legacy_app_data_present: bool,
    inventory_simulator_present: bool,
    resource_version: String,
    deployed_version: Option<String>,
    upstream_tag: String,
    upstream_commit: String,
    missing_files: Vec<String>,
    hash_mismatches: Vec<String>,
    gamedata_present: bool,
    core_guideline_compatible: Option<bool>,
    service_reachable: Option<bool>,
    service_checked_at: Option<String>,
    ready: bool,
    blocked_code: Option<String>,
    blocked_message: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventorySimulatorInstallResult {
    status: InventorySimulatorStatus,
    removed_legacy_plugin: bool,
    removed_legacy_app_data: bool,
    deployed_files: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventorySimulatorRemoveResult {
    status: InventorySimulatorStatus,
    removed_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventorySimulatorServiceStatus {
    reachable: bool,
    checked_at: String,
    message: String,
}

#[tauri::command]
pub fn inventory_simulator_get_status(
    app: AppHandle,
    root_path: String,
) -> Result<InventorySimulatorStatus, String> {
    status(&app, &root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn inventory_simulator_install(
    app: AppHandle,
    root_path: String,
) -> Result<InventorySimulatorInstallResult, String> {
    install(&app, &root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn inventory_simulator_remove(
    app: AppHandle,
    root_path: String,
) -> Result<InventorySimulatorRemoveResult, String> {
    remove(&app, &root_path).map_err(AppError::into_string)
}

#[tauri::command]
pub fn inventory_simulator_open_workshop() -> Result<(), String> {
    support::open_inventory_workshop().map_err(AppError::into_string)
}

#[tauri::command]
pub async fn inventory_simulator_check_service() -> Result<InventorySimulatorServiceStatus, String>
{
    let checked_at = Utc::now().to_rfc3339();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|error| format!("[SERVICE_UNREACHABLE] 无法创建服务检查：{error}"))?;
    match client.get(SERVICE_URL).send().await {
        Ok(response) if response.status().is_success() => Ok(InventorySimulatorServiceStatus {
            reachable: true,
            checked_at,
            message: "饰品服务连接正常。".into(),
        }),
        Ok(response) => Ok(InventorySimulatorServiceStatus {
            reachable: false,
            checked_at,
            message: format!("饰品服务暂时返回 HTTP {}。", response.status().as_u16()),
        }),
        Err(error) => Ok(InventorySimulatorServiceStatus {
            reachable: false,
            checked_at,
            message: format!("饰品服务暂时无法连接：{error}"),
        }),
    }
}

fn status(app: &AppHandle, root_path: &str) -> Result<InventorySimulatorStatus, AppError> {
    let csgo = resolve_csgo_root(root_path)?;
    let plugin_parent = checked_child(&csgo, PLUGIN_PARENT_RELATIVE)?;
    let plugin = checked_child(&plugin_parent, PLUGIN_NAME)?;
    let legacy = checked_child(&plugin_parent, LEGACY_PLUGIN_NAME)?;
    let gamedata = checked_child(&csgo, GAMEDATA_RELATIVE)?;
    let app_data = legacy_app_data(app)?;
    let counter_strike_sharp_installed = csgo.join(API_RELATIVE).is_file();
    let legacy_player_skin_mod_present = legacy.exists();
    let legacy_app_data_present = app_data.exists();
    let mut missing_files = Vec::new();
    let mut hash_mismatches = Vec::new();

    for (_, deployed, expected) in PLUGIN_RESOURCES {
        inspect_file(
            &plugin.join(deployed),
            &format!("plugins/{PLUGIN_NAME}/{deployed}"),
            expected,
            &mut missing_files,
            &mut hash_mismatches,
        )?;
    }
    inspect_file(
        &gamedata,
        "gamedata/inventory-simulator.json",
        GAMEDATA_HASH,
        &mut missing_files,
        &mut hash_mismatches,
    )?;

    let gamedata_present = gamedata.is_file();
    let inventory_simulator_present = plugin.join("InventorySimulator.dll").is_file();
    let deployed_version = read_deployed_version(&plugin.join("CS2AS05.inventory-simulator.json"));
    let core_guideline_compatible = read_core_compatibility(&csgo.join(CORE_CONFIG_RELATIVE))?;
    let cs2_running = cs2::check_cs2_process()?;
    let ready = counter_strike_sharp_installed
        && !legacy_player_skin_mod_present
        && !legacy_app_data_present
        && missing_files.is_empty()
        && hash_mismatches.is_empty()
        && deployed_version.as_deref() == Some(RESOURCE_VERSION)
        && core_guideline_compatible == Some(true);

    let (blocked_code, blocked_message) = if !counter_strike_sharp_installed {
        (
            Some("COUNTERSTRIKESHARP_MISSING".into()),
            Some("还缺少基础插件环境，请先到“安装与诊断”完成安装。".into()),
        )
    } else if core_guideline_compatible != Some(true) {
        (
            Some("CORE_GUIDELINE_ENABLED".into()),
            Some("CounterStrikeSharp 尚未允许本地库存模拟，请先修复基础环境。".into()),
        )
    } else if legacy_player_skin_mod_present || legacy_app_data_present {
        (
            Some("LEGACY_PRESENT".into()),
            Some("检测到旧换肤组件，点击“一键启用”会精确移除并切换新引擎。".into()),
        )
    } else if !hash_mismatches.is_empty() {
        (
            Some("DEPLOY_VERIFY_FAILED".into()),
            Some("库存换肤文件与受管版本不一致，请重新安装。".into()),
        )
    } else if !missing_files.is_empty() {
        (
            Some("PLUGIN_MISSING".into()),
            Some("库存换肤尚未完整安装。".into()),
        )
    } else {
        (None, None)
    };

    Ok(InventorySimulatorStatus {
        selected_root: root_path.into(),
        csgo_root: csgo.display().to_string(),
        cs2_running,
        counter_strike_sharp_installed,
        counter_strike_sharp_version: counter_strike_sharp_installed
            .then(|| "API DLL 已检测（目标 1.0.371）".into()),
        legacy_player_skin_mod_present,
        legacy_app_data_present,
        inventory_simulator_present,
        resource_version: RESOURCE_VERSION.into(),
        deployed_version,
        upstream_tag: UPSTREAM_TAG.into(),
        upstream_commit: UPSTREAM_COMMIT.into(),
        missing_files,
        hash_mismatches,
        gamedata_present,
        core_guideline_compatible,
        service_reachable: None,
        service_checked_at: None,
        ready,
        blocked_code,
        blocked_message,
    })
}

fn install(app: &AppHandle, root_path: &str) -> Result<InventorySimulatorInstallResult, AppError> {
    if cs2::check_cs2_process_for_write(root_path)? {
        return Err(AppError::runtime(
            "[CS2_RUNNING] 请先完全退出 CS2，再启用库存换肤。",
        ));
    }
    let csgo = resolve_csgo_root(root_path)?;
    if !csgo.join(API_RELATIVE).is_file() {
        return Err(AppError::runtime(
            "[COUNTERSTRIKESHARP_MISSING] 还缺少基础插件环境，请先到“安装与诊断”完成安装。",
        ));
    }
    if read_core_compatibility(&csgo.join(CORE_CONFIG_RELATIVE))? != Some(true) {
        return Err(AppError::runtime(
            "[CORE_GUIDELINE_ENABLED] CounterStrikeSharp 尚未允许本地库存模拟，请先在“安装与诊断”修复基础环境。",
        ));
    }

    let source = resolve_resource_dir(app)?;
    verify_resource_dir(&source)?;
    let plugin_parent = checked_child(&csgo, PLUGIN_PARENT_RELATIVE)?;
    fs::create_dir_all(&plugin_parent).map_err(io_error)?;
    let plugin_target = checked_child(&plugin_parent, PLUGIN_NAME)?;
    let legacy_target = checked_child(&plugin_parent, LEGACY_PLUGIN_NAME)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let staging = checked_child(
        &plugin_parent,
        &format!(
            ".cs2as-inventory-simulator-staging-{}-{stamp}",
            std::process::id()
        ),
    )?;
    fs::create_dir(&staging).map_err(io_error)?;

    let stage_result = (|| {
        for (source_name, deployed_name, _) in PLUGIN_RESOURCES {
            copy_file(&source.join(source_name), &staging.join(deployed_name))?;
        }
        verify_plugin_dir(&staging)
    })();
    if let Err(error) = stage_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    let gamedata_target = checked_child(&csgo, GAMEDATA_RELATIVE)?;
    let gamedata_parent = gamedata_target
        .parent()
        .ok_or_else(|| AppError::runtime("[PATH_ESCAPE] gamedata 路径无效。"))?;
    fs::create_dir_all(gamedata_parent).map_err(io_error)?;
    let gamedata_staging = checked_child(
        gamedata_parent,
        &format!(".inventory-simulator-{stamp}.tmp"),
    )?;
    copy_file(&source.join("inventory-simulator.json"), &gamedata_staging)?;
    verify_hash(&gamedata_staging, GAMEDATA_HASH, "inventory-simulator.json")?;

    let removed_legacy_plugin = legacy_target.exists();
    if removed_legacy_plugin {
        ensure_removable_directory(&legacy_target, &plugin_parent, LEGACY_PLUGIN_NAME)?;
        fs::remove_dir_all(&legacy_target).map_err(|error| {
            AppError::runtime(format!(
                "[LEGACY_REMOVE_FAILED] 无法移除旧 PlayerSkinMod：{error}"
            ))
        })?;
    }

    if plugin_target.exists() {
        ensure_removable_directory(&plugin_target, &plugin_parent, PLUGIN_NAME)?;
        fs::remove_dir_all(&plugin_target).map_err(io_error)?;
    }
    fs::rename(&staging, &plugin_target).map_err(|error| {
        AppError::runtime(format!(
            "[DEPLOY_FAILED] 无法启用 Inventory Simulator：{error}"
        ))
    })?;
    if gamedata_target.exists() {
        ensure_regular_file(&gamedata_target)?;
        fs::remove_file(&gamedata_target).map_err(io_error)?;
    }
    fs::rename(&gamedata_staging, &gamedata_target).map_err(|error| {
        AppError::runtime(format!(
            "[DEPLOY_FAILED] 无法更新 Inventory Simulator gamedata：{error}"
        ))
    })?;

    verify_plugin_dir(&plugin_target)?;
    verify_hash(&gamedata_target, GAMEDATA_HASH, "inventory-simulator.json")?;
    let legacy_data = legacy_app_data(app)?;
    let removed_legacy_app_data = legacy_data.exists();
    if removed_legacy_app_data {
        let app_data_parent = legacy_data
            .parent()
            .ok_or_else(|| AppError::runtime("[PATH_ESCAPE] 应用数据路径无效。"))?;
        ensure_removable_directory(&legacy_data, app_data_parent, "skin-forge")?;
        fs::remove_dir_all(&legacy_data).map_err(|error| {
            AppError::runtime(format!(
                "[LEGACY_REMOVE_FAILED] 无法清理旧换肤缓存：{error}"
            ))
        })?;
    }

    let final_status = status(app, root_path)?;
    if !final_status.ready {
        return Err(AppError::runtime(
            "[DEPLOY_VERIFY_FAILED] 新插件已经写入，但最终校验没有通过，请点击重新安装。",
        ));
    }
    Ok(InventorySimulatorInstallResult {
        status: final_status,
        removed_legacy_plugin,
        removed_legacy_app_data,
        deployed_files: PLUGIN_RESOURCES
            .iter()
            .map(|(_, deployed, _)| (*deployed).to_string())
            .chain(std::iter::once("gamedata/inventory-simulator.json".into()))
            .collect(),
    })
}

fn remove(app: &AppHandle, root_path: &str) -> Result<InventorySimulatorRemoveResult, AppError> {
    if cs2::check_cs2_process_for_write(root_path)? {
        return Err(AppError::runtime(
            "[CS2_RUNNING] 请先完全退出 CS2，再移除库存换肤插件。",
        ));
    }
    let csgo = resolve_csgo_root(root_path)?;
    let plugin_parent = checked_child(&csgo, PLUGIN_PARENT_RELATIVE)?;
    let plugin = checked_child(&plugin_parent, PLUGIN_NAME)?;
    let gamedata = checked_child(&csgo, GAMEDATA_RELATIVE)?;
    let config = checked_child(&csgo, CONFIG_RELATIVE)?;
    let mut removed_paths = Vec::new();

    if plugin.exists() {
        ensure_removable_directory(&plugin, &plugin_parent, PLUGIN_NAME)?;
        fs::remove_dir_all(&plugin).map_err(|error| {
            AppError::runtime(format!(
                "[REMOVE_FAILED] 无法移除 Inventory Simulator 插件：{error}"
            ))
        })?;
        removed_paths.push(PLUGIN_PARENT_RELATIVE.to_string() + "/" + PLUGIN_NAME);
    }
    if gamedata.exists() {
        ensure_regular_file(&gamedata)?;
        fs::remove_file(&gamedata).map_err(|error| {
            AppError::runtime(format!(
                "[REMOVE_FAILED] 无法移除库存换肤 gamedata：{error}"
            ))
        })?;
        removed_paths.push(GAMEDATA_RELATIVE.into());
    }
    if config.exists() {
        let config_parent = config
            .parent()
            .ok_or_else(|| AppError::runtime("[PATH_ESCAPE] 配置路径无效。"))?;
        ensure_removable_directory(&config, config_parent, PLUGIN_NAME)?;
        fs::remove_dir_all(&config).map_err(|error| {
            AppError::runtime(format!("[REMOVE_FAILED] 无法移除库存换肤配置：{error}"))
        })?;
        removed_paths.push(CONFIG_RELATIVE.into());
    }

    let final_status = status(app, root_path)?;
    Ok(InventorySimulatorRemoveResult {
        status: final_status,
        removed_paths,
    })
}

fn resolve_csgo_root(selected: &str) -> Result<PathBuf, AppError> {
    let selected = dunce::canonicalize(selected).map_err(|error| {
        AppError::runtime(format!("[CS2_ROOT_INVALID] 无法识别 CS2 目录：{error}"))
    })?;
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
    let csgo = dunce::canonicalize(candidate).map_err(|error| {
        AppError::runtime(format!("[CS2_ROOT_INVALID] 未找到 game/csgo：{error}"))
    })?;
    if !csgo.join("gameinfo.gi").is_file() {
        return Err(AppError::runtime(
            "[CS2_ROOT_INVALID] 所选目录不是完整的 CS2 游戏目录。",
        ));
    }
    Ok(csgo)
}

fn checked_child(parent: &Path, relative: &str) -> Result<PathBuf, AppError> {
    let candidate = parent.join(relative);
    if Path::new(relative).is_absolute()
        || relative.split(['/', '\\']).any(|part| part == "..")
        || !candidate.starts_with(parent)
    {
        return Err(AppError::runtime("[PATH_ESCAPE] 目标路径越过受管目录。"));
    }
    Ok(candidate)
}

fn legacy_app_data(app: &AppHandle) -> Result<PathBuf, AppError> {
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(error.to_string()))?;
    checked_child(&root, "skin-forge")
}

fn resolve_resource_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let resource = app
        .path()
        .resource_dir()
        .map_err(|error| AppError::runtime(error.to_string()))?;
    [
        resource.join("inventory-simulator"),
        resource.join("resources/inventory-simulator"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/inventory-simulator"),
    ]
    .into_iter()
    .find(|path| path.join("InventorySimulator.dll").is_file())
    .ok_or_else(|| AppError::runtime("[RESOURCE_MISSING] 安装包缺少 Inventory Simulator 资源。"))
}

fn verify_resource_dir(source: &Path) -> Result<(), AppError> {
    for (source_name, _, expected) in PLUGIN_RESOURCES {
        verify_hash(&source.join(source_name), expected, source_name)?;
    }
    verify_hash(
        &source.join("inventory-simulator.json"),
        GAMEDATA_HASH,
        "inventory-simulator.json",
    )
}

fn verify_plugin_dir(plugin: &Path) -> Result<(), AppError> {
    for (_, deployed_name, expected) in PLUGIN_RESOURCES {
        verify_hash(&plugin.join(deployed_name), expected, deployed_name)?;
    }
    Ok(())
}

fn inspect_file(
    path: &Path,
    label: &str,
    expected: &str,
    missing: &mut Vec<String>,
    mismatches: &mut Vec<String>,
) -> Result<(), AppError> {
    if !path.is_file() {
        missing.push(label.into());
    } else if sha256_file(path)? != expected {
        mismatches.push(label.into());
    }
    Ok(())
}

fn verify_hash(path: &Path, expected: &str, label: &str) -> Result<(), AppError> {
    if !path.is_file() {
        return Err(AppError::runtime(format!(
            "[RESOURCE_MISSING] 缺少 {label}。"
        )));
    }
    if sha256_file(path)? != expected {
        return Err(AppError::runtime(format!(
            "[RESOURCE_HASH_MISMATCH] {label} 校验失败。"
        )));
    }
    Ok(())
}

fn copy_file(source: &Path, target: &Path) -> Result<(), AppError> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    fs::copy(source, target).map_err(io_error)?;
    Ok(())
}

fn read_core_compatibility(path: &Path) -> Result<Option<bool>, AppError> {
    if !path.is_file() {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_slice(&fs::read(path).map_err(io_error)?)
        .map_err(|error| {
            AppError::runtime(format!("CounterStrikeSharp core.json 无法读取：{error}"))
        })?;
    Ok(value
        .get("FollowCS2ServerGuidelines")
        .and_then(serde_json::Value::as_bool)
        .map(|enabled| !enabled))
}

fn read_deployed_version(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    value.get("resourceVersion")?.as_str().map(str::to_string)
}

fn ensure_removable_directory(
    target: &Path,
    expected_parent: &Path,
    expected_name: &str,
) -> Result<(), AppError> {
    if target.parent() != Some(expected_parent)
        || !target
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case(expected_name))
    {
        return Err(AppError::runtime("[PATH_ESCAPE] 拒绝清理非白名单目录。"));
    }
    let metadata = fs::symlink_metadata(target).map_err(io_error)?;
    if !metadata.is_dir() || is_reparse_or_symlink(&metadata) {
        return Err(AppError::runtime(
            "[PATH_ESCAPE] 拒绝清理链接或重解析目录。",
        ));
    }
    Ok(())
}

fn ensure_regular_file(target: &Path) -> Result<(), AppError> {
    let metadata = fs::symlink_metadata(target).map_err(io_error)?;
    if !metadata.is_file() || is_reparse_or_symlink(&metadata) {
        return Err(AppError::runtime("[PATH_ESCAPE] 拒绝覆盖链接文件。"));
    }
    Ok(())
}

#[cfg(windows)]
fn is_reparse_or_symlink(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn is_reparse_or_symlink(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn sha256_file(path: &Path) -> Result<String, AppError> {
    let bytes = fs::read(path).map_err(io_error)?;
    Ok(format!("{:X}", Sha256::digest(bytes)))
}

fn io_error(error: std::io::Error) -> AppError {
    AppError::runtime(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        checked_child, ensure_removable_directory, read_core_compatibility, verify_resource_dir,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("cs2as-inventory-{label}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn bundled_inventory_simulator_resources_match_manifest() {
        let source =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/inventory-simulator");
        verify_resource_dir(&source).unwrap();
    }

    #[test]
    fn child_path_rejects_parent_escape() {
        let root = PathBuf::from("C:/safe/root");
        assert!(checked_child(&root, "plugins/InventorySimulator").is_ok());
        assert!(checked_child(&root, "../other-project").is_err());
    }

    #[test]
    fn removal_requires_exact_whitelisted_directory() {
        let root = temp_dir("boundary");
        let plugins = root.join("plugins");
        let legacy = plugins.join("PlayerSkinMod");
        let other = plugins.join("NadeSystem");
        fs::create_dir_all(&legacy).unwrap();
        fs::create_dir_all(&other).unwrap();
        assert!(ensure_removable_directory(&legacy, &plugins, "PlayerSkinMod").is_ok());
        assert!(ensure_removable_directory(&other, &plugins, "PlayerSkinMod").is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn core_guideline_check_preserves_unknown_fields() {
        let root = temp_dir("core");
        let config = root.join("core.json");
        fs::write(
            &config,
            br#"{"FollowCS2ServerGuidelines":false,"UnrelatedSetting":{"keep":true}}"#,
        )
        .unwrap();
        assert_eq!(read_core_compatibility(&config).unwrap(), Some(true));
        let after = fs::read_to_string(&config).unwrap();
        assert!(after.contains("UnrelatedSetting"));
        fs::remove_dir_all(root).unwrap();
    }
}
