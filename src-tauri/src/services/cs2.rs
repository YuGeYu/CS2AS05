use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use chrono::Local;
use semver::Version;
use serde::{Deserialize, Serialize};
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
    Cs2CloseOverride, Cs2EnvironmentStatus, Cs2ProcessInfo, Cs2ProcessSnapshot, Cs2RootCandidate,
    DiagnosticsPayload, OperationResult,
};
use crate::services::panel;

const CS2_FOLDER_NAME: &str = "Counter-Strike Global Offensive";
const BUNDLED_ZIP_NAME: &str = "CS2BotImprover.zip";
const EMBEDDED_BUNDLED_ZIP: &[u8] = include_bytes!("../../resources/CS2BotImprover.zip");
const SKIN_ONLY_GAMEINFO_ENTRY: &str = "backup/SkinOnly/gameinfo.gi";
const PANEL_FILE_NAME: &str = "Panel v1.4.5.exe";
const PANEL_SHA256: &str = "9C6BD8E2503AFC9CAEB5DD64C8B8BF0EC5967BF50CD442015E7CEBEB69038410";
const PANEL_SIZE: u64 = 6_032_896;
const PLUGIN_MARKER: &str = "addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json";
const BOT_VISION_STATE_FILE: &str = "cfg/cs2as05-volume-smoke.state";
const BOT_VISION_VDF: &str = "addons/metamod/BotVision.vdf";
const PLUGIN_PRODUCT: &str = "cs2-bot-improver";
const PLUGIN_ID: &str = "cs2as05-custom-package";
const LOG_DIR_NAME: &str = "CS2人机增强助手";
const INSTALL_LEDGER_RELATIVE: &str = "cfg/cs2as05-install-ledger.json";
const INSTALL_BACKUP_DIR_RELATIVE: &str = "cfg/cs2as05-install-backups";
const INSTALL_LEDGER_SCHEMA: u32 = 1;
// 这些目录来自旧的下游实验包。v1.4.5 只允许上游官方资源进入游戏目录；
// 安装事务会先把旧目录移入本次事务备份，成功后删除，失败则原样恢复。
const OBSOLETE_DOWNSTREAM_COMPONENTS: &[&str] = &[
    "addons/counterstrikesharp/plugins/MapRotation",
    "addons/counterstrikesharp/configs/plugins/MapRotation",
    "addons/counterstrikesharp/plugins/CS2BotLlmChat",
    "addons/counterstrikesharp/configs/plugins/CS2BotLlmChat",
];
const UPSTREAM_CORE_ENTRIES: &[&str] = &[
    "addons/metamod/counterstrikesharp.vdf",
    "addons/counterstrikesharp/bin/win64/counterstrikesharp.dll",
    "addons/counterstrikesharp/plugins/BotAI/BotAI.dll",
    "addons/counterstrikesharp/plugins/BotRandomizer/BotRandomizer.dll",
    "addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll",
    "overrides/Low/botprofile.vpk",
    "overrides/Medium/botprofile.vpk",
    "overrides/High/botprofile.vpk",
];
const UPSTREAM_PANEL_REQUIRED_FILES: &[&str] = &[
    "cfg/gamemode_armsrace.cfg",
    "cfg/gamemode_casual.cfg",
    "cfg/gamemode_competitive.cfg",
    "cfg/gamemode_competitive2v2.cfg",
    "cfg/gamemode_deathmatch.cfg",
    "cfg/gamemode_dm_freeforall.cfg",
    "cfg/gamemode_retakecasual.cfg",
    "cfg/gamemode_teamdeathmatch.cfg",
    "cfg/gamemode_workshop.cfg",
    "cfg/my_bot_rush_config.cfg",
    "gameinfo.gi",
];
const UPSTREAM_PANEL_CFG_FILES: &[&str] = &[
    "cfg/gamemode_armsrace.cfg",
    "cfg/gamemode_casual.cfg",
    "cfg/gamemode_competitive.cfg",
    "cfg/gamemode_competitive2v2.cfg",
    "cfg/gamemode_deathmatch.cfg",
    "cfg/gamemode_dm_freeforall.cfg",
    "cfg/gamemode_retakecasual.cfg",
    "cfg/gamemode_teamdeathmatch.cfg",
    "cfg/gamemode_workshop.cfg",
    "cfg/my_bot_rush_config.cfg",
];
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginMarker {
    schema: u32,
    product: String,
    plugin_id: String,
    version: String,
    #[serde(default)]
    payload_entries: Vec<String>,
    #[serde(default)]
    mutable_config_entries: Vec<String>,
}

/// Records the bytes that existed before this assistant replaced a package
/// file. The ledger is stored in the CS2 directory's project-owned cfg area;
/// it lets uninstall restore Steam files instead of deleting them by name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallLedger {
    schema: u32,
    entries: Vec<InstallLedgerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallLedgerEntry {
    relative: String,
    existed_before: bool,
    #[serde(default)]
    original_backup_relative: Option<String>,
    #[serde(default)]
    original_sha256: Option<String>,
    installed_sha256: String,
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
        bot_vision_enabled: bot_vision_enabled_at(&csgo),
    };
    write_log("INFO", &format!("已检查 CS2 目录：{}。", status.root_path));
    Ok(status)
}

/// Check the Steam-owned gameinfo and any layered mods it explicitly declares.
/// Newer CS2 builds may merge former layers into the root gameinfo, so the
/// check must follow the current file's `LayeredOnMod` declarations instead
/// of requiring historical directories unconditionally.
pub fn ensure_core_game_layers(root: &Path) -> Result<(), AppError> {
    let root_relative = "game/csgo/gameinfo.gi";
    let root_path = root.join(root_relative);
    let bytes = match fs::read(&root_path) {
        Ok(bytes) if !bytes.is_empty() => bytes,
        Ok(_) => {
            return Err(AppError::runtime(format!(
                "[CS2_CORE_LAYER_MISSING] CS2 官方根 gameinfo 文件为空：{root_relative}。请退出助手，在 Steam 中验证 Counter-Strike 2 游戏文件后重试."
            )))
        }
        Err(_) => {
            return Err(AppError::runtime(format!(
                "[CS2_CORE_LAYER_MISSING] CS2 官方根 gameinfo 文件不可读：{root_relative}。请退出助手，在 Steam 中验证 Counter-Strike 2 游戏文件后重试."
            )))
        }
    };

    let text = String::from_utf8_lossy(&bytes);
    let mut unavailable = Vec::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        if parts.next() != Some("LayeredOnMod") {
            continue;
        }
        let Some(mod_name) = parts.next() else {
            unavailable.push("game/<未声明层>/gameinfo.gi".to_string());
            continue;
        };
        if mod_name.contains('/') || mod_name.contains('\\') || mod_name.contains('.') {
            unavailable.push(format!("game/{mod_name}/gameinfo.gi"));
            continue;
        }
        let relative = format!("game/{mod_name}/gameinfo.gi");
        match fs::read(root.join(&relative)) {
            Ok(bytes) if !bytes.is_empty() => {}
            Ok(_) => unavailable.push(format!("{relative}（空文件）")),
            Err(_) => unavailable.push(relative),
        }
    }
    if unavailable.is_empty() {
        return Ok(());
    }
    Err(AppError::runtime(format!(
        "[CS2_CORE_LAYER_MISSING] CS2 声明的官方游戏层不完整，无法安全启动。缺少或不可读：{}。请退出助手，在 Steam 中验证 Counter-Strike 2 游戏文件后重试。",
        unavailable.join("、")
    )))
}

pub fn check_cs2_process() -> Result<bool, AppError> {
    Ok(!list_cs2_processes()?.is_empty())
}

struct ManualCloseState {
    root_path: String,
    confirmed_at: i64,
    expires_at: i64,
    process_count: usize,
}
static MANUAL_CLOSE_OVERRIDE: OnceLock<Mutex<Option<ManualCloseState>>> = OnceLock::new();

fn normalized_override_root(root_path: &str) -> Result<String, AppError> {
    normalize_root(root_path).map(|path| path.to_string_lossy().to_lowercase())
}

