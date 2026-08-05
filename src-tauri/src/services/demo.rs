use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    ffi::OsString,
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use cs2_demoparser::{
    first_pass::parser_settings::ParserInputs,
    first_pass::prop_controller::TICK_ID,
    parse_demo::{Parser, ParsingMode},
    second_pass::{
        game_events::GameEvent,
        parser_settings::create_huffman_lookup_table,
        variants::{VarVec, Variant},
    },
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    demo::{map_metadata, parse_gate},
    errors::AppError,
    models::demo::*,
    services::cs2,
};

const PARSER_COMMIT: &str = "ba39cc44cd5abfd7f34df2b3c0a7dd3630048311";
pub(crate) const REPORT_SCHEMA_VERSION: u32 = 6;
pub(crate) const PARSER_ADAPTER_VERSION: &str = "9";
pub(crate) const METRICS_VERSION: &str = "simple-rating-v1";
const JOB_LEASE_MS: i64 = 30_000;
const MIN_STEAM_ID64: u64 = 76_561_197_960_265_728;

#[derive(Default)]
pub struct DemoWatcherState(pub Mutex<Option<RecommendedWatcher>>);

type PositionCacheKey = (i64, i64, i64);
type PositionCache = Mutex<VecDeque<(PositionCacheKey, Arc<Vec<PositionPoint>>)>>;
type ExistingDemoFingerprint = (i64, String, i64, i64, Option<String>, i64, String, String);
const POSITION_CACHE_CAPACITY: usize = 8;
static POSITION_CACHE: OnceLock<PositionCache> = OnceLock::new();

fn position_cache() -> &'static PositionCache {
    POSITION_CACHE.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn cached_positions(key: PositionCacheKey) -> Option<Vec<PositionPoint>> {
    let mut cache = position_cache().lock().ok()?;
    let index = cache.iter().position(|(candidate, _)| *candidate == key)?;
    let entry = cache.remove(index)?;
    let points = entry.1.as_ref().clone();
    cache.push_front(entry);
    Some(points)
}

fn cache_positions(key: PositionCacheKey, points: &[PositionPoint]) {
    let Ok(mut cache) = position_cache().lock() else {
        return;
    };
    cache.retain(|(candidate, _)| *candidate != key);
    cache.push_front((key, Arc::new(points.to_vec())));
    cache.truncate(POSITION_CACHE_CAPACITY);
}

fn invalidate_position_cache(demo_id: i64, sampling_hz: i64) {
    if let Ok(mut cache) = position_cache().lock() {
        cache.retain(|((candidate_demo, _, candidate_hz), _)| {
            *candidate_demo != demo_id || *candidate_hz != sampling_hz
        });
    }
}

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

fn sha256_file(path: &Path) -> Result<String, AppError> {
    let mut file = fs::File::open(path).map_err(|e| err("DEMO_HASH_OPEN", e))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| err("DEMO_HASH_READ", e))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:X}", digest.finalize()))
}
fn db_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| err("DEMO_APP_DATA", e))?
        .join("demo-review/demo-review-v1.sqlite3"))
}

fn table_columns(db: &Connection, table: &str) -> Result<BTreeSet<String>, AppError> {
    db.prepare(&format!("PRAGMA table_info({table})"))
        .and_then(|mut stmt| {
            stmt.query_map([], |row| row.get::<_, String>(1))?
                .collect::<Result<BTreeSet<_>, _>>()
        })
        .map_err(|e| err("DEMO_DB_MIGRATION", e))
}

fn seed_map_metadata(db: &Connection) -> Result<(), AppError> {
    let maps = map_metadata::embedded().map_err(|e| err("DEMO_MAP_METADATA", e))?;
    if maps.len() != 44 {
        return Err(err(
            "DEMO_MAP_METADATA",
            format!("expected 44 maps, got {}", maps.len()),
        ));
    }
    if maps
        .iter()
        .any(|map| map.upstream_commit != map_metadata::UPSTREAM_COMMIT)
    {
        return Err(err(
            "DEMO_MAP_METADATA",
            "embedded upstream commit mismatch",
        ));
    }
    let tx = db
        .unchecked_transaction()
        .map_err(|e| err("DEMO_MAP_METADATA", e))?;
    for map in maps {
        tx.execute(
            "INSERT INTO map_metadata(map_name,upstream_commit,asset_sha256,pos_x,pos_y,scale,threshold_z,updated_at,radar_asset,lower_radar_asset,lower_asset_sha256,radar_size,source)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,'cs-demo-manager')
             ON CONFLICT(map_name) DO UPDATE SET upstream_commit=excluded.upstream_commit,asset_sha256=excluded.asset_sha256,pos_x=excluded.pos_x,pos_y=excluded.pos_y,scale=excluded.scale,threshold_z=excluded.threshold_z,updated_at=excluded.updated_at,radar_asset=excluded.radar_asset,lower_radar_asset=excluded.lower_radar_asset,lower_asset_sha256=excluded.lower_asset_sha256,radar_size=excluded.radar_size,source=excluded.source
             WHERE map_metadata.source='cs-demo-manager'",
            params![map.name,map.upstream_commit,map.radar_sha256,map.position_x,map.position_y,map.scale,map.threshold_z,now_ms(),map.radar_asset,map.lower_radar_asset,map.lower_radar_sha256,map.radar_size],
        ).map_err(|e| err("DEMO_MAP_METADATA", e))?;
    }
    let count: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM map_metadata WHERE source='cs-demo-manager'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| err("DEMO_MAP_METADATA", e))?;
    if count != 44 {
        return Err(err(
            "DEMO_MAP_METADATA",
            format!("seeded {count} upstream maps"),
        ));
    }
    tx.commit().map_err(|e| err("DEMO_MAP_METADATA", e))
}

fn open_db(app: &AppHandle) -> Result<Connection, AppError> {
    let path = db_path(app)?;
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| err("DEMO_DB_CREATE", e))?;
    let db = Connection::open(&path).map_err(|e| err("DEMO_DB_OPEN", e))?;
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
    if version < 4 {
        db.execute_batch("PRAGMA user_version=4;")
            .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
    }
    if version < 5 {
        let backup = path.with_extension(format!("sqlite3.v4-{}.bak", now_ms()));
        db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| err("DEMO_DB_BACKUP", e))?;
        db.execute("VACUUM INTO ?1", [backup.to_string_lossy().as_ref()])
            .map_err(|e| err("DEMO_DB_BACKUP", e))?;
        let tx = db
            .unchecked_transaction()
            .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        let columns = tx
            .prepare("PRAGMA table_info(demo_files)")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<BTreeSet<_>, _>>()
            })
            .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        if !columns.contains("checksum") {
            tx.execute("ALTER TABLE demo_files ADD COLUMN checksum TEXT", [])
                .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        }
        tx.execute_batch(
            "DROP INDEX IF EXISTS demo_files_checksum_unique;
             CREATE INDEX IF NOT EXISTS demo_files_checksum ON demo_files(checksum);
             CREATE TABLE IF NOT EXISTS demo_paths(
               id INTEGER PRIMARY KEY,
               demo_id INTEGER NOT NULL REFERENCES demo_files(id) ON DELETE CASCADE,
               canonical_path TEXT NOT NULL UNIQUE,
               exists_now INTEGER NOT NULL DEFAULT 1,
               last_seen_at INTEGER NOT NULL
             );
             INSERT OR IGNORE INTO demo_paths(demo_id, canonical_path, exists_now, last_seen_at)
               SELECT id, canonical_path, 1, COALESCE(discovered_at, 0) FROM demo_files;
             CREATE TABLE IF NOT EXISTS analysis_jobs(
               id INTEGER PRIMARY KEY,
               demo_id INTEGER NOT NULL REFERENCES demo_files(id) ON DELETE CASCADE,
               kind TEXT NOT NULL DEFAULT 'core',
               stage TEXT NOT NULL,
               progress INTEGER NOT NULL DEFAULT 0 CHECK(progress BETWEEN 0 AND 100),
               attempts INTEGER NOT NULL DEFAULT 0,
               error_code TEXT,
               error_detail TEXT,
               log_tail TEXT,
               cancel_requested INTEGER NOT NULL DEFAULT 0,
               parser_commit TEXT NOT NULL,
               adapter_version TEXT NOT NULL,
               schema_version INTEGER NOT NULL,
               metric_version TEXT NOT NULL,
               created_at INTEGER NOT NULL,
               started_at INTEGER,
               finished_at INTEGER,
               UNIQUE(demo_id, kind, parser_commit, adapter_version, schema_version, metric_version)
             );
             CREATE INDEX IF NOT EXISTS analysis_jobs_stage ON analysis_jobs(stage, created_at);
             CREATE TABLE IF NOT EXISTS matches(
               demo_id INTEGER PRIMARY KEY REFERENCES demo_files(id) ON DELETE CASCADE,
               map_name TEXT,
               server_name TEXT,
               tick_count INTEGER,
               tickrate REAL,
               duration_ms INTEGER,
               game_mode TEXT,
               team_a_name TEXT,
               team_b_name TEXT,
               team_a_score INTEGER,
               team_b_score INTEGER,
               winner_side TEXT,
               analyzed_at INTEGER,
               quality_json TEXT
             );
             CREATE TABLE IF NOT EXISTS players(
               id INTEGER PRIMARY KEY,
               steam_id TEXT,
               stable_key TEXT NOT NULL UNIQUE,
               current_name TEXT,
               is_bot INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS match_players(
               demo_id INTEGER NOT NULL REFERENCES matches(demo_id) ON DELETE CASCADE,
               player_id INTEGER NOT NULL REFERENCES players(id),
               team_name TEXT,
               team_number INTEGER,
               slot INTEGER,
               kills INTEGER,
               deaths INTEGER,
               assists INTEGER,
               damage_health INTEGER,
               damage_armor INTEGER,
               headshots INTEGER,
               mvp_count INTEGER,
               utility_damage INTEGER,
               enemies_flashed INTEGER,
               score INTEGER,
               first_kills INTEGER,
               first_deaths INTEGER,
               trade_kills INTEGER,
               trade_deaths INTEGER,
               kast_rounds INTEGER,
               rounds_played INTEGER,
               adr REAL,
               kast_percent REAL,
               rating REAL,
               rating_model TEXT,
               source_json TEXT NOT NULL DEFAULT '{}',
               PRIMARY KEY(demo_id, player_id)
             );
             CREATE TABLE IF NOT EXISTS match_rounds(
               demo_id INTEGER NOT NULL REFERENCES matches(demo_id) ON DELETE CASCADE,
               round_number INTEGER NOT NULL,
               start_tick INTEGER,
               freeze_end_tick INTEGER,
               end_tick INTEGER,
               official_end_tick INTEGER,
               winner_side TEXT,
               reason TEXT,
               quality_json TEXT,
               PRIMARY KEY(demo_id, round_number)
             );
             CREATE TABLE IF NOT EXISTS match_events(
               id INTEGER PRIMARY KEY,
               demo_id INTEGER NOT NULL REFERENCES matches(demo_id) ON DELETE CASCADE,
               round_number INTEGER,
               tick INTEGER NOT NULL,
               kind TEXT NOT NULL,
               actor_key TEXT,
               target_key TEXT,
               payload_json TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS match_events_lookup ON match_events(demo_id, round_number, tick);
             CREATE TABLE IF NOT EXISTS player_round_stats(
               demo_id INTEGER NOT NULL,
               round_number INTEGER NOT NULL,
               player_id INTEGER NOT NULL REFERENCES players(id),
               kills INTEGER,
               deaths INTEGER,
               assists INTEGER,
               damage_health INTEGER,
               survived INTEGER,
               traded INTEGER,
               kast INTEGER,
               equipment_value INTEGER,
               money_start INTEGER,
               money_spent INTEGER,
               source_json TEXT NOT NULL DEFAULT '{}',
               PRIMARY KEY(demo_id, round_number, player_id),
               FOREIGN KEY(demo_id, round_number) REFERENCES match_rounds(demo_id, round_number) ON DELETE CASCADE
             );
             CREATE TABLE IF NOT EXISTS round_economy(
               demo_id INTEGER NOT NULL,
               round_number INTEGER NOT NULL,
               team_number INTEGER NOT NULL,
               equipment_value INTEGER,
               money_start INTEGER,
               money_spent INTEGER,
               economy_type TEXT,
               metric_version TEXT,
               PRIMARY KEY(demo_id, round_number, team_number),
               FOREIGN KEY(demo_id, round_number) REFERENCES match_rounds(demo_id, round_number) ON DELETE CASCADE
             );
             CREATE TABLE IF NOT EXISTS position_frames(
               demo_id INTEGER NOT NULL REFERENCES matches(demo_id) ON DELETE CASCADE,
               round_number INTEGER NOT NULL,
               sampling_hz INTEGER NOT NULL,
               chunk_index INTEGER NOT NULL,
               encoding TEXT NOT NULL,
               payload BLOB NOT NULL,
               PRIMARY KEY(demo_id, round_number, sampling_hz, chunk_index)
             );
             PRAGMA user_version=5;",
        )
        .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        tx.commit().map_err(|e| err("DEMO_DB_MIGRATION", e))?;
    }
    if version < 6 {
        let backup = path.with_extension(format!("sqlite3.v5-{}.bak", now_ms()));
        db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| err("DEMO_DB_BACKUP", e))?;
        db.execute("VACUUM INTO ?1", [backup.to_string_lossy().as_ref()])
            .map_err(|e| err("DEMO_DB_BACKUP", e))?;
        let tx = db
            .unchecked_transaction()
            .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        let columns = tx
            .prepare("PRAGMA table_info(analysis_jobs)")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<BTreeSet<_>, _>>()
            })
            .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        for (name, definition) in [
            ("lease_owner", "TEXT"),
            ("lease_until", "INTEGER"),
            ("worker_id", "TEXT"),
        ] {
            if !columns.contains(name) {
                tx.execute(
                    &format!("ALTER TABLE analysis_jobs ADD COLUMN {name} {definition}"),
                    [],
                )
                .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
            }
        }
        let event_columns = tx
            .prepare("PRAGMA table_info(match_events)")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<BTreeSet<_>, _>>()
            })
            .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        for (name, definition) in [("source", "TEXT"), ("quality", "TEXT")] {
            if !event_columns.contains(name) {
                tx.execute(
                    &format!("ALTER TABLE match_events ADD COLUMN {name} {definition}"),
                    [],
                )
                .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
            }
        }
        for (table, additions) in [
            (
                "player_round_stats",
                vec![("side", "TEXT"), ("source", "TEXT")],
            ),
            ("round_economy", vec![("team", "TEXT"), ("quality", "TEXT")]),
        ] {
            let columns = tx
                .prepare(&format!("PRAGMA table_info({table})"))
                .and_then(|mut stmt| {
                    stmt.query_map([], |row| row.get::<_, String>(1))?
                        .collect::<Result<BTreeSet<_>, _>>()
                })
                .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
            for (name, definition) in additions {
                if !columns.contains(name) {
                    tx.execute(
                        &format!("ALTER TABLE {table} ADD COLUMN {name} {definition}"),
                        [],
                    )
                    .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
                }
            }
        }
        tx.execute_batch(
            "CREATE INDEX IF NOT EXISTS analysis_jobs_claim ON analysis_jobs(kind,stage,cancel_requested,lease_until,created_at);
             CREATE INDEX IF NOT EXISTS match_events_kind_tick ON match_events(demo_id,kind,tick);
             CREATE TABLE IF NOT EXISTS position_chunks(
               demo_id INTEGER NOT NULL REFERENCES matches(demo_id) ON DELETE CASCADE,
               round_number INTEGER NOT NULL,
               sampling_hz INTEGER NOT NULL,
               chunk_index INTEGER NOT NULL,
               encoding TEXT NOT NULL,
               payload BLOB NOT NULL,
               first_tick INTEGER NOT NULL,
               last_tick INTEGER NOT NULL,
               PRIMARY KEY(demo_id,round_number,sampling_hz,chunk_index)
             );
             CREATE TABLE IF NOT EXISTS map_metadata(
               map_name TEXT PRIMARY KEY,
               upstream_commit TEXT NOT NULL,
               asset_sha256 TEXT NOT NULL,
               pos_x REAL NOT NULL,
               pos_y REAL NOT NULL,
               scale REAL NOT NULL,
               threshold_z REAL,
               updated_at INTEGER NOT NULL
             );
             PRAGMA user_version=6;",
        )
        .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        tx.commit().map_err(|e| err("DEMO_DB_MIGRATION", e))?;
    }
    if version < 7 {
        let backup = path.with_extension(format!("sqlite3.v6-{}.bak", now_ms()));
        db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| err("DEMO_DB_BACKUP", e))?;
        db.execute("VACUUM INTO ?1", [backup.to_string_lossy().as_ref()])
            .map_err(|e| err("DEMO_DB_BACKUP", e))?;
        let tx = db
            .unchecked_transaction()
            .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        let columns = table_columns(&tx, "map_metadata")?;
        for (name, definition) in [
            ("radar_asset", "TEXT"),
            ("lower_radar_asset", "TEXT"),
            ("lower_asset_sha256", "TEXT"),
            ("radar_size", "INTEGER NOT NULL DEFAULT 1024"),
            ("source", "TEXT NOT NULL DEFAULT 'cs-demo-manager'"),
        ] {
            if !columns.contains(name) {
                tx.execute(
                    &format!("ALTER TABLE map_metadata ADD COLUMN {name} {definition}"),
                    [],
                )
                .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
            }
        }
        tx.execute_batch("PRAGMA user_version=7;")
            .map_err(|e| err("DEMO_DB_MIGRATION", e))?;
        tx.commit().map_err(|e| err("DEMO_DB_MIGRATION", e))?;
    }
    Ok(db)
}

