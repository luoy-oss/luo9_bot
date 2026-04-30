// src/plugin/mod.rs
pub mod bus;
pub mod subscriber;
pub mod publisher;
pub mod manager;
pub mod loader;
pub mod data;
pub mod task_bus;
pub mod task;

// 重新导出常用类型和函数
pub use bus::{publish, subscribe, get_bus_sender, stats as bus_stats};
pub use publisher::PluginMessagePublisher;
pub use subscriber::{
    PluginSubscriber,
    PluginProcessResult
};
pub use manager::{
    PluginSubscriptionManager, 
    PluginInfo,
    GLOBAL_PLUGIN_MANAGER,
    init_global_manager,
    get_manager_stats
};
pub use loader::PluginLoader;

use tracing::info;
use crate::message::Message;
use crate::event::MetaEvent;
use crate::notice::Notice;

/// 初始化插件系统
pub async fn initialize(plugins_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("正在初始化插件系统...");
    
    // 加载插件
    let loader = PluginLoader::new(plugins_dir);
    let plugins = loader.load_all()?;
    
    info!("成功加载 {} 个插件", plugins.len());
    
    // 初始化全局管理器
    init_global_manager(plugins).await;

    // 启动 task 总线接收器
    task::start_task_receiver();

    info!("插件系统初始化完成");
    Ok(())
}


/// 统一的消息分发函数
pub fn dispatch_message(msg: Message) {
    PluginMessagePublisher::publish_message(msg);
}

/// 分发元事件
pub fn dispatch_meta_event(event: MetaEvent) {
    PluginMessagePublisher::publish_meta_event(event);
}

/// 分发通知
pub fn dispatch_notice(notice: Notice) {
    PluginMessagePublisher::publish_notice(notice);
}

/// 获取插件系统状态
pub async fn status() -> String {
    get_manager_stats().await
}