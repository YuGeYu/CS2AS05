use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use cs2_demoparser::{
    first_pass::parser_settings::ParserInputs,
    parse_demo::{Parser, ParsingMode},
    second_pass::{
        game_events::GameEvent,
        parser_settings::create_huffman_lookup_table,
        variants::{VarVec, Variant},
    },
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::{params, Connection, OptionalExtension};
use tauri::{AppHandle, Emitter, Manager};

use crate::{errors::AppError, models::demo::*, services::cs2};

const PARSER_COMMIT: &str = "ba39cc44cd5abfd7f34df2b3c0a7dd3630048311";
const REPORT_SCHEMA_VERSION: u32 = 2;
const PARSER_ADAPTER_VERSION: &str = "2";
const METRICS_VERSION: &str = "scoreboard-v2";
const MIN_STEAM_ID64: u64 = 76_561_197_960_265_728;

#[derive(Default)]
pub struct DemoWatcherState(pub Mutex<Option<RecommendedWatcher>>);

fn err(code: &str, detail: impl std::fmt::Display) -> AppError {
    AppError::runtime(format!("[{code}] {detail}"))
}
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
fn mtime_ms(meta: &fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|v| v.duration_since(UNIX_EPOCH).ok())
        .map(|v| v.as_millis() as i64)
        .unwrap_or(0)
}
fn db_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| err("DEMO_APP_DATA", e))?
        .join("demo-review/demo-review-v1.sqlite3"))
}

fn open_db(app: &AppHandle) -> Result<Connection, AppError> {
    let path = db_path(app)?;
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| err("DEMO_DB_CREATE", e))?;
    let db = Connection::open(path).map_err(|e| err("DEMO_DB_OPEN", e))?;
    db.busy_timeout(Duration::from_secs(3))
        .map_err(|e| err("DEMO_DB_BUSY", e))?;
    db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
      CREATE TABLE IF NOT EXISTS demo_roots(id INTEGER PRIMARY KEY,path TEXT NOT NULL,canonical_path TEXT NOT NULL UNIQUE,enabled INTEGER NOT NULL DEFAULT 1,scan_depth INTEGER NOT NULL DEFAULT 5,last_scan_at INTEGER,last_error TEXT,created_at INTEGER NOT NULL,origin TEXT NOT NULL DEFAULT 'manual_legacy');
      CREATE TABLE IF NOT EXISTS demo_files(id INTEGER PRIMARY KEY,path TEXT NOT NULL,canonical_path TEXT NOT NULL UNIQUE,file_name TEXT NOT NULL,size_bytes INTEGER NOT NULL,mtime_ms INTEGER NOT NULL,status TEXT NOT NULL,error_code TEXT,error_detail TEXT,map_name TEXT,total_rounds INTEGER,kills INTEGER,parsed_at INTEGER,report_json TEXT,source TEXT NOT NULL,discovered_at INTEGER NOT NULL,report_schema_version INTEGER NOT NULL DEFAULT 1,parser_adapter_version TEXT NOT NULL DEFAULT '1',metrics_version TEXT NOT NULL DEFAULT 'events-v1',scoreboard_status TEXT NOT NULL DEFAULT 'unavailable');
      CREATE INDEX IF NOT EXISTS demo_files_mtime ON demo_files(mtime_ms DESC);
      CREATE INDEX IF NOT EXISTS demo_files_status ON demo_files(status);
      CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY,value_json TEXT NOT NULL,updated_at INTEGER NOT NULL);
      ").map_err(|e| err("DEMO_DB_MIGRATION", e))?;
    let version: i64 = db
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
    if version < 2 {
        let columns = db
            .prepare("PRAGMA table_info(demo_roots)")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<BTreeSet<_>, _>>()
            })
            .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        if !columns.contains("origin") {
            db.execute(
                "ALTER TABLE demo_roots ADD COLUMN origin TEXT NOT NULL DEFAULT 'manual_legacy'",
                [],
            )
            .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        }
        for (name, definition) in [
            ("report_schema_version", "INTEGER NOT NULL DEFAULT 1"),
            ("parser_adapter_version", "TEXT NOT NULL DEFAULT '1'"),
            ("metrics_version", "TEXT NOT NULL DEFAULT 'events-v1'"),
            ("scoreboard_status", "TEXT NOT NULL DEFAULT 'unavailable'"),
        ] {
            let has_column = db
                .prepare("PRAGMA table_info(demo_files)")
                .and_then(|mut stmt| {
                    stmt.query_map([], |row| row.get::<_, String>(1))?
                        .collect::<Result<BTreeSet<_>, _>>()
                })
                .map_err(|e| err("DEMO_DB_MIGRATION", e))?
                .contains(name);
            if !has_column {
                db.execute(
                    &format!("ALTER TABLE demo_files ADD COLUMN {name} {definition}"),
                    [],
                )
                .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
            }
        }
        db.execute_batch("UPDATE demo_roots SET origin='manual_legacy' WHERE origin IS NULL OR origin=''; PRAGMA user_version=2;").map_err(|e| err("DEMO_DB_MIGRATION", e))?;
    }
    Ok(db)
}

