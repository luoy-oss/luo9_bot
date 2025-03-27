//! 文件操作工具

use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

/// 确保目录存在，如果不存在则创建
pub fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 获取文件扩展名
pub fn get_file_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

/// 列出目录中的所有文件
pub fn list_files<P: AsRef<Path>>(dir: P, extension: Option<&str>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = extension {
                if let Some(file_ext) = get_file_extension(&path) {
                    if file_ext == ext {
                        files.push(path);
                    }
                }
            } else {
                files.push(path);
            }
        }
    }
    
    Ok(files)
}