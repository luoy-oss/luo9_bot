// src/plugin/mod.rs
pub mod bus;
pub mod manager;
pub mod loader;
pub mod data;
pub mod task;
pub mod sender;

// 重新导出常用类型和函数
pub use bus::publish_data;
pub use manager::{
    PluginManager,
    PluginInfo,
    GLOBAL_PLUGIN_MANAGER,
    init_global_manager,
    get_manager_stats
};
pub use loader::PluginLoader;

use tracing::{error, info};
use crate::message::Message;
use crate::event::MetaEvent;
use crate::notice::Notice;
use data::PluginData;

/// 初始化插件系统
pub async fn initialize(plugins_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("正在初始化插件系统...");

    // 初始化 FFI 总线
    if let Err(e) = bus::Bus::init(1024) {
        error!("FFI 总线初始化失败: {:?}", e);
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Bus init failed")));
    }

    // 加载插件（加载后自动启动各插件的 plugin_main 线程）
    let loader = PluginLoader::new(plugins_dir);
    let infos = loader.load_all()?;

    info!("成功加载 {} 个插件", infos.len());

    // 注册插件信息到全局管理器
    init_global_manager(infos).await;

    // 启动 task 总线接收器
    task::start_task_receiver();

    // 启动消息发送接收器
    sender::start_send_receiver();

    info!("插件系统初始化完成");
    Ok(())
}

/// 统一的消息分发函数
pub fn dispatch_message(msg: Message) {
    bus::publish_data(&PluginData::Message(msg));
}

/// 分发元事件
pub fn dispatch_meta_event(event: MetaEvent) {
    bus::publish_data(&PluginData::MetaEvent(event));
}

/// 分发通知
pub fn dispatch_notice(notice: Notice) {
    bus::publish_data(&PluginData::Notice(notice));
}

/// 获取插件系统状态
pub async fn status() -> String {
    get_manager_stats().await
}
