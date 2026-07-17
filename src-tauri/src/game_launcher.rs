use anyhow::{Context, Result};
use std::process::{Command, Child};
use std::sync::Mutex;
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use chrono::Utc;
use tauri::AppHandle;

lazy_static::lazy_static! {
    static ref ACTIVE_GAMES: Mutex<Vec<(String, Child)>> = Mutex::new(Vec::new());
}

/// 启动游戏（异步，不阻塞）
pub fn launch_game(app_handle: &AppHandle, path: &str, args: Option<&[String]>) -> Result<()> {
    let mut cmd = Command::new(path);
    if let Some(arg_list) = args {
        cmd.args(arg_list);
    }
    // 设置工作目录为游戏所在目录，避免相对路径问题
    if let Some(parent) = std::path::Path::new(path).parent() {
        cmd.current_dir(parent);
    }
    let child = cmd.spawn().context("启动游戏失败")?;
    
    // 存储进程信息用于后续监控
    let mut guard = ACTIVE_GAMES.lock().unwrap();
    guard.push((path.to_string(), child));
    Ok(())
}

/// 检查游戏是否正在运行（根据路径检测）
pub fn is_game_running(path: &str) -> bool {
    let s = System::new_all();
    for process in s.processes().values() {
        if let Some(exe) = process.exe() {
            if exe.to_string_lossy() == path {
                return true;
            }
        }
    }
    false
}

/// 获取所有正在运行的游戏路径列表
pub fn get_running_games() -> Vec<String> {
    let s = System::new_all();
    let mut result = Vec::new();
    for process in s.processes().values() {
        if let Some(exe) = process.exe() {
            result.push(exe.to_string_lossy().to_string());
        }
    }
    result
}

/// 结束游戏进程（通过路径）
pub fn kill_game(path: &str) -> Result<()> {
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