mod bangumi;
#[cfg(test)]
mod bangumi_test;
mod commands;
mod data_store;
mod error;
mod game_launcher;
mod logger;
mod models;
mod path_manager;
mod settings;
mod utils;

use tauri::Emitter;

#[cfg(target_os = "windows")]
fn start_f12_listener(app_handle: tauri::AppHandle) {
    use std::ptr;
    use winapi::{
        shared::minwindef::WPARAM,
        um::winuser::{GetAsyncKeyState, RegisterHotKey, UnregisterHotKey, WM_HOTKEY},
    };

    const HOTKEY_ID: i32 = 1001;
    const VK_F12: u32 = 0x7B;

    log::info!("正在注册 F12 全局热键...");

    let register_success = unsafe { RegisterHotKey(ptr::null_mut(), HOTKEY_ID, 0, VK_F12) != 0 };

    if register_success {
        log::info!("F12 全局热键注册成功，等待热键触发...");

        std::thread::spawn(move || {
            let mut msg: winapi::um::winuser::MSG = unsafe { std::mem::zeroed() };
            loop {
                let ret =
                    unsafe { winapi::um::winuser::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) };
                if ret <= 0 {
                    log::info!("热键消息循环退出，ret={}", ret);
                    break;
                }

                if msg.message == WM_HOTKEY && msg.wParam == HOTKEY_ID as WPARAM {
                    log::info!("F12 热键已触发，开始执行截图");
                    let app_handle_clone = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        log::info!("正在获取当前运行游戏 ID");
                        if let Some(game_id) = crate::game_launcher::get_current_running_game_id() {
                            log::info!("找到运行游戏 ID: {}", game_id);
                            match commands::capture_screenshot(game_id).await {
                                Ok(path) => {
                                    log::info!("截图成功，保存路径: {}", path);
                                    let _ = app_handle_clone.emit(
                                        "screenshot-captured",
                                        serde_json::json!({
                                            "path": path,
                                            "success": true
                                        }),
                                    );
                                }
                                Err(e) => {
                                    log::error!("截图失败: {}", e);
                                    let _ = app_handle_clone.emit(
                                        "screenshot-captured",
                                        serde_json::json!({
                                            "success": false,
                                            "error": e
                                        }),
                                    );
                                }
                            }
                        } else {
                            log::warn!("未找到运行中的游戏，无法截图");
                            let _ = app_handle_clone.emit(
                                "screenshot-captured",
                                serde_json::json!({
                                    "success": false,
                                    "error": "未找到运行中的游戏"
                                }),
                            );
                        }
                    });
                }
            }

            log::info!("正在注销 F12 全局热键...");
            unsafe {
                UnregisterHotKey(ptr::null_mut(), HOTKEY_ID);
            }
            log::info!("F12 全局热键已注销");
        });
    } else {
        let err = std::io::Error::last_os_error();
        log::warn!("注册 F12 全局热键失败: {}，切换到轮询模式", err);

        std::thread::spawn(move || {
            log::info!("F12 轮询模式启动，每 100ms 检查一次");
            let mut was_pressed = false;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));

                let key_state = unsafe { GetAsyncKeyState(VK_F12 as i32) };
                let is_pressed = (key_state & 0x8000u16 as i16) != 0;

                if is_pressed && !was_pressed {
                    log::info!("F12 热键已触发（轮询模式），开始执行截图");
                    let app_handle_clone = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        log::info!("正在获取当前运行游戏 ID");
                        if let Some(game_id) = crate::game_launcher::get_current_running_game_id() {
                            log::info!("找到运行游戏 ID: {}", game_id);
                            match commands::capture_screenshot(game_id).await {
                                Ok(path) => {
                                    log::info!("截图成功，保存路径: {}", path);
                                    let _ = app_handle_clone.emit(
                                        "screenshot-captured",
                                        serde_json::json!({
                                            "path": path,
                                            "success": true
                                        }),
                                    );
                                }
                                Err(e) => {
                                    log::error!("截图失败: {}", e);
                                    let _ = app_handle_clone.emit(
                                        "screenshot-captured",
                                        serde_json::json!({
                                            "success": false,
                                            "error": e
                                        }),
                                    );
                                }
                            }
                        } else {
                            log::warn!("未找到运行中的游戏，无法截图");
                            let _ = app_handle_clone.emit(
                                "screenshot-captured",
                                serde_json::json!({
                                    "success": false,
                                    "error": "未找到运行中的游戏"
                                }),
                            );
                        }
                    });
                }
                was_pressed = is_pressed;
            }
        });
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    log::info!("KurisuGal v{} 启动中...", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_games,
            commands::get_game_covers,
            commands::add_game,
            commands::update_game,
            commands::delete_game,
            commands::launch_game,
            commands::scan_folder,
            commands::get_running_games,
            commands::open_file_dialog,
            commands::copy_cover,
            commands::kill_game,
            commands::backup_data,
            commands::restore_data,
            commands::cleanup_invalid,
            commands::get_settings,
            commands::save_settings,
            commands::set_startup,
            commands::batch_update_category,
            commands::quick_update_status,
            commands::search_bangumi,
            commands::fetch_bangumi_game,
            commands::download_bangumi_cover,
            commands::add_rank_virtual_game,
            commands::remove_rank_virtual_game,
            commands::get_data_root,
            commands::set_data_root,
            commands::get_data_size_info,
            commands::pick_folder,
            commands::add_screenshot,
            commands::list_screenshots,
            commands::list_screenshots_with_paths,
            commands::list_screenshots_with_thumbs,
            commands::get_screenshot_base64,
            commands::get_screenshot_path,
            commands::get_screenshot_thumb_path,
            commands::delete_screenshot,
            commands::capture_screenshot,
            commands::toggle_favorite,
            commands::get_home_stats,
            commands::get_logo,
            commands::save_logo,
            commands::get_app_icon,
            commands::get_app_version,
            commands::get_rankings,
            commands::create_ranking,
            commands::update_ranking,
            commands::delete_ranking,
            commands::set_game_rank,
            commands::remove_game_from_rank,
            commands::add_rank_level,
            commands::delete_rank_level,
            commands::clear_rank_level,
            commands::update_rank_level,
        ])
        .setup(|app| {
            let install_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            crate::path_manager::init_from_config(install_dir);

            if let Err(e) = crate::path_manager::migrate_old_data(&app.handle()) {
                log::warn!("旧数据迁移失败: {}", e);
            }

            #[cfg(target_os = "windows")]
            start_f12_listener(app.handle().clone());

            log::info!("KurisuGal v{} 启动完成", env!("CARGO_PKG_VERSION"));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_logging() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .with_utc_timestamps()
        .init()
        .unwrap_or_default();
    #[cfg(debug_assertions)]
    log::set_max_level(log::LevelFilter::Debug);
    #[cfg(not(debug_assertions))]
    log::set_max_level(log::LevelFilter::Info);
}
