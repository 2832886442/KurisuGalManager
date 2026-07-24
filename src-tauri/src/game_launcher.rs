use crate::error::{AppError, ErrorCode};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::path::Path;
use std::process::{Child, Command};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use sysinfo::System;

/// 活跃游戏进程管理器
pub struct GameProcessManager {
    children: HashMap<String, Child>,
    /// 无 Child 句柄的运行中路径（由 launch_game 异步监控任务管理）
    running_paths: Vec<String>,
    /// 运行中游戏的 game_id -> path 映射
    running_games: HashMap<String, String>,
}

impl GameProcessManager {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            running_paths: Vec::new(),
            running_games: HashMap::new(),
        }
    }

    pub fn register_running(&mut self, path: &str) {
        if !self.running_paths.iter().any(|p| p == path) {
            self.running_paths.push(path.to_string());
        }
    }

    pub fn unregister_running(&mut self, path: &str) {
        self.running_paths.retain(|p| p != path);
    }

    pub fn register_game(&mut self, game_id: &str, path: &str) {
        self.running_games
            .insert(game_id.to_string(), path.to_string());
    }

    pub fn unregister_game(&mut self, game_id: &str) {
        self.running_games.remove(game_id);
    }

    pub fn get_running_game_id(&mut self) -> Option<String> {
        let s = System::new_all();

        let running_game_paths: std::collections::HashSet<&String> =
            self.running_games.values().collect();

        self.running_paths.retain(|path| {
            if running_game_paths.contains(path) {
                for process in s.processes().values() {
                    if let Some(exe) = process.exe() {
                        if exe.to_string_lossy().contains(path) {
                            return true;
                        }
                    }
                }
            }
            false
        });
        self.running_games.keys().next().cloned()
    }

    pub fn running_paths(&mut self) -> Vec<String> {
        self.children
            .retain(|_path, child| matches!(child.try_wait(), Ok(None)));
        let child_paths: Vec<String> = self.children.keys().cloned().collect();
        let mut all = child_paths;
        all.extend(self.running_paths.clone());
        all
    }
}

static PROCESS_MANAGER: OnceLock<Mutex<GameProcessManager>> = OnceLock::new();

fn get_process_manager() -> &'static Mutex<GameProcessManager> {
    PROCESS_MANAGER.get_or_init(|| Mutex::new(GameProcessManager::new()))
}

/// 在进程管理器中注册一个运行中路径（由 launch_game 调用）
pub fn register_running_path(path: &str) {
    if let Ok(mut mgr) = get_process_manager().lock() {
        mgr.register_running(path);
    }
}

/// 从进程管理器中移除运行中路径（由 launch_game 后台任务调用）
pub fn unregister_running_path(path: &str) {
    if let Ok(mut mgr) = get_process_manager().lock() {
        mgr.unregister_running(path);
    }
}

/// 启动游戏进程，返回 Child 句柄和启动时间
pub fn spawn_game_process(
    path: &str,
    args: Option<&[String]>,
) -> Result<(Child, Instant), AppError> {
    let mut cmd = Command::new(path);
    if let Some(arg_list) = args {
        cmd.args(arg_list);
    }
    if let Some(parent) = Path::new(path).parent() {
        cmd.current_dir(parent);
    }

    let start_time = Instant::now();
    let child = cmd.spawn().map_err(|e| {
        AppError::wrap(
            ErrorCode::GameLaunchFailed,
            format!("启动游戏失败: {}", path),
            e,
        )
    })?;

    info!("游戏已启动: {}", path);
    Ok((child, start_time))
}

/// 获取所有运行中游戏路径列表
pub fn get_running_games() -> Result<Vec<String>, AppError> {
    let mut mgr = get_process_manager().lock().map_err(|e| {
        AppError::new(
            ErrorCode::InternalError,
            format!("进程管理器锁定失败: {}", e),
        )
    })?;
    Ok(mgr.running_paths())
}

/// 注册运行中的游戏（game_id 和 path 的映射）
pub fn register_running_game(game_id: &str, path: &str) {
    if let Ok(mut mgr) = get_process_manager().lock() {
        mgr.register_game(game_id, path);
    }
}

/// 注销运行中的游戏
pub fn unregister_running_game(game_id: &str) {
    if let Ok(mut mgr) = get_process_manager().lock() {
        mgr.unregister_game(game_id);
    }
}

/// 获取当前运行中的游戏 ID
pub fn get_current_running_game_id() -> Option<String> {
    if let Ok(mut mgr) = get_process_manager().lock() {
        mgr.get_running_game_id()
    } else {
        None
    }
}

/// 结束游戏进程
pub fn kill_game(path: &str) -> Result<(), AppError> {
    // 规范化路径用于精确匹配
    let normalized = Path::new(path)
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string());

    {
        let mut mgr = get_process_manager().lock().map_err(|e| {
            AppError::new(
                ErrorCode::InternalError,
                format!("进程管理器锁定失败: {}", e),
            )
        })?;
        if let Some(mut child) = mgr.children.remove(&normalized) {
            if let Err(e) = child.kill() {
                warn!("终止子进程失败（可能已退出）: {} - {}", normalized, e);
            }
            debug!("通过进程管理器终止游戏: {}", normalized);
            mgr.unregister_running(&normalized);
            return Ok(());
        }
    }

    // 系统级扫描降级路径
    let s = System::new_all();
    for process in s.processes().values() {
        if let Some(exe) = process.exe() {
            if exe.to_string_lossy() == normalized || exe.to_string_lossy() == path {
                process.kill();
                info!("通过系统级查找终止游戏: {}", normalized);
                return Ok(());
            }
        }
    }
    Err(AppError::new(
        ErrorCode::GameNotRunning,
        "未找到运行中的游戏进程",
    ))
}
