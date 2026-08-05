use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};

use crate::demo::post_match;
use crate::errors::AppError;
use crate::models::demo::{DemoPlaybackResult, RevealDemoResult};
use crate::services::{cs2, cs2_discovery, demo};

#[derive(Default)]
pub struct DemoPlaybackState {
    active_session: Mutex<Option<String>>,
}

struct PlaybackClaim<'a> {
    state: &'a DemoPlaybackState,
    session_id: String,
    committed: bool,
}

impl DemoPlaybackState {
    fn claim(&self) -> Result<PlaybackClaim<'_>, AppError> {
        let mut active = self.lock()?;
        if active.is_some() {
            return Err(error(
                "DEMO_PLAYBACK_BUSY",
                "已有 Demo 播放会话正在运行，请退出 CS2 后重试。",
            ));
        }
        let session_id = format!(
            "playback-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );
        *active = Some(session_id.clone());
        Ok(PlaybackClaim {
            state: self,
            session_id,
            committed: false,
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Option<String>>, AppError> {
        self.active_session
            .lock()
            .map_err(|_| error("DEMO_PLAYBACK_STATE", "Demo 播放状态不可用，请重启应用。"))
    }

    fn release(&self, session_id: &str) {
        if let Ok(mut active) = self.active_session.lock() {
            if active.as_deref() == Some(session_id) {
                *active = None;
            }
        }
    }
}

impl Drop for PlaybackClaim<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.state.release(&self.session_id);
        }
    }
}

pub fn play_demo(
    app: &AppHandle,
    state: &DemoPlaybackState,
    demo_id: i64,
    root_path: &str,
    tick: Option<i64>,
) -> Result<DemoPlaybackResult, AppError> {
    if cs2::check_cs2_process()? {
        return Err(error(
            "DEMO_PLAYBACK_CS2_RUNNING",
            "CS2 已在运行。请先退出 CS2，再播放 Demo。",
        ));
    }
    let mut claim = state.claim()?;
    let source = canonical_demo(app, demo_id)?;
    let root = cs2::normalize_root(root_path).map_err(|_| {
        error(
            "DEMO_PLAYBACK_ROOT_INVALID",
            "所选 CS2 目录无效，请在设置中重新选择游戏目录。",
        )
    })?;
    let csgo = root.join("game").join("csgo");
    if !csgo.is_dir() {
        return Err(error(
            "DEMO_PLAYBACK_ROOT_INVALID",
            "所选目录缺少 game\\csgo，请重新选择 CS2 安装目录。",
        ));
    }
    let steam = cs2_discovery::find_steam_executable(Some(&root)).ok_or_else(|| {
        error(
            "STEAM_NOT_FOUND",
            "未找到 Steam 客户端。请先启动一次 Steam 后重试。",
        )
    })?;

    let (prepared, temporary) = prepare_demo(&source, &csgo, &claim.session_id)?;
    let relative = prepared.strip_prefix(&csgo).map_err(|_| {
        cleanup_temporary(&prepared, temporary);
        error("DEMO_PLAYBACK_ROOT_INVALID", "无法生成 CS2 内部播放路径。")
    })?;
    let args = playback_args(relative, tick);
    post_match::mark_next_playback(app, &claim.session_id);
    if let Err(error_value) = Command::new(&steam).args(&args).spawn() {
        post_match::cancel_pending(app, &claim.session_id);
        cleanup_temporary(&prepared, temporary);
        return Err(error(
            "DEMO_LAUNCH",
            format!("无法启动 Steam：{error_value}"),
        ));
    }

    let session_id = claim.session_id.clone();
    let app_handle = app.clone();
    let cleanup_path = prepared.clone();
    claim.committed = true;
    std::thread::Builder::new()
        .name(format!("demo-playback-{session_id}"))
        .spawn(move || {
            if !wait_for_cs2_lifecycle() {
                post_match::cancel_pending(&app_handle, &session_id);
            }
            cleanup_temporary(&cleanup_path, temporary);
            app_handle.state::<DemoPlaybackState>().release(&session_id);
        })
        .map_err(|spawn_error| {
            post_match::cancel_pending(app, &claim.session_id);
            cleanup_temporary(&prepared, temporary);
            state.release(&claim.session_id);
            error(
                "DEMO_PLAYBACK_STATE",
                format!("无法监视 CS2 播放会话：{spawn_error}"),
            )
        })?;

    Ok(DemoPlaybackResult {
        demo_file_id: demo_id,
        session_id: claim.session_id.clone(),
        source_path: source.display().to_string(),
        prepared_path: prepared.display().to_string(),
        started: true,
    })
}

pub fn reveal_demo_file(app: &AppHandle, demo_id: i64) -> Result<RevealDemoResult, AppError> {
    let path = canonical_demo(app, demo_id)?;
    reveal_path(&path)?;
    Ok(RevealDemoResult {
        demo_file_id: demo_id,
        path: path.display().to_string(),
        revealed: true,
    })
}

