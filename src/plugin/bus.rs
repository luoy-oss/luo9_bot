// src/plugin/bus.rs
use lazy_static::lazy_static;
use tokio::sync::broadcast;
use tracing::info;
use super::data::PluginData;

lazy_static! {
    /// 全局插件数据总线
    pub static ref PLUGIN_DATA_BUS: broadcast::Sender<PluginData> = {
        let (tx, _) = broadcast::channel(10000);
        info!("插件数据总线已初始化，缓冲区大小: 10000");
        tx
    };
}

/// 获取插件数据总线发送端
pub fn get_bus_sender() -> broadcast::Sender<PluginData> {
    PLUGIN_DATA_BUS.clone()
}

/// 获取新的插件数据接收者
pub fn subscribe() -> broadcast::Receiver<PluginData> {
    PLUGIN_DATA_BUS.subscribe()
}

/// 发布插件数据到总线
pub fn publish(data: PluginData) -> Result<usize, broadcast::error::SendError<PluginData>> {
    PLUGIN_DATA_BUS.send(data)
}

/// 获取当前接收者数量
pub fn receiver_count() -> usize {
    PLUGIN_DATA_BUS.receiver_count()
}

/// 检查总线是否活跃
pub fn is_active() -> bool {
    receiver_count() > 0
}

/// 获取总线统计信息
pub fn stats() -> BusStats {
    BusStats {
        receiver_count: receiver_count(),
        capacity: 10000,
        is_active: is_active(),
    }
}

#[derive(Debug)]
pub struct BusStats {
    pub receiver_count: usize,
    pub capacity: usize,
    pub is_active: bool,
}

impl std::fmt::Display for BusStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "消息总线状态: 接收者={}, 容量={}, 活跃={}",
            self.receiver_count, self.capacity, self.is_active
        )
    }
}