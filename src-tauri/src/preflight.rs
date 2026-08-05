use std::{
    env,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::Instant,
};

use chrono::Utc;
use serde::Serialize;
use serde_json::{json, Value};

static PROCESS_STARTED: OnceLock<Instant> = OnceLock::new();
static WRITE_LOCK: Mutex<()> = Mutex::new(());

const ALLOWED_EVENTS: &[&str] = &[
    "process_started",
    "interactive_ready",
    "viewer_mounted",
    "viewer_unmounted",
    "viewer_loaded",
    "viewer_play_started",
    "viewer_paused",
    "viewer_tab_hidden",
    "viewer_tab_visible",
    "viewer_measurement",
    "round_positions_measurement",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightRecordResult {
    elapsed_ms: f64,
    session_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightStatus {
    enabled: bool,
    elapsed_ms: f64,
    session_id: Option<String>,
}

fn session_id() -> Result<String, String> {
    let value = env::var("CS2AS_DEMO_PREFLIGHT_SESSION_ID")
        .map_err(|_| "CS2AS_DEMO_PREFLIGHT_SESSION_ID is not set".to_string())?;
    if value.is_empty()
        || value.len() > 80
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err("invalid preflight session id".into());
    }
    Ok(value)
}

fn runtime_path() -> Result<PathBuf, String> {
    let raw = env::var_os("CS2AS_DEMO_PREFLIGHT_EVIDENCE_DIR")
        .ok_or_else(|| "CS2AS_DEMO_PREFLIGHT_EVIDENCE_DIR is not set".to_string())?;
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        return Err("preflight evidence directory must be absolute".into());
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("preflight evidence directory is unavailable: {error}"))?;
    if !canonical.is_dir() {
        return Err("preflight evidence path is not a directory".into());
    }
    Ok(canonical.join("runtime.jsonl"))
}

fn elapsed_ms() -> f64 {
    PROCESS_STARTED
        .get_or_init(Instant::now)
        .elapsed()
        .as_secs_f64()
        * 1000.0
}

fn append_event(event: &str, data: Value) -> Result<PreflightRecordResult, String> {
    if !ALLOWED_EVENTS.contains(&event) {
        return Err(format!("unsupported preflight event: {event}"));
    }
    let session_id = session_id()?;
    let elapsed_ms = elapsed_ms();
    let record = json!({
        "wallClock": Utc::now().to_rfc3339(),
        "monotonicMs": elapsed_ms,
        "pid": std::process::id(),
        "sessionId": session_id,
        "event": event,
        "data": data,
    });
    let serialized = serde_json::to_string(&record).map_err(|error| error.to_string())?;
    let _guard = WRITE_LOCK
        .lock()
        .map_err(|_| "preflight writer lock poisoned".to_string())?;
    let mut output = OpenOptions::new()
        .create(true)
        .append(true)
        .open(runtime_path()?)
        .map_err(|error| format!("failed to open preflight runtime log: {error}"))?;
    writeln!(output, "{serialized}")
        .and_then(|_| output.flush())
        .map_err(|error| format!("failed to write preflight runtime log: {error}"))?;
    Ok(PreflightRecordResult {
        elapsed_ms,
        session_id,
    })
}

pub fn start() -> Result<(), String> {
    PROCESS_STARTED.get_or_init(Instant::now);
    if env::var_os("CS2AS_DEMO_PREFLIGHT_EVIDENCE_DIR").is_none() {
        return Ok(());
    }
    append_event("process_started", json!({}))?;
    Ok(())
}

#[tauri::command]
pub fn preflight_status() -> PreflightStatus {
    let enabled = env::var_os("CS2AS_DEMO_PREFLIGHT_EVIDENCE_DIR").is_some();
    PreflightStatus {
        enabled,
        elapsed_ms: elapsed_ms(),
        session_id: enabled.then(session_id).and_then(Result::ok),
    }
}

#[tauri::command]
pub fn preflight_record_event(event: String, data: Value) -> Result<PreflightRecordResult, String> {
    append_event(&event, data)
}

#[tauri::command]
pub fn preflight_exit_app(app: tauri::AppHandle) {
    app.exit(0);
}
