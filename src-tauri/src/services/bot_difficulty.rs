use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

use crate::errors::AppError;
use crate::models::bot_difficulty::{
    BotProfileDocument, BotProfileList, BotProfileOperation, BotProfileSummary, BotToolState,
    CreateBotProfileRequest, RenameBotProfileRequest, SaveBotProfileRequest, VpkEntry,
};

const TOOL_VERSION: &str = "VPKEdit CLI v5.0.0.4";
const TOOL_SHA256: &str = "df354e590d157abd633b4a047591363f17e066486e0f59184ff71c51a81b582a";
const EXTRACT_MAX_ATTEMPTS: u32 = 3;
const STALE_WORKSPACE_AGE: Duration = Duration::from_secs(48 * 60 * 60);
static TOOL_STATE_CACHE: OnceLock<Mutex<Option<(PathBuf, BotToolState)>>> = OnceLock::new();
static EXTRACT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn data_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_local_data_dir()
        .map(|path| path.join("bot-difficulty-workshop"))
        .map_err(|error| AppError::runtime(format!("[BOT_WORKSHOP_DATA_DIR] {error}")))
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn tool_candidates(app: &AppHandle) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(resource) = app.path().resource_dir() {
        paths.push(resource.join("vpkeditcli-x86_64-pc-windows-msvc.exe"));
        paths.push(resource.join("vpkeditcli.exe"));
    }
    paths
}

fn run_cli(exe: &Path, args: &[&str], cwd: &Path) -> Result<(String, String, i32), AppError> {
    let mut child = Command::new(exe)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::runtime(format!("[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] {e}")))?;
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(30) {
        if child
            .try_wait()
            .map_err(|e| AppError::runtime(e.to_string()))?
            .is_some()
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    if child
        .try_wait()
        .map_err(|e| AppError::runtime(e.to_string()))?
        .is_none()
    {
        let _ = child.kill();
        let _ = child.wait();
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_TOOL_TIMEOUT] VPKEdit CLI 超时",
        ));
    }
    let output = child
        .wait_with_output()
        .map_err(|e| AppError::runtime(e.to_string()))?;
    let stdout =
        String::from_utf8_lossy(&output.stdout[..output.stdout.len().min(64 * 1024)]).into_owned();
    let stderr =
        String::from_utf8_lossy(&output.stderr[..output.stderr.len().min(64 * 1024)]).into_owned();
    Ok((stdout, stderr, output.status.code().unwrap_or(-1)))
}

pub fn inspect_vpk_tool(app: &AppHandle) -> Result<BotToolState, AppError> {
    let path = tool_candidates(app).into_iter().find(|p| p.is_file());
    let Some(path) = path else {
        return Ok(BotToolState {
            status: "missing".into(),
            version: TOOL_VERSION.into(),
            sha256: None,
            path_hint: None,
            detail: Some("未找到 VPKEdit CLI".into()),
        });
    };
    let cache = TOOL_STATE_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = cache.lock() {
        if let Some((cached_path, state)) = guard.as_ref() {
            if cached_path == &path && state.status == "ready" {
                return Ok(state.clone());
            }
        }
    }
    let bytes = fs::read(&path)
        .map_err(|e| AppError::runtime(format!("[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] {e}")))?;
    let digest = hash(&bytes);
    if !digest.eq_ignore_ascii_case(TOOL_SHA256) {
        return Ok(BotToolState {
            status: "invalid".into(),
            version: TOOL_VERSION.into(),
            sha256: Some(digest),
            path_hint: Some(path.display().to_string()),
            detail: Some("EXE SHA-256 不匹配".into()),
        });
    }
    let cwd = data_dir(app)?.join("workspace").join("tool-check");
    fs::create_dir_all(&cwd).map_err(|e| AppError::runtime(e.to_string()))?;
    let (_, _, code) = run_cli(&path, &["--help"], &cwd)?;
    let status = if code == 0 { "ready" } else { "invalid" };
    let state = BotToolState {
        status: status.into(),
        version: TOOL_VERSION.into(),
        sha256: Some(digest),
        path_hint: Some(path.display().to_string()),
        detail: (code != 0).then(|| format!("--help 退出码 {code}")),
    };
    if let Ok(mut guard) = cache.lock() {
        *guard = Some((path, state.clone()));
    }
    Ok(state)
}

