//! 插件管理器模块
//! 
//! 负责管理和执行插件。

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use serde_json::Value as JsonValue;

use crate::config::Value;
use super::{Plugin, PluginLoader, loader::PluginInfo};

/// 插件管理器
pub struct PluginManager {
    /// 配置值
    value: Arc<Value>,
    /// 插件加载器
    loader: PluginLoader,
    /// 已加载的插件信息
    plugin_infos: Vec<PluginInfo>,
    /// 已加载的插件实例
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new(value: Arc<Value>) -> Self {
        Self {
            value: value.clone(),
            loader: PluginLoader::new(value),
            plugin_infos: Vec::new(),
            plugins: HashMap::new(),
        }
    }
    
    /// 加载插件
    pub async fn load_plugins(&mut self) -> Result<()> {
        // 扫描插件目录
        self.plugin_infos = self.loader.scan_plugins()?;
        
        // 在实际实现中，这里会加载插件实例
        // 由于我们使用Python插件，这里需要使用PyO3或其他方式加载
        // 这里只是一个示例实现
        
        tracing::info!("已加载 {} 个插件信息", self.plugin_infos.len());
        
        Ok(())
    }
    
    /// 初始化插件
    pub async fn initialize_plugins(&mut self) -> Result<()> {
        // 在实际实现中，这里会调用每个插件的on_load方法
        // 这里只是一个示例实现
        
        for plugin in self.plugins.values() {
            if let Err(e) = plugin.on_load() {
                tracing::error!("初始化插件 {} 失败: {}", plugin.name(), e);
            } else {
                tracing::info!("初始化插件 {} 成功", plugin.name());
            }
        }
        
        Ok(())
    }
    
    /// 卸载插件
    pub async fn unload_plugins(&mut self) -> Result<()> {
        // 在实际实现中，这里会调用每个插件的on_unload方法
        // 这里只是一个示例实现
        
        for plugin in self.plugins.values() {
            if let Err(e) = plugin.on_unload() {
                tracing::error!("卸载插件 {} 失败: {}", plugin.name(), e);
            } else {
                tracing::info!("卸载插件 {} 成功", plugin.name());
            }
        }
        
        // 清空插件列表
        self.plugins.clear();
        
        Ok(())
    }
    
    /// 处理消息
    pub async fn handle_message(&self, message_type: &str, data: &JsonValue) -> Result<bool> {
        let mut handled = false;
        
        // 依次调用每个插件的on_message方法
        for plugin in self.plugins.values() {
            match plugin.on_message(message_type, data) {
                Ok(true) => {
                    // 插件已处理消息
                    handled = true;
                    break;
                }
                Err(e) => {
                    tracing::error!("插件 {} 处理消息失败: {}", plugin.name(), e);
                }
                _ => {}
            }
        }
        
        Ok(handled)
    }
    
    /// 处理通知
    pub async fn handle_notice(&self, notice_type: &str, data: &JsonValue) -> Result<bool> {
        let mut handled = false;
        
        // 依次调用每个插件的on_notice方法
        for plugin in self.plugins.values() {
            match plugin.on_notice(notice_type, data) {
                Ok(true) => {
                    // 插件已处理通知
                    handled = true;
                    break;
                }
                Err(e) => {
                    tracing::error!("插件 {} 处理通知失败: {}", plugin.name(), e);
                }
                _ => {}
            }
        }
        
        Ok(handled)
    }
    
    /// 获取插件信息列表
    pub fn get_plugin_infos(&self) -> &[PluginInfo] {
        &self.plugin_infos
    }
}