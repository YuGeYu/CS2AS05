use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use zip::ZipArchive;

use crate::errors::AppError;
use crate::models::cs2::OperationResult;
use crate::models::demotracer::{DemoTracerInstallResult, DemoTracerStatus};
use crate::services::cs2;

const GUI_FILE: &str = "demotracer-gui-v1.5.3.exe";
const PLAYBACK_FILE: &str = "demotracer-css-v1.5.2.zip";
const GUI_SIZE: u64 = 8_282_721;
const GUI_SHA256: &str = "C1C8C23EEEEDAD97D9E7C00A7D68473817AEEF468D7EFEF5941E24595EF13402";
const PLAYBACK_SIZE: u64 = 2_926_064;
const PLAYBACK_SHA256: &str = "E19C420661ABE62CC3684C217BA93E0858BEBB7CC671F0CAB7ED3520CAFB909D";
const GUI_VERSION: &str = "1.5.3";
const PLAYBACK_VERSION: &str = "1.5.2";
const LEDGER_RELATIVE: &str = "cfg/cs2as05-demotracer-install-ledger.json";
const BACKUP_RELATIVE: &str = "cfg/cs2as05-demotracer-install-backups";
const LEDGER_SCHEMA: u32 = 1;

const REQUIRED_FILES: &[&str] = &[
    "addons/counterstrikesharp/plugins/DemoTracer/DemoTracer.dll",
    "addons/counterstrikesharp/plugins/DemoTracer/DemoTracer.deps.json",
    "addons/counterstrikesharp/plugins/DemoTracer/ZstdSharp.dll",
    "addons/counterstrikesharp/plugins/DtrHider/DtrHider.dll",
    "addons/counterstrikesharp/shared/DemoTracerApi/DemoTracerApi.dll",
    "addons/counterstrikesharp/shared/DtrHiderApi/DtrHiderApi.dll",
    "addons/counterstrikesharp/plugins/BotRandomizer/BotRandomizer.dll",
    "addons/counterstrikesharp/shared/BotRandomizerApi/BotRandomizerApi.dll",
    "addons/dtr-controller/bin/win64/dtr-controller.dll",
    "addons/dtr-controller/gamedata.json",
    "addons/dtr-hider/bin/win64/dtr-hider.dll",
    "addons/dtr-hider/gamedata.json",
    "addons/metamod/dtr-controller.vdf",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallLedger {
    schema: u32,
    entries: Vec<LedgerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LedgerEntry {
    relative: String,
    existed_before: bool,
    original_backup_relative: Option<String>,
    original_sha256: Option<String>,
    installed_sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PlaybackManifest {
    schema_version: u32,
    bundle_version: String,
    platform: String,
    files: Vec<ManifestFile>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestFile {
    path: String,
    size: u64,
    sha256: String,
}

#[derive(Debug, Clone)]
struct PackageFile {
    relative: PathBuf,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct AppliedFile {
    relative: PathBuf,
    previous_backup: Option<PathBuf>,
}

pub fn get_status(app: &AppHandle, root_path: Option<&str>) -> Result<DemoTracerStatus, AppError> {
    let resource = resolve_resource_dir(app);
    let gui_path = resource.as_ref().ok().map(|path| path.join(GUI_FILE));
    let playback_path = resource.as_ref().ok().map(|path| path.join(PLAYBACK_FILE));
    let gui_resource_ready = gui_path
        .as_deref()
        .is_some_and(|path| verify_file(path, GUI_SIZE, GUI_SHA256).is_ok());
    let playback_resource_ready = playback_path
        .as_deref()
        .is_some_and(|path| verify_playback_package(path).is_ok());
    let cs2_running = cs2::check_cs2_process()?;

    let (selected_root, csgo, root_error) = match root_path {
        Some(value) => match cs2::normalize_root(value) {
            Ok(root) => {
                let csgo = root.join("game").join("csgo");
                (Some(root.display().to_string()), Some(csgo), None)
            }
            Err(error) => (Some(value.to_string()), None, Some(error.into_string())),
        },
        None => (None, None, None),
    };

    let mut installed_file_count = 0;
    let mut missing_files = Vec::new();
    let mut hash_mismatches = Vec::new();
    let playback_installed = if let Some(csgo) = csgo.as_deref() {
        let counter_strike_sharp_ready = counter_strike_sharp_ready(csgo);
        let _ = counter_strike_sharp_ready;
        match load_ledger(csgo) {
            Ok(Some(ledger)) if !ledger.entries.is_empty() => {
                let mut all_owned = true;
                for entry in &ledger.entries {
                    installed_file_count += 1;
                    let relative = safe_relative(&entry.relative)?;
                    let target = csgo.join(&relative);
                    if ensure_safe_target(&csgo, &target).is_err() || !is_regular_file(&target) {
                        missing_files.push(entry.relative.clone());
                        all_owned = false;
                    } else if sha256_file(&target)? != entry.installed_sha256 {
                        hash_mismatches.push(entry.relative.clone());
                        all_owned = false;
                    }
                }
                all_owned
            }
            Ok(_) => false,
            Err(error) => {
                hash_mismatches.push(error.into_string());
                false
            }
        }
    } else {
        false
    };

    let (counter_strike_sharp_ready, metamod_ready) = csgo
        .as_deref()
        .map(|path| {
            (
                counter_strike_sharp_ready(path),
                path.join("addons/metamod").is_dir(),
            )
        })
        .unwrap_or((false, false));

    let resource_ready = gui_resource_ready && playback_resource_ready;
    let ready = resource_ready
        && playback_installed
        && counter_strike_sharp_ready
        && metamod_ready
        && !cs2_running;
    let (blocked_code, blocked_message) = if !gui_resource_ready {
        (
            Some("DEMOTRACER_GUI_MISSING".into()),
            Some("安装包缺少 DemoTracer GUI 1.5.3。请重新安装助手。".into()),
        )
    } else if !playback_resource_ready {
        (
            Some("DEMOTRACER_PLAYBACK_RESOURCE_INVALID".into()),
            Some("DemoTracer Playback 1.5.2 资源校验失败，请重新安装助手。".into()),
        )
    } else if let Some(message) = root_error {
        (Some("CS2_ROOT_INVALID".into()), Some(message))
    } else if csgo.is_none() {
        (
            Some("CS2_ROOT_MISSING".into()),
            Some("先选择 CS2 游戏目录，再安装 DemoTracer Playback。".into()),
        )
    } else if cs2_running {
        (
            Some("CS2_RUNNING".into()),
            Some("请先完全退出 CS2，再安装或启动 Demo 重玩。".into()),
        )
    } else if !metamod_ready || !counter_strike_sharp_ready {
        (
            Some("PLAYBACK_HOST_MISSING".into()),
            Some("Playback 需要已安装的 Metamod 和 CounterStrikeSharp 主机。请先安装人机增强核心资源。".into()),
        )
    } else if !missing_files.is_empty() || !hash_mismatches.is_empty() {
        (
            Some("PLAYBACK_INSTALL_DRIFTED".into()),
            Some("Playback 文件与受管版本不一致，可以重新安装修复。".into()),
        )
    } else if !playback_installed {
        (
            Some("PLAYBACK_NOT_INSTALLED".into()),
            Some("Playback 组件尚未安装到当前 CS2 目录。".into()),
        )
    } else {
        (None, None)
    };

    Ok(DemoTracerStatus {
        resource_version: GUI_VERSION.into(),
        playback_version: PLAYBACK_VERSION.into(),
        gui_path: gui_path.map(|path| path.display().to_string()),
        playback_package_path: playback_path.map(|path| path.display().to_string()),
        gui_resource_ready,
        playback_resource_ready,
        playback_installed,
        installed_file_count,
        missing_files,
        hash_mismatches,
        counter_strike_sharp_ready,
        metamod_ready,
        cs2_running,
        selected_root,
        ready,
        blocked_code,
        blocked_message,
    })
}

pub fn install_playback(
    app: &AppHandle,
    root_path: &str,
) -> Result<DemoTracerInstallResult, AppError> {
    if cs2::check_cs2_process_for_write(root_path)? {
        return Err(AppError::runtime(
            "[CS2_RUNNING] 请先完全退出 CS2，再安装 DemoTracer Playback。",
        ));
    }
    let root = cs2::normalize_root(root_path)?;
    let csgo = root.join("game").join("csgo");
    if !csgo.is_dir() {
        return Err(AppError::runtime(
            "[CS2_ROOT_INVALID] 所选目录缺少 game\\csgo，请重新选择 CS2 根目录。",
        ));
    }
    if !counter_strike_sharp_ready(&csgo) || !csgo.join("addons/metamod").is_dir() {
        return Err(AppError::runtime(
            "[PLAYBACK_HOST_MISSING] DemoTracer 只安装 Playback 组件；请先安装 Metamod 与 CounterStrikeSharp 主机。",
        ));
    }
    let package_path = resolve_resource_dir(app)?.join(PLAYBACK_FILE);
    let package_files = verify_playback_package(&package_path)?;
    let old_ledger = load_ledger(&csgo)?;
    validate_ledger_backups(&csgo, old_ledger.as_ref())?;
    let stamp = unique_stamp();
    let transaction_backup = csgo
        .join(BACKUP_RELATIVE)
        .join(format!(".transaction-{stamp}"));
    fs::create_dir_all(&transaction_backup).map_err(io_error)?;
    let mut applied = Vec::new();

    let write_result = (|| {
        for package_file in &package_files {
            let relative = safe_relative(&package_file.relative.to_string_lossy())?;
            let target = checked_child(&csgo, &relative)?;
            ensure_safe_target(&csgo, &target)?;
            let previous_backup = if target.exists() {
                if !is_regular_file(&target) {
                    return Err(AppError::runtime(format!(
                        "[PATH_ESCAPE] 拒绝覆盖非普通文件：{}",
                        target.display()
                    )));
                }
                let backup = transaction_backup.join(&relative);
                if let Some(parent) = backup.parent() {
                    fs::create_dir_all(parent).map_err(io_error)?;
                }
                fs::copy(&target, &backup).map_err(io_error)?;
                Some(backup)
            } else {
                None
            };
            let temporary = target.with_file_name(format!(
                ".{}.demotracer-{}.tmp",
                target
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("file"),
                std::process::id()
            ));
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(io_error)?;
            }
            applied.push(AppliedFile {
                relative: relative.clone(),
                previous_backup,
            });
            let replace_result = (|| {
                fs::write(&temporary, &package_file.bytes).map_err(io_error)?;
                if target.exists() {
                    fs::remove_file(&target).map_err(io_error)?;
                }
                fs::rename(&temporary, &target).map_err(io_error)
            })();
            if let Err(error) = replace_result {
                let _ = fs::remove_file(&temporary);
                return Err(error);
            }
        }
        Ok::<(), AppError>(())
    })();

    if let Err(error) = write_result {
        rollback_transaction(&csgo, &applied)?;
        let _ = fs::remove_dir_all(&transaction_backup);
        return Err(AppError::runtime(format!(
            "[PLAYBACK_INSTALL_ROLLBACK] Playback 安装未完成，已恢复写入前状态：{}",
            error.into_string()
        )));
    }

    let persistent_backup = csgo.join(BACKUP_RELATIVE);
    let mut ledger = old_ledger.unwrap_or(InstallLedger {
        schema: LEDGER_SCHEMA,
        entries: Vec::new(),
    });
    let mut created_persistent_backups = Vec::new();
    let ledger_result = (|| -> Result<Option<PathBuf>, AppError> {
        let mut backup_path = None;
        for package_file in &package_files {
            let relative = safe_relative(&package_file.relative.to_string_lossy())?;
            let relative_string = relative.to_string_lossy().replace('\\', "/");
            let target = csgo.join(&relative);
            let installed_sha256 = sha256_file(&target)?;
            if let Some(entry) = ledger
                .entries
                .iter_mut()
                .find(|entry| entry.relative == relative_string)
            {
                entry.installed_sha256 = installed_sha256;
                continue;
            }
            let current_backup = transaction_backup.join(&relative);
            let existed_before = current_backup.is_file();
            let original_backup_relative = existed_before.then(|| relative_string.clone());
            let original_sha256 = if existed_before {
                let destination = persistent_backup.join(&relative);
                ensure_safe_target(&csgo, &destination)?;
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent).map_err(io_error)?;
                }
                if destination.is_file() {
                    let expected = sha256_file(&current_backup)?;
                    if sha256_file(&destination)? != expected {
                        return Err(AppError::runtime(format!(
                            "[PLAYBACK_LEDGER_INVALID] 原始备份已存在但摘要不一致：{}",
                            destination.display()
                        )));
                    }
                } else {
                    fs::copy(&current_backup, &destination).map_err(io_error)?;
                    created_persistent_backups.push(destination.clone());
                    backup_path = Some(persistent_backup.clone());
                }
                Some(sha256_file(&destination)?)
            } else {
                None
            };
            ledger.entries.push(LedgerEntry {
                relative: relative_string,
                existed_before,
                original_backup_relative,
                original_sha256,
                installed_sha256,
            });
        }
        ledger
            .entries
            .sort_by(|left, right| left.relative.cmp(&right.relative));
        write_ledger(&csgo, &ledger)?;
        Ok(backup_path)
    })();
    let backup_path = match ledger_result {
        Ok(path) => path,
        Err(error) => {
            let rollback_error = rollback_transaction(&csgo, &applied).err();
            for path in &created_persistent_backups {
                let _ = fs::remove_file(path);
            }
            let _ = fs::remove_dir_all(&transaction_backup);
            if let Some(rollback_error) = rollback_error {
                return Err(AppError::runtime(format!(
                    "[PLAYBACK_INSTALL_ROLLBACK] Playback 安装失败，且恢复过程也失败：{}；{}",
                    error.into_string(),
                    rollback_error.into_string()
                )));
            }
            return Err(AppError::runtime(format!(
                "[PLAYBACK_INSTALL_ROLLBACK] Playback 安装未完成，已恢复写入前状态：{}",
                error.into_string()
            )));
        }
    };
    let _ = fs::remove_dir_all(&transaction_backup);

    let status = get_status(app, Some(root_path))?;
    if !status.playback_installed {
        return Err(AppError::runtime(
            "[PLAYBACK_INSTALL_VERIFY_FAILED] Playback 已写入，但最终 ownership 校验未通过。",
        ));
    }
    Ok(DemoTracerInstallResult {
        status,
        installed_files: package_files
            .into_iter()
            .map(|file| file.relative.to_string_lossy().replace('\\', "/"))
            .collect(),
        backup_path: backup_path.map(|path| path.display().to_string()),
    })
}

pub fn uninstall_playback(root_path: &str) -> Result<OperationResult, AppError> {
    if cs2::check_cs2_process_for_write(root_path)? {
        return Err(AppError::runtime(
            "[CS2_RUNNING] 请先完全退出 CS2，再卸载 DemoTracer Playback。",
        ));
    }
    let root = cs2::normalize_root(root_path)?;
    let csgo = root.join("game").join("csgo");
    let Some(ledger) = load_ledger(&csgo)? else {
        return Ok(OperationResult {
            success: true,
            message: "当前 CS2 目录没有 DemoTracer ownership 清单，未删除任何文件。".into(),
        });
    };
    validate_ledger_backups(&csgo, Some(&ledger))?;
    let persistent_backup = csgo.join(BACKUP_RELATIVE);
    let transaction_backup = persistent_backup.join(format!(".uninstall-{}", unique_stamp()));
    fs::create_dir_all(&transaction_backup).map_err(io_error)?;
    let mut applied = Vec::new();
    let mut remaining = Vec::new();
    let mut restored = 0usize;
    let mut skipped = 0usize;
    let uninstall_result = (|| -> Result<(), AppError> {
        for entry in ledger.entries {
            let relative = safe_relative(&entry.relative)?;
            let target = checked_child(&csgo, &relative)?;
            let safe_target = ensure_safe_target(&csgo, &target).is_ok();
            let current_owned = safe_target
                && is_regular_file(&target)
                && sha256_file(&target)? == entry.installed_sha256;
            if entry.existed_before {
                let Some(original_relative) = entry.original_backup_relative.as_deref() else {
                    return Err(AppError::runtime(
                        "[PLAYBACK_LEDGER_INVALID] 原始备份路径缺失。",
                    ));
                };
                let original = persistent_backup.join(safe_relative(original_relative)?);
                if !is_regular_file(&original) {
                    return Err(AppError::runtime(format!(
                        "[PLAYBACK_LEDGER_INVALID] 原始文件备份缺失：{}",
                        original.display()
                    )));
                }
                if let Some(expected) = entry.original_sha256.as_deref() {
                    if sha256_file(&original)? != expected {
                        return Err(AppError::runtime(
                            "[PLAYBACK_LEDGER_INVALID] 原始文件备份校验失败，已停止卸载。",
                        ));
                    }
                }
                if !current_owned {
                    skipped += 1;
                    remaining.push(entry);
                    continue;
                }
                ensure_safe_target(&csgo, &target)?;
                let current_backup = transaction_backup.join(&relative);
                if let Some(parent) = current_backup.parent() {
                    fs::create_dir_all(parent).map_err(io_error)?;
                }
                fs::copy(&target, &current_backup).map_err(io_error)?;
                applied.push(AppliedFile {
                    relative: relative.clone(),
                    previous_backup: Some(current_backup),
                });
                replace_file_from_source(&csgo, &target, &original, "uninstall")?;
                restored += 1;
            } else if current_owned {
                ensure_safe_target(&csgo, &target)?;
                let current_backup = transaction_backup.join(&relative);
                if let Some(parent) = current_backup.parent() {
                    fs::create_dir_all(parent).map_err(io_error)?;
                }
                fs::copy(&target, &current_backup).map_err(io_error)?;
                applied.push(AppliedFile {
                    relative: relative.clone(),
                    previous_backup: Some(current_backup),
                });
                fs::remove_file(&target).map_err(io_error)?;
                restored += 1;
            } else if target.exists() {
                skipped += 1;
                remaining.push(entry);
            } else {
                restored += 1;
            }
        }
        if remaining.is_empty() {
            fs::remove_file(ledger_path(&csgo)).map_err(io_error)?;
        } else {
            write_ledger(
                &csgo,
                &InstallLedger {
                    schema: LEDGER_SCHEMA,
                    entries: remaining.clone(),
                },
            )?;
        }
        Ok(())
    })();
    if let Err(error) = uninstall_result {
        let rollback_error = rollback_transaction(&csgo, &applied).err();
        let _ = fs::remove_dir_all(&transaction_backup);
        if let Some(rollback_error) = rollback_error {
            return Err(AppError::runtime(format!(
                "[PLAYBACK_UNINSTALL_ROLLBACK] 卸载失败，且恢复过程也失败：{}；{}",
                error.into_string(),
                rollback_error.into_string()
            )));
        }
        return Err(AppError::runtime(format!(
            "[PLAYBACK_UNINSTALL_ROLLBACK] 卸载未完成，已恢复写入前状态：{}",
            error.into_string()
        )));
    }
    let _ = fs::remove_dir_all(&transaction_backup);
    if remaining.is_empty() {
        let _ = fs::remove_dir_all(&persistent_backup);
    }
    Ok(OperationResult {
        success: true,
        message: format!(
            "DemoTracer Playback 已安全处理：恢复或移除 {} 项，保留 {} 项被玩家或其他插件改动的文件。Steam 官方文件不会按文件名删除。",
            restored, skipped
        ),
    })
}

pub fn open_gui(app: &AppHandle) -> Result<OperationResult, AppError> {
    let resource = resolve_resource_dir(app)?;
    let source = resource.join(GUI_FILE);
    verify_file(&source, GUI_SIZE, GUI_SHA256)?;
    let tool_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法确定 DemoTracer 数据目录：{error}")))?
        .join("tools")
        .join(format!("demotracer-v{GUI_VERSION}"));
    fs::create_dir_all(&tool_dir).map_err(io_error)?;
    let target = tool_dir.join(GUI_FILE);
    if !target.is_file() || verify_file(&target, GUI_SIZE, GUI_SHA256).is_err() {
        let temporary = tool_dir.join(format!(".{GUI_FILE}.{}.tmp", std::process::id()));
        fs::copy(&source, &temporary).map_err(io_error)?;
        verify_file(&temporary, GUI_SIZE, GUI_SHA256)?;
        if target.exists() {
            fs::remove_file(&target).map_err(io_error)?;
        }
        fs::rename(&temporary, &target).map_err(io_error)?;
    }
    Command::new(&target)
        .current_dir(&tool_dir)
        .spawn()
        .map_err(|error| {
            AppError::runtime(format!(
                "[DEMOTRACER_GUI_LAUNCH] 无法启动 DemoTracer：{error}"
            ))
        })?;
    Ok(OperationResult {
        success: true,
        message: format!("已启动 DemoTracer GUI v{GUI_VERSION}。它会在独立窗口中完成 Demo 分析、回合导出和重玩命令生成。"),
    })
}

fn resolve_resource_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let resource = app
        .path()
        .resource_dir()
        .map_err(|error| AppError::runtime(format!("无法定位应用资源目录：{error}")))?;
    [
        resource.join("demotracer"),
        resource.join("resources/demotracer"),
        resource.join("../demotracer"),
        resource.join("../resources/demotracer"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/demotracer"),
    ]
    .into_iter()
    .find(|path| path.join(GUI_FILE).is_file() || path.join(PLAYBACK_FILE).is_file())
    .ok_or_else(|| {
        AppError::runtime("[DEMOTRACER_RESOURCE_MISSING] 安装包缺少 DemoTracer 官方资源。")
    })
}

fn verify_playback_package(path: &Path) -> Result<Vec<PackageFile>, AppError> {
    verify_file(path, PLAYBACK_SIZE, PLAYBACK_SHA256).map_err(|error| {
        AppError::runtime(format!(
            "[DEMOTRACER_RESOURCE_HASH_MISMATCH] Playback 1.5.2 资源校验失败：{}",
            error.into_string()
        ))
    })?;
    let file = File::open(path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| AppError::runtime(format!("Playback ZIP 无法读取：{error}")))?;
    let mut entries = BTreeMap::<String, usize>::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(zip_error)?;
        let Some(relative) = package_relative(entry.name())? else {
            continue;
        };
        if entry.is_dir() {
            continue;
        }
        if entries
            .insert(relative.to_string_lossy().replace('\\', "/"), index)
            .is_some()
        {
            return Err(AppError::runtime(
                "[PLAYBACK_PACKAGE_INVALID] Playback ZIP 存在重复文件路径。",
            ));
        }
    }
    let manifest_path = Path::new("addons/demotracer-install.v1.json");
    let manifest_index = entries
        .get(&manifest_path.to_string_lossy().replace('\\', "/"))
        .copied()
        .ok_or_else(|| {
            AppError::runtime("[PLAYBACK_PACKAGE_INVALID] 缺少 DemoTracer 安装清单。")
        })?;
    let mut manifest_entry = archive.by_index(manifest_index).map_err(zip_error)?;
    let mut manifest_bytes = Vec::new();
    manifest_entry
        .read_to_end(&mut manifest_bytes)
        .map_err(io_error)?;
    drop(manifest_entry);
    let manifest: PlaybackManifest = serde_json::from_slice(&manifest_bytes).map_err(|error| {
        AppError::runtime(format!("[PLAYBACK_PACKAGE_INVALID] 安装清单无效：{error}"))
    })?;
    if manifest.bundle_version != PLAYBACK_VERSION {
        return Err(AppError::runtime(format!(
            "[PLAYBACK_PACKAGE_VERSION] 需要 Playback {PLAYBACK_VERSION}，实际为 {}。",
            manifest.bundle_version
        )));
    }
    if manifest.schema_version != 1 || manifest.platform != "windows-x64" {
        return Err(AppError::runtime(
            "[PLAYBACK_PACKAGE_VERSION] 仅支持 Windows x64 的 DemoTracer Playback 清单 v1。",
        ));
    }
    let mut package_files = Vec::new();
    let mut listed_paths = BTreeSet::new();
    for listed in &manifest.files {
        let relative = safe_relative(&listed.path)?;
        let key = relative.to_string_lossy().replace('\\', "/");
        if !listed_paths.insert(key.clone()) {
            return Err(AppError::runtime(format!(
                "[PLAYBACK_PACKAGE_INVALID] 安装清单存在重复文件路径：{key}"
            )));
        }
        let Some(index) = entries.get(&key).copied() else {
            return Err(AppError::runtime(format!(
                "[PLAYBACK_PACKAGE_INVALID] 清单文件缺失：{key}"
            )));
        };
        let mut entry = archive.by_index(index).map_err(zip_error)?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(io_error)?;
        if bytes.len() as u64 != listed.size
            || !sha256_bytes(&bytes).eq_ignore_ascii_case(&listed.sha256)
        {
            return Err(AppError::runtime(format!(
                "[PLAYBACK_PACKAGE_INVALID] 文件校验失败：{key}"
            )));
        }
        package_files.push(PackageFile { relative, bytes });
    }
    let actual: BTreeSet<String> = package_files
        .iter()
        .map(|file| file.relative.to_string_lossy().replace('\\', "/"))
        .collect();
    for required in REQUIRED_FILES {
        if !actual.contains(*required) {
            return Err(AppError::runtime(format!(
                "[PLAYBACK_PACKAGE_INVALID] 缺少必需组件：{required}"
            )));
        }
    }
    Ok(package_files)
}

fn package_relative(name: &str) -> Result<Option<PathBuf>, AppError> {
    let path = safe_relative(name)?;
    let mut components = path.components();
    let Some(first) = components.next() else {
        return Ok(None);
    };
    let first = first.as_os_str().to_string_lossy();
    if first == "addons" {
        return Ok(Some(path));
    }
    let remainder = components.collect::<PathBuf>();
    if remainder.starts_with("addons") {
        return Ok(Some(remainder));
    }
    Ok(None)
}

fn safe_relative(value: &str) -> Result<PathBuf, AppError> {
    let normalized = value.replace('\\', "/");
    let path = Path::new(&normalized);
    if path.is_absolute()
        || normalized
            .split('/')
            .next()
            .is_some_and(|segment| segment.contains(':'))
    {
        return Err(AppError::runtime(format!(
            "[PATH_ESCAPE] 拒绝绝对路径：{value}"
        )));
    }
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => output.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(AppError::runtime(format!(
                    "[PATH_ESCAPE] 拒绝越界路径：{value}"
                )))
            }
        }
    }
    if output.as_os_str().is_empty() {
        return Err(AppError::runtime("[PATH_ESCAPE] 拒绝空路径。"));
    }
    Ok(output)
}

