use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::services::demo;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingScoreboardReport {
    pub report_id: i64,
    pub sequence: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreboardOpenResult {
    pub accepted: bool,
    pub sequence: u64,
}

#[derive(Default)]
pub struct ScoreboardState {
    pending: Mutex<Option<PendingScoreboardReport>>,
    frontend_ready: AtomicBool,
    sequence: AtomicU64,
}

impl ScoreboardState {
    fn pending(&self) -> Result<Option<PendingScoreboardReport>, String> {
        self.pending
            .lock()
            .map(|pending| *pending)
            .map_err(|_| "战报窗口状态不可用。".into())
    }

    fn replace_pending(&self, report_id: i64) -> Result<PendingScoreboardReport, String> {
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed) + 1;
        let next = PendingScoreboardReport {
            report_id,
            sequence,
        };
        *self
            .pending
            .lock()
            .map_err(|_| "战报窗口状态不可用。".to_string())? = Some(next);
        Ok(next)
    }

    fn take_if_current(&self, sequence: u64) -> Result<bool, String> {
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| "战报窗口状态不可用。".to_string())?;
        if pending.is_some_and(|item| item.sequence == sequence) {
            *pending = None;
            return Ok(true);
        }
        Ok(false)
    }
}

#[tauri::command]
pub fn open_scoreboard(
    app: AppHandle,
    state: tauri::State<'_, ScoreboardState>,
    report_id: i64,
) -> Result<ScoreboardOpenResult, String> {
    if report_id <= 0 {
        return Err("战报 ID 无效。".into());
    }
    demo::presentable_report(&app, report_id).map_err(|e| e.into_string())?;

    let pending = state.replace_pending(report_id)?;
    if state.frontend_ready.load(Ordering::Acquire) {
        app.emit_to("scoreboard", "scoreboard://load-report", pending)
            .map_err(|error| error.to_string())?;
    }

    let timeout_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let state = timeout_app.state::<ScoreboardState>();
        if state
            .pending()
            .ok()
            .flatten()
            .is_some_and(|item| item.sequence == pending.sequence)
        {
            log::error!(
                "scoreboard sequence {} did not present in 5 seconds",
                pending.sequence
            );
            let _ = timeout_app.emit_to(
                "main",
                "scoreboard://boot-error",
                "战报窗口加载超时，可在对局报告中继续查看。",
            );
        }
    });

    Ok(ScoreboardOpenResult {
        accepted: true,
        sequence: pending.sequence,
    })
}

#[tauri::command]
pub fn scoreboard_frontend_ready(
    state: tauri::State<'_, ScoreboardState>,
) -> Result<Option<PendingScoreboardReport>, String> {
    state.frontend_ready.store(true, Ordering::Release);
    state.pending()
}

#[tauri::command]
pub fn scoreboard_present(
    app: AppHandle,
    state: tauri::State<'_, ScoreboardState>,
    report_id: i64,
    sequence: u64,
) -> Result<(), String> {
    let current = state.pending()?;
    if !current.is_some_and(|item| item.report_id == report_id && item.sequence == sequence) {
        return Err("战报请求已被更新。".into());
    }
    let window = app
        .get_webview_window("scoreboard")
        .ok_or_else(|| "战报窗口不可用。".to_string())?;
    window.unminimize().map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    state.take_if_current(sequence)?;
    Ok(())
}

#[tauri::command]
pub fn hide_scoreboard(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("scoreboard")
        .ok_or_else(|| "战报窗口不可用。".to_string())?
        .hide()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn destroy_scoreboard(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("scoreboard") {
        window.destroy().map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn report_scoreboard_boot_error(app: AppHandle, message: String) -> Result<(), String> {
    let sanitized: String = message.chars().take(240).collect();
    log::error!("scoreboard frontend error: {sanitized}");
    app.emit_to("main", "scoreboard://boot-error", sanitized)
        .map_err(|error| error.to_string())
}
