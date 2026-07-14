// src/plugin/handle.rs
use std::path::PathBuf;
use tracing::{info, warn};

use super::bus::Bus;
use super::runtime::PluginRuntime;

/// 插件运行时句柄
///
/// 持有 PluginRuntime trait 对象，支持原生 DLL、Python、Java 等多种运行时。
pub struct PluginHandle {
    pub name: String,
    pub runtime: Box<dyn PluginRuntime>,
    pub priority: i32,
    pub block_enabled: bool,
    pub active: bool,
    pub path: PathBuf,
}

impl PluginHandle {
    /// 取消所有 topic 的订阅，触发插件退出
    pub fn unsubscribe_all(&self) {
        for (topic, &sub_id) in self.runtime.subscriber_ids() {
            if let Err(e) = Bus::topic(topic).unsubscribe(sub_id) {
                warn!("插件 {} 取消订阅 {} 失败: {:?}", self.name, topic, e);
            } else {
                info!("插件 {} 已取消订阅 {} (sub_id={})", self.name, topic, sub_id);
            }
        }
    }

    /// 检查插件是否仍在运行
    pub fn is_alive(&self) -> bool {
        self.runtime.is_alive()
    }

    /// 等待插件退出（带超时）
    pub fn wait_exit(&mut self, _timeout: std::time::Duration) -> bool {
        self.runtime.stop().is_ok()
    }

    /// 强制等待插件退出（用于文件删除前）
    pub fn force_wait_exit(&mut self) -> bool {
        self.runtime.force_stop().is_ok()
    }
}
