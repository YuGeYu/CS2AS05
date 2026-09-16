use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, Url};

use crate::errors::AppError;
use crate::models::cs2::{
    AssistantAccount, AssistantPreferences, FaultSubmissionResult, OperationResult,
};
use crate::models::panel::PanelSnapshot;
use crate::services::{cs2, panel};

pub const OFFICIAL_SITE_URL: &str = "https://cs2as.600318.xyz/";
pub const IDEA_PAGE_URL: &str = "https://cs2as.600318.xyz/idea";
pub const RELEASE_PAGE_URL: &str = "https://cs2as.600318.xyz/rizhi";
pub const UPSTREAM_PROJECT_URL: &str = "https://github.com/ed0ard/CS2-Bot-Improver";
pub const INVENTORY_WORKSHOP_URL: &str = "https://inventory.cstrike.app";
const REFERENCE_PROJECTS: &[(&str, &str)] = &[
    ("bot-improver", UPSTREAM_PROJECT_URL),
    ("botvision", "https://github.com/XBribo/CS2-Bot-Vision"),
    ("demotracer", "https://github.com/unicbm/demotracer"),
    ("demoparser", "https://github.com/LaihoE/demoparser"),
    (
        "cs-demo-manager",
        "https://github.com/akiver/cs-demo-manager",
    ),
    (
        "inventory-simulator",
        "https://github.com/ianlucas/cs2-css-inventory-simulator",
    ),
    ("threejs", "https://github.com/mrdoob/three.js"),
    (
        "gametracking-cs2",
        "https://github.com/SteamDatabase/GameTracking-CS2",
    ),
    (
        "insight-agent",
        "https://github.com/DrEAmSs59/CS2-insight-agent",
    ),
];
const AUTOSTART_ENTRY: &str = "CS2BotImproverAssistant";
const FAULT_REPORT_URL: &str = "https://cs2as.600318.xyz/api/client-faults";
const QUICK_REGISTER_URL: &str = "https://cs2as.600318.xyz/api/auth/quick-register";
const LOGIN_URL: &str = "https://cs2as.600318.xyz/api/auth/login";
const REGISTER_URL: &str = "https://cs2as.600318.xyz/register";
const API_PURCHASE_URL: &str = "https://api.600318.xyz";
const DEFAULT_AI_URL: &str = "https://api.600318.xyz";
const DEFAULT_AI_MODEL: &str = "gpt-5.6-sol";
const AI_CONFIG_FILE: &str = "ai-connection.json";
const AI_CHAT_SESSIONS_FILE: &str = "quick-support-sessions.json";
const MAX_AI_CHAT_SESSIONS: usize = 24;
const MAX_AI_CHAT_MESSAGES_PER_SESSION: usize = 80;
const MAX_AI_CHAT_MESSAGE_CHARS: usize = 12_000;
const MAX_FAULT_DETAILS_CHARS: usize = 4_000;
const MAX_FAULT_LOG_CHARS: usize = 32_000;

