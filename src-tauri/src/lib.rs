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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

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
            commands::get_data_root,
            commands::set_data_root,
            commands::get_data_size_info,
            commands::pick_folder,
        ])
        .setup(|app| {
            // 初始化路径管理器
            let install_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            crate::path_manager::init_from_config(install_dir);

            // 迁移旧数据
            if let Err(e) = crate::path_manager::migrate_old_data(&app.handle()) {
                log::warn!("旧数据迁移失败: {}", e);
            }

            log::info!("KurisuGal v1.2.5 启动完成");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_logging() {
    #[cfg(debug_assertions)]
    log::set_max_level(log::LevelFilter::Debug);
    #[cfg(not(debug_assertions))]
    log::set_max_level(log::LevelFilter::Info);
}
