use std::process::Command;

use tauri::Url;

use crate::errors::AppError;

pub const OFFICIAL_SITE_URL: &str = "https://cs2as.600318.xyz/";
pub const IDEA_PAGE_URL: &str = "https://cs2as.600318.xyz/idea";
pub const RELEASE_PAGE_URL: &str = "https://cs2as.600318.xyz/rizhi";
pub const UPSTREAM_PROJECT_URL: &str = "https://github.com/ed0ard/CS2-Bot-Improver";

pub fn open_official_site() -> Result<(), AppError> { open_external(OFFICIAL_SITE_URL) }
pub fn open_idea_page() -> Result<(), AppError> { open_external(IDEA_PAGE_URL) }
pub fn open_release_page() -> Result<(), AppError> { open_external(RELEASE_PAGE_URL) }
pub fn open_upstream_project() -> Result<(), AppError> { open_external(UPSTREAM_PROJECT_URL) }

pub fn open_update_download(url: &str) -> Result<(), AppError> {
    if !is_allowed_update_url(url) {
        return Err(AppError::runtime("下载链接暂不可直接打开。"));
    }
    open_external(url)
}

fn is_allowed_update_url(value: &str) -> bool {
    let Ok(url) = Url::parse(value) else { return false; };
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    if url.port_or_known_default() != Some(443) { return false; }
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
    Command::new("open").arg(url).spawn().map(|_| ())
        .map_err(|error| AppError::runtime(format!("无法打开系统浏览器：{error}")))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_external(url: &str) -> Result<(), AppError> {
    Command::new("xdg-open").arg(url).spawn().map(|_| ())
        .map_err(|error| AppError::runtime(format!("无法打开系统浏览器：{error}")))
}

#[cfg(test)]
mod tests {
    use super::is_allowed_update_url;

    #[test]
    fn validates_update_download_urls() {
        assert!(is_allowed_update_url("https://pan.quark.cn/s/abc"));
        assert!(is_allowed_update_url("https://cs2as.600318.xyz/rizhi"));
        assert!(!is_allowed_update_url("http://pan.quark.cn/s/abc"));
        assert!(!is_allowed_update_url("https://pan.quark.cn.evil.test/s/abc"));
        assert!(!is_allowed_update_url("https://user@pan.quark.cn/s/abc"));
        assert!(!is_allowed_update_url("file:///C:/Windows/System32/calc.exe"));
        assert!(!is_allowed_update_url("javascript:alert(1)"));
    }
}
