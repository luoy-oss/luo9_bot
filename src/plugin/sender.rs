// src/plugin/sender.rs
// 基于 bus 总线的消息发送接收器
// 从 luo9_send topic 阻塞等待请求，调用 Sender 实际发送；
// 发送成功后把 NapCat 返回的 message_id 发布到 luo9_sent 回执 topic。

use serde::Deserialize;
use serde_json::Value;
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
    #[serde(rename = "delete_msg")]
    DeleteMsg { message_id: u64 },
    #[serde(rename = "set_msg_emoji_like")]
    SetMsgEmojiLike { message_id: u64, emoji_id: u64 },
}

// ── 全局 Sender ────────────────────────────────────────────────

static GLOBAL_SENDER: Mutex<Option<Sender>> = Mutex::const_new(None);

pub async fn init_sender(sender: Sender) {
    let mut guard = GLOBAL_SENDER.lock().await;
    *guard = Some(sender);
    info!("bus 消息发送器已初始化");
}

pub async fn clear_sender() {
    let mut guard = GLOBAL_SENDER.lock().await;
    *guard = None;
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
            match sender.send_group_message(*group_id, message).await {
                Ok(resp) => {
                    publish_sent_echo(Some(*group_id), 0, message, &resp);
                }
                Err(e) => error!("群消息发送失败: {}", e),
            }
        }
        SendAction::SendPrivateMsg { user_id, message } => {
            debug!("发送私聊消息: user_id={}, msg={}", user_id, message);
            match sender.send_private_message(*user_id, message).await {
                Ok(resp) => {
                    publish_sent_echo(None, *user_id, message, &resp);
                }
                Err(e) => error!("私聊消息发送失败: {}", e),
            }
        }
        SendAction::DeleteMsg { message_id } => {
            debug!("撤回消息: message_id={}", message_id);
            if let Err(e) = sender.delete_msg(*message_id).await {
                error!("撤回消息失败: message_id={}, {}", message_id, e);
            }
        }
        SendAction::SetMsgEmojiLike {
            message_id,
            emoji_id,
        } => {
            debug!(
                "设置表情回应: message_id={}, emoji_id={}",
                message_id, emoji_id
            );
            if let Err(e) = sender.set_msg_emoji_like(*message_id, *emoji_id).await {
                error!(
                    "表情回应失败: message_id={}, emoji_id={}, {}",
                    message_id, emoji_id, e
                );
            }
        }
    }
}

// ── 发送回执 ───────────────────────────────────────────────────

/// 从 NapCat 响应中提取 message_id（data.message_id，兼容数字与字符串）
fn extract_message_id(resp: &Value) -> Option<u64> {
    let data = resp.get("data")?;
    data.get("message_id").and_then(|v| {
        v.as_u64()
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    })
}

/// 发布发送回执到 luo9_sent（外层标签格式与 SDK BusPayload::Sent 对应）
fn publish_sent_echo(group_id: Option<u64>, user_id: u64, message: &str, resp: &Value) {
    let Some(message_id) = extract_message_id(resp) else {
        debug!("发送响应中未包含 message_id，跳过回执");
        return;
    };

    let payload = serde_json::json!({
        "Sent": {
            "group_id": group_id,
            "user_id": user_id,
            "message_id": message_id,
            "message": message,
            "time": chrono::Utc::now().timestamp().max(0) as u64,
        }
    });

    match serde_json::to_string(&payload) {
        Ok(json) => {
            if let Err(e) = bus::Bus::topic(bus::TOPIC_SENT).publish(&json) {
                error!("发布发送回执失败: {:?}", e);
            }
        }
        Err(e) => error!("序列化发送回执失败: {}", e),
    }
}
