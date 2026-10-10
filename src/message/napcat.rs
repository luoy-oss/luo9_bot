use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::sub_type::SubType;

/// 消息发送者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    pub user_id: u64,
    pub nickname: String,
    #[serde(default)]
    pub card: String,
    #[serde(default)]
    pub sex: String,
    #[serde(default)]
    pub age: u32,
    #[serde(default)]
    pub area: String,
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub title: String,
}

/// 匿名信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anonymous {
    pub id: u64,
    pub name: String,
    pub flag: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MsgType {
    Private,
    Group,
    Other,
}

impl Serialize for MsgType {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            MsgType::Private => serializer.serialize_str("private"),
            MsgType::Group => serializer.serialize_str("group"),
            MsgType::Other => serializer.serialize_str("other"),
        }
    }
}

/// 完整的 OneBot v11 消息结构
///
/// `message` 字段为纯文本消息（raw_message），保持与 SDK 的兼容性。
#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub time: u64,
    pub self_id: u64,
    pub post_type: String,
    pub message_type: MsgType,
    pub sub_type: SubType,
    pub message_id: u64,
    pub message_seq: Option<u64>,
    pub real_id: Option<u64>,
    pub real_seq: Option<String>,
    pub user_id: u64,
    pub group_id: Option<u64>,
    /// 兼容旧插件的纯文本消息内容（解码后的 raw_message）。
    pub message: String,
    /// NapCat 原始 raw_message，保留协议层内容。
    pub raw_message: String,
    /// 结构化消息段。保留 text、image、video、file、json、forward 等任意段类型及未知字段。
    pub message_segments: Vec<Value>,
    /// 完整原始事件对象，用于向前兼容尚未建模的 NapCat/OneBot 字段。
    pub raw_event: Value,
    pub font: u32,
    pub sender: Sender,
    pub anonymous: Option<Anonymous>,
    pub message_format: String,
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

impl Message {
    pub fn new(data: Value) -> Self {
        let message_type = match data.get("message_type").and_then(|v| v.as_str()) {
            Some("private") => MsgType::Private,
            Some("group") => MsgType::Group,
            _ => MsgType::Other,
        };

        let sub_type = SubType::deserialize(data.get("sub_type").unwrap_or(&Value::Null))
            .unwrap_or(SubType::None);

        let message_id = data
            .get("message_id")
            .and_then(|v| v.as_u64())
            .or_else(|| {
                data.get("message_id")
                    .and_then(|v| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0);

        let message_seq = data.get("message_seq").and_then(|v| v.as_u64());
        let real_id = data.get("real_id").and_then(|v| v.as_u64());
        let real_seq = data
            .get("real_seq")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let user_id = data.get("user_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let group_id = data.get("group_id").and_then(|v| v.as_u64());

        let raw_message = data
            .get("raw_message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let message = decode_html_entities(&raw_message);
        let message_segments = data
            .get("message")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let raw_event = data.clone();

        let font = data.get("font").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let sender: Sender = data
            .get("sender")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(Sender {
                user_id,
                nickname: String::new(),
                card: String::new(),
                sex: "unknown".to_string(),
                age: 0,
                area: String::new(),
                level: String::new(),
                role: "member".to_string(),
                title: String::new(),
            });

        let anonymous = data.get("anonymous").and_then(|v| {
            if v.is_null() {
                None
            } else {
                serde_json::from_value(v.clone()).ok()
            }
        });

        let message_format = data
            .get("message_format")
            .and_then(|v| v.as_str())
            .unwrap_or("array")
            .to_string();

        let time = data.get("time").and_then(|v| v.as_u64()).unwrap_or(0);
        let self_id = data.get("self_id").and_then(|v| v.as_u64()).unwrap_or(0);

        Self {
            time,
            self_id,
            post_type: "message".to_string(),
            message_type,
            sub_type,
            message_id,
            message_seq,
            real_id,
            real_seq,
            user_id,
            group_id,
            message, // 保持旧字段语义，避免破坏现有插件
            raw_message,
            message_segments,
            raw_event,
            font,
            sender,
            anonymous,
            message_format,
        }
    }

    /// 获取纯文本消息内容
    pub fn get_text(&self) -> &str {
        &self.message
    }
}


#[cfg(test)]
mod tests {
    use super::Message;
    use serde_json::json;

    #[test]
    fn preserves_structured_segments_and_unknown_event_fields() {
        let event = json!({
            "time": 1,
            "self_id": 2,
            "post_type": "message",
            "message_type": "group",
            "message_id": "123",
            "user_id": 4,
            "group_id": 5,
            "raw_message": "[CQ:video,file=clip.mp4]",
            "message": [
                {"type": "text", "data": {"text": "hello"}},
                {"type": "video", "data": {"file": "clip.mp4", "extra": {"codec": "h265"}}},
                {"type": "future_segment", "data": {"opaque": [1, 2, 3]}}
            ],
            "future_event_field": {"nested": true}
        });
        let message = Message::new(event.clone());

        assert_eq!(message.message_segments.len(), 3);
        assert_eq!(message.message_segments[1]["type"], "video");
        assert_eq!(message.message_segments[1]["data"]["extra"]["codec"], "h265");
        assert_eq!(message.message_segments[2]["type"], "future_segment");
        assert_eq!(message.raw_message, "[CQ:video,file=clip.mp4]");
        assert_eq!(message.raw_event["future_event_field"]["nested"], true);
        assert_eq!(message.message, "[CQ:video,file=clip.mp4]");
    }

    #[test]
    fn handles_string_message_id_and_missing_segments() {
        let message = Message::new(json!({
            "message_id": "42",
            "user_id": 9,
            "raw_message": "hello &amp; goodbye"
        }));
        assert_eq!(message.message_id, 42);
        assert!(message.message_segments.is_empty());
        assert_eq!(message.message, "hello & goodbye");
        assert_eq!(message.raw_message, "hello &amp; goodbye");
    }
}
