use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use chrono::Local;
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
#[cfg(not(windows))]
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tauri::{AppHandle, Manager};
#[cfg(windows)]
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_NO_MORE_FILES, FILETIME, HANDLE, HWND, INVALID_HANDLE_VALUE,
};
#[cfg(windows)]
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, TerminateProcess,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE,
};
use zip::ZipArchive;

use crate::errors::AppError;
use crate::models::cs2::{
    Cs2EnvironmentStatus, Cs2ProcessInfo, Cs2ProcessSnapshot, Cs2RootCandidate, DiagnosticsPayload,
    OperationResult,
};
use crate::services::panel;

const CS2_FOLDER_NAME: &str = "Counter-Strike Global Offensive";
const BUNDLED_ZIP_NAME: &str = "CS2BotImprover.zip";
const CUSTOM_ZIP_SHA256: &str = "634BC9B854A0F39349474EC73463E4CAED6454BB0344DA7C89B9A1D2D268FE7F";
const PANEL_FILE_NAME: &str = "Panel v1.4.4.exe";
const PANEL_SHA256: &str = "2797A3FE85E65959CAE9501525B67B3876CEF65152E88DC716F64D5485AC2182";
const PANEL_SIZE: u64 = 5_890_560;
const PLUGIN_MARKER: &str = "addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json";
const MAP_ROTATION_DEFAULT_CONFIG: &str =
    "addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json";
const BOTVISION_SOURCE_SHA256: &str =
    "40B596D34BF336D9E59E663DAC2F94BD7C61D951C56E421EF66B5190B8787290";
const BOTVISION_ENTRIES: &[&str] = &[
    "addons/BotVision/gamedata.json",
    "addons/BotVision/bin/win64/BotVision.dll",
    "addons/metamod/BotVision.vdf",
    "addons/counterstrikesharp/plugins/MapRotation/MapRotation.dll",
];
const PLUGIN_PRODUCT: &str = "cs2-bot-improver";
const PLUGIN_ID: &str = "cs2as05-custom-package";
const LOG_DIR_NAME: &str = "CS2人机增强助手";
const REQUIRED_ZIP_ENTRIES: &[&str] = &[
    PANEL_FILE_NAME,
    "gameinfo.gi",
    "backup/Online/gameinfo.gi",
    "backup/WithBots/gameinfo.gi",
    "addons/",
    "cfg/",
    "overrides/",
    PLUGIN_MARKER,
    "addons/BotVision/gamedata.json",
    "addons/BotVision/bin/win64/BotVision.dll",
    "addons/metamod/BotVision.vdf",
    "addons/counterstrikesharp/plugins/MapRotation/MapRotation.dll",
    MAP_ROTATION_DEFAULT_CONFIG,
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginMarker {
    schema: u32,
    product: String,
    plugin_id: String,
    version: String,
    payload_sha256: String,
    payload_entries: Vec<String>,
    #[serde(default)]
    mutable_config_entries: Vec<String>,
    #[serde(default)]
    components: Vec<PluginComponent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginComponent {
    id: String,
    version: Option<String>,
    source_sha256: Option<String>,
}

#[derive(Debug)]
pub enum PluginVersionStatus {
    Missing,
    Invalid {
        reason: String,
        version: Option<Version>,
    },
    Valid {
        version: Version,
    },
}

pub fn discover_cs2_roots() -> Result<Vec<Cs2RootCandidate>, AppError> {
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();
    for drive in existing_drives() {
        let steam = drive.join("Program Files (x86)").join("Steam");
        add_candidate(
            &mut seen,
            &mut candidates,
            steam.join("steamapps").join("common").join(CS2_FOLDER_NAME),
            "Steam 常规目录",
        );
        for library in parse_libraryfolders(&steam.join("steamapps").join("libraryfolders.vdf"))? {
            add_candidate(
                &mut seen,
                &mut candidates,
                library
                    .join("steamapps")
                    .join("common")
                    .join(CS2_FOLDER_NAME),
                "Steam 库配置",
            );
        }
    }
    write_log(
        "INFO",
        &format!("扫描到 {} 个 CS2 候选目录。", candidates.len()),
    );
    Ok(candidates)
}

pub fn inspect_cs2_root(root_path: &str) -> Result<Cs2EnvironmentStatus, AppError> {
    let root = normalize_root(root_path)?;
    let csgo = root.join("game").join("csgo");
    let game_dir = root.join("game");
    let status = Cs2EnvironmentStatus {
        root_path: root.display().to_string(),
        game_dir_exists: game_dir.is_dir(),
        csgo_dir_exists: csgo.is_dir(),
        metamod_exists: csgo.join("addons").join("metamod").is_dir(),
        counterstrike_sharp_exists: csgo.join("addons").join("counterstrikesharp").is_dir(),
        gameinfo_exists: csgo.join("gameinfo.gi").is_file(),
        backup_online_gameinfo_exists: csgo
            .join("backup")
            .join("Online")
            .join("gameinfo.gi")
            .is_file(),
        backup_withbots_gameinfo_exists: csgo
            .join("backup")
            .join("WithBots")
            .join("gameinfo.gi")
            .is_file(),
        base_environment_ready: csgo.join("addons").join("metamod").is_dir()
            && csgo.join("addons").join("counterstrikesharp").is_dir()
            && csgo.join("gameinfo.gi").is_file()
            && csgo
                .join("backup")
                .join("Online")
                .join("gameinfo.gi")
                .is_file()
            && csgo
                .join("backup")
                .join("WithBots")
                .join("gameinfo.gi")
                .is_file(),
    };
    write_log("INFO", &format!("已检查 CS2 目录：{}。", status.root_path));
    Ok(status)
}

pub fn check_cs2_process() -> Result<bool, AppError> {
    Ok(!list_cs2_processes()?.is_empty())
}

fn is_cs2_process_name(name: &str) -> bool {
    let name = name.trim_end_matches('\0');
    name.eq_ignore_ascii_case("cs2") || name.eq_ignore_ascii_case("cs2.exe")
}

#[cfg(windows)]
unsafe fn process_metadata(pid: u32) -> (Option<String>, Option<u64>) {
    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
    if handle.is_null() {
        return (None, None);
    }
    let mut buffer = [0u16; 1024];
    let mut length = buffer.len() as u32;
    let exe_path = if QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length) != 0 {
        Some(String::from_utf16_lossy(&buffer[..length as usize]))
    } else {
        None
    };
    let mut creation = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut exit = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut kernel = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut user = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let start_time = (GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user)
        != 0)
        .then(|| ((creation.dwHighDateTime as u64) << 32) | creation.dwLowDateTime as u64);
    CloseHandle(handle);
    (exe_path, start_time)
}