fn validate_entry_path(path: &str) -> bool {
    let p = Path::new(path);
    !p.is_absolute() && !path.split(['/', '\\']).any(|part| part == "..") && !path.is_empty()
}

pub fn list_vpk_entries(app: &AppHandle, input_path: &str) -> Result<Vec<VpkEntry>, AppError> {
    let input = Path::new(input_path);
    if !input.is_file() {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_PROFILE_NOT_FOUND] VPK 不存在",
        ));
    }
    let tool = tool_candidates(app)
        .into_iter()
        .find(|p| p.is_file())
        .ok_or_else(|| {
            AppError::runtime("[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] 未找到 VPKEdit CLI")
        })?;
    let cwd = data_dir(app)?
        .join("workspace")
        .join(format!("tree-{}", std::process::id()));
    fs::create_dir_all(&cwd).map_err(|e| AppError::runtime(e.to_string()))?;
    let (stdout, _, code) = run_cli(
        &tool,
        &[
            "--file-tree",
            "--no-progress",
            input.to_string_lossy().as_ref(),
        ],
        &cwd,
    )?;
    if code != 0 {
        return Err(AppError::runtime(format!(
            "[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] file-tree 退出码 {code}"
        )));
    }
    let mut entries = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("VPK")
            || trimmed.starts_with("File")
            || trimmed.starts_with('-')
        {
            continue;
        }
        let path = trimmed.split_whitespace().last().unwrap_or("");
        if !validate_entry_path(path) || path.ends_with('/') {
            continue;
        }
        entries.push(VpkEntry {
            path: path.replace('\\', "/"),
            size: 0,
            sha256: None,
        });
    }
    entries.sort_by(|a, b| {
        a.path
            .to_ascii_lowercase()
            .cmp(&b.path.to_ascii_lowercase())
    });
    if entries
        .iter()
        .filter(|e| e.path.eq_ignore_ascii_case("botprofile.db"))
        .count()
        != 1
    {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_ENTRY_MANIFEST_MISMATCH] botprofile.db 条目不唯一",
        ));
    }
    Ok(entries)
}

fn cleanup_stale_workspace(app: &AppHandle) {
    let Ok(workspace) = data_dir(app).map(|dir| dir.join("workspace")) else {
        return;
    };
    let Ok(entries) = fs::read_dir(&workspace) else {
        return;
    };
    let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let managed = name.starts_with("open-")
            || name.starts_with("tree-")
            || name.starts_with("extract-")
            || name.starts_with("verify-")
            || name.starts_with("manifest")
            || name.starts_with("tool-check");
        if !managed {
            continue;
        }
        let stale = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|age| now.saturating_sub(age) > STALE_WORKSPACE_AGE)
            .unwrap_or(false);
        if stale {
            let _ = fs::remove_dir_all(&path);
        }
    }
}