pub fn initialize(app: &AppHandle) -> Result<(), AppError> {
    open_db(app)?;
    let db = open_db(app)?;
    seed_map_metadata(&db)?;
    if !cs2::check_cs2_process()? {
        db.execute("UPDATE analysis_jobs SET stage='queued',progress=0,lease_owner=NULL,lease_until=NULL,worker_id=NULL WHERE stage IN ('fingerprinting','parsing_core','normalizing','persisting','computing_metrics','parsing_spatial') AND COALESCE(lease_until,0)<?1 AND attempts<2", [now_ms()]).map_err(|e| err("DEMO_JOB_RECOVERY", e))?;
        db.execute("UPDATE analysis_jobs SET stage='error',error_code='DEMO_JOB_RETRY_EXHAUSTED',error_detail='应用重启后任务已达到 2 次自动重试上限。',finished_at=?1,lease_owner=NULL,lease_until=NULL,worker_id=NULL WHERE stage IN ('fingerprinting','parsing_core','normalizing','persisting','computing_metrics','parsing_spatial') AND attempts>=2", [now_ms()]).map_err(|e| err("DEMO_JOB_RECOVERY", e))?;
    }
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let stale = open_db(&handle)
            .and_then(|db| {
                let mut stmt = db.prepare("SELECT path FROM demo_files WHERE report_schema_version<?1 OR parser_adapter_version<>?2 OR metrics_version<>?3 ORDER BY mtime_ms DESC").map_err(|e| err("DEMO_DB_QUERY", e))?;
                let rows = stmt.query_map(params![REPORT_SCHEMA_VERSION, PARSER_ADAPTER_VERSION, METRICS_VERSION], |row| row.get::<_, String>(0)).map_err(|e| err("DEMO_DB_QUERY", e))?;
                Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
            })
            .unwrap_or_default();
        for path in stale {
            while parse_gate::assert_parse_allowed().is_err() {
                std::thread::sleep(Duration::from_secs(1));
            }
            if let Err(error) = import_file(&handle, &path, "schema-migration", true) {
                log::warn!(
                    "demo schema migration deferred/failed: {}",
                    error.into_string()
                );
            }
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
    let selected = dunce::canonicalize(path).map_err(|e| err("DEMO_ROOT_INVALID", e))?;
    let canonical = normalize_demo_root(&selected)?;
    if !canonical.is_dir() {
        return Err(err("DEMO_ROOT_INVALID", "所选路径不是目录。"));
    }
    let db = open_db(app)?;
    let display = canonical.display().to_string();
    let now = now_ms();
    let effective_depth = if canonical
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("replays"))
    {
        0
    } else if canonical
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("csgo"))
    {
        1
    } else {
        depth
    };
    db.execute("INSERT INTO demo_roots(path,canonical_path,enabled,scan_depth,created_at,origin) VALUES(?1,?2,1,?3,?4,'manual') ON CONFLICT(canonical_path) DO UPDATE SET enabled=1,scan_depth=excluded.scan_depth,path=excluded.path,origin=CASE WHEN demo_roots.origin='selected_cs2_root' THEN 'manual' ELSE demo_roots.origin END",params![display,display.to_lowercase(),effective_depth,now]).map_err(|e|err("DEMO_ROOT_SAVE",e))?;
    refresh_watcher(app)?;
    list_roots(app)?
        .into_iter()
        .find(|r| r.path.eq_ignore_ascii_case(&display))
        .ok_or_else(|| err("DEMO_ROOT_SAVE", "目录保存后无法回读。"))
}

pub fn ensure_default_root(app: &AppHandle, root_path: &str) -> Result<DemoRoot, AppError> {
    let root = cs2::normalize_root(root_path)?;
    let demo_root = dunce::canonicalize(root.join("game").join("csgo"))
        .map_err(|e| err("DEMO_ROOT_INVALID", e))?;
    let display = demo_root.display().to_string();
    let canonical = display.to_lowercase();
    let db = open_db(app)?;
    let now = now_ms();
    db.execute(
        "DELETE FROM demo_roots WHERE origin='selected_cs2_root' AND canonical_path<>?1",
        [&canonical],
    )
    .map_err(|e| err("DEMO_ROOT_SAVE", e))?;
    db.execute("INSERT INTO demo_roots(path,canonical_path,enabled,scan_depth,created_at,origin) VALUES(?1,?2,1,1,?3,'selected_cs2_root') ON CONFLICT(canonical_path) DO UPDATE SET path=excluded.path,enabled=1,scan_depth=1,origin=CASE WHEN demo_roots.origin='manual' THEN demo_roots.origin ELSE 'selected_cs2_root' END", params![display, canonical, now]).map_err(|e| err("DEMO_ROOT_SAVE", e))?;
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
    let pending = Arc::new(Mutex::new(HashMap::<PathBuf, i64>::new()));
    let coordinator_pending = pending.clone();
    let coordinator_handle = app.clone();
    std::thread::Builder::new()
        .name("demo-watch-coordinator".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_millis(750));
            if cs2::check_cs2_process().unwrap_or(true) {
                continue;
            }
            let paths = coordinator_pending
                .lock()
                .map(|entries| entries.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            for path in paths {
                if !path.is_file() {
                    let _ = coordinator_pending.lock().map(|mut e| e.remove(&path));
                    continue;
                }
                match wait_until_stable(&path) {
                    Ok(_) => match import_file(
                        &coordinator_handle,
                        &path.display().to_string(),
                        "watcher",
                        false,
                    ) {
                        Ok(_) => {
                            let _ = coordinator_pending.lock().map(|mut e| e.remove(&path));
                            let _ = coordinator_handle.emit("demo://filesystem-changed", ());
                        }
                        Err(error) => {
                            let detail = error.into_string();
                            if !detail.contains("DEMO_PARSE_BLOCKED_CS2_RUNNING") {
                                log::warn!("demo watcher import failed: {detail}");
                            }
                        }
                    },
                    Err(error) => log::debug!(
                        "demo watcher waiting for stable file: {}",
                        error.into_string()
                    ),
                }
            }
        })
        .map_err(|e| err("DEMO_WATCHER", e))?;
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            for path in event.paths.into_iter().filter(|p| {
                p.extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("dem"))
            }) {
                let generation = now_ms();
                if let Ok(mut entries) = pending.lock() {
                    entries.insert(path, generation);
                }
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

fn normalize_demo_root(selected: &Path) -> Result<PathBuf, AppError> {
    if selected
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("replays"))
        && selected.parent().is_some_and(|parent| {
            parent
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("csgo"))
        })
    {
        return Ok(selected.to_path_buf());
    }
    if let Ok(root) = cs2::normalize_root(&selected.display().to_string()) {
        return dunce::canonicalize(root.join("game").join("csgo"))
            .map_err(|e| err("DEMO_ROOT_INVALID", e));
    }
    Ok(selected.to_path_buf())
}

struct CollectedDemos {
    paths: Vec<PathBuf>,
    scanned_directories: u64,
    permission_errors: u64,
}

fn collect(root: &Path, max_depth: i64) -> CollectedDemos {
    let mut out = CollectedDemos {
        paths: Vec::new(),
        scanned_directories: 0,
        permission_errors: 0,
    };
    let mut queue = VecDeque::from([(root.to_path_buf(), 0i64)]);
    while let Some((dir, depth)) = queue.pop_front() {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => {
                out.scanned_directories += 1;
                entries
            }
            Err(_) => {
                out.permission_errors += 1;
                continue;
            }
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
                out.paths.push(path)
            } else if ft.is_dir() && depth < max_depth {
                queue.push_back((path, depth + 1))
            }
        }
    }
    out.paths.sort_by_key(|p| {
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
    let mut stable_samples = 1;
    for _ in 0..24 {
        std::thread::sleep(Duration::from_millis(750));
        let current = validate(path)?;
        if current.len() != previous.len() || mtime_ms(&current) != mtime_ms(&previous) {
            previous = current;
            stable_samples = 1;
            continue;
        }
        previous = current;
        stable_samples += 1;
        if stable_samples >= 3 && now_ms().saturating_sub(mtime_ms(&previous)) >= 3_000 {
            return Ok(previous);
        }
    }
    Err(err(
        "DEMO_WAITING_STABLE",
        "Demo 文件在稳定窗口内仍在写入。",
    ))
}

fn cache_is_current(
    status: &str,
    fingerprint: (i64, i64),
    expected_fingerprint: (i64, i64),
    schema: i64,
    adapter: &str,
    metrics: &str,
) -> bool {
    status == "done"
        && fingerprint == expected_fingerprint
        && schema >= REPORT_SCHEMA_VERSION as i64
        && adapter == PARSER_ADAPTER_VERSION
        && metrics == METRICS_VERSION
}

#[derive(Default)]
struct RoundPlayerMetric {
    kills: i64,
    deaths: i64,
    assists: i64,
    damage: i64,
    traded: bool,
    trade_kills: i64,
    first_kills: i64,
    first_deaths: i64,
}

fn payload_i64(event: &DemoEvent, key: &str) -> Option<i64> {
    event.payload.get(key).and_then(|value| {
        value
            .as_i64()
            .or_else(|| value.as_u64().and_then(|v| i64::try_from(v).ok()))
    })
}

fn payload_string(event: &DemoEvent, key: &str) -> Option<String> {
    event.payload.get(key).and_then(|value| {
        value
            .as_str()
            .map(str::to_owned)
            .or_else(|| value.as_u64().map(|value| value.to_string()))
            .or_else(|| value.as_i64().map(|value| value.to_string()))
    })
}

fn within_trade_window(delta_ticks: i32, tick_interval_seconds: f64) -> bool {
    delta_ticks >= 0 && f64::from(delta_ticks) * tick_interval_seconds <= 5.0
}

fn persist_normalized_report(db: &Connection, report: &DemoReport) -> Result<(), AppError> {
    let tx = db
        .unchecked_transaction()
        .map_err(|e| err("DEMO_DB_PERSIST", e))?;
    let demo_id = report.summary.demo_file_id;
    let quality_json =
        serde_json::to_string(&report.data_quality).map_err(|e| err("DEMO_REPORT_SERIALIZE", e))?;
    tx.execute(
        "INSERT INTO matches(demo_id,map_name,server_name,team_a_score,team_b_score,analyzed_at,quality_json)
         VALUES(?1,?2,?3,?4,?5,?6,?7)
         ON CONFLICT(demo_id) DO UPDATE SET map_name=excluded.map_name,server_name=excluded.server_name,team_a_score=excluded.team_a_score,team_b_score=excluded.team_b_score,analyzed_at=excluded.analyzed_at,quality_json=excluded.quality_json",
        params![demo_id, report.summary.map_name, report.summary.server_name, report.summary.team_a_score, report.summary.team_b_score, report.summary.parsed_at, quality_json],
    )
    .map_err(|e| err("DEMO_DB_PERSIST", e))?;
    for table in [
        "match_events",
        "player_round_stats",
        "round_economy",
        "match_rounds",
        "match_players",
    ] {
        tx.execute(&format!("DELETE FROM {table} WHERE demo_id=?1"), [demo_id])
            .map_err(|e| err("DEMO_DB_PERSIST", e))?;
    }
    let mut player_ids = HashMap::<String, i64>::new();
    let mut player_ids_by_name = HashMap::<String, i64>::new();
    let mut player_teams = HashMap::<i64, i32>::new();
    for player in &report.players {
        tx.execute(
            "INSERT INTO players(steam_id,stable_key,current_name,is_bot) VALUES(?1,?2,?3,?4)
             ON CONFLICT(stable_key) DO UPDATE SET steam_id=COALESCE(excluded.steam_id,players.steam_id),current_name=COALESCE(excluded.current_name,players.current_name),is_bot=excluded.is_bot",
            params![player.steam_id, player.key, player.name, player.is_bot as i64],
        )
        .map_err(|e| err("DEMO_DB_PERSIST", e))?;
        let player_id: i64 = tx
            .query_row(
                "SELECT id FROM players WHERE stable_key=?1",
                [&player.key],
                |row| row.get(0),
            )
            .map_err(|e| err("DEMO_DB_PERSIST", e))?;
        player_ids.insert(player.key.clone(), player_id);
        if let Some(name) = player.name.as_ref() {
            player_ids_by_name.insert(name.clone(), player_id);
        }
        if let Some(team) = player.team_number {
            player_teams.insert(player_id, team);
        }
        let source_json = serde_json::json!({
            "identity": player.identity_source,
            "stats": player.stats_source,
            "userId": player.user_id,
        });
        tx.execute(
            "INSERT INTO match_players(demo_id,player_id,team_name,team_number,kills,deaths,assists,damage_health,headshots,first_kills,first_deaths,trade_kills,kast_rounds,rounds_played,adr,kast_percent,rating,rating_model,source_json)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
            params![demo_id, player_id, player.team, player.team_number, player.kills, player.deaths, player.assists, player.damage, player.headshots, player.first_kills, player.first_deaths, player.trade_kills, player.kast_rounds, player.rounds_played, player.adr, player.kast_percent, player.rating.as_ref().map(|value| value.rating), player.rating.as_ref().map(|value| value.model_version.as_str()), source_json.to_string()],
        )
        .map_err(|e| err("DEMO_DB_PERSIST", e))?;
    }
    for round in &report.rounds {
        tx.execute(
            "INSERT INTO match_rounds(demo_id,round_number,start_tick,end_tick,winner_side,reason,quality_json) VALUES(?1,?2,?3,?4,?5,?6,'{}')",
            params![demo_id, round.number, round.start_tick, round.end_tick, round.winner, round.reason],
        )
        .map_err(|e| err("DEMO_DB_PERSIST", e))?;
        let events = if round.events.is_empty() {
            round
                .kills
                .iter()
                .chain(&round.bomb_events)
                .collect::<Vec<_>>()
        } else {
            round.events.iter().collect::<Vec<_>>()
        };
        for event in &events {
            let payload =
                serde_json::to_string(event).map_err(|e| err("DEMO_REPORT_SERIALIZE", e))?;
            tx.execute(
                "INSERT INTO match_events(demo_id,round_number,tick,kind,actor_key,target_key,payload_json,source,quality) VALUES(?1,?2,?3,?4,?5,?6,?7,'demoparser','observed')",
                params![demo_id, round.number, event.tick, event.kind, event.actor_key, event.target_key, payload],
            )
            .map_err(|e| err("DEMO_DB_PERSIST", e))?;
        }
        if round.end_tick.is_none() {
            continue;
        }
        let resolve = |key: Option<&String>, name: Option<&String>| {
            key.and_then(|value| player_ids.get(value).copied())
                .or_else(|| name.and_then(|value| player_ids_by_name.get(value).copied()))
        };
        let mut metrics = player_ids
            .values()
            .copied()
            .map(|player_id| (player_id, RoundPlayerMetric::default()))
            .collect::<HashMap<_, _>>();
        let mut kills = events
            .iter()
            .filter(|event| event.kind == "player_death")
            .copied()
            .collect::<Vec<_>>();
        kills.sort_by_key(|event| event.tick);
        let mut opening_recorded = false;
        for event in &events {
            let actor = resolve(event.actor_key.as_ref(), event.actor.as_ref());
            let target = resolve(event.target_key.as_ref(), event.target.as_ref());
            match event.kind.as_str() {
                "player_death" => {
                    if let Some(player_id) = actor {
                        metrics.entry(player_id).or_default().kills += 1;
                    }
                    if let Some(player_id) = target {
                        metrics.entry(player_id).or_default().deaths += 1;
                    }
                    let assister_key = payload_string(event, "assister_steamid")
                        .filter(|value| {
                            value
                                .parse::<u64>()
                                .is_ok_and(|value| value >= MIN_STEAM_ID64)
                        })
                        .map(|value| format!("steam:{value}"));
                    let assister_name = payload_string(event, "assister_name");
                    if let Some(player_id) = resolve(assister_key.as_ref(), assister_name.as_ref())
                    {
                        metrics.entry(player_id).or_default().assists += 1;
                    }
                    let opposing = actor.zip(target).is_some_and(|(a, b)| {
                        a != b && player_teams.get(&a) != player_teams.get(&b)
                    });
                    if opposing && !opening_recorded {
                        if let Some(player_id) = actor {
                            metrics.entry(player_id).or_default().first_kills += 1;
                        }
                        if let Some(player_id) = target {
                            metrics.entry(player_id).or_default().first_deaths += 1;
                        }
                        opening_recorded = true;
                    }
                }
                "player_hurt" => {
                    if let Some(player_id) = actor {
                        if let Some(damage) = payload_i64(event, "dmg_health") {
                            metrics.entry(player_id).or_default().damage += damage.max(0);
                        }
                    }
                }
                _ => {}
            }
        }
        for death in &kills {
            let Some(victim) = resolve(death.target_key.as_ref(), death.target.as_ref()) else {
                continue;
            };
            let Some(killer) = resolve(death.actor_key.as_ref(), death.actor.as_ref()) else {
                continue;
            };
            let victim_team = player_teams.get(&victim);
            if victim_team.is_none() || victim_team == player_teams.get(&killer) {
                continue;
            }
            if let Some(trade) = kills.iter().find(|candidate| {
                let trader = resolve(candidate.actor_key.as_ref(), candidate.actor.as_ref());
                let traded_target =
                    resolve(candidate.target_key.as_ref(), candidate.target.as_ref());
                candidate.tick >= death.tick
                    && within_trade_window(candidate.tick - death.tick, 1.0 / 64.0)
                    && traded_target == Some(killer)
                    && trader.and_then(|id| player_teams.get(&id)) == victim_team
            }) {
                metrics.entry(victim).or_default().traded = true;
                if let Some(trader) = resolve(trade.actor_key.as_ref(), trade.actor.as_ref()) {
                    metrics.entry(trader).or_default().trade_kills += 1;
                }
            }
        }
        for (player_id, metric) in &metrics {
            let survived = metric.deaths == 0;
            let kast = metric.kills > 0 || metric.assists > 0 || survived || metric.traded;
            let side = player_teams
                .get(player_id)
                .and_then(|team| team_label(Some(*team)));
            tx.execute(
                "INSERT INTO player_round_stats(demo_id,round_number,player_id,side,kills,deaths,assists,damage_health,survived,traded,kast,equipment_value,money_start,money_spent,source,source_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,NULL,NULL,NULL,'events','{}')",
                params![demo_id, round.number, player_id, side, metric.kills, metric.deaths, metric.assists, metric.damage, survived as i64, metric.traded as i64, kast as i64],
            )
            .map_err(|e| err("DEMO_DB_PERSIST", e))?;
        }
        for team_number in player_teams.values().copied().collect::<BTreeSet<_>>() {
            let purchase_events = events.iter().filter(|event| {
                if event.kind != "item_purchase" {
                    return false;
                }
                let buyer = resolve(event.actor_key.as_ref(), event.actor.as_ref())
                    .or_else(|| resolve(event.target_key.as_ref(), event.target.as_ref()));
                buyer.and_then(|id| player_teams.get(&id).copied()) == Some(team_number)
            });
            let costs = purchase_events
                .filter_map(|event| payload_i64(event, "cost"))
                .collect::<Vec<_>>();
            let money_spent = (!costs.is_empty()).then(|| costs.into_iter().sum::<i64>());
            tx.execute(
                "INSERT INTO round_economy(demo_id,round_number,team_number,team,equipment_value,money_start,money_spent,economy_type,metric_version,quality) VALUES(?1,?2,?3,?4,NULL,NULL,?5,NULL,?6,?7)",
                params![demo_id, round.number, team_number, team_label(Some(team_number)), money_spent, METRICS_VERSION, if money_spent.is_some() { "partial" } else { "unavailable" }],
            )
            .map_err(|e| err("DEMO_DB_PERSIST", e))?;
        }
        for (player_id, metric) in metrics {
            tx.execute(
                "UPDATE match_players SET first_kills=COALESCE(first_kills,0)+?3,first_deaths=COALESCE(first_deaths,0)+?4,trade_kills=COALESCE(trade_kills,0)+?5,kast_rounds=COALESCE(kast_rounds,0)+?6 WHERE demo_id=?1 AND player_id=?2",
                params![demo_id, player_id, metric.first_kills, metric.first_deaths, metric.trade_kills, (metric.kills > 0 || metric.assists > 0 || metric.deaths == 0 || metric.traded) as i64],
            )
            .map_err(|e| err("DEMO_DB_PERSIST", e))?;
        }
    }
    tx.commit().map_err(|e| err("DEMO_DB_PERSIST", e))
}

pub fn import_file(
    app: &AppHandle,
    path: &str,
    source: &str,
    force_reparse: bool,
) -> Result<DemoImportResult, AppError> {
    parse_gate::assert_parse_allowed()?;
    let canonical = dunce::canonicalize(path).map_err(|e| err("DEMO_FILE_READ", e))?;
    if canonical
        .extension()
        .map_or(true, |x| !x.eq_ignore_ascii_case("dem"))
    {
        return Err(err("DEMO_EXTENSION_UNSUPPORTED", "第一版仅支持 .dem。"));
    }
    let meta = wait_until_stable(&canonical)?;
    parse_gate::assert_parse_allowed_for_path(&canonical)?;
    let checksum = sha256_file(&canonical)?;
    let display = canonical.display().to_string();
    let db = open_db(app)?;
    let now = now_ms();
    let existing: Option<ExistingDemoFingerprint> = db
        .query_row(
            "SELECT id,status,size_bytes,mtime_ms,checksum,report_schema_version,parser_adapter_version,metrics_version FROM demo_files WHERE canonical_path=?1",
            [display.to_lowercase()],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?)),
        )
        .optional()
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    if let Some((id, status, size, mtime, old_checksum, schema, adapter, metrics)) = existing {
        if !force_reparse
            && cache_is_current(
                &status,
                (size, mtime),
                (meta.len() as i64, mtime_ms(&meta)),
                schema,
                &adapter,
                &metrics,
            )
        {
            return Ok(DemoImportResult {
                demo_file_id: id,
                status,
                cache_hit: true,
            });
        }
        if status != "done"
            || old_checksum.as_deref() != Some(checksum.as_str())
            || size != meta.len() as i64
            || mtime != mtime_ms(&meta)
        {
            db.execute("UPDATE analysis_jobs SET stage='queued',progress=0,attempts=0,error_code=NULL,error_detail=NULL,cancel_requested=0,lease_owner=NULL,lease_until=NULL,worker_id=NULL,started_at=NULL,finished_at=NULL WHERE demo_id=?1 AND kind='core' AND parser_commit=?2 AND adapter_version=?3 AND schema_version=?4 AND metric_version=?5", params![id,PARSER_COMMIT,PARSER_ADAPTER_VERSION,REPORT_SCHEMA_VERSION,METRICS_VERSION]).map_err(|e| err("DEMO_JOB_SAVE", e))?;
        }
    }
    db.execute("INSERT INTO demo_files(path,canonical_path,file_name,size_bytes,mtime_ms,checksum,status,source,discovered_at) VALUES(?1,?2,?3,?4,?5,?6,'queued',?7,?8) ON CONFLICT(canonical_path) DO UPDATE SET size_bytes=excluded.size_bytes,mtime_ms=excluded.mtime_ms,checksum=excluded.checksum,status='queued',error_code=NULL,error_detail=NULL",params![display,display.to_lowercase(),canonical.file_name().unwrap_or_default().to_string_lossy(),meta.len() as i64,mtime_ms(&meta),checksum,source,now]).map_err(|e|err("DEMO_DB_SAVE",e))?;
    let id: i64 = db
        .query_row(
            "SELECT id FROM demo_files WHERE canonical_path=?1",
            [display.to_lowercase()],
            |r| r.get(0),
        )
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    db.execute(
        "INSERT INTO demo_paths(demo_id,canonical_path,exists_now,last_seen_at) VALUES(?1,?2,1,?3) ON CONFLICT(canonical_path) DO UPDATE SET demo_id=excluded.demo_id,exists_now=1,last_seen_at=excluded.last_seen_at",
        params![id, display.to_lowercase(), now],
    )
    .map_err(|e| err("DEMO_PATH_SAVE", e))?;
    db.execute(
        "INSERT OR IGNORE INTO analysis_jobs(demo_id,kind,stage,parser_commit,adapter_version,schema_version,metric_version,created_at) VALUES(?1,'core','queued',?2,?3,?4,?5,?6)",
        params![id, PARSER_COMMIT, PARSER_ADAPTER_VERSION, REPORT_SCHEMA_VERSION, METRICS_VERSION, now],
    )
    .map_err(|e| err("DEMO_JOB_SAVE", e))?;
    if force_reparse {
        db.execute(
            "UPDATE analysis_jobs SET stage='queued',progress=0,attempts=0,error_code=NULL,error_detail=NULL,cancel_requested=0,lease_owner=NULL,lease_until=NULL,worker_id=NULL,started_at=NULL,finished_at=NULL WHERE demo_id=?1 AND kind='core' AND parser_commit=?2 AND adapter_version=?3 AND schema_version=?4 AND metric_version=?5",
            params![id, PARSER_COMMIT, PARSER_ADAPTER_VERSION, REPORT_SCHEMA_VERSION, METRICS_VERSION],
        )
        .map_err(|e| err("DEMO_JOB_SAVE", e))?;
    }
    Ok(DemoImportResult {
        demo_file_id: id,
        status: "queued".into(),
        cache_hit: false,
    })
}

