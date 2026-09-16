use crate::errors::AppError;
use std::fs;
use std::path::{Path, PathBuf};

const RELATIVE: &str = "addons/counterstrikesharp/configs/plugins/CS2BotLlmChat/CS2BotLlmChat.json";

fn csgo(root: &str) -> PathBuf {
    Path::new(root).join("game").join("csgo")
}

pub fn read(root: &str) -> Result<String, AppError> {
    let path = csgo(root).join(RELATIVE);
    fs::read_to_string(&path)
        .map_err(|e| AppError::runtime(format!("无法读取 BOT 发言配置：{} ({e})", path.display())))
}

pub fn write(root: &str, content: &str) -> Result<String, AppError> {
    let value: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| AppError::runtime(format!("BOT 发言配置不是有效 JSON：{e}")))?;
    if !value.is_object() {
        return Err(AppError::runtime("BOT 发言配置必须是 JSON 对象。"));
    }
    let path = csgo(root).join(RELATIVE);
    let parent = path
        .parent()
        .ok_or_else(|| AppError::runtime("BOT 发言配置路径无效。"))?;
    fs::create_dir_all(parent).map_err(|e| AppError::runtime(format!("无法创建配置目录：{e}")))?;
    let tmp = parent.join(format!(".CS2BotLlmChat-{}.tmp", std::process::id()));
    let pretty =
        serde_json::to_string_pretty(&value).map_err(|e| AppError::runtime(e.to_string()))?;
    fs::write(&tmp, format!("{pretty}\n"))
        .map_err(|e| AppError::runtime(format!("无法写入临时配置：{e}")))?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| AppError::runtime(format!("无法替换配置：{e}")))?;
    }
    fs::rename(&tmp, &path).map_err(|e| AppError::runtime(format!("无法完成配置替换：{e}")))?;
    read(root)
}