include!(concat!(env!("OUT_DIR"), "/default_ai_key.rs"));

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FaultReportResponse {
    ticket_id: String,
    #[serde(default)]
    idea_section_id: String,
    #[serde(default)]
    auto_registered: bool,
    #[serde(default)]
    account: Option<AccountCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountCredentials {
    username: String,
    password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConnectionConfig {
    pub url: String,
    pub key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConnectionSummary {
    pub url: String,
    pub model: String,
    pub has_key: bool,
    pub key_hint: Option<String>,
    pub using_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiChatSession {
    pub id: String,
    pub title: String,
    pub messages: Vec<AiChatMessage>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityAuth {
    pub token: String,
    pub service_url: String,
    pub expires_at: u64,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    #[serde(default)]
    credentials: Option<AccountCredentials>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginPayload<'a> {
    username: &'a str,
    password: &'a str,
}

pub fn open_official_site() -> Result<(), AppError> {
    open_external(OFFICIAL_SITE_URL)
}
pub fn open_idea_page() -> Result<(), AppError> {
    open_external(IDEA_PAGE_URL)
}

pub fn open_api_purchase() -> Result<(), AppError> {
    open_external(API_PURCHASE_URL)
}

pub fn open_inventory_workshop() -> Result<(), AppError> {
    open_external(INVENTORY_WORKSHOP_URL)
}

pub fn get_ai_connection(app: &AppHandle) -> Result<AiConnectionSummary, AppError> {
    Ok(connection_summary(&load_ai_connection(app)?))
}

pub fn save_ai_connection(
    app: &AppHandle,
    url: &str,
    key: &str,
    model: &str,
) -> Result<AiConnectionSummary, AppError> {
    let existing = load_ai_connection(app).ok();
    let config = AiConnectionConfig {
        url: url.trim().trim_end_matches('/').to_string(),
        key: if key.trim().is_empty() {
            existing.map(|value| value.key).unwrap_or_default()
        } else {
            key.trim().to_string()
        },
        model: model.trim().to_string(),
    };
    validate_ai_url(&config.url)?;
    if config.model.is_empty() || config.model.chars().count() > 80 {
        return Err(AppError::runtime("模型名称不可用，请检查连接信息。"));
    }
    save_ai_connection_file(app, &config)?;
    Ok(connection_summary(&config))
}

pub async fn get_ai_models(app: &AppHandle, url: &str, key: &str) -> Result<Vec<String>, AppError> {
    let existing = load_ai_connection(app)?;
    let request_url = if url.trim().is_empty() {
        existing.url
    } else {
        url.trim().trim_end_matches('/').to_string()
    };
    validate_ai_url(&request_url)?;
    let request_key = if !key.trim().is_empty() {
        key.trim().to_string()
    } else if !existing.key.is_empty() {
        existing.key
    } else {
        default_ai_key()
    };
    if request_key.is_empty() {
        return Err(AppError::runtime(
            "暂时无法读取模型列表，请检查连接信息或额度。",
        ));
    }
    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|error| AppError::runtime(format!("暂时无法读取模型列表：{error}")))?
        .get(format!("{}/v1/models", request_url))
        .bearer_auth(request_key)
        .send()
        .await
        .map_err(|error| AppError::runtime(format!("暂时无法读取模型列表：{error}")))?;
    if !response.status().is_success() {
        return Err(AppError::runtime(
            "暂时无法读取模型列表，请检查连接信息或额度。",
        ));
    }
    let payload: Value = response
        .json()
        .await
        .map_err(|_| AppError::runtime("模型列表返回了无法读取的内容。"))?;
    let mut models = payload
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("id").and_then(Value::as_str))
        .map(str::to_string)
        .filter(|id| !id.trim().is_empty())
        .collect::<Vec<_>>();
    models.sort_by_key(|value| value.to_ascii_lowercase());
    models.dedup();
    if models.is_empty() {
        return Err(AppError::runtime("当前连接没有返回可用模型。"));
    }
    Ok(models)
}

pub fn run_ai_powershell(command: &str, working_dir: &Path) -> Result<String, AppError> {
    let command = command.trim();
    validate_ai_powershell(command)?;
    fs::create_dir_all(working_dir)
        .map_err(|error| AppError::runtime(format!("无法准备 PowerShell 工作目录：{error}")))?;
    let wrapped = format!("[Console]::OutputEncoding=[Text.Encoding]::UTF8; $OutputEncoding=[Text.Encoding]::UTF8; {command}");
    let mut child = Command::new("powershell.exe")
        .current_dir(working_dir)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &wrapped,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| AppError::runtime(format!("无法执行 PowerShell：{error}")))?;
    let mut stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| AppError::runtime("无法连接 PowerShell 标准输出。"))?;
    let mut stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| AppError::runtime("无法连接 PowerShell 错误输出。"))?;
    let stdout_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout_pipe.read_to_end(&mut bytes).map(|_| bytes)
    });
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr_pipe.read_to_end(&mut bytes).map(|_| bytes)
    });
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| AppError::runtime(format!("无法读取 PowerShell 状态：{error}")))?
        {
            break status;
        }
        if started.elapsed() >= Duration::from_secs(30) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(AppError::runtime("PowerShell 查询超过 30 秒，已自动停止。"));
        }
        thread::sleep(Duration::from_millis(50));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| AppError::runtime("PowerShell 输出读取线程异常结束。"))?
        .map_err(|error| AppError::runtime(format!("无法读取 PowerShell 输出：{error}")))?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| AppError::runtime("PowerShell 错误读取线程异常结束。"))?
        .map_err(|error| AppError::runtime(format!("无法读取 PowerShell 错误：{error}")))?;
    let mut text = String::from_utf8_lossy(&stdout).to_string();
    let error = String::from_utf8_lossy(&stderr);
    if !error.trim().is_empty() {
        text.push_str("\n[stderr]\n");
        text.push_str(&error);
    }
    if text.trim().is_empty() {
        text = format!("命令已结束，退出码：{}", status.code().unwrap_or(-1));
    }
    let mut bounded = text.chars().take(16000).collect::<String>();
    if text.chars().count() > 16000 {
        bounded.push_str("\n[输出已截断]");
    }
    Ok(bounded)
}

fn validate_ai_powershell(command: &str) -> Result<(), AppError> {
    if command.is_empty() || command.chars().count() > 4000 {
        return Err(AppError::runtime("PowerShell 操作内容不可用。"));
    }
    let lower = command.to_ascii_lowercase();
    if [
        "cs2as_default_api_key",
        "account.json",
        "ai-connection.json",
        "credential manager",
        "get-storedcredential",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return Err(AppError::runtime(
            "该 PowerShell 操作涉及助手凭据，已阻止执行。",
        ));
    }
    Ok(())
}