pub fn extract_db(
    app: &AppHandle,
    input_path: &str,
    workspace: &Path,
) -> Result<PathBuf, AppError> {
    let input = Path::new(input_path);
    let tool = tool_candidates(app)
        .into_iter()
        .find(|p| p.is_file())
        .ok_or_else(|| {
            AppError::runtime("[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] 未找到 VPKEdit CLI")
        })?;
    let _guard = EXTRACT_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| AppError::runtime("[BOT_WORKSHOP_EXTRACT_BUSY] 提取任务状态异常"))?;
    cleanup_stale_workspace(app);
    let mut last_stderr = String::new();
    let mut last_code = 0;
    for attempt in 0..EXTRACT_MAX_ATTEMPTS {
        let unique_workspace = workspace.join(format!(
            "extract-{}-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(),
            attempt
        ));
        fs::create_dir_all(&unique_workspace).map_err(|e| {
            AppError::runtime(format!(
                "[BOT_WORKSHOP_WORKSPACE_UNWRITABLE] 无法创建提取目录 {}: {e}",
                unique_workspace.display()
            ))
        })?;
        let probe = unique_workspace.join(".write-probe");
        if let Err(e) = fs::write(&probe, b"ok") {
            return Err(AppError::runtime(format!(
                "[BOT_WORKSHOP_WORKSPACE_UNWRITABLE] 无法写入提取目录 {}: {e}；请检查磁盘空间、目录权限或安全软件（如受控文件夹访问）设置。",
                unique_workspace.display()
            )));
        }
        let _ = fs::remove_file(&probe);
        let output = unique_workspace.join("botprofile.db");
        let output_arg = output.to_string_lossy().to_string();
        let input_arg = input.to_string_lossy().to_string();
        let (stdout, stderr, code) = run_cli(
            &tool,
            &[
                "--extract",
                "botprofile.db",
                "--output",
                &output_arg,
                "--no-progress",
                &input_arg,
            ],
            &unique_workspace,
        )?;
        if code != 0 {
            last_stderr = stderr.trim().to_string();
            last_code = code;
            if attempt + 1 < EXTRACT_MAX_ATTEMPTS {
                std::thread::sleep(Duration::from_millis(400 * (attempt as u64 + 1)));
                continue;
            }
            return Err(AppError::runtime(format!(
                "[BOT_WORKSHOP_EXTRACT_FAILED] VPKEdit 提取失败（退出码 {last_code}，已尝试 {EXTRACT_MAX_ATTEMPTS} 次）：{last_stderr}；请确认 botprofile.vpk 未被其他程序占用，并检查安全软件是否拦截了文件写入。",
            )));
        }
        if !output.is_file() {
            return Err(AppError::runtime(format!(
                "[BOT_WORKSHOP_ENTRY_MANIFEST_MISMATCH] botprofile.db 未成功提取: {}",
                stderr.trim()
            )));
        }
        let _ = stdout;
        return Ok(output);
    }
    Err(AppError::runtime(format!(
        "[BOT_WORKSHOP_EXTRACT_FAILED] VPKEdit 提取失败（退出码 {last_code}）：{last_stderr}"
    )))
}

fn walkdir(root: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut out = Vec::new();
    if root.is_dir() {
        for item in fs::read_dir(root)
            .map_err(|e| AppError::runtime(e.to_string()))?
            .flatten()
        {
            let p = item.path();
            if p.is_dir() {
                out.extend(walkdir(&p)?);
            } else {
                out.push(p);
            }
        }
    }
    Ok(out)
}

pub fn entry_manifest(
    app: &AppHandle,
    input_path: &str,
    workspace: &Path,
) -> Result<Vec<VpkEntry>, AppError> {
    let output = workspace.join("manifest");
    fs::create_dir_all(&output).map_err(|e| AppError::runtime(e.to_string()))?;
    let tool = tool_candidates(app)
        .into_iter()
        .find(|p| p.is_file())
        .ok_or_else(|| {
            AppError::runtime("[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] 未找到 VPKEdit CLI")
        })?;
    let output_arg = output.to_string_lossy().to_string();
    let input_arg = Path::new(input_path).to_string_lossy().to_string();
    let (_, stderr, code) = run_cli(
        &tool,
        &[
            "--extract",
            "/",
            "--output",
            &output_arg,
            "--no-progress",
            &input_arg,
        ],
        workspace,
    )?;
    if code != 0 {
        return Err(AppError::runtime(format!(
            "[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] manifest 提取失败: {}",
            stderr.trim()
        )));
    }
    let mut entries = Vec::new();
    for path in walkdir(&output)? {
        let rel = path
            .strip_prefix(&output)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if !validate_entry_path(&rel) {
            continue;
        }
        let bytes = fs::read(&path).map_err(|e| AppError::runtime(e.to_string()))?;
        entries.push(VpkEntry {
            path: rel,
            size: bytes.len() as u64,
            sha256: Some(hash(&bytes)),
        });
    }
    entries.sort_by(|a, b| {
        a.path
            .to_ascii_lowercase()
            .cmp(&b.path.to_ascii_lowercase())
    });
    Ok(entries)
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() < 80
        && id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ['-', '_'].contains(&character))
}