#[cfg(windows)]
fn list_cs2_processes() -> Result<Vec<Cs2ProcessInfo>, AppError> {
    struct Snapshot(HANDLE);

    impl Drop for Snapshot {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(AppError::runtime(format!(
            "创建 Windows 进程快照失败：{}",
            std::io::Error::last_os_error()
        )));
    }
    let snapshot = Snapshot(snapshot);
    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    if unsafe { Process32FirstW(snapshot.0, &mut entry) } == 0 {
        let error = unsafe { GetLastError() };
        if error == ERROR_NO_MORE_FILES {
            return Ok(Vec::new());
        }
        return Err(AppError::runtime(format!(
            "枚举 Windows 进程失败：{}",
            std::io::Error::from_raw_os_error(error as i32)
        )));
    }

    let mut result = Vec::new();
    loop {
        let name_end = entry
            .szExeFile
            .iter()
            .position(|character| *character == 0)
            .unwrap_or(entry.szExeFile.len());
        let name = String::from_utf16_lossy(&entry.szExeFile[..name_end]);
        if is_cs2_process_name(&name) {
            let (exe_path, start_time) = unsafe { process_metadata(entry.th32ProcessID) };
            result.push(Cs2ProcessInfo {
                pid: entry.th32ProcessID,
                exe_name: name,
                exe_path,
                parent_pid: Some(entry.th32ParentProcessID),
                start_time,
            });
        }
        if unsafe { Process32NextW(snapshot.0, &mut entry) } == 0 {
            let error = unsafe { GetLastError() };
            if error == ERROR_NO_MORE_FILES {
                return Ok(result);
            }
            return Err(AppError::runtime(format!(
                "枚举 Windows 进程失败：{}",
                std::io::Error::from_raw_os_error(error as i32)
            )));
        }
    }
}

#[cfg(not(windows))]
fn list_cs2_processes() -> Result<Vec<Cs2ProcessInfo>, AppError> {
    let system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
    );
    Ok(system
        .processes()
        .values()
        .filter(|process| is_cs2_process_name(&process.name().to_string_lossy()))
        .map(|process| Cs2ProcessInfo {
            pid: process.pid().as_u32(),
            exe_name: process.name().to_string_lossy().into_owned(),
            exe_path: process.exe().map(|path| path.display().to_string()),
            parent_pid: process.parent().map(|pid| pid.as_u32()),
            start_time: Some(process.start_time()),
        })
        .collect())
}

pub fn get_cs2_process_snapshot() -> Result<Cs2ProcessSnapshot, AppError> {
    let first = list_cs2_processes()?;
    std::thread::sleep(std::time::Duration::from_millis(180));
    let second = list_cs2_processes()?;
    let processes = if first.is_empty() && second.is_empty() {
        Vec::new()
    } else if !second.is_empty() {
        second
    } else {
        first
    };
    Ok(Cs2ProcessSnapshot {
        observed_at: chrono::Utc::now().timestamp_millis(),
        processes,
        confidence: "high".into(),
        sample_count: 2,
    })
}