pub async fn chat_ai(
    app: &AppHandle,
    messages: Vec<AiChatMessage>,
    context: Option<String>,
) -> Result<String, AppError> {
    if messages.is_empty() {
        return Err(AppError::runtime("对话内容暂时不可用，请稍后重试。"));
    }
    let mut config = load_ai_connection(app)?;
    if config.key.is_empty() {
        config.key = default_ai_key();
    }
    validate_ai_url(&config.url)?;
    if config.key.is_empty() {
        return Err(AppError::runtime(
            "默认聊天额度暂时不可用，请在连接信息中配置自己的服务。",
        ));
    }
    let source_query = messages
        .last()
        .map(|message| message.content.as_str())
        .unwrap_or_default();
    let source_context = fetch_public_project_context(source_query).await;
    let mut api_messages = Vec::with_capacity(messages.len() + 2);
    if let Some(context) = context.filter(|value| !value.trim().is_empty()) {
        api_messages.push(json!({ "role": "system", "content": format!("你是 CS2 人机增强助手中的快快客服，也是一个可连续使用工具的客户端 Agent。以下快照包含当前页面、可点击按钮 ID 与页面文字。可用工具：1. 导航：[助手操作：导航=overview|presets|items|knives|inventory|commands|demoReview|quickSupport|install]；2. 点击：[助手操作：点击=btn-N]；3. PowerShell：在回答末尾输出 [助手操作：PowerShell]，紧接一个 powershell fenced code block。玩家电脑不一定安装 git、gh 或 rg，也没有本项目源码；查询线上 GitHub 时优先使用 Windows PowerShell 自带的 Invoke-RestMethod，例如读取仓库信息、目录树、commits API 或 raw.githubusercontent.com 文件。只有 Get-Command 确认工具存在后才能使用可选命令。PowerShell 在应用专属临时目录中运行，可用于查询公开网络资料和分析本轮产生的临时数据。每轮最多输出一个工具动作；程序会把执行结果和新快照再次发给你，你必须基于结果继续完成用户目标，不要在工具执行前声称已完成。涉及删除、卸载、清空、提交、推送或其他不可逆操作时，先向用户确认。不要尝试读取连接密钥、账号凭据或系统凭据。没有工具需求时不要输出动作标记。\n{}", context.chars().take(12000).collect::<String>()) }));
    }
    if !source_context.is_empty() {
        api_messages.push(json!({ "role": "system", "content": format!("以下是本轮从当前项目与上游项目公开 GitHub 资料实时读取的片段。回答源码问题时优先引用这些资料，并明确资料范围仅覆盖已读取片段：\n{}", source_context) }));
    }
    let mut bounded_messages = messages.into_iter().rev().take(40).collect::<Vec<_>>();
    bounded_messages.reverse();
    for message in bounded_messages {
        let role = if message.role == "assistant" {
            "assistant"
        } else {
            "user"
        };
        api_messages.push(json!({ "role": role, "content": message.content.chars().take(12000).collect::<String>() }));
    }
    let endpoint = format!("{}/v1/chat/completions", config.url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|error| AppError::runtime(format!("聊天服务暂时不可用：{error}")))?;
    let response = client
        .post(endpoint)
        .bearer_auth(&config.key)
        .json(&json!({ "model": config.model, "messages": api_messages, "stream": false }))
        .send()
        .await
        .map_err(|error| AppError::runtime(format!("聊天服务暂时不可用：{error}")))?;
    let status = response.status();
    let payload: Value = response
        .json()
        .await
        .map_err(|_| AppError::runtime("聊天服务返回了无法读取的结果。"))?;
    if !status.is_success() {
        return Err(AppError::runtime(
            "聊天服务暂时不可用，请检查连接信息或额度。",
        ));
    }
    payload
        .get("choices")
        .and_then(|value| value.get(0))
        .and_then(|value| value.get("message"))
        .and_then(|value| value.get("content"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::runtime("聊天服务没有返回可显示的内容。"))
}

async fn fetch_public_project_context(query: &str) -> String {
    let lower = query.to_ascii_lowercase();
    if ![
        "github", "source", "repo", "upstream", "源码", "上游", "项目",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return String::new();
    }
    let sources = [
        (
            "CS2AS05",
            "https://api.github.com/repos/YuGeYu/CS2AS05/readme",
        ),
        (
            "CS2-Bot-Improver",
            "https://api.github.com/repos/ed0ard/CS2-Bot-Improver/readme",
        ),
        (
            "demoparser",
            "https://api.github.com/repos/LaihoE/demoparser/readme",
        ),
        (
            "Inventory Simulator",
            "https://api.github.com/repos/ianlucas/cs2-css-inventory-simulator/readme",
        ),
    ];
    let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .user_agent("CS2AS-Quick-Support/0.5.9")
        .build()
    else {
        return String::new();
    };
    let mut output = String::new();
    for (name, url) in sources {
        let Ok(response) = client
            .get(url)
            .header("Accept", "application/vnd.github.raw+json")
            .send()
            .await
        else {
            continue;
        };
        if !response.status().is_success() {
            continue;
        }
        let Ok(text) = response.text().await else {
            continue;
        };
        output.push_str(&format!(
            "\n--- {name} ---\n{}",
            text.chars().take(5000).collect::<String>()
        ));
        if output.chars().count() >= 16000 {
            break;
        }
    }
    output.chars().take(16000).collect()
}

fn validate_ai_url(value: &str) -> Result<(), AppError> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| AppError::runtime("连接信息不可用，请检查配置后重试。"))?;
    let host = url.host_str().unwrap_or_default();
    if url.scheme() != "https"
        || url.port().is_some()
        || !host.ends_with(".600318.xyz")
        || host.len() <= ".600318.xyz".len()
    {
        return Err(AppError::runtime("连接信息不可用，请检查配置后重试。"));
    }
    Ok(())
}

fn connection_summary(config: &AiConnectionConfig) -> AiConnectionSummary {
    AiConnectionSummary {
        url: config.url.clone(),
        model: config.model.clone(),
        has_key: !config.key.is_empty() || !default_ai_key().is_empty(),
        key_hint: (!config.key.is_empty())
            .then(|| format!("{}***", config.key.chars().take(4).collect::<String>())),
        using_default: config.url == DEFAULT_AI_URL
            && config.model == DEFAULT_AI_MODEL
            && config.key.is_empty(),
    }
}

fn default_ai_key() -> String {
    DEFAULT_AI_KEY_OBFUSCATED
        .iter()
        .enumerate()
        .map(|(index, byte)| char::from(byte ^ key_mask(index)))
        .collect()
}

fn key_mask(index: usize) -> u8 {
    0xA7u8.wrapping_add((index as u8).wrapping_mul(31))
}

