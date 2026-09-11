use crate::errors::AppError;
use crate::services::cs2;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const RELATIVE: &str = "addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapRotationDefault {
    pub root_path: String,
    pub config_path: String,
    pub enabled: bool,
    pub source: String,
    pub writable: bool,
    pub warning: Option<String>,
    pub updated_at: Option<String>,
    pub config_sha256: Option<String>,
    pub read_back_enabled: bool,
    pub observed_at: String,
    pub load_semantics: &'static str,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    enabled: bool,
}

fn config_path(root_path: &str) -> Result<(PathBuf, PathBuf), AppError> {
    let root = cs2::normalize_root(root_path)?;
    let csgo = root.join("game/csgo");
    Ok((root, csgo.join(RELATIVE)))
}

fn read_at(root: &Path, path: &Path, writable: bool) -> Result<MapRotationDefault, AppError> {
    let observed_at = Utc::now().to_rfc3339();
    let (enabled, source, warning, updated_at, config_sha256) = match fs::read(path) {
        Ok(bytes) => match serde_json::from_slice::<serde_json::Value>(&bytes) {
            Ok(value)
                if value.is_object() && value.get("enabled").map_or(true, |v| v.is_boolean()) =>
            {
                (
                    value
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                    "existing",
                    None,
                    fs::metadata(path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(DateTime::<Utc>::from)
                        .map(|d| d.to_rfc3339()),
                    Some(format!("{:X}", Sha256::digest(&bytes))),
                )
            }
            _ => (
                true,
                "fallback",
                Some("配置损坏，已回退开启；点击“恢复默认”后才会覆盖。".into()),
                None,
                None,
            ),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (
            true,
            "default",
            Some("尚未建立，当前默认开启。".into()),
            None,
            None,
        ),
        Err(error) => {
            return Err(AppError::runtime(format!(
                "[MAP_ROTATION_READ_FAILED] 读取配置失败：{error}"
            )))
        }
    };
    let read_back_enabled = config_sha256.is_some();
    Ok(MapRotationDefault {
        root_path: root.display().to_string(),
        config_path: path.display().to_string(),
        enabled,
        source: source.into(),
        writable,
        warning,
        updated_at,
        config_sha256,
        read_back_enabled,
        observed_at,
        load_semantics: "next-plugin-load",
    })
}

pub fn get(root_path: &str) -> Result<MapRotationDefault, AppError> {
    let (root, path) = config_path(root_path)?;
    read_at(&root, &path, !cs2::check_cs2_process()?)
}

fn write(root_path: &str, enabled: bool, reset: bool) -> Result<MapRotationDefault, AppError> {
    if cs2::check_cs2_process_for_write(root_path)? {
        return Err(AppError::runtime(
            "[MAP_ROTATION_CS2_RUNNING] 请先退出 CS2，再修改下一次载入默认值。",
        ));
    }
    let (root, path) = config_path(root_path)?;
    if !reset {
        let _ = read_at(&root, &path, true)?;
    }
    let bytes = serde_json::to_vec_pretty(&Config { enabled })
        .map_err(|e| AppError::runtime(format!("[MAP_ROTATION_WRITE_FAILED] 序列化失败：{e}")))?;
    let temp = path.with_extension("json.tmp");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::runtime(format!("[MAP_ROTATION_WRITE_FAILED] 创建目录失败：{e}"))
        })?;
    }
    fs::write(&temp, &bytes)
        .map_err(|e| AppError::runtime(format!("[MAP_ROTATION_WRITE_FAILED] 写入失败：{e}")))?;
    fs::rename(&temp, &path)
        .map_err(|e| AppError::runtime(format!("[MAP_ROTATION_WRITE_FAILED] 原子替换失败：{e}")))?;
    let result = read_at(&root, &path, true)?;
    cs2::write_runtime_log(
        "INFO",
        &format!(
            "[MAP_ROTATION_DEFAULT_UPDATED] enabled={} path={} sha256={} readBackEnabled={} loadSemantics=next-plugin-load",
            enabled,
            path.display(),
            result.config_sha256.as_deref().unwrap_or("none"),
            result.read_back_enabled,
        ),
    );
    Ok(result)
}

pub fn set(root_path: &str, enabled: bool) -> Result<MapRotationDefault, AppError> {
    write(root_path, enabled, false)
}
pub fn reset(root_path: &str) -> Result<MapRotationDefault, AppError> {
    write(root_path, true, true)
}
