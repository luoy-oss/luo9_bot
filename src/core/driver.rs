//! 驱动模块
//! 
//! 负责机器人的启动和关闭流程。

use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Mutex;
use crate::config::Value;
use crate::core::plugin_manager::PluginManager;

/// 机器人驱动
pub struct Driver {
    /// 插件管理器
    plugin_manager: Arc<Mutex<PluginManager>>,
}

impl Driver {
    /// 创建新的驱动实例
    pub async fn new(value: Arc<Value>) -> Result<Self> {
        // 异步创建插件管理器
        let plugin_manager = PluginManager::new(value.clone()).await?;
        
        Ok(Self {
            plugin_manager: Arc::new(Mutex::new(plugin_manager)),
        })
    }
    
    /// 运行启动流程
    pub async fn run_startup(&mut self) -> Result<()> {
        tracing::info!("正在启动驱动...");
        
        // 不需要显式加载插件，因为在创建插件管理器时已经加载了
        // 初始化插件
        let _plugin_manager = self.plugin_manager.lock().await;
        
        tracing::info!("驱动启动完成");
        Ok(())
    }
    
    /// 运行关闭流程
    pub async fn run_shutdown(&mut self) -> Result<()> {
        tracing::info!("正在关闭驱动...");
        
        // 卸载插件
        let _plugin_manager = self.plugin_manager.lock().await;
        // 如果需要，可以添加卸载插件的方法调用
        
        tracing::info!("驱动关闭完成");
        Ok(())
    }
    
    /// 获取插件管理器的引用
    pub fn plugin_manager(&self) -> &Arc<Mutex<PluginManager>> {
        &self.plugin_manager
    }
}