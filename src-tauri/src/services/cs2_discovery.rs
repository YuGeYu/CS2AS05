use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sysinfo::System;
use tauri::ipc::Channel;

use crate::errors::AppError;
use crate::models::cs2::{Cs2RootScanEvent, Cs2RootScanSummary, Cs2SuggestedRoot};

const SCAN_LIMIT: Duration = Duration::from_secs(10);
const MAX_CANDIDATES: usize = 3;
const CS2_FOLDER_NAME: &str = "Counter-Strike Global Offensive";

#[derive(Clone, Default)]
pub struct ScanCoordinator {
    token: Arc<Mutex<Option<Arc<AtomicBool>>>>,
}

impl ScanCoordinator {
    pub fn start(&self) -> Result<Arc<AtomicBool>, AppError> {
        let mut active = self
            .token
            .lock()
            .map_err(|_| AppError::runtime("[CS2_SCAN_COORDINATOR] 扫描状态不可用。"))?;
        if active.is_some() {
            return Err(AppError::runtime(
                "[CS2_SCAN_ALREADY_RUNNING] 已有目录扫描正在运行。",
            ));
        }
        let token = Arc::new(AtomicBool::new(false));
        *active = Some(token.clone());
        Ok(token)
    }

    pub fn stop(&self) -> bool {
        self.token
            .lock()
            .ok()
            .and_then(|active| {
                active
                    .as_ref()
                    .map(|token| token.swap(true, Ordering::SeqCst))
            })
            .is_some()
    }

    fn finish(&self) {
        if let Ok(mut active) = self.token.lock() {
            *active = None;
        }
    }
}

pub fn scan(
    coordinator: &ScanCoordinator,
    channel: Channel<Cs2RootScanEvent>,
) -> Result<Cs2RootScanSummary, AppError> {
    let token = coordinator.start()?;
    let result = scan_with(
        &token,
        SCAN_LIMIT,
        MAX_CANDIDATES,
        discover_locations(),
        |event| {
            let _ = channel.send(event);
        },
    );
    coordinator.finish();
    Ok(result)
}

#[derive(Clone, Debug)]
struct Probe {
    path: PathBuf,
    source: String,
    manifest: bool,
    priority: u8,
}

fn discover_locations() -> Vec<Probe> {
    let mut steam_roots = registry_steam_roots();
    steam_roots.extend(running_steam_roots());
    steam_roots.extend(common_steam_roots());
    dedupe_paths(&mut steam_roots);

    let mut probes = registry_cs2_locations();
    for root in steam_roots {
        collect_library_probes(&root, &mut probes);
    }
    dedupe_probes(&mut probes);
    probes.sort_by(|left, right| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| canonical_key(&left.path).cmp(&canonical_key(&right.path)))
    });
    probes
}

fn collect_library_probes(steam_root: &Path, probes: &mut Vec<Probe>) {
    let mut libraries = vec![steam_root.to_path_buf()];
    let library_file = steam_root.join("steamapps/libraryfolders.vdf");
    if let Ok(text) = fs::read_to_string(&library_file) {
        if let Ok(parsed) = parse_key_values(&text) {
            libraries.extend(library_paths(&parsed));
        }
    }
    dedupe_paths(&mut libraries);

    for library in libraries {
        let steamapps = library.join("steamapps");
        let manifest = steamapps.join("appmanifest_730.acf");
        if let Ok(text) = fs::read_to_string(&manifest) {
            if let Ok(parsed) = parse_key_values(&text) {
                if let Some(install_dir) = first_string(&parsed, "installdir") {
                    probes.push(Probe {
                        path: steamapps.join("common").join(install_dir),
                        source: "Steam App 730 清单".into(),
                        manifest: true,
                        priority: 1,
                    });
                }
            }
        }
        probes.push(Probe {
            path: steamapps.join("common").join(CS2_FOLDER_NAME),
            source: "Steam 库配置".into(),
            manifest: false,
            priority: 3,
        });
    }
}

