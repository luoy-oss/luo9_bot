// src/plugin/handle.rs
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::JoinHandle;
use libloading::Library;
use tracing::{info, warn};

use super::bus::Bus;

/// 插件运行时句柄，持有动态库引用、线程句柄和 subscriber ID
pub struct PluginHandle {
    pub name: String,
    pub lib: Arc<Library>,
    pub thread_handle: Option<JoinHandle<()>>,
    /// topic_name -> subscriber_id 的映射
    pub subscriber_ids: HashMap<String, usize>,
    pub priority: i32,
    pub block_enabled: bool,
    pub active: bool,
    pub path: PathBuf,
}

impl PluginHandle {
    /// 取消所有 topic 的订阅，触发插件线程退出
    pub fn unsubscribe_all(&self) {
        for (topic, &sub_id) in &self.subscriber_ids {
            if let Err(e) = Bus::topic(topic).unsubscribe(sub_id) {
                warn!("插件 {} 取消订阅 {} 失败: {:?}", self.name, topic, e);
            } else {
                info!("插件 {} 已取消订阅 {} (sub_id={})", self.name, topic, sub_id);
            }
        }
    }

    /// 检查插件线程是否仍在运行
    pub fn is_alive(&self) -> bool {
        match &self.thread_handle {
            Some(handle) => !handle.is_finished(),
            None => false,
        }
    }

    /// 等待插件线程退出（带超时）
    ///
    /// 返回 true 表示线程已退出，false 表示超时
    pub fn wait_exit(&mut self, timeout: std::time::Duration) -> bool {
        if let Some(handle) = self.thread_handle.take() {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = handle.join();
                let _ = tx.send(());
            });
            rx.recv_timeout(timeout).is_ok()
        } else {
            true
        }
    }
}
