//! 插件管理器模块
//! 
//! 这个模块提供了一个泛型的插件系统，用于加载和管理各种插件。

use std::fs;
use std::path::Path;
use std::sync::Arc;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_yaml;

use crate::core::plugin_registry;

use crate::config::Value;
use crate::core::message::{GroupMessage, PrivateMessage};


/// 插件配置结构
#[derive(Debug, Deserialize)]
struct PluginConfig {
    plugins: Vec<PluginEntry>,
}

/// 插件条目结构
#[derive(Debug, Deserialize)]
struct PluginEntry {
    name: String,
    enable: bool,
    priority: i32,
}

/// 插件元数据结构
#[derive(Debug, Deserialize, Serialize)]
pub struct PluginMetadata {
    pub name: String,
    pub describe: String,
    pub author: String,
    pub version: String,
    pub message_types: Vec<String>,
}

/// 插件特性，定义了插件需要实现的方法
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 获取插件元数据
    fn metadata(&self) -> &PluginMetadata;
    
    /// 处理群消息
    async fn handle_group_message(&self, message: &GroupMessage) -> Result<()>;
    
    /// 处理私聊消息
    async fn handle_private_message(&self, message: &PrivateMessage) -> Result<()>;
    
    /// 处理群戳一戳事件
    async fn handle_group_poke(&self, target_id: &str, user_id: &str, group_id: &str) -> Result<()>;
}

/// 插件信息结构
struct PluginInfo {
    plugin: Box<dyn Plugin>,
    priority: i32,
}

/// 插件管理器
pub struct PluginManager {
    plugins: Vec<PluginInfo>,
    config: Arc<Value>,
}

impl PluginManager {
    /// 创建一个新的插件管理器
    pub async fn new(config: Arc<Value>) -> Result<Self> {
        let mut manager = Self {
            plugins: Vec::new(),
            config: config.clone(),
        };
        
        manager.load_plugins().await?;
        
        Ok(manager)
    }
    
    /// 加载插件
    async fn load_plugins(&mut self) -> Result<()> {
        let plugin_dir = &self.config.plugin_path;
        let config_path = Path::new(plugin_dir).join("config.yaml");
        tracing::info!("插件配置文件路径：{}", config_path.display());
        
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| anyhow!("无法读取插件配置文件: {}", e))?;
        
        let config: PluginConfig = serde_yaml::from_str(&config_content)
            .map_err(|e| anyhow!("解析插件配置文件失败: {}", e))?;

        tracing::info!("插件总数：{}", config.plugins.len());        
        tracing::info!("---------------------------");();
        
        let mut load_num = 0;
        
        for plugin_entry in &config.plugins {
            if plugin_entry.enable {
                let plugin_name = &plugin_entry.name;
                let plugin_path = Path::new(plugin_dir).join(plugin_name);
                
                if plugin_path.is_dir() {
                    // 创建插件数据目录
                    let plugin_data_path = Path::new(&self.config.data_path).join("plugins").join(plugin_name);
                    tracing::info!("插件数据目录：{}", plugin_data_path.display());
                    
                    if !plugin_data_path.exists() {
                        fs::create_dir_all(&plugin_data_path)
                            .map_err(|e| anyhow!("创建插件数据目录失败: {}", e))?;
                        
                        #[cfg(not(target_os = "windows"))]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let metadata = fs::metadata(&plugin_data_path)?;
                            let mut permissions = metadata.permissions();
                            permissions.set_mode(0o777);
                            fs::set_permissions(&plugin_data_path, permissions)?;
                        }
                    }
                    
                    // 加载插件
                    match self.load_plugin(plugin_name, plugin_entry.priority).await {
                        Ok(plugin) => {
                            let metadata = plugin.metadata();                            
                            tracing::info!(
                                "\"插件\'：{}, \"作者\"：{}, \"插件描述\"：{}, \"版本\"：{}, \"插件需求\"：{:?}",
                                metadata.name, metadata.author, metadata.describe, metadata.version, metadata.message_types
                            );
                            tracing::info!("---------------------------");();
                            
                            self.plugins.push(PluginInfo {
                                plugin,
                                priority: plugin_entry.priority,
                            });
                            
                            load_num += 1;
                        },
                        Err(e) => {
                            tracing::info!("加载插件 {} 失败: {}", plugin_name, e);
                            tracing::info!("---------------------------");();
                        }
                    }
                }
            }
        }
        
        tracing::info!("加载完成：{}/{}", load_num, config.plugins.len());
        tracing::info!("---------------------------");();
        
        Ok(())
    }
    
    /// 加载单个插件
    async fn load_plugin(&self, name: &str, _priority: i32) -> Result<Box<dyn Plugin>> {
        // 首先尝试从注册表中创建插件
        let registry = plugin_registry::PLUGIN_REGISTRY.lock().unwrap();
        
        // 如果插件已经注册，直接创建
        if let Ok(plugin) = registry.create(name, self.config.clone()) {
            return Ok(plugin);
        }
        
        // 如果插件未注册，尝试加载外部插件
        drop(registry); // 释放锁
        
        let plugin_dir = Path::new(&self.config.plugin_path);
        
        // 使用 spawn_blocking 包装可能阻塞的文件系统操作
        let name_clone = name.to_string();
        let plugin_dir_clone = plugin_dir.to_path_buf();
        let config_clone = self.config.clone();
        
        // 在单独的线程中执行插件加载操作
        let plugin = tokio::task::spawn_blocking(move || {
            let mut registry = plugin_registry::PLUGIN_REGISTRY.lock().unwrap();
            
            tracing::info!("尝试加载外部插件:{}", name_clone);

            if let Err(e) = registry.load_external_plugin(&plugin_dir_clone, &name_clone) {
                return Err(e);
            }
            tracing::info!("插件加载完成:{}", name_clone);
            tracing::info!("---------------------------");();
            // 创建插件实例
            registry.create(&name_clone, config_clone)
        }).await??;
        
        Ok(plugin)
    }
    
    /// 处理群消息
    pub async fn handle_group_message(&self, message: &GroupMessage) -> Result<()> {
        // 按优先级排序插件
        let mut plugins = self.plugins.iter().collect::<Vec<_>>();
        plugins.sort_by_key(|p| p.priority);
        
        for plugin_info in plugins {
            let metadata = plugin_info.plugin.metadata();
            if metadata.message_types.contains(&"group_message".to_string()) {
                plugin_info.plugin.handle_group_message(message).await?;
            }
        }
        
        Ok(())
    }
    
    /// 处理私聊消息
    pub async fn handle_private_message(&self, message: &PrivateMessage) -> Result<()> {
        // 按优先级排序插件
        let mut plugins = self.plugins.iter().collect::<Vec<_>>();
        plugins.sort_by_key(|p| p.priority);
        
        for plugin_info in plugins {
            let metadata = plugin_info.plugin.metadata();
            if metadata.message_types.contains(&"private_message".to_string()) {
                plugin_info.plugin.handle_private_message(message).await?;
            }
        }
        
        Ok(())
    }
    
    /// 处理群戳一戳事件
    pub async fn handle_group_poke(&self, target_id: &str, user_id: &str, group_id: &str) -> Result<()> {
        // 按优先级排序插件
        let mut plugins = self.plugins.iter().collect::<Vec<_>>();
        plugins.sort_by_key(|p| p.priority);
        
        for plugin_info in plugins {
            let metadata = plugin_info.plugin.metadata();
            if metadata.message_types.contains(&"group_poke".to_string()) {
                plugin_info.plugin.handle_group_poke(target_id, user_id, group_id).await?;
            }
        }
        
        Ok(())
    }

}
