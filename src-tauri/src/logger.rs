use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
#[allow(dead_code)]
pub enum LogCategory {
    Operation,
    Game,
    Screenshot,
    Data,
    System,
}

impl LogCategory {
    fn file_name(&self) -> &str {
        match self {
            LogCategory::Operation => "operation.log",
            LogCategory::Game => "game.log",
            LogCategory::Screenshot => "screenshot.log",
            LogCategory::Data => "data.log",
            LogCategory::System => "system.log",
        }
    }
}

struct LogWriter {
    file: std::fs::File,
    #[allow(dead_code)]
    category: LogCategory,
}

use std::sync::LazyLock;

static LOG_WRITERS: LazyLock<Mutex<std::collections::HashMap<LogCategory, LogWriter>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

fn now() -> String {
    chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S%.3f")
        .to_string()
}

fn get_writer(
    category: LogCategory,
) -> Option<std::sync::MutexGuard<'static, std::collections::HashMap<LogCategory, LogWriter>>> {
    let mut guard = LOG_WRITERS.lock().unwrap_or_else(|e| e.into_inner());

    if !guard.contains_key(&category) {
        let log_dir = crate::path_manager::logs_dir();
        let _ = std::fs::create_dir_all(&log_dir);
        let log_path = log_dir.join(category.file_name());

        if let Ok(f) = OpenOptions::new().create(true).append(true).open(&log_path) {
            guard.insert(category.clone(), LogWriter { file: f, category });
        } else {
            return None;
        }
    }

    Some(guard)
}

fn write_log(category: LogCategory, level: &str, module: &str, action: &str, detail: Value) {
    if let Some(mut guard) = get_writer(category) {
        if let Some(writer) = guard.get_mut(&category) {
            let entry = serde_json::json!({
                "time": now(),
                "level": level,
                "module": module,
                "action": action,
                "detail": detail
            });
            let line = serde_json::to_string(&entry).unwrap_or_default();
            let _ = writeln!(writer.file, "{}", line);
            let _ = writer.file.flush();
        }
    }
}

pub fn info(category: LogCategory, module: &str, action: &str, detail: Value) {
    write_log(category, "INFO", module, action, detail);
}

#[allow(dead_code)]
pub fn warn(category: LogCategory, module: &str, action: &str, detail: Value) {
    write_log(category, "WARN", module, action, detail);
}

pub fn error(category: LogCategory, module: &str, action: &str, detail: Value) {
    write_log(category, "ERROR", module, action, detail);
}

#[allow(dead_code)]
pub fn debug(category: LogCategory, module: &str, action: &str, detail: Value) {
    write_log(category, "DEBUG", module, action, detail);
}

pub fn log_game_launch(game_id: &str, name: &str, path: &str) {
    info(
        LogCategory::Game,
        "launch",
        "game_start",
        serde_json::json!({ "game_id": game_id, "name": name, "path": path }),
    );
}

pub fn log_game_exit(game_id: &str, name: &str, play_time_added: u64) {
    info(
        LogCategory::Game,
        "launch",
        "game_exit",
        serde_json::json!({
            "game_id": game_id,
            "name": name,
            "play_time_added": play_time_added
        }),
    );
}

pub fn log_game_update_playtime(game_id: &str, name: &str, old_time: u64, new_time: u64) {
    info(
        LogCategory::Data,
        "game",
        "playtime_update",
        serde_json::json!({
            "game_id": game_id,
            "name": name,
            "old_play_time": old_time,
            "new_play_time": new_time
        }),
    );
}

pub fn log_game_add(game_id: &str, name: &str) {
    info(
        LogCategory::Data,
        "game",
        "game_add",
        serde_json::json!({ "game_id": game_id, "name": name }),
    );
}

#[allow(dead_code)]
pub fn log_game_update(game_id: &str, name: &str) {
    info(
        LogCategory::Data,
        "game",
        "game_update",
        serde_json::json!({ "game_id": game_id, "name": name }),
    );
}

#[allow(dead_code)]
pub fn log_game_delete(game_id: &str, name: &str) {
    info(
        LogCategory::Data,
        "game",
        "game_delete",
        serde_json::json!({ "game_id": game_id, "name": name }),
    );
}

pub fn log_screenshot_capture(
    game_id: &str,
    save_path: &str,
    success: bool,
    err_msg: Option<&str>,
) {
    match success {
        true => info(
            LogCategory::Screenshot,
            "capture",
            "screenshot_success",
            serde_json::json!({ "game_id": game_id, "save_path": save_path }),
        ),
        false => error(
            LogCategory::Screenshot,
            "capture",
            "screenshot_failed",
            serde_json::json!({
                "game_id": game_id,
                "save_path": save_path,
                "error": err_msg.unwrap_or("unknown")
            }),
        ),
    }
}

