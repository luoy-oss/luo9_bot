//! 插件加载器模块
//! 
//! 负责从文件系统加载插件。

use std::path::Path;
use std::sync::Arc;
use anyhow::{Result, Context};
// 移除未使用的导入
// use walkdir::WalkDir;
use regex::Regex;
use serde::Deserialize;

use crate::config::Value;
// 移除未使用的导入
// use super::Plugin;

/// 插件信息
#[derive(Debug, Clone, Deserialize)]
pub struct PluginInfo {
    /// 插件名称
    pub name: String,
    /// 插件描述
    #[serde(rename = "describe")]
    pub description: String,
    /// 插件作者
    pub author: String,
}

/// 插件加载器
pub struct PluginLoader {
    /// 配置值
    value: Arc<Value>,
}

impl PluginLoader {
    /// 创建新的插件加载器
    pub fn new(value: Arc<Value>) -> Self {
        Self { value }
    }
    
    /// 扫描插件目录
    pub fn scan_plugins(&self) -> Result<Vec<PluginInfo>> {
        let plugin_dir = &self.value.plugin_path;
        let mut plugins = Vec::new();
        
        tracing::info!("正在扫描插件目录: {}", plugin_dir);
        
        // 遍历插件目录
        for entry in std::fs::read_dir(plugin_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // 检查是否是目录且包含main.py
            if path.is_dir() && path.join("main.py").exists() {
                // 尝试读取插件信息
                match self.extract_plugin_info(&path) {
                    Ok(info) => {
                        tracing::info!("发现插件: {} ({})", info.name, info.description);
                        plugins.push(info);
                    }
                    Err(e) => {
                        tracing::warn!("无法读取插件信息 {}: {}", path.display(), e);
                        // 添加默认信息
                        let plugin_name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        plugins.push(PluginInfo {
                            name: plugin_name.clone(),
                            description: format!("{} 插件", plugin_name),
                            author: "未知".to_string(),
                        });
                    }
                }
            }
        }
        
        // 按名称排序
        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        
        tracing::info!("共发现 {} 个插件", plugins.len());
        Ok(plugins)
    }
    
    /// 从插件目录提取插件信息
    fn extract_plugin_info(&self, plugin_path: &Path) -> Result<PluginInfo> {
        let main_py_path = plugin_path.join("main.py");
        
        // 默认信息
        let mut info = PluginInfo {
            name: plugin_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            description: String::new(),
            author: "未知".to_string(),
        };
        
        // 尝试从main.py中提取配置
        if main_py_path.exists() {
            let content = std::fs::read_to_string(&main_py_path)
                .with_context(|| format!("无法读取 {}", main_py_path.display()))?;
            
            // 使用正则表达式提取配置
            let re_config = Regex::new(r"config\s*=\s*\{([^}]*)\}").unwrap();
            if let Some(caps) = re_config.captures(&content) {
                let config_str = caps.get(1).unwrap().as_str();
                
                // 提取name
                let re_name = Regex::new(r"'name'\s*:\s*'([^']*)'").unwrap();
                if let Some(caps) = re_name.captures(config_str) {
                    info.name = caps.get(1).unwrap().as_str().to_string();
                }
                
                // 提取describe
                let re_desc = Regex::new(r"'describe'\s*:\s*'([^']*)'").unwrap();
                if let Some(caps) = re_desc.captures(config_str) {
                    info.description = caps.get(1).unwrap().as_str().to_string();
                }
                
                // 提取author
                let re_author = Regex::new(r"'author'\s*:\s*'([^']*)'").unwrap();
                if let Some(caps) = re_author.captures(config_str) {
                    info.author = caps.get(1).unwrap().as_str().to_string();
                }
            }
        }
        
        // 如果仍然没有描述，尝试从info.yaml读取
        if info.description.is_empty() {
            let info_yaml_path = plugin_path.join("info.yaml");
            if info_yaml_path.exists() {
                let yaml_str = std::fs::read_to_string(&info_yaml_path)?;
                if let Ok(yaml_info) = serde_yaml::from_str::<serde_yaml::Value>(&yaml_str) {
                    if let Some(describe) = yaml_info["describe"].as_str() {
                        info.description = describe.to_string();
                    }
                }
            }
        }
        
        // 如果仍然没有描述，使用默认描述
        if info.description.is_empty() {
            info.description = format!("{} 插件", info.name);
        }
        
        Ok(info)
    }
}