use crate::data_store;
use crate::error::{to_frontend_error, AppError, ErrorCode};
use crate::models::{AppData, Game, Settings};
use crate::utils;
use chrono::Local;
use log::{debug, info, warn};
use rfd::FileDialog;
use rodio;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tauri::Manager;
use tauri::{AppHandle, Emitter};

// ======================== 游戏数据缓存 ========================

struct CacheState {
    games: Option<AppData>,
}

static CACHE: OnceLock<Mutex<CacheState>> = OnceLock::new();

fn get_cache() -> &'static Mutex<CacheState> {
    CACHE.get_or_init(|| Mutex::new(CacheState { games: None }))
}

fn get_cached_games() -> Result<AppData, AppError> {
    let mut cache = get_cache().lock().unwrap_or_else(|e| e.into_inner());
    if cache.games.is_none() {
        let data = data_store::read_games()?;
        cache.games = Some(data.clone());
        Ok(data)
    } else {
        Ok(cache.games.clone().unwrap())
    }
}

fn update_cache(data: &AppData) {
    if let Ok(mut cache) = get_cache().lock() {
        cache.games = Some(data.clone());
    }
}

// ======================== 封面处理辅助 ========================

/// 处理封面字段：如果是 Base64 data URI 则保存为文件并返回文件名
fn process_cover(game_id: &str, cover_field: &str) -> Result<String, AppError> {
    if cover_field.is_empty() {
        return Ok(String::new());
    }
    if cover_field.starts_with("data:") {
        // Base64 → 文件
        crate::path_manager::save_cover_file(game_id, cover_field)
    } else {
        // 已经是文件名
        Ok(cover_field.to_string())
    }
}

// ======================== Tauri 异步命令 ========================

/// 获取所有游戏列表
#[tauri::command]
pub async fn get_games() -> Result<AppData, String> {
    tauri::async_runtime::spawn_blocking(|| get_cached_games())
        .await
        .unwrap_or_else(|e| {
            Err(AppError::new(
                ErrorCode::InternalError,
                format!("任务执行失败: {}", e),
            ))
        })
        .map_err(to_frontend_error)
}

