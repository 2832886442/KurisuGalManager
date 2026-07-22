use crate::error::{AppError, ErrorCode};
use crate::models::{AppData, Game, Ranking, Settings};
use crate::path_manager;
use log::info;
use std::fs;
use std::path::PathBuf;

/// 获取游戏数据文件路径（旧格式，向后兼容）
pub fn get_games_path() -> PathBuf {
    path_manager::games_file()
}

/// 获取新格式游戏文件路径
pub fn get_games_only_path() -> PathBuf {
    path_manager::games_only_file()
}

/// 获取排名文件路径
pub fn get_rankings_path() -> PathBuf {
    path_manager::rankings_file()
}

/// 获取设置文件路径
pub fn get_settings_path() -> PathBuf {
    path_manager::settings_file()
}

/// 原子写入 JSON 到目标文件（先写 .tmp 再 rename）
fn atomic_write(path: &PathBuf, content: &str) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "创建数据目录失败", e))?;
    }
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, content)
        .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "写入数据失败", e))?;
    fs::rename(&tmp_path, path)
        .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "保存数据失败", e))?;
    Ok(())
}

fn read_json_file<T: serde::de::DeserializeOwned + Default>(path: &PathBuf) -> Result<T, AppError> {
    if !path.exists() {
        return Ok(T::default());
    }
    let content = fs::read_to_string(path)
        .map_err(|e| AppError::wrap(ErrorCode::DataReadFailed, "读取数据文件失败", e))?;
    serde_json::from_str(&content)
        .map_err(|e| AppError::wrap(ErrorCode::DataDeserializeFailed, "解析数据失败", e))
}

/// 迁移旧 game_list.json 到新的 games.json + rankings.json 分文件格式
fn migrate_if_needed() -> Result<(), AppError> {
    let new_games = get_games_only_path();
    let new_rankings = get_rankings_path();
    let old_file = get_games_path();

    // 只有两个新格式文件都存在时，才跳过迁移
    if new_games.exists() && new_rankings.exists() {
        return Ok(());
    }

    // 如果旧文件不存在，跳过
    if !old_file.exists() {
        return Ok(());
    }

    info!("检测到旧格式 game_list.json，正在迁移为 games.json + rankings.json...");

    let content = fs::read_to_string(&old_file)
        .map_err(|e| AppError::wrap(ErrorCode::DataReadFailed, "读取旧数据文件失败", e))?;
    let data: AppData = serde_json::from_str(&content)
        .map_err(|e| AppError::wrap(ErrorCode::DataDeserializeFailed, "解析旧数据失败", e))?;

    // 写入新格式文件
    let games_json = serde_json::to_string_pretty(&data.games)
        .map_err(|e| AppError::wrap(ErrorCode::DataSerializeFailed, "序列化游戏数据失败", e))?;
    atomic_write(&new_games, &games_json)?;

    let rankings_json = serde_json::to_string_pretty(&data.rankings)
        .map_err(|e| AppError::wrap(ErrorCode::DataSerializeFailed, "序列化排名数据失败", e))?;
    atomic_write(&new_rankings, &rankings_json)?;

    // 迁移成功后删除旧文件
    if let Err(e) = fs::remove_file(&old_file) {
        log::warn!("删除旧 game_list.json 失败: {}", e);
    }

    info!(
        "迁移完成: {} 个游戏 -> games.json, {} 个排名 -> rankings.json",
        data.games.len(),
        data.rankings.len()
    );
    Ok(())
}

/// 读取完整游戏数据（合并 games.json + rankings.json）
pub fn read_games() -> Result<AppData, AppError> {
    // 自动迁移旧格式
    migrate_if_needed()?;

    let games: Vec<Game> = read_json_file(&get_games_only_path())?;
    let rankings: Vec<Ranking> = read_json_file(&get_rankings_path())?;

    Ok(AppData { games, rankings })
}

/// 写入完整游戏数据到两个文件（用于备份/恢复等场景）
pub fn write_games(data: &AppData) -> Result<(), AppError> {
    write_games_only(&data.games)?;
    write_rankings_only(&data.rankings)?;
    Ok(())
}

/// 仅写入游戏数据到 games.json
pub fn write_games_only(games: &[Game]) -> Result<(), AppError> {
    let content = serde_json::to_string_pretty(games)
        .map_err(|e| AppError::wrap(ErrorCode::DataSerializeFailed, "序列化游戏数据失败", e))?;
    atomic_write(&get_games_only_path(), &content)
}

/// 仅写入排名数据到 rankings.json
pub fn write_rankings_only(rankings: &[Ranking]) -> Result<(), AppError> {
    let content = serde_json::to_string_pretty(rankings)
        .map_err(|e| AppError::wrap(ErrorCode::DataSerializeFailed, "序列化排名数据失败", e))?;
    atomic_write(&get_rankings_path(), &content)
}

/// 读取设置
pub fn read_settings() -> Result<Settings, AppError> {
    read_json_file(&get_settings_path())
}

/// 写入设置
pub fn write_settings(settings: &Settings) -> Result<(), AppError> {
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| AppError::wrap(ErrorCode::DataSerializeFailed, "序列化设置失败", e))?;
    let path = get_settings_path();
    atomic_write(&path, &content)
}