pub fn initialize(app: &AppHandle) -> Result<(), AppError> {
    open_db(app)?;
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let stale = open_db(&handle).and_then(|db| {
            let mut stmt = db.prepare("SELECT path FROM demo_files WHERE report_schema_version<?1 OR parser_adapter_version<>?2 ORDER BY mtime_ms DESC").map_err(|e| err("DEMO_DB_QUERY", e))?;
            let rows = stmt.query_map(params![REPORT_SCHEMA_VERSION, PARSER_ADAPTER_VERSION], |row| row.get::<_, String>(0)).map_err(|e| err("DEMO_DB_QUERY", e))?;
            Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
        }).unwrap_or_default();
        for path in stale {
            let _ = import_file(&handle, &path, "schema-migration", true);
        }
    });
    Ok(())
}

pub fn list_roots(app: &AppHandle) -> Result<Vec<DemoRoot>, AppError> {
    let db = open_db(app)?;
    let mut stmt = db.prepare("SELECT id,path,enabled,scan_depth,last_scan_at,last_error,origin FROM demo_roots ORDER BY created_at").map_err(|e| err("DEMO_DB_QUERY", e))?;
    let rows = stmt
        .query_map([], |r| {
            Ok(DemoRoot {
                id: r.get(0)?,
                path: r.get(1)?,
                enabled: r.get::<_, i64>(2)? != 0,
                scan_depth: r.get(3)?,
                last_scan_at: r.get(4)?,
                last_error: r.get(5)?,
                origin: r.get(6)?,
            })
        })
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn add_root(app: &AppHandle, path: &str, depth: i64) -> Result<DemoRoot, AppError> {
    if ![0, 1, 2, 3, 5].contains(&depth) {
        return Err(err("DEMO_DEPTH_INVALID", "扫描深度必须为 0/1/2/3/5。"));
    }
    let canonical = dunce::canonicalize(path).map_err(|e| err("DEMO_ROOT_INVALID", e))?;
    if !canonical.is_dir() {
        return Err(err("DEMO_ROOT_INVALID", "所选路径不是目录。"));
    }
    let db = open_db(app)?;
    let display = canonical.display().to_string();
    let now = now_ms();
    db.execute("INSERT INTO demo_roots(path,canonical_path,enabled,scan_depth,created_at,origin) VALUES(?1,?2,1,?3,?4,'manual') ON CONFLICT(canonical_path) DO UPDATE SET enabled=1,scan_depth=excluded.scan_depth,path=excluded.path,origin=CASE WHEN demo_roots.origin='selected_cs2_root' THEN 'manual' ELSE demo_roots.origin END",params![display,display.to_lowercase(),depth,now]).map_err(|e|err("DEMO_ROOT_SAVE",e))?;
    refresh_watcher(app)?;
    list_roots(app)?
        .into_iter()
        .find(|r| r.path.eq_ignore_ascii_case(&display))
        .ok_or_else(|| err("DEMO_ROOT_SAVE", "目录保存后无法回读。"))
}

pub fn ensure_default_root(app: &AppHandle, root_path: &str) -> Result<DemoRoot, AppError> {
    let root = cs2::normalize_root(root_path)?;
    let display = root.display().to_string();
    let canonical = display.to_lowercase();
    let db = open_db(app)?;
    let now = now_ms();
    db.execute(
        "UPDATE demo_roots SET enabled=0 WHERE origin='selected_cs2_root' AND canonical_path<>?1",
        [&canonical],
    )
    .map_err(|e| err("DEMO_ROOT_SAVE", e))?;
    db.execute("INSERT INTO demo_roots(path,canonical_path,enabled,scan_depth,created_at,origin) VALUES(?1,?2,1,5,?3,'selected_cs2_root') ON CONFLICT(canonical_path) DO UPDATE SET path=excluded.path,enabled=CASE WHEN demo_roots.origin='selected_cs2_root' THEN 1 ELSE demo_roots.enabled END,scan_depth=CASE WHEN demo_roots.origin='selected_cs2_root' THEN 5 ELSE demo_roots.scan_depth END,origin=demo_roots.origin", params![display, canonical, now]).map_err(|e| err("DEMO_ROOT_SAVE", e))?;
    refresh_watcher(app)?;
    list_roots(app)?
        .into_iter()
        .find(|root| root.path.eq_ignore_ascii_case(&display))
        .ok_or_else(|| err("DEMO_ROOT_SAVE", "默认 CS2 目录保存后无法回读。"))
}

pub fn update_root(app: &AppHandle, id: i64, enabled: bool, depth: i64) -> Result<(), AppError> {
    if ![0, 1, 2, 3, 5].contains(&depth) {
        return Err(err("DEMO_DEPTH_INVALID", "扫描深度无效。"));
    }
    open_db(app)?
        .execute(
            "UPDATE demo_roots SET enabled=?2,scan_depth=?3,origin=CASE WHEN origin='selected_cs2_root' THEN 'manual' ELSE origin END WHERE id=?1",
            params![id, enabled as i64, depth],
        )
        .map_err(|e| err("DEMO_ROOT_SAVE", e))?;
    refresh_watcher(app)
}
pub fn remove_root(app: &AppHandle, id: i64) -> Result<(), AppError> {
    open_db(app)?
        .execute("DELETE FROM demo_roots WHERE id=?1", [id])
        .map_err(|e| err("DEMO_ROOT_REMOVE", e))?;
    refresh_watcher(app)
}

pub fn refresh_watcher(app: &AppHandle) -> Result<(), AppError> {
    let state = app.state::<DemoWatcherState>();
    let mut slot = state
        .0
        .lock()
        .map_err(|_| err("DEMO_WATCHER_LOCK", "监听器状态不可用。"))?;
    let handle = app.clone();
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            if event
                .paths
                .iter()
                .any(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("dem")))
            {
                let _ = handle.emit("demo://filesystem-changed", ());
            }
        }
    })
    .map_err(|e| err("DEMO_WATCHER", e))?;
    for root in list_roots(app)?.into_iter().filter(|r| r.enabled) {
        let _ = watcher.watch(Path::new(&root.path), RecursiveMode::Recursive);
    }
    *slot = Some(watcher);
    Ok(())
}

