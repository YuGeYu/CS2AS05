use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

use crate::errors::AppError;
use crate::models::bot_difficulty::{
    BotProfileDocument, BotProfileList, BotProfileOperation, BotProfileSummary, BotToolState,
    CreateBotProfileRequest, SaveBotProfileRequest, VpkEntry,
};

const TOOL_VERSION: &str = "VPKEdit CLI v5.0.0.4";
const TOOL_SHA256: &str = "df354e590d157abd633b4a047591363f17e066486e0f59184ff71c51a81b582a";

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
    Ok(BotToolState {
        status: status.into(),
        version: TOOL_VERSION.into(),
        sha256: Some(digest),
        path_hint: Some(path.display().to_string()),
        detail: (code != 0).then(|| format!("--help 退出码 {code}")),
    })
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
    fs::create_dir_all(workspace).map_err(|e| AppError::runtime(e.to_string()))?;
    let output = workspace.join("botprofile.db");
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
        workspace,
    )?;
    if code != 0 {
        return Err(AppError::runtime(format!(
            "[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] extract 退出码 {code}: {}",
            stderr.trim()
        )));
    }
    if !output.is_file() {
        return Err(AppError::runtime(format!(
            "[BOT_WORKSHOP_ENTRY_MANIFEST_MISMATCH] botprofile.db 未成功提取: {}",
            stderr.trim()
        )));
    }
    let _ = stdout;
    Ok(output)
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