fn manual_close_active(root_path: &str) -> Result<bool, AppError> {
    let root = normalized_override_root(root_path)?;
    let now = chrono::Utc::now().timestamp_millis();
    let state = MANUAL_CLOSE_OVERRIDE.get_or_init(|| Mutex::new(None));
    let mut guard = state
        .lock()
        .map_err(|_| AppError::runtime("[CS2_MANUAL_CONFIRM_STATE] 状态锁异常"))?;
    if guard
        .as_ref()
        .is_some_and(|value| value.expires_at <= now || value.root_path != root)
    {
        *guard = None;
    }
    Ok(guard.is_some())
}

pub fn confirm_cs2_closed(root_path: &str) -> Result<Cs2CloseOverride, AppError> {
    let root = normalized_override_root(root_path)?;
    let snapshot = get_cs2_process_snapshot()?;
    let confirmed_at = chrono::Utc::now().timestamp_millis();
    let expires_at = confirmed_at + 5 * 60 * 1000;
    let state = MANUAL_CLOSE_OVERRIDE.get_or_init(|| Mutex::new(None));
    *state
        .lock()
        .map_err(|_| AppError::runtime("[CS2_MANUAL_CONFIRM_STATE] 状态锁异常"))? =
        Some(ManualCloseState {
            root_path: root.clone(),
            confirmed_at,
            expires_at,
            process_count: snapshot.processes.len(),
        });
    write_runtime_log(
        "WARN",
        &format!(
            "[CS2_MANUAL_CONFIRM] root={} process_count={} expires_at={}。",
            root,
            snapshot.processes.len(),
            expires_at
        ),
    );
    Ok(Cs2CloseOverride {
        confirmed_at,
        expires_at,
        root_path: root,
        process_count: snapshot.processes.len(),
        active: true,
    })
}

pub fn revoke_cs2_closed_confirmation(root_path: &str) -> Result<(), AppError> {
    let root = normalized_override_root(root_path)?;
    if let Some(state) = MANUAL_CLOSE_OVERRIDE.get() {
        if let Ok(mut guard) = state.lock() {
            if guard.as_ref().is_some_and(|value| value.root_path == root) {
                *guard = None;
            }
        }
    }
    Ok(())
}

pub fn manual_close_override(root_path: &str) -> Result<Option<Cs2CloseOverride>, AppError> {
    let root = normalized_override_root(root_path)?;
    let now = chrono::Utc::now().timestamp_millis();
    let state = MANUAL_CLOSE_OVERRIDE.get_or_init(|| Mutex::new(None));
    let mut guard = state
        .lock()
        .map_err(|_| AppError::runtime("[CS2_MANUAL_CONFIRM_STATE] 状态锁异常"))?;
    if guard
        .as_ref()
        .is_some_and(|value| value.expires_at <= now || value.root_path != root)
    {
        *guard = None;
    }
    Ok(guard.as_ref().map(|value| Cs2CloseOverride {
        confirmed_at: value.confirmed_at,
        expires_at: value.expires_at,
        root_path: value.root_path.clone(),
        process_count: value.process_count,
        active: true,
    }))
}

pub fn check_cs2_process_for_write(root_path: &str) -> Result<bool, AppError> {
    Ok(check_cs2_process()? && !manual_close_active(root_path)?)
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
    let signal_processes = |processes: &[Cs2ProcessInfo]| -> Result<(), AppError> {
        for process in processes {
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
        Ok(())
    };
    signal_processes(&snapshot.processes)?;
    let deadline =
        std::time::Instant::now() + std::time::Duration::from_secs(if force { 5 } else { 2 });
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
        if force {
            // A launcher or crash reporter can recreate cs2.exe while the first
            // termination pass is in flight. Sweep every current PID again.
            signal_processes(&current)?;
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
                message: if force {
                    "CS2 强制关闭后仍在运行。"
                } else {
                    "CS2 未能在 2 秒内退出，将自动执行强制关闭。"
                }
                .into(),
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
    ensure_cs2_not_running(root_path)?;
    let root = normalize_root(root_path)?;
    let destination = root.join("game").join("csgo");
    if !destination.is_dir() {
        return Err(AppError::runtime(format!(
            "[INSTALL_TARGET_MISSING]\n未找到 CS2 游戏目录：{}",
            destination.display()
        )));
    }

    let zip_path = resolve_zip_path(app)?;
    let retained_backup = install_game_files_transactionally(&zip_path, &destination, keep_backup)?;
    let vision_enabled = bot_vision_enabled_at(&destination);
    apply_bot_vision_state(&destination, vision_enabled)?;

    write_log(
        "INFO",
        &format!(
            "基于上游 v1.4.5 的插件资源包已安装到 {}。",
            destination.display()
        ),
    );
    Ok(OperationResult {
        success: true,
        message: format!(
            "基于上游 CS2-Bot-Improver v1.4.5 的资源包已安装。\n目标目录：{}\n体积烟：{}。如一场游戏出现卡顿，可在安装页取消勾选体积烟功能。",
            destination.display(),
            if vision_enabled { "已启用" } else { "已关闭" }
        ) + &retained_backup.map_or_else(String::new, |path| format!("\n本次写前备份已保留：{}", path.display())),
    })
}

pub fn open_upstream_panel(app: &AppHandle) -> Result<OperationResult, AppError> {
    let zip_path = resolve_zip_path(app)?;
    let tool_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法确定应用数据目录：{error}")))?
        .join("tools")
        .join("official-panel-v1.4.5");
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
        message: format!("已启动官方 Panel v1.4.5。\n{}", panel_path.display()),
    })
}

pub fn uninstall_bot_package(root_path: &str) -> Result<OperationResult, AppError> {
    ensure_cs2_not_running(root_path)?;
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
            "已安全处理插件文件：{} 项。\n已按安装 ownership 恢复 Steam 官方文件，并保留 CS2 核心文件、官方 cfg 与 Inventory Simulator。",
            removed
        ),
    })
}