fn collect(root: &Path, max_depth: i64) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut queue = VecDeque::from([(root.to_path_buf(), 0i64)]);
    while let Some((dir, depth)) = queue.pop_front() {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_symlink() {
                continue;
            }
            let path = entry.path();
            if ft.is_file()
                && path
                    .extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("dem"))
            {
                out.push(path)
            } else if ft.is_dir() && depth < max_depth {
                queue.push_back((path, depth + 1))
            }
        }
    }
    out.sort_by_key(|p| {
        fs::metadata(p)
            .ok()
            .map(|m| std::cmp::Reverse(mtime_ms(&m)))
    });
    out
}

fn validate(path: &Path) -> Result<fs::Metadata, AppError> {
    let meta = fs::metadata(path).map_err(|e| err("DEMO_FILE_READ", e))?;
    if meta.len() < 1024 {
        return Err(err("DEMO_FILE_TOO_SMALL", "Demo 文件小于 1 KiB。"));
    }
    let mut header = [0u8; 8];
    fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .map_err(|e| err("DEMO_FILE_READ", e))?;
    if !header.starts_with(b"PBDEMS2") && !header.starts_with(b"HL2DEMO") {
        return Err(err("DEMO_HEADER_UNSUPPORTED", "不是受支持的 Demo header。"));
    }
    Ok(meta)
}

fn wait_until_stable(path: &Path) -> Result<fs::Metadata, AppError> {
    let mut previous = validate(path)?;
    for _ in 0..2 {
        std::thread::sleep(Duration::from_millis(250));
        let current = validate(path)?;
        if current.len() != previous.len() || mtime_ms(&current) != mtime_ms(&previous) {
            return Err(err(
                "DEMO_WAITING_STABLE",
                "Demo 文件仍在写入，请稍后重试。",
            ));
        }
        previous = current;
    }
    Ok(previous)
}