fn common_steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for drive in b'A'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\", drive as char));
        if !root.is_dir() {
            continue;
        }
        for relative in [
            "Program Files (x86)/Steam",
            "Program Files/Steam",
            "Steam",
            "SteamLibrary",
            "Games/SteamLibrary",
            "Games/Steam",
            "Steam Games",
        ] {
            roots.push(root.join(relative));
        }
    }
    roots
}

fn running_steam_roots() -> Vec<PathBuf> {
    let system = System::new_all();
    system
        .processes()
        .values()
        .filter(|process| process.name().eq_ignore_ascii_case("steam.exe"))
        .filter_map(|process| process.exe())
        .filter_map(Path::parent)
        .map(Path::to_path_buf)
        .collect()
}

#[cfg(windows)]
fn registry_steam_roots() -> Vec<PathBuf> {
    use winreg::enums::{
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
    };
    use winreg::RegKey;

    let mut roots = Vec::new();
    let locations = [
        (HKEY_CURRENT_USER, r"Software\Valve\Steam", "SteamPath"),
        (HKEY_CURRENT_USER, r"Software\Valve\Steam", "SteamExe"),
        (HKEY_LOCAL_MACHINE, r"Software\Valve\Steam", "InstallPath"),
        (
            HKEY_LOCAL_MACHINE,
            r"Software\Wow6432Node\Valve\Steam",
            "InstallPath",
        ),
    ];
    for view in [KEY_READ | KEY_WOW64_32KEY, KEY_READ | KEY_WOW64_64KEY] {
        for (hive, key, value) in locations {
            let hive = RegKey::predef(hive);
            if let Ok(key) = hive.open_subkey_with_flags(key, view) {
                if let Ok(path) = key.get_value::<String, _>(value) {
                    let path = PathBuf::from(path.replace('/', "\\"));
                    roots.push(
                        if path
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
                        {
                            path.parent().unwrap_or(&path).to_path_buf()
                        } else {
                            path
                        },
                    );
                }
            }
        }
    }
    roots
}

#[cfg(not(windows))]
fn registry_steam_roots() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(windows)]
fn registry_cs2_locations() -> Vec<Probe> {
    use winreg::enums::{
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
    };
    use winreg::RegKey;

    let mut probes = Vec::new();
    let keys = [
        (
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Steam App 730",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Steam App 730",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Steam App 730",
        ),
    ];
    for view in [KEY_READ | KEY_WOW64_32KEY, KEY_READ | KEY_WOW64_64KEY] {
        for (hive, path) in keys {
            if let Ok(key) = RegKey::predef(hive).open_subkey_with_flags(path, view) {
                if let Ok(location) = key.get_value::<String, _>("InstallLocation") {
                    probes.push(Probe {
                        path: PathBuf::from(location),
                        source: "Windows App 730 注册表".into(),
                        manifest: true,
                        priority: 0,
                    });
                }
            }
        }
    }
    probes
}

#[cfg(not(windows))]
fn registry_cs2_locations() -> Vec<Probe> {
    Vec::new()
}