pub fn set_bot_vision_enabled(
    app: &AppHandle,
    root_path: &str,
    enabled: bool,
) -> Result<OperationResult, AppError> {
    ensure_cs2_not_running(root_path)?;
    let root = normalize_root(root_path)?;
    let csgo = root.join("game").join("csgo");
    if !csgo.is_dir() {
        return Err(AppError::runtime(
            "未找到 CS2 游戏目录，无法切换体积烟功能。",
        ));
    }
    if enabled {
        let zip_path = resolve_zip_path(app)?;
        restore_bot_vision_vdf(&zip_path, &csgo)?;
    }
    apply_bot_vision_state(&csgo, enabled)?;
    write_log(
        "INFO",
        &format!(
            "体积烟功能已{}：{}。",
            if enabled { "启用" } else { "关闭" },
            csgo.display()
        ),
    );
    Ok(OperationResult {
        success: true,
        message: if enabled {
            "体积烟功能已启用；下次启动 BOT 对局会加载 BotVision。".into()
        } else {
            "体积烟功能已关闭；已删除 BotVision.vdf，下次启动 BOT 对局将减少一项视觉扩展。".into()
        },
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
    let mut candidates = vec![
        resource_dir.join(BUNDLED_ZIP_NAME),
        resource_dir.join("resources").join(BUNDLED_ZIP_NAME),
        resource_dir.join("../resources").join(BUNDLED_ZIP_NAME),
        resource_dir.join("../").join(BUNDLED_ZIP_NAME),
    ];
    // Windows bundle layouts have changed between Tauri releases. Search only
    // the resource tree (and only a few levels) so a stale unrelated ZIP can
    // never become an install source.
    collect_bundled_zip_candidates(&resource_dir, 3, &mut candidates);
    let mut rejected = Vec::new();
    if let Some(path) = select_verified_zip_candidate(candidates, &mut rejected) {
        write_runtime_log(
            "INFO",
            &format!("已定位并验证上游 v1.4.5 资源包：{}。", path.display()),
        );
        return Ok(path);
    }

    // Tauri upgrades can leave an old resources/CS2BotImprover.zip beside the
    // new executable. Never select a merely readable archive: repair the
    // app-owned fallback cache from the ZIP embedded in this exact build.
    let fallback_root = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法定位应用数据目录：{error}")))?
        .join("bundled-resources");
    fs::create_dir_all(&fallback_root).map_err(io_error)?;
    let fallback = fallback_root.join(BUNDLED_ZIP_NAME);
    if verify_custom_zip(&fallback).is_ok() {
        write_runtime_log(
            "INFO",
            &format!("使用已验证的上游 v1.4.5 资源缓存：{}。", fallback.display()),
        );
        return Ok(fallback);
    }

    materialize_embedded_zip(&fallback)?;
    if let Err(error) = verify_custom_zip(&fallback) {
        return Err(AppError::runtime(format!(
            "[BUNDLED_RESOURCE_INVALID] 已从当前版本内嵌资源重建缓存，但完整校验仍未通过：{}\n资源路径：{}",
            error.into_string(),
            fallback.display()
        )));
    }
    write_runtime_log(
        "WARN",
        &format!(
            "已跳过 {} 个旧版/无效资源包，并从当前版本内嵌资源修复缓存：{}。",
            rejected.len(),
            fallback.display()
        ),
    );
    Ok(fallback)
}

fn select_verified_zip_candidate(
    candidates: impl IntoIterator<Item = PathBuf>,
    rejected: &mut Vec<String>,
) -> Option<PathBuf> {
    let mut seen = std::collections::HashSet::new();
    for path in candidates {
        let Ok(path) = dunce::canonicalize(&path) else {
            continue;
        };
        if !seen.insert(path.clone()) || !path.is_file() {
            continue;
        }
        match verify_custom_zip(&path) {
            Ok(()) => return Some(path),
            Err(error) => {
                let reason = error.into_string();
                write_runtime_log(
                    "WARN",
                    &format!(
                        "跳过未通过上游 v1.4.5 校验的资源包：{}；{}。",
                        path.display(),
                        reason
                    ),
                );
                rejected.push(format!("{}（{}）", path.display(), reason));
            }
        }
    }
    None
}

fn materialize_embedded_zip(destination: &Path) -> Result<(), AppError> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let temporary = destination.with_file_name(format!(
        ".{BUNDLED_ZIP_NAME}.{}.{}.tmp",
        std::process::id(),
        nonce
    ));
    let previous = destination.with_file_name(format!(
        ".{BUNDLED_ZIP_NAME}.{}.{}.previous",
        std::process::id(),
        nonce
    ));
    fs::write(&temporary, EMBEDDED_BUNDLED_ZIP).map_err(io_error)?;
    if let Err(error) = verify_custom_zip(&temporary) {
        let _ = fs::remove_file(&temporary);
        return Err(AppError::runtime(format!(
            "[BUNDLED_RESOURCE_INVALID] 当前版本内嵌的上游 v1.4.5 资源无法通过校验：{}",
            error.into_string()
        )));
    }
    let had_previous = destination.is_file();
    if had_previous {
        fs::rename(destination, &previous).map_err(io_error)?;
    }
    if let Err(error) = fs::rename(&temporary, destination) {
        let _ = fs::remove_file(&temporary);
        if had_previous {
            let _ = fs::rename(&previous, destination);
        }
        return Err(AppError::runtime(format!(
            "[BUNDLED_RESOURCE_REPAIR_FAILED] 无法写入已校验的上游资源缓存：{error}"
        )));
    }
    if had_previous {
        let _ = fs::remove_file(previous);
    }
    Ok(())
}

fn collect_bundled_zip_candidates(root: &Path, depth: usize, output: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case(BUNDLED_ZIP_NAME))
        {
            output.push(path);
        } else if path.is_dir() {
            collect_bundled_zip_candidates(&path, depth - 1, output);
        }
    }
}

// Kept as a small compatibility probe for diagnostics and release-contract
// tests. Selection itself uses verify_custom_zip so a readable ZIP without
// the exact Panel v1.4.5 is never accepted.
fn zip_has_root_manifest(path: &Path) -> bool {
    let Ok(file) = File::open(path) else {
        return false;
    };
    let Ok(mut archive) = ZipArchive::new(file) else {
        return false;
    };
    find_zip_entry_index(&mut archive, "gameinfo.manifest.json").is_some()
}

fn find_zip_entry_index<R: Read + io::Seek>(
    archive: &mut ZipArchive<R>,
    expected: &str,
) -> Option<usize> {
    (0..archive.len()).find(|index| {
        archive
            .by_index(*index)
            .ok()
            .and_then(|entry| safe_zip_path(entry.name()).ok())
            .is_some_and(|path| path == Path::new(expected))
    })
}

fn is_panel_entry(name: &str) -> bool {
    safe_zip_path(name)
        .map(|path| path == Path::new(PANEL_FILE_NAME))
        .unwrap_or(false)
}

fn read_zip_entry<R: Read + io::Seek>(
    archive: &mut ZipArchive<R>,
    expected: &str,
) -> Result<Vec<u8>, AppError> {
    let index = find_zip_entry_index(archive, expected)
        .ok_or_else(|| AppError::runtime(format!("缺少资源条目：{expected}")))?;
    let mut entry = archive
        .by_index(index)
        .map_err(|error| AppError::runtime(format!("无法读取资源条目 {expected}：{error}")))?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes).map_err(io_error)?;
    Ok(bytes)
}

/// Materialize the SkinOnly gameinfo for installations created before the
/// resource was added to the bundled ZIP. This intentionally touches only the
/// new backup file and leaves player BOT profiles and active gameinfo intact.
pub fn ensure_skin_only_gameinfo(app: &AppHandle, root_path: &str) -> Result<(), AppError> {
    let root = normalize_root(root_path)?;
    let csgo = root.join("game").join("csgo");
    if !csgo.is_dir() {
        return Err(AppError::runtime(format!(
            "[SKIN_ONLY_TARGET_MISSING] 未找到 CS2 游戏目录：{}",
            csgo.display()
        )));
    }
    let zip_path = resolve_zip_path(app)?;
    verify_custom_zip(&zip_path)?;
    let file = File::open(&zip_path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file).map_err(|error| {
        AppError::runtime(format!(
            "[SKIN_ONLY_RESOURCE_INVALID] 无法读取内置资源包：{error}"
        ))
    })?;
    let bytes = read_zip_entry(&mut archive, SKIN_ONLY_GAMEINFO_ENTRY).map_err(|_| {
        AppError::runtime("[SKIN_ONLY_RESOURCE_INVALID] 内置资源缺少 SkinOnly gameinfo。")
    })?;
    let target = csgo.join(SKIN_ONLY_GAMEINFO_ENTRY);
    if target.is_file() && fs::read(&target).ok().as_deref() == Some(bytes.as_slice()) {
        return Ok(());
    }
    let parent = target
        .parent()
        .ok_or_else(|| AppError::runtime("[SKIN_ONLY_MIGRATION_FAILED] SkinOnly 目标路径无效。"))?;
    fs::create_dir_all(parent).map_err(io_error)?;
    let temporary = parent.join(format!(".gameinfo.gi.skin-only-{}.tmp", std::process::id()));
    let backup = parent.join(format!(".gameinfo.gi.skin-only-{}.bak", std::process::id()));
    fs::write(&temporary, &bytes).map_err(io_error)?;
    let had_target = target.is_file();
    if had_target {
        let _ = fs::remove_file(&backup);
        fs::rename(&target, &backup).map_err(io_error)?;
    }
    if let Err(error) = fs::rename(&temporary, &target) {
        let _ = fs::remove_file(&temporary);
        if had_target {
            let _ = fs::rename(&backup, &target);
        }
        return Err(AppError::runtime(format!(
            "[SKIN_ONLY_MIGRATION_FAILED] 无法写入 SkinOnly gameinfo：{error}"
        )));
    }
    let _ = fs::remove_file(&backup);
    if fs::read(&target).ok().as_deref() != Some(bytes.as_slice()) {
        return Err(AppError::runtime(
            "[SKIN_ONLY_MIGRATION_FAILED] SkinOnly gameinfo 写入后校验不一致。",
        ));
    }
    write_log(
        "INFO",
        &format!(
            "已为既有 CS2 安装补齐 SkinOnly gameinfo：{}。",
            target.display()
        ),
    );
    Ok(())
}