fn valid_profile_name(name: &str) -> bool {
    let trimmed = name.trim();
    !trimmed.is_empty()
        && trimmed.chars().count() <= 48
        && !trimmed.chars().any(|character| {
            character.is_control()
                || ['\\', '/', ':', '*', '?', '"', '<', '>', '|'].contains(&character)
        })
        && trimmed != "."
        && trimmed != ".."
}

fn read_custom_profile(
    app: &AppHandle,
    profile_id: &str,
) -> Result<(PathBuf, BotProfileSummary), AppError> {
    if !profile_id.starts_with("custom-") || !valid_id(profile_id) {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_PROFILE_NOT_FOUND] 非法自定义档案 ID",
        ));
    }
    let dir = profile_dir(app, profile_id)?;
    let json = fs::read_to_string(dir.join("profile.json"))
        .map_err(|_| AppError::runtime("[BOT_WORKSHOP_PROFILE_NOT_FOUND] 自定义档案不存在"))?;
    let profile: BotProfileSummary = serde_json::from_str(&json)
        .map_err(|_| AppError::runtime("[BOT_WORKSHOP_PROFILE_CORRUPT] 档案元数据损坏"))?;
    if profile.source != "custom" || profile.read_only {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_READ_ONLY] 内置档案不可管理",
        ));
    }
    let metadata = fs::symlink_metadata(&dir)
        .map_err(|_| AppError::runtime("[BOT_WORKSHOP_PROFILE_NOT_FOUND] 自定义档案不存在"))?;
    if metadata.file_type().is_symlink() {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_PROFILE_INVALID] 不支持链接档案目录",
        ));
    }
    Ok((dir, profile))
}

pub fn list(app: &AppHandle, root_path: &str) -> Result<BotProfileList, AppError> {
    let root = crate::services::cs2::normalize_root(root_path)?;
    let active = fs::read(root.join("game/csgo/overrides/botprofile.vpk")).ok();
    let active_sha = active.as_deref().map(hash);
    let mut profiles = Vec::new();
    let tool_state = inspect_vpk_tool(app)?;
    let tool_ready = tool_state.status == "ready";
    for level in ["Low", "Medium", "High"] {
        let bytes = fs::read(
            root.join("game/csgo/overrides")
                .join(level)
                .join("botprofile.vpk"),
        )
        .ok();
        let vpk_sha = bytes.as_deref().map(hash);
        profiles.push(BotProfileSummary {
            id: format!("builtin-{level}"),
            name: level.to_string(),
            source: "builtin".into(),
            base_difficulty: Some(level.into()),
            active: vpk_sha.is_some() && vpk_sha == active_sha,
            read_only: true,
            db_sha256: None,
            vpk_sha256: vpk_sha,
            warnings: if !tool_ready {
                vec!["VPKEdit CLI 未在运行时资源目录中找到，编辑与重包已锁定".into()]
            } else if bytes.is_none() {
                vec!["内置 VPK 尚未安装或无法读取".into()]
            } else {
                Vec::new()
            },
        });
    }

    let profiles_dir = data_dir(app)?.join("profiles");
    if profiles_dir.exists() {
        for entry in fs::read_dir(profiles_dir)
            .map_err(|error| AppError::runtime(format!("[BOT_WORKSHOP_PROFILE_READ] {error}")))?
            .flatten()
        {
            let id = entry.file_name().to_string_lossy().to_string();
            if !valid_id(&id) {
                continue;
            }
            if let Ok(json) = fs::read_to_string(entry.path().join("profile.json")) {
                if let Ok(mut profile) = serde_json::from_str::<BotProfileSummary>(&json) {
                    profile.active = profile.vpk_sha256.is_some()
                        && profile.vpk_sha256.as_ref() == active_sha.as_ref();
                    profiles.push(profile);
                }
            }
        }
    }

    Ok(BotProfileList {
        profiles,
        tool_version: Some(TOOL_VERSION.into()),
        tool_sha256: Some(TOOL_SHA256.into()),
    })
}

