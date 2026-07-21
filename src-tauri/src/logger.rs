//! 操作日志模块
//! 将用户操作和 API 调用记录到日志文件，方便问题追溯
//!
//! 日志文件位于 %APPDATA%/CleanGal/Logs/operation_log.jsonl
//!
//! 日志格式（每行一条 JSON）：
//! { "time": "YYYY-MM-DD HH:MM:SS.sss", "module": "search", "action": "search_start", "detail": { "keyword": "xxx" } }

use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

fn now() -> String {
    chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S%.3f")
        .to_string()
}

fn get_file() -> &'static Mutex<Option<std::fs::File>> {
    // 初始化日志文件路径
    let mut guard = LOG_FILE.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        let log_path = crate::path_manager::operation_log_file();
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(f) = OpenOptions::new().create(true).append(true).open(&log_path) {
            *guard = Some(f);
        }
    }
    drop(guard);
    &LOG_FILE
}

/// 写入一条操作日志
pub fn log_op(module: &str, action: &str, detail: Value) {
    let entry = serde_json::json!({
        "time": now(),
        "module": module,
        "action": action,
        "detail": detail
    });
    let line = serde_json::to_string(&entry).unwrap_or_default();
    let mut guard = get_file().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref mut f) = *guard {
        let _ = writeln!(f, "{}", line);
        let _ = f.flush();
    }
}

/// 记录 Bangumi 搜索操作
pub fn log_bgm_search_start(keyword: &str) {
    log_op(
        "bangumi_search",
        "start",
        serde_json::json!({ "keyword": keyword }),
    );
}

/// 记录 Bangumi 搜索完成
pub fn log_bgm_search_done(keyword: &str, count: usize, elapsed_ms: u128) {
    log_op(
        "bangumi_search",
        "done",
        serde_json::json!({
            "keyword": keyword,
            "result_count": count,
            "elapsed_ms": elapsed_ms
        }),
    );
}

/// 记录 Bangumi 搜索失败
pub fn log_bgm_search_error(keyword: &str, error: &str, elapsed_ms: u128) {
    log_op(
        "bangumi_search",
        "error",
        serde_json::json!({
            "keyword": keyword,
            "error": error,
            "elapsed_ms": elapsed_ms
        }),
    );
}

/// 记录获取游戏详情开始
pub fn log_bgm_detail_start(subject_id: u32) {
    log_op(
        "bangumi_detail",
        "start",
        serde_json::json!({ "subject_id": subject_id }),
    );
}

/// 记录获取游戏详情完成
pub fn log_bgm_detail_done(subject_id: u32, name: &str, elapsed_ms: u128) {
    log_op(
        "bangumi_detail",
        "done",
        serde_json::json!({
            "subject_id": subject_id,
            "name": name,
            "elapsed_ms": elapsed_ms
        }),
    );
}

/// 记录获取游戏详情失败
pub fn log_bgm_detail_error(subject_id: u32, error: &str, elapsed_ms: u128) {
    log_op(
        "bangumi_detail",
        "error",
        serde_json::json!({
            "subject_id": subject_id,
            "error": error,
            "elapsed_ms": elapsed_ms
        }),
    );
}

/// 记录封面下载完成
pub fn log_bgm_cover_done(subject_id: u32, size_bytes: usize, elapsed_ms: u128) {
    log_op(
        "bangumi_cover",
        "done",
        serde_json::json!({
            "subject_id": subject_id,
            "size_bytes": size_bytes,
            "elapsed_ms": elapsed_ms
        }),
    );
}

/// 记录封面下载失败
pub fn log_bgm_cover_error(url: &str, error: &str) {
    log_op(
        "bangumi_cover",
        "error",
        serde_json::json!({
            "url": url,
            "error": error
        }),
    );
}

/// 记录游戏数据填充
pub fn log_bgm_fill(subject_id: u32, name: &str, has_cover: bool, tag_count: usize) {
    log_op(
        "bangumi_fill",
        "ok",
        serde_json::json!({
            "subject_id": subject_id,
            "name": name,
            "has_cover": has_cover,
            "tag_count": tag_count
        }),
    );
}
