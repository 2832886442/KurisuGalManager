use crate::data_store;
use crate::error::{AppError, ErrorCode};
use crate::models::Settings;
use log::{debug, warn};
use tauri::AppHandle;

/// 加载设置
pub fn load_settings() -> Result<Settings, AppError> {
    data_store::read_settings()
}

/// 保存设置
pub fn save_settings(app_handle: &AppHandle, settings: &Settings) -> Result<(), AppError> {
    validate_settings(settings)?;
    // 如果数据根目录变更，更新路径管理器
    if !settings.data_root.is_empty() {
        crate::path_manager::set_data_root(&settings.data_root)?;
    }
    data_store::write_settings(settings)?;
    // 处理开机自启
    if let Err(e) = set_startup_enabled(app_handle, settings.startup) {
        warn!("设置开机自启失败（非关键错误）: {}", e);
    }
    debug!("设置已保存");
    Ok(())
}

fn validate_settings(settings: &Settings) -> Result<(), AppError> {
    if !["light", "dark", "system"].contains(&settings.theme.as_str()) {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            format!("无效的主题设置: {}", settings.theme),
        ));
    }
    if settings.window_radius > 50 {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "窗口圆角不能超过 50px",
        ));
    }
    if settings.zoom < 0.5 || settings.zoom > 3.0 {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "缩放比例必须在 0.5-3.0 之间",
        ));
    }
    if !["exit", "tray"].contains(&settings.close_action.as_str()) {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            format!("无效的关闭行为设置: {}", settings.close_action),
        ));
    }
    if !["grid", "list"].contains(&settings.default_view.as_str()) {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            format!("无效的默认视图设置: {}", settings.default_view),
        ));
    }
    Ok(())
}

fn set_startup_enabled(app_handle: &AppHandle, enabled: bool) -> Result<(), AppError> {
    let app_name = &app_handle.config().identifier;
    let app_path = std::env::current_exe()
        .map_err(|e| AppError::wrap(ErrorCode::StartupConfigFailed, "获取可执行文件路径失败", e))?
        .to_string_lossy()
        .to_string();

    let launch = auto_launch::AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&app_path)
        .set_use_launch_agent(false)
        .build()
        .map_err(|e| AppError::wrap(ErrorCode::StartupConfigFailed, "创建启动项配置失败", e))?;

    if enabled {
        launch
            .enable()
            .map_err(|e| AppError::wrap(ErrorCode::StartupConfigFailed, "启用开机自启失败", e))?;
    } else {
        launch
            .disable()
            .map_err(|e| AppError::wrap(ErrorCode::StartupConfigFailed, "禁用开机自启失败", e))?;
    }
    Ok(())
}