/// 批量获取封面 Base64 data URI（{ game_id: data_uri }）
#[tauri::command]
pub async fn get_game_covers(game_ids: Vec<String>) -> Result<HashMap<String, String>, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<HashMap<String, String>, AppError> {
        Ok(crate::path_manager::read_covers_batch(&game_ids))
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 添加游戏
#[tauri::command]
pub async fn add_game(_app_handle: AppHandle, mut game: Game) -> Result<AppData, String> {
    let game_id = game.id.clone();
    let game_name = game.name.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<AppData, AppError> {
        utils::validate_path(&game.path)?;
        // 处理封面：Base64 → 文件
        game.cover = process_cover(&game.id, &game.cover)?;
        let mut data = get_cached_games()?;
        if data
            .games
            .iter()
            .any(|g| g.path == game.path && g.name == game.name)
        {
            warn!("添加了可能重复的游戏: {} ({})", game.name, game.path);
        }
        data.games.push(game);
        data_store::write_games_only(&data.games)?;
        update_cache(&data);
        crate::logger::log_game_add(&game_id, &game_name);
        info!("游戏已添加, 总数: {}", data.games.len());
        Ok(data)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 更新游戏
#[tauri::command]
pub async fn update_game(mut game: Game) -> Result<AppData, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<AppData, AppError> {
        utils::validate_path(&game.path)?;
        // 处理封面：Base64 → 文件
        game.cover = process_cover(&game.id, &game.cover)?;
        let mut data = get_cached_games()?;
        let pos = data
            .games
            .iter()
            .position(|g| g.id == game.id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "游戏不存在"))?;
        // 如果封面文件名变更，删除旧封面文件
        let old_cover = data.games[pos].cover.clone();
        if !old_cover.is_empty() && old_cover != game.cover {
            crate::path_manager::delete_cover(&old_cover);
        }
        data.games[pos] = game;
        data_store::write_games_only(&data.games)?;
        update_cache(&data);
        debug!("游戏已更新, id: {}", data.games[pos].id);
        Ok(data)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 删除游戏
#[tauri::command]
pub async fn delete_game(id: String) -> Result<AppData, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<AppData, AppError> {
        let mut data = get_cached_games()?;
        let count_before = data.games.len();
        // 删除封面文件
        if let Some(game) = data.games.iter().find(|g| g.id == id) {
            crate::path_manager::delete_cover(&game.cover);
        }
        data.games.retain(|g| g.id != id);
        if data.games.len() == count_before {
            return Err(AppError::new(ErrorCode::GameNotFound, "游戏不存在"));
        }
        data_store::write_games_only(&data.games)?;
        update_cache(&data);
        info!("游戏已删除, id: {}", id);
        Ok(data)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 启动游戏（含后台进程监控，退出时自动更新游玩时间和最后游玩日期）
#[tauri::command]
pub async fn launch_game(
    app_handle: AppHandle,
    game_id: String,
    path: String,
    args: Option<Vec<String>>,
) -> Result<(), String> {
    let path_clone = path.clone();
    let (mut child, start_time) = tauri::async_runtime::spawn_blocking(move || {
        let args_ref = args.as_ref().map(|v| v.as_slice());
        crate::game_launcher::spawn_game_process(&path_clone, args_ref)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)?;

    crate::game_launcher::register_running_path(&path);
    crate::game_launcher::register_running_game(&game_id, &path);

    let handle2 = app_handle.clone();
    let path2 = path.clone();
    let game_id2 = game_id.clone();
    tauri::async_runtime::spawn(async move {
        info!("开始监控游戏进程: {} ({})", game_id2, path2);
        let exit_status = child.wait();
        let elapsed = start_time.elapsed();
        let minutes = (elapsed.as_secs() / 60).max(1);

        info!(
            "游戏已退出: {} ({}) - 运行了 {} 分钟, exit: {:?}",
            game_id2, path2, minutes, exit_status
        );

        crate::game_launcher::unregister_running_path(&path2);
        crate::game_launcher::unregister_running_game(&game_id2);

        let update_result: Result<(), AppError> = (|| {
            let mut data = data_store::read_games()?;
            if let Some(game) = data.games.iter_mut().find(|g| g.id == game_id2) {
                game.play_time += minutes;
                game.last_play = Some(Local::now().format("%Y-%m-%d").to_string());
                if game.status == "未游玩" {
                    game.status = "游玩中".to_string();
                }
                data_store::write_games_only(&data.games)?;
                update_cache(&data);
            }
            Ok(())
        })();

        let event_payload = match &update_result {
            Ok(_) => serde_json::json!({
                "game_id": game_id2,
                "play_time_added": minutes,
                "updated": true
            }),
            Err(e) => serde_json::json!({
                "game_id": game_id2,
                "updated": false,
                "error": e.to_string()
            }),
        };
        let _ = handle2.emit("game-exited", event_payload);
    });

    Ok(())
}

/// 扫描文件夹获取 exe 列表
#[tauri::command]
pub async fn scan_folder(folder: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || utils::scan_exe_files(&folder))
        .await
        .unwrap_or_else(|e| {
            Err(AppError::new(
                ErrorCode::InternalError,
                format!("任务执行失败: {}", e),
            ))
        })
        .map_err(to_frontend_error)
}

/// 获取运行中的游戏列表
#[tauri::command]
pub async fn get_running_games() -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(|| crate::game_launcher::get_running_games())
        .await
        .unwrap_or_else(|e| {
            Err(AppError::new(
                ErrorCode::InternalError,
                format!("任务执行失败: {}", e),
            ))
        })
        .map_err(to_frontend_error)
}

/// 结束游戏进程
#[tauri::command]
pub async fn kill_game(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || crate::game_launcher::kill_game(&path))
        .await
        .unwrap_or_else(|e| {
            Err(AppError::new(
                ErrorCode::InternalError,
                format!("任务执行失败: {}", e),
            ))
        })
        .map_err(to_frontend_error)
}

/// 文件选择对话框
#[tauri::command]
pub fn open_file_dialog(
    title: Option<String>,
    extensions: Option<Vec<String>>,
) -> Result<Option<String>, String> {
    let mut dialog = FileDialog::new();
    if let Some(t) = title {
        dialog = dialog.set_title(&t);
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

/// 复制封面图片到程序数据目录，返回文件名
#[tauri::command]
pub async fn copy_cover(source_path: String, game_id: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<String, AppError> {
        let canonical = utils::validate_path(&source_path)?;
        if !canonical.exists() {
            return Err(AppError::new(ErrorCode::FileNotFound, "源文件不存在"));
        }
        let img_data = fs::read(&canonical)
            .map_err(|e| AppError::wrap(ErrorCode::FileReadFailed, "读取图片文件失败", e))?;
        // 转为 Base64 data URI 然后统一通过 save_cover_file 保存
        let data_uri = crate::bangumi::bytes_to_data_uri(&img_data);
        crate::path_manager::save_cover_file(&game_id, &data_uri)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 备份数据到 %APPDATA%/KurisuGal/Backup/
#[tauri::command]
pub async fn backup_data() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<String, AppError> {
        let start = std::time::Instant::now();
        let backup_dir = crate::path_manager::backup_dir();
        fs::create_dir_all(&backup_dir)
            .map_err(|e| AppError::wrap(ErrorCode::BackupFailed, "创建备份目录失败", e))?;

        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_file = backup_dir.join(format!("kurisu_gal_backup_{}.json", timestamp));

        let mut all_data = serde_json::Map::new();
        let games_path = crate::path_manager::games_file();
        let games_content = fs::read_to_string(&games_path).unwrap_or_default();
        all_data.insert(
            "games".to_string(),
            serde_json::from_str(&games_content).unwrap_or(json!({})),
        );
        let settings_path = crate::path_manager::settings_file();
        let settings_content = fs::read_to_string(&settings_path).unwrap_or_default();
        all_data.insert(
            "settings".to_string(),
            serde_json::from_str(&settings_content).unwrap_or(json!({})),
        );

        let backup_json = serde_json::to_string_pretty(&all_data)
            .map_err(|e| AppError::wrap(ErrorCode::BackupFailed, "序列化备份数据失败", e))?;
        fs::write(&backup_file, &backup_json)
            .map_err(|e| AppError::wrap(ErrorCode::BackupFailed, "写入备份文件失败", e))?;

        info!(
            "数据备份完成: {}, 耗时 {}ms",
            backup_file.display(),
            start.elapsed().as_millis()
        );
        Ok(backup_file.to_string_lossy().to_string())
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 恢复数据
#[tauri::command]
pub async fn restore_data(file_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<(), AppError> {
        let canonical = utils::validate_path(&file_path)?;
        let content = fs::read_to_string(&canonical)
            .map_err(|e| AppError::wrap(ErrorCode::RestoreFailed, "读取备份文件失败", e))?;
        let data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| AppError::wrap(ErrorCode::RestoreFailed, "解析备份文件失败", e))?;

        if let Some(games) = data.get("games") {
            let games_data: AppData = serde_json::from_value(games.clone())
                .map_err(|e| AppError::wrap(ErrorCode::RestoreFailed, "解析游戏数据失败", e))?;
            data_store::write_games(&games_data)?;
            update_cache(&games_data);
        }
        if let Some(settings) = data.get("settings") {
            let settings_data: Settings = serde_json::from_value(settings.clone())
                .map_err(|e| AppError::wrap(ErrorCode::RestoreFailed, "解析设置数据失败", e))?;
            data_store::write_settings(&settings_data)?;
        }
        info!("数据恢复完成");
        Ok(())
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 清理路径不存在的无效游戏
#[tauri::command]
pub async fn cleanup_invalid() -> Result<AppData, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<AppData, AppError> {
        let mut data = get_cached_games()?;
        let before = data.games.len();
        data.games.retain(|g| Path::new(&g.path).exists());
        let removed = before - data.games.len();
        if removed > 0 {
            data_store::write_games_only(&data.games)?;
            update_cache(&data);
            info!(
                "清理无效数据完成, 移除了 {} 条, 剩余 {}",
                removed,
                data.games.len()
            );
        }
        Ok(data)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 获取设置
#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    tauri::async_runtime::spawn_blocking(|| crate::settings::load_settings())
        .await
        .unwrap_or_else(|e| {
            Err(AppError::new(
                ErrorCode::InternalError,
                format!("任务执行失败: {}", e),
            ))
        })
        .map_err(to_frontend_error)
}

/// 保存设置
#[tauri::command]
pub async fn save_settings(app_handle: AppHandle, settings: Settings) -> Result<(), String> {
    let handle = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || crate::settings::save_settings(&handle, &settings))
        .await
        .unwrap_or_else(|e| {
            Err(AppError::new(
                ErrorCode::InternalError,
                format!("任务执行失败: {}", e),
            ))
        })
        .map_err(to_frontend_error)
}

/// 设置开机自启
#[tauri::command]
pub async fn set_startup(app_handle: AppHandle, enabled: bool) -> Result<(), String> {
    let handle = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), AppError> {
        let mut settings = crate::settings::load_settings()?;
        settings.startup = enabled;
        crate::settings::save_settings(&handle, &settings)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 批量移动游戏到指定分类
#[tauri::command]
pub async fn batch_update_category(
    game_ids: Vec<String>,
    category: String,
) -> Result<AppData, String> {
    let cat = category.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<AppData, AppError> {
        let mut data = get_cached_games()?;
        let ids_set: std::collections::HashSet<&str> =
            game_ids.iter().map(|s| s.as_str()).collect();
        let mut count = 0usize;
        for game in data.games.iter_mut() {
            if ids_set.contains(game.id.as_str()) {
                game.category = cat.clone();
                count += 1;
            }
        }
        data_store::write_games_only(&data.games)?;
        update_cache(&data);
        info!("批量移动 {} 个游戏到分类: {}", count, category);
        Ok(data)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 快捷更新游戏状态
#[tauri::command]
pub async fn quick_update_status(game_id: String, status: String) -> Result<AppData, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<AppData, AppError> {
        let mut data = get_cached_games()?;
        let mut game_found = false;
        if let Some(game) = data.games.iter_mut().find(|g| g.id == game_id) {
            game.status = status;
            game_found = true;
        }
        if !game_found {
            return Err(AppError::new(ErrorCode::GameNotFound, "游戏不存在"));
        }
        data_store::write_games_only(&data.games)?;
        update_cache(&data);
        Ok(data)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

// ======================== 数据路径管理命令 ========================

/// 获取当前数据根目录路径
#[tauri::command]
pub fn get_data_root() -> String {
    crate::path_manager::data_root_str()
}

/// 设置数据根目录并迁移数据
#[tauri::command]
pub async fn set_data_root(path: String, migrate: Option<bool>) -> Result<String, String> {
    let should_migrate = migrate.unwrap_or(false);
    tauri::async_runtime::spawn_blocking(move || -> Result<String, AppError> {
        if should_migrate {
            let report = crate::path_manager::set_data_root_with_migrate(&path)?;
            info!(
                "数据迁移完成: {}个文件, {}张封面, {:.1}MB, 错误: {}",
                report.files_count,
                report.covers_count,
                report.total_mb(),
                report.errors.len()
            );
        } else {
            crate::path_manager::set_data_root(&path)?;
        }
        // 清空缓存
        get_cache().lock().unwrap_or_else(|e| e.into_inner()).games = None;
        Ok(crate::path_manager::data_root_str())
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 获取数据目录大小信息（供确认对话框展示）
#[tauri::command]
pub fn get_data_size_info() -> serde_json::Value {
    crate::path_manager::get_data_size_info()
}

/// 文件夹选择对话框
#[tauri::command]
pub fn pick_folder(title: Option<String>) -> Result<Option<String>, String> {
    let mut dialog = FileDialog::new();
    if let Some(t) = title {
        dialog = dialog.set_title(&t);
    }
    match dialog.pick_folder() {
        Some(path) => Ok(Some(path.to_string_lossy().to_string())),
        None => Ok(None),
    }
}

// ======================== Bangumi API 命令 ========================

/// 在 Bangumi 上搜索游戏
#[tauri::command]
pub async fn search_bangumi(
    keyword: String,
) -> Result<Vec<crate::bangumi::BangumiSearchItem>, String> {
    let start = std::time::Instant::now();
    crate::logger::log_bgm_search_start(&keyword);
    match crate::bangumi::search_games(&keyword).await {
        Ok(results) => {
            crate::logger::log_bgm_search_done(
                &keyword,
                results.len(),
                start.elapsed().as_millis(),
            );
            Ok(results)
        }
        Err(e) => {
            crate::logger::log_bgm_search_error(&keyword, &e, start.elapsed().as_millis());
            Err(e)
        }
    }
}

/// 获取 Bangumi 游戏详情（含封面 Base64），返回精简数据供前端填充
#[tauri::command]
pub async fn fetch_bangumi_game(
    subject_id: u32,
    keyword: Option<String>,
) -> Result<crate::models::BangumiFillData, String> {
    let start = std::time::Instant::now();
    crate::logger::log_bgm_detail_start(subject_id);

    // === 获取详情（最多 2 次重试） ===
    let detail = 'retry_detail: {
        let mut last_err = String::new();
        for attempt in 1..=2 {
            match crate::bangumi::get_game_detail(subject_id).await {
                Ok(d) => break 'retry_detail Ok(d),
                Err(e) => {
                    last_err = e;
                    if attempt < 2 {
                        log::warn!(
                            "获取详情失败 (第 {} 次, id={}): {}——0.8s 后重试…",
                            attempt,
                            subject_id,
                            last_err
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    }
                }
            }
        }
        // 详情 API 失败，尝试搜索回退
        if let Some(ref kw) = keyword {
            log::info!(
                "详情 API 失败 (id={})，尝试通过搜索「{}」回退获取数据",
                subject_id,
                kw
            );
            match crate::bangumi::search_games(kw).await {
                Ok(results) => {
                    if let Some(item) = results.iter().find(|i| i.id == subject_id) {
                        let detail = crate::bangumi::BangumiGameDetail {
                            id: item.id,
                            name: item.name.clone(),
                            name_cn: item.name_cn.clone(),
                            summary: item.summary.clone(),
                            image: item.image.clone(),
                            image_large: item.image_large.clone(),
                            date: item.date.clone(),
                            score: item.score,
                            rank: item.rank,
                            tags: vec![],
                            platform: String::new(),
                        };
                        break 'retry_detail Ok(detail);
                    }
                }
                Err(e) => log::warn!("搜索回退也失败 (id={}, kw={}): {}", subject_id, kw, e),
            }
        }
        crate::logger::log_bgm_detail_error(subject_id, &last_err, start.elapsed().as_millis());
        Err(last_err)
    };

    let detail = match detail {
        Ok(d) => d,
        Err(e) => return Err(e),
    };

    crate::logger::log_bgm_detail_done(
        subject_id,
        if !detail.name_cn.is_empty() {
            &detail.name_cn
        } else {
            &detail.name
        },
        start.elapsed().as_millis(),
    );

    // === 下载封面（最多 2 次重试；封面失败不致命） ===
    let cover_url = if !detail.image_large.is_empty() {
        &detail.image_large
    } else {
        &detail.image
    };
    let cover = if !cover_url.is_empty() {
        let mut last_err = String::new();
        let mut result = String::new();
        for attempt in 1..=2 {
            match crate::bangumi::download_cover_as_base64(cover_url).await {
                Ok(bytes) => {
                    result = crate::bangumi::bytes_to_data_uri(&bytes);
                    break;
                }
                Err(e) => {
                    last_err = e;
                    if attempt < 2 {
                        log::warn!(
                            "下载封面失败 (第 {} 次): {}——0.8s后重试…",
                            attempt,
                            last_err
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    }
                }
            }
        }
        if result.is_empty() && !last_err.is_empty() {
            log::warn!("下载封面最终失败 (id={}): {}", subject_id, last_err);
            crate::logger::log_bgm_cover_error(cover_url, &last_err);
        }
        if !result.is_empty() {
            crate::logger::log_bgm_cover_done(
                subject_id,
                result.len(),
                start.elapsed().as_millis(),
            );
        }
        result
    } else {
        String::new()
    };

    let fill = crate::models::BangumiFillData {
        name: detail.name,
        name_cn: detail.name_cn,
        summary: detail.summary,
        cover,
        tags: detail.tags,
        date: detail.date,
    };

    crate::logger::log_bgm_fill(
        subject_id,
        if !fill.name_cn.is_empty() {
            &fill.name_cn
        } else {
            &fill.name
        },
        !fill.cover.is_empty(),
        fill.tags.len(),
    );

    Ok(fill)
}

/// 单独下载 Bangumi 封面，返回 Base64 data URI
#[tauri::command]
pub async fn download_bangumi_cover(image_url: String) -> Result<String, String> {
    crate::bangumi::download_cover_as_base64(&image_url)
        .await
        .map(|bytes| crate::bangumi::bytes_to_data_uri(&bytes))
}

/// 向排名中添加 Bangumi 虚拟游戏（不进入全局游戏库，仅排名内有效）
#[tauri::command]
pub async fn add_rank_virtual_game(
    ranking_id: String,
    mut game: Game,
) -> Result<crate::models::Ranking, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<crate::models::Ranking, AppError> {
        game.cover = process_cover(&game.id, &game.cover)?;
        let mut data = get_cached_games()?;
        let ranking = data
            .rankings
            .iter_mut()
            .find(|r| r.id == ranking_id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "排名不存在"))?;
        // 去重
        if ranking
            .virtual_games
            .iter()
            .any(|g| g.id == game.id || g.name == game.name)
        {
            return Err(AppError::new(ErrorCode::InvalidInput, "该游戏已在此排名中"));
        }
        ranking.virtual_games.push(game);
        ranking.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = ranking.clone();
        data_store::write_rankings_only(&data.rankings)?;
        update_cache(&data);
        info!("向排名 {} 添加虚拟游戏成功", ranking_id);
        Ok(result)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 从排名中删除虚拟游戏（同时从所有等级中移除）
#[tauri::command]
pub async fn remove_rank_virtual_game(
    ranking_id: String,
    game_id: String,
) -> Result<crate::models::Ranking, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<crate::models::Ranking, AppError> {
        let mut data = get_cached_games()?;
        let ranking = data
            .rankings
            .iter_mut()
            .find(|r| r.id == ranking_id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "排名不存在"))?;
        ranking.virtual_games.retain(|g| g.id != game_id);
        // 从所有等级中移除该游戏
        for level in &mut ranking.levels {
            level.game_ids.retain(|id| id != &game_id);
        }
        ranking.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = ranking.clone();
        data_store::write_rankings_only(&data.rankings)?;
        update_cache(&data);
        info!("从排名 {} 删除虚拟游戏 {}", ranking_id, game_id);
        Ok(result)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

// ======================== 截图管理 ========================

/// 添加截图：将图片文件复制到游戏截图目录，并生成缩略图
#[tauri::command]
pub async fn add_screenshot(game_id: String, source_path: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<String>, AppError> {
        use crate::path_manager::{game_screenshots_dir, screenshot_path, screenshot_thumb_path};

        let source = Path::new(&source_path);
        if !source.exists() {
            return Err(AppError::new(ErrorCode::PathInvalid, "源图片文件不存在"));
        }

        let dir = game_screenshots_dir(&game_id);
        fs::create_dir_all(&dir)
            .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "创建截图目录失败", e))?;

        // 生成唯一文件名
        let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("png");
        let timestamp = Local::now().format("%Y%m%d%H%M%S%3f").to_string();
        let filename = format!("{}_{}.{}", game_id, timestamp, ext);

        let dest = screenshot_path(&game_id, &filename);
        fs::copy(source, &dest)
            .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "复制截图文件失败", e))?;

        // 生成缩略图（最大宽度 300px，JPEG）——必须在返回前完成
        let thumb_dest = screenshot_thumb_path(&game_id, &filename);
        match generate_thumbnail(&dest, &thumb_dest) {
            Ok(()) => {
                let thumb_size = std::fs::metadata(&thumb_dest)
                    .map(|m| m.len())
                    .unwrap_or(0);
                let orig_size = std::fs::metadata(&dest)
                    .map(|m| m.len())
                    .unwrap_or(0);
                log::info!("缩略图已生成: {}", thumb_dest.display());
                #[cfg(debug_assertions)]
                eprintln!(
                    "[截图诊断] 添加截图完成 | 原图: {} ({} bytes) | 缩略图: {} ({} bytes) | 缩略图含_thumb: {}",
                    filename, orig_size,
                    thumb_dest.file_name().unwrap_or_default().to_string_lossy(),
                    thumb_size,
                    thumb_dest.to_string_lossy().contains("_thumb")
                );
            }
            Err(e) => {
                log::warn!("生成缩略图失败 ({}): {}", filename, e);
                #[cfg(debug_assertions)]
                eprintln!("[截图诊断] 缩略图生成失败! 原图: {} | 错误: {}", filename, e);
            }
        }

        // 更新游戏数据中的截图列表
        let mut data = get_cached_games()?;
        if let Some(game) = data.games.iter_mut().find(|g| g.id == game_id) {
            game.screenshots.push(filename.clone());
            data_store::write_games_only(&data.games)?;
            update_cache(&data);
        }

        info!("截图已添加: {} -> {}", source_path, dest.display());
        #[cfg(debug_assertions)]
        eprintln!(
            "[截图诊断] 返回前端 | game_id={} | 原图文件名={} | 缩略图路径={} | _thumb验证={}",
            game_id,
            filename,
            thumb_dest.to_string_lossy(),
            thumb_dest.to_string_lossy().contains("_thumb")
        );
        Ok(vec![filename])
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 生成缩略图（最大宽度 300px，JPEG 格式，质量 60）
fn generate_thumbnail(source: &Path, dest: &Path) -> Result<(), AppError> {
    use std::io::Cursor;
    let img = image::open(source)
        .map_err(|e| AppError::wrap(ErrorCode::DataReadFailed, "读取原图失败", e))?;
    let orig_w = img.width();
    let orig_h = img.height();
    let thumb = if img.width() > 300 {
        let h = (300.0 / img.width() as f64 * img.height() as f64) as u32;
        img.resize_exact(300, h, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let mut buf = Cursor::new(Vec::new());
    // 使用 JPEG 编码器并设置较低质量以获得更小的文件
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 60);
    thumb
        .write_with_encoder(encoder)
        .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "编码缩略图失败", e))?;
    let data = buf.into_inner();
    let thumb_size = data.len();
    fs::write(dest, &data)
        .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "保存缩略图失败", e))?;
    #[cfg(debug_assertions)]
    eprintln!(
        "[截图诊断] 缩略图生成 | 原图尺寸: {}x{} | 缩略图尺寸: {}x{} | 缩略图大小: {} bytes | 目标: {}",
        orig_w, orig_h,
        thumb.width(), thumb.height(),
        thumb_size,
        dest.file_name().unwrap_or_default().to_string_lossy()
    );
    Ok(())
}

/// 获取游戏的截图列表（返回文件名数组）
#[tauri::command]
pub async fn list_screenshots(game_id: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<String>, AppError> {
        let data = get_cached_games()?;
        let game = data
            .games
            .iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "游戏不存在"))?;
        Ok(game.screenshots.clone())
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 获取截图列表及其缩略图路径（一次调用，避免前端多次请求）
/// 确保返回的 thumb_path 100% 是缩略图（文件名含 _thumb 后缀且文件真实存在）
#[tauri::command]
pub async fn list_screenshots_with_paths(
    game_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let gid = game_id.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<serde_json::Value>, AppError> {
        let data = get_cached_games()?;
        let game = data
            .games
            .iter()
            .find(|g| g.id == gid)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "游戏不存在"))?;

        #[cfg(debug_assertions)]
        eprintln!("[截图诊断] === list_screenshots_with_paths 开始 | game_id={} | 截图总数={} ===", gid, game.screenshots.len());

        let mut result = Vec::new();
        for (idx, filename) in game.screenshots.iter().enumerate() {
            let thumb_path = crate::path_manager::screenshot_thumb_path(&gid, filename);
            let thumb_str = thumb_path.to_string_lossy().to_string();
            let thumb_name = thumb_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // 【安全检查1】验证缩略图文件名是否以 _thumb 结尾（不含扩展名时）或包含 _thumb
            let has_thumb_marker = thumb_name.contains("_thumb");
            if !has_thumb_marker {
                log::error!("[截图] 缩略图路径缺少 _thumb 后缀: {}", thumb_str);
                #[cfg(debug_assertions)]
                eprintln!("[截图诊断] [{}/{}] 跳过! 缩略图名不含_thumb: {}", idx + 1, game.screenshots.len(), thumb_name);
                continue;
            }

            // 缩略图不存在时自动生成
            if !thumb_path.exists() {
                let src = crate::path_manager::screenshot_path(&gid, filename);
                if src.exists() {
                    log::info!("[截图] 旧数据自动生成缩略图: {} -> {}", src.display(), thumb_str);
                    #[cfg(debug_assertions)]
                    eprintln!("[截图诊断] [{}/{}] 补生成缩略图: {} -> {}", idx + 1, game.screenshots.len(), filename, thumb_name);
                    if let Err(e) = generate_thumbnail(&src, &thumb_path) {
                        log::warn!("[截图] 自动生成缩略图失败 ({}): {}", filename, e);
                        #[cfg(debug_assertions)]
                        eprintln!("[截图诊断] [{}/{}] 补生成失败: {} | {}", idx + 1, game.screenshots.len(), filename, e);
                    }
                }
            }

            // 【安全检查2】最终验证：确保缩略图文件确实存在
            if !thumb_path.exists() {
                log::error!("[截图] 缩略图不存在，跳过: {}", thumb_str);
                #[cfg(debug_assertions)]
                eprintln!("[截图诊断] [{}/{}] 跳过! 缩略图文件不存在: {}", idx + 1, game.screenshots.len(), thumb_str);
                continue;
            }

            // 输出诊断日志
            let size = std::fs::metadata(&thumb_path).map(|m| m.len()).unwrap_or(0);
            log::info!("[截图] 返回缩略图: {} ({} bytes)", thumb_str, size);
            #[cfg(debug_assertions)]
            eprintln!(
                "[截图诊断] [{}/{}] 返回前端 => filename={} | thumb_path={} | size={}bytes | _thumb验证=OK",
                idx + 1, game.screenshots.len(),
                filename, thumb_str, size
            );

            result.push(serde_json::json!({
                "filename": filename,
                "thumb_path": thumb_str,
            }));
        }
        log::info!("[截图] 共返回 {} 个缩略图路径", result.len());
        #[cfg(debug_assertions)]
        eprintln!("[截图诊断] === list_screenshots_with_paths 结束 | 返回 {} 条 | 跳过 {} 条 ===", result.len(), game.screenshots.len() - result.len());
        Ok(result)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 获取截图列表及其缩略图 Base64 data URI（一次调用，前端无需 convertFileSrc，性能最优）
/// 返回 [{filename, thumb_data_uri}]，thumb_data_uri 格式为 "data:image/jpeg;base64,..."
#[tauri::command]
pub async fn list_screenshots_with_thumbs(
    game_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let gid = game_id.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<serde_json::Value>, AppError> {
        let data = get_cached_games()?;
        let game = data
            .games
            .iter()
            .find(|g| g.id == gid)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "游戏不存在"))?;

        #[cfg(debug_assertions)]
        eprintln!("[截图诊断] === list_screenshots_with_thumbs 开始 | game_id={} | 截图总数={} ===", gid, game.screenshots.len());

        let mut result = Vec::new();
        for (idx, filename) in game.screenshots.iter().enumerate() {
            let thumb_path = crate::path_manager::screenshot_thumb_path(&gid, filename);

            // 缩略图不存在时自动生成
            if !thumb_path.exists() {
                let src = crate::path_manager::screenshot_path(&gid, filename);
                if src.exists() {
                    #[cfg(debug_assertions)]
                    eprintln!("[截图诊断] [{}/{}] 补生成缩略图: {}", idx + 1, game.screenshots.len(), filename);
                    if let Err(e) = generate_thumbnail(&src, &thumb_path) {
                        log::warn!("[截图] 自动生成缩略图失败 ({}): {}", filename, e);
                        #[cfg(debug_assertions)]
                        eprintln!("[截图诊断] [{}/{}] 补生成失败，跳过: {}", idx + 1, game.screenshots.len(), filename);
                        continue;
                    }
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!("[截图诊断] [{}/{}] 原图不存在，跳过: {}", idx + 1, game.screenshots.len(), filename);
                    continue;
                }
            }

            // 再次确认缩略图存在
            if !thumb_path.exists() {
                #[cfg(debug_assertions)]
                eprintln!("[截图诊断] [{}/{}] 缩略图仍不存在，跳过: {}", idx + 1, game.screenshots.len(), filename);
                continue;
            }

            // 读取缩略图并编码为 Base64 data URI
            let thumb_bytes = match fs::read(&thumb_path) {
                Ok(b) => b,
                Err(e) => {
                    log::warn!("[截图] 读取缩略图失败 ({}): {}", filename, e);
                    continue;
                }
            };
            let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &thumb_bytes);
            let data_uri = format!("data:image/jpeg;base64,{}", b64);

            #[cfg(debug_assertions)]
            {
                let thumb_name = thumb_path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                eprintln!(
                    "[截图诊断] [{}/{}] 返回前端 Base64 | filename={} | thumb={} | raw={}bytes | b64={}chars",
                    idx + 1, game.screenshots.len(),
                    filename, thumb_name,
                    thumb_bytes.len(), b64.len()
                );
            }

            result.push(serde_json::json!({
                "filename": filename,
                "thumb_data_uri": data_uri,
            }));
        }
        #[cfg(debug_assertions)]
        eprintln!("[截图诊断] === list_screenshots_with_thumbs 结束 | 返回 {} 条 ===", result.len());
        Ok(result)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 获取截图文件路径（前端用 convertFileSrc 加载）
#[tauri::command]
pub fn get_screenshot_path(game_id: String, filename: String) -> Result<String, String> {
    let path = crate::path_manager::screenshot_path(&game_id, &filename);
    Ok(path.to_string_lossy().to_string())
}

/// 获取截图缩略图路径（前端用 convertFileSrc 加载），缩略图不存在时自动生成
#[tauri::command]
pub async fn get_screenshot_thumb_path(
    game_id: String,
    filename: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<String, AppError> {
        let thumb = crate::path_manager::screenshot_thumb_path(&game_id, &filename);
        if !thumb.exists() {
            let src = crate::path_manager::screenshot_path(&game_id, &filename);
            if src.exists() {
                generate_thumbnail(&src, &thumb).unwrap_or_else(|e| {
                    log::warn!("自动生成缩略图失败 ({}): {}", filename, e);
                });
            }
        }
        Ok(thumb.to_string_lossy().to_string())
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 获取截图的 Base64 data URI（仅用于预览大图时加载原图）
#[tauri::command]
pub async fn get_screenshot_base64(game_id: String, filename: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<String, AppError> {
        let path = crate::path_manager::screenshot_path(&game_id, &filename);
        let bytes = fs::read(&path)
            .map_err(|e| AppError::wrap(ErrorCode::DataReadFailed, "读取截图文件失败", e))?;
        let ext = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_lowercase();
        let mime = match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            _ => "image/png",
        };
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        Ok(format!("data:{};base64,{}", mime, b64))
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 删除指定截图
#[tauri::command]
pub async fn delete_screenshot(game_id: String, filename: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<String>, AppError> {
        let path = crate::path_manager::screenshot_path(&game_id, &filename);
        let thumb_path = crate::path_manager::screenshot_thumb_path(&game_id, &filename);
        if path.exists() {
            if let Err(e) = fs::remove_file(&path) {
                log::warn!("删除截图文件失败 ({}): {}", filename, e);
            }
        } else {
            log::info!("截图文件不存在，跳过文件删除: {}", filename);
        }
        // 同时删除缩略图
        if thumb_path.exists() {
            if let Err(e) = fs::remove_file(&thumb_path) {
                log::warn!("删除缩略图文件失败: {}", e);
            }
        }
        // 无论文件删除成功与否，都从游戏数据中移除引用
        let mut data = get_cached_games()?;
        if let Some(game) = data.games.iter_mut().find(|g| g.id == game_id) {
            game.screenshots.retain(|s| s != &filename);
            data_store::write_games_only(&data.games)?;
            update_cache(&data);
        }
        Ok(vec![])
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 切换收藏状态
#[tauri::command]
pub async fn toggle_favorite(game_id: String) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<bool, AppError> {
        let mut data = get_cached_games()?;
        let game = data
            .games
            .iter_mut()
            .find(|g| g.id == game_id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "游戏不存在"))?;
        game.favorite = !game.favorite;
        let new_state = game.favorite;
        data_store::write_games_only(&data.games)?;
        update_cache(&data);
        info!("收藏状态已切换: {} -> {}", game_id, new_state);
        Ok(new_state)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 获取首页统计数据
#[tauri::command]
pub async fn get_home_stats() -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(|| -> Result<serde_json::Value, AppError> {
        let data = get_cached_games()?;
        let games = &data.games;

        let total_games = games.len();
        let total_favorites = games.iter().filter(|g| g.favorite).count();
        let total_play_time: u64 = games.iter().map(|g| g.play_time).sum();

        let mut categories: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for g in games {
            *categories.entry(g.category.clone()).or_default() += 1;
        }
        let mut cat_dist: Vec<serde_json::Value> = categories
            .into_iter()
            .map(|(cat, count)| serde_json::json!({ "name": cat, "count": count }))
            .collect();
        cat_dist.sort_by(|a, b| b["count"].as_u64().cmp(&a["count"].as_u64()));

        let mut sorted: Vec<&Game> = games.iter().collect();
        sorted.sort_by(|a, b| b.play_time.cmp(&a.play_time));
        let top5: Vec<serde_json::Value> = sorted
            .iter()
            .take(5)
            .filter(|g| g.play_time > 0)
            .map(|g| {
                serde_json::json!({
                    "name": g.name,
                    "play_time": g.play_time,
                    "category": g.category,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "total_games": total_games,
            "total_favorites": total_favorites,
            "total_play_time": total_play_time,
            "category_distribution": cat_dist,
            "top5_playtime": top5,
        }))
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 获取自定义 Logo（返回 base64 data URI，无 logo 返回空字符串）
#[tauri::command]
pub async fn get_logo() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| -> Result<String, AppError> {
        let logo_path = crate::path_manager::logo_file();
        if !logo_path.exists() {
            return Ok(String::new());
        }
        let data = std::fs::read(&logo_path)
            .map_err(|e| AppError::wrap(ErrorCode::DataReadFailed, "读取 Logo 失败", e))?;
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
        let mime = match logo_path.extension().and_then(|e| e.to_str()) {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            _ => "image/png",
        };
        Ok(format!("data:{};base64,{}", mime, b64))
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 保存自定义 Logo（接收 base64 data URI）
#[tauri::command]
pub async fn save_logo(data_uri: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<(), AppError> {
        let assets_dir = crate::path_manager::assets_dir();
        std::fs::create_dir_all(&assets_dir)
            .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "创建资源目录失败", e))?;

        if data_uri.is_empty() {
            return Ok(());
        }

        // 移除旧 logo
        if let Ok(entries) = std::fs::read_dir(&assets_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("logo.") {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }

        let (mime, data) = crate::path_manager::parse_data_uri(&data_uri)?;
        let ext = match mime {
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => "jpg",
        };
        let filename = format!("logo.{}", ext);
        let filepath = assets_dir.join(&filename);
        std::fs::write(&filepath, &data)
            .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "保存 Logo 失败", e))?;

        info!("Logo 已保存: {}", filepath.display());
        Ok(())
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

// ======================== 排名管理 ========================

/// 返回程序图标 (base64 data URI)
#[tauri::command]
pub fn get_app_icon() -> Result<String, String> {
    let icon_data = include_bytes!("../icons/128x128.ico");
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, icon_data);
    Ok(format!("data:image/x-icon;base64,{}", b64))
}

/// 返回应用版本号 (从 Cargo.toml 编译时读取)
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn get_rankings() -> Result<Vec<crate::models::Ranking>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let data = get_cached_games()?;
        Ok(data.rankings)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

#[tauri::command]
pub async fn create_ranking(name: String) -> Result<crate::models::Ranking, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<crate::models::Ranking, AppError> {
        let id = utils::generate_id();
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let levels = vec![
            crate::models::RankLevel {
                level: 1,
                name: "夯".to_string(),
                game_ids: vec![],
                color: "rgb(255, 127, 127)".to_string(),
            },
            crate::models::RankLevel {
                level: 2,
                name: "顶级".to_string(),
                game_ids: vec![],
                color: "rgb(255, 191, 127)".to_string(),
            },
            crate::models::RankLevel {
                level: 3,
                name: "人上人".to_string(),
                game_ids: vec![],
                color: "rgb(255, 223, 127)".to_string(),
            },
            crate::models::RankLevel {
                level: 4,
                name: "NPC".to_string(),
                game_ids: vec![],
                color: "rgb(255, 255, 127)".to_string(),
            },
            crate::models::RankLevel {
                level: 5,
                name: "拉完了".to_string(),
                game_ids: vec![],
                color: "rgb(191, 255, 127)".to_string(),
            },
        ];
        let ranking = crate::models::Ranking {
            id,
            name,
            levels,
            created_at: now.clone(),
            updated_at: now,
            virtual_games: vec![],
        };
        let mut data = get_cached_games()?;
        data.rankings.push(ranking.clone());
        data_store::write_rankings_only(&data.rankings)?;
        update_cache(&data);
        Ok(ranking)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

#[tauri::command]
pub async fn update_ranking(
    ranking: crate::models::Ranking,
) -> Result<crate::models::Ranking, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<crate::models::Ranking, AppError> {
        let mut data = get_cached_games()?;
        let idx = data
            .rankings
            .iter()
            .position(|r| r.id == ranking.id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "排名不存在"))?;
        let mut updated = ranking.clone();
        updated.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        data.rankings[idx] = updated.clone();
        data_store::write_rankings_only(&data.rankings)?;
        update_cache(&data);
        Ok(updated)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

#[tauri::command]
pub async fn delete_ranking(id: String) -> Result<AppData, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<AppData, AppError> {
        let mut data = get_cached_games()?;
        let count_before = data.rankings.len();
        data.rankings.retain(|r| r.id != id);
        if data.rankings.len() == count_before {
            return Err(AppError::new(ErrorCode::GameNotFound, "排名不存在"));
        }
        data_store::write_rankings_only(&data.rankings)?;
        update_cache(&data);
        Ok(data)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

#[tauri::command]
pub async fn set_game_rank(
    ranking_id: String,
    game_id: String,
    level: i32,
) -> Result<crate::models::Ranking, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<crate::models::Ranking, AppError> {
        let mut data = get_cached_games()?;
        let ranking = data
            .rankings
            .iter_mut()
            .find(|r| r.id == ranking_id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "排名不存在"))?;

        for lvl in ranking.levels.iter_mut() {
            lvl.game_ids.retain(|g| g != &game_id);
        }

        if let Some(target_lvl) = ranking.levels.iter_mut().find(|l| l.level == level) {
            target_lvl.game_ids.push(game_id);
        }

        ranking.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = ranking.clone();
        data_store::write_rankings_only(&data.rankings)?;
        update_cache(&data);
        Ok(result)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

#[tauri::command]
pub async fn remove_game_from_rank(
    ranking_id: String,
    game_id: String,
) -> Result<crate::models::Ranking, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<crate::models::Ranking, AppError> {
        let mut data = get_cached_games()?;
        let ranking = data
            .rankings
            .iter_mut()
            .find(|r| r.id == ranking_id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "排名不存在"))?;

        for lvl in ranking.levels.iter_mut() {
            lvl.game_ids.retain(|g| g != &game_id);
        }

        ranking.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = ranking.clone();
        data_store::write_rankings_only(&data.rankings)?;
        update_cache(&data);
        Ok(result)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 在指定等级后添加新等级(insert_after_level 为 0 表示在最前插入)
#[tauri::command]
pub async fn add_rank_level(
    ranking_id: String,
    name: String,
    color: String,
    insert_after_level: i32,
) -> Result<crate::models::Ranking, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<crate::models::Ranking, AppError> {
        let mut data = get_cached_games()?;
        let ranking = data
            .rankings
            .iter_mut()
            .find(|r| r.id == ranking_id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "排名不存在"))?;

        let insert_idx = if insert_after_level <= 0 {
            0
        } else {
            ranking
                .levels
                .iter()
                .position(|l| l.level == insert_after_level)
                .map(|i| i + 1)
                .unwrap_or(ranking.levels.len())
        };

        // 后续等级 level+1
        for lvl in ranking.levels.iter_mut() {
            if lvl.level > insert_after_level {
                lvl.level += 1;
            }
        }

        let new_level = crate::models::RankLevel {
            level: insert_after_level + 1,
            name,
            game_ids: vec![],
            color,
        };
        ranking.levels.insert(insert_idx, new_level);
        ranking.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = ranking.clone();
        data_store::write_rankings_only(&data.rankings)?;
        update_cache(&data);
        Ok(result)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 删除指定等级,后续等级 level-1,游戏回到游戏库
#[tauri::command]
pub async fn delete_rank_level(
    ranking_id: String,
    level: i32,
) -> Result<crate::models::Ranking, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<crate::models::Ranking, AppError> {
        let mut data = get_cached_games()?;
        let ranking = data
            .rankings
            .iter_mut()
            .find(|r| r.id == ranking_id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "排名不存在"))?;

        ranking.levels.retain(|l| l.level != level);
        for (i, lvl) in ranking.levels.iter_mut().enumerate() {
            lvl.level = (i + 1) as i32;
        }
        ranking.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = ranking.clone();
        data_store::write_rankings_only(&data.rankings)?;
        update_cache(&data);
        Ok(result)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 清空指定等级中的游戏(游戏回到游戏库)
#[tauri::command]
pub async fn clear_rank_level(
    ranking_id: String,
    level: i32,
) -> Result<crate::models::Ranking, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<crate::models::Ranking, AppError> {
        let mut data = get_cached_games()?;
        let ranking = data
            .rankings
            .iter_mut()
            .find(|r| r.id == ranking_id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "排名不存在"))?;

        if let Some(lvl) = ranking.levels.iter_mut().find(|l| l.level == level) {
            lvl.game_ids.clear();
        }
        ranking.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = ranking.clone();
        data_store::write_rankings_only(&data.rankings)?;
        update_cache(&data);
        Ok(result)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

/// 修改等级名称和颜色
#[tauri::command]
pub async fn update_rank_level(
    ranking_id: String,
    level: i32,
    name: String,
    color: String,
) -> Result<crate::models::Ranking, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<crate::models::Ranking, AppError> {
        let mut data = get_cached_games()?;
        let ranking = data
            .rankings
            .iter_mut()
            .find(|r| r.id == ranking_id)
            .ok_or_else(|| AppError::new(ErrorCode::GameNotFound, "排名不存在"))?;

        if let Some(lvl) = ranking.levels.iter_mut().find(|l| l.level == level) {
            lvl.name = name;
            lvl.color = color;
        }
        ranking.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = ranking.clone();
        data_store::write_rankings_only(&data.rankings)?;
        update_cache(&data);
        Ok(result)
    })
    .await
    .unwrap_or_else(|e| {
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}

pub fn play_screenshot_sound(app_handle: Option<&tauri::AppHandle>) {
    let sound_path = match app_handle {
        Some(handle) => {
            match handle.path().resolve(
                "resources/sound/screenshot.wav",
                tauri::path::BaseDirectory::Resource,
            ) {
                Ok(p) => p,
                Err(_) => crate::path_manager::sound_dir().join("screenshot.wav"),
            }
        }
        None => crate::path_manager::sound_dir().join("screenshot.wav"),
    };

    if !sound_path.exists() {
        warn!("截图音效文件不存在: {}", sound_path.display());
        return;
    }

    std::thread::spawn(move || {
        let (_stream, stream_handle) = match rodio::OutputStream::try_default() {
            Ok(s) => s,
            Err(e) => {
                warn!("无法获取音频输出流: {}", e);
                return;
            }
        };

        let file = match std::fs::File::open(&sound_path) {
            Ok(f) => f,
            Err(e) => {
                warn!("打开音效文件失败: {}", e);
                return;
            }
        };

        let source = match rodio::Decoder::new(file) {
            Ok(s) => s,
            Err(e) => {
                warn!("解码音效文件失败: {}", e);
                return;
            }
        };

        let sink = match rodio::Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(e) => {
                warn!("创建音频接收器失败: {}", e);
                return;
            }
        };

        sink.append(source);
        sink.sleep_until_end();
    });
}

#[tauri::command]
pub async fn capture_screenshot(
    game_id: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let game_id_clone = game_id.clone();
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<String, AppError> {
        info!("开始截取前台窗口截图, game_id: {}", game_id);

        #[cfg(target_os = "windows")]
        let image = {
            use std::ptr;
            use winapi::{
                shared::windef::RECT,
                um::{
                    wingdi::{
                        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, GetDIBits, SelectObject,
                    },
                    winuser::{
                        GetForegroundWindow, GetWindowDC, GetWindowRect, PrintWindow, ReleaseDC,
                    },
                },
            };

            let hwnd = unsafe { GetForegroundWindow() };
            if hwnd == ptr::null_mut() {
                return Err(AppError::new(
                    ErrorCode::DataWriteFailed,
                    "无法获取前台窗口",
                ));
            }

            let mut rect: RECT = unsafe { std::mem::zeroed() };
            if unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
                return Err(AppError::new(
                    ErrorCode::DataWriteFailed,
                    "获取窗口矩形失败",
                ));
            }

            let width = (rect.right - rect.left) as i32;
            let height = (rect.bottom - rect.top) as i32;

            if width <= 0 || height <= 0 {
                return Err(AppError::new(ErrorCode::DataWriteFailed, "窗口尺寸无效"));
            }

            let hdc_window = unsafe { GetWindowDC(hwnd) };
            if hdc_window == ptr::null_mut() {
                return Err(AppError::new(ErrorCode::DataWriteFailed, "获取窗口DC失败"));
            }

            let hdc_mem = unsafe { CreateCompatibleDC(hdc_window) };
            if hdc_mem == ptr::null_mut() {
                unsafe { ReleaseDC(hwnd, hdc_window) };
                return Err(AppError::new(ErrorCode::DataWriteFailed, "创建兼容DC失败"));
            }

            let hbitmap = unsafe { CreateCompatibleBitmap(hdc_window, width, height) };
            if hbitmap == ptr::null_mut() {
                unsafe {
                    winapi::um::wingdi::DeleteDC(hdc_mem);
                    ReleaseDC(hwnd, hdc_window);
                };
                return Err(AppError::new(ErrorCode::DataWriteFailed, "创建位图失败"));
            }

            let old_bitmap = unsafe { SelectObject(hdc_mem, hbitmap as *mut _) };

            let print_success = unsafe { PrintWindow(hwnd, hdc_mem, 3) } != 0;

            if !print_success {
                debug!("PrintWindow失败，回退到BitBlt");
                if unsafe { BitBlt(hdc_mem, 0, 0, width, height, hdc_window, 0, 0, 0x00CC0020) }
                    == 0
                {
                    unsafe {
                        SelectObject(hdc_mem, old_bitmap);
                        winapi::um::wingdi::DeleteObject(hbitmap as *mut _);
                        winapi::um::wingdi::DeleteDC(hdc_mem);
                        ReleaseDC(hwnd, hdc_window);
                    };
                    return Err(AppError::new(ErrorCode::DataWriteFailed, "复制位图失败"));
                }
            }

            let mut bmp_info: winapi::um::wingdi::BITMAPINFO = unsafe { std::mem::zeroed() };
            bmp_info.bmiHeader.biSize =
                std::mem::size_of::<winapi::um::wingdi::BITMAPINFOHEADER>() as u32;
            bmp_info.bmiHeader.biWidth = width;
            bmp_info.bmiHeader.biHeight = -height;
            bmp_info.bmiHeader.biPlanes = 1;
            bmp_info.bmiHeader.biBitCount = 32;
            bmp_info.bmiHeader.biCompression = winapi::um::wingdi::BI_RGB;

            let buffer_size = (width * height * 4) as usize;
            let mut buffer: Vec<u8> = vec![0; buffer_size];

            unsafe {
                GetDIBits(
                    hdc_window,
                    hbitmap,
                    0,
                    height as u32,
                    buffer.as_mut_ptr() as *mut _,
                    &mut bmp_info,
                    winapi::um::wingdi::DIB_RGB_COLORS,
                );
            };

            unsafe {
                SelectObject(hdc_mem, old_bitmap);
                winapi::um::wingdi::DeleteObject(hbitmap as *mut _);
                winapi::um::wingdi::DeleteDC(hdc_mem);
                ReleaseDC(hwnd, hdc_window);
            };

            for pixel in buffer.chunks_mut(4) {
                pixel.swap(0, 2);
                pixel[3] = 255;
            }

            image::RgbaImage::from_raw(width as u32, height as u32, buffer)
                .ok_or_else(|| AppError::new(ErrorCode::DataWriteFailed, "创建图像失败"))?
        };

        #[cfg(not(target_os = "windows"))]
        let image = {
            return Err(AppError::new(
                ErrorCode::DataWriteFailed,
                "截图功能仅支持 Windows",
            ));
        };

        let dir = crate::path_manager::game_screenshots_dir(&game_id);
        fs::create_dir_all(&dir)
            .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "创建截图目录失败", e))?;

        let timestamp = Local::now().format("%Y%m%d%H%M%S%3f").to_string();
        let filename = format!("{}_capture_{}.png", game_id, timestamp);
        let dest = crate::path_manager::screenshot_path(&game_id, &filename);

        image
            .save(&dest)
            .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "保存截图文件失败", e))?;

        let thumb_dest = crate::path_manager::screenshot_thumb_path(&game_id, &filename);
        if let Err(e) = generate_thumbnail(&dest, &thumb_dest) {
            log::warn!("生成缩略图失败 ({}): {}", filename, e);
        }

        let mut data = get_cached_games()?;
        if let Some(game) = data.games.iter_mut().find(|g| g.id == game_id) {
            game.screenshots.push(filename.clone());
            data_store::write_games_only(&data.games)?;
            update_cache(&data);
        }

        let path_str = dest.to_string_lossy().to_string();
        crate::logger::log_screenshot_capture(&game_id, &path_str, true, None);
        info!("截图已捕获并保存: {}", path_str);

        play_screenshot_sound(Some(&app_handle_clone));

        Ok(path_str)
    })
    .await
    .unwrap_or_else(|e| {
        crate::logger::log_screenshot_capture(&game_id_clone, "", false, Some(&e.to_string()));
        Err(AppError::new(
            ErrorCode::InternalError,
            format!("任务执行失败: {}", e),
        ))
    })
    .map_err(to_frontend_error)
}
