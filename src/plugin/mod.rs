// src/plugin/mod.rs
pub mod bus;
pub mod manager;
pub mod loader;
pub mod handle;
pub mod data;
pub mod task;
pub mod sender;
pub mod version;
pub mod dispatch;

// 重新导出常用类型和函数
pub use manager::{
    PluginManager,
    PluginInfo,
    GLOBAL_PLUGIN_MANAGER,
    init_global_manager,
    get_manager_stats
};
pub use loader::PluginLoader;
pub use dispatch::{update_dispatch_list, priority_dispatch_message, priority_dispatch_notice, priority_dispatch_meta_event};

use tracing::{error, info};
use crate::message::Message;
use crate::event::MetaEvent;
use crate::notice::Notice;

/// 初始化插件系统
pub async fn initialize(plugins_dir: &str, config_entries: &[crate::config::PluginEntry]) -> Result<(), Box<dyn std::error::Error>> {
    info!("正在初始化插件系统...");

    // 初始化 FFI 总线（无界队列，不丢消息）
    if let Err(e) = bus::Bus::init() {
        error!("FFI 总线初始化失败: {:?}", e);
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Bus init failed")));
    }

    // 先启动总线接收器，再加载插件，避免插件发布消息时接收器尚未订阅
    task::start_task_receiver();
    sender::start_send_receiver();

    // 加载插件（加载后自动启动各插件的 plugin_main 线程，并创建 subscriber）
    let plugin_loader = PluginLoader::new(plugins_dir);
    let (infos, handles) = plugin_loader.load_all()?;

    info!("成功加载 {} 个插件", infos.len());

    // 注册插件信息和句柄到全局管理器（应用配置中的 priority/block_enabled）
    init_global_manager(infos, handles, config_entries).await;

    // 初始化优先级分发列表
    let dispatch_entries = {
        let manager = GLOBAL_PLUGIN_MANAGER.lock().await;
        manager.get_dispatch_list()
    };
    update_dispatch_list(dispatch_entries);

    // 查询插件版本（5秒超时）
    version::query_versions(std::time::Duration::from_secs(5)).await;

    info!("插件系统初始化完成");
    Ok(())
}

/// 统一的消息分发函数（使用优先级分发）
pub fn dispatch_message(msg: Message) {
    priority_dispatch_message(msg);
}

/// 分发元事件（使用优先级分发）
pub fn dispatch_meta_event(event: MetaEvent) {
    priority_dispatch_meta_event(event);
}

/// 分发通知（使用优先级分发）
pub fn dispatch_notice(notice: Notice) {
    priority_dispatch_notice(notice);
}

/// 启用插件（运行时热加载）
pub async fn enable_plugin(
    name: &str,
    path: &std::path::Path,
    config_entries: &[crate::config::PluginEntry],
) -> Result<String, String> {
    let mut manager = GLOBAL_PLUGIN_MANAGER.lock().await;
    manager.enable_plugin(name, path, config_entries).await
}

/// 热重载插件（禁用后重新加载）
pub async fn reload_plugin(
    name: &str,
    config_entries: &[crate::config::PluginEntry],
) -> Result<String, String> {
    let mut manager = GLOBAL_PLUGIN_MANAGER.lock().await;
    manager.reload_plugin(name, config_entries).await
}

/// 获取插件系统状态
pub async fn status() -> String {
    get_manager_stats().await
}