fn checked_child(parent: &Path, relative: &Path) -> Result<PathBuf, AppError> {
    let candidate = parent.join(relative);
    if !candidate.starts_with(parent) {
        return Err(AppError::runtime("[PATH_ESCAPE] 目标路径越过受管目录。"));
    }
    Ok(candidate)
}

fn ensure_safe_target(root: &Path, target: &Path) -> Result<(), AppError> {
    let mut current = target.to_path_buf();
    if target.exists() {
        let metadata = fs::symlink_metadata(target).map_err(io_error)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(AppError::runtime(format!(
                "[PATH_ESCAPE] 拒绝覆盖非普通文件：{}",
                target.display()
            )));
        }
    }
    while current.starts_with(root) {
        if current.exists() {
            let metadata = fs::symlink_metadata(&current).map_err(io_error)?;
            if metadata.file_type().is_symlink() {
                return Err(AppError::runtime(format!(
                    "[PATH_ESCAPE] 拒绝写入链接路径：{}",
                    current.display()
                )));
            }
        }
        if current == *root {
            break;
        }
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent.to_path_buf();
    }
    Ok(())
}

fn counter_strike_sharp_ready(csgo: &Path) -> bool {
    csgo.join("addons/counterstrikesharp/api/CounterStrikeSharp.API.dll")
        .is_file()
        && csgo
            .join("addons/counterstrikesharp/bin/win64/counterstrikesharp.dll")
            .is_file()
}

