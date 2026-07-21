use crate::error::{AppError, ErrorCode};
use crate::models::{AppData, Settings};
use crate::path_manager;
use std::fs;
use std::path::PathBuf;

/// 获取游戏数据文件路径
pub fn get_games_path() -> PathBuf {
    path_manager::games_file()
}

/// 获取设置文件路径
pub fn get_settings_path() -> PathBuf {
    path_manager::settings_file()
}

/// 读取游戏数据
pub fn read_games() -> Result<AppData, AppError> {
    let path = get_games_path();
    if !path.exists() {
        return Ok(AppData::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| AppError::wrap(ErrorCode::DataReadFailed, "读取游戏数据文件失败", e))?;
    serde_json::from_str(&content)
        .map_err(|e| AppError::wrap(ErrorCode::DataDeserializeFailed, "解析游戏数据失败", e))
}

/// 写入游戏数据（原子写入：先写临时文件再重命名）
pub fn write_games(data: &AppData) -> Result<(), AppError> {
    let path = get_games_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "创建数据目录失败", e))?;
    }
    let content = serde_json::to_string_pretty(data)
        .map_err(|e| AppError::wrap(ErrorCode::DataSerializeFailed, "序列化游戏数据失败", e))?;
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &content)
        .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "写入游戏数据失败", e))?;
    fs::rename(&tmp_path, &path)
        .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "保存游戏数据失败", e))?;
    Ok(())
}

/// 读取设置
pub fn read_settings() -> Result<Settings, AppError> {
    let path = get_settings_path();
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| AppError::wrap(ErrorCode::SettingsLoadFailed, "读取设置文件失败", e))?;
    serde_json::from_str(&content)
        .map_err(|e| AppError::wrap(ErrorCode::DataDeserializeFailed, "解析设置数据失败", e))
}

/// 写入设置
pub fn write_settings(settings: &Settings) -> Result<(), AppError> {
    let path = get_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::wrap(ErrorCode::SettingsSaveFailed, "创建设置目录失败", e))?;
    }
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| AppError::wrap(ErrorCode::DataSerializeFailed, "序列化设置失败", e))?;
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &content)
        .map_err(|e| AppError::wrap(ErrorCode::SettingsSaveFailed, "写入设置失败", e))?;
    fs::rename(&tmp_path, &path)
        .map_err(|e| AppError::wrap(ErrorCode::SettingsSaveFailed, "保存设置失败", e))?;
    Ok(())
}
