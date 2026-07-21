//! 路径管理模块
//!
//! 统一管理所有数据存储路径，支持可配置的数据根目录。
//!
//! 路径规划：
//!   安装目录 (install_dir)
//!     └── Data/                       ← 数据根目录（默认，可配置）
//!           ├── game_list.json
//!           ├── CoverArt/             ← 封面图片
//!           ├── Saves/               ← 存档（预留）
//!           └── Cache/               ← 临时缓存
//!
//!   %APPDATA%/CleanGal/
//!     ├── Config/
//!     │   ├── System/path_config.json ← 数据路径配置
//!     │   └── User/setting.json       ← 用户偏好
//!     ├── Logs/                       ← 操作日志 & 追踪
//!     └── Backup/                     ← 数据备份

use crate::error::{AppError, ErrorCode};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

// ==================== 数据路径配置 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathConfig {
    /// 数据根目录路径
    pub data_root: String,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            data_root: String::new(), // 空表示使用默认路径
        }
    }
}

// ==================== 全局单例 ====================

static ROOT_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
static DATA_ROOT: Mutex<Option<String>> = Mutex::new(None);

/// 初始化路径管理器（必须在 setup 中调用）
pub fn init(install_dir: PathBuf) {
    let mut root = ROOT_PATH.lock().unwrap_or_else(|e| e.into_inner());
    *root = Some(install_dir.clone());
    info!("路径管理器初始化: install_dir={}", install_dir.display());
    drop(root);

    // 确保所有必要目录存在
    ensure_all_dirs();
}

fn get_install_dir() -> PathBuf {
    ROOT_PATH
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .expect("PathManager 未初始化")
}

fn get_configured_data_root() -> Option<String> {
    DATA_ROOT.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn set_configured_data_root(path: Option<String>) {
    let mut dr = DATA_ROOT.lock().unwrap_or_else(|e| e.into_inner());
    *dr = path;
}

// ==================== 路径获取函数 ====================

/// 获取系统配置目录 (%APPDATA%/CleanGal/Config)
pub fn system_config_dir() -> PathBuf {
    dirs_sys_config()
        .join("CleanGal")
        .join("Config")
        .join("System")
}

/// 获取用户配置目录 (%APPDATA%/CleanGal/Config/User)
pub fn user_config_dir() -> PathBuf {
    dirs_sys_config()
        .join("CleanGal")
        .join("Config")
        .join("User")
}

/// 获取日志目录 (%APPDATA%/CleanGal/Logs)
pub fn logs_dir() -> PathBuf {
    dirs_sys_config().join("CleanGal").join("Logs")
}

/// 获取备份目录 (%APPDATA%/CleanGal/Backup)
pub fn backup_dir() -> PathBuf {
    dirs_sys_config().join("CleanGal").join("Backup")
}

/// 获取 path_config.json 路径
pub fn path_config_file() -> PathBuf {
    system_config_dir().join("path_config.json")
}

/// 获取 setting.json 路径
pub fn settings_file() -> PathBuf {
    user_config_dir().join("setting.json")
}

/// 获取操作日志路径
pub fn operation_log_file() -> PathBuf {
    logs_dir().join("operation_log.jsonl")
}

/// 获取 API 追踪日志路径
pub fn bangumi_trace_file() -> PathBuf {
    logs_dir().join("bangumi_trace.jsonl")
}

// ==================== 数据根目录 ====================

/// 获取数据根目录
/// 优先级: 用户配置路径 > 默认(install_dir/Data)
pub fn data_root() -> PathBuf {
    if let Some(custom) = get_configured_data_root() {
        if !custom.is_empty() {
            let p = PathBuf::from(&custom);
            if p.exists() || p.parent().map_or(false, |parent| parent.exists()) {
                return p;
            }
            warn!("自定义数据路径不存在: {}, 回退到默认路径", custom);
        }
    }
    get_install_dir().join("Data")
}

/// 设置数据根目录（不含迁移，仅改配置+创建目录）
pub fn set_data_root(path: &str) -> Result<(), AppError> {
    let target = crate::utils::validate_path(path)?;
    do_set_data_root(&target)
}

/// 带迁移的设置数据根目录
pub fn set_data_root_with_migrate(new_path: &str) -> Result<DataMigrationReport, AppError> {
    let new_root = crate::utils::validate_path(new_path)?;
    let old_root = data_root();

    // 如果新旧路径相同，跳过
    if old_root == new_root {
        return Ok(DataMigrationReport::empty());
    }

    // 创建新目录
    if !new_root.is_dir() {
        fs::create_dir_all(&new_root)
            .map_err(|e| AppError::wrap(ErrorCode::PathInvalid, "创建数据目录失败", e))?;
    }

    // 执行迁移
    let report = migrate_data_dir(&old_root, &new_root)?;

    // 切换配置
    do_set_data_root(&new_root)?;

    // 清理旧目录（如果旧目录是默认路径则保留，只清理用户自定义路径下的）
    let install_data = get_install_dir().join("Data");
    if old_root != install_data && old_root.exists() {
        if let Err(e) = fs::remove_dir_all(&old_root) {
            warn!("清理旧数据目录失败: {} - {}", old_root.display(), e);
        }
    }

    info!(
        "数据已迁移: {} -> {}, 文件数: {}, 封面数: {}, 大小: {}MB",
        old_root.display(),
        new_root.display(),
        report.files_count,
        report.covers_count,
        report.total_mb()
    );
    Ok(report)
}

fn do_set_data_root(target: &PathBuf) -> Result<(), AppError> {
    let config = PathConfig {
        data_root: target.to_string_lossy().to_string(),
    };
    save_path_config(&config)?;
    set_configured_data_root(Some(config.data_root));
    ensure_data_dirs();
    Ok(())
}

// ==================== 数据迁移 ====================

#[derive(Debug, Clone, serde::Serialize)]
pub struct DataMigrationReport {
    pub files_count: usize,
    pub covers_count: usize,
    pub total_bytes: u64,
    pub errors: Vec<String>,
}

impl DataMigrationReport {
    fn empty() -> Self {
        Self {
            files_count: 0,
            covers_count: 0,
            total_bytes: 0,
            errors: vec![],
        }
    }

    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / 1_048_576.0
    }
}