pub fn close_cs2(force: bool) -> Result<OperationResult, AppError> {
    write_runtime_log("INFO", &format!("[CS2_CLOSE_BEGIN] force={}。", force));
    let snapshot = get_cs2_process_snapshot()?;
    write_runtime_log(
        "INFO",
        &format!(
            "[CS2_CLOSE_SNAPSHOT] {}。",
            format_process_snapshot(&snapshot)
        ),
    );
    if snapshot.processes.is_empty() {
        write_runtime_log("INFO", "[CS2_CLOSE_FINAL] processes=0，CS2 已关闭。");
        return Ok(OperationResult {
            success: true,
            message: "CS2 已关闭。".into(),
        });
    }
    for process in &snapshot.processes {
        #[cfg(windows)]
        {
            if force {
                write_runtime_log(
                    "INFO",
                    &format!("[CS2_CLOSE_SIGNAL] stage=terminate pid={}。", process.pid),
                );
                unsafe {
                    terminate_process_by_pid(process.pid)?;
                }
            } else {
                write_runtime_log(
                    "INFO",
                    &format!("[CS2_CLOSE_SIGNAL] stage=wm_close pid={}。", process.pid),
                );
                let sent = unsafe { post_close_by_pid(process.pid) };
                if !sent {
                    write_log(
                        "WARN",
                        &format!(
                            "[CS2_CLOSE_NO_WINDOW] PID {} 没有可发送 WM_CLOSE 的窗口。",
                            process.pid
                        ),
                    );
                }
            }
        }
        #[cfg(not(windows))]
        {
            let mut command = Command::new("kill");
            command.args(["-TERM", &process.pid.to_string()]);
            command.output().map_err(|error| {
                AppError::runtime(format!(
                    "[CS2_CLOSE_SIGNAL] PID {} 操作失败：{error}",
                    process.pid
                ))
            })?;
        }
    }
    let deadline =
        std::time::Instant::now() + std::time::Duration::from_secs(if force { 5 } else { 8 });
    loop {
        let current = list_cs2_processes()?;
        if current.is_empty() {
            write_runtime_log("INFO", "[CS2_CLOSE_FINAL] processes=0，关闭确认成功。");
            return Ok(OperationResult {
                success: true,
                message: if force {
                    "CS2 已强制关闭。".into()
                } else {
                    "CS2 已关闭。".into()
                },
            });
        }
        if std::time::Instant::now() >= deadline {
            write_runtime_log(
                "WARN",
                &format!(
                    "[CS2_CLOSE_TIMEOUT] processes={}，仍有 CS2 进程。",
                    current
                        .iter()
                        .map(|p| p.pid.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            );
            return Ok(OperationResult {
                success: false,
                message: "CS2 仍在运行，可能未响应；确认后可强制关闭。".into(),
            });
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

fn format_process_snapshot(snapshot: &Cs2ProcessSnapshot) -> String {
    let pids = snapshot
        .processes
        .iter()
        .map(|process| process.pid.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "observedAt={} sampleCount={} confidence={} pids=[{}]",
        snapshot.observed_at, snapshot.sample_count, snapshot.confidence, pids
    )
}

#[cfg(windows)]
struct CloseWindowContext {
    pid: u32,
    sent: bool,
}

#[cfg(windows)]
unsafe extern "system" fn find_window_for_pid(window: HWND, lparam: isize) -> i32 {
    let context = &mut *(lparam as *mut CloseWindowContext);
    let mut pid = 0u32;
    GetWindowThreadProcessId(window, &mut pid);
    if pid == context.pid {
        context.sent |= PostMessageW(window, WM_CLOSE, 0, 0) != 0;
    }
    1
}

#[cfg(windows)]
unsafe fn post_close_by_pid(pid: u32) -> bool {
    let mut context = CloseWindowContext { pid, sent: false };
    let _ = EnumWindows(
        Some(find_window_for_pid),
        (&mut context as *mut CloseWindowContext) as isize,
    );
    context.sent
}

#[cfg(windows)]
unsafe fn terminate_process_by_pid(pid: u32) -> Result<(), AppError> {
    let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
    if handle.is_null() {
        return Err(AppError::runtime(format!(
            "[CS2_CLOSE_OPEN_PROCESS] 无法打开 CS2 PID {}：{}",
            pid,
            std::io::Error::last_os_error()
        )));
    }
    let result = TerminateProcess(handle, 1);
    CloseHandle(handle);
    if result == 0 {
        return Err(AppError::runtime(format!(
            "[CS2_CLOSE_TERMINATE] 无法强制关闭 CS2 PID {}：{}",
            pid,
            std::io::Error::last_os_error()
        )));
    }
    Ok(())
}

pub fn install_bot_package(
    app: &AppHandle,
    root_path: &str,
    keep_backup: bool,
) -> Result<OperationResult, AppError> {
    ensure_cs2_not_running()?;
    let root = normalize_root(root_path)?;
    let destination = root.join("game").join("csgo");
    if !destination.is_dir() {
        return Err(AppError::runtime(format!(
            "[INSTALL_TARGET_MISSING]\n未找到 CS2 游戏目录：{}",
            destination.display()
        )));
    }

    let zip_path = resolve_zip_path(app)?;
    verify_custom_zip(&zip_path)?;
    let retained_backup = install_game_files_transactionally(&zip_path, &destination, keep_backup)?;

    write_log(
        "INFO",
        &format!(
            "基于上游 v1.4.3 的最小定制插件包已安装到 {}。",
            destination.display()
        ),
    );
    Ok(OperationResult {
        success: true,
        message: format!(
            "基于上游 CS2-Bot-Improver v1.4.3 的最小定制包已安装。\n目标目录：{}\n已保留可识别的模式、难度、Aim、Nades、Bot 物品和刀具选择。",
            destination.display()
        ) + &retained_backup.map_or_else(String::new, |path| format!("\n本次写前备份已保留：{}", path.display())),
    })
}

pub fn open_upstream_panel(app: &AppHandle) -> Result<OperationResult, AppError> {
    let zip_path = resolve_zip_path(app)?;
    verify_custom_zip(&zip_path)?;
    let tool_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法确定应用数据目录：{error}")))?
        .join("tools")
        .join("official-panel-v1.4.3");
    let panel_path = tool_dir.join(PANEL_FILE_NAME);
    if !panel_is_valid(&panel_path)? {
        extract_panel_atomically(&zip_path, &tool_dir, &panel_path)?;
    }
    Command::new(&panel_path)
        .current_dir(&tool_dir)
        .spawn()
        .map_err(|error| {
            AppError::runtime(format!(
                "无法启动官方 Panel：{}\n路径：{}",
                error,
                panel_path.display()
            ))
        })?;
    write_log(
        "INFO",
        &format!("已启动官方 Panel：{}。", panel_path.display()),
    );
    Ok(OperationResult {
        success: true,
        message: format!("已启动官方 Panel v1.4.3。\n{}", panel_path.display()),
    })
}

pub fn uninstall_bot_package(root_path: &str) -> Result<OperationResult, AppError> {
    ensure_cs2_not_running()?;
    let root = normalize_root(root_path)?;
    let csgo = root.join("game").join("csgo");
    if !csgo.is_dir() {
        return Err(AppError::runtime("未找到 CS2 游戏目录，无法卸载。"));
    }
    let removed = remove_upstream_package(&csgo)?;
    write_log(
        "INFO",
        &format!("已从 {} 卸载官方插件文件。", csgo.display()),
    );
    Ok(OperationResult {
        success: true,
        message: format!(
            "已移除官方插件文件：{} 项。\n保留了 CS2 核心文件和 gameinfo.gi。",
            removed
        ),
    })
}

pub fn get_diagnostics_payload(root_path: Option<&str>) -> Result<DiagnosticsPayload, AppError> {
    let log_path = diagnostics_log_path();
    let full_log =
        fs::read_to_string(&log_path).unwrap_or_else(|_| "日志文件尚未创建。".to_string());
    let summary = match root_path {
        Some(root) => {
            let status = inspect_cs2_root(root)?;
            format!(
                "CS2 目录：{}\n游戏运行中：{}\n插件环境：{}",
                status.root_path,
                yes_no(check_cs2_process()?),
                if status.base_environment_ready {
                    "完整"
                } else {
                    "未完整安装"
                }
            )
        }
        None => format!(
            "尚未选择 CS2 目录。\n游戏运行中：{}",
            yes_no(check_cs2_process()?)
        ),
    };
    Ok(DiagnosticsPayload {
        summary,
        full_log,
        log_path: log_path.display().to_string(),
    })
}

fn resolve_zip_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|error| AppError::runtime(format!("无法定位内置资源：{error}")))?;
    let candidates = [
        resource_dir.join(BUNDLED_ZIP_NAME),
        resource_dir.join("resources").join(BUNDLED_ZIP_NAME),
    ];
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| AppError::runtime("未找到内置定制 CS2BotImprover.zip，无法继续。"))
}

fn verify_custom_zip(path: &Path) -> Result<(), AppError> {
    let digest = sha256_file(path)?;
    if digest != CUSTOM_ZIP_SHA256 {
        return Err(AppError::runtime(format!("[ZIP_HASH_INVALID]\n内置定制资源摘要不匹配。\n期望：{CUSTOM_ZIP_SHA256}\n实际：{digest}")));
    }
    let file = File::open(path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| AppError::runtime(format!("无法读取内置资源包：{error}")))?;
    for required in REQUIRED_ZIP_ENTRIES {
        let present = if required.ends_with('/') {
            // ZIP creators commonly omit explicit directory entries. Treat a
            // directory as present when it has at least one child entry.
            archive.file_names().any(|name| name.starts_with(required))
        } else {
            archive.by_name(required).is_ok()
        };
        if !present {
            return Err(AppError::runtime(format!(
                "[ZIP_STRUCTURE_INVALID]\n内置定制资源缺少必需条目：{required}"
            )));
        }
    }
    let manifest_bytes = archive
        .by_name("gameinfo.manifest.json")
        .map_err(|_| {
            AppError::runtime("[GAMEINFO_ASSET_INVALID] 内置资源缺少 gameinfo manifest。")
        })?
        .bytes()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            AppError::runtime(format!("[GAMEINFO_ASSET_INVALID] manifest 读取失败：{e}"))
        })?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|_| AppError::runtime("[GAMEINFO_ASSET_INVALID] gameinfo manifest 无法解析。"))?;
    let marker_bytes = archive
        .by_name(PLUGIN_MARKER)
        .map_err(|_| AppError::runtime("[BOT_PLUGIN_PAYLOAD_INVALID] 内置包缺少 marker。"))?
        .bytes()
        .collect::<Result<Vec<_>, _>>()
        .map_err(io_error)?;
    let marker: PluginMarker = serde_json::from_slice(&marker_bytes)
        .map_err(|_| AppError::runtime("[BOT_PLUGIN_PAYLOAD_INVALID] marker 无法解析。"))?;
    if marker
        .payload_entries
        .iter()
        .any(|entry| entry == MAP_ROTATION_DEFAULT_CONFIG)
        || !marker
            .mutable_config_entries
            .iter()
            .any(|entry| entry == MAP_ROTATION_DEFAULT_CONFIG)
    {
        return Err(AppError::runtime(
            "[BOT_PLUGIN_PAYLOAD_INVALID] 可变 MapRotation 配置分组无效。",
        ));
    }
    for name in [
        "gameinfo.gi",
        "backup/Online/gameinfo.gi",
        "backup/WithBots/gameinfo.gi",
    ] {
        let expected = manifest["entries"][name]["sha256"]
            .as_str()
            .unwrap_or_default();
        let mut entry = archive
            .by_name(name)
            .map_err(|_| AppError::runtime("[GAMEINFO_ASSET_INVALID] gameinfo 条目缺失。"))?;
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut bytes).map_err(io_error)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        if format!("{:X}", hasher.finalize()) != expected {
            return Err(AppError::runtime(format!(
                "[GAMEINFO_ASSET_INVALID] {name} 摘要不匹配。"
            )));
        }
    }
    let entry = archive
        .by_name(PANEL_FILE_NAME)
        .map_err(|_| AppError::runtime("内置定制资源缺少 Panel。"))?;
    if entry.size() != PANEL_SIZE {
        return Err(AppError::runtime(
            "[PANEL_SIZE_INVALID]\n官方 Panel 文件大小不正确。",
        ));
    }
    Ok(())
}

