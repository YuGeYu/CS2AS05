use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use chrono::Local;
use sha2::{Digest, Sha256};
#[cfg(not(windows))]
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tauri::{AppHandle, Manager};
#[cfg(windows)]
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE,
};
#[cfg(windows)]
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use zip::ZipArchive;

use crate::errors::AppError;
use crate::models::cs2::{Cs2EnvironmentStatus, Cs2RootCandidate, DiagnosticsPayload, OperationResult};

const CS2_FOLDER_NAME: &str = "Counter-Strike Global Offensive";
const BUNDLED_ZIP_NAME: &str = "CS2BotImprover.zip";
const CUSTOM_ZIP_SHA256: &str = "D6BCFF73BBACFEBC0BB3ED91A440A584EFE2C812962655F392F319463DA8D9E0";
const PANEL_FILE_NAME: &str = "Panel v1.4.2.exe";
const PANEL_SHA256: &str = "9C3AB83909E506C0D4BD4886C961DFC0E871DA71BB47E1D1BEA7EF2CCFE40AB2";
const PANEL_SIZE: u64 = 5_839_872;
const LOG_DIR_NAME: &str = "CS2人机增强助手";
const REQUIRED_ZIP_ENTRIES: &[&str] = &[
    PANEL_FILE_NAME,
    "gameinfo.gi",
    "backup/Online/gameinfo.gi",
    "backup/WithBots/gameinfo.gi",
    "addons/",
    "cfg/",
    "overrides/",
];

pub fn discover_cs2_roots() -> Result<Vec<Cs2RootCandidate>, AppError> {
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();
    for drive in existing_drives() {
        let steam = drive.join("Program Files (x86)").join("Steam");
        add_candidate(&mut seen, &mut candidates, steam.join("steamapps").join("common").join(CS2_FOLDER_NAME), "Steam 常规目录");
        for library in parse_libraryfolders(&steam.join("steamapps").join("libraryfolders.vdf"))? {
            add_candidate(&mut seen, &mut candidates, library.join("steamapps").join("common").join(CS2_FOLDER_NAME), "Steam 库配置");
        }
    }
    write_log("INFO", &format!("扫描到 {} 个 CS2 候选目录。", candidates.len()));
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
        backup_online_gameinfo_exists: csgo.join("backup").join("Online").join("gameinfo.gi").is_file(),
        backup_withbots_gameinfo_exists: csgo.join("backup").join("WithBots").join("gameinfo.gi").is_file(),
        base_environment_ready: csgo.join("addons").join("metamod").is_dir()
            && csgo.join("addons").join("counterstrikesharp").is_dir()
            && csgo.join("gameinfo.gi").is_file()
            && csgo.join("backup").join("Online").join("gameinfo.gi").is_file()
            && csgo.join("backup").join("WithBots").join("gameinfo.gi").is_file(),
    };
    write_log("INFO", &format!("已检查 CS2 目录：{}。", status.root_path));
    Ok(status)
}

pub fn check_cs2_process() -> Result<bool, AppError> {
    check_cs2_process_platform()
}

fn is_cs2_process_name(name: &str) -> bool {
    let name = name.trim_end_matches('\0');
    name.eq_ignore_ascii_case("cs2") || name.eq_ignore_ascii_case("cs2.exe")
}

#[cfg(windows)]
fn check_cs2_process_platform() -> Result<bool, AppError> {
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
            return Ok(false);
        }
        return Err(AppError::runtime(format!(
            "枚举 Windows 进程失败：{}",
            std::io::Error::from_raw_os_error(error as i32)
        )));
    }

    loop {
        let name_end = entry
            .szExeFile
            .iter()
            .position(|character| *character == 0)
            .unwrap_or(entry.szExeFile.len());
        let name = String::from_utf16_lossy(&entry.szExeFile[..name_end]);
        if is_cs2_process_name(&name) {
            return Ok(true);
        }
        if unsafe { Process32NextW(snapshot.0, &mut entry) } == 0 {
            let error = unsafe { GetLastError() };
            if error == ERROR_NO_MORE_FILES {
                return Ok(false);
            }
            return Err(AppError::runtime(format!(
                "枚举 Windows 进程失败：{}",
                std::io::Error::from_raw_os_error(error as i32)
            )));
        }
    }
}

