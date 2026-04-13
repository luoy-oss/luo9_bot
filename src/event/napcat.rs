use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::sub_type::SubType;

#[derive(Debug, Clone)]
pub enum MetaEventType {
    Lifecycle,
    Heartbeat,
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Status{
    pub good: bool,
    pub online: bool,
}

#[derive(Debug, Clone)]
pub struct MetaEvent {
    pub interval: Option<u64>,
    pub meta_event_type: MetaEventType,
    pub sub_type: SubType,
    pub self_id: u64,
    pub status: Option<Status>,
    pub time: u64,
}


impl MetaEvent {
    pub fn new(data: Value) -> Self {
        let meta_event_type = match data.get("meta_event_type").and_then(|v| v.as_str()) {
            Some("lifecycle") => MetaEventType::Lifecycle,
            Some("heartbeat") => MetaEventType::Heartbeat,
            _ => MetaEventType::Unknown,
        };
        
        
        let sub_type = SubType::deserialize(
                data.get("sub_type")
                                    .unwrap_or(&Value::Null))
                                    .unwrap_or(SubType::None);

        Self{
            interval: data.get("interval").and_then(|v| v.as_u64()).map(|v| v),
            meta_event_type,
            sub_type,
            self_id: data.get("self_id").and_then(|v| v.as_u64()).unwrap_or(0),
            status: data.get("status").map(|v| serde_json::from_value(v.clone()).unwrap()),
            time: data.get("time").and_then(|v| v.as_u64()).unwrap_or(0),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostType {
    MetaEvent,
    Message,
    MessageSent,
    Request,
    Notice,

    Other(String),
}

impl<'de> Deserialize<'de> for PostType {
    fn deserialize<D>(deserializer: D) -> Result<PostType, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let post_type = match s.as_str() {
            "message" => PostType::Message,
            "message_sent" => PostType::MessageSent,
            "meta_event" => PostType::MetaEvent,
            "request" => PostType::Request,
            "notice" => PostType::Notice,
            _ => PostType::Other(s),
        };
        Ok(post_type)
    }
}