fn ledger_path(csgo: &Path) -> PathBuf {
    csgo.join(LEDGER_RELATIVE)
}

fn load_ledger(csgo: &Path) -> Result<Option<InstallLedger>, AppError> {
    let path = ledger_path(csgo);
    if !path.is_file() {
        return Ok(None);
    }
    let ledger: InstallLedger = serde_json::from_slice(&fs::read(&path).map_err(io_error)?)
        .map_err(|error| {
            AppError::runtime(format!(
                "[PLAYBACK_LEDGER_INVALID] 无法读取 ownership 清单：{error}"
            ))
        })?;
    if ledger.schema != LEDGER_SCHEMA {
        return Err(AppError::runtime(
            "[PLAYBACK_LEDGER_INVALID] 不支持的 ownership 清单版本。",
        ));
    }
    let mut seen = BTreeSet::new();
    for entry in &ledger.entries {
        let relative = safe_relative(&entry.relative)?;
        if !is_playback_relative(&relative)
            || !seen.insert(relative.to_string_lossy().replace('\\', "/"))
            || entry.installed_sha256.is_empty()
        {
            return Err(AppError::runtime(
                "[PLAYBACK_LEDGER_INVALID] ownership 清单存在重复或不完整条目。",
            ));
        }
        if let Some(original_relative) = entry.original_backup_relative.as_deref() {
            let original = safe_relative(original_relative)?;
            if !is_playback_relative(&original) {
                return Err(AppError::runtime(
                    "[PLAYBACK_LEDGER_INVALID] 原始备份路径不在 addons 白名单内。",
                ));
            }
        }
        if entry.existed_before
            && (entry.original_backup_relative.is_none() || entry.original_sha256.is_none())
        {
            return Err(AppError::runtime(
                "[PLAYBACK_LEDGER_INVALID] 原始备份信息不完整。",
            ));
        }
    }
    Ok(Some(ledger))
}

