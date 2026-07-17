use crate::data_store;
use crate::game_launcher;
use crate::models::{AppData, Game, Settings};
use crate::settings;
use crate::utils;
use anyhow::Result;
use serde_json::json;
use tauri::command;

/// 获取所有游戏列表
#[command]
pub fn get_games(app_handle: tauri::AppHandle) -> Result<AppData, String> {
    data_store::read_games(&app_handle).map_err(|e| e.to_string())
}

/// 添加游戏
#[command]
pub fn add_game(app_handle: tauri::AppHandle, game: Game) -> Result<AppData, String> {
    let mut data = data_store::read_games(&app_handle).map_err(|e| e.to_string())?;
    data.games.push(game);
    data_store::write_games(&app_handle, &data).map_err(|e| e.to_string())?;
    Ok(data)
}

/// 更新游戏
#[command]
pub fn update_game(app_handle: tauri::AppHandle, game: Game) -> Result<AppData, String> {
    let mut data = data_store::read_games(&app_handle).map_err(|e| e.to_string())?;
    if let Some(idx) = data.games.iter().position(|g| g.id == game.id) {
        data.games[idx] = game;
    } else {
        return Err("游戏不存在".to_string());
    }
    data_store::write_games(&app_handle, &data).map_err(|e| e.to_string())?;
    Ok(data)
}

/// 删除游戏
#[command]
pub fn delete_game(app_handle: tauri::AppHandle, id: String) -> Result<AppData, String> {
    let mut data = data_store::read_games(&app_handle).map_err(|e| e.to_string())?;
    data.games.retain(|g| g.id != id);
    data_store::write_games(&app_handle, &data).map_err(|e| e.to_string())?;
    Ok(data)
}

/// 启动游戏
#[command]
pub fn launch_game(app_handle: tauri::AppHandle, path: String, args: Option<Vec<String>>) -> Result<(), String> {
    let args_ref = args.as_ref().map(|v| v.as_slice());
    game_launcher::launch_game(&app_handle, &path, args_ref).map_err(|e| e.to_string())
}

/// 扫描文件夹获取 exe 列表
#[command]
pub fn scan_folder(folder: String) -> Result<Vec<String>, String> {
    utils::scan_exe_files(&folder).map_err(|e| e.to_string())
}

/// 获取运行中的游戏列表
#[command]
pub fn get_running_games() -> Vec<String> {
    game_launcher::get_running_games()
}

/// 结束游戏进程
#[command]
pub fn kill_game(path: String) -> Result<(), String> {
    game_launcher::kill_game(&path).map_err(|e| e.to_string())
}

/// 备份数据（导出为 zip 或直接复制文件夹）
#[command]
pub fn backup_data(app_handle: tauri::AppHandle) -> Result<String, String> {
    use std::fs;
    use std::io::Write;
    use chrono::Local;
    let data_dir = data_store::get_data_dir(&app_handle).map_err(|e| e.to_string())?;
    let backup_dir = data_dir.parent().unwrap().join("backup");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_file = backup_dir.join(format!("galmanager_backup_{}.json", timestamp));
    // 简单合并所有 JSON 文件为一个
    let mut all_data = serde_json::Map::new();
    // 读取游戏数据
    let games_path = data_store::get_games_path(&app_handle).map_err(|e| e.to_string())?;
    let games_content = fs::read_to_string(&games_path).unwrap_or_default();
    all_data.insert("games".to_string(), serde_json::from_str(&games_content).unwrap_or(json!({})));
    // 读取设置
    let settings_path = data_store::get_settings_path(&app_handle).map_err(|e| e.to_string())?;
    let settings_content = fs::read_to_string(&settings_path).unwrap_or_default();
    all_data.insert("settings".to_string(), serde_json::from_str(&settings_content).unwrap_or(json!({})));
    // 写入备份文件
    let backup_json = serde_json::to_string_pretty(&all_data).map_err(|e| e.to_string())?;
    let mut file = fs::File::create(&backup_file).map_err(|e| e.to_string())?;
    file.write_all(backup_json.as_bytes()).map_err(|e| e.to_string())?;
    Ok(backup_file.to_string_lossy().to_string())
}

/// 恢复数据（从备份文件）
#[command]
pub fn restore_data(app_handle: tauri::AppHandle, file_path: String) -> Result<(), String> {
    use std::fs;
    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let data: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    if let Some(games) = data.get("games") {
        let games_data: AppData = serde_json::from_value(games.clone()).map_err(|e| e.to_string())?;
        data_store::write_games(&app_handle, &games_data).map_err(|e| e.to_string())?;
    }
    if let Some(settings) = data.get("settings") {
        let settings_data: Settings = serde_json::from_value(settings.clone()).map_err(|e| e.to_string())?;
        data_store::write_settings(&app_handle, &settings_data).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 清理无效数据（删除路径不存在的游戏）
#[command]
pub fn cleanup_invalid(app_handle: tauri::AppHandle) -> Result<AppData, String> {
    let mut data = data_store::read_games(&app_handle).map_err(|e| e.to_string())?;
    let original_len = data.games.len();
    data.games.retain(|g| std::path::Path::new(&g.path).exists());
    let removed = original_len - data.games.len();
    if removed > 0 {
        data_store::write_games(&app_handle, &data).map_err(|e| e.to_string())?;
    }
    Ok(data)
}

/// 获取设置
#[command]
pub fn get_settings(app_handle: tauri::AppHandle) -> Result<Settings, String> {
    settings::load_settings(&app_handle).map_err(|e| e.to_string())
}

/// 保存设置
#[command]
pub fn save_settings(app_handle: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    settings::save_settings(&app_handle, &settings).map_err(|e| e.to_string())
}