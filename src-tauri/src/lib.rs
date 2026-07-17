// src-tauri/src/lib.rs
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Local;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::System;
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;

// ======================== 数据模型 ========================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub alias: Option<String>,
    pub path: String,
    pub cover: Option<String>, // 存储 Base64 数据 URI
    pub category: String,
    pub tags: Vec<String>,
    pub status: String,
    pub description: Option<String>,
    pub play_time: u64,
    pub last_play: Option<String>,
    pub favorite: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppData {
    pub games: Vec<Game>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub window_radius: u32,
    pub zoom: f64,
    pub startup: bool,
    pub close_action: String,
    pub default_view: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            window_radius: 14,
            zoom: 1.0,
            startup: false,
            close_action: "tray".to_string(),
            default_view: "grid".to_string(),
        }
    }
}

// ======================== 数据存储 ========================

fn get_data_dir(app_handle: &AppHandle) -> Result<PathBuf> {
    let dir = app_handle.path().app_data_dir()?.join("galmanager");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

fn get_games_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let mut path = get_data_dir(app_handle)?;
    path.push("game_list.json");
    Ok(path)
}

fn get_settings_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let mut path = get_data_dir(app_handle)?;
    path.push("setting.json");
    Ok(path)
}

fn read_games(app_handle: &AppHandle) -> Result<AppData> {
    let path = get_games_path(app_handle)?;
    if !path.exists() {
        return Ok(AppData::default());
    }
    let content = fs::read_to_string(&path)?;
    let data: AppData = serde_json::from_str(&content)?;
    Ok(data)
}

fn write_games(app_handle: &AppHandle, data: &AppData) -> Result<()> {
    let path = get_games_path(app_handle)?;
    let content = serde_json::to_string_pretty(data)?;
    fs::write(&path, content)?;
    Ok(())
}

fn read_settings(app_handle: &AppHandle) -> Result<Settings> {
    let path = get_settings_path(app_handle)?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = fs::read_to_string(&path)?;
    let settings: Settings = serde_json::from_str(&content)?;
    Ok(settings)
}

fn write_settings(app_handle: &AppHandle, settings: &Settings) -> Result<()> {
    let path = get_settings_path(app_handle)?;
    let content = serde_json::to_string_pretty(settings)?;
    fs::write(&path, content)?;
    Ok(())
}

// ======================== 文件操作命令 ========================

#[tauri::command]
fn open_file_dialog(
    title: Option<String>,
    extensions: Option<Vec<String>>,
) -> Result<Option<String>, String> {
    let mut dialog = FileDialog::new();
    if let Some(title) = title {
        dialog = dialog.set_title(&title);
    }
    if let Some(exts) = extensions {
        let ext_refs: Vec<&str> = exts.iter().map(|s| s.as_str()).collect();
        if !ext_refs.is_empty() {
            dialog = dialog.add_filter("", &ext_refs);
        }
    }
    match dialog.pick_file() {
        Some(path) => Ok(Some(path.to_string_lossy().to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
fn copy_cover(
    app_handle: tauri::AppHandle,
    source_path: String,
    game_id: String,
) -> Result<String, String> {
    let source = Path::new(&source_path);
    if !source.exists() {
        return Err("源文件不存在".to_string());
    }
    // 读取文件内容并转为 Base64
    let img_data = fs::read(source).map_err(|e| e.to_string())?;
    let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
    let mime = match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        _ => "image/jpeg",
    };
    let base64_str = BASE64.encode(&img_data);
    let data_uri = format!("data:{};base64,{}", mime, base64_str);
    Ok(data_uri)
}

// ======================== 游戏启动与进程管理 ========================

fn launch_game_process(path: &str, args: Option<&[String]>) -> Result<()> {
    let mut cmd = std::process::Command::new(path);
    if let Some(arg_list) = args {
        cmd.args(arg_list);
    }
    if let Some(parent) = Path::new(path).parent() {
        cmd.current_dir(parent);
    }
    cmd.spawn().context("启动游戏失败")?;
    Ok(())
}

fn kill_game_process(path: &str) -> Result<()> {
    let s = System::new_all();
    for process in s.processes().values() {
        if let Some(exe) = process.exe() {
            if exe.to_string_lossy() == path {
                process.kill();
                return Ok(());
            }
        }
    }
    Err(anyhow::anyhow!("未找到运行中的游戏进程"))
}

// ======================== 工具函数 ========================

fn scan_exe_files(folder: &str) -> Result<Vec<String>> {
    let mut exes = Vec::new();
    for entry in WalkDir::new(folder)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "exe") {
            exes.push(path.to_string_lossy().to_string());
        }
    }
    Ok(exes)
}

// ======================== 设置管理 ========================

fn set_startup_enabled(app_handle: &AppHandle, enabled: bool) -> Result<()> {
    let app_name = &app_handle.config().identifier;
    let app_path = std::env::current_exe()?.to_string_lossy().to_string();
    let launch = auto_launch::AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&app_path)
        .set_use_launch_agent(false)
        .build()?;
    if enabled {
        launch.enable()?;
    } else {
        launch.disable()?;
    }
    Ok(())
}