pub fn inspect_bot_plugin_version(root_path: &str) -> Result<PluginVersionStatus, AppError> {
    let root = normalize_root(root_path)?;
    inspect_bot_plugin_version_at(&root.join("game/csgo"))
}

pub fn ensure_bot_plugin_current(app: &AppHandle, root_path: &str) -> Result<String, AppError> {
    ensure_cs2_not_running()?;
    let root = normalize_root(root_path)?;
    let destination = root.join("game/csgo");
    let zip_path = resolve_zip_path(app)?;
    verify_custom_zip(&zip_path)?;
    let zip_hash = sha256_file(&zip_path)?;
    let _ = install_game_files_transactionally(&zip_path, &destination, false)?;
    let expected = current_plugin_version()?;
    match inspect_bot_plugin_version_at(&destination)? {
        PluginVersionStatus::Valid { version }
            if version == expected =>
        {
            write_log("INFO", &format!("BOT 插件已自动更新到 {version}。"));
            Ok(version.to_string())
        }
        PluginVersionStatus::Invalid { reason, .. } => Err(AppError::runtime(format!(
            "[BOT_PLUGIN_AUTO_INSTALL_FAILED] 自动安装后插件校验失败：{reason}\nexpectedVersion={expected}\ninstalledVersion=invalid\nzipSha256={zip_hash}\nmarkerPath={}", destination.join(PLUGIN_MARKER).display()
        ))),
        PluginVersionStatus::Missing => Err(AppError::runtime(
            format!("[BOT_PLUGIN_AUTO_INSTALL_FAILED] 自动安装后未找到插件版本标记。\nexpectedVersion={expected}\ninstalledVersion=missing\nzipSha256={zip_hash}\nmarkerPath={}", destination.join(PLUGIN_MARKER).display()),
        )),
        PluginVersionStatus::Valid { version } => Err(AppError::runtime(format!(
            "[BOT_PLUGIN_AUTO_INSTALL_FAILED] 自动安装后的插件版本 {version} 与当前程序版本 {expected} 不一致。\nexpectedVersion={expected}\ninstalledVersion={version}\nzipSha256={zip_hash}\nmarkerPath={}", destination.join(PLUGIN_MARKER).display()
        ))),
    }
}

fn current_plugin_version() -> Result<Version, AppError> {
    Version::parse(env!("CARGO_PKG_VERSION")).map_err(|error| {
        AppError::runtime(format!(
            "[BOT_PLUGIN_VERSION_INVALID] 当前程序版本无效：{error}"
        ))
    })
}