fn profile_dir(app: &AppHandle, id: &str) -> Result<PathBuf, AppError> {
    if !valid_id(id) {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_PROFILE_NOT_FOUND] 非法档案 ID",
        ));
    }
    Ok(data_dir(app)?.join("profiles").join(id))
}

fn require_tool(app: &AppHandle) -> Result<PathBuf, AppError> {
    let state = inspect_vpk_tool(app)?;
    if state.status != "ready" {
        return Err(AppError::runtime(format!(
            "[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] {}",
            state.detail.unwrap_or_else(|| "VPKEdit CLI 未就绪".into())
        )));
    }
    tool_candidates(app)
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| {
            AppError::runtime("[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] 未找到 VPKEdit CLI")
        })
}

fn require_cs2_closed() -> Result<(), AppError> {
    if crate::services::cs2::check_cs2_process()? {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_CS2_RUNNING] 请先退出 CS2，再修改或应用强度档案。",
        ));
    }
    Ok(())
}

fn read_text(path: &Path) -> Result<String, AppError> {
    let bytes = fs::read(path)
        .map_err(|e| AppError::runtime(format!("[BOT_WORKSHOP_DB_ENTRY_MISSING] {e}")))?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_DB_TOO_LARGE] botprofile.db 超过 2 MiB 限制",
        ));
    }
    String::from_utf8(bytes)
        .map_err(|_| AppError::runtime("[BOT_WORKSHOP_DB_ENCODING] botprofile.db 不是 UTF-8 文本"))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::runtime("[BOT_WORKSHOP_WRITE] 无效文件路径"))?;
    fs::create_dir_all(parent).map_err(|e| AppError::runtime(e.to_string()))?;
    let temp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("file"),
        std::process::id()
    ));
    let mut file = fs::File::create(&temp).map_err(|e| AppError::runtime(e.to_string()))?;
    file.write_all(bytes)
        .map_err(|e| AppError::runtime(e.to_string()))?;
    file.sync_all()
        .map_err(|e| AppError::runtime(e.to_string()))?;
    drop(file);
    fs::rename(&temp, path).map_err(|e| AppError::runtime(format!("[BOT_WORKSHOP_WRITE] {e}")))
}

fn backup(path: &Path, destination: &Path) -> Result<Option<String>, AppError> {
    if !path.is_file() {
        return Ok(None);
    }
    fs::create_dir_all(destination).map_err(|e| AppError::runtime(e.to_string()))?;
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S-%3f");
    let target = destination.join(format!("botprofile-{stamp}.vpk"));
    fs::copy(path, &target).map_err(|e| AppError::runtime(format!("[BOT_WORKSHOP_BACKUP] {e}")))?;
    Ok(Some(target.display().to_string()))
}

pub fn open(
    app: &AppHandle,
    root_path: &str,
    profile_id: &str,
) -> Result<BotProfileDocument, AppError> {
    require_cs2_closed()?;
    if !valid_id(profile_id) {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_PROFILE_NOT_FOUND] 非法档案 ID",
        ));
    }
    let profile = list(app, root_path)?
        .profiles
        .into_iter()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| AppError::runtime("[BOT_WORKSHOP_PROFILE_NOT_FOUND] 档案不存在"))?;
    let path = if profile.source == "builtin" {
        let root = crate::services::cs2::normalize_root(root_path)?;
        root.join("game/csgo/overrides")
            .join(&profile.name)
            .join("botprofile.vpk")
    } else {
        profile_dir(app, profile_id)?.join("botprofile.vpk")
    };
    let workspace = data_dir(app)?.join("workspace").join(format!(
        "open-{}-{}",
        profile_id,
        std::process::id()
    ));
    let extracted = extract_db(app, path.to_string_lossy().as_ref(), &workspace)?;
    let text = read_text(&extracted)?;
    Ok(BotProfileDocument {
        profile,
        text: Some(text),
        entry_path: "botprofile.db".into(),
        validation: "valid".into(),
        dirty: false,
        tool_version: Some(TOOL_VERSION.into()),
    })
}

