use std::path::PathBuf;

use crate::core::error::AppError;
use crate::core::types::{Settings, SortKey};

const SETTINGS_FILE: &str = "settings.json";
const SETTINGS_TEMP: &str = "settings.json.tmp";
const CURRENT_SCHEMA_VERSION: u32 = 1;

pub fn settings_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("RUSTFILES_TEST_SETTINGS_DIR") {
        return PathBuf::from(dir);
    }
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("rustfiles")
}

fn settings_path() -> PathBuf {
    settings_dir().join(SETTINGS_FILE)
}

fn temp_path() -> PathBuf {
    settings_dir().join(SETTINGS_TEMP)
}

pub fn default_settings() -> Settings {
    Settings {
        schema_version: CURRENT_SCHEMA_VERSION,
        show_hidden_files: false,
        show_file_extensions: true,
        sort_key: SortKey::Name,
        sort_ascending: true,
    }
}

pub fn read_settings() -> Result<Settings, AppError> {
    let path = settings_path();
    if !path.exists() {
        return Ok(default_settings());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| {
        AppError::new(
            crate::core::error::ErrorCode::InternalError,
            format!("读取设置失败: {}", e),
        )
    })?;
    let settings: Settings = serde_json::from_str(&content).map_err(|e| {
        AppError::new(
            crate::core::error::ErrorCode::InternalError,
            format!("解析设置失败: {}", e),
        )
    })?;
    Ok(settings)
}

pub fn write_settings(settings: &Settings) -> Result<(), AppError> {
    let dir = settings_dir();
    std::fs::create_dir_all(&dir).map_err(|e| {
        AppError::new(
            crate::core::error::ErrorCode::InternalError,
            format!("创建设置目录失败: {}", e),
        )
    })?;

    let content = serde_json::to_string(settings).map_err(|e| {
        AppError::new(
            crate::core::error::ErrorCode::InternalError,
            format!("序列化设置失败: {}", e),
        )
    })?;

    let temp = temp_path();
    std::fs::write(&temp, &content).map_err(|e| {
        AppError::new(
            crate::core::error::ErrorCode::InternalError,
            format!("写入临时设置文件失败: {}", e),
        )
    })?;

    let target = settings_path();
    std::fs::rename(&temp, &target).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        AppError::new(
            crate::core::error::ErrorCode::InternalError,
            format!("替换设置文件失败: {}", e),
        )
    })?;

    Ok(())
}