fn inspect_bot_plugin_version_at(csgo: &Path) -> Result<PluginVersionStatus, AppError> {
    let marker_path = csgo.join(PLUGIN_MARKER);
    if !marker_path.is_file() {
        return Ok(PluginVersionStatus::Missing);
    }
    let bytes = fs::read(&marker_path).map_err(io_error)?;
    let marker = match serde_json::from_slice::<PluginMarker>(&bytes) {
        Ok(marker) => marker,
        Err(error) => {
            return Ok(PluginVersionStatus::Invalid {
                reason: format!("JSON 无法解析：{error}"),
                version: None,
            })
        }
    };
    let version = Version::parse(&marker.version).ok();
    if marker.schema != 2 || marker.product != PLUGIN_PRODUCT || marker.plugin_id != PLUGIN_ID {
        return Ok(PluginVersionStatus::Invalid {
            reason: "标记身份字段不匹配。".into(),
            version,
        });
    }
    let Some(version) = version else {
        return Ok(PluginVersionStatus::Invalid {
            reason: "[BOT_PLUGIN_VERSION_INVALID] 版本字段无效。".into(),
            version: None,
        });
    };
    if marker.payload_entries.is_empty()
        || !marker
            .payload_entries
            .iter()
            .any(|entry| entry == "addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll")
        || BOTVISION_ENTRIES
            .iter()
            .any(|required| !marker.payload_entries.iter().any(|entry| entry == required))
        || marker
            .payload_entries
            .iter()
            .any(|entry| safe_zip_path(entry).is_err())
        || marker
            .payload_entries
            .iter()
            .any(|entry| entry == MAP_ROTATION_DEFAULT_CONFIG)
        || (csgo.join(MAP_ROTATION_DEFAULT_CONFIG).exists()
            && !marker
                .mutable_config_entries
                .iter()
                .any(|entry| entry == MAP_ROTATION_DEFAULT_CONFIG))
    {
        return Ok(PluginVersionStatus::Invalid {
            reason: "[BOT_PLUGIN_PAYLOAD_INVALID] payload 条目无效。".into(),
            version: Some(version),
        });
    }
    let botvision = marker
        .components
        .iter()
        .find(|component| component.id == "botvision");
    if botvision.and_then(|component| component.version.as_deref()) != Some("0.2.2")
        || botvision.and_then(|component| component.source_sha256.as_deref())
            != Some(BOTVISION_SOURCE_SHA256)
    {
        return Ok(PluginVersionStatus::Invalid {
            reason: "[BOTVISION_MARKER_INVALID] BotVision 组件 provenance 不匹配。".into(),
            version: Some(version),
        });
    }
    let digest = match payload_digest_from_files(csgo, &marker.payload_entries) {
        Ok(digest) => digest,
        Err(error) => {
            return Ok(PluginVersionStatus::Invalid {
                reason: format!("[BOT_PLUGIN_PAYLOAD_INVALID] {}", error.into_string()),
                version: Some(version),
            })
        }
    };
    if digest != marker.payload_sha256 {
        return Ok(PluginVersionStatus::Invalid {
            reason: format!(
                "[BOT_PLUGIN_PAYLOAD_INVALID] payload 摘要不匹配，期望 {}，实际 {digest}。",
                marker.payload_sha256
            ),
            version: Some(version),
        });
    }
    Ok(PluginVersionStatus::Valid { version })
}

fn payload_digest_from_files(base: &Path, entries: &[String]) -> Result<String, AppError> {
    let mut hasher = Sha256::new();
    let mut seen = BTreeSet::new();
    for entry in entries {
        if !seen.insert(entry) {
            return Err(AppError::runtime(format!("payload 包含重复条目：{entry}")));
        }
        let relative = safe_zip_path(entry)?;
        let path = base.join(relative);
        let metadata = fs::metadata(&path).map_err(|error| {
            AppError::runtime(format!("缺少 payload 文件 {}：{error}", path.display()))
        })?;
        if !metadata.is_file() {
            return Err(AppError::runtime(format!(
                "payload 不是文件：{}",
                path.display()
            )));
        }
        hasher.update(entry.as_bytes());
        hasher.update(b"\0");
        hasher.update(metadata.len().to_string().as_bytes());
        hasher.update(b"\0");
        let mut input = File::open(&path).map_err(io_error)?;
        let mut buffer = [0u8; 65_536];
        loop {
            let count = input.read(&mut buffer).map_err(io_error)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
    }
    Ok(format!("{:X}", hasher.finalize()))
}

fn install_game_files_transactionally(
    zip_path: &Path,
    destination: &Path,
    keep_backup: bool,
) -> Result<Option<PathBuf>, AppError> {
    let parent = destination
        .parent()
        .ok_or_else(|| AppError::runtime("[BOT_PLUGIN_AUTO_INSTALL_FAILED] CS2 目录层级无效。"))?;
    let nonce = format!(".cs2as05-install-{}", std::process::id());
    let staging = parent.join(format!("{nonce}-staging"));
    let backup = parent.join(format!("{nonce}-backup"));
    let mut touched: Vec<(PathBuf, bool)> = Vec::new();
    let result = (|| -> Result<(), AppError> {
        fs::create_dir_all(&staging).map_err(io_error)?;
        extract_game_files(zip_path, &staging)?;
        match inspect_bot_plugin_version_at(&staging)? {
            PluginVersionStatus::Valid { .. } => {}
            PluginVersionStatus::Missing => {
                return Err(AppError::runtime(
                    "[BOT_PLUGIN_AUTO_INSTALL_FAILED] 内置包缺少插件标记。",
                ))
            }
            PluginVersionStatus::Invalid { reason, .. } => {
                return Err(AppError::runtime(format!(
                    "[BOT_PLUGIN_AUTO_INSTALL_FAILED] 内置包标记无效：{reason}"
                )))
            }
        }
        let preferences = panel::capture_panel_preferences(destination)?;
        let files = collect_game_file_entries(zip_path)?;
        let state_relative = PathBuf::from("cfg/cs2as05-panel-state.json");
        let state_target = destination.join(&state_relative);
        let state_existed = state_target.is_file();
        if state_existed {
            let saved = backup.join(&state_relative);
            if let Some(parent) = saved.parent() {
                fs::create_dir_all(parent).map_err(io_error)?;
            }
            fs::copy(&state_target, &saved).map_err(io_error)?;
        }
        touched.push((state_relative, state_existed));
        let gameinfo_state_relative = PathBuf::from("cfg/cs2as05-gameinfo-state.json");
        let gameinfo_state_target = destination.join(&gameinfo_state_relative);
        let gameinfo_state_existed = gameinfo_state_target.is_file();
        if gameinfo_state_existed {
            let saved = backup.join(&gameinfo_state_relative);
            if let Some(parent) = saved.parent() {
                fs::create_dir_all(parent).map_err(io_error)?;
            }
            fs::copy(&gameinfo_state_target, &saved).map_err(io_error)?;
        }
        touched.push((gameinfo_state_relative, gameinfo_state_existed));
        let official_relative = PathBuf::from("gameinfo.gi.official.bin");
        let official_target = destination.join(&official_relative);
        let official_existed = official_target.is_file();
        if official_existed {
            let saved = backup.join(&official_relative);
            fs::copy(&official_target, &saved).map_err(io_error)?;
        }
        touched.push((official_relative, official_existed));
        for relative in &files {
            if relative == Path::new(MAP_ROTATION_DEFAULT_CONFIG)
                && destination.join(relative).is_file()
            {
                continue;
            }
            let source = staging.join(relative);
            let target = destination.join(relative);
            let existed = target.is_file();
            if existed {
                let saved = backup.join(relative);
                if let Some(parent) = saved.parent() {
                    fs::create_dir_all(parent).map_err(io_error)?;
                }
                fs::copy(&target, &saved).map_err(io_error)?;
            }
            touched.push((relative.clone(), existed));
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(io_error)?;
            }
            fs::copy(&source, &target).map_err(io_error)?;
        }
        let online_bytes =
            fs::read(destination.join("backup/Online/gameinfo.gi")).map_err(io_error)?;
        fs::write(destination.join("gameinfo.gi.official.bin"), &online_bytes).map_err(io_error)?;
        panel::write_gameinfo_sidecar(destination, env!("CARGO_PKG_VERSION"), "2026-08-26")?;
        panel::restore_panel_preferences(destination, &preferences, preferences.is_empty())?;
        Ok(())
    })();
    let _ = fs::remove_dir_all(&staging);
    match result {
        Ok(()) => {
            if keep_backup {
                write_log(
                    "INFO",
                    &format!("已按本次操作请求保留写前备份：{}。", backup.display()),
                );
            } else {
                let _ = fs::remove_dir_all(&backup);
            }
            Ok(if keep_backup { Some(backup) } else { None })
        }
        Err(error) => match rollback_transaction(&touched, &backup, destination) {
            Ok(()) => {
                let _ = fs::remove_dir_all(&backup);
                Err(error)
            }
            Err(rollback_error) => Err(AppError::runtime(format!(
                "{}\n[BOT_PLUGIN_ROLLBACK_FAILED] 回滚失败：{}\n备份保留于：{}",
                error.into_string(),
                rollback_error.into_string(),
                backup.display()
            ))),
        },
    }
}