// ======================== Tauri 命令 ========================

#[tauri::command]
fn get_games(app_handle: tauri::AppHandle) -> Result<AppData, String> {
    read_games(&app_handle).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_game(app_handle: tauri::AppHandle, game: Game) -> Result<AppData, String> {
    let mut data = read_games(&app_handle).map_err(|e| e.to_string())?;
    data.games.push(game);
    write_games(&app_handle, &data).map_err(|e| e.to_string())?;
    Ok(data)
}

#[tauri::command]
fn update_game(app_handle: tauri::AppHandle, game: Game) -> Result<AppData, String> {
    let mut data = read_games(&app_handle).map_err(|e| e.to_string())?;
    if let Some(idx) = data.games.iter().position(|g| g.id == game.id) {
        data.games[idx] = game;
    } else {
        return Err("游戏不存在".to_string());
    }
    write_games(&app_handle, &data).map_err(|e| e.to_string())?;
    Ok(data)
}

#[tauri::command]
fn delete_game(app_handle: tauri::AppHandle, id: String) -> Result<AppData, String> {
    let mut data = read_games(&app_handle).map_err(|e| e.to_string())?;
    // 封面数据是 Base64，直接存在 JSON 中，无需删除文件
    data.games.retain(|g| g.id != id);
    write_games(&app_handle, &data).map_err(|e| e.to_string())?;
    Ok(data)
}

#[tauri::command]
fn launch_game(
    app_handle: tauri::AppHandle,
    path: String,
    args: Option<Vec<String>>,
) -> Result<(), String> {
    let args_ref = args.as_ref().map(|v| v.as_slice());
    launch_game_process(&path, args_ref).map_err(|e| e.to_string())
}

#[tauri::command]
fn scan_folder(folder: String) -> Result<Vec<String>, String> {
    scan_exe_files(&folder).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_running_games() -> Vec<String> {
    let s = System::new_all();
    let mut result = Vec::new();
    for process in s.processes().values() {
        if let Some(exe) = process.exe() {
            result.push(exe.to_string_lossy().to_string());
        }
    }
    result
}

#[tauri::command]
fn kill_game(path: String) -> Result<(), String> {
    kill_game_process(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn backup_data(app_handle: tauri::AppHandle) -> Result<String, String> {
    let data_dir = get_data_dir(&app_handle).map_err(|e| e.to_string())?;
    let backup_dir = data_dir.parent().unwrap().join("backup");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_file = backup_dir.join(format!("galmanager_backup_{}.json", timestamp));

    let mut all_data = serde_json::Map::new();
    let games_path = get_games_path(&app_handle).map_err(|e| e.to_string())?;
    let games_content = fs::read_to_string(&games_path).unwrap_or_default();
    all_data.insert(
        "games".to_string(),
        serde_json::from_str(&games_content).unwrap_or(serde_json::json!({})),
    );
    let settings_path = get_settings_path(&app_handle).map_err(|e| e.to_string())?;
    let settings_content = fs::read_to_string(&settings_path).unwrap_or_default();
    all_data.insert(
        "settings".to_string(),
        serde_json::from_str(&settings_content).unwrap_or(serde_json::json!({})),
    );

    let backup_json = serde_json::to_string_pretty(&all_data).map_err(|e| e.to_string())?;
    let mut file = fs::File::create(&backup_file).map_err(|e| e.to_string())?;
    std::io::Write::write_all(&mut file, backup_json.as_bytes()).map_err(|e| e.to_string())?;
    Ok(backup_file.to_string_lossy().to_string())
}

#[tauri::command]
fn restore_data(app_handle: tauri::AppHandle, file_path: String) -> Result<(), String> {
    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let data: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    if let Some(games) = data.get("games") {
        let games_data: AppData =
            serde_json::from_value(games.clone()).map_err(|e| e.to_string())?;
        write_games(&app_handle, &games_data).map_err(|e| e.to_string())?;
    }
    if let Some(settings) = data.get("settings") {
        let settings_data: Settings =
            serde_json::from_value(settings.clone()).map_err(|e| e.to_string())?;
        write_settings(&app_handle, &settings_data).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn cleanup_invalid(app_handle: tauri::AppHandle) -> Result<AppData, String> {
    let mut data = read_games(&app_handle).map_err(|e| e.to_string())?;
    data.games
        .retain(|g| std::path::Path::new(&g.path).exists());
    write_games(&app_handle, &data).map_err(|e| e.to_string())?;
    Ok(data)
}

#[tauri::command]
fn get_settings(app_handle: tauri::AppHandle) -> Result<Settings, String> {
    read_settings(&app_handle).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_settings(app_handle: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    write_settings(&app_handle, &settings).map_err(|e| e.to_string())?;
    set_startup_enabled(&app_handle, settings.startup).map_err(|e| e.to_string())?;
    Ok(())
}

// ======================== 库入口 ========================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_games,
            add_game,
            update_game,
            delete_game,
            launch_game,
            scan_folder,
            get_running_games,
            open_file_dialog,
            copy_cover,
            kill_game,
            backup_data,
            restore_data,
            cleanup_invalid,
            get_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
