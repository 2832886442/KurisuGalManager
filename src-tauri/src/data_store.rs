use crate::models::{AppData, Game, Settings};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use tauri::api::path::app_data_dir;
use tauri::AppHandle;

/// 获取应用数据目录（%APPDATA%/galmanager）
pub fn get_data_dir(app_handle: &AppHandle) -> Result<PathBuf> {
    let dir = app_data_dir(&app_handle.config())
        .ok_or_else(|| anyhow::anyhow!("无法获取应用数据目录"))?
        .join("galmanager");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// 获取游戏数据文件路径
pub fn get_games_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let mut path = get_data_dir(app_handle)?;
    path.push("game_list.json");
    Ok(path)
}

/// 获取设置文件路径
pub fn get_settings_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let mut path = get_data_dir(app_handle)?;
    path.push("setting.json");
    Ok(path)
}

/// 读取游戏数据
pub fn read_games(app_handle: &AppHandle) -> Result<AppData> {
    let path = get_games_path(app_handle)?;
    if !path.exists() {
        // 返回默认空数据
        return Ok(AppData::default());
    }
    let content = fs::read_to_string(&path)?;
    let data: AppData = serde_json::from_str(&content)?;
    Ok(data)
}

/// 写入游戏数据
pub fn write_games(app_handle: &AppHandle, data: &AppData) -> Result<()> {
    let path = get_games_path(app_handle)?;
    let content = serde_json::to_string_pretty(data)?;
    fs::write(&path, content)?;
    Ok(())
}

/// 读取设置
pub fn read_settings(app_handle: &AppHandle) -> Result<Settings> {
    let path = get_settings_path(app_handle)?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = fs::read_to_string(&path)?;
    let settings: Settings = serde_json::from_str(&content)?;
    Ok(settings)
}

/// 写入设置
pub fn write_settings(app_handle: &AppHandle, settings: &Settings) -> Result<()> {
    let path = get_settings_path(app_handle)?;
    let content = serde_json::to_string_pretty(settings)?;
    fs::write(&path, content)?;
    Ok(())
}