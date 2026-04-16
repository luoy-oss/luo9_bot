// src/plugin/manager.rs
use std::sync::Arc;
use libloading::Library;
use tokio::sync::Mutex;
use tracing::info;

use super::subscriber::{PluginSubscriber, PluginProcessResult};
use super::bus;

/// 插件信息
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: usize,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub path: Option<String>,
}

/// 插件订阅管理器
pub struct PluginSubscriptionManager {
    subscribers: Vec<PluginSubscriber>,
    plugin_infos: Vec<PluginInfo>,
    stats: PluginStats,
}

#[derive(Debug, Default)]
pub struct PluginStats {
    pub total_plugins: usize,
    pub active_plugins: usize,
    pub total_messages_processed: u64,
    pub total_errors: u64,
}

impl PluginSubscriptionManager {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            plugin_infos: Vec::new(),
            stats: PluginStats::default(),
        }
    }
    
    /// 注册单个插件
    pub fn register_plugin(&mut self, lib: Arc<Library>, info: PluginInfo) {
        let subscriber = PluginSubscriber::new(Arc::clone(&lib), info.id);
        subscriber.start();
        
        self.subscribers.push(subscriber);
        self.plugin_infos.push(info.clone());
        self.stats.total_plugins += 1;
        self.stats.active_plugins += 1;
        
        info!("插件 #{} 已注册到消息总线", info.id);
    }
        
    /// 批量注册插件
    pub fn register_all_plugins(&mut self, plugins: Vec<(Arc<Library>, PluginInfo)>) {
        for (lib, info) in plugins {
            self.register_plugin(lib, info);
        }
        info!("共注册 {} 个插件到消息总线", self.stats.total_plugins);
    }
    
    /// 获取插件信息
    pub fn get_plugin_info(&self, plugin_id: usize) -> Option<&PluginInfo> {
        self.plugin_infos.iter().find(|info| info.id == plugin_id)
    }
    
    /// 获取所有插件信息
    pub fn get_all_plugins(&self) -> &[PluginInfo] {
        &self.plugin_infos
    }
    
    /// 获取总线统计信息
    pub fn get_bus_stats(&self) -> bus::BusStats {
        bus::stats()
    }
    
    /// 获取插件统计信息
    pub fn get_stats(&self) -> &PluginStats {
        &self.stats
    }
    
    /// 更新统计信息
    pub fn update_stats(&mut self, result: &PluginProcessResult) {
        self.stats.total_messages_processed += 1;
        if !result.success {
            self.stats.total_errors += 1;
        }
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_PLUGIN_MANAGER: Mutex<PluginSubscriptionManager> = {
        Mutex::new(PluginSubscriptionManager::new())
    };
}

/// 初始化全局插件管理器
pub async fn init_global_manager(plugins: Vec<(Arc<Library>, PluginInfo)>) {
    let mut manager = GLOBAL_PLUGIN_MANAGER.lock().await;
    manager.register_all_plugins(plugins);
    info!("全局插件管理器初始化完成");
}

/// 获取插件管理器统计信息
pub async fn get_manager_stats() -> String {
    let manager = GLOBAL_PLUGIN_MANAGER.lock().await;
    format!(
        "插件统计: 总数={}, 活跃={}, 已处理消息={}, 错误数={}\n{}",
        manager.stats.total_plugins,
        manager.stats.active_plugins,
        manager.stats.total_messages_processed,
        manager.stats.total_errors,
        manager.get_bus_stats()
    )
}