fn job_cancel_requested(db: &Connection, job_id: i64) -> Result<bool, AppError> {
    db.query_row(
        "SELECT cancel_requested<>0 FROM analysis_jobs WHERE id=?1",
        [job_id],
        |row| row.get(0),
    )
    .map_err(|e| err("DEMO_JOB_QUERY", e))
}

fn finish_canceled_job(db: &Connection, job_id: i64, demo_id: i64) -> Result<(), AppError> {
    let now = now_ms();
    db.execute(
        "UPDATE analysis_jobs SET stage='canceled',progress=0,finished_at=?2,lease_owner=NULL,lease_until=NULL,worker_id=NULL WHERE id=?1",
        params![job_id, now],
    )
    .map_err(|e| err("DEMO_JOB_SAVE", e))?;
    db.execute(
        "UPDATE demo_files SET status=CASE WHEN report_json IS NULL THEN 'error' ELSE 'done' END,error_code=CASE WHEN report_json IS NULL THEN 'DEMO_JOB_CANCELED' ELSE error_code END,error_detail=CASE WHEN report_json IS NULL THEN '分析已取消，未写入部分结果。' ELSE error_detail END WHERE id=?1",
        [demo_id],
    )
    .map_err(|e| err("DEMO_DB_SAVE", e))?;
    Ok(())
}

fn execute_core_job(
    app: &AppHandle,
    job_id: i64,
    demo_id: i64,
    path: &str,
) -> Result<(), AppError> {
    parse_gate::assert_parse_allowed()?;
    let canonical = dunce::canonicalize(path).map_err(|e| err("DEMO_FILE_READ", e))?;
    parse_gate::assert_parse_allowed_for_path(&canonical)?;
    let meta = validate(&canonical)?;
    let snapshot = (meta.len() as i64, mtime_ms(&meta));
    let db = open_db(app)?;
    if job_cancel_requested(&db, job_id)? {
        return finish_canceled_job(&db, job_id, demo_id);
    }
    parse_gate::assert_parse_allowed()?;
    db.execute(
        "UPDATE analysis_jobs SET stage='parsing_core',progress=20,lease_until=?2 WHERE id=?1",
        params![job_id, now_ms() + JOB_LEASE_MS],
    )
    .map_err(|e| err("DEMO_JOB_SAVE", e))?;
    parse_gate::assert_parse_allowed_for_path(&canonical)?;
    let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        parse_report(demo_id, &canonical, &meta)
    }))
    .map_err(|_| err("DEMO_PARSER_PANIC", "解析器发生 panic，任务已隔离。"))?;
    let report = parsed?;
    parse_gate::assert_parse_allowed_for_path(&canonical)?;
    let latest = validate(&canonical)?;
    if (latest.len() as i64, mtime_ms(&latest)) != snapshot {
        return Err(err(
            "DEMO_FILE_CHANGED_DURING_PARSE",
            "Demo 在解析过程中继续写入，结果已丢弃并重新排队。",
        ));
    }
    if job_cancel_requested(&db, job_id)? {
        return finish_canceled_job(&db, job_id, demo_id);
    }
    db.execute(
        "UPDATE analysis_jobs SET stage='normalizing',progress=65,lease_until=?2 WHERE id=?1",
        params![job_id, now_ms() + JOB_LEASE_MS],
    )
    .map_err(|e| err("DEMO_JOB_SAVE", e))?;
    if job_cancel_requested(&db, job_id)? {
        return finish_canceled_job(&db, job_id, demo_id);
    }
    db.execute(
        "UPDATE analysis_jobs SET stage='persisting',progress=80,lease_until=?2 WHERE id=?1",
        params![job_id, now_ms() + JOB_LEASE_MS],
    )
    .map_err(|e| err("DEMO_JOB_SAVE", e))?;
    persist_normalized_report(&db, &report)?;
    parse_gate::assert_parse_allowed_for_path(&canonical)?;
    let latest = validate(&canonical)?;
    if (latest.len() as i64, mtime_ms(&latest)) != snapshot {
        return Err(err(
            "DEMO_FILE_CHANGED_DURING_PARSE",
            "Demo 在提交前发生变化，结果未标记完成。",
        ));
    }
    if job_cancel_requested(&db, job_id)? {
        return finish_canceled_job(&db, job_id, demo_id);
    }
    db.execute(
        "UPDATE analysis_jobs SET stage='computing_metrics',progress=92,lease_until=?2 WHERE id=?1",
        params![job_id, now_ms() + JOB_LEASE_MS],
    )
    .map_err(|e| err("DEMO_JOB_SAVE", e))?;
    let json = serde_json::to_string(&report).map_err(|e| err("DEMO_REPORT_SERIALIZE", e))?;
    let tx = db
        .unchecked_transaction()
        .map_err(|e| err("DEMO_DB_SAVE", e))?;
    if job_cancel_requested(&tx, job_id)? {
        drop(tx);
        return finish_canceled_job(&db, job_id, demo_id);
    }
    tx.execute("UPDATE demo_files SET status='done',map_name=?2,total_rounds=?3,kills=?4,parsed_at=?5,report_json=?6,report_schema_version=?7,parser_adapter_version=?8,metrics_version=?9,scoreboard_status=?10,error_code=NULL,error_detail=NULL WHERE id=?1",params![demo_id,report.summary.map_name,report.summary.total_rounds,report.summary.total_kills,report.summary.parsed_at,json,REPORT_SCHEMA_VERSION,PARSER_ADAPTER_VERSION,METRICS_VERSION,report.data_quality.scoreboard_status]).map_err(|e|err("DEMO_DB_SAVE",e))?;
    tx.execute(
        "UPDATE analysis_jobs SET stage='done',progress=100,finished_at=?2,error_code=NULL,error_detail=NULL,lease_owner=NULL,lease_until=NULL,worker_id=NULL WHERE id=?1",
        params![job_id, now_ms()],
    )
    .map_err(|e| err("DEMO_JOB_SAVE", e))?;
    tx.commit().map_err(|e| err("DEMO_DB_SAVE", e))
}

pub(crate) fn process_next_job(app: &AppHandle, worker_id: &str) -> Result<bool, AppError> {
    parse_gate::assert_parse_allowed()?;
    let db = open_db(app)?;
    db.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| err("DEMO_JOB_CLAIM", e))?;
    let now = now_ms();
    let claimed: Option<(i64, i64, String)> = db
        .query_row(
            "SELECT j.id,j.demo_id,d.path FROM analysis_jobs j JOIN demo_files d ON d.id=j.demo_id WHERE j.kind='core' AND j.stage='queued' AND j.cancel_requested=0 AND (j.lease_until IS NULL OR j.lease_until<?1) ORDER BY j.created_at,j.id LIMIT 1",
            [now],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|e| err("DEMO_JOB_CLAIM", e))?;
    if let Some((job_id, demo_id, path)) = claimed {
        db.execute(
            "UPDATE analysis_jobs SET stage='fingerprinting',progress=5,attempts=attempts+1,started_at=COALESCE(started_at,?2),lease_owner=?3,lease_until=?4,worker_id=?3,error_code=NULL,error_detail=NULL WHERE id=?1 AND stage='queued'",
            params![job_id, now, worker_id, now + JOB_LEASE_MS],
        )
        .map_err(|e| err("DEMO_JOB_CLAIM", e))?;
        db.execute_batch("COMMIT")
            .map_err(|e| err("DEMO_JOB_CLAIM", e))?;
        drop(db);
        if let Err(error) = execute_core_job(app, job_id, demo_id, &path) {
            let db = open_db(app)?;
            let detail = error.into_string();
            let code = detail
                .split(']')
                .next()
                .unwrap_or("[DEMO_PARSE_FAILED")
                .trim_start_matches('[');
            let attempts: i64 = db
                .query_row(
                    "SELECT attempts FROM analysis_jobs WHERE id=?1",
                    [job_id],
                    |r| r.get(0),
                )
                .unwrap_or(2);
            let blocked = cs2::check_cs2_process().unwrap_or(true)
                || detail.contains("DEMO_PARSE_BLOCKED_CS2_RUNNING")
                || detail.contains("DEMO_FILE_CHANGED_DURING_PARSE");
            let stage = if blocked || attempts < 2 {
                "queued"
            } else {
                "error"
            };
            db.execute(
                "UPDATE analysis_jobs SET stage=?2,progress=0,error_code=CASE WHEN ?2='queued' THEN NULL ELSE ?3 END,error_detail=CASE WHEN ?2='queued' THEN NULL ELSE ?4 END,finished_at=CASE WHEN ?2='error' THEN ?5 ELSE NULL END,lease_owner=NULL,lease_until=NULL,worker_id=NULL WHERE id=?1 AND stage<>'canceled'",
                params![job_id, stage, code, detail, now_ms()],
            )
            .map_err(|e| err("DEMO_JOB_SAVE", e))?;
            if stage == "error" && !blocked {
                db.execute("UPDATE demo_files SET status=CASE WHEN report_json IS NULL THEN 'error' ELSE 'done' END,error_code=?2,error_detail=?3 WHERE id=?1", params![demo_id, code, detail]).map_err(|e| err("DEMO_DB_SAVE", e))?;
            }
        }
        return Ok(true);
    }
    db.execute_batch("COMMIT")
        .map_err(|e| err("DEMO_JOB_CLAIM", e))?;
    Ok(false)
}