fn load_ai_connection(app: &AppHandle) -> Result<AiConnectionConfig, AppError> {
    let path = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法定位助手数据目录：{error}")))?
        .join(AI_CONFIG_FILE);
    if !path.exists() {
        return Ok(AiConnectionConfig {
            url: DEFAULT_AI_URL.to_string(),
            key: String::new(),
            model: DEFAULT_AI_MODEL.to_string(),
        });
    }
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|error| AppError::runtime(format!("无法读取聊天配置：{error}")))?,
    )
    .map_err(|error| AppError::runtime(format!("聊天配置已损坏，请重新保存：{error}")))
}

fn save_ai_connection_file(app: &AppHandle, config: &AiConnectionConfig) -> Result<(), AppError> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法定位助手数据目录：{error}")))?;
    fs::create_dir_all(&dir)
        .map_err(|error| AppError::runtime(format!("无法初始化聊天配置目录：{error}")))?;
    let value = serde_json::to_string(config)
        .map_err(|error| AppError::runtime(format!("无法保存聊天配置：{error}")))?;
    fs::write(dir.join(AI_CONFIG_FILE), value)
        .map_err(|error| AppError::runtime(format!("无法保存聊天配置：{error}")))
}

pub fn get_ai_chat_sessions(app: &AppHandle) -> Result<Vec<AiChatSession>, AppError> {
    let path = ai_chat_sessions_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let sessions: Vec<AiChatSession> = serde_json::from_str(
        &fs::read_to_string(&path)
            .map_err(|error| AppError::runtime(format!("无法读取客服会话：{error}")))?,
    )
    .map_err(|_| AppError::runtime("客服会话文件已损坏，未加载历史会话。"))?;
    normalize_ai_chat_sessions(sessions)
}

pub fn save_ai_chat_sessions(
    app: &AppHandle,
    sessions: Vec<AiChatSession>,
) -> Result<(), AppError> {
    let sessions = normalize_ai_chat_sessions(sessions)?;
    let path = ai_chat_sessions_path(app)?;
    let parent = path
        .parent()
        .ok_or_else(|| AppError::runtime("无法定位客服会话目录。"))?;
    fs::create_dir_all(parent)
        .map_err(|error| AppError::runtime(format!("无法初始化客服会话目录：{error}")))?;
    let value = serde_json::to_string(&sessions)
        .map_err(|error| AppError::runtime(format!("无法保存客服会话：{error}")))?;
    fs::write(path, value).map_err(|error| AppError::runtime(format!("无法保存客服会话：{error}")))
}

fn ai_chat_sessions_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    Ok(app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法定位助手数据目录：{error}")))?
        .join(AI_CHAT_SESSIONS_FILE))
}

fn normalize_ai_chat_sessions(
    mut sessions: Vec<AiChatSession>,
) -> Result<Vec<AiChatSession>, AppError> {
    if sessions.len() > MAX_AI_CHAT_SESSIONS {
        return Err(AppError::runtime(
            "客服会话数量过多，请先删除部分历史会话。",
        ));
    }
    for session in &mut sessions {
        session.id = session.id.trim().to_string();
        session.title = session.title.trim().chars().take(80).collect();
        if session.id.is_empty()
            || session.id.len() > 80
            || !session
                .id
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
        {
            return Err(AppError::runtime("客服会话标识无效。"));
        }
        if session.title.is_empty() {
            session.title = "新的对话".to_string();
        }
        if session.messages.len() > MAX_AI_CHAT_MESSAGES_PER_SESSION {
            return Err(AppError::runtime(
                "单个客服会话消息过多，请新建会话后继续。",
            ));
        }
        for message in &mut session.messages {
            if !matches!(message.role.as_str(), "user" | "assistant") {
                return Err(AppError::runtime("客服会话包含无法识别的消息类型。"));
            }
            message.content = message
                .content
                .trim()
                .chars()
                .take(MAX_AI_CHAT_MESSAGE_CHARS)
                .collect();
            if message.content.is_empty() {
                return Err(AppError::runtime("客服会话包含空消息。"));
            }
        }
    }
    sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(sessions)
}

pub fn open_fault_idea_page(ticket_id: &str) -> Result<(), AppError> {
    if !is_valid_fault_ticket(ticket_id) {
        return Err(AppError::runtime("故障单编号格式无效，已阻止打开链接。"));
    }
    open_external(&format!("{}?fault={}", IDEA_PAGE_URL, ticket_id))
}

fn is_valid_fault_ticket(ticket_id: &str) -> bool {
    let mut parts = ticket_id.split('-');
    if parts.next() != Some("CS2") {
        return false;
    }
    let rest = parts.collect::<Vec<_>>();
    rest.len() == 2
        && rest.iter().all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
}
pub fn open_release_page() -> Result<(), AppError> {
    open_external(RELEASE_PAGE_URL)
}
pub fn open_upstream_project() -> Result<(), AppError> {
    open_external(UPSTREAM_PROJECT_URL)
}

pub fn open_reference_project(project: &str) -> Result<(), AppError> {
    let url = reference_project_url(project).ok_or_else(|| {
        AppError::runtime("[REFERENCE_PROJECT_NOT_ALLOWED] 未知的参考项目，已阻止打开。")
    })?;
    open_external(url)
}

fn reference_project_url(project: &str) -> Option<&'static str> {
    REFERENCE_PROJECTS
        .iter()
        .find_map(|(id, url)| (*id == project).then_some(*url))
}

