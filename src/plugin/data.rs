// src/plugin/data.rs
// 插件数据类型定义

use serde::Serialize;
use crate::message::Message;
use crate::event::MetaEvent;
use crate::notice::Notice;

/// 插件数据类型枚举
/// 统一所有可能传递给插件的数据类型
#[derive(Debug, Clone, Serialize)]
pub enum PluginData {
    /// 消息类型
    Message(Message),
    /// 元事件类型
    MetaEvent(MetaEvent),
    /// 通知类型
    Notice(Notice),
}

impl PluginData {
    /// 获取数据类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            PluginData::Message(_) => "Message",
            PluginData::MetaEvent(_) => "MetaEvent",
            PluginData::Notice(_) => "Notice",
        }
    }
    
    /// 尝试转换为 Message 类型
    pub fn as_message(&self) -> Option<&Message> {
        match self {
            PluginData::Message(msg) => Some(msg),
            _ => None,
        }
    }
    
    /// 尝试转换为 MetaEvent 类型
    pub fn as_meta_event(&self) -> Option<&MetaEvent> {
        match self {
            PluginData::MetaEvent(event) => Some(event),
            _ => None,
        }
    }
    
    /// 尝试转换为 Notice 类型
    pub fn as_notice(&self) -> Option<&Notice> {
        match self {
            PluginData::Notice(notice) => Some(notice),
            _ => None,
        }
    }
}