fn scan_with(
    token: &AtomicBool,
    limit: Duration,
    max_candidates: usize,
    locations: Vec<Probe>,
    mut emit: impl FnMut(Cs2RootScanEvent),
) -> Cs2RootScanSummary {
    let started = Instant::now();
    let mut checked = 0u32;
    let mut warnings = Vec::new();
    let mut candidates: Vec<(u8, Cs2SuggestedRoot)> = Vec::new();
    let mut seen = HashSet::new();

    for location in locations {
        if stop_reason(token, started, limit, candidates.len(), max_candidates).is_some() {
            break;
        }
        checked += 1;
        emit(Cs2RootScanEvent::Progress {
            elapsed_ms: started.elapsed().as_millis() as u64,
            checked_locations: checked,
            current_location: Some(location.path.display().to_string()),
        });
        match candidate_from_probe(&location) {
            Ok(Some(candidate)) => {
                let canonical = canonical_key(Path::new(&candidate.path));
                if seen.insert(canonical) && candidates.len() < max_candidates {
                    candidates.push((location.priority, candidate.clone()));
                    emit(Cs2RootScanEvent::Candidate {
                        elapsed_ms: started.elapsed().as_millis() as u64,
                        checked_locations: checked,
                        candidate,
                    });
                }
            }
            Ok(None) => {}
            Err(error) => warnings.push(format!("{}：{error}", location.path.display())),
        }
    }

    while stop_reason(token, started, limit, candidates.len(), max_candidates).is_none() {
        std::thread::sleep(Duration::from_millis(25).min(limit));
    }
    let reason =
        stop_reason(token, started, limit, candidates.len(), max_candidates).unwrap_or("timeout");
    candidates.sort_by(|(left_priority, left), (right_priority, right)| {
        confidence_rank(&left.confidence)
            .cmp(&confidence_rank(&right.confidence))
            .then_with(|| left_priority.cmp(right_priority))
            .then_with(|| {
                canonical_key(Path::new(&left.path)).cmp(&canonical_key(Path::new(&right.path)))
            })
    });
    Cs2RootScanSummary {
        candidates: candidates
            .into_iter()
            .map(|(_, candidate)| candidate)
            .collect(),
        elapsed_ms: started.elapsed().as_millis() as u64,
        checked_locations: checked,
        stop_reason: reason.into(),
        warnings,
    }
}

fn stop_reason(
    token: &AtomicBool,
    started: Instant,
    limit: Duration,
    count: usize,
    max_candidates: usize,
) -> Option<&'static str> {
    if count >= max_candidates {
        Some("threeFound")
    } else if started.elapsed() >= limit {
        Some("timeout")
    } else if token.load(Ordering::SeqCst) {
        Some("userStopped")
    } else {
        None
    }
}

fn candidate_from_probe(probe: &Probe) -> Result<Option<Cs2SuggestedRoot>, String> {
    let root = match normalize_candidate(&probe.path) {
        Ok(root) => root,
        Err(_) if !probe.path.exists() => return Ok(None),
        Err(error) => return Err(error),
    };
    let csgo = root.join("game/csgo");
    if !csgo.is_dir() || !csgo.join("gameinfo.gi").is_file() {
        return Ok(None);
    }
    let executable = root.join("game/bin/win64/cs2.exe");
    let mut evidence = vec!["game/csgo/gameinfo.gi".into()];
    if executable.is_file() {
        evidence.push("game/bin/win64/cs2.exe".into());
    }
    if probe.manifest {
        evidence.push("appmanifest_730.acf".into());
    }
    Ok(Some(Cs2SuggestedRoot {
        path: root.display().to_string(),
        source: probe.source.clone(),
        confidence: if executable.is_file() || probe.manifest {
            "verified".into()
        } else {
            "likely".into()
        },
        evidence,
    }))
}

fn normalize_candidate(path: &Path) -> Result<PathBuf, String> {
    let root = if path.join("game/csgo").is_dir() {
        path.to_path_buf()
    } else if path
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("game"))
    {
        path.parent().ok_or("game 目录没有根目录")?.to_path_buf()
    } else if path
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("csgo"))
    {
        path.parent()
            .and_then(Path::parent)
            .ok_or("csgo 目录层级无效")?
            .to_path_buf()
    } else {
        return Err("目录层级无效".into());
    };
    dunce::canonicalize(root).map_err(|error| error.to_string())
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeyValue {
    String(String),
    Object(Vec<(String, KeyValue)>),
}

pub fn parse_key_values(input: &str) -> Result<KeyValue, String> {
    let tokens = tokenize(input)?;
    let mut index = 0;
    let entries = parse_entries(&tokens, &mut index, false)?;
    if index != tokens.len() {
        return Err("VDF 含有未解析内容。".into());
    }
    Ok(KeyValue::Object(entries))
}