/// 递归复制目录内容（从 old_root 到 new_root）
fn migrate_data_dir(
    old_root: &PathBuf,
    new_root: &PathBuf,
) -> Result<DataMigrationReport, AppError> {
    let mut report = DataMigrationReport::empty();

    let read_dir = match fs::read_dir(old_root) {
        Ok(d) => d,
        Err(e) => {
            return Err(AppError::wrap(
                ErrorCode::DataWriteFailed,
                "无法读取源数据目录",
                e,
            ));
        }
    };

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                report.errors.push(format!("读取目录项失败: {}", e));
                continue;
            }
        };

        let src = entry.path();
        let rel = match src.strip_prefix(old_root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let dest = new_root.join(rel);

        if src.is_dir() {
            if let Err(e) = fs::create_dir_all(&dest) {
                report
                    .errors
                    .push(format!("创建目录失败 {}: {}", dest.display(), e));
                continue;
            }
            match migrate_data_dir(&src, new_root) {
                Ok(sub) => {
                    report.files_count += sub.files_count;
                    report.covers_count += sub.covers_count;
                    report.total_bytes += sub.total_bytes;
                    report.errors.extend(sub.errors);
                }
                Err(e) => {
                    report
                        .errors
                        .push(format!("迁移子目录失败 {}: {}", src.display(), e));
                }
            }
        } else if src.is_file() {
            match fs::copy(&src, &dest) {
                Ok(bytes) => {
                    report.files_count += 1;
                    report.total_bytes += bytes;
                    // 统计封面文件
                    if rel.starts_with("CoverArt") {
                        report.covers_count += 1;
                    }
                }
                Err(e) => {
                    report
                        .errors
                        .push(format!("复制文件失败 {}: {}", src.display(), e));
                }
            }
        }
    }

    Ok(report)
}

/// 获取当前数据目录的大小信息
pub fn get_data_size_info() -> serde_json::Value {
    let root = data_root();
    let (file_count, total_bytes) = count_files(&root);
    let cover_dir = cover_dir();
    let cover_count = std::fs::read_dir(&cover_dir)
        .map(|d| d.count())
        .unwrap_or(0);

    serde_json::json!({
        "data_root": root.to_string_lossy(),
        "file_count": file_count,
        "total_mb": total_bytes as f64 / 1_048_576.0,
        "cover_count": cover_count,
        "cover_dir": cover_dir.to_string_lossy(),
        "config_dir": user_config_dir().to_string_lossy(),
    })
}

fn count_files(dir: &PathBuf) -> (usize, u64) {
    let mut file_count = 0usize;
    let mut total_bytes = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let (fc, tb) = count_files(&path);
                file_count += fc;
                total_bytes += tb;
            } else if let Ok(meta) = entry.metadata() {
                file_count += 1;
                total_bytes += meta.len();
            }
        }
    }
    (file_count, total_bytes)
}

/// 获取数据根目录字符串（给前端）
pub fn data_root_str() -> String {
    data_root().to_string_lossy().to_string()
}

// ==================== 数据子目录 & 文件 ====================

pub fn games_file() -> PathBuf {
    data_root().join("game_list.json")
}

