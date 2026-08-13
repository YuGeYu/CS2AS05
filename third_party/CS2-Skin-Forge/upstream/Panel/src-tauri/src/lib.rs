use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

// NOTE: Loadout JSON is passed through save_loadout/load_loadout untyped
// (serde_json::Value). The Rust layer only persists the file — the data
// model lives in the frontend (utils/types.ts) and the plugin
// (Models/PlayerLoadout.cs). Keeping Rust untyped means new loadout fields
// never require a Rust change and partial/legacy files never fail to load.

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: Option<String>,
    #[serde(rename = "cs2Path")]
    pub cs2_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCheckResult {
    pub all_present: bool,
    pub missing_files: Vec<String>,
    pub version_mismatch: bool,
    pub deployed_version: Option<String>,
    pub panel_version: String,
    pub counterstrikesharp_installed: bool,
}

/// Read the CS2 path from config and return the PlayerSkinMod plugin directory.
fn get_plugin_dir_from_config() -> Result<PathBuf, String> {
    let config_path = get_config_path();
    if !config_path.exists() {
        return Err("CS2 path not configured. Please set it in Settings.".to_string());
    }
    let data = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    let cs2_path = config
        .cs2_path
        .ok_or_else(|| "CS2 path not configured.".to_string())?;
    Ok(PathBuf::from(&cs2_path)
        .join("addons")
        .join("counterstrikesharp")
        .join("plugins")
        .join("PlayerSkinMod"))
}

const PLUGIN_FILES: &[&str] = &["PlayerSkinMod.dll", "PlayerSkinMod.json", "skins_en.json"];
const VERSION_FILE: &str = "plugin_version.txt";

// Embedded plugin files — always available regardless of Tauri resource bundling.
// build.rs creates a placeholder DLL if the real one hasn't been built yet.
const EMBEDDED_DLL: &[u8] =
    include_bytes!("../../../addons/counterstrikesharp/plugins/PlayerSkinMod/PlayerSkinMod.dll");
const EMBEDDED_MANIFEST: &[u8] =
    include_bytes!("../../../addons/counterstrikesharp/plugins/PlayerSkinMod/PlayerSkinMod.json");
const EMBEDDED_SKINS: &[u8] =
    include_bytes!("../../../addons/counterstrikesharp/plugins/PlayerSkinMod/skins_en.json");

fn get_embedded_bytes(filename: &str) -> Option<&'static [u8]> {
    match filename {
        "PlayerSkinMod.dll" => Some(EMBEDDED_DLL),
        "PlayerSkinMod.json" => Some(EMBEDDED_MANIFEST),
        "skins_en.json" => Some(EMBEDDED_SKINS),
        _ => None,
    }
}

/// Search for a source file in the resource directory, trying multiple layout variants.
fn find_source_in_resources(resource_dir: &Path, filename: &str) -> Option<PathBuf> {
    // Layout variants that different Tauri / platform bundlers produce:
    let candidates: &[&[&str]] = &[
        // Tauri 2 NSIS with directory structure preserved:
        &["addons", "counterstrikesharp", "plugins", "PlayerSkinMod"],
        // Tauri 2 flat resources (individual files):
        &[],
        // Tauri 2 resources with only the leaf directory name:
        &["PlayerSkinMod"],
        // Tauri < 2 / MSI style:
        &[
            "data",
            "resources",
            "addons",
            "counterstrikesharp",
            "plugins",
            "PlayerSkinMod",
        ],
        // Top-level addons layout (some custom setups):
        &["addons"],
    ];

    for parts in candidates {
        let mut p = resource_dir.to_path_buf();
        for part in *parts {
            p.push(part);
        }
        p.push(filename);
        if p.exists() && fs::metadata(&p).map(|m| m.len()).unwrap_or(0) > 0 {
            return Some(p);
        }
    }
    None
}

/// Check whether CounterStrikeSharp is installed at the given CS2 path.
fn check_counterstrikesharp(cs2_path: &Path) -> bool {
    let api_dir = cs2_path
        .join("addons")
        .join("counterstrikesharp")
        .join("api");
    if !api_dir.exists() {
        return false;
    }
    if cfg!(target_os = "windows") {
        api_dir.join("CounterStrikeSharp.API.dll").exists()
    } else {
        api_dir.join("CounterStrikeSharp.API.so").exists()
    }
}