fn parse_entries(
    tokens: &[String],
    index: &mut usize,
    nested: bool,
) -> Result<Vec<(String, KeyValue)>, String> {
    let mut entries = Vec::new();
    while *index < tokens.len() {
        if tokens[*index] == "}" {
            if !nested {
                return Err("VDF 出现多余右括号。".into());
            }
            *index += 1;
            return Ok(entries);
        }
        let key = tokens[*index].clone();
        *index += 1;
        let value = tokens.get(*index).ok_or("VDF 键缺少值。")?;
        if value == "{" {
            *index += 1;
            entries.push((key, KeyValue::Object(parse_entries(tokens, index, true)?)));
        } else if value == "}" {
            return Err("VDF 键缺少值。".into());
        } else {
            entries.push((key, KeyValue::String(value.clone())));
            *index += 1;
        }
    }
    if nested {
        Err("VDF 对象未闭合。".into())
    } else {
        Ok(entries)
    }
}

fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            character if character.is_whitespace() => {}
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                while chars.next().is_some_and(|next| next != '\n') {}
            }
            '{' | '}' => tokens.push(character.to_string()),
            '"' => {
                let mut value = String::new();
                let mut closed = false;
                while let Some(next) = chars.next() {
                    if next == '"' {
                        closed = true;
                        break;
                    }
                    if next == '\\' {
                        match chars.peek().copied() {
                            Some('"') | Some('\\') => value.push(chars.next().unwrap_or(next)),
                            _ => value.push(next),
                        }
                    } else {
                        value.push(next);
                    }
                }
                if !closed {
                    return Err("VDF 引号未闭合。".into());
                }
                tokens.push(value);
            }
            _ => {
                let mut value = character.to_string();
                while let Some(next) = chars.peek().copied() {
                    if next.is_whitespace() || matches!(next, '{' | '}') {
                        break;
                    }
                    value.push(next);
                    chars.next();
                }
                tokens.push(value);
            }
        }
    }
    Ok(tokens)
}

fn library_paths(root: &KeyValue) -> Vec<PathBuf> {
    let Some(folders) = object_child(root, "libraryfolders") else {
        return Vec::new();
    };
    let KeyValue::Object(entries) = folders else {
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(|(_, value)| match value {
            KeyValue::String(path) => Some(PathBuf::from(path)),
            KeyValue::Object(_) => first_string(value, "path").map(PathBuf::from),
        })
        .collect()
}

fn object_child<'a>(value: &'a KeyValue, key: &str) -> Option<&'a KeyValue> {
    let KeyValue::Object(entries) = value else {
        return None;
    };
    entries
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(key))
        .map(|(_, value)| value)
}

fn first_string<'a>(value: &'a KeyValue, key: &str) -> Option<&'a str> {
    let KeyValue::Object(entries) = value else {
        return None;
    };
    for (name, child) in entries {
        if name.eq_ignore_ascii_case(key) {
            if let KeyValue::String(value) = child {
                return Some(value);
            }
        }
        if let Some(value) = first_string(child, key) {
            return Some(value);
        }
    }
    None
}

fn dedupe_paths(paths: &mut Vec<PathBuf>) {
    let mut seen = HashSet::new();
    paths.retain(|path| seen.insert(canonical_key(path)));
}

fn dedupe_probes(probes: &mut Vec<Probe>) {
    let mut by_path = HashMap::<String, usize>::new();
    let mut merged: Vec<Probe> = Vec::new();
    for probe in probes.drain(..) {
        let key = canonical_key(&probe.path);
        if let Some(index) = by_path.get(&key).copied() {
            let existing = &mut merged[index];
            existing.manifest |= probe.manifest;
            if probe.priority < existing.priority {
                existing.source = probe.source;
                existing.priority = probe.priority;
            }
        } else {
            by_path.insert(key, merged.len());
            merged.push(probe);
        }
    }
    *probes = merged;
}

fn canonical_key(path: &Path) -> String {
    let canonical = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    canonical
        .to_string_lossy()
        .replace('/', "\\")
        .to_lowercase()
}