pub fn open_update_download(url: &str) -> Result<(), AppError> {
    if !is_allowed_update_url(url) {
        return Err(AppError::runtime("下载链接暂不可直接打开。"));
    }
    open_external(url)
}

pub fn open_resource_link(url: &str) -> Result<(), AppError> {
    let parsed = Url::parse(url).map_err(|_| AppError::runtime("资源链接格式无效。"))?;
    if parsed.scheme() != "https" || !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(AppError::runtime("资源链接暂不可打开。"));
    }
    if !((parsed.scheme() == "http" && parsed.host_str() == Some("8.133.185.37"))
        || matches!(
            parsed.host_str(),
            Some("vault.600318.xyz" | "pan.quark.cn" | "quark.cn")
        ))
    {
        return Err(AppError::runtime("资源链接域名不在允许范围内。"));
    }
    open_external(url)
}

pub fn open_community_download(url: &str) -> Result<(), AppError> {
    let parsed = Url::parse(url).map_err(|_| AppError::runtime("资源链接格式无效。"))?;
    if parsed.scheme() != "http"
        || parsed.host_str() != Some("8.133.185.37")
        || !parsed.path().starts_with("/community/file/")
    {
        return Err(AppError::runtime("圈子资源链接不受信任。"));
    }
    open_external(url)
}
pub fn open_account_register() -> Result<(), AppError> {
    open_external(REGISTER_URL)
}

pub fn launch_community_connect(
    app: &AppHandle,
    root_path: &str,
    connection: &str,
) -> Result<PanelSnapshot, AppError> {
    let value = connection
        .trim()
        .strip_prefix("connect ")
        .unwrap_or(connection.trim());
    let valid = value.starts_with("[A:")
        && value.contains("] (")
        && value.ends_with(')')
        && value.chars().all(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, '[' | ']' | ':' | ' ' | '(' | ')')
        });
    if !valid || value.len() > 96 {
        return Err(AppError::runtime("连接信息格式无效。"));
    }
    // Source2 的 Steam server ID 含有空格、方括号和括号。通过
    // `steam.exe -applaunch` 传递时，Steam 在部分版本会丢失该参数，
    // 最终触发 Source2ServerConfig001。使用官方 rungame URI 让 Steam
    // 原样转交启动参数，并对 URI 保留字符进行百分号编码。
    let encoded = steam_uri_encode(value);
    let uri = format!("steam://rungame/730/76561202255233023/+connect%20{encoded}");
    let steam = crate::services::cs2_discovery::find_steam_executable(None)
        .ok_or_else(|| AppError::runtime("未找到 Steam 客户端，请先安装或启动 Steam。"))?;

    // 圈子连接属于在线会话。必须先通过与概览页相同的事务切回
    // 官方 Online gameinfo，再把连接参数交给 Steam。
    let snapshot = panel::set_mode_with_app(app, root_path, "online")?;
    cs2::write_runtime_log(
        "INFO",
        &format!(
            "玩家圈子连接已切换在线模式，使用 Steam 客户端：{}",
            steam.display()
        ),
    );
    Command::new(&steam)
        .arg(&uri)
        .spawn()
        .map_err(|error| AppError::runtime(format!("无法通过 Steam 启动 CS2：{error}")))?;
    Ok(snapshot)
}

fn steam_uri_encode(value: &str) -> String {
    value.bytes().fold(String::new(), |mut out, byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~' | b':') {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push_str(&format!("{byte:02X}"));
        }
        out
    })
}

pub fn should_show_volume_smoke_guide(app: &AppHandle) -> Result<bool, AppError> {
    let marker = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法定位助手数据目录：{error}")))?
        .join("onboarding-volume-smoke-v0.5.14.done");
    Ok(!marker.is_file())
}

pub fn dismiss_volume_smoke_guide(app: &AppHandle) -> Result<(), AppError> {
    let marker = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法定位助手数据目录：{error}")))?
        .join("onboarding-volume-smoke-v0.5.14.done");
    if let Some(parent) = marker.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| AppError::runtime(format!("无法保存新版本引导状态：{error}")))?;
    }
    fs::write(marker, b"seen\n")
        .map_err(|error| AppError::runtime(format!("无法保存新版本引导状态：{error}")))
}

pub fn is_bot_profile_guide_seen(app: &AppHandle) -> Result<bool, AppError> {
    let marker = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法定位助手数据目录：{error}")))?
        .join("onboarding-botprofile-db-v0.5.14.done");
    Ok(marker.exists())
}

pub fn dismiss_bot_profile_guide(app: &AppHandle) -> Result<(), AppError> {
    let marker = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法定位助手数据目录：{error}")))?
        .join("onboarding-botprofile-db-v0.5.14.done");
    if let Some(parent) = marker.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::runtime(format!("无法保存 botprofile.db 阅读状态：{error}"))
        })?;
    }
    fs::write(marker, b"seen\n")
        .map_err(|error| AppError::runtime(format!("无法保存 botprofile.db 阅读状态：{error}")))
}

pub fn get_assistant_preferences() -> Result<AssistantPreferences, AppError> {
    Ok(AssistantPreferences {
        autostart_enabled: autostart_enabled()?,
    })
}

pub fn set_assistant_autostart(enabled: bool) -> Result<AssistantPreferences, AppError> {
    set_autostart(enabled)?;
    get_assistant_preferences()
}

