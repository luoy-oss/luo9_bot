// src/plugin/bus.rs

use tracing::{debug, error, info};
pub use luo9_sdk::bus::{
    Bus, BusError
};

use super::data::PluginData;

pub const TOPIC_MESSAGE: &str = "luo9_message";
pub const TOPIC_META_EVENT: &str = "luo9_meta_event";
pub const TOPIC_NOTICE: &str = "luo9_notice";
pub const TOPIC_TASK_MISO: &str = "luo9_task_miso";
pub const TOPIC_TASK: &str = "luo9_task";
pub const TOPIC_SEND: &str = "luo9_send";

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

/// 启动一个 topic 接收循环，在独立 tokio task 中运行。
///
/// 内部使用 `spawn_blocking` + `wait_pop` 阻塞等待消息，不占用 tokio worker 线程。
/// 收到消息后调用 `handler` 回调处理。handler 可以是 async 函数。
pub fn start_topic_receiver<F, Fut>(topic_name: &'static str, handler: F)
where
    F: Fn(String) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    tokio::spawn(async move {
        let sub_id = Bus::topic(topic_name)
            .subscribe()
            .unwrap_or_else(|e| panic!("订阅 {} topic 失败: {:?}", topic_name, e));
        info!("bus 接收器已启动: topic={}, subscriber_id={}", topic_name, sub_id);

        loop {
            let result = tokio::task::spawn_blocking({
                move || Bus::topic(topic_name).wait_pop(sub_id)
            })
            .await
            .expect("spawn_blocking panicked");

            match result {
                Ok(json) => handler(json).await,
                Err(e) => error!("bus wait_pop 失败: {:?}", e),
            }
        }
    });
}