fn validate_ledger_backups(csgo: &Path, ledger: Option<&InstallLedger>) -> Result<(), AppError> {
    let Some(ledger) = ledger else { return Ok(()) };
    let persistent_backup = csgo.join(BACKUP_RELATIVE);
    for entry in &ledger.entries {
        if !entry.existed_before {
            continue;
        }
        let Some(original_relative) = entry.original_backup_relative.as_deref() else {
            return Err(AppError::runtime(
                "[PLAYBACK_LEDGER_INVALID] 原始备份路径缺失。",
            ));
        };
        let original_relative = safe_relative(original_relative)?;
        if !is_playback_relative(&original_relative) {
            return Err(AppError::runtime(
                "[PLAYBACK_LEDGER_INVALID] 原始备份路径不在 addons 白名单内。",
            ));
        }
        let original = persistent_backup.join(original_relative);
        ensure_safe_target(csgo, &original)?;
        if !is_regular_file(&original) {
            return Err(AppError::runtime(format!(
                "[PLAYBACK_LEDGER_INVALID] 原始文件备份缺失：{}",
                original.display()
            )));
        }
        if let Some(expected) = entry.original_sha256.as_deref() {
            if sha256_file(&original)? != expected {
                return Err(AppError::runtime(format!(
                    "[PLAYBACK_LEDGER_INVALID] 原始文件备份校验失败：{}",
                    original.display()
                )));
            }
        }
    }
    Ok(())
}