pub fn cover_dir() -> PathBuf {
    data_root().join("CoverArt")
}

pub fn cover_path(filename: &str) -> PathBuf {
    cover_dir().join(filename)
}

pub fn saves_dir() -> PathBuf {
    data_root().join("Saves")
}

pub fn cache_dir() -> PathBuf {
    data_root().join("Cache")
}

// ==================== 路径配置持久化 ====================

/// 加载路径配置
pub fn load_path_config() -> PathConfig {
    let file = path_config_file();
    if !file.exists() {
        return PathConfig::default();
    }
    match fs::read_to_string(&file) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(e) => {
            warn!("读取路径配置失败: {}，使用默认", e);
            PathConfig::default()
        }
    }
}

fn save_path_config(config: &PathConfig) -> Result<(), AppError> {
    let file = path_config_file();
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "创建配置目录失败", e))?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::wrap(ErrorCode::DataSerializeFailed, "序列化路径配置失败", e))?;
    let tmp = file.with_extension("json.tmp");
    fs::write(&tmp, &content)
        .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "写入路径配置失败", e))?;
    fs::rename(&tmp, &file)
        .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "保存路径配置失败", e))?;
    Ok(())
}

// ==================== 初始化 & 迁移 ====================

/// 初始化时调用：加载路径配置、加载/迁移数据
pub fn init_from_config(install_dir: PathBuf) {
    init(install_dir);
    let config = load_path_config();
    if !config.data_root.is_empty() {
        set_configured_data_root(Some(config.data_root.clone()));
        info!("已加载自定义数据路径: {}", config.data_root);
    }
}

/// 确保所有必要目录存在
pub fn ensure_all_dirs() {
    // 系统路径
    let dirs = [
        system_config_dir(),
        user_config_dir(),
        logs_dir(),
        backup_dir(),
    ];
    for d in &dirs {
        let _ = fs::create_dir_all(d);
    }
    ensure_data_dirs();
}

fn ensure_data_dirs() {
    let dirs = [data_root(), cover_dir(), saves_dir(), cache_dir()];
    for d in &dirs {
        let _ = fs::create_dir_all(d);
    }
    debug!("数据目录已就绪: {}", data_root().display());
}

/// 迁移旧数据（从旧 %APPDATA%/galmanager/ 迁移到新路径）
/// 返回 (迁移的游戏数, 迁移的封面数)
pub fn migrate_old_data(app_handle: &tauri::AppHandle) -> Result<(usize, usize), AppError> {
    use tauri::Manager;

    // 旧数据路径
    let old_dir = match app_handle.path().app_data_dir() {
        Ok(d) => d.join("galmanager"),
        Err(_) => return Ok((0, 0)),
    };

    let old_games = old_dir.join("game_list.json");
    let old_settings = old_dir.join("setting.json");

    // 如果旧数据不存在则跳过
    if !old_games.exists() {
        return Ok((0, 0));
    }

    info!("检测到旧数据: {}", old_games.display());

    let mut migrated_games = 0usize;
    let mut migrated_covers = 0usize;

    // 迁移游戏数据
    let new_games_file = games_file();
    if !new_games_file.exists() || fs::metadata(&new_games_file).map_or(true, |m| m.len() == 0) {
        // 读取旧数据
        if let Ok(content) = fs::read_to_string(&old_games) {
            if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                // 迁移封面：从 Base64 提取为文件
                if let Some(covers_migrated) = migrate_covers_from_json(&mut json) {
                    migrated_covers = covers_migrated;
                }
                // 写入新路径
                let pretty = serde_json::to_string_pretty(&json).unwrap_or(content);
                if let Some(parent) = new_games_file.parent() {
                    fs::create_dir_all(parent).ok();
                }
                fs::write(&new_games_file, &pretty).map_err(|e| {
                    AppError::wrap(ErrorCode::DataWriteFailed, "迁移游戏数据写入失败", e)
                })?;
                migrated_games = 1;
                info!(
                    "游戏数据已迁移: {} -> {}",
                    old_games.display(),
                    new_games_file.display()
                );
            }
        }
    }

    // 迁移设置
    let new_settings = settings_file();
    if old_settings.exists()
        && (!new_settings.exists() || fs::metadata(&new_settings).map_or(true, |m| m.len() == 0))
    {
        if let Ok(content) = fs::read_to_string(&old_settings) {
            if let Some(parent) = new_settings.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&new_settings, &content).ok();
            info!(
                "设置数据已迁移: {} -> {}",
                old_settings.display(),
                new_settings.display()
            );
        }
    }

    Ok((migrated_games, migrated_covers))
}