fn collect_game_file_entries(zip_path: &Path) -> Result<Vec<PathBuf>, AppError> {
    let file = File::open(zip_path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| AppError::runtime(format!("无法读取资源包：{error}")))?;
    let mut files = Vec::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| AppError::runtime(format!("无法读取资源条目：{error}")))?;
        if entry.is_dir() || entry.name() == PANEL_FILE_NAME {
            continue;
        }
        files.push(safe_zip_path(entry.name())?);
    }
    files.sort_by_key(|path| path.to_string_lossy().to_string());
    files.sort_by_key(|path| path == Path::new(PLUGIN_MARKER));
    Ok(files)
}

fn rollback_transaction(
    touched: &[(PathBuf, bool)],
    backup: &Path,
    destination: &Path,
) -> Result<(), AppError> {
    for (relative, existed) in touched.iter().rev() {
        let target = destination.join(relative);
        if target.is_file() {
            fs::remove_file(&target).map_err(io_error)?;
        }
        if *existed {
            let saved = backup.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(io_error)?;
            }
            fs::copy(saved, target).map_err(io_error)?;
        }
    }
    Ok(())
}

fn extract_game_files(zip_path: &Path, destination: &Path) -> Result<(), AppError> {
    let file = File::open(zip_path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| AppError::runtime(format!("无法读取资源包：{error}")))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| AppError::runtime(format!("无法读取资源条目：{error}")))?;
        if entry.name() == PANEL_FILE_NAME {
            continue;
        }
        let relative = safe_zip_path(entry.name())?;
        if relative.as_os_str().is_empty() {
            continue;
        }
        let target = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(io_error)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        let mut output = File::create(&target).map_err(io_error)?;
        io::copy(&mut entry, &mut output).map_err(io_error)?;
    }
    Ok(())
}

fn extract_panel_atomically(
    zip_path: &Path,
    tool_dir: &Path,
    panel_path: &Path,
) -> Result<(), AppError> {
    fs::create_dir_all(tool_dir).map_err(io_error)?;
    let temporary = tool_dir.join("Panel-v1.4.3.tmp");
    let file = File::open(zip_path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| AppError::runtime(format!("无法读取资源包：{error}")))?;
    let mut entry = archive
        .by_name(PANEL_FILE_NAME)
        .map_err(|_| AppError::runtime("官方资源缺少 Panel。"))?;
    let mut output = File::create(&temporary).map_err(io_error)?;
    io::copy(&mut entry, &mut output).map_err(io_error)?;
    output.flush().map_err(io_error)?;
    drop(output);
    if !panel_is_valid(&temporary)? {
        return Err(AppError::runtime(
            "[PANEL_HASH_INVALID]\n从官方资源提取的 Panel 校验失败。",
        ));
    }
    if panel_path.exists() {
        fs::remove_file(panel_path).map_err(io_error)?;
    }
    fs::rename(&temporary, panel_path).map_err(io_error)
}

fn panel_is_valid(path: &Path) -> Result<bool, AppError> {
    Ok(path.is_file()
        && fs::metadata(path).map_err(io_error)?.len() == PANEL_SIZE
        && sha256_file(path)? == PANEL_SHA256)
}

fn remove_upstream_package(csgo: &Path) -> Result<usize, AppError> {
    let paths = [
        csgo.join("addons"),
        csgo.join("backup"),
        csgo.join("cfg").join("plugins"),
        csgo.join("overrides").join("Low"),
        csgo.join("overrides").join("Medium"),
        csgo.join("overrides").join("High"),
    ];
    let files = [
        csgo.join("cfg").join("my_bot_normal_config.cfg"),
        csgo.join("cfg").join("my_bot_ffa_config.cfg"),
        csgo.join("overrides").join("botprofile.vpk"),
        csgo.join("metamod.vdf"),
        csgo.join("metamod_x64.vdf"),
    ];
    let mut removed = 0;
    for path in paths {
        if path.exists() {
            fs::remove_dir_all(&path).map_err(io_error)?;
            removed += 1;
        }
    }
    for path in files {
        if path.exists() {
            fs::remove_file(&path).map_err(io_error)?;
            removed += 1;
        }
    }
    Ok(removed)
}