/// Deploy CounterStrikeSharp automatically — downloads the latest release from GitHub
/// and extracts it into the CS2 game directory.
fn deploy_counterstrikesharp(cs2_path: &Path) -> Result<String, String> {
    let platform_zip = if cfg!(target_os = "windows") {
        "counterstrikesharp-windows"
    } else {
        "counterstrikesharp-linux"
    };

    // 1. Fetch the latest release tag from GitHub API.
    let api_url = "https://api.github.com/repos/roflmuffin/CounterStrikeSharp/releases/latest";
    let resp = ureq::get(api_url)
        .set("User-Agent", "CS2-Skin-Forge/auto-deploy")
        .call()
        .map_err(|e| format!("Failed to query GitHub API: {}", e))?;
    let json: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Failed to parse API response: {}", e))?;
    let tag = json["tag_name"]
        .as_str()
        .ok_or_else(|| "Missing tag_name in release".to_string())?;
    let version = tag.strip_prefix('v').unwrap_or(tag);

    // 2. Build download URL.
    let dl_url = format!(
        "https://github.com/roflmuffin/CounterStrikeSharp/releases/download/{tag}/{platform_zip}-{version}.zip"
    );

    // 3. Download the zip into memory.
    let resp = ureq::get(&dl_url)
        .set("User-Agent", "CS2-Skin-Forge/auto-deploy")
        .call()
        .map_err(|e| format!("Failed to download CounterStrikeSharp: {}", e))?;
    let mut zip_bytes: Vec<u8> = Vec::new();
    resp.into_reader()
        .read_to_end(&mut zip_bytes)
        .map_err(|e| format!("Failed to read download: {}", e))?;
    if zip_bytes.is_empty() {
        return Err("Downloaded zip is empty.".to_string());
    }

    // 4. Extract the zip to the CS2 path.
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("Failed to open zip: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry {i}: {e}"))?;
        let Some(out_path) = file.enclosed_name() else {
            continue;
        };
        let dest = cs2_path.join(&out_path);
        if file.is_dir() {
            fs::create_dir_all(&dest).ok();
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut outfile = fs::File::create(&dest)
                .map_err(|e| format!("Failed to create {}: {}", dest.display(), e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract {}: {}", dest.display(), e))?;
        }
    }

    Ok(format!(
        "CounterStrikeSharp {} installed successfully.",
        tag
    ))
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("CS2-Skin-Mod");
    fs::create_dir_all(&path).ok();
    path.push("config.json");
    path
}

fn get_loadout_path_from_config() -> Option<PathBuf> {
    let config_path = get_config_path();
    if !config_path.exists() {
        return None;
    }
    let data = fs::read_to_string(&config_path).ok()?;
    let config: AppConfig = serde_json::from_str(&data).ok()?;
    let cs2_path = config.cs2_path?;
    let mut path = PathBuf::from(cs2_path);
    path.push("addons");
    path.push("counterstrikesharp");
    path.push("plugins");
    path.push("PlayerSkinMod");
    path.push("player_loadout.json");
    Some(path)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_url: String,
    pub release_notes: String,
}

/// Compare two dotted version strings (e.g. "1.5.16" vs "1.6.0").
fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v')
            .split('.')
            .map(|p| {
                p.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u64>()
                    .unwrap_or(0)
            })
            .collect()
    };
    let l = parse(latest);
    let c = parse(current);
    for i in 0..l.len().max(c.len()) {
        let lv = l.get(i).copied().unwrap_or(0);
        let cv = c.get(i).copied().unwrap_or(0);
        if lv != cv {
            return lv > cv;
        }
    }
    false
}

#[tauri::command]
fn check_update() -> Result<UpdateCheckResult, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let api_url = "https://api.github.com/repos/kaecho/CS2-Skin-Forge/releases/latest";
    let resp = ureq::get(api_url)
        .set("User-Agent", "CS2-Skin-Forge/update-check")
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| format!("Failed to query GitHub API: {}", e))?;
    let json: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Failed to parse API response: {}", e))?;
    let tag = json["tag_name"]
        .as_str()
        .ok_or_else(|| "Missing tag_name in release".to_string())?;
    let latest_version = tag.trim_start_matches('v').to_string();
    let release_url = json["html_url"]
        .as_str()
        .unwrap_or("https://github.com/kaecho/CS2-Skin-Forge/releases/latest")
        .to_string();
    let release_notes = json["body"].as_str().unwrap_or("").to_string();
    let update_available = is_newer_version(&latest_version, &current_version);

    Ok(UpdateCheckResult {
        current_version,
        latest_version,
        update_available,
        release_url,
        release_notes,
    })
}

