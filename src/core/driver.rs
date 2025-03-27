//! 驱动模块
//! 
//! 负责机器人的启动和关闭流程。

use std::sync::Arc;
use anyhow::Result;
use crate::config::Value;
use crate::plugins::PluginManager;

/// 机器人驱动
pub struct Driver {
    /// 配置值
    value: Arc<Value>,
    /// 插件管理器
    plugin_manager: PluginManager,
}

impl Driver {
    /// 创建新的驱动实例
    pub fn new(value: Arc<Value>) -> Self {
        Self {
            value: value.clone(),
            plugin_manager: PluginManager::new(value),
        }
    }
    
    /// 运行启动流程
    pub async fn run_startup(&mut self) -> Result<()> {
        tracing::info!("正在启动驱动...");
        
        // 加载插件
        self.plugin_manager.load_plugins().await?;
        
        // 初始化插件
        self.plugin_manager.initialize_plugins().await?;
        
        tracing::info!("驱动启动完成");
        Ok(())
    }
    
    /// 运行关闭流程
    pub async fn run_shutdown(&mut self) -> Result<()> {
        tracing::info!("正在关闭驱动...");
        
        // 卸载插件
        self.plugin_manager.unload_plugins().await?;
        
        tracing::info!("驱动关闭完成");
        Ok(())
    }
    
    /// 获取插件管理器的引用
    pub fn plugin_manager(&self) -> &PluginManager {
        &self.plugin_manager
    }
    
    /// 获取插件管理器的可变引用
    pub fn plugin_manager_mut(&mut self) -> &mut PluginManager {
        &mut self.plugin_manager
    }
}