pub(crate) fn normalize_root(path: &str) -> Result<PathBuf, AppError> {
    let selected = PathBuf::from(path.trim());
    let root = if selected.join("game").join("csgo").is_dir() {
        selected
    } else if selected
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("game"))
        && selected.join("csgo").is_dir()
    {
        selected
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| AppError::runtime("CS2 根目录无效。"))?
    } else if selected
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("csgo"))
        && selected.parent().is_some_and(|parent| {
            parent
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("game"))
        })
    {
        selected
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .ok_or_else(|| AppError::runtime("CS2 根目录无效。"))?
    } else {
        return Err(AppError::runtime(
            "请选择 Counter-Strike Global Offensive 目录，或其中的 game/csgo 目录。",
        ));
    };
    dunce::canonicalize(&root).map_err(|error| {
        AppError::runtime(format!(
            "无法规范化 CS2 根目录：{}\n{error}",
            root.display()
        ))
    })
}

fn safe_zip_path(name: &str) -> Result<PathBuf, AppError> {
    let path = Path::new(name);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err(AppError::runtime(format!("资源包包含不安全路径：{name}")));
    }
    Ok(path.to_path_buf())
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

fn existing_drives() -> Vec<PathBuf> {
    (b'A'..=b'Z')
        .filter_map(|letter| {
            let path = PathBuf::from(format!("{}:\\", letter as char));
            path.is_dir().then_some(path)
        })
        .collect()
}

fn parse_libraryfolders(path: &Path) -> Result<Vec<PathBuf>, AppError> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).map_err(io_error)?;
    let mut paths = Vec::new();
    for line in content.lines() {
        let value = line
            .split('"')
            .nth(3)
            .unwrap_or_default()
            .replace("\\\\", "\\");
        if value.contains(":\\") {
            paths.push(PathBuf::from(value));
        }
    }
    Ok(paths)
}

fn add_candidate(
    seen: &mut BTreeSet<String>,
    candidates: &mut Vec<Cs2RootCandidate>,
    path: PathBuf,
    source: &str,
) {
    if path.join("game").join("csgo").is_dir() {
        let normalized = path.display().to_string();
        if seen.insert(normalized.clone()) {
            candidates.push(Cs2RootCandidate {
                path: normalized,
                source: source.to_string(),
            });
        }
    }
}

fn ensure_cs2_not_running() -> Result<(), AppError> {
    if check_cs2_process()? {
        return Err(AppError::runtime(
            "检测到 cs2.exe 正在运行。请先退出 CS2，再执行此操作。",
        ));
    }
    Ok(())
}

fn diagnostics_log_path() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(LOG_DIR_NAME)
        .join("logs")
        .join("runtime.log")
}

fn write_log(level: &str, message: &str) {
    let path = diagnostics_log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(
            file,
            "[{}] [{}] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            level,
            message
        );
    }
}

