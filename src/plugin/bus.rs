// src/plugin/bus.rs

use tracing::{debug, error};
pub use luo9_sdk::bus::{
    Bus, BusError
};

use super::data::PluginData;

pub const TOPIC_MESSAGE: &str = "luo9_message";
pub const TOPIC_META_EVENT: &str = "luo9_meta_event";
pub const TOPIC_NOTICE: &str = "luo9_notice";
pub const TOPIC_TASK: &str = "luo9_task";

pub fn publish_data(data: &PluginData) {
    let topic_name = match data {
        PluginData::Message(_) => TOPIC_MESSAGE,
        PluginData::MetaEvent(_) => TOPIC_META_EVENT,
        PluginData::Notice(_) => TOPIC_NOTICE,
    };

    let payload = match serde_json::to_string(data) {
        Ok(json) => json,
        Err(e) => {
            error!("序列化 PluginData 失败: {}", e);
            return;
        }
    };

    debug!("消息推送内容: {}", payload);
    let topic = Bus::topic(topic_name);
    match topic.publish(&payload) {
        Ok(()) => debug!("已发布到 topic: {}", topic_name),
        Err(e) => error!("发布到 topic {} 失败: {:?}", topic_name, e),
    }
}