fn confidence_rank(confidence: &str) -> u8 {
    if confidence == "verified" {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str, executable: bool) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("cs2-discovery-{name}-{unique}"));
        fs::create_dir_all(root.join("game/csgo")).unwrap();
        fs::write(root.join("game/csgo/gameinfo.gi"), "test").unwrap();
        if executable {
            fs::create_dir_all(root.join("game/bin/win64")).unwrap();
            fs::write(root.join("game/bin/win64/cs2.exe"), "test").unwrap();
        }
        root
    }

    fn probe(path: PathBuf) -> Probe {
        Probe {
            path,
            source: "测试".into(),
            manifest: false,
            priority: 2,
        }
    }

    #[test]
    fn parses_old_and_new_libraryfolders_and_manifest() {
        let old =
            parse_key_values(r#""libraryfolders" { "0" "D:\\SteamLibrary" "1" "E:\\Steam" }"#)
                .unwrap();
        assert_eq!(
            library_paths(&old),
            vec![
                PathBuf::from(r"D:\SteamLibrary"),
                PathBuf::from(r"E:\Steam")
            ]
        );
        let new = parse_key_values(
            r#""libraryfolders" { "0" { "path" "F:\\Games" "apps" { "730" "1" } } }"#,
        )
        .unwrap();
        assert_eq!(library_paths(&new), vec![PathBuf::from(r"F:\Games")]);
        let manifest = parse_key_values(
            r#""AppState" { "appid" "730" "installdir" "Counter-Strike Global Offensive" }"#,
        )
        .unwrap();
        assert_eq!(first_string(&manifest, "installdir"), Some(CS2_FOLDER_NAME));
    }

    #[test]
    fn stops_after_three_unique_candidates_and_sorts_verified_first() {
        let roots = [
            temp_root("one", false),
            temp_root("two", true),
            temp_root("three", false),
        ];
        let summary = scan_with(
            &AtomicBool::new(false),
            Duration::from_millis(100),
            3,
            roots.iter().cloned().map(probe).collect(),
            |_| {},
        );
        assert_eq!(summary.stop_reason, "threeFound");
        assert_eq!(summary.candidates.len(), 3);
        assert_eq!(summary.candidates[0].confidence, "verified");
        for root in roots {
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn waits_for_timeout_when_sources_are_exhausted() {
        let summary = scan_with(
            &AtomicBool::new(false),
            Duration::from_millis(35),
            3,
            Vec::new(),
            |_| {},
        );
        assert_eq!(summary.stop_reason, "timeout");
        assert!(summary.elapsed_ms >= 35);
    }

    #[test]
    fn honors_cancellation_and_rejects_parallel_scan() {
        let token = AtomicBool::new(true);
        let summary = scan_with(&token, Duration::from_secs(1), 3, Vec::new(), |_| {});
        assert_eq!(summary.stop_reason, "userStopped");
        let coordinator = ScanCoordinator::default();
        let _first = coordinator.start().unwrap();
        let error = coordinator.start().unwrap_err().into_string();
        assert!(error.contains("CS2_SCAN_ALREADY_RUNNING"));
        assert!(coordinator.stop());
        coordinator.finish();
    }

    #[test]
    fn canonical_keys_dedupe_case_and_separator_variants() {
        assert_eq!(
            canonical_key(Path::new(r"D:\Steam\CS2")),
            canonical_key(Path::new("d:/steam/cs2"))
        );
    }

    #[test]
    fn scan_events_serialize_with_the_frontend_contract() {
        let event = Cs2RootScanEvent::Candidate {
            elapsed_ms: 12,
            checked_locations: 3,
            candidate: Cs2SuggestedRoot {
                path: r"D:\Steam\CS2".into(),
                source: "测试".into(),
                confidence: "verified".into(),
                evidence: Vec::new(),
            },
        };
        let value = serde_json::to_value(event).unwrap();
        assert_eq!(value["kind"], "candidate");
        assert_eq!(value["elapsedMs"], 12);
        assert_eq!(value["checkedLocations"], 3);
        assert!(value.get("checked_locations").is_none());
    }
}