pub(crate) fn write_runtime_log(level: &str, message: &str) {
    write_log(level, message);
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "是"
    } else {
        "否"
    }
}
fn io_error(error: io::Error) -> AppError {
    AppError::runtime(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn matches_only_exact_cs2_process_names() {
        for name in ["cs2.exe", "CS2.EXE", "cs2", "cs2.exe\0\0"] {
            assert!(is_cs2_process_name(name), "expected {name:?} to match");
        }
        for name in ["steam.exe", "cs2.exe.bak", "", "cs2 "] {
            assert!(!is_cs2_process_name(name), "expected {name:?} not to match");
        }
    }

    fn write_test_marker(csgo: &Path, version: &str) {
        let entries = vec![
            "addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll",
            "addons/BotVision/gamedata.json",
            "addons/BotVision/bin/win64/BotVision.dll",
            "addons/metamod/BotVision.vdf",
            "addons/counterstrikesharp/plugins/MapRotation/MapRotation.dll",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        for (index, entry) in entries.iter().enumerate() {
            let file = csgo.join(entry);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, format!("test-payload-{index}")).unwrap();
        }
        let digest = payload_digest_from_files(csgo, &entries).unwrap();
        let marker = serde_json::json!({
            "schema": 2,
            "product": PLUGIN_PRODUCT,
            "pluginId": PLUGIN_ID,
            "version": version,
            "components": [{"id": "botvision", "version": "0.2.2", "sourceSha256": BOTVISION_SOURCE_SHA256}],
            "payloadSha256": digest,
            "payloadEntries": entries,
            "generatedFrom": "test"
        });
        fs::write(
            csgo.join(PLUGIN_MARKER),
            serde_json::to_vec(&marker).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn plugin_marker_accepts_current_prerelease_and_higher_core_versions() {
        for version in ["0.5.4", "0.5.4-test.1", "0.5.5-beta.1", "0.6.0-test"] {
            let root = std::env::temp_dir().join(format!("plugin-marker-{version}"));
            let csgo = root.join("game/csgo");
            write_test_marker(&csgo, version);
            assert!(
                matches!(inspect_bot_plugin_version_at(&csgo).unwrap(), PluginVersionStatus::Valid { version: parsed } if parsed == Version::parse(version).unwrap())
            );
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn plugin_marker_reports_missing_invalid_and_tampered_payload() {
        let root =
            std::env::temp_dir().join(format!("plugin-marker-invalid-{}", std::process::id()));
        let csgo = root.join("game/csgo");
        fs::create_dir_all(&csgo).unwrap();
        assert!(matches!(
            inspect_bot_plugin_version_at(&csgo).unwrap(),
            PluginVersionStatus::Missing
        ));
        fs::create_dir_all(csgo.join(Path::new(PLUGIN_MARKER).parent().unwrap())).unwrap();
        fs::write(csgo.join(PLUGIN_MARKER), b"not-json").unwrap();
        assert!(matches!(
            inspect_bot_plugin_version_at(&csgo).unwrap(),
            PluginVersionStatus::Invalid { version: None, .. }
        ));
        write_test_marker(&csgo, "0.6.0-test");
        fs::write(
            csgo.join("addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll"),
            b"tampered",
        )
        .unwrap();
        assert!(
            matches!(inspect_bot_plugin_version_at(&csgo).unwrap(), PluginVersionStatus::Invalid { version: Some(version), .. } if version == Version::parse("0.6.0-test").unwrap())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bundled_custom_zip_verifies_and_extracts_into_fake_cs2_root() {
        let zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(BUNDLED_ZIP_NAME);
        verify_custom_zip(&zip_path).expect("bundled custom ZIP must pass its release contract");

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        let fake_root =
            std::env::temp_dir().join(format!("ai-pc-fac-{}-{nonce}", env!("CARGO_PKG_VERSION")));
        let csgo = fake_root.join("game").join("csgo");
        fs::create_dir_all(&csgo).expect("fake CS2 root must be creatable");
        extract_game_files(&zip_path, &csgo)
            .expect("custom ZIP must extract through installer logic");

        let dll = csgo
            .join("addons")
            .join("counterstrikesharp")
            .join("plugins")
            .join("NadeSystem")
            .join("NadeSystem.dll");
        let marker_status =
            inspect_bot_plugin_version_at(&csgo).expect("marker inspection must succeed");
        let assertions = (
            csgo.join("gameinfo.gi").is_file(),
            csgo.join("backup")
                .join("Online")
                .join("gameinfo.gi")
                .is_file(),
            !csgo.join(PANEL_FILE_NAME).exists(),
            sha256_file(&dll).expect("custom NadeSystem DLL must be readable"),
            matches!(&marker_status, PluginVersionStatus::Valid { version } if version == &Version::parse(env!("CARGO_PKG_VERSION")).unwrap()),
        );
        fs::remove_dir_all(&fake_root).expect("fake CS2 root must be removable");

        assert!(assertions.0, "gameinfo.gi must be installed");
        assert!(assertions.1, "Online gameinfo backup must be installed");
        assert!(assertions.2, "Panel must not be installed into game files");
        assert_eq!(
            assertions.3,
            "2668B41B019F2BDBB7C89051136B95EFD0E46A7A044553408FDF33048B4A2654"
        );
        assert!(
            assertions.4,
            "installed payload marker must verify: {marker_status:?}"
        );
    }

    #[test]
    fn transactional_install_preserves_unknown_files_and_finishes_with_valid_marker() {
        let zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(BUNDLED_ZIP_NAME);
        let root = std::env::temp_dir().join(format!("plugin-transaction-{}", std::process::id()));
        let csgo = root.join("game/csgo");
        fs::create_dir_all(csgo.join("addons/unknown-plugin")).unwrap();
        fs::write(csgo.join("addons/unknown-plugin/user.txt"), b"keep").unwrap();
        install_game_files_transactionally(&zip_path, &csgo, false).unwrap();
        assert_eq!(
            fs::read(csgo.join("addons/unknown-plugin/user.txt")).unwrap(),
            b"keep"
        );
        assert!(
            matches!(inspect_bot_plugin_version_at(&csgo).unwrap(), PluginVersionStatus::Valid { version } if version == Version::parse(env!("CARGO_PKG_VERSION")).unwrap())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn transactional_install_returns_retained_backup_path_when_requested() {
        let zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(BUNDLED_ZIP_NAME);
        let root =
            std::env::temp_dir().join(format!("plugin-backup-retain-{}", std::process::id()));
        let csgo = root.join("game/csgo");
        fs::create_dir_all(&csgo).unwrap();
        install_game_files_transactionally(&zip_path, &csgo, false).unwrap();
        let retained = install_game_files_transactionally(&zip_path, &csgo, true)
            .unwrap()
            .expect("keep_backup=true must return the transaction backup path");
        assert!(retained.is_absolute());
        assert!(retained.is_dir());
        assert!(retained
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("-backup"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn transactional_reinstall_preserves_all_explicit_panel_preferences() {
        let zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(BUNDLED_ZIP_NAME);
        let root = std::env::temp_dir().join(format!(
            "panel-reinstall-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let csgo = root.join("game/csgo");
        fs::create_dir_all(&csgo).unwrap();
        install_game_files_transactionally(&zip_path, &csgo, false).unwrap();
        let bot_items_path = csgo.join("addons/counterstrikesharp/configs/core.json");
        fs::write(
            &bot_items_path,
            br#"{"bot_hider github.com/XBribo all":true,"bot_randomizer github.com/ed0ard agents":true,"bot_randomizer github.com/ed0ard music":true,"bot_randomizer github.com/ed0ard weapons":true,"bot_randomizer github.com/ed0ard knives":true,"bot_randomizer github.com/ed0ard gloves":true,"bot_randomizer github.com/ed0ard stickers":true,"bot_randomizer github.com/ed0ard charms":true,"futureOption":42}"#,
        )
        .unwrap();
        let root_text = root.to_string_lossy();
        panel::set_mode(&root_text, "online").unwrap();
        panel::set_difficulty(&root_text, "High").unwrap();
        panel::set_preset(&root_text, "bot_aim", "head").unwrap();
        panel::set_preset(&root_text, "bot_nades", "less").unwrap();
        for item in [
            "profiles", "agents", "music", "weapons", "knives", "gloves", "stickers", "charms",
        ] {
            panel::set_bot_item(&root_text, item, false).unwrap();
        }
        panel::set_drop_knives(&root_text, "f8", &[]).unwrap();

        install_game_files_transactionally(&zip_path, &csgo, false).unwrap();
        let snapshot = panel::snapshot(&root_text).unwrap();
        assert_eq!(snapshot.mode.current.as_deref(), Some("online"));
        assert_eq!(snapshot.difficulty.current.as_deref(), Some("High"));
        assert_eq!(snapshot.presets.aim.as_deref(), Some("head"));
        assert_eq!(snapshot.presets.nades.as_deref(), Some("less"));
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
        assert_eq!(snapshot.drop_knives.bind_key, "f8");
        assert!(snapshot.drop_knives.selected.is_empty());
        let bot_items: serde_json::Value =
            serde_json::from_slice(&fs::read(bot_items_path).unwrap()).unwrap();
        assert_eq!(bot_items["futureOption"], 42);
        assert!(csgo.join("cfg/cs2as05-panel-state.json").is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn transaction_rollback_restores_existing_files_and_removes_new_files() {
        let root = std::env::temp_dir().join(format!(
            "plugin-rollback-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let destination = root.join("destination");
        let backup = root.join("backup");
        let existing = PathBuf::from("addons/existing.dll");
        let added = PathBuf::from("addons/added.dll");
        fs::create_dir_all(destination.join("addons")).unwrap();
        fs::create_dir_all(backup.join("addons")).unwrap();
        fs::write(destination.join(&existing), b"replacement").unwrap();
        fs::write(destination.join(&added), b"new").unwrap();
        fs::write(backup.join(&existing), b"original").unwrap();

        rollback_transaction(
            &[(existing.clone(), true), (added.clone(), false)],
            &backup,
            &destination,
        )
        .unwrap();

        assert_eq!(fs::read(destination.join(existing)).unwrap(), b"original");
        assert!(!destination.join(added).exists());
        fs::remove_dir_all(root).unwrap();
    }
}
