use serde::{Serialize, Serializer};
use serde_json::Value;


/*
{
  "font": 14,
  "group_id": 736215193,
  "message": [
    {
      "data": {
        "text": "eeeeeee"
      },
      "type": "text"
    }
  ],
  "message_format": "array",
  "message_id": 1471815977,
  "message_seq": 1471815977,
  "message_type": "group",
  "post_type": "message",
  "raw_message": "eeeeeee",
  "real_id": 1471815977,
  "real_seq": "46182",
  "self_id": 512166443,
  "sender": {
    "card": "[QQ红包] 恭喜发财",
    "nickname": "洛",
    "role": "admin",
    "user_id": 2557657882
  },
  "sub_type": "normal",
  "time": 1774857824,
  "user_id": 2557657882
}
*/


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


#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub message_type: MsgType,
    pub user_id: u64,
    pub group_id: Option<u64>,
    pub message: String,
}

impl Message {
    pub fn new(data: Value) -> Self {
        let message_type = match data.get("message_type").and_then(|v| v.as_str()) {
            Some("private") => MsgType::Private,
            Some("group") => MsgType::Group,
            _ => MsgType::Other,
        };

        let user_id = data.get("user_id").and_then(|v| v.as_u64()).unwrap_or(0);

        let group_id = data.get("group_id").and_then(|v| v.as_u64());

        let message = data.get("raw_message").and_then(|v| v.as_str()).unwrap_or("").to_string();

        Self{
            message_type,
            user_id,
            group_id,
            message,
        }
    }
}