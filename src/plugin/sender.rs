// src/plugin/sender.rs
// 基于 bus 总线的消息发送接收器
// 从 luo9_send topic 阻塞等待请求，调用 Sender 实际发送

use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use super::bus;
use crate::connection::Sender;

// ── 请求结构（与 SDK 端一致）──────────────────────────────────

#[derive(Debug, Deserialize)]
struct SendRequest {
    action: SendAction,
}

#[derive(Debug, Deserialize)]
enum SendAction {
    #[serde(rename = "send_group_msg")]
    SendGroupMsg { group_id: u64, message: String },
    #[serde(rename = "send_private_msg")]
    SendPrivateMsg { user_id: u64, message: String },
}

// ── 全局 Sender ────────────────────────────────────────────────

static GLOBAL_SENDER: Mutex<Option<Sender>> = Mutex::const_new(None);

pub async fn init_sender(sender: Sender) {
    let mut guard = GLOBAL_SENDER.lock().await;
    *guard = Some(sender);
    info!("bus 消息发送器已初始化");
}

// ── 接收器 ─────────────────────────────────────────────────────

pub fn start_send_receiver() {
    bus::start_topic_receiver(bus::TOPIC_SEND, |json| async move {
        match serde_json::from_str::<SendRequest>(&json) {
            Ok(req) => handle_request(req).await,
            Err(e) => error!("解析发送请求失败: {}", e),
        }
    });
}

async fn handle_request(req: SendRequest) {
    let guard = GLOBAL_SENDER.lock().await;
    let Some(sender) = guard.as_ref() else {
        error!("Sender 未初始化，无法处理发送请求");
        return;
    };

    match &req.action {
        SendAction::SendGroupMsg { group_id, message } => {
            debug!("发送群消息: group_id={}, msg={}", group_id, message);
            if let Err(e) = sender.send_group_message(*group_id, message).await {
                error!("群消息发送失败: {}", e);
            }
        }
        SendAction::SendPrivateMsg { user_id, message } => {
            debug!("发送私聊消息: user_id={}, msg={}", user_id, message);
            if let Err(e) = sender.send_private_message(*user_id, message).await {
                error!("私聊消息发送失败: {}", e);
            }
        }
    }
}