fn write_ledger(csgo: &Path, ledger: &InstallLedger) -> Result<(), AppError> {
    let path = ledger_path(csgo);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    let temporary = path.with_file_name(format!(
        ".{}-{}.tmp",
        LEDGER_RELATIVE.replace('/', "-"),
        std::process::id()
    ));
    ensure_safe_target(csgo, &path)?;
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(ledger).map_err(|error| AppError::runtime(error.to_string()))?,
    )
    .map_err(io_error)?;
    if !path.exists() {
        return fs::rename(&temporary, &path).map_err(io_error);
    }
    let backup = path.with_file_name(format!(".cs2as05-demotracer-ledger-{}.bak", unique_stamp()));
    fs::rename(&path, &backup).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        io_error(error)
    })?;
    match fs::rename(&temporary, &path) {
        Ok(()) => {
            let _ = fs::remove_file(&backup);
            Ok(())
        }
        Err(error) => {
            let _ = fs::rename(&backup, &path);
            let _ = fs::remove_file(&temporary);
            Err(io_error(error))
        }
    }
}

fn is_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_file() && !metadata.file_type().is_symlink())
        .unwrap_or(false)
}

fn is_playback_relative(path: &Path) -> bool {
    path.components()
        .next()
        .is_some_and(|component| component.as_os_str().eq_ignore_ascii_case("addons"))
}

