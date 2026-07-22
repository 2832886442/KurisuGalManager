use serde::{Deserialize, Serialize};

/// 游戏条目
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub alias: Option<String>,
    pub path: String,
    /// 封面文件名（存储在 CoverArt/ 目录下），空字符串表示无封面
    #[serde(default)]
    pub cover: String,
    pub category: String,
    pub tags: Vec<String>,
    /// 状态: 未游玩 | 游玩中 | 已通关 | 搁置
    pub status: String,
    /// 简介
    #[serde(default)]
    pub description: String,
    /// 游玩时长（分钟）
    pub play_time: u64,
    /// ISO 日期字符串
    pub last_play: Option<String>,
    pub favorite: bool,
    /// 截图文件名列表
    #[serde(default)]
    pub screenshots: Vec<String>,
}

fn default_color() -> String { "#9ca3af".to_string() }

/// 排名等级
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RankLevel {
    pub level: i32,
    pub name: String,
    pub game_ids: Vec<String>,
    #[serde(default = "default_color")]
    pub color: String,
}

/// 排名
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ranking {
    pub id: String,
    pub name: String,
    pub levels: Vec<RankLevel>,
    pub created_at: String,
    pub updated_at: String,
}

/// Bangumi 查询返回的填充数据（前端用于覆盖表单字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiFillData {
    pub name: String,
    pub name_cn: String,
    pub summary: String,
    pub cover: String, // Base64 data URI（前端填充用，添加游戏时才转为文件）
    pub tags: Vec<String>,
    pub date: String,
}

/// 应用主数据
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AppData {
    pub games: Vec<Game>,
    #[serde(default)]
    pub rankings: Vec<Ranking>,
}

/// 应用设置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub theme: String,
    pub window_radius: u32,
    pub zoom: f64,
    pub startup: bool,
    pub close_action: String,
    pub default_view: String,
    /// 自定义数据存储路径（空字符串 = 使用默认 install_dir/Data）
    #[serde(default)]
    pub data_root: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            window_radius: 14,
            zoom: 1.0,
            startup: false,
            close_action: "tray".into(),
            default_view: "grid".into(),
            data_root: String::new(),
        }
    }
}