/// 从 JSON 中提取 Base64 封面为文件，返回迁移的封面数
fn migrate_covers_from_json(json: &mut serde_json::Value) -> Option<usize> {
    let games = json.get_mut("games")?.as_array_mut()?;
    let cover_dir = cover_dir();
    let _ = fs::create_dir_all(&cover_dir);

    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    let mut count = 0usize;

    for game in games.iter_mut() {
        let cover_val = game.get("cover")?;
        let cover_str = cover_val.as_str()?;
        if cover_str.is_empty() || !cover_str.starts_with("data:") {
            continue;
        }

        // 提取 MIME 和 Base64 数据
        let (mime, b64_data) = if let Some(idx) = cover_str.find(";base64,") {
            let mime_part = &cover_str[5..idx]; // 跳过 "data:"
            let data = &cover_str[idx + 8..]; // 跳过 ";base64,"
            (mime_part, data)
        } else {
            continue;
        };

        let ext = match mime {
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => "jpg",
        };

        let img_bytes = match BASE64.decode(b64_data) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let filename = format!("{}.{}", game.get("id")?.as_str()?, ext);
        let filepath = cover_dir.join(&filename);

        if fs::write(&filepath, &img_bytes).is_ok() {
            *game.get_mut("cover").unwrap() = serde_json::Value::String(filename);
            count += 1;
        }
    }

    Some(count)
}

// ==================== 内部辅助 ====================

#[cfg(target_os = "windows")]
fn dirs_sys_config() -> PathBuf {
    match std::env::var("APPDATA") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            let home = std::env::var("USERPROFILE").unwrap_or_default();
            PathBuf::from(home).join("AppData").join("Roaming")
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn dirs_sys_config() -> PathBuf {
    match std::env::var("XDG_CONFIG_HOME") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(home).join(".config")
        }
    }
}

// ==================== 封面读写 ====================

/// 读取封面文件返回 Base64 data URI
pub fn read_cover_as_data_uri(filename: &str) -> Option<String> {
    if filename.is_empty() {
        return None;
    }
    let path = cover_path(filename);
    if !path.exists() {
        return None;
    }
    let data = fs::read(&path).ok()?;
    let ext = path.extension()?.to_str()?;
    let mime = match ext.to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/jpeg",
    };
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
    Some(format!("data:{};base64,{}", mime, b64))
}

/// 保存封面文件（支持 Base64 data URI 或原始二进制）
/// 返回文件名
pub fn save_cover_file(game_id: &str, data_uri: &str) -> Result<String, AppError> {
    let cover_dir = cover_dir();
    fs::create_dir_all(&cover_dir)
        .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "创建封面目录失败", e))?;

    // 移除旧封面文件
    remove_cover_for_game(game_id);

    let (mime, data) = parse_data_uri(data_uri)?;

    let ext = match mime {
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "jpg",
    };
    let filename = format!("{}.{}", game_id, ext);
    let filepath = cover_dir.join(&filename);

    fs::write(&filepath, &data)
        .map_err(|e| AppError::wrap(ErrorCode::DataWriteFailed, "写入封面文件失败", e))?;

    debug!("封面已保存: {}", filename);
    Ok(filename)
}

/// 解析 data URI，返回 (mime, binary_data)
fn parse_data_uri(uri: &str) -> Result<(&str, Vec<u8>), AppError> {
    if !uri.starts_with("data:") {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "不是有效的 data URI",
        ));
    }
    let idx = uri
        .find(";base64,")
        .ok_or_else(|| AppError::new(ErrorCode::InvalidInput, "data URI 缺少 base64 标识"))?;
    let mime = &uri[5..idx];
    let b64_data = &uri[idx + 8..];
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64_data)
        .map_err(|e| AppError::wrap(ErrorCode::InvalidInput, "Base64 解码失败", e))?;
    Ok((mime, bytes))
}

/// 删除某个游戏的所有封面文件（不同扩展名都尝试删除）
fn remove_cover_for_game(game_id: &str) {
    for ext in &["jpg", "png", "gif", "webp", "jpeg"] {
        let path = cover_dir().join(format!("{}.{}", game_id, ext));
        if path.exists() {
            let _ = fs::remove_file(&path);
        }
    }
}

/// 删除游戏的封面文件
pub fn delete_cover(filename: &str) {
    if filename.is_empty() {
        return;
    }
    let path = cover_path(filename);
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
}

/// 批量读取封面（为多个游戏 ID 返回 Base64 data URI 映射）
pub fn read_covers_batch(game_ids: &[String]) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for id in game_ids {
        // 尝试各种扩展名
        for ext in &["jpg", "png", "gif", "webp", "jpeg"] {
            let filename = format!("{}.{}", id, ext);
            if let Some(data_uri) = read_cover_as_data_uri(&filename) {
                map.insert(id.clone(), data_uri);
                break;
            }
        }
    }
    map
}
