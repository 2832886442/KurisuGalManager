use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub alias: Option<String>,
    pub path: String,
    pub cover: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub status: String, // 未游玩, 游玩中, 已通关, 搁置
    pub description: Option<String>,
    pub play_time: u64,       // 分钟
    pub last_play: Option<String>, // ISO 日期
    pub favorite: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppData {
    pub games: Vec<Game>,
    // 分类和标签由游戏数据动态生成，不单独存储
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String, // "light", "dark", "system"
    pub window_radius: u32,
    pub zoom: f64,
    pub startup: bool,
    pub close_action: String, // "exit" or "tray"
    pub default_view: String, // "grid" or "list"
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            window_radius: 14,
            zoom: 1.0,
            startup: false,
            close_action: "tray".to_string(),
            default_view: "grid".to_string(),
        }
    }
}