fn verify_custom_zip(path: &Path) -> Result<(), AppError> {
    let file = File::open(path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| AppError::runtime(format!("无法读取内置资源包：{error}")))?;
    // The archive is intentionally the byte-for-byte Windows release from
    // CS2-Bot-Improver v1.4.5. Reject downstream add-ons here so a stale local
    // package cannot silently reintroduce MapRotation or CS2BotLlmChat.
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| AppError::runtime(format!("无法读取资源条目：{error}")))?;
        let relative = safe_zip_path(entry.name())?;
        if OBSOLETE_DOWNSTREAM_COMPONENTS.iter().any(|component| {
            relative == Path::new(component) || relative.starts_with(Path::new(component))
        }) {
            return Err(AppError::runtime(format!(
                "[UPSTREAM_ARCHIVE_DOWNSTREAM_COMPONENT] 官方 v1.4.5 资源包不应包含下游组件：{}。请使用原始上游整包。",
                entry.name()
            )));
        }
    }
    let manifest = find_zip_entry_index(&mut archive, "gameinfo.manifest.json")
        .and_then(|_| read_zip_entry(&mut archive, "gameinfo.manifest.json").ok())
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .filter(|value| {
            value.get("schema").and_then(serde_json::Value::as_u64) == Some(1)
                && value
                    .get("entries")
                    .and_then(serde_json::Value::as_object)
                    .is_some()
        });
    let marker = if find_zip_entry_index(&mut archive, PLUGIN_MARKER).is_some() {
        let bytes = read_zip_entry(&mut archive, PLUGIN_MARKER)?;
        Some(
            serde_json::from_slice::<PluginMarker>(&bytes)
                .map_err(|_| AppError::runtime("[BOT_PLUGIN_PAYLOAD_INVALID] marker 无法解析。"))?,
        )
    } else {
        None
    };
    for name in [
        "gameinfo.gi",
        "backup/Online/gameinfo.gi",
        "backup/WithBots/gameinfo.gi",
    ] {
        let bytes = read_zip_entry(&mut archive, name)
            .map_err(|_| AppError::runtime("[GAMEINFO_ASSET_INVALID] gameinfo 条目缺失。"))?;
        if let Some(manifest) = &manifest {
            let expected = manifest["entries"][name]["sha256"]
                .as_str()
                .unwrap_or_default();
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            if format!("{:X}", hasher.finalize()) != expected {
                return Err(AppError::runtime(format!(
                    "[GAMEINFO_ASSET_INVALID] {name} 摘要不匹配。"
                )));
            }
        }
    }
    for name in UPSTREAM_CORE_ENTRIES {
        let bytes = read_zip_entry(&mut archive, name).map_err(|_| {
            AppError::runtime(format!(
                "[UPSTREAM_ARCHIVE_INVALID] 上游 v1.4.5 资源缺少核心文件：{name}。"
            ))
        })?;
        if bytes.is_empty() {
            return Err(AppError::runtime(format!(
                "[UPSTREAM_ARCHIVE_INVALID] 上游 v1.4.5 核心文件为空：{name}。"
            )));
        }
    }
    let panel = read_zip_entry(&mut archive, PANEL_FILE_NAME)
        .map_err(|_| {
            AppError::runtime(format!(
                "[PANEL_ASSET_INVALID] 资源包缺少官方 Panel v1.4.5：{}。旧版 Panel v1.4.4 不能代替此文件。",
                path.display()
            ))
        })?;
    let mut panel_hash = Sha256::new();
    panel_hash.update(&panel);
    if panel.len() as u64 != PANEL_SIZE || format!("{:X}", panel_hash.finalize()) != PANEL_SHA256 {
        return Err(AppError::runtime(
            "[PANEL_HASH_INVALID] 上游资源中的 Panel v1.4.5 摘要不匹配。",
        ));
    }
    let _ = marker;
    Ok(())
}

pub fn inspect_bot_plugin_version(root_path: &str) -> Result<PluginVersionStatus, AppError> {
    let root = normalize_root(root_path)?;
    inspect_bot_plugin_version_at(&root.join("game/csgo"))
}

pub fn ensure_bot_plugin_current(app: &AppHandle, root_path: &str) -> Result<String, AppError> {
    ensure_cs2_not_running(root_path)?;
    let root = normalize_root(root_path)?;
    let destination = root.join("game/csgo");
    let zip_path = resolve_zip_path(app)?;
    let zip_hash = sha256_file(&zip_path)?;
    let _ = install_game_files_transactionally(&zip_path, &destination, false)?;
    apply_bot_vision_state(&destination, bot_vision_enabled_at(&destination))?;
    match inspect_bot_plugin_version_at(&destination)? {
        PluginVersionStatus::Valid { version } => {
            write_log(
                "INFO",
                &format!("BOT 插件资源已安装，使用 marker 版本 {version}。"),
            );
            Ok(version.to_string())
        }
        PluginVersionStatus::Invalid { reason, .. } => Err(AppError::runtime(format!(
            "[BOT_PLUGIN_AUTO_INSTALL_FAILED] 自动安装后插件校验失败：{reason}\ninstalledVersion=invalid\nzipSha256={zip_hash}\nmarkerPath={}", destination.join(PLUGIN_MARKER).display()
        ))),
        PluginVersionStatus::Missing => {
            write_log("INFO", "上游 v1.4.5 原样资源包未包含下游 marker，安装完成。");
            Ok("1.4.5".to_string())
        }
    }
}

fn bot_vision_enabled_at(csgo: &Path) -> bool {
    let vdf = csgo.join("addons/metamod/BotVision.vdf");
    if let Ok(state) = fs::read_to_string(csgo.join(BOT_VISION_STATE_FILE)) {
        return state.trim() != "disabled";
    }
    vdf.is_file()
}

fn apply_bot_vision_state(csgo: &Path, enabled: bool) -> Result<(), AppError> {
    let vdf = csgo.join(BOT_VISION_VDF);
    if enabled {
        if !vdf.is_file() {
            return Err(AppError::runtime(
                "资源包中未找到 BotVision.vdf，无法启用体积烟功能。",
            ));
        }
        fs::write(csgo.join(BOT_VISION_STATE_FILE), b"enabled\n").map_err(io_error)?;
    } else {
        if vdf.is_file() {
            fs::remove_file(&vdf).map_err(io_error)?;
        }
        if let Some(parent) = csgo.join(BOT_VISION_STATE_FILE).parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        fs::write(csgo.join(BOT_VISION_STATE_FILE), b"disabled\n").map_err(io_error)?;
    }
    Ok(())
}

