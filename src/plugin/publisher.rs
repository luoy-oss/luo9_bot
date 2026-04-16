// src/plugin/publisher.rs
use tracing::{debug, error};
use super::bus;
use crate::message::Message;
use crate::event::MetaEvent;
use crate::notice::Notice;
use super::data::PluginData;

/// 消息发布器
pub struct PluginMessagePublisher;

impl PluginMessagePublisher {
    /// 发布消息
    pub fn publish_message(msg: Message) {
        let plugin_data = PluginData::Message(msg);
        Self::publish_data(plugin_data, "消息");
    }
    
    /// 发布元事件
    pub fn publish_meta_event(event: MetaEvent) {
        let plugin_data = PluginData::MetaEvent(event);
        Self::publish_data(plugin_data, "元事件");
    }
    
    /// 发布通知
    pub fn publish_notice(notice: Notice) {
        let plugin_data = PluginData::Notice(notice);
        Self::publish_data(plugin_data, "通知");
    }
    
    fn publish_data(data: PluginData, data_type_name: &str) {
        match bus::publish(data) {
            Ok(receivers) => {
                debug!("{}已发布，{} 个插件将接收", data_type_name, receivers);
            }
            Err(e) => {
                error!("发布{}失败: {:?}", data_type_name, e);
            }
        }
    }
    
    // pub fn has_subscribers() -> bool {
    //     bus::is_active()
    // }
}