#[allow(dead_code)]
pub fn log_screenshot_import(game_id: &str, filename: &str) {
    info(
        LogCategory::Screenshot,
        "import",
        "screenshot_import",
        serde_json::json!({ "game_id": game_id, "filename": filename }),
    );
}

#[allow(dead_code)]
pub fn log_screenshot_delete(game_id: &str, filename: &str) {
    info(
        LogCategory::Screenshot,
        "delete",
        "screenshot_delete",
        serde_json::json!({ "game_id": game_id, "filename": filename }),
    );
}

#[allow(dead_code)]
pub fn log_data_backup(path: &str, success: bool) {
    match success {
        true => info(
            LogCategory::System,
            "backup",
            "backup_success",
            serde_json::json!({ "path": path }),
        ),
        false => error(
            LogCategory::System,
            "backup",
            "backup_failed",
            serde_json::json!({ "path": path }),
        ),
    }
}

#[allow(dead_code)]
pub fn log_data_restore(path: &str, success: bool) {
    match success {
        true => info(
            LogCategory::System,
            "restore",
            "restore_success",
            serde_json::json!({ "path": path }),
        ),
        false => error(
            LogCategory::System,
            "restore",
            "restore_failed",
            serde_json::json!({ "path": path }),
        ),
    }
}

#[allow(dead_code)]
pub fn log_status_change(game_id: &str, name: &str, old_status: &str, new_status: &str) {
    info(
        LogCategory::Data,
        "game",
        "status_change",
        serde_json::json!({
            "game_id": game_id,
            "name": name,
            "old_status": old_status,
            "new_status": new_status
        }),
    );
}

#[allow(dead_code)]
pub fn log_favorite_toggle(game_id: &str, name: &str, is_favorite: bool) {
    info(
        LogCategory::Data,
        "game",
        "favorite_toggle",
        serde_json::json!({
            "game_id": game_id,
            "name": name,
            "is_favorite": is_favorite
        }),
    );
}

#[allow(dead_code)]
pub fn log_covers_load(game_ids: &[String], success_count: usize, failed_count: usize) {
    info(
        LogCategory::System,
        "covers",
        "covers_load",
        serde_json::json!({
            "game_ids": game_ids,
            "success_count": success_count,
            "failed_count": failed_count
        }),
    );
}

pub fn log_bgm_search_start(keyword: &str) {
    info(
        LogCategory::Operation,
        "bangumi_search",
        "start",
        serde_json::json!({ "keyword": keyword }),
    );
}

pub fn log_bgm_search_done(keyword: &str, count: usize, elapsed_ms: u128) {
    info(
        LogCategory::Operation,
        "bangumi_search",
        "done",
        serde_json::json!({
            "keyword": keyword,
            "result_count": count,
            "elapsed_ms": elapsed_ms
        }),
    );
}

pub fn log_bgm_search_error(keyword: &str, err: &str, elapsed_ms: u128) {
    error(
        LogCategory::Operation,
        "bangumi_search",
        "error",
        serde_json::json!({
            "keyword": keyword,
            "error": err,
            "elapsed_ms": elapsed_ms
        }),
    );
}

pub fn log_bgm_detail_start(subject_id: u32) {
    info(
        LogCategory::Operation,
        "bangumi_detail",
        "start",
        serde_json::json!({ "subject_id": subject_id }),
    );
}

pub fn log_bgm_detail_done(subject_id: u32, name: &str, elapsed_ms: u128) {
    info(
        LogCategory::Operation,
        "bangumi_detail",
        "done",
        serde_json::json!({
            "subject_id": subject_id,
            "name": name,
            "elapsed_ms": elapsed_ms
        }),
    );
}

pub fn log_bgm_detail_error(subject_id: u32, err: &str, elapsed_ms: u128) {
    error(
        LogCategory::Operation,
        "bangumi_detail",
        "error",
        serde_json::json!({
            "subject_id": subject_id,
            "error": err,
            "elapsed_ms": elapsed_ms
        }),
    );
}

pub fn log_bgm_cover_done(subject_id: u32, size_bytes: usize, elapsed_ms: u128) {
    info(
        LogCategory::Operation,
        "bangumi_cover",
        "done",
        serde_json::json!({
            "subject_id": subject_id,
            "size_bytes": size_bytes,
            "elapsed_ms": elapsed_ms
        }),
    );
}

pub fn log_bgm_cover_error(url: &str, err: &str) {
    error(
        LogCategory::Operation,
        "bangumi_cover",
        "error",
        serde_json::json!({ "url": url, "error": err }),
    );
}

pub fn log_bgm_fill(subject_id: u32, name: &str, has_cover: bool, tag_count: usize) {
    info(
        LogCategory::Operation,
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
