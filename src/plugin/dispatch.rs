// src/plugin/dispatch.rs
use std::sync::RwLock;
use tracing::{error, info};

use super::bus::Bus;
use super::manager::DispatchEntry;
use crate::message::Message;
use crate::event::MetaEvent;
use crate::notice::Notice;
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

/// 优先级分发消息（使用 publish_to 定向推送）
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
        error!("[dispatch] 分发列表为空！消息被丢弃");
        return;
    }

    for entry in list.iter() {
        let Some(sub_id) = entry.message_sub_id else {
            info!("[dispatch] 跳过 {} (未订阅 luo9_message)", entry.name);
            continue;
        };
        match Bus::topic(super::bus::TOPIC_MESSAGE).publish_to(&payload, &[sub_id]) {
            Ok(()) => {
                info!("[dispatch] 已分发消息到 {} (sub_id={}, priority={})", entry.name, sub_id, entry.priority);
            }
            Err(e) => {
                error!("[dispatch] 定向分发消息到 {} 失败: {:?}", entry.name, e);
            }
        }
        if entry.block_enabled {
            info!("[dispatch] 插件 {} 启用了阻断，停止后续分发", entry.name);
            break;
        }
    }
}

/// 优先级分发通知
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
        return;
    }

    for entry in list.iter() {
        let Some(sub_id) = entry.notice_sub_id else { continue };
        if let Err(e) = Bus::topic(super::bus::TOPIC_NOTICE).publish_to(&payload, &[sub_id]) {
            error!("定向分发通知到 {} 失败: {:?}", entry.name, e);
        }
        if entry.block_enabled {
            break;
        }
    }
}

/// 优先级分发元事件
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
        return;
    }

    for entry in list.iter() {
        let Some(sub_id) = entry.meta_event_sub_id else { continue };
        if let Err(e) = Bus::topic(super::bus::TOPIC_META_EVENT).publish_to(&payload, &[sub_id]) {
            error!("定向分发元事件到 {} 失败: {:?}", entry.name, e);
        }
        if entry.block_enabled {
            break;
        }
    }
}
