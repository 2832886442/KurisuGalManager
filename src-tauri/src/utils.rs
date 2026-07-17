use anyhow::Result;
use walkdir::WalkDir;

/// 扫描文件夹内的所有 .exe 文件
pub fn scan_exe_files(folder: &str) -> Result<Vec<String>> {
    let mut exes = Vec::new();
    for entry in WalkDir::new(folder)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "exe") {
            exes.push(path.to_string_lossy().to_string());
        }
    }
    Ok(exes)
}