pub fn import_file(
    app: &AppHandle,
    path: &str,
    source: &str,
    force_reparse: bool,
) -> Result<DemoImportResult, AppError> {
    let canonical = dunce::canonicalize(path).map_err(|e| err("DEMO_FILE_READ", e))?;
    if canonical
        .extension()
        .map_or(true, |x| !x.eq_ignore_ascii_case("dem"))
    {
        return Err(err("DEMO_EXTENSION_UNSUPPORTED", "第一版仅支持 .dem。"));
    }
    let meta = wait_until_stable(&canonical)?;
    let display = canonical.display().to_string();
    let db = open_db(app)?;
    let now = now_ms();
    let existing: Option<(i64, String, i64, i64, i64, String)> = db
        .query_row(
            "SELECT id,status,size_bytes,mtime_ms,report_schema_version,parser_adapter_version FROM demo_files WHERE canonical_path=?1",
            [display.to_lowercase()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
        )
        .optional()
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    if let Some((id, status, size, mtime, schema, adapter)) = existing {
        if !force_reparse
            && status == "done"
            && size == meta.len() as i64
            && mtime == mtime_ms(&meta)
            && schema >= REPORT_SCHEMA_VERSION as i64
            && adapter == PARSER_ADAPTER_VERSION
        {
            return Ok(DemoImportResult {
                demo_file_id: id,
                status,
            });
        }
    }
    db.execute("INSERT INTO demo_files(path,canonical_path,file_name,size_bytes,mtime_ms,status,source,discovered_at) VALUES(?1,?2,?3,?4,?5,'parsing',?6,?7) ON CONFLICT(canonical_path) DO UPDATE SET size_bytes=excluded.size_bytes,mtime_ms=excluded.mtime_ms,status='parsing',error_code=NULL,error_detail=NULL",params![display,display.to_lowercase(),canonical.file_name().unwrap_or_default().to_string_lossy(),meta.len() as i64,mtime_ms(&meta),source,now]).map_err(|e|err("DEMO_DB_SAVE",e))?;
    let id: i64 = db
        .query_row(
            "SELECT id FROM demo_files WHERE canonical_path=?1",
            [display.to_lowercase()],
            |r| r.get(0),
        )
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    match parse_report(id, &canonical, &meta) {
        Ok(report) => {
            let json =
                serde_json::to_string(&report).map_err(|e| err("DEMO_REPORT_SERIALIZE", e))?;
            db.execute("UPDATE demo_files SET status='done',map_name=?2,total_rounds=?3,kills=?4,parsed_at=?5,report_json=?6,report_schema_version=?7,parser_adapter_version=?8,metrics_version=?9,scoreboard_status=?10,error_code=NULL,error_detail=NULL WHERE id=?1",params![id,report.summary.map_name,report.summary.total_rounds,report.summary.total_kills,report.summary.parsed_at,json,REPORT_SCHEMA_VERSION,PARSER_ADAPTER_VERSION,METRICS_VERSION,report.data_quality.scoreboard_status]).map_err(|e|err("DEMO_DB_SAVE",e))?;
            Ok(DemoImportResult {
                demo_file_id: id,
                status: "done".into(),
            })
        }
        Err(e) => {
            let detail = e.into_string();
            let code = detail
                .split(']')
                .next()
                .unwrap_or("[DEMO_PARSE_FAILED")
                .trim_start_matches('[');
            let _ = db.execute("UPDATE demo_files SET status=CASE WHEN report_json IS NULL THEN 'error' ELSE 'done' END,error_code=?2,error_detail=?3 WHERE id=?1", params![id, code, detail]);
            Ok(DemoImportResult {
                demo_file_id: id,
                status: "error".into(),
            })
        }
    }
}

pub fn scan(app: &AppHandle) -> Result<DemoScanResult, AppError> {
    let mut out = DemoScanResult {
        discovered: 0,
        parsed: 0,
        failed: 0,
    };
    for root in list_roots(app)?.into_iter().filter(|r| r.enabled) {
        for path in collect(Path::new(&root.path), root.scan_depth) {
            out.discovered += 1;
            match import_file(app, &path.display().to_string(), "scan", false) {
                Ok(v) if v.status == "done" => out.parsed += 1,
                _ => out.failed += 1,
            }
        }
        let _ = open_db(app)?.execute(
            "UPDATE demo_roots SET last_scan_at=?2,last_error=NULL WHERE id=?1",
            params![root.id, now_ms()],
        );
    }
    Ok(out)
}

pub fn list(
    app: &AppHandle,
    query: &str,
    status: &str,
    page: i64,
    page_size: i64,
) -> Result<DemoListPage, AppError> {
    let page = page.max(1);
    let size = if [25, 50, 100].contains(&page_size) {
        page_size
    } else {
        25
    };
    let q = format!("%{}%", query.trim());
    let status_arg = if status == "all" { "" } else { status };
    let db = open_db(app)?;
    let total=db.query_row("SELECT COUNT(*) FROM demo_files WHERE (?1='' OR file_name LIKE ?2 OR map_name LIKE ?2) AND (?3='' OR status=?3)",params![query.trim(),q,status_arg],|r|r.get(0)).map_err(|e|err("DEMO_DB_QUERY",e))?;
    let mut stmt=db.prepare("SELECT id,file_name,path,size_bytes,mtime_ms,status,error_code,map_name,total_rounds,kills,parsed_at FROM demo_files WHERE (?1='' OR file_name LIKE ?2 OR map_name LIKE ?2) AND (?3='' OR status=?3) ORDER BY mtime_ms DESC LIMIT ?4 OFFSET ?5").map_err(|e|err("DEMO_DB_QUERY",e))?;
    let rows = stmt
        .query_map(
            params![query.trim(), q, status_arg, size, (page - 1) * size],
            |r| {
                Ok(DemoListItem {
                    id: r.get(0)?,
                    file_name: r.get(1)?,
                    path: r.get(2)?,
                    size_bytes: r.get(3)?,
                    mtime_ms: r.get(4)?,
                    status: r.get(5)?,
                    error_code: r.get(6)?,
                    map_name: r.get(7)?,
                    total_rounds: r.get(8)?,
                    kills: r.get(9)?,
                    parsed_at: r.get(10)?,
                })
            },
        )
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    Ok(DemoListPage {
        items: rows.filter_map(Result::ok).collect(),
        total,
        page,
        page_size: size,
    })
}
pub fn report(app: &AppHandle, id: i64) -> Result<DemoReport, AppError> {
    let json: Option<String> = open_db(app)?
        .query_row(
            "SELECT report_json FROM demo_files WHERE id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| err("DEMO_DB_QUERY", e))?
        .flatten();
    serde_json::from_str(&json.ok_or_else(|| err("DEMO_REPORT_NOT_READY", "报告尚未生成。"))?)
        .map_err(|e| err("DEMO_REPORT_INVALID", e))
}

pub fn path_for_id(app: &AppHandle, id: i64) -> Result<String, AppError> {
    open_db(app)?
        .query_row("SELECT path FROM demo_files WHERE id=?1", [id], |r| {
            r.get(0)
        })
        .map_err(|e| err("DEMO_NOT_FOUND", e))
}

pub fn observe_assistant_launch(app: AppHandle, started_at: i64) {
    tauri::async_runtime::spawn_blocking(move || {
        let mut seen = false;
        for _ in 0..360 {
            match cs2::check_cs2_process() {
                Ok(true) => seen = true,
                Ok(false) if seen => break,
                _ => {}
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        if !seen {
            return;
        }
        let _ = scan(&app);
        if let Ok(page) = list(&app, "", "done", 1, 25) {
            if let Some(item) = page
                .items
                .into_iter()
                .find(|item| item.mtime_ms >= started_at - 5_000)
            {
                let _ = app.emit("demo://report-ready", item.id);
            }
        }
    });
}

fn field<'a>(event: &'a GameEvent, name: &str) -> Option<&'a Variant> {
    event
        .fields
        .iter()
        .find(|f| f.name == name)
        .and_then(|f| f.data.as_ref())
}
fn text(event: &GameEvent, name: &str) -> Option<String> {
    match field(event, name) {
        Some(Variant::String(v)) => Some(v.clone()),
        Some(Variant::I32(v)) => Some(v.to_string()),
        Some(Variant::U32(v)) => Some(v.to_string()),
        Some(Variant::U64(v)) => Some(v.to_string()),
        _ => None,
    }
}
fn integer(event: &GameEvent, name: &str) -> Option<i32> {
    match field(event, name) {
        Some(Variant::I32(v)) => Some(*v),
        Some(Variant::U32(v)) => i32::try_from(*v).ok(),
        _ => None,
    }
}
fn boolean(event: &GameEvent, name: &str) -> Option<bool> {
    match field(event, name) {
        Some(Variant::Bool(v)) => Some(*v),
        _ => None,
    }
}
fn last_u32(column: Option<&cs2_demoparser::second_pass::variants::PropColumn>) -> Option<u32> {
    match column?.data.as_ref()? {
        VarVec::U32(values) => values.iter().rev().flatten().next().copied(),
        VarVec::I32(values) => values
            .iter()
            .rev()
            .flatten()
            .next()
            .and_then(|value| u32::try_from(*value).ok()),
        _ => None,
    }
}

fn team_label(team_number: Option<i32>) -> Option<String> {
    match team_number {
        Some(2) => Some("T".into()),
        Some(3) => Some("CT".into()),
        _ => None,
    }
}

fn event_identity(
    event: &GameEvent,
    prefix: &str,
) -> Option<(String, Option<String>, Option<i32>, bool)> {
    let steam_id = text(event, &format!("{prefix}_steamid")).filter(|value| {
        value
            .parse::<u64>()
            .is_ok_and(|steamid| steamid >= MIN_STEAM_ID64)
    });
    let user_id = integer(event, &format!("{prefix}_userid")).filter(|value| *value > 0);
    let name = text(event, &format!("{prefix}_name")).filter(|value| !value.trim().is_empty());
    let key = steam_id
        .as_ref()
        .map(|value| format!("steam:{value}"))
        .or_else(|| user_id.map(|value| format!("bot:{value}")))
        .or_else(|| name.as_ref().map(|value| format!("event:{value}")))?;
    Some((key, name, user_id, steam_id.is_none()))
}

fn ensure_event_player(
    players: &mut Vec<DemoPlayer>,
    identity: &(String, Option<String>, Option<i32>, bool),
) -> usize {
    if let Some(index) = players.iter().position(|player| player.key == identity.0) {
        return index;
    }
    if let Some(user_id) = identity.2 {
        if let Some(index) = players
            .iter()
            .position(|player| player.user_id == Some(user_id))
        {
            return index;
        }
    }
    if let Some(name) = &identity.1 {
        let matches = players
            .iter()
            .enumerate()
            .filter(|(_, player)| player.name.as_deref() == Some(name))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if matches.len() == 1 {
            return matches[0];
        }
    }
    players.push(DemoPlayer {
        key: identity.0.clone(),
        steam_id: identity.0.strip_prefix("steam:").map(str::to_owned),
        user_id: identity.2,
        name: identity.1.clone(),
        is_bot: identity.3,
        team_number: None,
        team: None,
        kills: Some(0),
        deaths: Some(0),
        assists: Some(0),
        damage: Some(0),
        headshots: Some(0),
        identity_source: "event".into(),
        stats_source: "event_aggregate".into(),
    });
    players.len() - 1
}

fn build_scoreboard(
    output: &cs2_demoparser::parse_demo::DemoOutput,
    entity_status: &str,
) -> (Vec<DemoPlayer>, DemoDataQuality) {
    let mut players = Vec::<DemoPlayer>::new();
    for source in output.player_md.iter().chain(output.roster.iter()) {
        if source.team_number == Some(1)
            || source
                .name
                .as_deref()
                .map_or(true, |name| name.trim().is_empty())
        {
            continue;
        }
        let steam_id = source
            .steamid
            .filter(|value| *value >= MIN_STEAM_ID64)
            .map(|value| value.to_string());
        let key = steam_id
            .as_ref()
            .map(|value| format!("steam:{value}"))
            .or_else(|| source.user_id.map(|value| format!("bot:{value}")))
            .unwrap_or_else(|| format!("bot:controller:{}", source.controller_id.unwrap_or(-1)));
        let existing = players
            .iter()
            .position(|player| player.key == key)
            .or_else(|| {
                if !source.is_bot {
                    return None;
                }
                let matches = players
                    .iter()
                    .enumerate()
                    .filter(|(_, player)| player.is_bot && player.name == source.name)
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>();
                (matches.len() == 1).then(|| matches[0])
            });
        if let Some(index) = existing {
            let player = &mut players[index];
            if player.user_id.is_none() {
                player.user_id = source.user_id;
            }
            if player.team_number.is_none() {
                player.team_number = source.team_number;
                player.team = team_label(source.team_number);
            }
            if let Some(user_id) = source.user_id.filter(|_| !player.key.starts_with("steam:")) {
                player.key = format!("bot:{user_id}");
            }
            continue;
        }
        players.push(DemoPlayer {
            key,
            steam_id,
            user_id: source.user_id,
            name: source.name.clone(),
            is_bot: source.is_bot,
            team_number: source.team_number,
            team: team_label(source.team_number),
            kills: None,
            deaths: None,
            assists: None,
            damage: None,
            headshots: None,
            identity_source: if source.controller_id.is_some() {
                "controller".into()
            } else if source.user_id.is_some() {
                "userinfo".into()
            } else {
                "end_message".into()
            },
            stats_source: "partial".into(),
        });
    }

    let prop_id = |name: &str| {
        output
            .prop_controller
            .prop_infos
            .iter()
            .find(|prop| prop.prop_friendly_name == name)
            .map(|prop| prop.id)
    };
    for player in &mut players {
        let Some(steamid) = player
            .steam_id
            .as_ref()
            .and_then(|value| value.parse::<u64>().ok())
        else {
            continue;
        };
        let Some(columns) = output.df_per_player.get(&steamid) else {
            continue;
        };
        player.kills = prop_id("kills_total").and_then(|id| last_u32(columns.get(&id)));
        player.deaths = prop_id("deaths_total").and_then(|id| last_u32(columns.get(&id)));
        player.assists = prop_id("assists_total").and_then(|id| last_u32(columns.get(&id)));
        player.damage = prop_id("damage_total").and_then(|id| last_u32(columns.get(&id)));
        player.headshots =
            prop_id("headshot_kills_total").and_then(|id| last_u32(columns.get(&id)));
        if [
            player.kills,
            player.deaths,
            player.assists,
            player.damage,
            player.headshots,
        ]
        .iter()
        .all(Option::is_some)
        {
            player.stats_source = "controller_total".into();
        }
    }

    for event in &output.game_events {
        if event.name == "player_hurt" {
            if let Some(attacker) = event_identity(event, "attacker") {
                let index = ensure_event_player(&mut players, &attacker);
                if players[index].stats_source != "controller_total" {
                    players[index].damage =
                        Some(players[index].damage.unwrap_or(0).saturating_add(
                            integer(event, "dmg_health").unwrap_or(0).max(0) as u32,
                        ));
                    players[index].stats_source = "event_aggregate".into();
                }
            }
        } else if event.name == "player_death" {
            let victim = event_identity(event, "user");
            let attacker = event_identity(event, "attacker");
            let assister = event_identity(event, "assister");
            if let Some(victim) = &victim {
                let index = ensure_event_player(&mut players, victim);
                if players[index].stats_source != "controller_total" {
                    players[index].deaths = Some(players[index].deaths.unwrap_or(0) + 1);
                    players[index].stats_source = "event_aggregate".into();
                }
            }
            if let Some(attacker) = &attacker {
                if victim
                    .as_ref()
                    .map_or(true, |victim| victim.0 != attacker.0)
                {
                    let index = ensure_event_player(&mut players, attacker);
                    if players[index].stats_source != "controller_total" {
                        players[index].kills = Some(players[index].kills.unwrap_or(0) + 1);
                        if boolean(event, "headshot") == Some(true) {
                            players[index].headshots =
                                Some(players[index].headshots.unwrap_or(0) + 1);
                        }
                        players[index].stats_source = "event_aggregate".into();
                    }
                }
            }
            if let Some(assister) = &assister {
                let index = ensure_event_player(&mut players, assister);
                if players[index].stats_source != "controller_total" {
                    players[index].assists = Some(players[index].assists.unwrap_or(0) + 1);
                    players[index].stats_source = "event_aggregate".into();
                }
            }
        }
    }
    players.retain(|player| {
        player.team_number != Some(1)
            && player
                .name
                .as_deref()
                .is_some_and(|name| !name.trim().is_empty())
    });
    players.sort_by_key(|player| {
        (
            player.team_number.unwrap_or(9),
            player.name.clone().unwrap_or_default(),
        )
    });
    let complete = entity_status == "strict"
        && !players.is_empty()
        && players.iter().all(|player| {
            [
                player.kills,
                player.deaths,
                player.assists,
                player.damage,
                player.headshots,
            ]
            .iter()
            .all(Option::is_some)
        });
    let status = if players.is_empty() {
        "unavailable"
    } else if complete {
        "complete"
    } else {
        "partial"
    };
    let mut warnings = Vec::new();
    if entity_status == "recovered" {
        warnings.push(
            "实体解析遇到已知 IllegalPathOp，已从 userinfo 与事件安全恢复；部分终局统计不可用。"
                .into(),
        );
    }
    if status == "partial" && warnings.is_empty() {
        warnings.push(format!(
            "已恢复 {} 名玩家；部分终局统计不可用。",
            players.len()
        ));
    }
    if status == "unavailable" {
        warnings.push("解析器未能从 controller、userinfo 或事件中恢复玩家身份。".into());
    }
    (
        players,
        DemoDataQuality {
            scoreboard_status: status.into(),
            warnings,
            entity_parse_status: entity_status.into(),
        },
    )
}

fn parse_report(id: i64, path: &Path, meta: &fs::Metadata) -> Result<DemoReport, AppError> {
    let bytes = fs::read(path).map_err(|e| err("DEMO_FILE_READ", e))?;
    let h = create_huffman_lookup_table();
    let wanted: Vec<String> = [
        "round_start",
        "round_freeze_end",
        "round_end",
        "round_officially_ended",
        "player_death",
        "player_hurt",
        "bomb_beginplant",
        "bomb_planted",
        "bomb_begindefuse",
        "bomb_defused",
        "bomb_exploded",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let player_props = [
        (
            "kills_total",
            "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iKills",
        ),
        (
            "deaths_total",
            "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iDeaths",
        ),
        (
            "assists_total",
            "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iAssists",
        ),
        (
            "damage_total",
            "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iDamage",
        ),
        (
            "headshot_kills_total",
            "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iHeadShotKills",
        ),
    ];
    let parse = |parse_ents: bool| {
        let settings = ParserInputs {
            real_name_to_og_name: player_props
                .iter()
                .map(|(friendly, real)| (real.to_string(), friendly.to_string()))
                .collect(),
            wanted_players: vec![],
            wanted_player_props: player_props
                .iter()
                .map(|(_, real)| real.to_string())
                .collect(),
            wanted_other_props: vec![],
            wanted_prop_states: Default::default(),
            wanted_ticks: vec![],
            wanted_events: wanted.clone(),
            parse_ents,
            parse_projectiles: false,
            parse_grenades: false,
            only_header: false,
            only_convars: false,
            huffman_lookup_table: &h,
            order_by_steamid: true,
            list_props: false,
            fallback_bytes: None,
        };
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Parser::new(settings, ParsingMode::ForceSingleThreaded).parse_demo(&bytes)
        }))
        .map_err(|_| "ParserPanic".to_string())?
        .map_err(|error| format!("{error:?}"))
    };
    let (output, entity_status) = match parse(true) {
        Ok(output) => (output, "strict"),
        Err(error) if error.contains("IllegalPathOp") => (
            parse(false).map_err(|fallback| {
                err(
                    "DEMO_PARSER_UNSUPPORTED",
                    format!("严格实体解析失败：{error}；userinfo 回退失败：{fallback}"),
                )
            })?,
            "recovered",
        ),
        Err(error) => return Err(err("DEMO_PARSER_UNSUPPORTED", error)),
    };
    let (players, data_quality) = build_scoreboard(&output, entity_status);
    let header = output
        .header
        .unwrap_or_default()
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let mut rounds: Vec<DemoRound> = Vec::new();
    let mut current: Option<usize> = None;
    for event in output.game_events {
        if event.name == "round_start" {
            rounds.push(DemoRound {
                number: (rounds.len() + 1) as u32,
                start_tick: Some(event.tick),
                end_tick: None,
                winner: None,
                reason: None,
                kills: vec![],
                bomb_events: vec![],
            });
            current = Some(rounds.len() - 1);
            continue;
        }
        if event.name == "round_freeze_end" {
            if let Some(index) = current.filter(|index| rounds[*index].end_tick.is_none()) {
                rounds[index].start_tick = Some(event.tick);
            } else {
                rounds.push(DemoRound {
                    number: (rounds.len() + 1) as u32,
                    start_tick: Some(event.tick),
                    end_tick: None,
                    winner: None,
                    reason: None,
                    kills: vec![],
                    bomb_events: vec![],
                });
                current = Some(rounds.len() - 1);
            }
            continue;
        }
        if current.is_none() {
            rounds.push(DemoRound {
                number: 1,
                start_tick: None,
                end_tick: None,
                winner: None,
                reason: None,
                kills: vec![],
                bomb_events: vec![],
            });
            current = Some(0)
        }
        let round = &mut rounds[current.unwrap()];
        if event.name == "round_end" || event.name == "round_officially_ended" {
            round.end_tick = Some(event.tick);
            round.winner = text(&event, "winner");
            round.reason = text(&event, "reason");
            continue;
        }
        let item = DemoEvent {
            tick: event.tick,
            kind: event.name.clone(),
            actor: text(&event, "attacker_name").or_else(|| text(&event, "user_name")),
            target: text(&event, "user_name"),
            weapon: text(&event, "weapon"),
            headshot: boolean(&event, "headshot"),
            detail: text(&event, "site"),
        };
        if event.name == "player_death" {
            round.kills.push(item)
        } else if event.name.starts_with("bomb_") {
            round.bomb_events.push(item)
        }
    }
    let total_kills = rounds.iter().map(|r| r.kills.len() as u32).sum();
    let parsed = now_ms();
    Ok(DemoReport {
        schema_version: REPORT_SCHEMA_VERSION,
        parser_name: "laihoe-demoparser".into(),
        parser_commit: PARSER_COMMIT.into(),
        parser_adapter_version: PARSER_ADAPTER_VERSION.into(),
        metrics_version: METRICS_VERSION.into(),
        app_version: env!("CARGO_PKG_VERSION").into(),
        data_quality,
        players,
        summary: DemoSummary {
            demo_file_id: id,
            file_name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            path: path.display().to_string(),
            map_name: header.get("map_name").cloned(),
            server_name: header.get("server_name").cloned(),
            file_time_ms: mtime_ms(meta),
            size_bytes: meta.len() as i64,
            total_rounds: rounds.len() as u32,
            total_kills,
            team_a_score: None,
            team_b_score: None,
            parsed_at: parsed,
        },
        rounds,
    })
}