#[tauri::command]
fn get_config() -> Result<AppConfig, String> {
    let path = get_config_path();
    if !path.exists() {
        return Ok(AppConfig {
            language: Some("english".to_string()),
            cs2_path: None,
        });
    }
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
fn save_config(config: AppConfig) -> Result<(), String> {
    let path = get_config_path();
    let data = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn save_loadout(slot: u32, loadout: serde_json::Value) -> Result<(), String> {
    let loadout_path = get_loadout_path_from_config()
        .ok_or_else(|| "CS2 path not configured. Please set it in Settings.".to_string())?;

    // Read existing loadouts to preserve other slots; tolerate a missing or
    // corrupt file by starting fresh.
    let mut loadouts: serde_json::Map<String, serde_json::Value> = if loadout_path.exists() {
        fs::read_to_string(&loadout_path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    } else {
        Default::default()
    };

    loadouts.insert(slot.to_string(), loadout);

    // Ensure the directory exists
    if let Some(parent) = loadout_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let data = serde_json::to_string_pretty(&loadouts).map_err(|e| e.to_string())?;
    fs::write(&loadout_path, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_loadout(slot: u32) -> Result<Option<serde_json::Value>, String> {
    let loadout_path = match get_loadout_path_from_config() {
        Some(p) => p,
        None => return Ok(None),
    };

    if !loadout_path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&loadout_path).map_err(|e| e.to_string())?;
    let loadouts: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(loadouts.get(&slot.to_string()).cloned())
}

/// Default Steam installation roots per platform.
fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if cfg!(target_os = "windows") {
        for drive in ["C", "D", "E", "F"] {
            roots.push(PathBuf::from(format!("{drive}:\\Program Files (x86)\\Steam")));
            roots.push(PathBuf::from(format!("{drive}:\\Steam")));
            roots.push(PathBuf::from(format!("{drive}:\\SteamLibrary")));
        }
    } else if cfg!(target_os = "macos") {
        if let Some(home) = dirs::home_dir() {
            roots.push(home.join("Library/Application Support/Steam"));
        }
    } else {
        if let Some(home) = dirs::home_dir() {
            roots.push(home.join(".steam/steam"));
            roots.push(home.join(".local/share/Steam"));
            // Flatpak Steam
            roots.push(home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"));
        }
    }
    roots
}

/// Extract additional library paths from Steam's libraryfolders.vdf
/// ("path" entries). A naive line scan is enough for this format.
fn parse_library_folders(vdf_path: &Path) -> Vec<PathBuf> {
    let Ok(content) = fs::read_to_string(vdf_path) else {
        return Vec::new();
    };
    let mut libs = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("\"path\"") {
            let value = rest.trim().trim_matches('"');
            if !value.is_empty() {
                libs.push(PathBuf::from(value.replace("\\\\", "\\")));
            }
        }
    }
    libs
}

#[tauri::command]
fn detect_cs2_path() -> Result<Option<String>, String> {
    // Collect candidate Steam libraries: the default roots plus every extra
    // library registered in each root's libraryfolders.vdf.
    let mut libraries: Vec<PathBuf> = Vec::new();
    for root in steam_roots() {
        for lib in parse_library_folders(&root.join("config").join("libraryfolders.vdf")) {
            libraries.push(lib);
        }
        libraries.push(root);
    }

    for lib in libraries {
        let cs2 = lib
            .join("steamapps")
            .join("common")
            .join("Counter-Strike Global Offensive")
            .join("game")
            .join("csgo");
        if cs2.exists() {
            return Ok(Some(cs2.to_string_lossy().to_string()));
        }
    }

    Ok(None)
}

#[tauri::command]
fn check_plugin_files() -> Result<PluginCheckResult, String> {
    let panel_version = env!("CARGO_PKG_VERSION").to_string();
    let plugin_dir = match get_plugin_dir_from_config() {
        Ok(dir) => dir,
        Err(_) => {
            return Ok(PluginCheckResult {
                all_present: false,
                missing_files: PLUGIN_FILES.iter().map(|s| s.to_string()).collect(),
                version_mismatch: false,
                deployed_version: None,
                panel_version,
                counterstrikesharp_installed: false,
            });
        }
    };

    // Figure out CS2 path from plugin dir (go up from plugins/PlayerSkinMod to game dir)
    let cs2_path = plugin_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new(""));
    let css_installed = check_counterstrikesharp(cs2_path);

    // Check each required plugin file
    let mut missing_files: Vec<String> = Vec::new();
    for file in PLUGIN_FILES {
        let path = plugin_dir.join(file);
        if !path.exists() || fs::metadata(&path).map(|m| m.len()).unwrap_or(0) == 0 {
            missing_files.push(file.to_string());
        }
    }

    // Check version file
    let version_path = plugin_dir.join(VERSION_FILE);
    let deployed_version = if version_path.exists() {
        fs::read_to_string(&version_path)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    let version_mismatch = deployed_version
        .as_ref()
        .map_or(false, |v| v != &panel_version);

    Ok(PluginCheckResult {
        all_present: missing_files.is_empty() && !version_mismatch,
        missing_files,
        version_mismatch,
        deployed_version,
        panel_version,
        counterstrikesharp_installed: css_installed,
    })
}

#[tauri::command]
fn deploy_addons(app: tauri::AppHandle) -> Result<String, String> {
    let target_dir = get_plugin_dir_from_config()?;
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    // Determine CS2 game directory (go up from .../plugins/PlayerSkinMod to game dir)
    let cs2_path = target_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or_else(|| "Cannot determine CS2 game directory".to_string())?;

    // Auto-deploy CounterStrikeSharp if missing
    let mut css_result = String::new();
    if !check_counterstrikesharp(cs2_path) {
        match deploy_counterstrikesharp(cs2_path) {
            Ok(msg) => css_result = format!(" | CounterStrikeSharp: {}", msg),
            Err(e) => css_result = format!(" | CounterStrikeSharp: download failed ({})", e),
        }
    }

    // Primary: use Tauri's official resource_dir API (works for all bundle types)
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Cannot resolve resource directory: {}", e))?;

    let mut deployed = Vec::new();
    let mut skipped = Vec::new();
    let mut verify_failed = Vec::new();
    let mut used_embedded = false;

    for file in PLUGIN_FILES {
        // 1) Try to find the file in the resource directory
        let src = find_source_in_resources(&resource_dir, file);

        let src_size = src
            .as_ref()
            .and_then(|p| fs::metadata(p).ok())
            .map(|m| m.len())
            .unwrap_or(0);

        if src.is_some() && src_size > 0 {
            let src_path = src.unwrap();
            let dst = target_dir.join(file);
            let dst_size = if dst.exists() {
                fs::metadata(&dst).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };
            fs::copy(&src_path, &dst).map_err(|e| format!("Failed to copy {}: {}", file, e))?;

            match fs::metadata(&dst) {
                Ok(meta) if meta.len() > 0 => {
                    let msg = if dst_size > 0 {
                        format!("{} ({}B -> {}B, bundled)", file, dst_size, src_size)
                    } else {
                        format!("{} ({}B, bundled)", file, src_size)
                    };
                    deployed.push(msg);
                }
                _ => {
                    verify_failed.push(format!(
                        "{} (copy succeeded but destination is missing/empty)",
                        file
                    ));
                }
            }
        } else if let Some(embedded) = get_embedded_bytes(file) {
            // 2) Fallback: write from embedded bytes (always available)
            if embedded.is_empty() {
                skipped.push(format!(
                    "{file} (embedded is a placeholder - real DLL missing)"
                ));
                continue;
            }
            let dst = target_dir.join(file);
            let dst_size = if dst.exists() {
                fs::metadata(&dst).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };
            fs::write(&dst, embedded)
                .map_err(|e| format!("Failed to write {} from embedded: {}", file, e))?;

            match fs::metadata(&dst) {
                Ok(meta) if meta.len() > 0 => {
                    let msg = if dst_size > 0 {
                        format!("{} ({}B -> {}B, embedded)", file, dst_size, embedded.len())
                    } else {
                        format!("{} ({}B, embedded)", file, embedded.len())
                    };
                    deployed.push(msg);
                    used_embedded = true;
                }
                _ => {
                    verify_failed.push(format!(
                        "{} (embedded write succeeded but destination is missing/empty)",
                        file
                    ));
                }
            }
        } else {
            skipped.push(file.to_string());
        }
    }

    if deployed.is_empty() && verify_failed.is_empty() {
        return Err(format!(
            "No plugin files found to deploy.\n\
             Resource dir: {}\n\
             Searched layouts: [addons/.../PlayerSkinMod, flat, PlayerSkinMod/, data/resources/...]",
            resource_dir.display()
        ));
    }

    let mut result = format!(
        "Deployed [{}] to {}",
        deployed.join(", "),
        target_dir.display()
    );

    if used_embedded {
        result.push_str(" | (from embedded)");
    }
    if !verify_failed.is_empty() {
        result.push_str(&format!(" | Verify-failed: [{}]", verify_failed.join(", ")));
    }
    if !skipped.is_empty() {
        result.push_str(&format!(" | Skipped: [{}]", skipped.join(", ")));
    }
    if !css_result.is_empty() {
        result.push_str(&css_result);
    }

    // Write version file so we can detect panel-plugin version mismatch
    let version = env!("CARGO_PKG_VERSION");
    let version_path = target_dir.join(VERSION_FILE);
    fs::write(&version_path, version)
        .map_err(|e| format!("Failed to write version file: {}", e))?;
    result.push_str(&format!(" | Version: {}", version));

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            save_loadout,
            load_loadout,
            detect_cs2_path,
            check_plugin_files,
            deploy_addons,
            check_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