fn vec_i32(
    column: Option<&cs2_demoparser::second_pass::variants::PropColumn>,
    index: usize,
) -> Option<i32> {
    match column?.data.as_ref()? {
        VarVec::I32(values) => values.get(index).copied().flatten(),
        VarVec::U32(values) => values
            .get(index)
            .copied()
            .flatten()
            .and_then(|value| i32::try_from(value).ok()),
        _ => None,
    }
}

fn vec_f32(
    column: Option<&cs2_demoparser::second_pass::variants::PropColumn>,
    index: usize,
) -> Option<f32> {
    match column?.data.as_ref()? {
        VarVec::F32(values) => values.get(index).copied().flatten(),
        _ => None,
    }
}

fn vec_bool(
    column: Option<&cs2_demoparser::second_pass::variants::PropColumn>,
    index: usize,
) -> Option<bool> {
    match column?.data.as_ref()? {
        VarVec::Bool(values) => values.get(index).copied().flatten(),
        _ => None,
    }
}

fn vec_string(
    column: Option<&cs2_demoparser::second_pass::variants::PropColumn>,
    index: usize,
) -> Option<String> {
    match column?.data.as_ref()? {
        VarVec::String(values) => values.get(index).cloned().flatten(),
        _ => None,
    }
}

fn execute_spatial_job(
    app: &AppHandle,
    job_id: i64,
    demo_id: i64,
    path: &str,
    sampling_hz: i64,
) -> Result<(), AppError> {
    let report = report(app, demo_id)?;
    let max_tick = report
        .rounds
        .iter()
        .filter_map(|round| round.end_tick)
        .max()
        .ok_or_else(|| err("DEMO_SPATIAL_ROUNDS", "没有可采样的完整回合。"))?;
    let step = (64 / sampling_hz.clamp(4, 16)) as i32;
    let _ = max_tick;
    parse_gate::assert_parse_allowed_for_path(Path::new(path))?;
    let bytes = fs::read(path).map_err(|e| err("DEMO_FILE_READ", e))?;
    let h = create_huffman_lookup_table();
    let props = [
        "X",
        "Y",
        "Z",
        "yaw",
        "health",
        "armor",
        "CCSPlayerController.m_iTeamNum",
        "is_alive",
        "active_weapon_name",
    ];
    let settings = ParserInputs {
        real_name_to_og_name: Default::default(),
        wanted_players: vec![],
        wanted_player_props: props.iter().map(|value| value.to_string()).collect(),
        wanted_other_props: vec![],
        wanted_prop_states: Default::default(),
        // demoparser samples at packet ticks; synthetic exact ticks can miss every snapshot.
        // Collect packet ticks and downsample after parsing instead.
        wanted_ticks: vec![],
        wanted_events: vec![],
        parse_ents: true,
        // Projectile collection returns early before player entity snapshots;
        // spatial pass needs the player dataframe, so keep it separate.
        parse_projectiles: false,
        parse_grenades: false,
        only_header: false,
        only_convars: false,
        huffman_lookup_table: &h,
        order_by_steamid: true,
        list_props: false,
        fallback_bytes: None,
    };
    let output = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Parser::new(settings, ParsingMode::ForceSingleThreaded).parse_demo(&bytes)
    }))
    .map_err(|_| err("DEMO_PARSER_PANIC", "空间解析发生 panic，任务已隔离。"))?
    .map_err(|error| err("DEMO_SPATIAL_PARSE", format!("{error:?}")))?;
    let prop_id = |name: &str| {
        output
            .prop_controller
            .prop_infos
            .iter()
            .find(|prop| {
                prop.prop_friendly_name.eq_ignore_ascii_case(name)
                    || prop.prop_name.eq_ignore_ascii_case(name)
            })
            .map(|prop| prop.id)
    };
    let x_id = prop_id("X").ok_or_else(|| err("DEMO_SPATIAL_COLUMNS", "缺少 X 坐标列。"))?;
    let y_id = prop_id("Y").ok_or_else(|| err("DEMO_SPATIAL_COLUMNS", "缺少 Y 坐标列。"))?;
    let z_id = prop_id("Z").ok_or_else(|| err("DEMO_SPATIAL_COLUMNS", "缺少 Z 坐标列。"))?;
    let mut by_round = BTreeMap::<i64, Vec<PositionPoint>>::new();
    for (steam_id, player_df) in &output.df_per_player {
        let ticks = player_df
            .get(&TICK_ID)
            .ok_or_else(|| err("DEMO_SPATIAL_COLUMNS", "缺少 tick 列。"))?;
        let row_count = ticks.data.as_ref().map_or(0, |values| match values {
            VarVec::I32(values) => values.len(),
            VarVec::U32(values) => values.len(),
            _ => 0,
        });
        let stable_key = if *steam_id >= MIN_STEAM_ID64 {
            format!("steam:{steam_id}")
        } else {
            format!("bot:{steam_id}")
        };
        let name = report
            .players
            .iter()
            .find(|player| player.key == stable_key)
            .and_then(|player| player.name.clone());
        for index in 0..row_count {
            let Some(tick) = vec_i32(Some(ticks), index) else {
                continue;
            };
            let Some(x) = vec_f32(player_df.get(&x_id), index) else {
                continue;
            };
            let Some(y) = vec_f32(player_df.get(&y_id), index) else {
                continue;
            };
            let Some(z) = vec_f32(player_df.get(&z_id), index) else {
                continue;
            };
            if !x.is_finite() || !y.is_finite() || !z.is_finite() {
                continue;
            }
            let Some(round_number) = report
                .rounds
                .iter()
                .find(|round| {
                    tick >= round.start_tick.unwrap_or(i32::MIN)
                        && tick <= round.end_tick.unwrap_or(i32::MAX)
                })
                .map(|round| i64::from(round.number))
            else {
                continue;
            };
            if step > 1
                && tick % step != 0
                && !report
                    .rounds
                    .iter()
                    .any(|round| round.events.iter().any(|event| event.tick == tick))
            {
                continue;
            }
            by_round
                .entry(round_number)
                .or_default()
                .push(PositionPoint {
                    tick,
                    stable_key: stable_key.clone(),
                    name: name.clone(),
                    x,
                    y,
                    z,
                    yaw: prop_id("yaw").and_then(|id| vec_f32(player_df.get(&id), index)),
                    health: prop_id("health").and_then(|id| vec_i32(player_df.get(&id), index)),
                    armor: prop_id("armor").and_then(|id| vec_i32(player_df.get(&id), index)),
                    team_number: prop_id("CCSPlayerController.m_iTeamNum")
                        .or_else(|| prop_id("team_num"))
                        .and_then(|id| vec_i32(player_df.get(&id), index))
                        .or_else(|| {
                            report
                                .players
                                .iter()
                                .find(|player| player.key == stable_key)
                                .and_then(|player| player.team_number)
                        }),
                    participant_role: report
                        .players
                        .iter()
                        .find(|player| player.key == stable_key)
                        .map(|player| player.participant_role.clone())
                        .unwrap_or_else(|| "unknown".into()),
                    alive: prop_id("is_alive").and_then(|id| vec_bool(player_df.get(&id), index)),
                    weapon: prop_id("active_weapon_name")
                        .and_then(|id| vec_string(player_df.get(&id), index)),
                });
        }
    }
    if by_round.values().all(Vec::is_empty) {
        return Err(err("DEMO_SPATIAL_EMPTY", "解析器未返回有效坐标点。"));
    }
    let db = open_db(app)?;
    if job_cancel_requested(&db, job_id)? {
        return finish_canceled_job(&db, job_id, demo_id);
    }
    let tx = db
        .unchecked_transaction()
        .map_err(|e| err("DEMO_SPATIAL_SAVE", e))?;
    tx.execute(
        "DELETE FROM position_chunks WHERE demo_id=?1 AND sampling_hz=?2",
        params![demo_id, sampling_hz],
    )
    .map_err(|e| err("DEMO_SPATIAL_SAVE", e))?;
    for (round_number, points) in by_round {
        for (chunk_index, chunk) in points.chunks(4_000).enumerate() {
            let json = serde_json::to_vec(chunk).map_err(|e| err("DEMO_SPATIAL_ENCODE", e))?;
            let payload = zstd::stream::encode_all(json.as_slice(), 3)
                .map_err(|e| err("DEMO_SPATIAL_ENCODE", e))?;
            tx.execute("INSERT INTO position_chunks(demo_id,round_number,sampling_hz,chunk_index,encoding,payload,first_tick,last_tick) VALUES(?1,?2,?3,?4,'json-zstd-v1',?5,?6,?7)", params![demo_id, round_number, sampling_hz, chunk_index as i64, payload, chunk.first().map(|point| point.tick).unwrap_or(0), chunk.last().map(|point| point.tick).unwrap_or(0)]).map_err(|e| err("DEMO_SPATIAL_SAVE", e))?;
        }
    }
    tx.execute("UPDATE analysis_jobs SET stage='spatial_done',progress=100,finished_at=?2,lease_owner=NULL,lease_until=NULL,worker_id=NULL WHERE id=?1", params![job_id, now_ms()]).map_err(|e| err("DEMO_SPATIAL_SAVE", e))?;
    tx.commit().map_err(|e| err("DEMO_SPATIAL_SAVE", e))?;
    invalidate_position_cache(demo_id, sampling_hz);
    Ok(())
}

pub(crate) fn process_next_spatial_job(app: &AppHandle, worker_id: &str) -> Result<bool, AppError> {
    parse_gate::assert_parse_allowed()?;
    let db = open_db(app)?;
    db.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| err("DEMO_JOB_CLAIM", e))?;
    let now = now_ms();
    let claimed: Option<(i64, i64, String, i64)> = db.query_row("SELECT j.id,j.demo_id,d.path,COALESCE(CAST(json_extract(j.log_tail,'$.samplingHz') AS INTEGER),8) FROM analysis_jobs j JOIN demo_files d ON d.id=j.demo_id WHERE j.kind='spatial' AND j.stage='queued' AND j.cancel_requested=0 AND (j.lease_until IS NULL OR j.lease_until<?1) ORDER BY j.created_at,j.id LIMIT 1", [now], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).optional().map_err(|e| err("DEMO_JOB_CLAIM", e))?;
    if let Some((job_id, demo_id, path, sampling_hz)) = claimed {
        db.execute("UPDATE analysis_jobs SET stage='parsing_spatial',progress=10,attempts=attempts+1,started_at=COALESCE(started_at,?2),lease_owner=?3,lease_until=?4,worker_id=?3 WHERE id=?1", params![job_id, now, worker_id, now + 300_000]).map_err(|e| err("DEMO_JOB_CLAIM", e))?;
        db.execute_batch("COMMIT")
            .map_err(|e| err("DEMO_JOB_CLAIM", e))?;
        drop(db);
        if let Err(error) = execute_spatial_job(app, job_id, demo_id, &path, sampling_hz) {
            let db = open_db(app)?;
            let detail = error.into_string();
            let code = detail
                .split(']')
                .next()
                .unwrap_or("[DEMO_SPATIAL_FAILED")
                .trim_start_matches('[');
            let stage = if cs2::check_cs2_process().unwrap_or(true)
                || detail.contains("DEMO_PARSE_BLOCKED_CS2_RUNNING")
            {
                "queued"
            } else {
                "error"
            };
            db.execute("UPDATE analysis_jobs SET stage=?2,progress=0,error_code=CASE WHEN ?2='queued' THEN NULL ELSE ?3 END,error_detail=CASE WHEN ?2='queued' THEN NULL ELSE ?4 END,finished_at=CASE WHEN ?2='error' THEN ?5 ELSE NULL END,lease_owner=NULL,lease_until=NULL,worker_id=NULL WHERE id=?1 AND stage<>'canceled'", params![job_id, stage, code, detail, now_ms()]).map_err(|e| err("DEMO_JOB_SAVE", e))?;
        }
        return Ok(true);
    }
    db.execute_batch("COMMIT")
        .map_err(|e| err("DEMO_JOB_CLAIM", e))?;
    Ok(false)
}

pub fn ensure_spatial_analysis(
    app: &AppHandle,
    demo_id: i64,
    sampling_hz: i64,
) -> Result<i64, AppError> {
    parse_gate::assert_parse_allowed()?;
    if ![4, 8, 16].contains(&sampling_hz) {
        return Err(err("DEMO_SPATIAL_RATE", "sampling_hz 仅支持 4、8 或 16。"));
    }
    let db = open_db(app)?;
    let now = now_ms();
    db.execute("INSERT OR IGNORE INTO analysis_jobs(demo_id,kind,stage,progress,attempts,log_tail,cancel_requested,parser_commit,adapter_version,schema_version,metric_version,created_at) VALUES(?1,'spatial','queued',0,0,?2,0,?3,?4,?5,?6,?7)", params![demo_id, serde_json::json!({"samplingHz": sampling_hz}).to_string(), PARSER_COMMIT, PARSER_ADAPTER_VERSION, REPORT_SCHEMA_VERSION, METRICS_VERSION, now]).map_err(|e| err("DEMO_JOB_SAVE", e))?;
    let job_id = db.query_row("SELECT id FROM analysis_jobs WHERE demo_id=?1 AND kind='spatial' AND parser_commit=?2 AND adapter_version=?3 AND schema_version=?4 AND metric_version=?5", params![demo_id,PARSER_COMMIT,PARSER_ADAPTER_VERSION,REPORT_SCHEMA_VERSION,METRICS_VERSION], |row| row.get(0)).map_err(|e| err("DEMO_JOB_QUERY", e))?;
    let existing: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM position_chunks WHERE demo_id=?1 AND sampling_hz=?2",
            params![demo_id, sampling_hz],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if existing == 0 {
        db.execute("UPDATE analysis_jobs SET stage='queued',progress=0,attempts=0,log_tail=?2,cancel_requested=0,error_code=NULL,error_detail=NULL,started_at=NULL,finished_at=NULL WHERE id=?1", params![job_id, serde_json::json!({"samplingHz": sampling_hz}).to_string()]).map_err(|e| err("DEMO_JOB_SAVE", e))?;
    }
    Ok(job_id)
}