#[cfg(not(windows))]
fn check_cs2_process_platform() -> Result<bool, AppError> {
    let system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
    );
    Ok(system
        .processes()
        .values()
        .any(|process| is_cs2_process_name(&process.name().to_string_lossy())))
}

pub fn install_bot_package(app: &AppHandle, root_path: &str) -> Result<OperationResult, AppError> {
    ensure_cs2_not_running()?;
    let root = normalize_root(root_path)?;
    let destination = root.join("game").join("csgo");
    if !destination.is_dir() {
        return Err(AppError::runtime(format!("[INSTALL_TARGET_MISSING]\n未找到 CS2 游戏目录：{}", destination.display())));
    }

    let zip_path = resolve_zip_path(app)?;
    verify_custom_zip(&zip_path)?;
    let removed = remove_upstream_package(&destination)?;
    extract_game_files(&zip_path, &destination)?;

    write_log("INFO", &format!("基于上游 v1.4.2 的最小定制插件包已安装到 {}。", destination.display()));
    Ok(OperationResult {
        success: true,
        message: format!(
            "基于上游 CS2-Bot-Improver v1.4.2 的最小定制包已安装。\n目标目录：{}\n已清理旧文件：{} 项。\n模式、难度、Aim、Nades 和外观请在官方 Panel 中设置。",
            destination.display(), removed
        ),
    })
}

pub fn open_upstream_panel(app: &AppHandle) -> Result<OperationResult, AppError> {
    let zip_path = resolve_zip_path(app)?;
    verify_custom_zip(&zip_path)?;
    let tool_dir = app.path().app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法确定应用数据目录：{error}")))?
        .join("tools")
        .join("official-panel-v1.4.2");
    let panel_path = tool_dir.join(PANEL_FILE_NAME);
    if !panel_is_valid(&panel_path)? {
        extract_panel_atomically(&zip_path, &tool_dir, &panel_path)?;
    }
    Command::new(&panel_path)
        .current_dir(&tool_dir)
        .spawn()
        .map_err(|error| AppError::runtime(format!("无法启动官方 Panel：{}\n路径：{}", error, panel_path.display())))?;
    write_log("INFO", &format!("已启动官方 Panel：{}。", panel_path.display()));
    Ok(OperationResult { success: true, message: format!("已启动官方 Panel v1.4.2。\n{}", panel_path.display()) })
}

pub fn uninstall_bot_package(root_path: &str) -> Result<OperationResult, AppError> {
    ensure_cs2_not_running()?;
    let root = normalize_root(root_path)?;
    let csgo = root.join("game").join("csgo");
    if !csgo.is_dir() {
        return Err(AppError::runtime("未找到 CS2 游戏目录，无法卸载。"));
    }
    let removed = remove_upstream_package(&csgo)?;
    write_log("INFO", &format!("已从 {} 卸载官方插件文件。", csgo.display()));
    Ok(OperationResult { success: true, message: format!("已移除官方插件文件：{} 项。\n保留了 CS2 核心文件和 gameinfo.gi。", removed) })
}

pub fn get_diagnostics_payload(root_path: Option<&str>) -> Result<DiagnosticsPayload, AppError> {
    let log_path = diagnostics_log_path();
    let full_log = fs::read_to_string(&log_path).unwrap_or_else(|_| "日志文件尚未创建。".to_string());
    let summary = match root_path {
        Some(root) => {
            let status = inspect_cs2_root(root)?;
            format!("CS2 目录：{}\n游戏运行中：{}\n插件环境：{}", status.root_path, yes_no(check_cs2_process()?), if status.base_environment_ready { "完整" } else { "未完整安装" })
        }
        None => format!("尚未选择 CS2 目录。\n游戏运行中：{}", yes_no(check_cs2_process()?)),
    };
    Ok(DiagnosticsPayload { summary, full_log, log_path: log_path.display().to_string() })
}

fn resolve_zip_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let resource_dir = app.path().resource_dir().map_err(|error| AppError::runtime(format!("无法定位内置资源：{error}")))?;
    let candidates = [resource_dir.join(BUNDLED_ZIP_NAME), resource_dir.join("resources").join(BUNDLED_ZIP_NAME)];
    candidates.into_iter().find(|path| path.is_file()).ok_or_else(|| AppError::runtime("未找到内置定制 CS2BotImprover.zip，无法继续。"))
}