pub fn clear_assistant_data(app: &AppHandle) -> Result<OperationResult, AppError> {
    let local_data = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法定位助手数据目录：{error}")))?;
    let mut removed = 0usize;
    for relative in [
        "cache",
        "logs",
        "account.json",
        AI_CONFIG_FILE,
        AI_CHAT_SESSIONS_FILE,
    ] {
        removed += remove_data_path(&local_data.join(relative))?;
    }

    let runtime_log = diagnostics_log_path();
    removed += remove_data_path(&runtime_log)?;
    cs2::write_runtime_log("INFO", "用户已清除助手缓存、日志与界面偏好。");

    Ok(OperationResult {
        success: true,
        message: format!(
            "已清理 {removed} 处助手缓存或日志。CS2 文件、插件、Demo 与复盘记录均已保留。"
        ),
    })
}

pub async fn submit_fault_report(
    app: &AppHandle,
    details: &str,
    root_path: Option<&str>,
) -> Result<FaultSubmissionResult, AppError> {
    let details = details.trim();
    let detail_chars = details.chars().count();
    if !(10..=MAX_FAULT_DETAILS_CHARS).contains(&detail_chars) {
        return Err(AppError::runtime("故障详情需要填写 10-4000 个字符。"));
    }

    let diagnostics = cs2::get_diagnostics_payload(root_path)?;
    let log = sanitize_diagnostics(&format!(
        "{}\n{}",
        diagnostics.summary, diagnostics.full_log
    ));
    let payload = json!({
        "details": details,
        "diagnostics": log,
        "appVersion": app.package_info().version.to_string(),
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    });
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|error| AppError::runtime(format!("无法初始化故障提交：{error}")))?;
    let (cookie, credentials, auto_registered) = match load_account(app)? {
        Some(account) => (authenticate(&client, &account).await?, account, false),
        None => {
            let (cookie, generated) = quick_register(&client).await?;
            save_account(app, &generated)?;
            (cookie, generated, true)
        }
    };
    let response = client
        .post(FAULT_REPORT_URL)
        .header("Origin", "http://tauri.localhost")
        .header("Cookie", &cookie)
        .json(&payload)
        .send()
        .await
        .map_err(|error| AppError::runtime(format!("故障单提交失败，请检查网络后重试：{error}")))?;
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_default();
        return Err(AppError::runtime(format!(
            "故障单提交失败（HTTP {}）：{}",
            status.as_u16(),
            message.chars().take(240).collect::<String>()
        )));
    }
    let response = response
        .json::<FaultReportResponse>()
        .await
        .map_err(|error| AppError::runtime(format!("官网返回了无效的故障单编号：{error}")))?;
    cs2::write_runtime_log("INFO", &format!("故障单 {} 已提交。", response.ticket_id));
    Ok(FaultSubmissionResult {
        success: true,
        message: format!("故障单 {} 已提交，诊断日志已一并送达。", response.ticket_id),
        ticket_id: response.ticket_id,
        idea_section_id: response.idea_section_id,
        account_username: response
            .account
            .map(|account| account.username)
            .unwrap_or(credentials.username),
        auto_registered: auto_registered || response.auto_registered,
    })
}

pub fn get_assistant_account(app: &AppHandle) -> Result<AssistantAccount, AppError> {
    let account = load_account(app)?;
    Ok(AssistantAccount {
        username: account.as_ref().map(|value| value.username.clone()),
        logged_in: account.is_some(),
    })
}

pub async fn login_assistant(
    app: &AppHandle,
    username: &str,
    password: &str,
) -> Result<AssistantAccount, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|error| AppError::runtime(format!("无法初始化登录请求：{error}")))?;
    let credentials = AccountCredentials {
        username: username.trim().to_string(),
        password: password.to_string(),
    };
    let _ = authenticate(&client, &credentials).await?;
    save_account(app, &credentials)?;
    get_assistant_account(app)
}

pub async fn get_community_auth(app: &AppHandle) -> Result<CommunityAuth, AppError> {
    let account = load_account(app)?.ok_or_else(|| AppError::runtime("请先登录官网账号"))?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| AppError::runtime(format!("无法初始化圈子请求：{e}")))?;
    let cookie = authenticate(&client, &account).await?;
    let response = client
        .post("https://cs2as.600318.xyz/api/community/token")
        .header("Cookie", cookie)
        .json(&json!({}))
        .send()
        .await
        .map_err(|e| AppError::runtime(format!("圈子令牌请求失败：{e}")))?;
    let status = response.status();
    if !status.is_success() {
        return Err(AppError::runtime(format!(
            "圈子令牌请求失败（HTTP {}）",
            status.as_u16()
        )));
    }
    response
        .json::<CommunityAuth>()
        .await
        .map_err(|e| AppError::runtime(format!("圈子服务返回无效令牌：{e}")))
}

pub async fn download_community_file(
    app: &AppHandle,
    url: &str,
    filename: &str,
    token: &str,
) -> Result<String, AppError> {
    if !(url.starts_with("http://8.133.185.37/") || url.starts_with("https://8.133.185.37/")) {
        return Err(AppError::runtime("资源地址不受信任"));
    }
    let safe_name: String = filename
        .chars()
        .map(|c| if "\\/:*?\"<>|".contains(c) { '_' } else { c })
        .collect();
    let path = app
        .path()
        .download_dir()
        .map_err(|e| AppError::runtime(format!("无法定位下载目录：{e}")))?
        .join(safe_name);
    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| AppError::runtime(format!("资源下载失败：{e}")))?;
    if !response.status().is_success() {
        return Err(AppError::runtime(format!(
            "资源下载失败（HTTP {}）",
            response.status().as_u16()
        )));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::runtime(format!("资源读取失败：{e}")))?;
    fs::write(&path, &bytes).map_err(|e| AppError::runtime(format!("无法保存资源：{e}")))?;
    #[cfg(windows)]
    {
        let _ = Command::new("explorer.exe")
            .arg("/select,")
            .arg(&path)
            .spawn();
    }
    Ok(path.to_string_lossy().into_owned())
}