pub fn recording_desired(app: &AppHandle) -> Result<bool, AppError> {
    let db = open_db(app)?;
    Ok(db
        .query_row(
            "SELECT value_json FROM settings WHERE key='recording'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| err("DEMO_DB_QUERY", e))?
        .as_deref()
        == Some("true"))
}
pub fn set_recording_desired(app: &AppHandle, enabled: bool) -> Result<(), AppError> {
    open_db(app)?.execute("INSERT INTO settings(key,value_json,updated_at)VALUES('recording',?1,?2)ON CONFLICT(key)DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at",params![if enabled{"true"}else{"false"},now_ms()]).map_err(|e|err("DEMO_SETTINGS_SAVE",e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cs2_demoparser::second_pass::game_events::EventField;

    #[test]
    fn scoreboard_event_identity_keeps_bot_userids_distinct() {
        let event = |user_id| GameEvent {
            name: "player_death".into(),
            tick: 1,
            fields: vec![
                EventField {
                    name: "user_userid".into(),
                    data: Some(Variant::I32(user_id)),
                },
                EventField {
                    name: "user_name".into(),
                    data: Some(Variant::String("BOT Alpha".into())),
                },
            ],
        };
        assert_eq!(event_identity(&event(3), "user").unwrap().0, "bot:3");
        assert_eq!(event_identity(&event(7), "user").unwrap().0, "bot:7");
    }

    #[test]
    #[ignore = "uses a local user Demo selected through CS2AS_DEMO"]
    fn real_demo_scoreboard() {
        let path = PathBuf::from(std::env::var("CS2AS_DEMO").expect("CS2AS_DEMO path"));
        let meta = fs::metadata(&path).expect("Demo metadata");
        let report = parse_report(1, &path, &meta).expect("Demo parse");
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        assert_eq!(report.schema_version, REPORT_SCHEMA_VERSION);
        assert!(!report.players.is_empty(), "scoreboard must not be empty");
    }
}
