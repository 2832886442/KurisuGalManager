use crate::error::{AppError, ErrorCode};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 扫描文件夹内所有 .exe 文件（win: .exe, linux: 无扩展名可执行, mac: .app）
pub fn scan_exe_files(folder: &str) -> Result<Vec<String>, AppError> {
    let folder_path = Path::new(folder);
    if !folder_path.is_dir() {
        return Err(AppError::new(
            ErrorCode::FolderNotFound,
            "指定的文件夹不存在",
        ));
    }

    let mut exes = Vec::new();
    let canonical_root = folder_path
        .canonicalize()
        .map_err(|e| AppError::wrap(ErrorCode::ScanFailed, "无法解析文件夹路径", e))?;

    for entry in WalkDir::new(&canonical_root)
        .follow_links(false)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let is_exe =
            cfg!(target_os = "windows") && path.extension().map_or(false, |ext| ext == "exe");
        #[cfg(not(target_os = "windows"))]
        let is_exe = {
            use std::os::unix::fs::PermissionsExt;
            path.metadata()
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        };

        if is_exe {
            exes.push(path.to_string_lossy().to_string());
        }
    }
    Ok(exes)
}

/// 验证路径安全性：必须是绝对路径，不能包含路径遍历
pub fn validate_path(path_str: &str) -> Result<PathBuf, AppError> {
    let path = Path::new(path_str);
    if !path.is_absolute() {
        return Err(AppError::new(
            ErrorCode::PathNotAbsolute,
            format!("路径必须是绝对路径: {}", path_str),
        ));
    }
    // 规范化路径，消除 .. 和 .
    let canonical = path
        .canonicalize()
        .map_err(|e| AppError::wrap(ErrorCode::PathInvalid, "路径无效或不存在", e))?;
    Ok(canonical)
}
