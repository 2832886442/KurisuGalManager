use crate::data_store;
use crate::error::{to_frontend_error, AppError, ErrorCode};
use crate::models::{AppData, Game, Settings};
use crate::utils;
use chrono::Local;
use log::{debug, info, warn};
use rfd::FileDialog;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
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
        data_store::write_games(&data)?;
        update_cache(&data);
        crate::logger::log_op(
            "game",
            "add",
            serde_json::json!({"count": data.games.len()}),
        );
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
        data_store::write_games(&data)?;
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
        data_store::write_games(&data)?;
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

        let update_result: Result<(), AppError> = (|| {
            let mut data = data_store::read_games()?;
            if let Some(game) = data.games.iter_mut().find(|g| g.id == game_id2) {
                game.play_time += minutes;
                game.last_play = Some(Local::now().format("%Y-%m-%d").to_string());
                if game.status == "未游玩" {
                    game.status = "游玩中".to_string();
                }
                data_store::write_games(&data)?;
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
            data_store::write_games(&data)?;
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
        data_store::write_games(&data)?;
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
        data_store::write_games(&data)?;
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
