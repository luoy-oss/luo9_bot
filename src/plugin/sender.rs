// src/plugin/sender.rs
// 基于 bus 总线的消息发送接收器
// 从 luo9_send topic pop 请求，调用 Sender 实际发送，结果发到 luo9_response

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use super::bus::Bus;
use crate::connection::Sender;

const TOPIC_SEND: &str = "luo9_send";
const TOPIC_RESPONSE: &str = "luo9_response";
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

// ── 请求/响应结构（与 SDK 端一致）─────────────────────────────

#[derive(Debug, Deserialize)]
struct SendRequest {
    request_id: u64,
    action: SendAction,
}

#[derive(Debug, Deserialize)]
enum SendAction {
    #[serde(rename = "send_group_msg")]
    SendGroupMsg { group_id: u64, message: String },
    #[serde(rename = "send_private_msg")]
    SendPrivateMsg { user_id: u64, message: String },
}

#[derive(Debug, Serialize)]
struct SendResponse {
    request_id: u64,
    code: i64,
    error: Option<String>,
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
    tokio::spawn(async {
        let topic = Bus::topic(TOPIC_SEND);
        info!("消息发送接收器已启动，监听 topic: {}", TOPIC_SEND);

        loop {
            match topic.pop() {
                Some(json) => {
                    match serde_json::from_str::<SendRequest>(&json) {
                        Ok(req) => {
                            let response_topic = Bus::topic(TOPIC_RESPONSE);
                            let resp = handle_request(req).await;
                            if let Ok(resp_json) = serde_json::to_string(&resp) {
                                let _ = response_topic.publish(&resp_json);
                            }
                        }
                        Err(e) => {
                            error!("解析发送请求失败: {}", e);
                        }
                    }
                }
                None => {
                    tokio::time::sleep(POLL_INTERVAL).await;
                }
            }
        }
    });
}

async fn handle_request(req: SendRequest) -> SendResponse {
    let guard = GLOBAL_SENDER.lock().await;
    let Some(sender) = guard.as_ref() else {
        error!("Sender 未初始化，无法处理发送请求 #{}", req.request_id);
        return SendResponse {
            request_id: req.request_id,
            code: -1,
            error: Some("Sender not initialized".to_string()),
        };
    };

    let result = match &req.action {
        SendAction::SendGroupMsg { group_id, message } => {
            debug!("发送群消息: group_id={}, msg={}", group_id, message);
            sender.send_group_message(*group_id, message).await
        }
        SendAction::SendPrivateMsg { user_id, message } => {
            debug!("发送私聊消息: user_id={}, msg={}", user_id, message);
            sender.send_private_message(*user_id, message).await
        }
    };

    match result {
        Ok(_) => SendResponse {
            request_id: req.request_id,
            code: 0,
            error: None,
        },
        Err(e) => SendResponse {
            request_id: req.request_id,
            code: -1,
            error: Some(e.to_string()),
        },
    }
}