fn restore_bot_vision_vdf(zip_path: &Path, csgo: &Path) -> Result<(), AppError> {
    let file = File::open(zip_path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| AppError::runtime(format!("无法读取资源包：{error}")))?;
    let bytes = read_zip_entry(&mut archive, BOT_VISION_VDF)
        .map_err(|_| AppError::runtime("上游 v1.4.5 资源包缺少 BotVision.vdf。"))?;
    let target = csgo.join(BOT_VISION_VDF);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    let temporary = target.with_extension("vdf.cs2as05.tmp");
    fs::write(&temporary, bytes).map_err(io_error)?;
    if target.is_file() {
        fs::remove_file(&target).map_err(io_error)?;
    }
    fs::rename(temporary, target).map_err(io_error)
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
    for required in [
        "addons/counterstrikesharp/plugins/BotState/BotState.dll",
        "addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll",
    ] {
        if !csgo.join(required).is_file() {
            return Ok(PluginVersionStatus::Invalid {
                reason: format!("[BOT_PLUGIN_CORE_MISSING] 缺少关键插件：{required}"),
                version: Some(version),
            });
        }
    }
    if !csgo.join("addons/counterstrikesharp/plugins").is_dir() {
        return Ok(PluginVersionStatus::Invalid {
            reason: "[BOT_PLUGIN_CORE_MISSING] CounterStrikeSharp 插件目录缺失。".into(),
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
    let mut moved_obsolete: Vec<(PathBuf, PathBuf)> = Vec::new();
    let result = (|| -> Result<(), AppError> {
        fs::create_dir_all(&staging).map_err(io_error)?;
        extract_game_files(zip_path, &staging)?;
        match inspect_bot_plugin_version_at(&staging)? {
            PluginVersionStatus::Valid { .. } | PluginVersionStatus::Missing => {}
            PluginVersionStatus::Invalid { reason, .. } => {
                return Err(AppError::runtime(format!(
                    "[BOT_PLUGIN_AUTO_INSTALL_FAILED] 内置包标记无效：{reason}"
                )))
            }
        }
        let preferences = panel::capture_panel_preferences(destination)?;
        let files = collect_game_file_entries(zip_path)?;
        // A malformed or tampered ownership ledger must stop the write before
        // any game file is changed. A missing ledger is valid for a first
        // install and will be created after the transaction succeeds.
        let _ = load_install_ledger(destination)?;
        move_obsolete_components(destination, &backup, &mut moved_obsolete)?;
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
        let skin_only = destination.join(SKIN_ONLY_GAMEINFO_ENTRY);
        if !skin_only.is_file() {
            let file = File::open(zip_path).map_err(io_error)?;
            let mut archive = ZipArchive::new(file)
                .map_err(|error| AppError::runtime(format!("无法读取资源包：{error}")))?;
            let bytes =
                read_zip_entry(&mut archive, "backup/WithBots/gameinfo.gi").map_err(|error| {
                    AppError::runtime(format!("无法读取兼容 gameinfo：{}", error.into_string()))
                })?;
            if let Some(parent) = skin_only.parent() {
                fs::create_dir_all(parent).map_err(io_error)?;
            }
            fs::write(&skin_only, bytes).map_err(io_error)?;
        }
        panel::write_gameinfo_sidecar(destination, env!("CARGO_PKG_VERSION"), "2026-08-26")?;
        panel::restore_panel_preferences(destination, &preferences, preferences.is_empty())?;
        // The upstream Panel treats these cfg files as part of its install
        // payload. Restore them after preference migration so a reinstall
        // cannot leave an old or partially deleted cfg set behind.
        restore_upstream_panel_cfg_files(&staging, destination, &mut touched, &backup)?;
        verify_upstream_panel_files(destination)?;
        let mut ledger_paths = files.clone();
        for relative in [
            PathBuf::from("cfg/cs2as05-panel-state.json"),
            PathBuf::from("cfg/cs2as05-gameinfo-state.json"),
            PathBuf::from("gameinfo.gi.official.bin"),
        ] {
            if !ledger_paths.contains(&relative) {
                ledger_paths.push(relative);
            }
        }
        persist_install_ledger(destination, &backup, &ledger_paths)?;
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
        Err(error) => {
            let rollback_result = rollback_transaction(&touched, &backup, destination);
            let restore_result = restore_obsolete_components(&moved_obsolete);
            match (rollback_result, restore_result) {
                (Ok(()), Ok(())) => {
                    let _ = fs::remove_dir_all(&backup);
                    Err(error)
                }
                (Err(rollback_error), Ok(())) | (Ok(()), Err(rollback_error)) => {
                    Err(AppError::runtime(format!(
                        "{}\n[BOT_PLUGIN_ROLLBACK_FAILED] 回滚失败：{}\n备份保留于：{}",
                        error.into_string(),
                        rollback_error.into_string(),
                        backup.display()
                    )))
                }
                (Err(rollback_error), Err(restore_error)) => Err(AppError::runtime(format!(
                    "{}\n[BOT_PLUGIN_ROLLBACK_FAILED] 文件回滚失败：{}；旧组件恢复失败：{}\n备份保留于：{}",
                    error.into_string(),
                    rollback_error.into_string(),
                    restore_error.into_string(),
                    backup.display()
                ))),
            }
        }
    }
}

fn move_obsolete_components(
    destination: &Path,
    backup: &Path,
    moved: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), AppError> {
    for relative in OBSOLETE_DOWNSTREAM_COMPONENTS {
        let target = destination.join(relative);
        if !target.exists() {
            continue;
        }
        let saved = backup.join("obsolete").join(relative);
        if let Some(parent) = saved.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        if saved.exists() {
            if saved.is_dir() {
                fs::remove_dir_all(&saved).map_err(io_error)?;
            } else {
                fs::remove_file(&saved).map_err(io_error)?;
            }
        }
        fs::rename(&target, &saved).map_err(|error| {
            AppError::runtime(format!(
                "[OBSOLETE_COMPONENT_REMOVE_FAILED] 无法暂存旧下游组件 {}：{error}",
                target.display()
            ))
        })?;
        moved.push((target, saved));
    }
    Ok(())
}

fn restore_obsolete_components(moved: &[(PathBuf, PathBuf)]) -> Result<(), AppError> {
    for (target, saved) in moved.iter().rev() {
        if !saved.exists() {
            continue;
        }
        if target.exists() {
            if target.is_dir() {
                fs::remove_dir_all(target).map_err(io_error)?;
            } else {
                fs::remove_file(target).map_err(io_error)?;
            }
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        fs::rename(saved, target).map_err(io_error)?;
    }
    Ok(())
}

fn restore_upstream_panel_cfg_files(
    staging: &Path,
    destination: &Path,
    touched: &mut Vec<(PathBuf, bool)>,
    backup: &Path,
) -> Result<(), AppError> {
    for relative_text in UPSTREAM_PANEL_CFG_FILES {
        let relative = Path::new(relative_text);
        let source = staging.join(relative);
        let target = destination.join(relative);
        if !source.is_file() {
            return Err(AppError::runtime(format!(
                "[UPSTREAM_PANEL_PAYLOAD_MISSING] 资源包缺少上游 Panel 文件：{relative_text}"
            )));
        }
        let existed = target.is_file();
        if existed {
            let saved = backup.join(relative);
            // The main archive loop already captured the pre-install bytes.
            // Do not overwrite that snapshot with the first package copy when
            // the Panel cfg pass runs a second time.
            if !saved.is_file() {
                if let Some(parent) = saved.parent() {
                    fs::create_dir_all(parent).map_err(io_error)?;
                }
                fs::copy(&target, &saved).map_err(io_error)?;
            }
        }
        touched.push((relative.to_path_buf(), existed));
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        fs::copy(source, target).map_err(io_error)?;
    }
    Ok(())
}

fn verify_upstream_panel_files(destination: &Path) -> Result<(), AppError> {
    let missing = UPSTREAM_PANEL_REQUIRED_FILES
        .iter()
        .filter(|relative| !destination.join(relative).is_file())
        .copied()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Ok(());
    }
    Err(AppError::runtime(format!(
        "[UPSTREAM_PANEL_FILES_MISSING] 安装后上游 Panel 仍缺少文件：{}。请重新安装完整 v1.4.5 资源包。",
        missing.join(", ")
    )))
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
        if entry.is_dir() || is_panel_entry(entry.name()) {
            continue;
        }
        files.push(safe_zip_path(entry.name())?);
    }
    if find_zip_entry_index(&mut archive, "backup/WithBots/gameinfo.gi").is_some()
        && find_zip_entry_index(&mut archive, SKIN_ONLY_GAMEINFO_ENTRY).is_none()
    {
        files.push(PathBuf::from(SKIN_ONLY_GAMEINFO_ENTRY));
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
    let has_skin_only = find_zip_entry_index(&mut archive, SKIN_ONLY_GAMEINFO_ENTRY).is_some();
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| AppError::runtime(format!("无法读取资源条目：{error}")))?;
        if is_panel_entry(entry.name()) {
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
    if !has_skin_only {
        let bytes =
            read_zip_entry(&mut archive, "backup/WithBots/gameinfo.gi").map_err(|error| {
                AppError::runtime(format!("无法读取兼容 gameinfo：{}", error.into_string()))
            })?;
        let target = destination.join(SKIN_ONLY_GAMEINFO_ENTRY);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        fs::write(target, bytes).map_err(io_error)?;
    }
    Ok(())
}

fn extract_panel_atomically(
    zip_path: &Path,
    tool_dir: &Path,
    panel_path: &Path,
) -> Result<(), AppError> {
    fs::create_dir_all(tool_dir).map_err(io_error)?;
    let temporary = tool_dir.join("Panel-v1.4.5.tmp");
    let file = File::open(zip_path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| AppError::runtime(format!("无法读取资源包：{error}")))?;
    let bytes = read_zip_entry(&mut archive, PANEL_FILE_NAME).map_err(|_| {
        AppError::runtime(format!(
            "[PANEL_ASSET_INVALID] 无法从已验证资源包提取官方 Panel v1.4.5：{}。",
            zip_path.display()
        ))
    })?;
    let mut output = File::create(&temporary).map_err(io_error)?;
    if let Err(error) = output.write_all(&bytes).and_then(|_| output.flush()) {
        let _ = fs::remove_file(&temporary);
        return Err(io_error(error));
    }
    drop(output);
    if !panel_is_valid(&temporary)? {
        let _ = fs::remove_file(&temporary);
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

fn install_ledger_path(csgo: &Path) -> PathBuf {
    csgo.join(INSTALL_LEDGER_RELATIVE)
}

fn install_backup_root(csgo: &Path) -> PathBuf {
    csgo.join(INSTALL_BACKUP_DIR_RELATIVE)
}

fn validate_install_ledger_relative(value: &str) -> Result<PathBuf, AppError> {
    let relative = safe_zip_path(value)?;
    if relative.as_os_str().is_empty()
        || relative == Path::new(INSTALL_LEDGER_RELATIVE)
        || relative.starts_with(Path::new(INSTALL_BACKUP_DIR_RELATIVE))
    {
        return Err(AppError::runtime(format!(
            "[BOT_PLUGIN_LEDGER_INVALID] ownership 清单包含受保护或空路径：{value}"
        )));
    }
    Ok(relative)
}

fn load_install_ledger(csgo: &Path) -> Result<Option<InstallLedger>, AppError> {
    let path = install_ledger_path(csgo);
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(io_error)?;
    let ledger = serde_json::from_slice::<InstallLedger>(&bytes).map_err(|error| {
        AppError::runtime(format!(
            "[BOT_PLUGIN_LEDGER_INVALID] 无法解析卸载 ownership 清单 {}：{error}",
            path.display()
        ))
    })?;
    if ledger.schema != INSTALL_LEDGER_SCHEMA {
        return Err(AppError::runtime(format!(
            "[BOT_PLUGIN_LEDGER_INVALID] 不支持的 ownership 清单版本：{}。请重新安装插件后再卸载。",
            ledger.schema
        )));
    }
    let mut seen = BTreeSet::new();
    for entry in &ledger.entries {
        let relative = validate_install_ledger_relative(&entry.relative)?;
        let key = relative.to_string_lossy().to_string();
        if !seen.insert(key) {
            return Err(AppError::runtime(format!(
                "[BOT_PLUGIN_LEDGER_INVALID] ownership 清单存在重复路径：{}",
                entry.relative
            )));
        }
        if entry.installed_sha256.is_empty()
            || (entry.existed_before
                && (entry.original_backup_relative.is_none() || entry.original_sha256.is_none()))
        {
            return Err(AppError::runtime(format!(
                "[BOT_PLUGIN_LEDGER_INVALID] ownership 清单条目不完整：{}",
                entry.relative
            )));
        }
        if let Some(original) = &entry.original_backup_relative {
            validate_install_ledger_relative(original)?;
        }
    }
    Ok(Some(ledger))
}

fn write_install_ledger(csgo: &Path, mut ledger: InstallLedger) -> Result<(), AppError> {
    ledger
        .entries
        .sort_by(|left, right| left.relative.cmp(&right.relative));
    let bytes = serde_json::to_vec_pretty(&ledger).map_err(|error| {
        AppError::runtime(format!(
            "[BOT_PLUGIN_LEDGER_WRITE] 无法序列化 ownership 清单：{error}"
        ))
    })?;
    let path = install_ledger_path(csgo);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    let temporary = path.with_file_name(format!(
        ".{}-tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("cs2as05-install-ledger.json"),
        std::process::id()
    ));
    fs::write(&temporary, bytes).map_err(io_error)?;
    if path.exists() {
        fs::remove_file(&path).map_err(io_error)?;
    }
    fs::rename(&temporary, &path).map_err(io_error)
}

fn persist_install_ledger(
    destination: &Path,
    transaction_backup: &Path,
    paths: &[PathBuf],
) -> Result<(), AppError> {
    let mut ledger = load_install_ledger(destination)?.unwrap_or(InstallLedger {
        schema: INSTALL_LEDGER_SCHEMA,
        entries: Vec::new(),
    });
    let persistent_backup = install_backup_root(destination);
    for relative in paths {
        let relative = validate_install_ledger_relative(&relative.to_string_lossy())?;
        let target = destination.join(&relative);
        if !target.is_file() {
            return Err(AppError::runtime(format!(
                "[BOT_PLUGIN_LEDGER_WRITE] 安装后缺少 ownership 文件：{}",
                target.display()
            )));
        }
        let installed_sha256 = sha256_file(&target)?;
        if let Some(existing) = ledger
            .entries
            .iter_mut()
            .find(|entry| entry.relative == relative.to_string_lossy())
        {
            if existing.existed_before {
                let Some(original) = existing.original_backup_relative.as_deref() else {
                    return Err(AppError::runtime(format!(
                        "[BOT_PLUGIN_LEDGER_INVALID] 原始备份路径缺失：{}",
                        existing.relative
                    )));
                };
                if !persistent_backup.join(original).is_file() {
                    return Err(AppError::runtime(format!(
                        "[BOT_PLUGIN_LEDGER_INVALID] 原始备份文件缺失：{}",
                        persistent_backup.join(original).display()
                    )));
                }
            }
            existing.installed_sha256 = installed_sha256;
            continue;
        }

        let transaction_original = transaction_backup.join(&relative);
        let existed_before = transaction_original.is_file();
        let original_backup_relative =
            existed_before.then(|| relative.to_string_lossy().to_string());
        let original_sha256 = if existed_before {
            let saved = persistent_backup.join(&relative);
            if let Some(parent) = saved.parent() {
                fs::create_dir_all(parent).map_err(io_error)?;
            }
            fs::copy(&transaction_original, &saved).map_err(io_error)?;
            Some(sha256_file(&saved)?)
        } else {
            None
        };
        ledger.entries.push(InstallLedgerEntry {
            relative: relative.to_string_lossy().to_string(),
            existed_before,
            original_backup_relative,
            original_sha256,
            installed_sha256,
        });
    }
    write_install_ledger(destination, ledger)
}

fn restore_install_ledger(csgo: &Path, ledger: InstallLedger) -> Result<(usize, usize), AppError> {
    let persistent_backup = install_backup_root(csgo);
    let mut remaining = Vec::new();
    let mut restored = 0;
    let mut skipped = 0;
    for entry in ledger.entries {
        let relative = validate_install_ledger_relative(&entry.relative)?;
        let target = csgo.join(&relative);
        let current_sha256 = if target.is_file() {
            Some(sha256_file(&target)?)
        } else {
            None
        };
        let current_is_owned = current_sha256.as_deref() == Some(entry.installed_sha256.as_str());
        if entry.existed_before {
            let Some(original) = entry.original_backup_relative.as_deref() else {
                return Err(AppError::runtime(format!(
                    "[BOT_PLUGIN_LEDGER_INVALID] 原始备份路径缺失：{}",
                    entry.relative
                )));
            };
            let saved = persistent_backup.join(validate_install_ledger_relative(original)?);
            if !saved.is_file() {
                return Err(AppError::runtime(format!(
                    "[BOT_PLUGIN_LEDGER_INVALID] 原始备份文件缺失：{}",
                    saved.display()
                )));
            }
            if let Some(expected) = entry.original_sha256.as_deref() {
                let actual = sha256_file(&saved)?;
                if actual != expected {
                    return Err(AppError::runtime(format!(
                        "[BOT_PLUGIN_LEDGER_INVALID] 原始备份摘要不匹配：{}",
                        saved.display()
                    )));
                }
            }
            if !current_is_owned && target.exists() {
                // A player or another plugin changed this file after install.
                // Leave it in place and retain the ledger for a later, explicit
                // Steam verification instead of overwriting unrelated work.
                skipped += 1;
                remaining.push(entry);
                continue;
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(io_error)?;
            }
            fs::copy(&saved, &target).map_err(io_error)?;
            restored += 1;
        } else if target.is_file() {
            if current_is_owned {
                fs::remove_file(&target).map_err(io_error)?;
                restored += 1;
            } else {
                skipped += 1;
                remaining.push(entry);
            }
        }
    }

    let ledger_path = install_ledger_path(csgo);
    if skipped == 0 {
        if ledger_path.is_file() {
            fs::remove_file(&ledger_path).map_err(io_error)?;
        }
        if persistent_backup.is_dir() {
            fs::remove_dir_all(&persistent_backup).map_err(io_error)?;
        }
    } else {
        write_install_ledger(
            csgo,
            InstallLedger {
                schema: INSTALL_LEDGER_SCHEMA,
                entries: remaining,
            },
        )?;
    }
    Ok((restored, skipped))
}

fn remove_project_state_files(csgo: &Path) -> Result<usize, AppError> {
    let mut removed = 0;
    for relative in [BOT_VISION_STATE_FILE, "cfg/cs2as05-skin-only.state"] {
        let path = csgo.join(relative);
        if path.is_file() {
            fs::remove_file(path).map_err(io_error)?;
            removed += 1;
        }
    }
    Ok(removed)
}

fn remove_upstream_package(csgo: &Path) -> Result<usize, AppError> {
    if let Some(ledger) = load_install_ledger(csgo)? {
        let (restored, skipped) = restore_install_ledger(csgo, ledger)?;
        let removed_state = remove_project_state_files(csgo)?;
        if skipped > 0 {
            write_log(
                "WARN",
                &format!(
                    "卸载已恢复 {restored} 个 ownership 文件，保留 {skipped} 个被用户或其他插件改动的文件。"
                ),
            );
        }
        return Ok(restored + removed_state);
    }

    // Legacy 0.6.4 installations do not have an ownership ledger. Keep
    // Steam-owned cfg/gameinfo files untouched in this fallback path; the
    // user can run Steam validation once to repair any earlier modification.
    // Never use a broad delete on game/csgo, CounterStrikeSharp, or Inventory
    // Simulator when ownership evidence is unavailable.
    let paths = [
        "addons/BotController",
        "addons/BotHider",
        "addons/BotVision",
        "addons/counterstrikesharp/plugins/BotAI",
        "addons/counterstrikesharp/plugins/BotAimImprover",
        "addons/counterstrikesharp/plugins/BotBuy",
        "addons/counterstrikesharp/plugins/BotControllerImpl",
        "addons/counterstrikesharp/plugins/BotHiderImpl",
        "addons/counterstrikesharp/plugins/BotRandomizer",
        "addons/counterstrikesharp/plugins/BotState",
        "addons/counterstrikesharp/plugins/NadeSystem",
        "addons/counterstrikesharp/plugins/RoundDamageRecap",
        "overrides/Low",
        "overrides/Medium",
        "overrides/High",
    ]
    .into_iter()
    .map(|relative| csgo.join(relative))
    .collect::<Vec<_>>();
    let files = [
        "overrides/botprofile.vpk",
        "addons/metamod/BotController.vdf",
        "addons/metamod/BotHider.vdf",
        "addons/metamod/BotVision.vdf",
        "addons/metamod/RayTrace.vdf",
        "addons/metamod/counterstrikesharp.vdf",
        "addons/metamod/metaplugins.ini",
        "metamod.vdf",
        "metamod_x64.vdf",
    ]
    .into_iter()
    .map(|relative| csgo.join(relative))
    .collect::<Vec<_>>();
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
    removed += remove_project_state_files(csgo)?;
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
    let normalized = name.trim_start_matches("./");
    let path = Path::new(normalized);
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

pub(crate) fn ensure_cs2_not_running(root_path: &str) -> Result<(), AppError> {
    if check_cs2_process_for_write(root_path)? {
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
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn write_stale_panel_zip(path: &Path) {
        let file = File::create(path).unwrap();
        let mut archive = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        let mut entries = vec![
            "gameinfo.gi",
            "backup/Online/gameinfo.gi",
            "backup/WithBots/gameinfo.gi",
            "Panel v1.4.4.exe",
        ];
        entries.extend(UPSTREAM_CORE_ENTRIES.iter().copied());
        for name in entries {
            archive.start_file(name, options).unwrap();
            archive.write_all(b"stale-test-payload").unwrap();
        }
        archive.finish().unwrap();
    }

    #[test]
    fn matches_only_exact_cs2_process_names() {
        for name in ["cs2.exe", "CS2.EXE", "cs2", "cs2.exe\0\0"] {
            assert!(is_cs2_process_name(name), "expected {name:?} to match");
        }
        for name in ["steam.exe", "cs2.exe.bak", "", "cs2 "] {
            assert!(!is_cs2_process_name(name), "expected {name:?} not to match");
        }
    }

    #[test]
    fn verified_resource_selection_rejects_stale_panel_and_continues() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let stale = std::env::temp_dir().join(format!(
            "cs2as05-stale-panel-{}-{nonce}.zip",
            std::process::id()
        ));
        write_stale_panel_zip(&stale);
        let current = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(BUNDLED_ZIP_NAME);
        let mut rejected = Vec::new();
        let selected =
            select_verified_zip_candidate(vec![stale.clone(), current.clone()], &mut rejected)
                .expect("candidate selection must continue to the valid v1.4.5 archive");
        assert_eq!(selected, dunce::canonicalize(current).unwrap());
        assert!(rejected
            .iter()
            .any(|entry| entry.contains("PANEL_ASSET_INVALID")));
        let _ = fs::remove_file(stale);
    }

    #[test]
    fn panel_entry_matching_normalizes_upstream_dot_prefix() {
        assert!(is_panel_entry(PANEL_FILE_NAME));
        assert!(is_panel_entry("./Panel v1.4.5.exe"));
        assert!(!is_panel_entry("Panel v1.4.4.exe"));
    }

    #[test]
    fn core_game_layer_preflight_requires_readable_steam_files() {
        let root =
            std::env::temp_dir().join(format!("core-game-layer-preflight-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("game/csgo")).unwrap();

        let error = ensure_core_game_layers(&root).unwrap_err().into_string();
        assert!(error.contains("CS2_CORE_LAYER_MISSING"));
        assert!(error.contains("game/csgo/gameinfo.gi"));

        fs::write(
            root.join("game/csgo/gameinfo.gi"),
            b"GameInfo\n{\n\tFileSystem\n\t{\n\t}\n}\n",
        )
        .unwrap();
        ensure_core_game_layers(&root)
            .expect("a current merged Steam gameinfo without layers must pass");

        fs::write(
            root.join("game/csgo/gameinfo.gi"),
            b"GameInfo\n{\n\tLayeredOnMod csgo_imported\n}\n",
        )
        .unwrap();
        let error = ensure_core_game_layers(&root).unwrap_err().into_string();
        assert!(error.contains("game/csgo_imported/gameinfo.gi"));
        fs::create_dir_all(root.join("game/csgo_imported")).unwrap();
        fs::write(root.join("game/csgo_imported/gameinfo.gi"), b"GameInfo").unwrap();
        ensure_core_game_layers(&root).expect("declared readable Steam layer must pass");
        fs::remove_dir_all(root).unwrap();
    }

    fn write_test_marker(csgo: &Path, version: &str) {
        let entries = vec![
            "addons/counterstrikesharp/plugins/BotState/BotState.dll",
            "addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll",
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
            "components": [{"id": "cs2as05-custom-package"}],
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
    fn plugin_marker_allows_payload_customization_but_repairs_missing_core() {
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
            matches!(inspect_bot_plugin_version_at(&csgo).unwrap(), PluginVersionStatus::Valid { version } if version == Version::parse("0.6.0-test").unwrap())
        );
        fs::write(
            csgo.join("custom-plugin-settings.json"),
            b"player customization",
        )
        .unwrap();
        assert!(matches!(
            inspect_bot_plugin_version_at(&csgo).unwrap(),
            PluginVersionStatus::Valid { .. }
        ));
        fs::remove_file(csgo.join("addons/counterstrikesharp/plugins/BotState/BotState.dll"))
            .unwrap();
        assert!(
            matches!(inspect_bot_plugin_version_at(&csgo).unwrap(), PluginVersionStatus::Invalid { version: Some(version), .. } if version == Version::parse("0.6.0-test").unwrap())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bot_vision_state_removes_only_loader_vdf_and_persists_choice() {
        let root = std::env::temp_dir().join(format!(
            "bot-vision-state-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let csgo = root.join("game/csgo");
        let vdf = csgo.join(BOT_VISION_VDF);
        let dll = csgo.join("addons/BotVision/bin/win64/BotVision.dll");
        fs::create_dir_all(vdf.parent().unwrap()).unwrap();
        fs::create_dir_all(dll.parent().unwrap()).unwrap();
        fs::write(&vdf, b"loader").unwrap();
        fs::write(&dll, b"plugin").unwrap();

        apply_bot_vision_state(&csgo, false).unwrap();
        assert!(!vdf.exists());
        assert!(dll.is_file());
        assert!(!bot_vision_enabled_at(&csgo));
        assert_eq!(
            fs::read_to_string(csgo.join(BOT_VISION_STATE_FILE)).unwrap(),
            "disabled\n"
        );

        fs::write(&vdf, b"loader").unwrap();
        apply_bot_vision_state(&csgo, true).unwrap();
        assert!(vdf.is_file());
        assert!(bot_vision_enabled_at(&csgo));
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
            csgo.join("backup")
                .join("SkinOnly")
                .join("gameinfo.gi")
                .is_file(),
            !csgo.join(PANEL_FILE_NAME).exists(),
            sha256_file(&dll).expect("custom NadeSystem DLL must be readable"),
            matches!(&marker_status, PluginVersionStatus::Missing),
        );
        for relative in UPSTREAM_PANEL_REQUIRED_FILES {
            assert!(
                csgo.join(relative).is_file(),
                "upstream Panel file must be installed: {relative}"
            );
        }
        assert!(assertions.0, "gameinfo.gi must be installed");
        assert!(assertions.1, "Online gameinfo backup must be installed");
        assert!(assertions.2, "SkinOnly gameinfo backup must be installed");
        assert!(assertions.3, "Panel must not be installed into game files");
        assert_ne!(assertions.4, "");
        assert!(
            assertions.5,
            "official v1.4.5 archive must remain marker-free: {marker_status:?}"
        );
        fs::remove_dir_all(&fake_root).expect("fake CS2 root must be removable");
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
        assert!(matches!(
            inspect_bot_plugin_version_at(&csgo).unwrap(),
            PluginVersionStatus::Missing
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_uninstall_preserves_steam_cfg_gameinfo_and_inventory() {
        let root = std::env::temp_dir().join(format!(
            "legacy-uninstall-safety-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let csgo = root.join("game/csgo");
        let official_cfg = csgo.join("cfg/gamemode_casual.cfg");
        let bot_buy = csgo.join("cfg/bot_buy.cfg");
        let gameinfo = csgo.join("gameinfo.gi");
        let backup_online = csgo.join("backup/Online/gameinfo.gi");
        let plugin = csgo.join("addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll");
        let inventory = csgo
            .join("addons/counterstrikesharp/plugins/InventorySimulator/InventorySimulator.dll");
        let css_custom = csgo.join("addons/counterstrikesharp/api/player-custom.dll");
        for (path, bytes) in [
            (&official_cfg, b"steam-casual".as_slice()),
            (&bot_buy, b"steam-bot-buy".as_slice()),
            (&gameinfo, b"steam-gameinfo".as_slice()),
            (&backup_online, b"assistant-backup".as_slice()),
            (&plugin, b"upstream-plugin".as_slice()),
            (&inventory, b"inventory".as_slice()),
            (&css_custom, b"custom-css".as_slice()),
        ] {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }

        remove_upstream_package(&csgo).unwrap();

        assert_eq!(fs::read(&official_cfg).unwrap(), b"steam-casual");
        assert_eq!(fs::read(&bot_buy).unwrap(), b"steam-bot-buy");
        assert_eq!(fs::read(&gameinfo).unwrap(), b"steam-gameinfo");
        assert!(
            backup_online.is_file(),
            "legacy fallback must preserve backups"
        );
        assert!(
            !plugin.exists(),
            "known legacy plugin directory should be removed"
        );
        assert!(
            inventory.is_file(),
            "Inventory Simulator is managed separately"
        );
        assert!(
            css_custom.is_file(),
            "CounterStrikeSharp itself must be preserved"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ownership_ledger_restores_original_steam_files_on_uninstall() {
        let zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(BUNDLED_ZIP_NAME);
        let root = std::env::temp_dir().join(format!(
            "ownership-uninstall-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let csgo = root.join("game/csgo");
        let official_cfg = csgo.join("cfg/gamemode_casual.cfg");
        let official_gameinfo = csgo.join("gameinfo.gi");
        let inventory = csgo
            .join("addons/counterstrikesharp/plugins/InventorySimulator/InventorySimulator.dll");
        let css_custom = csgo.join("addons/counterstrikesharp/api/player-custom.dll");
        for (path, bytes) in [
            (&official_cfg, b"original-cfg".as_slice()),
            (&official_gameinfo, b"original-gameinfo".as_slice()),
            (&inventory, b"inventory".as_slice()),
            (&css_custom, b"custom-css".as_slice()),
        ] {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }

        install_game_files_transactionally(&zip_path, &csgo, false).unwrap();
        assert!(install_ledger_path(&csgo).is_file());
        assert_ne!(fs::read(&official_cfg).unwrap(), b"original-cfg");
        assert_ne!(fs::read(&official_gameinfo).unwrap(), b"original-gameinfo");

        remove_upstream_package(&csgo).unwrap();

        assert_eq!(fs::read(&official_cfg).unwrap(), b"original-cfg");
        assert_eq!(fs::read(&official_gameinfo).unwrap(), b"original-gameinfo");
        assert!(
            inventory.is_file(),
            "Inventory Simulator must survive uninstall"
        );
        assert!(
            css_custom.is_file(),
            "unknown CounterStrikeSharp files must survive"
        );
        assert!(!csgo.join("cfg/my_bot_normal_config.cfg").exists());
        assert!(!install_ledger_path(&csgo).exists());
        assert!(!install_backup_root(&csgo).exists());
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
    fn reinstall_restores_upstream_panel_cfg_after_user_deletion() {
        let zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(BUNDLED_ZIP_NAME);
        let root = std::env::temp_dir().join(format!("panel-cfg-reinstall-{}", std::process::id()));
        let csgo = root.join("game/csgo");
        fs::create_dir_all(&csgo).unwrap();
        install_game_files_transactionally(&zip_path, &csgo, false).unwrap();
        for relative in UPSTREAM_PANEL_CFG_FILES {
            assert!(
                csgo.join(relative).is_file(),
                "initial install must include {relative}"
            );
            fs::remove_file(csgo.join(relative)).unwrap();
        }
        install_game_files_transactionally(&zip_path, &csgo, false).unwrap();
        for relative in UPSTREAM_PANEL_CFG_FILES {
            assert!(
                csgo.join(relative).is_file(),
                "reinstall must restore {relative}"
            );
        }
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