fn replace_file_from_source(
    root: &Path,
    target: &Path,
    source: &Path,
    tag: &str,
) -> Result<(), AppError> {
    ensure_safe_target(root, target)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    let temporary = target.with_file_name(format!(
        ".{}.demotracer-{}-{}.tmp",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file"),
        tag,
        std::process::id()
    ));
    fs::copy(source, &temporary).map_err(io_error)?;
    if let Err(error) = (|| {
        if target.exists() {
            fs::remove_file(target).map_err(io_error)?;
        }
        fs::rename(&temporary, target).map_err(io_error)
    })() {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

fn rollback_transaction(root: &Path, applied: &[AppliedFile]) -> Result<(), AppError> {
    for applied_file in applied.iter().rev() {
        let target = checked_child(root, &applied_file.relative)?;
        ensure_safe_target(root, &target)?;
        if let Some(backup) = &applied_file.previous_backup {
            if target.exists() {
                fs::remove_file(&target).map_err(io_error)?;
            }
            fs::copy(backup, &target).map_err(io_error)?;
        } else if target.is_file() {
            fs::remove_file(&target).map_err(io_error)?;
        }
    }
    Ok(())
}

fn verify_file(path: &Path, expected_size: u64, expected_sha256: &str) -> Result<(), AppError> {
    if !path.is_file() {
        return Err(AppError::runtime(format!(
            "资源文件不存在：{}",
            path.display()
        )));
    }
    if fs::metadata(path).map_err(io_error)?.len() != expected_size {
        return Err(AppError::runtime(format!(
            "资源文件大小不匹配：{}",
            path.display()
        )));
    }
    if !sha256_file(path)?.eq_ignore_ascii_case(expected_sha256) {
        return Err(AppError::runtime(format!(
            "资源文件摘要不匹配：{}",
            path.display()
        )));
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, AppError> {
    Ok(format!(
        "{:X}",
        Sha256::digest(fs::read(path).map_err(io_error)?)
    ))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:X}", Sha256::digest(bytes))
}

fn unique_stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn zip_error(error: zip::result::ZipError) -> AppError {
    AppError::runtime(format!("Playback ZIP 条目读取失败：{error}"))
}

fn io_error(error: io::Error) -> AppError {
    AppError::runtime(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{package_relative, safe_relative, verify_playback_package, REQUIRED_FILES};
    use std::path::{Path, PathBuf};

    #[test]
    fn rejects_zip_traversal_and_absolute_paths() {
        assert!(safe_relative("addons/demo/file.dll").is_ok());
        assert!(safe_relative("../outside.dll").is_err());
        assert!(safe_relative("C:/outside.dll").is_err());
        assert!(safe_relative("/outside.dll").is_err());
    }

    #[test]
    fn strips_release_archive_root() {
        assert_eq!(
            package_relative("demotracer-css-v1.5.2/addons/demo/file.dll")
                .unwrap()
                .unwrap(),
            Path::new("addons/demo/file.dll")
        );
        assert!(package_relative("demotracer-css-v1.5.2/LICENSE")
            .unwrap()
            .is_none());
    }

    #[test]
    fn bundled_playback_manifest_is_verified() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("demotracer")
            .join("demotracer-css-v1.5.2.zip");
        let files =
            verify_playback_package(&path).expect("bundled Playback must pass its manifest");
        assert!(files.len() >= REQUIRED_FILES.len());
        assert!(files.iter().any(|file| {
            file.relative == Path::new("addons/dtr-controller/bin/win64/dtr-controller.dll")
        }));
    }
}