#[cfg(windows)]
fn reveal_path(path: &Path) -> Result<(), AppError> {
    Command::new("explorer.exe")
        .arg("/select,")
        .arg(path)
        .spawn()
        .map_err(|value| error("DEMO_REVEAL", format!("无法打开资源管理器：{value}")))?;
    Ok(())
}

#[cfg(not(windows))]
fn reveal_path(path: &Path) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| error("DEMO_REVEAL", "Demo 父目录无效。"))?;
    Command::new("xdg-open")
        .arg(parent)
        .spawn()
        .map_err(|value| error("DEMO_REVEAL", format!("无法打开文件目录：{value}")))?;
    Ok(())
}

fn canonical_demo(app: &AppHandle, demo_id: i64) -> Result<PathBuf, AppError> {
    let stored = demo::path_for_id(app, demo_id)?;
    let path = dunce::canonicalize(&stored).map_err(|_| {
        error(
            "DEMO_FILE_NOT_FOUND",
            "Demo 文件已移动或删除，请重新扫描录像库。",
        )
    })?;
    if !path.is_file()
        || !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("dem"))
    {
        return Err(error(
            "DEMO_FILE_NOT_FOUND",
            "Demo 文件不存在或不是 .dem 文件，请重新扫描录像库。",
        ));
    }
    Ok(path)
}

fn prepare_demo(source: &Path, csgo: &Path, session_id: &str) -> Result<(PathBuf, bool), AppError> {
    if source.starts_with(csgo) {
        return Ok((source.to_path_buf(), false));
    }
    let before = metadata_snapshot(source)?;
    let replay_dir = csgo.join("replays");
    fs::create_dir_all(&replay_dir)
        .map_err(|value| error("DEMO_LAUNCH", format!("无法创建回放目录：{value}")))?;
    let target = replay_dir.join(format!("_cs2as_play_{session_id}.dem"));
    if target.exists() {
        return Err(error("DEMO_PLAYBACK_BUSY", "临时播放文件已存在，请重试。"));
    }
    fs::copy(source, &target)
        .map_err(|value| error("DEMO_LAUNCH", format!("无法准备 Demo 临时副本：{value}")))?;
    let after = metadata_snapshot(source)?;
    if before != after || fs::metadata(&target).map(|value| value.len()).ok() != Some(before.0) {
        let _ = fs::remove_file(&target);
        return Err(error(
            "DEMO_PLAYBACK_COPY_CHANGED",
            "Demo 在准备播放时仍在变化，请等待写入完成后重试。",
        ));
    }
    Ok((target, true))
}

fn metadata_snapshot(path: &Path) -> Result<(u64, Option<SystemTime>), AppError> {
    let metadata = fs::metadata(path).map_err(|_| {
        error(
            "DEMO_FILE_NOT_FOUND",
            "Demo 文件已移动或删除，请重新扫描录像库。",
        )
    })?;
    Ok((metadata.len(), metadata.modified().ok()))
}

fn playback_args(relative: &Path, tick: Option<i64>) -> Vec<OsString> {
    let mut args = vec![
        OsString::from("-applaunch"),
        OsString::from("730"),
        OsString::from("-insecure"),
        OsString::from("-novid"),
        OsString::from("+playdemo"),
        relative.as_os_str().to_owned(),
    ];
    if let Some(tick) = tick.filter(|value| *value > 0) {
        args.extend([
            OsString::from("+demo_gototick"),
            OsString::from(tick.to_string()),
        ]);
    }
    args
}

fn wait_for_cs2_lifecycle() -> bool {
    let mut seen = false;
    for _ in 0..12 {
        if cs2::check_cs2_process().unwrap_or(false) {
            seen = true;
            break;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    while seen && cs2::check_cs2_process().unwrap_or(false) {
        std::thread::sleep(Duration::from_secs(1));
    }
    seen
}

fn cleanup_temporary(path: &Path, temporary: bool) {
    if temporary && path.is_file() {
        let _ = fs::remove_file(path);
    }
}

fn error(code: &str, message: impl std::fmt::Display) -> AppError {
    AppError::runtime(format!("[{code}]\n{message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_arguments_keep_demo_path_as_one_argument() {
        let args = playback_args(Path::new(r"replays\中文, demo & one.dem"), Some(42));
        assert_eq!(args[4], "+playdemo");
        assert_eq!(args[5], r"replays\中文, demo & one.dem");
        assert_eq!(args[6], "+demo_gototick");
        assert_eq!(args[7], "42");
    }

    #[test]
    fn playback_state_releases_failed_claim() {
        let state = DemoPlaybackState::default();
        {
            let _claim = state.claim().unwrap();
        }
        assert!(state.claim().is_ok());
    }
}
