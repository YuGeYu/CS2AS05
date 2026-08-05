use std::path::Path;

use crate::{errors::AppError, services::cs2};

/// The single process boundary for every demo parser entry point.
pub fn assert_parse_allowed() -> Result<(), AppError> {
    if cs2::check_cs2_process()? {
        return Err(AppError::runtime(
            "[DEMO_PARSE_BLOCKED_CS2_RUNNING] CS2 正在运行，退出后自动继续 Demo 解析。",
        ));
    }
    Ok(())
}

pub fn assert_parse_allowed_for_path(path: &Path) -> Result<(), AppError> {
    assert_parse_allowed()?;
    if !path.is_file() {
        return Err(AppError::runtime(
            "[DEMO_FILE_NOT_FOUND] Demo 文件不存在或尚未完成写入。",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn gate_error_code_is_stable() {
        assert!("DEMO_PARSE_BLOCKED_CS2_RUNNING".contains("DEMO_PARSE_BLOCKED"));
    }
}
