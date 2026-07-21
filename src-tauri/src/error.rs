use serde::Serialize;
use thiserror::Error;

/// 统一错误码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    DataReadFailed,
    DataWriteFailed,
    DataSerializeFailed,
    DataDeserializeFailed,
    GameNotFound,
    GameLaunchFailed,
    GameNotRunning,
    FileNotFound,
    FileReadFailed,
    PathInvalid,
    PathNotAbsolute,
    ScanFailed,
    FolderNotFound,
    BackupFailed,
    RestoreFailed,
    SettingsLoadFailed,
    SettingsSaveFailed,
    StartupConfigFailed,
    InternalError,
    InvalidInput,
}

/// 统一错误类型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("[{code:?}] {message}")]
    Business { code: ErrorCode, message: String },

    #[error("[{code:?}] {message}: {source}")]
    Wrapped {
        code: ErrorCode,
        message: String,
        #[source]
        source: anyhow::Error,
    },
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::Business {
            code,
            message: message.into(),
        }
    }

    pub fn wrap(
        code: ErrorCode,
        message: impl Into<String>,
        error: impl Into<anyhow::Error>,
    ) -> Self {
        Self::Wrapped {
            code,
            message: message.into(),
            source: error.into(),
        }
    }
}

// 便捷构造方法
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::wrap(ErrorCode::InternalError, "IO 操作失败", e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::wrap(ErrorCode::DataSerializeFailed, "数据序列化/反序列化失败", e)
    }
}

/// 将 AppError 转换为前端可用的字符串（仅包含用户友好消息）
pub fn to_frontend_error(err: AppError) -> String {
    err.to_string()
}