pub fn create(
    app: &AppHandle,
    root_path: &str,
    request: CreateBotProfileRequest,
) -> Result<BotProfileOperation, AppError> {
    require_cs2_closed()?;
    require_tool(app)?;
    if !request.base_profile_id.starts_with("builtin-")
        || request.name.trim().is_empty()
        || request.name.chars().count() > 48
    {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_PROFILE_INVALID] 请选择内置基线，并填写不超过 48 个字符的名称。",
        ));
    }
    let base = request.base_profile_id.trim_start_matches("builtin-");
    if !["Low", "Medium", "High"].contains(&base) {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_PROFILE_INVALID] 不支持的内置基线",
        ));
    }
    let id = format!("custom-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%3f"));
    let dir = profile_dir(app, &id)?;
    let root = crate::services::cs2::normalize_root(root_path)?;
    let source = root
        .join("game/csgo/overrides")
        .join(base)
        .join("botprofile.vpk");
    if !source.is_file() {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_PROFILE_NOT_FOUND] 内置 VPK 不存在",
        ));
    }
    fs::create_dir_all(&dir).map_err(|e| AppError::runtime(e.to_string()))?;
    let vpk = dir.join("botprofile.vpk");
    fs::copy(&source, &vpk).map_err(|e| AppError::runtime(format!("[BOT_WORKSHOP_CREATE] {e}")))?;
    let db = extract_db(app, vpk.to_string_lossy().as_ref(), &dir.join("extract"))?;
    let bytes = fs::read(&db).map_err(|e| AppError::runtime(e.to_string()))?;
    atomic_write(&dir.join("botprofile.db"), &bytes)?;
    let profile = BotProfileSummary {
        id: id.clone(),
        name: request.name.trim().to_string(),
        source: "custom".into(),
        base_difficulty: Some(base.into()),
        active: false,
        read_only: false,
        db_sha256: Some(hash(&bytes)),
        vpk_sha256: Some(hash(
            &fs::read(&vpk).map_err(|e| AppError::runtime(e.to_string()))?,
        )),
        warnings: Vec::new(),
    };
    atomic_write(
        &dir.join("profile.json"),
        serde_json::to_vec_pretty(&profile)
            .map_err(|e| AppError::runtime(e.to_string()))?
            .as_slice(),
    )?;
    Ok(BotProfileOperation {
        profile,
        backup_path: None,
        applied: false,
        message: "已从内置基线创建自定义档案；保存后再点击应用。".into(),
    })
}

