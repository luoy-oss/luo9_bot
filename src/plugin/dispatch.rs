// src/plugin/dispatch.rs
use std::sync::RwLock;
use tracing::{error, info};

use super::bus::Bus;
use super::manager::{DispatchEntry, GLOBAL_PLUGIN_MANAGER};
use crate::message::Message;
use crate::event::MetaEvent;
use crate::notice::Notice;
use crate::request::Request;
use super::data::PluginData;

/// 优先级分发列表（无锁快速路径读取）
static DISPATCH_LIST: RwLock<Vec<DispatchEntry>> = RwLock::new(Vec::new());

/// 更新分发列表（插件启用/禁用/配置变更时调用）
pub fn update_dispatch_list(entries: Vec<DispatchEntry>) {
    match DISPATCH_LIST.write() {
        Ok(mut list) => {
            info!("[dispatch] 分发列表更新，共 {} 个活跃插件:", entries.len());
            for e in &entries {
                let msg = e.message_sub_id.map(|v| v.to_string()).unwrap_or_else(|| "无".into());
                let notice = e.notice_sub_id.map(|v| v.to_string()).unwrap_or_else(|| "无".into());
                let meta = e.meta_event_sub_id.map(|v| v.to_string()).unwrap_or_else(|| "无".into());
                info!("  - {} (priority={}, block={}, msg_sub={}, notice_sub={}, meta_sub={})",
                    e.name, e.priority, e.block_enabled, msg, notice, meta);
            }
            *list = entries;
        }
        Err(e) => {
            error!("更新分发列表失败: {}", e);
        }
    }
}

/// 优先级分发消息（使用 publish 广播，确保向后兼容）
pub fn priority_dispatch_message(msg: Message) {
    let payload = match serde_json::to_string(&PluginData::Message(msg)) {
        Ok(json) => json,
        Err(e) => {
            error!("序列化消息失败: {}", e);
            return;
        }
    };

    let list = match DISPATCH_LIST.read() {
        Ok(list) => list,
        Err(e) => {
            error!("读取分发列表失败: {}", e);
            return;
        }
    };

    if list.is_empty() {
        // 没有插件注册，直接广播
        let start = std::time::Instant::now();
        if let Err(e) = Bus::topic(super::bus::TOPIC_MESSAGE).publish(&payload) {
            error!("广播消息失败: {:?}", e);
        } else {
            let elapsed = start.elapsed().as_micros() as u64;
            info!("[dispatch] 已广播消息 (耗时={}μs)", elapsed);
        }
        return;
    }

    // 使用广播模式，确保所有 subscriber 都能收到消息
    // 这解决了插件未使用预分配 subscriber ID 的兼容性问题
    let start = std::time::Instant::now();
    if let Err(e) = Bus::topic(super::bus::TOPIC_MESSAGE).publish(&payload) {
        error!("广播消息失败: {:?}", e);
        return;
    }
    let elapsed = start.elapsed().as_micros() as u64;

    // 更新所有活跃插件的统计
    if let Ok(mut manager) = GLOBAL_PLUGIN_MANAGER.try_lock() {
        for entry in list.iter() {
            if entry.message_sub_id.is_some() {
                manager.update_message_stats(&entry.name, elapsed);
            }
        }
    }

    info!("[dispatch] 已广播消息到 {} 个插件 (耗时={}μs)", list.len(), elapsed);
}

/// 优先级分发通知（使用广播模式）
pub fn priority_dispatch_notice(notice: Notice) {
    let payload = match serde_json::to_string(&PluginData::Notice(notice)) {
        Ok(json) => json,
        Err(e) => {
            error!("序列化通知失败: {}", e);
            return;
        }
    };

    let list = match DISPATCH_LIST.read() {
        Ok(list) => list,
        Err(e) => {
            error!("读取分发列表失败: {}", e);
            return;
        }
    };

    if list.is_empty() {
        if let Err(e) = Bus::topic(super::bus::TOPIC_NOTICE).publish(&payload) {
            error!("广播通知失败: {:?}", e);
        }
        return;
    }

    let start = std::time::Instant::now();
    if let Err(e) = Bus::topic(super::bus::TOPIC_NOTICE).publish(&payload) {
        error!("广播通知失败: {:?}", e);
        return;
    }
    let elapsed = start.elapsed().as_micros() as u64;

    if let Ok(mut manager) = GLOBAL_PLUGIN_MANAGER.try_lock() {
        for entry in list.iter() {
            if entry.notice_sub_id.is_some() {
                manager.update_notice_stats(&entry.name, elapsed);
            }
        }
    }
}

/// 优先级分发元事件（使用广播模式）
pub fn priority_dispatch_meta_event(event: MetaEvent) {
    let payload = match serde_json::to_string(&PluginData::MetaEvent(event)) {
        Ok(json) => json,
        Err(e) => {
            error!("序列化元事件失败: {}", e);
            return;
        }
    };

    let list = match DISPATCH_LIST.read() {
        Ok(list) => list,
        Err(e) => {
            error!("读取分发列表失败: {}", e);
            return;
        }
    };

    if list.is_empty() {
        if let Err(e) = Bus::topic(super::bus::TOPIC_META_EVENT).publish(&payload) {
            error!("广播元事件失败: {:?}", e);
        }
        return;
    }

    let start = std::time::Instant::now();
    if let Err(e) = Bus::topic(super::bus::TOPIC_META_EVENT).publish(&payload) {
        error!("广播元事件失败: {:?}", e);
        return;
    }
    let elapsed = start.elapsed().as_micros() as u64;

    if let Ok(mut manager) = GLOBAL_PLUGIN_MANAGER.try_lock() {
        for entry in list.iter() {
            if entry.meta_event_sub_id.is_some() {
                manager.update_meta_event_stats(&entry.name, elapsed);
            }
        }
    }
}

/// 优先级分发请求事件（使用广播模式）
pub fn priority_dispatch_request(request: Request) {
    let payload = match serde_json::to_string(&PluginData::Request(request)) {
        Ok(json) => json,
        Err(e) => {
            error!("序列化请求失败: {}", e);
            return;
        }
    };

    let list = match DISPATCH_LIST.read() {
        Ok(list) => list,
        Err(e) => {
            error!("读取分发列表失败: {}", e);
            return;
        }
    };

    if list.is_empty() {
        if let Err(e) = Bus::topic(super::bus::TOPIC_REQUEST).publish(&payload) {
            error!("广播请求失败: {:?}", e);
        }
        return;
    }

    let start = std::time::Instant::now();
    if let Err(e) = Bus::topic(super::bus::TOPIC_REQUEST).publish(&payload) {
        error!("广播请求失败: {:?}", e);
        return;
    }
    let elapsed = start.elapsed().as_micros() as u64;

    if let Ok(mut manager) = GLOBAL_PLUGIN_MANAGER.try_lock() {
        for entry in list.iter() {
            if entry.request_sub_id.is_some() {
                manager.update_request_stats(&entry.name, elapsed);
            }
        }
    }
}