fn verify_custom_zip(path: &Path) -> Result<(), AppError> {
    let digest = sha256_file(path)?;
    if digest != CUSTOM_ZIP_SHA256 {
        return Err(AppError::runtime(format!("[ZIP_HASH_INVALID]\n内置定制资源摘要不匹配。\n期望：{CUSTOM_ZIP_SHA256}\n实际：{digest}")));
    }
    let file = File::open(path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file).map_err(|error| AppError::runtime(format!("无法读取内置资源包：{error}")))?;
    for required in REQUIRED_ZIP_ENTRIES {
        archive.by_name(required).map_err(|_| AppError::runtime(format!("[ZIP_STRUCTURE_INVALID]\n内置定制资源缺少必需条目：{required}")))?;
    }
    let entry = archive.by_name(PANEL_FILE_NAME).map_err(|_| AppError::runtime("内置定制资源缺少 Panel。"))?;
    if entry.size() != PANEL_SIZE {
        return Err(AppError::runtime("[PANEL_SIZE_INVALID]\n官方 Panel 文件大小不正确。"));
    }
    Ok(())
}

fn extract_game_files(zip_path: &Path, destination: &Path) -> Result<(), AppError> {
    let file = File::open(zip_path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file).map_err(|error| AppError::runtime(format!("无法读取资源包：{error}")))?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| AppError::runtime(format!("无法读取资源条目：{error}")))?;
        if entry.name() == PANEL_FILE_NAME { continue; }
        let relative = safe_zip_path(entry.name())?;
        if relative.as_os_str().is_empty() { continue; }
        let target = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(io_error)?;
            continue;
        }
        if let Some(parent) = target.parent() { fs::create_dir_all(parent).map_err(io_error)?; }
        let mut output = File::create(&target).map_err(io_error)?;
        io::copy(&mut entry, &mut output).map_err(io_error)?;
    }
    Ok(())
}

fn extract_panel_atomically(zip_path: &Path, tool_dir: &Path, panel_path: &Path) -> Result<(), AppError> {
    fs::create_dir_all(tool_dir).map_err(io_error)?;
    let temporary = tool_dir.join("Panel-v1.4.2.tmp");
    let file = File::open(zip_path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file).map_err(|error| AppError::runtime(format!("无法读取资源包：{error}")))?;
    let mut entry = archive.by_name(PANEL_FILE_NAME).map_err(|_| AppError::runtime("官方资源缺少 Panel。"))?;
    let mut output = File::create(&temporary).map_err(io_error)?;
    io::copy(&mut entry, &mut output).map_err(io_error)?;
    output.flush().map_err(io_error)?;
    drop(output);
    if !panel_is_valid(&temporary)? {
        return Err(AppError::runtime("[PANEL_HASH_INVALID]\n从官方资源提取的 Panel 校验失败。"));
    }
    if panel_path.exists() { fs::remove_file(panel_path).map_err(io_error)?; }
    fs::rename(&temporary, panel_path).map_err(io_error)
}

fn panel_is_valid(path: &Path) -> Result<bool, AppError> {
    Ok(path.is_file() && fs::metadata(path).map_err(io_error)?.len() == PANEL_SIZE && sha256_file(path)? == PANEL_SHA256)
}

fn remove_upstream_package(csgo: &Path) -> Result<usize, AppError> {
    let paths = [
        csgo.join("addons"), csgo.join("backup"), csgo.join("cfg").join("plugins"),
        csgo.join("overrides").join("Low"), csgo.join("overrides").join("Medium"), csgo.join("overrides").join("High"),
    ];
    let files = [
        csgo.join("cfg").join("my_bot_normal_config.cfg"), csgo.join("cfg").join("my_bot_ffa_config.cfg"),
        csgo.join("overrides").join("botprofile.vpk"), csgo.join("metamod.vdf"), csgo.join("metamod_x64.vdf"),
    ];
    let mut removed = 0;
    for path in paths { if path.exists() { fs::remove_dir_all(&path).map_err(io_error)?; removed += 1; } }
    for path in files { if path.exists() { fs::remove_file(&path).map_err(io_error)?; removed += 1; } }
    Ok(removed)
}