pub fn save(
    app: &AppHandle,
    root_path: &str,
    request: SaveBotProfileRequest,
) -> Result<BotProfileOperation, AppError> {
    require_cs2_closed()?;
    let tool = require_tool(app)?;
    if request.text.len() > 2 * 1024 * 1024 {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_DB_TOO_LARGE] 文本超过 2 MiB 限制",
        ));
    }
    let dir = profile_dir(app, &request.profile_id)?;
    let json = fs::read_to_string(dir.join("profile.json"))
        .map_err(|_| AppError::runtime("[BOT_WORKSHOP_PROFILE_NOT_FOUND] 自定义档案不存在"))?;
    let mut profile: BotProfileSummary = serde_json::from_str(&json)
        .map_err(|_| AppError::runtime("[BOT_WORKSHOP_PROFILE_CORRUPT] 档案元数据损坏"))?;
    if profile.source != "custom" || profile.read_only {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_READ_ONLY] 内置档案请先复制为自定义档案",
        ));
    }
    let current = fs::read(dir.join("botprofile.db"))
        .map_err(|_| AppError::runtime("[BOT_WORKSHOP_DB_ENTRY_MISSING] 自定义 DB 不存在"))?;
    if hash(&current) != request.expected_db_sha256 {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_SOURCE_CHANGED] 档案已在其他位置变更，请重新打开后再保存。",
        ));
    }
    let candidate_db = dir.join("candidate.db");
    atomic_write(&candidate_db, request.text.as_bytes())?;
    let source_vpk = dir.join("botprofile.vpk");
    let candidate_vpk = dir.join("candidate.vpk");
    fs::copy(&source_vpk, &candidate_vpk).map_err(|e| AppError::runtime(e.to_string()))?;
    let db_arg = candidate_db.to_string_lossy().to_string();
    let vpk_arg = candidate_vpk.to_string_lossy().to_string();
    let (_, stderr, code) = run_cli(
        &tool,
        &[
            "--remove-file",
            "botprofile.db",
            "--add-file",
            &db_arg,
            "botprofile.db",
            "--no-progress",
            &vpk_arg,
        ],
        &dir,
    )?;
    if code != 0 {
        return Err(AppError::runtime(format!(
            "[BOT_WORKSHOP_SAVE] VPK 写入失败: {}",
            stderr.trim()
        )));
    }
    let verified = extract_db(
        app,
        &vpk_arg,
        &dir.join(format!("verify-{}", chrono::Utc::now().timestamp_millis())),
    )?;
    let verified_bytes = fs::read(verified).map_err(|e| AppError::runtime(e.to_string()))?;
    if verified_bytes != request.text.as_bytes() {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_SAVE_ROLLBACK] 回读校验失败，未覆盖原档案。",
        ));
    }
    let backup_path = backup(&source_vpk, &dir.join("backups"))?;
    atomic_write(&dir.join("botprofile.db"), request.text.as_bytes())?;
    fs::rename(&candidate_vpk, &source_vpk)
        .map_err(|e| AppError::runtime(format!("[BOT_WORKSHOP_SAVE] {e}")))?;
    profile.db_sha256 = Some(hash(request.text.as_bytes()));
    profile.vpk_sha256 = Some(hash(
        &fs::read(&source_vpk).map_err(|e| AppError::runtime(e.to_string()))?,
    ));
    atomic_write(
        &dir.join("profile.json"),
        serde_json::to_vec_pretty(&profile)
            .map_err(|e| AppError::runtime(e.to_string()))?
            .as_slice(),
    )?;
    let _ = root_path;
    Ok(BotProfileOperation {
        profile,
        backup_path,
        applied: false,
        message: "已回写自定义 VPK，并完成提取回读校验。".into(),
    })
}

pub fn rename(
    app: &AppHandle,
    request: RenameBotProfileRequest,
) -> Result<BotProfileOperation, AppError> {
    require_cs2_closed()?;
    if !valid_profile_name(&request.name) {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_PROFILE_INVALID] 档案名称必须为 1-48 个字符，且不能包含路径或控制字符。",
        ));
    }
    let (dir, mut profile) = read_custom_profile(app, &request.profile_id)?;
    profile.name = request.name.trim().to_string();
    atomic_write(
        &dir.join("profile.json"),
        serde_json::to_vec_pretty(&profile)
            .map_err(|e| AppError::runtime(e.to_string()))?
            .as_slice(),
    )?;
    Ok(BotProfileOperation {
        profile,
        backup_path: None,
        applied: false,
        message: "自定义档案名称已更新。".into(),
    })
}