pub fn round_positions(
    app: &AppHandle,
    demo_id: i64,
    round_number: i64,
    sampling_hz: i64,
) -> Result<RoundPositions, AppError> {
    if round_number < 1 || ![4, 8, 16].contains(&sampling_hz) {
        return Err(err("DEMO_SPATIAL_RANGE", "回合或采样率无效。"));
    }
    let db = open_db(app)?;
    let current_spatial: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM analysis_jobs WHERE demo_id=?1 AND kind='spatial' AND stage='spatial_done' AND parser_commit=?2 AND adapter_version=?3 AND schema_version=?4 AND metric_version=?5",
            params![demo_id, PARSER_COMMIT, PARSER_ADAPTER_VERSION, REPORT_SCHEMA_VERSION, METRICS_VERSION],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if current_spatial == 0 {
        return Err(err(
            "DEMO_SPATIAL_STALE",
            "空间数据版本过旧，请重新生成地图回放。",
        ));
    }
    let cache_key = (demo_id, round_number, sampling_hz);
    if let Some(points) = cached_positions(cache_key) {
        return Ok(RoundPositions {
            demo_file_id: demo_id,
            round_number,
            sampling_hz,
            points,
        });
    }
    let mut stmt = db.prepare("SELECT payload,encoding FROM position_chunks WHERE demo_id=?1 AND round_number=?2 AND sampling_hz=?3 ORDER BY chunk_index").map_err(|e| err("DEMO_DB_QUERY", e))?;
    let rows = stmt
        .query_map(params![demo_id, round_number, sampling_hz], |row| {
            Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    let mut points = Vec::new();
    for row in rows {
        let (payload, encoding) = row.map_err(|e| err("DEMO_DB_QUERY", e))?;
        if encoding != "json-zstd-v1" {
            return Err(err("DEMO_SPATIAL_ENCODING", encoding));
        }
        let json = zstd::stream::decode_all(payload.as_slice())
            .map_err(|e| err("DEMO_SPATIAL_DECODE", e))?;
        points.extend(
            serde_json::from_slice::<Vec<PositionPoint>>(&json)
                .map_err(|e| err("DEMO_SPATIAL_DECODE", e))?,
        );
    }
    if points.is_empty() {
        return Err(err("DEMO_SPATIAL_NOT_READY", "该回合尚无空间数据。"));
    }
    cache_positions(cache_key, &points);
    Ok(RoundPositions {
        demo_file_id: demo_id,
        round_number,
        sampling_hz,
        points,
    })
}

pub fn heatmap_points(
    app: &AppHandle,
    demo_id: i64,
    filters: &HeatmapFilters,
) -> Result<Vec<HeatmapPoint>, AppError> {
    let db = open_db(app)?;
    heatmap_points_from_db(&db, demo_id, filters)
}

fn heatmap_points_from_db(
    db: &Connection,
    demo_id: i64,
    filters: &HeatmapFilters,
) -> Result<Vec<HeatmapPoint>, AppError> {
    let allowed = [
        "player_death",
        "weapon_fire",
        "flashbang_detonate",
        "hegrenade_detonate",
        "smokegrenade_detonate",
        "inferno_startburn",
    ];
    if !allowed.contains(&filters.kind.as_str()) {
        return Err(err("DEMO_HEATMAP_KIND", "不支持的热力图事件类型。"));
    }
    if !matches!(filters.layer.as_str(), "upper" | "lower" | "all") {
        return Err(err(
            "DEMO_HEATMAP_LAYER",
            "楼层必须为 upper、lower 或 all。",
        ));
    }
    if !(4..=64).contains(&filters.radius) || !(0.05..=1.0).contains(&filters.opacity) {
        return Err(err(
            "DEMO_HEATMAP_STYLE",
            "半径必须为 4..64，透明度必须为 0.05..1.0。",
        ));
    }
    if filters.round_numbers.iter().any(|value| *value < 0)
        || filters
            .team_numbers
            .iter()
            .any(|value| !matches!(value, 2 | 3))
    {
        return Err(err("DEMO_HEATMAP_FILTER", "回合或队伍筛选值无效。"));
    }
    let map_name: String = db
        .query_row(
            "SELECT map_name FROM matches WHERE demo_id=?1",
            [demo_id],
            |row| row.get(0),
        )
        .map_err(|e| err("DEMO_MAP_UNAVAILABLE", e))?;
    let metadata = map_metadata::embedded()
        .map_err(|e| err("DEMO_MAP_METADATA", e))?
        .into_iter()
        .find(|map| map.name == map_name)
        .ok_or_else(|| {
            err(
                "DEMO_MAP_UNAVAILABLE",
                format!("地图 {map_name} 没有固定元数据。"),
            )
        })?;
    if filters.layer == "lower" && metadata.lower_radar_asset.is_none() {
        return Err(err(
            "DEMO_RADAR_LOWER_UNAVAILABLE",
            format!("地图 {map_name} 没有下层雷达。"),
        ));
    }
    let mut stmt = db.prepare("SELECT e.tick,e.round_number,COALESCE(e.actor_key,e.target_key),CAST(json_extract(e.payload_json,'$.payload.x') AS REAL),CAST(json_extract(e.payload_json,'$.payload.y') AS REAL),CAST(json_extract(e.payload_json,'$.payload.z') AS REAL),(SELECT mp.team_number FROM match_players mp JOIN players p ON p.id=mp.player_id WHERE mp.demo_id=e.demo_id AND p.stable_key=COALESCE(e.actor_key,e.target_key) LIMIT 1) FROM match_events e WHERE e.demo_id=?1 AND e.kind=?2 AND json_extract(e.payload_json,'$.payload.x') IS NOT NULL AND json_extract(e.payload_json,'$.payload.y') IS NOT NULL ORDER BY e.tick").map_err(|e| err("DEMO_DB_QUERY", e))?;
    let rows = stmt
        .query_map(params![demo_id, filters.kind], |row| {
            Ok(HeatmapPoint {
                tick: row.get(0)?,
                round_number: row.get(1)?,
                player_key: row.get(2)?,
                x: row.get(3)?,
                y: row.get(4)?,
                z: row.get(5)?,
                team_number: row.get(6)?,
                weight: 1.0,
                kind: filters.kind.clone(),
            })
        })
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    Ok(rows
        .filter_map(Result::ok)
        .filter(|point| heatmap_point_matches(&metadata, filters, point))
        .collect())
}

fn heatmap_point_matches(
    metadata: &crate::demo::map_metadata::MapMetadata,
    filters: &HeatmapFilters,
    point: &HeatmapPoint,
) -> bool {
    (filters.round_numbers.is_empty()
        || point
            .round_number
            .is_some_and(|value| filters.round_numbers.contains(&value)))
        && (filters.player_keys.is_empty()
            || point
                .player_key
                .as_ref()
                .is_some_and(|value| filters.player_keys.contains(value)))
        && (filters.team_numbers.is_empty()
            || point
                .team_number
                .is_some_and(|value| filters.team_numbers.contains(&value)))
        && (filters.layer == "all" || point.z.is_some_and(|z| metadata.layer(z) == filters.layer))
}

pub fn save_heatmap_png(
    app: &AppHandle,
    demo_id: i64,
    filters: &HeatmapFilters,
    destination_path: &str,
) -> Result<HeatmapExportResult, AppError> {
    let points = heatmap_points(app, demo_id, filters)?;
    if points.is_empty() {
        return Err(err(
            "DEMO_HEATMAP_EMPTY",
            "该筛选条件没有可保存的热力图点。",
        ));
    }
    let destination = PathBuf::from(destination_path);
    let parent = destination
        .parent()
        .filter(|path| path.is_dir())
        .ok_or_else(|| err("DEMO_EXPORT_PATH", "导出目录不存在。"))?;
    let db = open_db(app)?;
    let map_name: String = db
        .query_row(
            "SELECT map_name FROM matches WHERE demo_id=?1",
            [demo_id],
            |row| row.get(0),
        )
        .map_err(|e| err("DEMO_MAP_UNAVAILABLE", e))?;
    let metadata = map_metadata::embedded()
        .map_err(|e| err("DEMO_MAP_METADATA", e))?
        .into_iter()
        .find(|map| map.name == map_name)
        .ok_or_else(|| {
            err(
                "DEMO_MAP_UNAVAILABLE",
                format!("地图 {map_name} 没有固定元数据。"),
            )
        })?;
    let (asset, expected_hash) = if filters.layer == "lower" {
        (
            metadata
                .lower_radar_asset
                .as_ref()
                .ok_or_else(|| err("DEMO_RADAR_LOWER_UNAVAILABLE", &map_name))?,
            metadata
                .lower_radar_sha256
                .as_ref()
                .ok_or_else(|| err("DEMO_RADAR_HASH", &map_name))?,
        )
    } else {
        (&metadata.radar_asset, &metadata.radar_sha256)
    };
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| err("DEMO_RESOURCE_DIR", e))?;
    let candidates = [
        resource_dir.join("resources/demo-maps/radars").join(asset),
        resource_dir.join("demo-maps/radars").join(asset),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/demo-maps/radars")
            .join(asset),
    ];
    let radar_path = candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| err("DEMO_RADAR_UNAVAILABLE", asset))?;
    let radar = fs::read(&radar_path).map_err(|e| err("DEMO_RADAR_READ", e))?;
    let asset_hash = crate::demo::heatmap::sha256(&radar);
    if &asset_hash != expected_hash {
        return Err(err(
            "DEMO_RADAR_HASH",
            format!("{} hash mismatch", radar_path.display()),
        ));
    }
    let (pixels, rendered, discarded) =
        crate::demo::heatmap::render(&metadata, &radar, &points, filters.radius, filters.opacity)?;
    let part = parent.join(format!(
        ".{}.{}.part",
        destination
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        now_ms()
    ));
    crate::demo::heatmap::encode_rgba(&part, &pixels)?;
    crate::demo::heatmap::replace_file(&part, &destination).inspect_err(|_| {
        let _ = fs::remove_file(&part);
    })?;
    let output = fs::read(&destination).map_err(|e| err("DEMO_EXPORT_READ", e))?;
    Ok(HeatmapExportResult {
        path: destination.display().to_string(),
        bytes_written: output.len() as u64,
        width: 1024,
        height: 1024,
        map_name,
        layer: filters.layer.clone(),
        input_points: points.len(),
        rendered_points: rendered,
        discarded_points: discarded,
        asset_sha256: asset_hash,
        output_sha256: crate::demo::heatmap::sha256(&output),
    })
}

pub fn launch_demo_at_tick(
    app: &AppHandle,
    demo_id: i64,
    tick: i64,
    player_key: Option<&str>,
) -> Result<LaunchDemoResult, AppError> {
    validate_demo_launch(tick, player_key)?;
    let path = path_for_id(app, demo_id)?;
    let canonical = dunce::canonicalize(&path).map_err(|_| {
        err(
            "DEMO_FILE_NOT_FOUND",
            "Demo 文件已移动或删除，请重新扫描录像库。",
        )
    })?;
    if !canonical.is_file()
        || !canonical
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("dem"))
    {
        return Err(err(
            "DEMO_FILE_NOT_FOUND",
            "Demo 文件不存在或不是 .dem 文件。",
        ));
    }
    if cs2::check_cs2_process()? {
        return Err(err(
            "DEMO_PLAYBACK_CS2_RUNNING",
            "CS2 已在运行。请先退出 CS2 后重试。",
        ));
    }
    let steam = crate::services::cs2_discovery::find_steam_executable(None)
        .ok_or_else(|| err("STEAM_NOT_FOUND", "未找到 Steam 客户端。"))?;
    let session_id = format!("playback-tick-{demo_id}-{}", now_ms());
    crate::demo::post_match::mark_next_playback(app, &session_id);
    if let Err(error) = std::process::Command::new(&steam)
        .args(demo_launch_args(&canonical, tick))
        .spawn()
    {
        crate::demo::post_match::cancel_pending(app, &session_id);
        return Err(err("DEMO_LAUNCH", error));
    }
    Ok(LaunchDemoResult {
        demo_file_id: demo_id,
        tick,
        path,
    })
}

fn validate_demo_launch(tick: i64, player_key: Option<&str>) -> Result<(), AppError> {
    if !(0..=10_000_000).contains(&tick) {
        return Err(err("DEMO_TICK_RANGE", "tick 超出允许范围。"));
    }
    if player_key.is_some_and(|value| !value.trim().is_empty()) {
        return Err(err(
            "DEMO_PLAYER_UNSUPPORTED",
            "当前 CS2 启动协议只能定位 tick，暂不支持按 stable player key 自动切换观察者。",
        ));
    }
    Ok(())
}

fn demo_launch_args(path: &Path, tick: i64) -> Vec<OsString> {
    vec![
        OsString::from("-applaunch"),
        OsString::from("730"),
        OsString::from("+playdemo"),
        path.as_os_str().to_owned(),
        OsString::from("+demo_gototick"),
        OsString::from(tick.to_string()),
    ]
}

pub fn cancel_analysis_job(app: &AppHandle, job_id: i64) -> Result<(), AppError> {
    let db = open_db(app)?;
    let changed = db
        .execute(
            "UPDATE analysis_jobs SET cancel_requested=1,stage=CASE WHEN stage='queued' THEN 'canceled' ELSE stage END,finished_at=CASE WHEN stage='queued' THEN ?2 ELSE finished_at END WHERE id=?1 AND stage NOT IN ('done','spatial_done','error','canceled')",
            params![job_id, now_ms()],
        )
        .map_err(|e| err("DEMO_JOB_CANCEL", e))?;
    if changed == 0 {
        return Err(err("DEMO_JOB_NOT_CANCELABLE", "任务不存在或已结束。"));
    }
    Ok(())
}

pub fn retry_analysis_job(app: &AppHandle, job_id: i64) -> Result<(), AppError> {
    parse_gate::assert_parse_allowed()?;
    let changed = open_db(app)?
        .execute(
            "UPDATE analysis_jobs SET stage='queued',progress=0,attempts=0,error_code=NULL,error_detail=NULL,cancel_requested=0,lease_owner=NULL,lease_until=NULL,worker_id=NULL,started_at=NULL,finished_at=NULL WHERE id=?1 AND stage IN ('error','canceled')",
            [job_id],
        )
        .map_err(|e| err("DEMO_JOB_RETRY", e))?;
    if changed == 0 {
        return Err(err(
            "DEMO_JOB_NOT_RETRYABLE",
            "任务不存在或当前状态不可重试。",
        ));
    }
    Ok(())
}

