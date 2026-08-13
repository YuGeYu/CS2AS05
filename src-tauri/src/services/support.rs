use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use serde_json::json;
use tauri::{AppHandle, Manager, Url};

use crate::errors::AppError;
use crate::models::cs2::{AssistantPreferences, FaultSubmissionResult, OperationResult};
use crate::services::cs2;

pub const OFFICIAL_SITE_URL: &str = "https://cs2as.600318.xyz/";
pub const IDEA_PAGE_URL: &str = "https://cs2as.600318.xyz/idea";
pub const RELEASE_PAGE_URL: &str = "https://cs2as.600318.xyz/rizhi";
pub const UPSTREAM_PROJECT_URL: &str = "https://github.com/ed0ard/CS2-Bot-Improver";
const REFERENCE_PROJECTS: &[(&str, &str)] = &[
    ("bot-improver", UPSTREAM_PROJECT_URL),
    ("demotracer", "https://github.com/unicbm/demotracer"),
    ("demoparser", "https://github.com/LaihoE/demoparser"),
    (
        "cs-demo-manager",
        "https://github.com/akiver/cs-demo-manager",
    ),
    ("skin-forge", "https://github.com/kaecho/CS2-Skin-Forge"),
];
const AUTOSTART_ENTRY: &str = "CS2BotImproverAssistant";
const FAULT_REPORT_URL: &str = "https://cs2as.600318.xyz/api/client-faults";
const MAX_FAULT_DETAILS_CHARS: usize = 4_000;
const MAX_FAULT_LOG_CHARS: usize = 32_000;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FaultReportResponse {
    ticket_id: String,
}

pub fn open_official_site() -> Result<(), AppError> {
    open_external(OFFICIAL_SITE_URL)
}
pub fn open_idea_page() -> Result<(), AppError> {
    open_external(IDEA_PAGE_URL)
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
    for relative in ["skin-forge/cache", "cache", "logs"] {
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
    let log = sanitize_diagnostics(&format!("{}\n{}", diagnostics.summary, diagnostics.full_log));
    let payload = json!({
        "details": details,
        "diagnostics": log,
        "appVersion": app.package_info().version.to_string(),
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    });
    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|error| AppError::runtime(format!("无法初始化故障提交：{error}")))?
        .post(FAULT_REPORT_URL)
        .header("Origin", "http://tauri.localhost")
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
    })
}

fn remove_data_path(path: &Path) -> Result<usize, AppError> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|error| {
            AppError::runtime(format!("无法清理 {}：{error}", path.display()))
        })?;
        return Ok(1);
    }
    if path.is_file() {
        fs::remove_file(path).map_err(|error| {
            AppError::runtime(format!("无法清理 {}：{error}", path.display()))
        })?;
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
    let mut lines = value
        .lines()
        .rev()
        .take(400)
        .collect::<Vec<_>>();
    lines.reverse();
    let mut output = lines
        .into_iter()
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            if ["password", "passwd", "authorization", "bearer ", "private_key", "token="]
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
    use super::{is_allowed_update_url, reference_project_url, REFERENCE_PROJECTS};

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
        assert_eq!(REFERENCE_PROJECTS.len(), 5);
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
            reference_project_url("skin-forge"),
            Some("https://github.com/kaecho/CS2-Skin-Forge")
        );
        assert_eq!(reference_project_url("https://example.com"), None);
        assert_eq!(reference_project_url("unknown"), None);
    }
}
