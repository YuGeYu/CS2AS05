use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter, Manager};

use crate::models::demo::{DemoListItem, PostMatchReportFailed, PostMatchReportReady};
use crate::{demo::parse_gate, services::demo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionKind {
    LiveMatch,
    DemoPlayback,
}

#[derive(Debug, Clone)]
struct PendingSession {
    id: String,
    kind: SessionKind,
}

#[derive(Debug, Clone)]
struct RunningSession {
    id: String,
    kind: SessionKind,
    baseline: HashMap<i64, (i64, i64)>,
}

#[derive(Default)]
struct CoordinatorInner {
    pending: Option<PendingSession>,
    running: Option<RunningSession>,
}

#[derive(Default)]
pub struct GameSessionCoordinator(Mutex<CoordinatorInner>);

pub fn start(app: &AppHandle) {
    let handle = app.clone();
    std::thread::Builder::new()
        .name("game-session-coordinator".into())
        .spawn(move || observe(handle))
        .expect("failed to start game session coordinator");
}

pub fn mark_next_live(app: &AppHandle) -> String {
    mark_next(app, SessionKind::LiveMatch, None)
}

pub fn mark_next_playback(app: &AppHandle, requested_id: &str) {
    mark_next(app, SessionKind::DemoPlayback, Some(requested_id));
}

fn mark_next(app: &AppHandle, kind: SessionKind, requested_id: Option<&str>) -> String {
    let id = requested_id.map(str::to_owned).unwrap_or_else(|| {
        format!(
            "live-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        )
    });
    if let Ok(mut state) = app.state::<GameSessionCoordinator>().0.lock() {
        state.pending = Some(PendingSession {
            id: id.clone(),
            kind,
        });
    }
    id
}

pub fn cancel_pending(app: &AppHandle, session_id: &str) {
    if let Ok(mut state) = app.state::<GameSessionCoordinator>().0.lock() {
        if state
            .pending
            .as_ref()
            .is_some_and(|item| item.id == session_id)
        {
            state.pending = None;
        }
    }
}

fn observe(app: AppHandle) {
    let mut session_active = parse_gate::observe_process()
        .map(|observation| observation.running)
        .unwrap_or(false);
    if session_active {
        begin_session(&app);
    }
    loop {
        std::thread::sleep(Duration::from_secs(1));
        let Ok(observation) = parse_gate::observe_process() else {
            continue;
        };
        if observation.running && !session_active {
            begin_session(&app);
            session_active = true;
        } else if observation.confirmed_stopped && session_active {
            if let Some(session) = end_session(&app) {
                let handle = app.clone();
                let _ = std::thread::Builder::new()
                    .name(format!("post-match-{}", session.id))
                    .spawn(move || finish_session(handle, session));
            }
            session_active = false;
        }
    }
}

fn begin_session(app: &AppHandle) {
    let baseline = snapshot(app);
    if let Ok(mut state) = app.state::<GameSessionCoordinator>().0.lock() {
        if state.running.is_some() {
            return;
        }
        let pending = state.pending.take().unwrap_or_else(|| PendingSession {
            id: format!(
                "live-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            ),
            kind: SessionKind::LiveMatch,
        });
        state.running = Some(RunningSession {
            id: pending.id,
            kind: pending.kind,
            baseline,
        });
    }
}

fn end_session(app: &AppHandle) -> Option<RunningSession> {
    app.state::<GameSessionCoordinator>()
        .0
        .lock()
        .ok()?
        .running
        .take()
}

fn snapshot(app: &AppHandle) -> HashMap<i64, (i64, i64)> {
    demo::list_all_for_session(app)
        .map(|items| {
            items
                .into_iter()
                .map(|item| (item.id, (item.size_bytes, item.mtime_ms)))
                .collect()
        })
        .unwrap_or_default()
}

fn finish_session(app: AppHandle, session: RunningSession) {
    if session.kind == SessionKind::DemoPlayback {
        return;
    }
    // Give filesystem notifications a final stable sample window before importing.
    std::thread::sleep(Duration::from_millis(1_200));
    if let Err(error) = demo::scan(&app, None) {
        emit_failed(
            &app,
            &session.id,
            None,
            "POST_MATCH_SCAN",
            error.into_string(),
        );
        return;
    }
    let items = match demo::list_all_for_session(&app) {
        Ok(items) => items,
        Err(error) => {
            emit_failed(
                &app,
                &session.id,
                None,
                "POST_MATCH_QUERY",
                error.into_string(),
            );
            return;
        }
    };
    let has_skipped = items
        .iter()
        .any(|item| item.status == "skipped" && item.map_source != "official");
    let candidate = items
        .into_iter()
        .filter(|item| {
            item.map_source == "official"
                && item.status != "skipped"
                && session
                    .baseline
                    .get(&item.id)
                    .map_or(true, |old| *old != (item.size_bytes, item.mtime_ms))
        })
        .max_by_key(|item| (item.mtime_ms, item.id));
    let Some(candidate) = candidate else {
        if has_skipped {
            emit_failed(
                &app,
                &session.id,
                None,
                "POST_MATCH_WORKSHOP_DEMO_SKIPPED",
                "录像已保存，创意工坊或来源未确认的 Demo 未解析。".into(),
            );
            return;
        }
        emit_failed(
            &app,
            &session.id,
            None,
            "POST_MATCH_NO_DEMO",
            "本次游戏会话结束后未发现新的 Demo。".into(),
        );
        return;
    };
    wait_for_report(&app, &session.id, candidate);
}

fn wait_for_report(app: &AppHandle, session_id: &str, candidate: DemoListItem) {
    for _ in 0..600 {
        let item = demo::demo_item(app, candidate.id).ok().flatten();
        match item.as_ref().map(|item| item.status.as_str()) {
            Some("done") => match demo::presentable_report(app, candidate.id) {
                Ok(report) => {
                    let _ = app.emit(
                        "demo://report-ready",
                        PostMatchReportReady {
                            session_id: session_id.into(),
                            report_id: candidate.id,
                            file_name: candidate.file_name,
                            completed_at: report.summary.parsed_at,
                            origin: "post_match".into(),
                        },
                    );
                    return;
                }
                Err(error) => {
                    emit_failed(
                        app,
                        session_id,
                        Some(candidate.id),
                        "POST_MATCH_REPORT_INVALID",
                        error.into_string(),
                    );
                    return;
                }
            },
            Some("error" | "canceled") => {
                emit_failed(
                    app,
                    session_id,
                    Some(candidate.id),
                    "POST_MATCH_PARSE_FAILED",
                    "最新 Demo 解析失败，请在录像库中重试。".into(),
                );
                return;
            }
            _ => std::thread::sleep(Duration::from_secs(1)),
        }
    }
    emit_failed(
        app,
        session_id,
        Some(candidate.id),
        "POST_MATCH_TIMEOUT",
        "等待最新 Demo 解析超时，请在录像库中查看任务状态。".into(),
    );
}

fn emit_failed(
    app: &AppHandle,
    session_id: &str,
    demo_id: Option<i64>,
    code: &str,
    message: String,
) {
    let _ = app.emit(
        "demo://report-failed",
        PostMatchReportFailed {
            session_id: session_id.into(),
            demo_id,
            error_code: code.into(),
            message,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_changed_candidate_wins_even_if_older_is_done() {
        let baseline = HashMap::from([(1, (10, 100)), (2, (10, 200))]);
        let rows = [
            (1, 10, 100, "done"),
            (2, 12, 300, "parsing"),
            (3, 8, 250, "done"),
        ];
        let selected = rows
            .into_iter()
            .filter(|(id, size, mtime, _)| {
                baseline.get(id).map_or(true, |old| *old != (*size, *mtime))
            })
            .max_by_key(|(id, _, mtime, _)| (*mtime, *id))
            .unwrap();
        assert_eq!(selected.0, 2);
        assert_eq!(selected.3, "parsing");
    }
}
