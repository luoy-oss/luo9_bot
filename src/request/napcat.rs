use serde::{Serialize, Serializer};
use serde_json::Value;

/// 请求类型
#[derive(Debug, Clone, PartialEq)]
pub enum RequestType {
    Friend, // 好友请求
    Group,  // 群请求
    Unknown,
}

impl Serialize for RequestType {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            RequestType::Friend => serializer.serialize_str("friend"),
            RequestType::Group => serializer.serialize_str("group"),
            RequestType::Unknown => serializer.serialize_str("unknown"),
        }
    }
}

/// 群请求子类型
#[derive(Debug, Clone, PartialEq)]
pub enum GroupRequestSubType {
    Add,    // 加群请求
    Invite, // 邀请入群
    Unknown,
}

impl Serialize for GroupRequestSubType {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            GroupRequestSubType::Add => serializer.serialize_str("add"),
            GroupRequestSubType::Invite => serializer.serialize_str("invite"),
            GroupRequestSubType::Unknown => serializer.serialize_str("unknown"),
        }
    }
}

/// 完整的 OneBot v11 请求事件结构
#[derive(Debug, Serialize, Clone)]
pub struct Request {
    pub time: u64,
    pub self_id: u64,
    pub post_type: String,
    pub request_type: RequestType,
    pub sub_type: GroupRequestSubType,
    pub user_id: u64,
    pub group_id: Option<u64>,
    pub comment: String,
    pub flag: String,
}

impl Request {
    pub fn new(data: Value) -> Self {
        let request_type = match data.get("request_type").and_then(|v| v.as_str()) {
            Some("friend") => RequestType::Friend,
            Some("group") => RequestType::Group,
            _ => RequestType::Unknown,
        };

        let sub_type = match data.get("sub_type").and_then(|v| v.as_str()) {
            Some("add") => GroupRequestSubType::Add,
            Some("invite") => GroupRequestSubType::Invite,
            _ => GroupRequestSubType::Unknown,
        };

        let user_id = data.get("user_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let group_id = data.get("group_id").and_then(|v| v.as_u64());

        let comment = data
            .get("comment")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let flag = data
            .get("flag")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let time = data.get("time").and_then(|v| v.as_u64()).unwrap_or(0);
        let self_id = data.get("self_id").and_then(|v| v.as_u64()).unwrap_or(0);

        Self {
            time,
            self_id,
            post_type: "request".to_string(),
            request_type,
            sub_type,
            user_id,
            group_id,
            comment,
            flag,
        }
    }
}