pub fn delete(
    app: &AppHandle,
    root_path: &str,
    profile_id: &str,
) -> Result<BotProfileOperation, AppError> {
    require_cs2_closed()?;
    let (dir, mut profile) = read_custom_profile(app, profile_id)?;
    let root = crate::services::cs2::normalize_root(root_path)?;
    let active_path = root.join("game/csgo/overrides/botprofile.vpk");
    let active_bytes = fs::read(&active_path).ok();
    let is_active = active_bytes.as_deref().map(hash).as_ref() == profile.vpk_sha256.as_ref();
    let mut backup_path = None;
    if is_active {
        let base = profile
            .base_difficulty
            .as_deref()
            .ok_or_else(|| AppError::runtime("[BOT_WORKSHOP_PROFILE_CORRUPT] 缺少基础难度"))?;
        if !["Low", "Medium", "High"].contains(&base) {
            return Err(AppError::runtime(
                "[BOT_WORKSHOP_PROFILE_CORRUPT] 基础难度无效",
            ));
        }
        let builtin = root
            .join("game/csgo/overrides")
            .join(base)
            .join("botprofile.vpk");
        let builtin_bytes = fs::read(&builtin).map_err(|_| {
            AppError::runtime("[BOT_WORKSHOP_PROFILE_NOT_FOUND] 对应内置难度档案不存在")
        })?;
        backup_path = backup(&active_path, &data_dir(app)?.join("activation-backups"))?;
        atomic_write(&active_path, &builtin_bytes)?;
        if fs::read(&active_path).ok().as_deref() != Some(builtin_bytes.as_slice()) {
            if let Some(previous) = active_bytes.as_deref() {
                let _ = atomic_write(&active_path, previous);
            }
            return Err(AppError::runtime(
                "[BOT_WORKSHOP_ACTIVE_ROLLBACK] 恢复内置档案回读不一致。",
            ));
        }
        profile.active = false;
    }
    let archive_root = data_dir(app)?.join("deleted-profiles");
    fs::create_dir_all(&archive_root).map_err(|e| AppError::runtime(e.to_string()))?;
    let deleted_at = chrono::Utc::now();
    let archive = archive_root.join(format!(
        "{}-{}",
        profile_id,
        deleted_at.format("%Y%m%d-%H%M%S-%3f")
    ));
    let manifest = serde_json::json!({
        "profileId": profile_id,
        "name": profile.name.clone(),
        "baseDifficulty": profile.base_difficulty.clone(),
        "vpkSha256": profile.vpk_sha256.clone(),
        "wasActive": is_active,
        "deletedAt": deleted_at.to_rfc3339(),
        "activationBackupPath": backup_path.clone(),
    });
    atomic_write(
        &dir.join("deletion.json"),
        serde_json::to_vec_pretty(&manifest)
            .map_err(|e| AppError::runtime(e.to_string()))?
            .as_slice(),
    )?;
    if let Err(error) = fs::rename(&dir, &archive) {
        if is_active {
            if let Some(previous) = active_bytes.as_deref() {
                let _ = atomic_write(&active_path, previous);
            }
        }
        return Err(AppError::runtime(format!(
            "[BOT_WORKSHOP_DELETE] 无法归档档案：{error}"
        )));
    }
    Ok(BotProfileOperation {
        profile,
        backup_path,
        applied: is_active,
        message: if is_active {
            "已删除自定义档案，并恢复对应的内置 BOT 难度。".into()
        } else {
            "已删除自定义档案，原档案已移入可恢复归档。".into()
        },
    })
}

pub fn apply(
    app: &AppHandle,
    root_path: &str,
    profile_id: &str,
) -> Result<BotProfileOperation, AppError> {
    require_cs2_closed()?;
    let dir = profile_dir(app, profile_id)?;
    let json = fs::read_to_string(dir.join("profile.json"))
        .map_err(|_| AppError::runtime("[BOT_WORKSHOP_PROFILE_NOT_FOUND] 自定义档案不存在"))?;
    let mut profile: BotProfileSummary = serde_json::from_str(&json)
        .map_err(|_| AppError::runtime("[BOT_WORKSHOP_PROFILE_CORRUPT] 档案元数据损坏"))?;
    let source = dir.join("botprofile.vpk");
    let source_bytes = fs::read(&source)
        .map_err(|_| AppError::runtime("[BOT_WORKSHOP_PROFILE_CORRUPT] 自定义 VPK 不存在"))?;
    if profile.vpk_sha256.as_deref() != Some(hash(&source_bytes).as_str()) {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_SOURCE_CHANGED] 自定义 VPK 已漂移，请重新保存后再应用。",
        ));
    }
    let root = crate::services::cs2::normalize_root(root_path)?;
    let active = root.join("game/csgo/overrides/botprofile.vpk");
    let backup_path = backup(&active, &data_dir(app)?.join("activation-backups"))?;
    atomic_write(&active, &source_bytes)?;
    if fs::read(&active).map_err(|e| AppError::runtime(e.to_string()))? != source_bytes {
        return Err(AppError::runtime(
            "[BOT_WORKSHOP_ACTIVE_ROLLBACK] 活动 VPK 回读不一致。",
        ));
    }
    profile.active = true;
    Ok(BotProfileOperation {
        profile,
        backup_path,
        applied: true,
        message: "已应用到 BOT 模式；请重新启动 BOT 对局后生效。".into(),
    })
}
