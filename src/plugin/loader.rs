// src/plugin/loader.rs
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use libloading::Library;
use tracing::{info, error};

use super::manager::PluginInfo;

/// 插件加载器
pub struct PluginLoader {
    plugin_dir: PathBuf,
}

impl PluginLoader {
    pub fn new<P: AsRef<Path>>(plugin_dir: P) -> Self {
        Self {
            plugin_dir: plugin_dir.as_ref().to_path_buf(),
        }
    }
    
    /// 加载所有插件
    pub fn load_all(&self) -> Result<Vec<(Arc<Library>, PluginInfo)>, String> {
        let mut plugins = Vec::new();
        
        // 确保插件目录存在
        if !self.plugin_dir.exists() {
            fs::create_dir_all(&self.plugin_dir)
                .map_err(|e| format!("创建插件目录失败: {}", e))?;
            info!("已创建插件目录: {:?}", self.plugin_dir);
            return Ok(plugins);
        }
        
        // 遍历目录加载插件
        let entries = fs::read_dir(&self.plugin_dir)
            .map_err(|e| format!("读取插件目录失败: {}", e))?;
        
        for (idx, entry) in entries.enumerate() {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();
            
            // 只加载动态库文件
            if !Self::is_plugin_file(&path) {
                continue;
            }
            
            match self.load_single(&path, idx) {
                Ok((lib, info)) => {
                    info!("成功加载插件: {} (ID: {})", info.name, info.id);
                    plugins.push((lib, info));
                }
                Err(e) => {
                    error!("加载插件失败 {:?}: {}", path, e);
                }
            }
        }
        
        Ok(plugins)
    }
    
    /// 加载单个插件
    fn load_single(&self, path: &Path, default_id: usize) -> Result<(Arc<Library>, PluginInfo), String> {
        unsafe {
            let lib = Library::new(path)
                .map_err(|e| format!("加载动态库失败: {}", e))?;
            
            let lib = Arc::new(lib);
            
            // 获取插件信息
            let info = PluginInfo {
                id: default_id,
                name: path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                version: "".to_string(),
                enabled: true,
                path: Some(path.to_string_lossy().to_string()),
            };
            
            Ok((lib, info))
        }
    }
    
    /// 检查是否为插件文件
    fn is_plugin_file(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        #[cfg(target_os = "linux")]
        return ext == "so";
        
        #[cfg(target_os = "windows")]
        return ext == "dll";
        
        #[allow(unreachable_code)]
        false
    }
}