fn normalize_root(path: &str) -> Result<PathBuf, AppError> {
    let selected = PathBuf::from(path.trim());
    if selected.join("game").join("csgo").is_dir() { return Ok(selected); }
    if selected.file_name().is_some_and(|name| name.eq_ignore_ascii_case("game")) && selected.join("csgo").is_dir() {
        return selected.parent().map(Path::to_path_buf).ok_or_else(|| AppError::runtime("CS2 根目录无效。"));
    }
    if selected.file_name().is_some_and(|name| name.eq_ignore_ascii_case("csgo")) && selected.parent().is_some_and(|parent| parent.file_name().is_some_and(|name| name.eq_ignore_ascii_case("game"))) {
        return selected.parent().and_then(Path::parent).map(Path::to_path_buf).ok_or_else(|| AppError::runtime("CS2 根目录无效。"));
    }
    Err(AppError::runtime("请选择 Counter-Strike Global Offensive 目录，或其中的 game/csgo 目录。"))
}

fn safe_zip_path(name: &str) -> Result<PathBuf, AppError> {
    let path = Path::new(name);
    if path.is_absolute() || path.components().any(|component| matches!(component, Component::ParentDir | Component::Prefix(_) | Component::RootDir)) {
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
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:X}", hasher.finalize()))
}

fn existing_drives() -> Vec<PathBuf> {
    (b'A'..=b'Z').filter_map(|letter| { let path = PathBuf::from(format!("{}:\\", letter as char)); path.is_dir().then_some(path) }).collect()
}

fn parse_libraryfolders(path: &Path) -> Result<Vec<PathBuf>, AppError> {
    if !path.is_file() { return Ok(Vec::new()); }
    let content = fs::read_to_string(path).map_err(io_error)?;
    let mut paths = Vec::new();
    for line in content.lines() {
        let value = line.split('"').nth(3).unwrap_or_default().replace("\\\\", "\\");
        if value.contains(":\\") { paths.push(PathBuf::from(value)); }
    }
    Ok(paths)
}

fn add_candidate(seen: &mut BTreeSet<String>, candidates: &mut Vec<Cs2RootCandidate>, path: PathBuf, source: &str) {
    if path.join("game").join("csgo").is_dir() {
        let normalized = path.display().to_string();
        if seen.insert(normalized.clone()) { candidates.push(Cs2RootCandidate { path: normalized, source: source.to_string() }); }
    }
}

fn ensure_cs2_not_running() -> Result<(), AppError> {
    if check_cs2_process()? { return Err(AppError::runtime("检测到 cs2.exe 正在运行。请先退出 CS2，再执行此操作。")); }
    Ok(())
}

fn diagnostics_log_path() -> PathBuf {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join(LOG_DIR_NAME).join("logs").join("runtime.log")
}

fn write_log(level: &str, message: &str) {
    let path = diagnostics_log_path();
    if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "[{}] [{}] {}", Local::now().format("%Y-%m-%d %H:%M:%S"), level, message);
    }
}

fn yes_no(value: bool) -> &'static str { if value { "是" } else { "否" } }
fn io_error(error: io::Error) -> AppError { AppError::runtime(error.to_string()) }

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
        let fake_root = std::env::temp_dir().join(format!("ai-pc-fac-0.5.2-{nonce}"));
        let csgo = fake_root.join("game").join("csgo");
        fs::create_dir_all(&csgo).expect("fake CS2 root must be creatable");
        extract_game_files(&zip_path, &csgo).expect("custom ZIP must extract through installer logic");

        let dll = csgo
            .join("addons")
            .join("counterstrikesharp")
            .join("plugins")
            .join("NadeSystem")
            .join("NadeSystem.dll");
        let assertions = (
            csgo.join("gameinfo.gi").is_file(),
            csgo.join("backup").join("Online").join("gameinfo.gi").is_file(),
            !csgo.join(PANEL_FILE_NAME).exists(),
            sha256_file(&dll).expect("custom NadeSystem DLL must be readable"),
        );
        fs::remove_dir_all(&fake_root).expect("fake CS2 root must be removable");

        assert!(assertions.0, "gameinfo.gi must be installed");
        assert!(assertions.1, "Online gameinfo backup must be installed");
        assert!(assertions.2, "Panel must not be installed into game files");
        assert_eq!(
            assertions.3,
            "9A0BE54CACB426D9DAF31B1DEB4D1C665089F9F274AF5CA0DFF24BCB2BFE5661"
        );
    }
}
