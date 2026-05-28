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

/// 消息段类型（用于结构化消息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSegment {
    #[serde(rename = "type")]
    pub seg_type: String,
    pub data: Value,
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
    pub message: Vec<MessageSegment>,
    pub raw_message: String,
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

        let sub_type = SubType::deserialize(
            data.get("sub_type").unwrap_or(&Value::Null)
        ).unwrap_or(SubType::None);

        let message_id = data.get("message_id")
            .and_then(|v| v.as_u64())
            .or_else(|| data.get("message_id").and_then(|v| v.as_str().and_then(|s| s.parse().ok())))
            .unwrap_or(0);

        let message_seq = data.get("message_seq").and_then(|v| v.as_u64());
        let real_id = data.get("real_id").and_then(|v| v.as_u64());
        let real_seq = data.get("real_seq").and_then(|v| v.as_str()).map(|s| s.to_string());

        let user_id = data.get("user_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let group_id = data.get("group_id").and_then(|v| v.as_u64());

        // 解析消息段数组
        let message = match data.get("message") {
            Some(Value::Array(arr)) => {
                arr.iter()
                    .filter_map(|seg| serde_json::from_value::<MessageSegment>(seg.clone()).ok())
                    .collect()
            }
            Some(Value::String(s)) => {
                // 兼容字符串格式的消息
                vec![MessageSegment {
                    seg_type: "text".to_string(),
                    data: serde_json::json!({"text": s}),
                }]
            }
            _ => vec![],
        };

        let raw_message = data.get("raw_message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let raw_message = decode_html_entities(&raw_message);

        let font = data.get("font").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let sender: Sender = data.get("sender")
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

        let anonymous = data.get("anonymous")
            .and_then(|v| if v.is_null() { None } else { serde_json::from_value(v.clone()).ok() });

        let message_format = data.get("message_format")
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
            message,
            raw_message,
            font,
            sender,
            anonymous,
            message_format,
        }
    }

    /// 获取纯文本消息内容（兼容旧接口）
    pub fn get_text(&self) -> &str {
        &self.raw_message
    }
}