pub fn logout_assistant(app: &AppHandle) -> Result<AssistantAccount, AppError> {
    let path = account_path(app)?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| AppError::runtime(format!("无法退出登录：{error}")))?;
    }
    get_assistant_account(app)
}

async fn quick_register(
    client: &reqwest::Client,
) -> Result<(String, AccountCredentials), AppError> {
    let response = client
        .post(QUICK_REGISTER_URL)
        .json(&json!({}))
        .send()
        .await
        .map_err(|error| AppError::runtime(format!("一键注册失败，请检查网络后重试：{error}")))?;
    let status = response.status();
    let cookie =
        session_cookie(&response).ok_or_else(|| AppError::runtime("官网没有返回有效登录会话。"))?;
    if !status.is_success() {
        return Err(AppError::runtime(format!(
            "一键注册失败（HTTP {}）。",
            status.as_u16()
        )));
    }
    let payload = response
        .json::<AuthResponse>()
        .await
        .map_err(|error| AppError::runtime(format!("官网返回了无效的注册信息：{error}")))?;
    let credentials = payload
        .credentials
        .ok_or_else(|| AppError::runtime("官网没有返回新账号信息。"))?;
    Ok((cookie, credentials))
}

async fn authenticate(
    client: &reqwest::Client,
    account: &AccountCredentials,
) -> Result<String, AppError> {
    let response = client
        .post(LOGIN_URL)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0.0.0 Safari/537.36")
        .header("Accept", "application/json")
        .header("Origin", OFFICIAL_SITE_URL.trim_end_matches('/'))
        .header("Referer", OFFICIAL_SITE_URL)
        .json(&LoginPayload {
            username: &account.username,
            password: &account.password,
        })
        .send()
        .await
        .map_err(|error| AppError::runtime(format!("登录官网失败，请检查网络后重试：{error}")))?;
    let status = response.status();
    if !status.is_success() {
        return Err(AppError::runtime(format!(
            "官网登录失败（HTTP {}），请重新登录。",
            status.as_u16()
        )));
    }
    session_cookie(&response).ok_or_else(|| AppError::runtime("官网没有返回有效登录会话。"))
}

fn session_cookie(response: &reqwest::Response) -> Option<String> {
    response
        .headers()
        .get("set-cookie")?
        .to_str()
        .ok()?
        .split(';')
        .next()
        .map(str::to_string)
}

fn account_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::runtime(format!("无法定位助手数据目录：{error}")))?;
    fs::create_dir_all(&dir)
        .map_err(|error| AppError::runtime(format!("无法初始化账号目录：{error}")))?;
    Ok(dir.join("account.json"))
}

fn load_account(app: &AppHandle) -> Result<Option<AccountCredentials>, AppError> {
    let path = account_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let value = fs::read_to_string(path)
        .map_err(|error| AppError::runtime(format!("无法读取登录状态：{error}")))?;
    serde_json::from_str(&value)
        .map(Some)
        .map_err(|error| AppError::runtime(format!("登录状态已损坏，请重新登录：{error}")))
}

fn save_account(app: &AppHandle, account: &AccountCredentials) -> Result<(), AppError> {
    let path = account_path(app)?;
    let value = serde_json::to_string(account)
        .map_err(|error| AppError::runtime(format!("无法保存登录状态：{error}")))?;
    fs::write(path, value).map_err(|error| AppError::runtime(format!("无法保存登录状态：{error}")))
}

fn remove_data_path(path: &Path) -> Result<usize, AppError> {
    if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|error| AppError::runtime(format!("无法清理 {}：{error}", path.display())))?;
        return Ok(1);
    }
    if path.is_file() {
        fs::remove_file(path)
            .map_err(|error| AppError::runtime(format!("无法清理 {}：{error}", path.display())))?;
        return Ok(1);
    }
    Ok(0)
}

fn diagnostics_log_path() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("CS2人机增强助手/logs/runtime.log")
}

fn sanitize_diagnostics(value: &str) -> String {
    let profile = std::env::var("USERPROFILE").unwrap_or_default();
    let mut lines = value.lines().rev().take(400).collect::<Vec<_>>();
    lines.reverse();
    let mut output = lines
        .into_iter()
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            if [
                "password",
                "passwd",
                "authorization",
                "bearer ",
                "private_key",
                "token=",
            ]
            .iter()
            .any(|needle| lower.contains(needle))
            {
                return "[已隐藏敏感日志行]".to_string();
            }
            if profile.is_empty() {
                line.to_string()
            } else {
                line.replace(&profile, "%USERPROFILE%")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if output.chars().count() > MAX_FAULT_LOG_CHARS {
        output = output
            .chars()
            .rev()
            .take(MAX_FAULT_LOG_CHARS)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        output.insert_str(0, "[较早日志已裁剪]\n");
    }
    output
}

#[cfg(windows)]
fn autostart_enabled() -> Result<bool, AppError> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let run = match current_user.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
        Ok(run) => run,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(AppError::runtime(format!("无法读取开机启动设置：{error}"))),
    };
    Ok(run.get_value::<String, _>(AUTOSTART_ENTRY).is_ok())
}

