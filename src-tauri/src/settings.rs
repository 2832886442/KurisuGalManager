use crate::data_store;
use crate::models::Settings;
use anyhow::Result;
use tauri::AppHandle;

pub fn load_settings(app_handle: &AppHandle) -> Result<Settings> {
    data_store::read_settings(app_handle)
}

pub fn save_settings(app_handle: &AppHandle, settings: &Settings) -> Result<()> {
    data_store::write_settings(app_handle, settings)?;
    // 处理开机自启（使用 auto-launch 库）
    set_startup_enabled(app_handle, settings.startup)?;
    Ok(())
}

fn set_startup_enabled(app_handle: &AppHandle, enabled: bool) -> Result<()> {
    let app_name = &app_handle.config().tauri.bundle.identifier;
    let app_path = std::env::current_exe()?.to_string_lossy().to_string();
    let launch = auto_launch::AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&app_path)
        .set_use_launch_agent(false) // Windows 使用注册表
        .build()?;
    if enabled {
        launch.enable()?;
    } else {
        launch.disable()?;
    }
    Ok(())
}