pub fn scan(app: &AppHandle, root_id: Option<i64>) -> Result<DemoScanResult, AppError> {
    parse_gate::assert_parse_allowed()?;
    let last_scan_at = now_ms();
    let mut out = DemoScanResult {
        root_paths: Vec::new(),
        scanned_directories: 0,
        discovered_dem_files: 0,
        imported: 0,
        cache_hits: 0,
        parsed: 0,
        failed: 0,
        permission_errors: 0,
        last_scan_at,
    };
    for root in list_roots(app)?
        .into_iter()
        .filter(|r| r.enabled && root_id.map_or(true, |id| r.id == id))
    {
        out.root_paths.push(root.path.clone());
        let collected = collect(Path::new(&root.path), root.scan_depth);
        out.scanned_directories += collected.scanned_directories;
        out.permission_errors += collected.permission_errors;
        out.discovered_dem_files += collected.paths.len() as u64;
        for path in collected.paths {
            match import_file(app, &path.display().to_string(), "scan", false) {
                Ok(v) if v.cache_hit => out.cache_hits += 1,
                Ok(v) if v.status == "queued" => {
                    out.imported += 1;
                }
                _ => out.failed += 1,
            }
        }
        let last_error = (collected.permission_errors > 0).then_some(format!(
            "[DEMO_ROOT_PERMISSION] {} 个目录无法读取。",
            collected.permission_errors
        ));
        let _ = open_db(app)?.execute(
            "UPDATE demo_roots SET last_scan_at=?2,last_error=?3 WHERE id=?1",
            params![root.id, last_scan_at, last_error],
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

pub(crate) fn list_all_for_session(app: &AppHandle) -> Result<Vec<DemoListItem>, AppError> {
    let db = open_db(app)?;
    let mut stmt = db
        .prepare("SELECT id,file_name,path,size_bytes,mtime_ms,status,error_code,map_name,total_rounds,kills,parsed_at FROM demo_files ORDER BY mtime_ms DESC,id DESC")
        .map_err(|error| err("DEMO_DB_QUERY", error))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(DemoListItem {
                id: row.get(0)?,
                file_name: row.get(1)?,
                path: row.get(2)?,
                size_bytes: row.get(3)?,
                mtime_ms: row.get(4)?,
                status: row.get(5)?,
                error_code: row.get(6)?,
                map_name: row.get(7)?,
                total_rounds: row.get(8)?,
                kills: row.get(9)?,
                parsed_at: row.get(10)?,
            })
        })
        .map_err(|error| err("DEMO_DB_QUERY", error))?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub(crate) fn demo_item(app: &AppHandle, id: i64) -> Result<Option<DemoListItem>, AppError> {
    Ok(list_all_for_session(app)?
        .into_iter()
        .find(|item| item.id == id))
}

pub fn list_analysis_jobs(
    app: &AppHandle,
    active_only: bool,
) -> Result<Vec<DemoAnalysisJob>, AppError> {
    let db = open_db(app)?;
    let mut stmt = db
        .prepare(
            "SELECT j.id,j.demo_id,d.file_name,j.kind,j.stage,j.progress,j.attempts,j.error_code,j.error_detail,j.created_at,j.started_at,j.finished_at
             FROM analysis_jobs j JOIN demo_files d ON d.id=j.demo_id
             WHERE (?1=0 OR j.stage NOT IN ('done','spatial_done','error','canceled'))
             ORDER BY CASE WHEN j.stage IN ('done','spatial_done','error','canceled') THEN 1 ELSE 0 END,j.created_at DESC
             LIMIT 100",
        )
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    let rows = stmt
        .query_map([active_only as i64], |row| {
            Ok(DemoAnalysisJob {
                id: row.get(0)?,
                demo_file_id: row.get(1)?,
                file_name: row.get(2)?,
                kind: row.get(3)?,
                stage: row.get(4)?,
                progress: row.get(5)?,
                attempts: row.get(6)?,
                error_code: row.get(7)?,
                error_detail: row.get(8)?,
                created_at: row.get(9)?,
                started_at: row.get(10)?,
                finished_at: row.get(11)?,
            })
        })
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn match_overview(app: &AppHandle, demo_id: i64) -> Result<MatchOverview, AppError> {
    let db = open_db(app)?;
    db.query_row(
        "SELECT m.demo_id,d.file_name,m.map_name,m.server_name,m.team_a_score,m.team_b_score,
                (SELECT COUNT(*) FROM match_rounds r WHERE r.demo_id=m.demo_id),
                (SELECT COUNT(*) FROM match_events e WHERE e.demo_id=m.demo_id AND e.kind='player_death'),
                m.analyzed_at,m.quality_json
         FROM matches m JOIN demo_files d ON d.id=m.demo_id WHERE m.demo_id=?1",
        [demo_id],
        |row| {
            Ok(MatchOverview {
                demo_file_id: row.get(0)?,
                file_name: row.get(1)?,
                map_name: row.get(2)?,
                server_name: row.get(3)?,
                team_a_score: row.get(4)?,
                team_b_score: row.get(5)?,
                total_rounds: row.get(6)?,
                total_kills: row.get(7)?,
                analyzed_at: row.get(8)?,
                quality_json: row.get(9)?,
            })
        },
    )
    .map_err(|e| err("DEMO_MATCH_NOT_FOUND", e))
}

pub fn match_scoreboard(
    app: &AppHandle,
    demo_id: i64,
) -> Result<Vec<MatchScoreboardPlayer>, AppError> {
    let db = open_db(app)?;
    let mut stmt = db
        .prepare(
            "SELECT p.stable_key,p.current_name,p.steam_id,p.is_bot,mp.team_name,mp.team_number,mp.kills,mp.deaths,mp.assists,mp.damage_health,mp.headshots,mp.adr,mp.kast_percent,mp.rating,mp.rating_model
             FROM match_players mp JOIN players p ON p.id=mp.player_id WHERE mp.demo_id=?1
             ORDER BY COALESCE(mp.team_number,99),COALESCE(mp.kills,-1) DESC,p.current_name",
        )
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    let rows = stmt
        .query_map([demo_id], |row| {
            Ok(MatchScoreboardPlayer {
                stable_key: row.get(0)?,
                name: row.get(1)?,
                steam_id: row.get(2)?,
                is_bot: row.get::<_, i64>(3)? != 0,
                team_name: row.get(4)?,
                team_number: row.get(5)?,
                kills: row.get(6)?,
                deaths: row.get(7)?,
                assists: row.get(8)?,
                damage_health: row.get(9)?,
                headshots: row.get(10)?,
                adr: row.get(11)?,
                kast_percent: row.get(12)?,
                rating: row.get(13)?,
                rating_model: row.get(14)?,
            })
        })
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn match_rounds(app: &AppHandle, demo_id: i64) -> Result<Vec<MatchRoundSummary>, AppError> {
    let db = open_db(app)?;
    let mut stmt = db
        .prepare(
            "SELECT r.round_number,r.start_tick,r.freeze_end_tick,r.end_tick,r.official_end_tick,r.winner_side,r.reason,
                    (SELECT COUNT(*) FROM match_events e WHERE e.demo_id=r.demo_id AND e.round_number=r.round_number)
             FROM match_rounds r WHERE r.demo_id=?1 ORDER BY r.round_number",
        )
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    let rows = stmt
        .query_map([demo_id], |row| {
            Ok(MatchRoundSummary {
                round_number: row.get(0)?,
                start_tick: row.get(1)?,
                freeze_end_tick: row.get(2)?,
                end_tick: row.get(3)?,
                official_end_tick: row.get(4)?,
                winner_side: row.get(5)?,
                reason: row.get(6)?,
                event_count: row.get(7)?,
            })
        })
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn match_economy(app: &AppHandle, demo_id: i64) -> Result<Vec<MatchEconomyRow>, AppError> {
    let db = open_db(app)?;
    let mut stmt = db
        .prepare("SELECT round_number,team_number,team,equipment_value,money_start,money_spent,economy_type,quality FROM round_economy WHERE demo_id=?1 ORDER BY round_number,team_number")
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    let rows = stmt
        .query_map([demo_id], |row| {
            Ok(MatchEconomyRow {
                round_number: row.get(0)?,
                team_number: row.get(1)?,
                team: row.get(2)?,
                equipment_value: row.get(3)?,
                money_start: row.get(4)?,
                money_spent: row.get(5)?,
                economy_type: row.get(6)?,
                quality: row.get(7)?,
            })
        })
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn match_duels(app: &AppHandle, demo_id: i64) -> Result<MatchDuelMatrix, AppError> {
    let db = open_db(app)?;
    let mut stmt = db
        .prepare(
            "SELECT e.actor_key,pa.current_name,pa.is_bot,ma.team_number,e.target_key,pb.current_name,pb.is_bot,mb.team_number,COUNT(*)
             FROM match_events e
             JOIN players pa ON pa.stable_key=e.actor_key JOIN match_players ma ON ma.player_id=pa.id AND ma.demo_id=e.demo_id
             JOIN players pb ON pb.stable_key=e.target_key JOIN match_players mb ON mb.player_id=pb.id AND mb.demo_id=e.demo_id
             WHERE e.demo_id=?1 AND e.kind='player_death' AND e.actor_key IS NOT NULL AND e.target_key IS NOT NULL
               AND ma.team_number IN (2,3) AND mb.team_number IN (2,3) AND ma.team_number != mb.team_number
             GROUP BY e.actor_key,e.target_key ORDER BY COUNT(*) DESC,e.actor_key,e.target_key",
        )
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    let rows = stmt
        .query_map([demo_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)? != 0,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)? != 0,
                row.get::<_, i64>(7)?,
                row.get::<_, i64>(8)?,
            ))
        })
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    let mut pairs = std::collections::BTreeMap::<(String, String), i64>::new();
    let mut players = std::collections::BTreeMap::<String, (Option<String>, bool, i64)>::new();
    for row in rows.filter_map(Result::ok) {
        let (
            actor,
            actor_name,
            actor_bot,
            actor_team,
            target,
            target_name,
            target_bot,
            target_team,
            count,
        ) = row;
        players
            .entry(actor.clone())
            .or_insert((actor_name, actor_bot, actor_team));
        players
            .entry(target.clone())
            .or_insert((target_name, target_bot, target_team));
        pairs.insert((actor, target), count);
    }
    let mut teams = [2_i64, 3_i64];
    teams.sort_unstable();
    let make_team = |number: i64| MatchDuelTeam {
        number: Some(number),
        label: team_label(Some(number as i32)).unwrap_or_else(|| format!("队伍 {number}")),
        players: players
            .iter()
            .filter(|(_, (_, _, team))| *team == number)
            .map(|(key, (name, is_bot, _))| MatchDuelPlayer {
                key: key.clone(),
                name: name.clone(),
                is_bot: *is_bot,
            })
            .collect(),
    };
    let mut team_x = make_team(teams[0]);
    let mut team_y = make_team(teams[1]);
    let score = |key: &str, other: &str| {
        pairs
            .get(&(key.to_string(), other.to_string()))
            .copied()
            .unwrap_or(0)
    };
    team_x.players.sort_by_key(|player| {
        std::cmp::Reverse(
            team_y
                .players
                .iter()
                .map(|other| score(&player.key, &other.key))
                .sum::<i64>(),
        )
    });
    team_y.players.sort_by_key(|player| {
        std::cmp::Reverse(
            team_x
                .players
                .iter()
                .map(|other| score(&other.key, &player.key))
                .sum::<i64>(),
        )
    });
    let cells = team_x
        .players
        .iter()
        .flat_map(|x| {
            team_y.players.iter().map(move |y| MatchDuelCell {
                x_player_key: x.key.clone(),
                y_player_key: y.key.clone(),
                x_kills_y: score(&x.key, &y.key),
                y_kills_x: score(&y.key, &x.key),
            })
        })
        .collect();
    Ok(MatchDuelMatrix {
        team_x,
        team_y,
        cells,
    })
}

pub fn match_utility(app: &AppHandle, demo_id: i64) -> Result<Vec<MatchUtilityRow>, AppError> {
    let db = open_db(app)?;
    let mut stmt = db
        .prepare(
            "SELECT p.stable_key,p.current_name,
                    SUM(CASE WHEN e.kind='player_hurt' AND json_extract(e.payload_json,'$.weapon')='hegrenade' THEN CAST(json_extract(e.payload_json,'$.payload.dmg_health') AS INTEGER) END),
                    SUM(CASE WHEN e.kind='player_blind' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN e.kind IN ('flashbang_detonate','hegrenade_detonate','smokegrenade_detonate','inferno_startburn') THEN 1 ELSE 0 END)
             FROM match_players mp JOIN players p ON p.id=mp.player_id
             LEFT JOIN match_events e ON e.demo_id=mp.demo_id AND e.actor_key=p.stable_key
             WHERE mp.demo_id=?1 GROUP BY p.id ORDER BY p.current_name",
        )
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    let rows = stmt
        .query_map([demo_id], |row| {
            Ok(MatchUtilityRow {
                stable_key: row.get(0)?,
                player_name: row.get(1)?,
                he_damage: row.get(2)?,
                flash_victims: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                throws: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
            })
        })
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn match_events(
    app: &AppHandle,
    demo_id: i64,
    filters: MatchEventFilters,
    page: i64,
    page_size: i64,
) -> Result<MatchEventPage, AppError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 200);
    let db = open_db(app)?;
    let kind = filters.kind.filter(|value| !value.trim().is_empty());
    let player_key = filters.player_key.filter(|value| !value.trim().is_empty());
    let total = db
        .query_row(
            "SELECT COUNT(*) FROM match_events WHERE demo_id=?1 AND (?2 IS NULL OR kind=?2) AND (?3 IS NULL OR round_number=?3) AND (?4 IS NULL OR actor_key=?4 OR target_key=?4)",
            params![demo_id, kind, filters.round_number, player_key],
            |row| row.get(0),
        )
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    let mut stmt = db
        .prepare("SELECT id,round_number,tick,kind,actor_key,target_key,payload_json,quality FROM match_events WHERE demo_id=?1 AND (?2 IS NULL OR kind=?2) AND (?3 IS NULL OR round_number=?3) AND (?4 IS NULL OR actor_key=?4 OR target_key=?4) ORDER BY tick,id LIMIT ?5 OFFSET ?6")
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    let rows = stmt
        .query_map(
            params![
                demo_id,
                kind,
                filters.round_number,
                player_key,
                page_size,
                (page - 1) * page_size
            ],
            |row| {
                Ok(MatchEventRow {
                    id: row.get(0)?,
                    round_number: row.get(1)?,
                    tick: row.get(2)?,
                    kind: row.get(3)?,
                    actor_key: row.get(4)?,
                    target_key: row.get(5)?,
                    payload_json: row.get(6)?,
                    quality: row.get(7)?,
                })
            },
        )
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    Ok(MatchEventPage {
        items: rows.filter_map(Result::ok).collect(),
        total,
        page,
        page_size,
    })
}

pub fn player_match_detail(
    app: &AppHandle,
    demo_id: i64,
    stable_key: &str,
) -> Result<PlayerMatchDetail, AppError> {
    let player = match_scoreboard(app, demo_id)?
        .into_iter()
        .find(|player| player.stable_key == stable_key)
        .ok_or_else(|| err("DEMO_PLAYER_NOT_FOUND", "该玩家不在此对局中。"))?;
    let db = open_db(app)?;
    let mut stmt = db
        .prepare("SELECT prs.round_number,prs.side,prs.kills,prs.deaths,prs.assists,prs.damage_health,prs.survived,prs.traded,prs.kast FROM player_round_stats prs JOIN players p ON p.id=prs.player_id WHERE prs.demo_id=?1 AND p.stable_key=?2 ORDER BY prs.round_number")
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    let rows = stmt
        .query_map(params![demo_id, stable_key], |row| {
            Ok(PlayerRoundDetail {
                round_number: row.get(0)?,
                side: row.get(1)?,
                kills: row.get(2)?,
                deaths: row.get(3)?,
                assists: row.get(4)?,
                damage_health: row.get(5)?,
                survived: row.get::<_, Option<i64>>(6)?.map(|value| value != 0),
                traded: row.get::<_, Option<i64>>(7)?.map(|value| value != 0),
                kast: row.get::<_, Option<i64>>(8)?.map(|value| value != 0),
            })
        })
        .map_err(|e| err("DEMO_DB_QUERY", e))?;
    Ok(PlayerMatchDetail {
        player,
        rounds: rows.filter_map(Result::ok).collect(),
    })
}

fn csv_cell(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

pub fn export_match(
    app: &AppHandle,
    demo_id: i64,
    format: &str,
    destination_path: &str,
) -> Result<ExportMatchResult, AppError> {
    let destination = PathBuf::from(destination_path);
    let parent = destination
        .parent()
        .filter(|path| path.is_dir())
        .ok_or_else(|| err("DEMO_EXPORT_PATH", "导出目录不存在。"))?;
    let bytes = match format {
        "json" => serde_json::to_vec_pretty(&report(app, demo_id)?)
            .map_err(|e| err("DEMO_EXPORT_SERIALIZE", e))?,
        "csv" => {
            let mut output = String::from("stable_key,name,steam_id,team,kills,deaths,assists,damage,adr,kast_percent,rating\r\n");
            for player in match_scoreboard(app, demo_id)? {
                let cells = [
                    csv_cell(&player.stable_key),
                    csv_cell(player.name.as_deref().unwrap_or("")),
                    csv_cell(player.steam_id.as_deref().unwrap_or("")),
                    csv_cell(player.team_name.as_deref().unwrap_or("")),
                    player
                        .kills
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    player
                        .deaths
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    player
                        .assists
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    player
                        .damage_health
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    player
                        .adr
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    player
                        .kast_percent
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    player
                        .rating
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                ];
                output.push_str(&cells.join(","));
                output.push_str("\r\n");
            }
            output.into_bytes()
        }
        _ => return Err(err("DEMO_EXPORT_FORMAT", "仅支持 json 或 csv。")),
    };
    let part = parent.join(format!(
        ".{}.{}.part",
        destination
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        now_ms()
    ));
    fs::write(&part, &bytes).map_err(|e| err("DEMO_EXPORT_WRITE", e))?;
    if destination.exists() {
        fs::remove_file(&destination).map_err(|e| err("DEMO_EXPORT_REPLACE", e))?;
    }
    fs::rename(&part, &destination).map_err(|e| {
        let _ = fs::remove_file(&part);
        err("DEMO_EXPORT_RENAME", e)
    })?;
    Ok(ExportMatchResult {
        path: destination.display().to_string(),
        bytes_written: bytes.len() as u64,
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

pub fn report_is_presentable(report: &DemoReport, expected_id: i64) -> bool {
    report.schema_version == REPORT_SCHEMA_VERSION
        && report.parser_adapter_version == PARSER_ADAPTER_VERSION
        && report.metrics_version == METRICS_VERSION
        && report.summary.demo_file_id == expected_id
        && report.summary.total_rounds > 0
        && report
            .players
            .iter()
            .any(|player| player.participant_role == "player")
        && matches!(
            report.data_quality.scoreboard_status.as_str(),
            "complete" | "partial"
        )
}

pub fn presentable_report(app: &AppHandle, id: i64) -> Result<DemoReport, AppError> {
    let committed: i64 = open_db(app)?
        .query_row(
            "SELECT COUNT(*) FROM demo_files d WHERE d.id=?1 AND d.status='done' AND d.report_schema_version=?2 AND d.parser_adapter_version=?3 AND d.metrics_version=?4 AND d.scoreboard_status IN ('complete','partial') AND EXISTS(SELECT 1 FROM analysis_jobs j WHERE j.demo_id=d.id AND j.kind='core' AND j.stage='done' AND j.parser_commit=?5 AND j.adapter_version=?3 AND j.schema_version=?2 AND j.metric_version=?4)",
            params![id, REPORT_SCHEMA_VERSION, PARSER_ADAPTER_VERSION, METRICS_VERSION, PARSER_COMMIT],
            |row| row.get(0),
        )
        .map_err(|error| err("DEMO_DB_QUERY", error))?;
    if committed != 1 {
        return Err(err(
            "DEMO_REPORT_NOT_PRESENTABLE",
            "报告尚未完整提交，或解析版本已过期，请重新解析。",
        ));
    }
    let report = report(app, id)?;
    if !report_is_presentable(&report, id) {
        return Err(err(
            "DEMO_REPORT_NOT_PRESENTABLE",
            "报告没有有效回合、参赛玩家或可用计分板。",
        ));
    }
    Ok(report)
}

pub fn path_for_id(app: &AppHandle, id: i64) -> Result<String, AppError> {
    open_db(app)?
        .query_row("SELECT path FROM demo_files WHERE id=?1", [id], |r| {
            r.get(0)
        })
        .map_err(|e| err("DEMO_NOT_FOUND", e))
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
fn completed_round_count(events: &[GameEvent], logical_rounds: &[DemoRound]) -> u32 {
    let match_start = events
        .iter()
        .rposition(|event| event.name == "round_announce_match_start")
        .map_or(0, |index| index + 1);
    let round_ends = events
        .iter()
        .skip(match_start)
        .filter(|event| event.name == "round_end")
        .count() as u32;
    if round_ends > 0 {
        round_ends
    } else {
        logical_rounds
            .iter()
            .filter(|round| round.winner.is_some() && round.end_tick.is_some())
            .count() as u32
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

fn normalize_winner_side(value: Option<&str>) -> Option<&'static str> {
    match value?.trim().to_ascii_uppercase().as_str() {
        "CT" | "COUNTERTERRORIST" | "COUNTER-TERRORIST" | "3" => Some("CT"),
        "T" | "TERRORIST" | "TERRORISTS" | "2" => Some("T"),
        _ => None,
    }
}

fn winner_team_number(event: &GameEvent) -> Option<i32> {
    integer(event, "winner").or_else(|| {
        text(event, "winner").and_then(|winner| match winner.trim().to_ascii_uppercase().as_str() {
            "2" | "T" | "TERRORIST" | "TERRORISTS" => Some(2),
            "3" | "CT" | "COUNTERTERRORIST" | "COUNTER-TERRORIST" => Some(3),
            _ => None,
        })
    })
}

fn final_team_scores(output: &cs2_demoparser::parse_demo::DemoOutput) -> Option<(u32, u32)> {
    if let Some(scores) = output.game_events.iter().rev().find_map(|event| {
        let ct =
            integer(event, "ct_CCSTeam.m_iScore").and_then(|value| u32::try_from(value).ok())?;
        let t = integer(event, "t_CCSTeam.m_iScore").and_then(|value| u32::try_from(value).ok())?;
        Some((ct, t))
    }) {
        return Some(scores);
    }
    let prop_id = |name: &str| {
        output
            .prop_controller
            .prop_infos
            .iter()
            .find(|prop| prop.prop_name == name)
            .map(|prop| prop.id)
    };
    let team_id = prop_id("CCSTeam.m_iTeamNum")?;
    let first_half_id = prop_id("CCSTeam.m_scoreFirstHalf")?;
    let second_half_id = prop_id("CCSTeam.m_scoreSecondHalf")?;
    let overtime_id = prop_id("CCSTeam.m_scoreOvertime");
    let mut ct_score: Option<u32> = None;
    let mut t_score: Option<u32> = None;
    for columns in output
        .df_per_player
        .values()
        .chain(std::iter::once(&output.df))
    {
        let (Some(team_column), Some(first_half_column), Some(second_half_column)) = (
            columns.get(&team_id),
            columns.get(&first_half_id),
            columns.get(&second_half_id),
        ) else {
            continue;
        };
        let row_count = team_column
            .len()
            .min(first_half_column.len())
            .min(second_half_column.len());
        for index in 0..row_count {
            let Some(team) = vec_i32(Some(team_column), index) else {
                continue;
            };
            let Some(first_half) = vec_i32(Some(first_half_column), index) else {
                continue;
            };
            let Some(second_half) = vec_i32(Some(second_half_column), index) else {
                continue;
            };
            let overtime = overtime_id
                .and_then(|id| columns.get(&id))
                .and_then(|column| vec_i32(Some(column), index))
                .unwrap_or(0);
            let Some(score) = first_half
                .checked_add(second_half)
                .and_then(|value| value.checked_add(overtime))
                .and_then(|value| u32::try_from(value).ok())
            else {
                continue;
            };
            match team {
                2 => t_score = Some(t_score.map_or(score, |current| current.max(score))),
                3 => ct_score = Some(ct_score.map_or(score, |current| current.max(score))),
                _ => {}
            }
        }
    }
    Some((ct_score?, t_score?))
}

#[allow(dead_code)]
fn round_winner_scores(rounds: &[DemoRound]) -> Option<(u32, u32)> {
    derive_canonical_score(rounds).map(|score| (score.ct, score.t))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CanonicalScore {
    ct: u32,
    t: u32,
    completed: u32,
}

fn derive_canonical_score(rounds: &[DemoRound]) -> Option<CanonicalScore> {
    let mut seen = BTreeSet::new();
    let mut score = CanonicalScore {
        ct: 0,
        t: 0,
        completed: 0,
    };
    for round in rounds {
        if round.end_tick.is_none() || !seen.insert(round.number) {
            continue;
        }
        match normalize_winner_side(round.winner.as_deref()) {
            Some("CT") => {
                score.ct += 1;
                score.completed += 1;
            }
            Some("T") => {
                score.t += 1;
                score.completed += 1;
            }
            _ => {}
        }
    }
    (score.completed > 0).then_some(score)
}

fn side_switch_ticks(events: &[GameEvent]) -> Vec<i32> {
    let mut counts = BTreeMap::<i32, usize>::new();
    for event in events.iter().filter(|event| event.name == "player_team") {
        let old_team = integer(event, "oldteam");
        let new_team = integer(event, "team");
        if matches!(
            (old_team, new_team),
            (Some(2), Some(3)) | (Some(3), Some(2))
        ) {
            *counts.entry(event.tick).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .filter_map(|(tick, count)| (count >= 2).then_some(tick))
        .collect()
}

fn derive_team_identity_score(
    rounds: &[DemoRound],
    switch_ticks: &[i32],
) -> Option<CanonicalScore> {
    if switch_ticks.is_empty() {
        return derive_canonical_score(rounds);
    }
    let mut seen = BTreeSet::new();
    let mut score = CanonicalScore {
        ct: 0,
        t: 0,
        completed: 0,
    };
    for round in rounds {
        let Some(end_tick) = round.end_tick else {
            continue;
        };
        if !seen.insert(round.number) {
            continue;
        }
        let Some(winner) = normalize_winner_side(round.winner.as_deref()) else {
            continue;
        };
        let switches_before_round = switch_ticks.iter().filter(|tick| **tick < end_tick).count();
        let team_a_side = if switches_before_round % 2 == 0 {
            "T"
        } else {
            "CT"
        };
        if winner == team_a_side {
            score.ct += 1;
        } else {
            score.t += 1;
        }
        score.completed += 1;
    }
    (score.completed > 0).then_some(score)
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
        participant_role: "unknown".into(),
        team_number: None,
        team: None,
        kills: Some(0),
        deaths: Some(0),
        assists: Some(0),
        damage: Some(0),
        headshots: Some(0),
        identity_source: if identity.0.starts_with("steam:") {
            "steamid".into()
        } else if identity.0.starts_with("bot:") {
            "userid".into()
        } else {
            "name".into()
        },
        stats_source: "event_aggregate".into(),
        rounds_played: None,
        rounds_survived: None,
        kast_rounds: None,
        multi_kills: None,
        first_kills: None,
        first_deaths: None,
        trade_kills: None,
        trade_denials: None,
        adr: None,
        kast_percent: None,
        headshot_percent: None,
        round_swing: None,
        economy_adjustment: None,
        rating_status: "unavailable".into(),
        rating: None,
    });
    players.len() - 1
}

fn build_scoreboard(
    output: &cs2_demoparser::parse_demo::DemoOutput,
    entity_status: &str,
    completed_rounds: u32,
    metric_inputs_complete: bool,
) -> (Vec<DemoPlayer>, DemoDataQuality) {
    let mut players = Vec::<DemoPlayer>::new();
    for source in output.player_md.iter().chain(output.roster.iter()) {
        if source
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
        let existing = players.iter().position(|player| player.key == key);
        if let Some(index) = existing {
            let player = &mut players[index];
            player.is_bot |= source.is_bot;
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
            participant_role: if matches!(source.team_number, Some(2 | 3)) {
                "player".into()
            } else if source.team_number == Some(1) {
                "observer".into()
            } else {
                "unknown".into()
            },
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
            rounds_played: None,
            rounds_survived: None,
            kast_rounds: None,
            multi_kills: None,
            first_kills: None,
            first_deaths: None,
            trade_kills: None,
            trade_denials: None,
            adr: None,
            kast_percent: None,
            headshot_percent: None,
            round_swing: None,
            economy_adjustment: None,
            rating_status: "unavailable".into(),
            rating: None,
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
        player.is_bot |= prop_id("bot_difficulty")
            .and_then(|id| last_u32(columns.get(&id)))
            .is_some();
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
        player
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
    for player in &mut players {
        let eligible_team = matches!(player.team_number, Some(2 | 3))
            || matches!(player.team.as_deref(), Some("T" | "CT"));
        if !eligible_team {
            let participated = [player.kills, player.deaths, player.assists, player.damage]
                .into_iter()
                .flatten()
                .any(|value| value > 0);
            player.participant_role = if matches!(player.team_number, Some(0 | 1)) {
                "observer"
            } else if participated {
                "unknown"
            } else {
                "observer"
            }
            .into();
            if player.participant_role == "observer" {
                player.kills = None;
                player.deaths = None;
                player.assists = None;
                player.damage = None;
                player.headshots = None;
                player.stats_source = "observer_excluded".into();
            }
        }
        let has_reliable_totals = matches!(
            player.stats_source.as_str(),
            "controller_total" | "event_aggregate"
        );
        if eligible_team && has_reliable_totals && completed_rounds > 0 && metric_inputs_complete {
            player.rounds_played = Some(completed_rounds);
        }
        player.rating = crate::services::simple_rating::calculate(
            player.kills,
            player.deaths,
            player.assists,
            player.damage,
            player.rounds_played,
        );
        player.rating_status = if player.rating.is_some() {
            "complete"
        } else {
            "unavailable"
        }
        .into();
        if let Some(rounds) = player.rounds_played.filter(|rounds| *rounds > 0) {
            player.adr = player.damage.map(|damage| damage as f64 / rounds as f64);
            player.kast_percent = player
                .kast_rounds
                .map(|kast| kast as f64 * 100.0 / rounds as f64);
        }
        player.headshot_percent = match (player.headshots, player.kills) {
            (Some(headshots), Some(kills)) if kills > 0 => {
                Some(headshots as f64 * 100.0 / kills as f64)
            }
            _ => None,
        };
    }
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
    let rated_players = players
        .iter()
        .filter(|player| matches!(player.team_number, Some(2 | 3)))
        .collect::<Vec<_>>();
    let rated_count = rated_players
        .iter()
        .filter(|player| player.rating.is_some())
        .count();
    let rating_status = if rated_count == 0 {
        "unavailable"
    } else if rated_count == rated_players.len() {
        "complete"
    } else {
        "partial"
    };
    let rating_warnings = if !metric_inputs_complete {
        vec!["Demo 末尾包含未完成回合，终局 totals 已混入该回合；ADR 与简易 Rating 不可用。".into()]
    } else {
        match rating_status {
            "partial" => vec!["部分玩家缺少 K/D/A、伤害或报告回合，未计算简易 Rating。".into()],
            "unavailable" => {
                vec!["当前 Demo 缺少 K/D/A、伤害或已完成回合，无法计算简易 Rating。".into()]
            }
            _ => vec![],
        }
    };
    (
        players,
        DemoDataQuality {
            scoreboard_status: status.into(),
            warnings,
            entity_parse_status: entity_status.into(),
            rating_status: rating_status.into(),
            rating_warnings,
            score_source: "unavailable".into(),
            score_quality: "unavailable".into(),
            score_warnings: Vec::new(),
            canonical_ct_score: None,
            canonical_t_score: None,
            props_ct_score: None,
            props_t_score: None,
            logical_rounds: completed_rounds,
            completed_rounds,
            unfinished_rounds: 0,
        },
    )
}

fn normalized_event(event: &GameEvent) -> DemoEvent {
    DemoEvent {
        tick: event.tick,
        kind: event.name.clone(),
        actor: text(event, "attacker_name").or_else(|| text(event, "user_name")),
        target: text(event, "user_name"),
        actor_key: event_identity(event, "attacker").map(|identity| identity.0),
        target_key: event_identity(event, "user").map(|identity| identity.0),
        weapon: text(event, "weapon"),
        headshot: boolean(event, "headshot"),
        detail: text(event, "site"),
        payload: serde_json::to_value(event).unwrap_or(serde_json::Value::Null),
    }
}

fn parse_report(id: i64, path: &Path, meta: &fs::Metadata) -> Result<DemoReport, AppError> {
    let bytes = fs::read(path).map_err(|e| err("DEMO_FILE_READ", e))?;
    let demo_sha256 = format!("{:x}", Sha256::digest(&bytes));
    let h = create_huffman_lookup_table();
    let wanted: Vec<String> = [
        "round_start",
        "round_announce_match_start",
        "round_announce_last_round_half",
        "round_freeze_end",
        "round_end",
        "round_officially_ended",
        "player_team",
        "player_death",
        "player_hurt",
        "item_purchase",
        "weapon_fire",
        "player_blind",
        "flashbang_detonate",
        "hegrenade_detonate",
        "smokegrenade_detonate",
        "inferno_startburn",
        "inferno_expire",
        "chat_message",
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
        ("bot_difficulty", "CCSPlayerController.m_iPawnBotDifficulty"),
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
            wanted_other_props: vec![
                "CCSTeam.m_iTeamNum".to_string(),
                "CCSTeam.m_iScore".to_string(),
                "CCSTeam.m_scoreFirstHalf".to_string(),
                "CCSTeam.m_scoreSecondHalf".to_string(),
                "CCSTeam.m_scoreOvertime".to_string(),
            ],
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
        Err(error) if error == "ParserPanic" => {
            return Err(err("DEMO_PARSER_PANIC", "解析器发生 panic，任务已隔离。"));
        }
        Err(error) => return Err(err("DEMO_PARSER_UNSUPPORTED", error)),
    };
    let header = output
        .header
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let mut rounds: Vec<DemoRound> = Vec::new();
    let mut current: Option<usize> = None;
    for event in &output.game_events {
        if event.name == "round_announce_match_start" {
            rounds.clear();
            current = None;
            continue;
        }
        if event.name == "round_start" {
            rounds.push(DemoRound {
                number: (rounds.len() + 1) as u32,
                start_tick: Some(event.tick),
                end_tick: None,
                winner: None,
                reason: None,
                kills: vec![],
                bomb_events: vec![],
                events: vec![normalized_event(event)],
            });
            current = Some(rounds.len() - 1);
            continue;
        }
        if event.name == "round_freeze_end" {
            if let Some(index) = current.filter(|index| rounds[*index].end_tick.is_none()) {
                rounds[index].start_tick = Some(event.tick);
                rounds[index].events.push(normalized_event(event));
            } else {
                rounds.push(DemoRound {
                    number: (rounds.len() + 1) as u32,
                    start_tick: Some(event.tick),
                    end_tick: None,
                    winner: None,
                    reason: None,
                    kills: vec![],
                    bomb_events: vec![],
                    events: vec![normalized_event(event)],
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
                events: vec![],
            });
            current = Some(0)
        }
        let round = &mut rounds[current.unwrap()];
        if event.name == "round_end" {
            round.end_tick = Some(event.tick);
            round.winner = team_label(winner_team_number(event));
            round.reason = text(event, "reason");
            round.events.push(normalized_event(event));
            continue;
        }
        if event.name == "round_officially_ended" {
            round.end_tick.get_or_insert(event.tick);
            round.events.push(normalized_event(event));
            continue;
        }
        let item = normalized_event(event);
        round.events.push(item.clone());
        if event.name == "player_death" {
            round.kills.push(item)
        } else if event.name.starts_with("bomb_") {
            round.bomb_events.push(item)
        }
    }
    let score_switch_ticks = side_switch_ticks(&output.game_events);
    let raw_round_winners = rounds
        .iter()
        .map(|round| serde_json::json!({"number": round.number, "endTick": round.end_tick, "winner": round.winner}))
        .collect::<Vec<_>>();
    let canonical_score = derive_team_identity_score(&rounds, &score_switch_ticks);
    if !score_switch_ticks.is_empty() {
        for round in &mut rounds {
            let Some(end_tick) = round.end_tick else {
                continue;
            };
            let Some(winner) = normalize_winner_side(round.winner.as_deref()) else {
                continue;
            };
            let switches_before_round = score_switch_ticks
                .iter()
                .filter(|tick| **tick < end_tick)
                .count();
            let team_a_side = if switches_before_round % 2 == 0 {
                "T"
            } else {
                "CT"
            };
            round.winner = Some(if winner == team_a_side { "CT" } else { "T" }.into());
        }
    }
    let completed_rounds = canonical_score
        .map(|score| score.completed)
        .unwrap_or_else(|| completed_round_count(&output.game_events, &rounds));
    let metric_inputs_complete = completed_rounds as usize == rounds.len()
        && rounds.iter().all(|round| round.end_tick.is_some());
    let (players, mut data_quality) = build_scoreboard(
        &output,
        entity_status,
        completed_rounds,
        metric_inputs_complete,
    );
    let total_kills = rounds.iter().map(|r| r.kills.len() as u32).sum();
    let props_scores = final_team_scores(&output);
    let winner_scores = canonical_score.map(|score| (score.ct, score.t));
    data_quality.props_ct_score = props_scores.map(|score| score.0);
    data_quality.props_t_score = props_scores.map(|score| score.1);
    data_quality.canonical_ct_score = winner_scores.map(|score| score.0);
    data_quality.canonical_t_score = winner_scores.map(|score| score.1);
    let scores = match (props_scores, winner_scores) {
        (Some(props), Some(winners)) => {
            data_quality.score_source = "round_winners+team_switches".into();
            data_quality.score_quality = "complete".into();
            if props != winners {
                if props.0 == winners.1 && props.1 == winners.0 {
                    data_quality
                        .score_warnings
                        .push("TEAM_PROPS_SIDE_MISMATCH".into());
                } else {
                    data_quality
                        .score_warnings
                        .push("TEAM_PROPS_SCORE_MISMATCH".into());
                }
            }
            Some(winners)
        }
        (None, Some(winners)) => {
            data_quality.score_source = "round_winners+team_switches".into();
            data_quality.score_quality = "complete".into();
            Some(winners)
        }
        (Some(props), None) => {
            data_quality.score_source = "team_props_fallback".into();
            data_quality.score_quality = "partial".into();
            Some(props)
        }
        _ => {
            data_quality.score_source = "unavailable".into();
            data_quality.score_quality = "unavailable".into();
            data_quality.score_warnings.push("SCORE_UNAVAILABLE".into());
            None
        }
    };
    let scores = scores.filter(|(ct, t)| ct + t == completed_rounds);
    if winner_scores.is_some() && scores.is_none() {
        data_quality.score_quality = "conflict".into();
        data_quality
            .score_warnings
            .push("SCORE_ROUND_COUNT_CONFLICT".into());
    }
    data_quality.logical_rounds = rounds.len() as u32;
    data_quality.completed_rounds = completed_rounds;
    data_quality.unfinished_rounds = rounds.len().saturating_sub(completed_rounds as usize) as u32;
    let parsed = now_ms();
    let report = DemoReport {
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
            total_rounds: completed_rounds,
            total_kills,
            team_a_score: scores.map(|score| score.0),
            team_b_score: scores.map(|score| score.1),
            parsed_at: parsed,
        },
        rounds,
    };
    let diagnostic = serde_json::json!({
        "demoPath": path.display().to_string(),
        "demoSizeBytes": meta.len(),
        "demoMtimeMs": mtime_ms(meta),
        "demoSha256": demo_sha256,
        "roundEndEvents": output.game_events.iter().filter(|event| event.name == "round_end").map(|event| serde_json::json!({"tick": event.tick, "winner": winner_team_number(event), "winnerSide": text(event, "winner")})).collect::<Vec<_>>(),
        "halfAnnouncements": output.game_events.iter().filter(|event| event.name.contains("half")).map(|event| serde_json::json!({"tick": event.tick, "name": event.name})).collect::<Vec<_>>(),
        "teamSwitchTicks": score_switch_ticks,
        "playerTeamEvents": output.game_events.iter().filter(|event| event.name == "player_team").map(|event| serde_json::to_value(event).unwrap_or_default()).collect::<Vec<_>>(),
        "finalRoster": report.players.iter().map(|player| serde_json::json!({"key": player.key, "name": player.name, "userId": player.user_id, "teamNumber": player.team_number, "role": player.participant_role})).collect::<Vec<_>>(),
        "rawRoundWinners": raw_round_winners,
        "canonicalRounds": report.rounds.iter().map(|round| serde_json::json!({"number": round.number, "endTick": round.end_tick, "winner": round.winner})).collect::<Vec<_>>(),
        "canonicalScore": { "ct": report.data_quality.canonical_ct_score, "t": report.data_quality.canonical_t_score, "completed": report.data_quality.completed_rounds },
        "teamProps": { "ct": report.data_quality.props_ct_score, "t": report.data_quality.props_t_score },
        "summary": { "teamAScore": report.summary.team_a_score, "teamBScore": report.summary.team_b_score, "totalRounds": report.summary.total_rounds },
        "warnings": report.data_quality.score_warnings,
    });
    let diagnostic_path = path.with_file_name("score-diagnostic.json");
    if let Ok(bytes) = serde_json::to_vec_pretty(&diagnostic) {
        let _ = fs::write(diagnostic_path, bytes);
    }
    Ok(report)
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

    #[test]
    fn position_cache_reuses_and_invalidates_spatial_results() {
        let key = (-9_001, 3, 8);
        cache_positions(key, &[]);
        assert!(cached_positions(key).is_some_and(|points| points.is_empty()));
        invalidate_position_cache(key.0, key.2);
        assert!(cached_positions(key).is_none());
    }
    use cs2_demoparser::second_pass::game_events::EventField;

    fn create_v7_map_table(db: &Connection) {
        db.execute_batch(
            "CREATE TABLE map_metadata(map_name TEXT PRIMARY KEY,upstream_commit TEXT NOT NULL,asset_sha256 TEXT NOT NULL,pos_x REAL NOT NULL,pos_y REAL NOT NULL,scale REAL NOT NULL,threshold_z REAL,updated_at INTEGER NOT NULL,radar_asset TEXT,lower_radar_asset TEXT,lower_asset_sha256 TEXT,radar_size INTEGER NOT NULL DEFAULT 1024,source TEXT NOT NULL DEFAULT 'cs-demo-manager');",
        ).unwrap();
    }

    #[test]
    fn map_metadata_seed_is_idempotent_and_preserves_custom_rows() {
        let db = Connection::open_in_memory().unwrap();
        create_v7_map_table(&db);
        db.execute("INSERT INTO map_metadata(map_name,upstream_commit,asset_sha256,pos_x,pos_y,scale,threshold_z,updated_at,radar_size,source)VALUES('workshop_custom','user','user',0,0,1,0,0,1024,'user')", []).unwrap();
        seed_map_metadata(&db).unwrap();
        seed_map_metadata(&db).unwrap();
        let upstream: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM map_metadata WHERE source='cs-demo-manager'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let custom: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM map_metadata WHERE source='user'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!((upstream, custom), (44, 1));
        let lower: Option<String> = db
            .query_row(
                "SELECT lower_radar_asset FROM map_metadata WHERE map_name='de_nuke'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(lower.as_deref(), Some("de_nuke_lower.png"));
    }

    #[test]
    fn heatmap_filters_select_round_player_team_and_layer_independently() {
        let metadata = map_metadata::embedded()
            .unwrap()
            .into_iter()
            .find(|map| map.name == "de_dust2")
            .unwrap();
        let point = |round_number: i64, player_key: &str, team_number: i64, z: f64| HeatmapPoint {
            tick: 100,
            round_number: Some(round_number),
            player_key: Some(player_key.into()),
            team_number: Some(team_number),
            x: metadata.position_x,
            y: metadata.position_y,
            z: Some(z),
            weight: 1.0,
            kind: "player_death".into(),
        };
        let upper = point(1, "steam:one", 2, 100.0);
        let lower = point(2, "steam:two", 3, -100.0);
        assert!(heatmap_point_matches(
            &metadata,
            &HeatmapFilters::default(),
            &upper
        ));

        let mut filters = HeatmapFilters {
            round_numbers: vec![2],
            ..HeatmapFilters::default()
        };
        assert!(!heatmap_point_matches(&metadata, &filters, &upper));
        assert!(heatmap_point_matches(&metadata, &filters, &lower));

        filters = HeatmapFilters {
            player_keys: vec!["steam:one".into()],
            ..HeatmapFilters::default()
        };
        assert!(heatmap_point_matches(&metadata, &filters, &upper));
        assert!(!heatmap_point_matches(&metadata, &filters, &lower));

        filters = HeatmapFilters {
            team_numbers: vec![3],
            ..HeatmapFilters::default()
        };
        assert!(!heatmap_point_matches(&metadata, &filters, &upper));
        assert!(heatmap_point_matches(&metadata, &filters, &lower));

        filters = HeatmapFilters {
            layer: "lower".into(),
            ..HeatmapFilters::default()
        };
        assert!(!heatmap_point_matches(&metadata, &filters, &upper));
        assert!(heatmap_point_matches(&metadata, &filters, &lower));
    }

    #[test]
    fn launch_demo_validation_and_arguments_are_safe() {
        assert!(validate_demo_launch(0, None).is_ok());
        assert!(validate_demo_launch(10_000_000, Some(" ")).is_ok());
        assert!(validate_demo_launch(-1, None)
            .unwrap_err()
            .into_string()
            .contains("DEMO_TICK_RANGE"));
        assert!(validate_demo_launch(10_000_001, None)
            .unwrap_err()
            .into_string()
            .contains("DEMO_TICK_RANGE"));
        assert!(validate_demo_launch(1, Some("steam:player"))
            .unwrap_err()
            .into_string()
            .contains("DEMO_PLAYER_UNSUPPORTED"));

        let path = Path::new(r"D:\演示目录\round one.dem");
        let args = demo_launch_args(path, 10_000_000);
        assert_eq!(args.len(), 6);
        assert_eq!(args[3], path.as_os_str());
        assert_eq!(args[5], OsString::from("10000000"));
    }

    #[test]
    #[ignore = "requires CS2AS_PREFLIGHT_DB and CS2AS_PREFLIGHT_HEATMAP_OUTPUT"]
    fn preflight_real_db_heatmap_export_and_filters() {
        let db_path = std::env::var("CS2AS_PREFLIGHT_DB").expect("preflight DB path");
        let output = PathBuf::from(
            std::env::var("CS2AS_PREFLIGHT_HEATMAP_OUTPUT").expect("heatmap output path"),
        );
        let db = Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .unwrap();
        let filters = HeatmapFilters {
            kind: "smokegrenade_detonate".into(),
            ..HeatmapFilters::default()
        };
        let points = heatmap_points_from_db(&db, 4, &filters).unwrap();
        assert!(!points.is_empty());
        assert!(points.iter().all(|point| point.tick >= 0));

        let first = points.first().unwrap();
        let round_points = heatmap_points_from_db(
            &db,
            4,
            &HeatmapFilters {
                kind: filters.kind.clone(),
                round_numbers: vec![first.round_number.unwrap()],
                ..HeatmapFilters::default()
            },
        )
        .unwrap();
        let player_points = heatmap_points_from_db(
            &db,
            4,
            &HeatmapFilters {
                kind: filters.kind.clone(),
                player_keys: vec![first.player_key.clone().unwrap()],
                ..HeatmapFilters::default()
            },
        )
        .unwrap();
        let team_points = heatmap_points_from_db(
            &db,
            4,
            &HeatmapFilters {
                kind: filters.kind.clone(),
                team_numbers: vec![first.team_number.unwrap()],
                ..HeatmapFilters::default()
            },
        )
        .unwrap();
        let upper_points = heatmap_points_from_db(
            &db,
            4,
            &HeatmapFilters {
                kind: filters.kind.clone(),
                layer: "upper".into(),
                ..HeatmapFilters::default()
            },
        )
        .unwrap();
        for filtered in [&round_points, &player_points, &team_points, &upper_points] {
            assert!(!filtered.is_empty());
            assert!(filtered.len() <= points.len());
        }
        assert!(heatmap_points_from_db(
            &db,
            4,
            &HeatmapFilters {
                kind: "player_death".into(),
                ..HeatmapFilters::default()
            }
        )
        .unwrap()
        .is_empty());

        let maps = map_metadata::embedded().unwrap();
        let dust2 = maps.iter().find(|map| map.name == "de_dust2").unwrap();
        let radar = include_bytes!("../../resources/demo-maps/radars/de_dust2.png");
        let (pixels, rendered, discarded) =
            crate::demo::heatmap::render(dust2, radar, &points, 18, 0.72).unwrap();
        let part = output.with_extension("png.part");
        crate::demo::heatmap::encode_rgba(&part, &pixels).unwrap();
        crate::demo::heatmap::replace_file(&part, &output).unwrap();
        assert!(!part.exists());
        let png = fs::read(&output).unwrap();
        let (width, height, _) = crate::demo::heatmap::decode_rgba(&png).unwrap();
        assert_eq!((width, height), (1024, 1024));
        println!(
            "{}",
            serde_json::json!({
                "path": output,
                "inputPoints": points.len(),
                "renderedPoints": rendered,
                "discardedPoints": discarded,
                "roundFilteredPoints": round_points.len(),
                "playerFilteredPoints": player_points.len(),
                "teamFilteredPoints": team_points.len(),
                "upperFilteredPoints": upper_points.len(),
                "width": width,
                "height": height,
                "assetSha256": crate::demo::heatmap::sha256(radar),
                "outputSha256": crate::demo::heatmap::sha256(&png),
            })
        );
    }

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
    fn completed_rounds_do_not_double_count_official_events() {
        let event = |name: &str| GameEvent {
            name: name.into(),
            tick: 1,
            fields: vec![],
        };
        let mut events = (0..6).map(|_| event("round_end")).collect::<Vec<_>>();
        events.extend((0..10).map(|_| event("round_officially_ended")));
        assert_eq!(completed_round_count(&events, &[]), 6);
    }

    #[test]
    fn completed_rounds_ignore_warmup_before_match_start() {
        let event = |name: &str| GameEvent {
            name: name.into(),
            tick: 1,
            fields: vec![],
        };
        let events = vec![
            event("round_end"),
            event("round_announce_match_start"),
            event("round_end"),
            event("round_officially_ended"),
            event("round_start"),
        ];
        assert_eq!(completed_round_count(&events, &[]), 1);
    }

    #[test]
    fn round_winner_values_produce_visible_scores() {
        let round_end = |winner: Variant| GameEvent {
            name: "round_end".into(),
            tick: 1,
            fields: vec![EventField {
                name: "winner".into(),
                data: Some(winner),
            }],
        };
        assert_eq!(winner_team_number(&round_end(Variant::I32(3))), Some(3));
        assert_eq!(
            winner_team_number(&round_end(Variant::String("T".into()))),
            Some(2)
        );

        let rounds = vec![
            DemoRound {
                number: 1,
                start_tick: None,
                end_tick: Some(1),
                winner: Some("CT".into()),
                reason: None,
                kills: vec![],
                bomb_events: vec![],
                events: vec![],
            },
            DemoRound {
                number: 2,
                start_tick: None,
                end_tick: Some(2),
                winner: Some("T".into()),
                reason: None,
                kills: vec![],
                bomb_events: vec![],
                events: vec![],
            },
            DemoRound {
                number: 3,
                start_tick: None,
                end_tick: Some(3),
                winner: Some("CT".into()),
                reason: None,
                kills: vec![],
                bomb_events: vec![],
                events: vec![],
            },
        ];
        assert_eq!(round_winner_scores(&rounds), Some((2, 1)));
    }

    #[test]
    fn canonical_score_is_ct_16_t_12_for_28_rounds() {
        let rounds = (0..28)
            .map(|index| DemoRound {
                number: index + 1,
                start_tick: Some(index as i32),
                end_tick: Some(index as i32 + 1),
                winner: Some(if index < 16 { "CT" } else { "T" }.into()),
                reason: None,
                kills: vec![],
                bomb_events: vec![],
                events: vec![],
            })
            .collect::<Vec<_>>();
        assert_eq!(
            derive_canonical_score(&rounds),
            Some(CanonicalScore {
                ct: 16,
                t: 12,
                completed: 28
            })
        );
    }

    #[test]
    fn canonical_score_deduplicates_rounds_and_ignores_unfinished_tail() {
        let round = |number: u32, winner: &str, end_tick: Option<i32>| DemoRound {
            number,
            start_tick: None,
            end_tick,
            winner: Some(winner.into()),
            reason: None,
            kills: vec![],
            bomb_events: vec![],
            events: vec![],
        };
        let rounds = vec![
            round(1, "CT", Some(10)),
            round(1, "CT", Some(10)),
            round(2, "T", Some(20)),
            round(3, "T", None),
        ];
        assert_eq!(
            derive_canonical_score(&rounds),
            Some(CanonicalScore {
                ct: 1,
                t: 1,
                completed: 2
            })
        );
    }

    #[test]
    fn canonical_string_winner_is_not_side_reversed() {
        let round = DemoRound {
            number: 1,
            start_tick: None,
            end_tick: Some(1),
            winner: Some("T".into()),
            reason: None,
            kills: vec![],
            bomb_events: vec![],
            events: vec![],
        };
        assert_eq!(
            derive_canonical_score(&[round]),
            Some(CanonicalScore {
                ct: 0,
                t: 1,
                completed: 1
            })
        );
    }

    #[test]
    fn trade_window_includes_exact_five_seconds_only() {
        assert!(within_trade_window(319, 1.0 / 64.0));
        assert!(within_trade_window(320, 1.0 / 64.0));
        assert!(!within_trade_window(321, 1.0 / 64.0));
        assert!(!within_trade_window(-1, 1.0 / 64.0));
    }

    #[test]
    fn cache_requires_current_schema_adapter_and_metrics() {
        assert!(!cache_is_current(
            "done",
            (10, 20),
            (10, 20),
            3,
            "2",
            "openrating-demo-v1"
        ));
        assert!(!cache_is_current(
            "done",
            (10, 20),
            (10, 20),
            5,
            "2",
            "simple-rating-v1"
        ));
        assert!(cache_is_current(
            "done",
            (10, 20),
            (10, 20),
            6,
            "9",
            "simple-rating-v1"
        ));
    }

    #[test]
    #[ignore = "uses a local user Demo selected through CS2AS_DEMO"]
    fn real_demo_scoreboard() {
        let path = PathBuf::from(std::env::var("CS2AS_DEMO").expect("CS2AS_DEMO path"));
        let meta = fs::metadata(&path).expect("Demo metadata");
        let demo_id = std::env::var("CS2AS_SCORE_DB")
            .ok()
            .and_then(|db| Connection::open(db).ok())
            .and_then(|db| {
                db.query_row(
                    "SELECT id FROM demo_files WHERE path=?1",
                    [&path.display().to_string()],
                    |row| row.get(0),
                )
                .ok()
            })
            .unwrap_or(1);
        let report = parse_report(demo_id, &path, &meta).expect("Demo parse");
        let replay = parse_report(demo_id, &path, &meta).expect("deterministic Demo replay");
        assert_eq!(report.summary.team_a_score, replay.summary.team_a_score);
        assert_eq!(report.summary.team_b_score, replay.summary.team_b_score);
        assert_eq!(
            report
                .rounds
                .iter()
                .map(|round| round.winner.as_deref())
                .collect::<Vec<_>>(),
            replay
                .rounds
                .iter()
                .map(|round| round.winner.as_deref())
                .collect::<Vec<_>>()
        );
        println!(
            "map={:?} score={:?}:{:?} completed_rounds={} logical_rounds={} players={} bots={} rating_status={} score_quality={} score_warnings={:?}",
            report.summary.map_name,
            report.summary.team_a_score,
            report.summary.team_b_score,
            report.summary.total_rounds,
            report.rounds.len(),
            report.players.len(),
            report.players.iter().filter(|player| player.is_bot).count(),
            report.data_quality.rating_status,
            report.data_quality.score_quality,
            report.data_quality.score_warnings
        );
        assert_eq!(report.schema_version, REPORT_SCHEMA_VERSION);
        assert_eq!(report.metrics_version, "simple-rating-v1");
        assert_eq!(report.summary.map_name.as_deref(), Some("de_nuke"));
        assert_eq!(report.summary.team_a_score, Some(16));
        assert_eq!(report.summary.team_b_score, Some(12));
        assert_eq!(report.summary.total_rounds, 28);
        assert!(report
            .data_quality
            .score_source
            .starts_with("round_winners"));
        assert!(!report
            .data_quality
            .score_warnings
            .contains(&"SCORE_ROUND_COUNT_CONFLICT".into()));
        assert!(report
            .data_quality
            .score_warnings
            .contains(&"TEAM_PROPS_SIDE_MISMATCH".into()));
        if let Ok(db_path) = std::env::var("CS2AS_SCORE_DB") {
            let db = Connection::open(db_path).expect("score DB");
            persist_normalized_report(&db, &report).expect("persist score report");
            let json = serde_json::to_string(&report).expect("serialize score report");
            db.execute("UPDATE demo_files SET status='done',map_name=?2,total_rounds=?3,kills=?4,parsed_at=?5,report_json=?6,report_schema_version=?7,parser_adapter_version=?8,metrics_version=?9,scoreboard_status=?10,error_code=NULL,error_detail=NULL WHERE id=?1", params![demo_id, report.summary.map_name, report.summary.total_rounds, report.summary.total_kills, report.summary.parsed_at, json, REPORT_SCHEMA_VERSION, PARSER_ADAPTER_VERSION, METRICS_VERSION, report.data_quality.scoreboard_status]).expect("update demo report");
        }
    }
}