#[cfg(windows)]
fn set_autostart(enabled: bool) -> Result<(), AppError> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let (run, _) = current_user
        .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .map_err(|error| AppError::runtime(format!("无法写入开机启动设置：{error}")))?;
    if enabled {
        let executable = std::env::current_exe()
            .map_err(|error| AppError::runtime(format!("无法定位助手程序：{error}")))?;
        run.set_value(AUTOSTART_ENTRY, &format!("\"{}\"", executable.display()))
            .map_err(|error| AppError::runtime(format!("无法开启开机启动：{error}")))?;
    } else if let Err(error) = run.delete_value(AUTOSTART_ENTRY) {
        if error.kind() != std::io::ErrorKind::NotFound {
            return Err(AppError::runtime(format!("无法关闭开机启动：{error}")));
        }
    }
    Ok(())
}

#[cfg(not(windows))]
fn autostart_enabled() -> Result<bool, AppError> {
    Ok(false)
}

#[cfg(not(windows))]
fn set_autostart(_enabled: bool) -> Result<(), AppError> {
    Err(AppError::runtime("当前系统暂不支持开机启动设置。"))
}

fn is_allowed_update_url(value: &str) -> bool {
    let Ok(url) = Url::parse(value) else {
        return false;
    };
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    if url.port_or_known_default() != Some(443) {
        return false;
    }
    matches!(url.host_str(), Some("pan.quark.cn" | "cs2as.600318.xyz"))
}

#[cfg(target_os = "windows")]
fn open_external(url: &str) -> Result<(), AppError> {
    Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn()
        .map(|_| ())
        .map_err(|error| AppError::runtime(format!("无法打开系统浏览器：{error}")))
}

#[cfg(target_os = "macos")]
fn open_external(url: &str) -> Result<(), AppError> {
    Command::new("open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| AppError::runtime(format!("无法打开系统浏览器：{error}")))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_external(url: &str) -> Result<(), AppError> {
    Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| AppError::runtime(format!("无法打开系统浏览器：{error}")))
}

#[cfg(test)]
mod tests {
    use super::{
        is_allowed_update_url, reference_project_url, run_ai_powershell, steam_uri_encode,
        validate_ai_powershell, REFERENCE_PROJECTS,
    };

    #[test]
    fn allows_portable_public_github_queries() {
        assert!(validate_ai_powershell(
            "Invoke-RestMethod https://api.github.com/repos/YuGeYu/CS2AS05"
        )
        .is_ok());
        assert!(validate_ai_powershell("Get-Command git -ErrorAction SilentlyContinue").is_ok());
    }

    #[test]
    fn blocks_assistant_credentials_from_powershell() {
        assert!(validate_ai_powershell("Get-ChildItem Env:CS2AS_DEFAULT_API_KEY").is_err());
        assert!(validate_ai_powershell("Get-Content ai-connection.json").is_err());
        assert!(validate_ai_powershell("Get-Content account.json").is_err());
    }

    #[cfg(windows)]
    #[test]
    fn runs_powershell_in_an_arbitrary_player_cache_directory() {
        let working_dir = std::env::temp_dir().join("cs2as-quick-support-test");
        let output = run_ai_powershell("(Get-Location).Path", &working_dir)
            .expect("PowerShell should run from a player-local cache directory");
        assert_eq!(
            dunce::canonicalize(output.trim()).expect("PowerShell working directory exists"),
            dunce::canonicalize(&working_dir).expect("working directory exists")
        );
    }

    #[test]
    fn validates_update_download_urls() {
        assert!(is_allowed_update_url("https://pan.quark.cn/s/abc"));
        assert!(is_allowed_update_url("https://cs2as.600318.xyz/rizhi"));
        assert!(!is_allowed_update_url("http://pan.quark.cn/s/abc"));
        assert!(!is_allowed_update_url(
            "https://pan.quark.cn.evil.test/s/abc"
        ));
        assert!(!is_allowed_update_url("https://user@pan.quark.cn/s/abc"));
        assert!(!is_allowed_update_url(
            "file:///C:/Windows/System32/calc.exe"
        ));
        assert!(!is_allowed_update_url("javascript:alert(1)"));
    }

    #[test]
    fn maps_only_fixed_reference_project_ids() {
        assert_eq!(REFERENCE_PROJECTS.len(), 9);
        assert_eq!(
            reference_project_url("bot-improver"),
            Some("https://github.com/ed0ard/CS2-Bot-Improver")
        );
        assert_eq!(
            reference_project_url("demotracer"),
            Some("https://github.com/unicbm/demotracer")
        );
        assert_eq!(
            reference_project_url("demoparser"),
            Some("https://github.com/LaihoE/demoparser")
        );
        assert_eq!(
            reference_project_url("cs-demo-manager"),
            Some("https://github.com/akiver/cs-demo-manager")
        );
        assert_eq!(
            reference_project_url("inventory-simulator"),
            Some("https://github.com/ianlucas/cs2-css-inventory-simulator")
        );
        assert_eq!(reference_project_url("https://example.com"), None);
        assert_eq!(reference_project_url("unknown"), None);
    }

    #[test]
    fn encodes_source2_server_id_for_steam_uri() {
        assert_eq!(
            steam_uri_encode("[A:1:2004590609:51382] (90292678561603601)"),
            "%5BA:1:2004590609:51382%5D%20%2890292678561603601%29"
        );
    }
}
